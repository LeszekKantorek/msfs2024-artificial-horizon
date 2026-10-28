//! Embedded web assets, HTTP liveness, listener ownership and graceful shutdown.

use std::{future::Future, io, net::SocketAddr};

use axum::{Json, Router, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::telemetry::Subscription;
use crate::{Config, ConfigError};

mod assets;
mod connection;
mod sse;

pub struct Server {
    listener: TcpListener,
    subscription: Subscription,
}

// Each owner signals shutdown on drop, including cancellation of Server::run.
// Clones share the first deadline; repeated shutdown never extends the grace period.
#[derive(Clone)]
struct ShutdownGuard {
    closing: tokio::sync::watch::Sender<Option<tokio::time::Instant>>,
}

impl ShutdownGuard {
    fn signal(&self) {
        self.closing.send_if_modified(|started| {
            if started.is_some() {
                return false;
            }
            *started = Some(tokio::time::Instant::now());
            true
        });
    }
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        self.signal();
    }
}

impl Server {
    pub async fn bind(config: Config, subscription: Subscription) -> Result<Self, ServerError> {
        config.validate()?;
        let address = config.socket_address();
        let listener = TcpListener::bind(address)
            .await
            .map_err(|source| ServerError::Bind { address, source })?;
        Ok(Self {
            listener,
            subscription,
        })
    }

    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Stop accepting requests on shutdown and finish requests already in flight.
    pub async fn run(self, shutdown: impl Future<Output = ()> + Send + 'static) -> io::Result<()> {
        let (closing, closed) = tokio::sync::watch::channel(None);
        let shutdown_guard = ShutdownGuard { closing };
        let stop_on_shutdown = shutdown_guard.clone();
        let listener = connection::DeadlineListener::new(self.listener, closed);
        let http_result = axum::serve(listener, router_with_telemetry(self.subscription))
            .with_graceful_shutdown(async move {
                shutdown.await;
                stop_on_shutdown.signal();
            })
            .await;
        shutdown_guard.signal();
        http_result
    }
}

pub fn router() -> Router {
    assets::router().route("/health", get(|| async { Json(Health { status: "ok" }) }))
}

/// Serve one shared telemetry source; each request owns a latest-value subscription.
pub fn router_with_telemetry(subscription: Subscription) -> Router {
    router().merge(
        Router::new()
            .route("/api/v1/events", get(sse::events))
            .with_state(subscription),
    )
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("cannot listen on {address}: {source}; choose an available local address and port")]
    Bind {
        address: SocketAddr,
        #[source]
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{self, State};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        sync::oneshot,
        time::{Duration, timeout},
    };

    #[tokio::test(start_paused = true)]
    async fn repeated_shutdown_preserves_the_first_deadline() {
        let (closing, closed) = tokio::sync::watch::channel(None);
        let guard = ShutdownGuard { closing };
        let other = guard.clone();
        guard.signal();
        let first = closed.borrow().unwrap();
        tokio::time::advance(Duration::from_secs(2)).await;
        drop(other);
        guard.signal();
        assert_eq!(*closed.borrow(), Some(first));
    }

    async fn check_cancelled_server(abort: bool) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (mut publisher, subscription) = telemetry::channel(crate::Source::Demo);
        let mut telemetry = subscription.clone();
        let server = Server {
            listener,
            subscription,
        };
        let (drop_run, dropped) = oneshot::channel();
        let task = tokio::spawn(async move {
            tokio::select! {
                result = server.run(std::future::pending()) => result,
                _ = dropped => Ok(()),
            }
        });
        let mut clients = Vec::new();
        for _ in 0..2 {
            let mut client = TcpStream::connect(address).await.unwrap();
            client
                .write_all(b"GET /api/v1/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();
            timeout(Duration::from_secs(5), async {
                let mut received = String::new();
                while !received.contains("event: telemetry") {
                    let mut bytes = [0; 4096];
                    let count = client.read(&mut bytes).await.unwrap();
                    assert!(count > 0);
                    received.push_str(&String::from_utf8_lossy(&bytes[..count]));
                }
            })
            .await
            .unwrap();
            clients.push(client);
        }
        publisher.publish(State::Paused).unwrap();
        timeout(Duration::from_secs(5), telemetry.changed())
            .await
            .unwrap()
            .unwrap();
        if abort {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            drop_run.send(()).unwrap();
            task.await.unwrap().unwrap();
        }
        // HTTP cancellation must not stop or own externally supplied publication.
        publisher.publish(State::Disconnected).unwrap();
        let snapshot = telemetry.changed().await.unwrap();
        assert_eq!(
            serde_json::to_value(snapshot).unwrap()["state"],
            "disconnected"
        );
        for mut client in clients {
            timeout(Duration::from_secs(7), client.read_to_end(&mut Vec::new()))
                .await
                .unwrap()
                .unwrap();
        }
        let rebound = TcpListener::bind(address).await.unwrap();
        assert_eq!(rebound.local_addr().unwrap(), address);
    }

    #[tokio::test]
    async fn aborting_server_closes_clients_without_stopping_publication() {
        check_cancelled_server(true).await;
    }

    #[tokio::test]
    async fn dropping_polled_server_closes_clients_without_stopping_publication() {
        check_cancelled_server(false).await;
    }

    #[tokio::test]
    async fn shutdown_closes_active_sse_and_releases_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (_publisher, subscription) = telemetry::channel(crate::Source::Demo);
        let server = Server {
            listener,
            subscription,
        };
        let (stop, stopped) = oneshot::channel();
        let task = tokio::spawn(server.run(async {
            let _ = stopped.await;
        }));
        let mut clients = Vec::new();
        for _ in 0..2 {
            let mut client = TcpStream::connect(address).await.unwrap();
            client
                .write_all(b"GET /api/v1/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();
            let mut received = Vec::new();
            timeout(Duration::from_secs(5), async {
                while !String::from_utf8_lossy(&received).contains("event: telemetry") {
                    let mut bytes = [0; 4096];
                    let count = client.read(&mut bytes).await.unwrap();
                    assert!(count > 0);
                    received.extend_from_slice(&bytes[..count]);
                }
            })
            .await
            .unwrap();
            assert!(String::from_utf8_lossy(&received).starts_with("HTTP/1.1 200 OK"));
            clients.push(client);
        }
        stop.send(()).unwrap();
        timeout(Duration::from_secs(7), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        for mut client in clients {
            let mut rest = Vec::new();
            timeout(Duration::from_secs(1), client.read_to_end(&mut rest))
                .await
                .unwrap()
                .unwrap();
        }
        let rebound = TcpListener::bind(address).await.unwrap();
        assert_eq!(rebound.local_addr().unwrap(), address);
    }

    #[tokio::test]
    async fn shutdown_releases_the_listening_socket() {
        // Port zero is confined to the listener-owning test, not public configuration.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (mut publisher, subscription) = telemetry::channel(crate::Source::Demo);
        let mut telemetry = subscription.clone();
        let server = Server {
            listener,
            subscription,
        };
        let (stop, stopped) = oneshot::channel();
        let task = tokio::spawn(server.run(async {
            let _ = stopped.await;
        }));
        let response = timeout(Duration::from_secs(5), async {
            let mut connection = TcpStream::connect(address).await.unwrap();
            connection
                .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            let mut response = String::new();
            connection.read_to_string(&mut response).await.unwrap();
            response
        })
        .await
        .unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("{\"status\":\"ok\"}"));
        publisher.publish(State::Paused).unwrap();
        let sample = timeout(Duration::from_secs(5), telemetry.changed())
            .await
            .unwrap()
            .unwrap();
        assert!(
            serde_json::to_value(sample).unwrap()["sequence"]
                .as_u64()
                .unwrap()
                > 0
        );
        stop.send(()).unwrap();
        timeout(Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        publisher.publish(State::Disconnected).unwrap();
        assert!(telemetry.changed().await.is_ok());
        let rebound = TcpListener::bind(address).await.unwrap();
        assert_eq!(rebound.local_addr().unwrap(), address);
    }

    #[tokio::test]
    async fn bind_reports_occupied_port_and_rejects_invalid_config() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let config = Config {
            port: address.port(),
            ..Config::default()
        };
        let (_publisher, subscription) = telemetry::channel(crate::Source::SimConnect);
        let error = Server::bind(config, subscription.clone())
            .await
            .err()
            .unwrap();
        assert!(matches!(error, ServerError::Bind { .. }));
        assert!(error.to_string().contains(&address.to_string()));
        assert!(matches!(
            Server::bind(
                Config {
                    port: 0,
                    ..Config::default()
                },
                subscription
            )
            .await
            .err()
            .unwrap(),
            ServerError::Config(ConfigError::InvalidPort)
        ));
    }
}
