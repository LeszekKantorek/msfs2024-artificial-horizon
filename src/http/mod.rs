//! Embedded web assets, HTTP liveness, listener ownership and graceful shutdown.

use std::{future::Future, io, net::SocketAddr};

use axum::{Json, Router, http::header, response::Html, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::telemetry::{self, Publisher, Subscription};
use crate::{Config, ConfigError};

mod connection;
mod sse;

pub struct Server {
    listener: TcpListener,
    publisher: Publisher,
    subscription: Subscription,
}

impl Server {
    pub async fn bind(config: Config) -> Result<Self, ServerError> {
        config.validate()?;
        let address = config.socket_address();
        let listener = TcpListener::bind(address)
            .await
            .map_err(|source| ServerError::Bind { address, source })?;
        let (publisher, subscription) = telemetry::channel(config.source);
        Ok(Self {
            listener,
            publisher,
            subscription,
        })
    }

    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// All subscribers share the same producer and retain only the latest state.
    pub fn telemetry(&self) -> Subscription {
        self.subscription.clone()
    }

    /// Stop accepting requests on shutdown and finish requests already in flight.
    pub async fn run(self, shutdown: impl Future<Output = ()> + Send + 'static) -> io::Result<()> {
        let (stop, stopped) = tokio::sync::watch::channel(false);
        let (closing, closed) = tokio::sync::watch::channel(None);
        let (finished, completion) = tokio::sync::oneshot::channel();
        let producer = tokio::spawn(async move {
            let result = crate::providers::demo::run(self.publisher, async move {
                let mut stopped = stopped;
                let _ = stopped.wait_for(|stop| *stop).await;
            })
            .await;
            let _ = finished.send(());
            result
        });
        let stop_on_shutdown = stop.clone();
        let listener = connection::DeadlineListener::new(self.listener, closed);
        let http_result = axum::serve(listener, router_with_telemetry(self.subscription))
            .with_graceful_shutdown(async move {
                tokio::select! { _ = shutdown => {}, _ = completion => {} }
                closing.send_replace(Some(tokio::time::Instant::now()));
                let _ = stop_on_shutdown.send(true);
            })
            .await;
        let _ = stop.send(true);
        let producer_result = producer.await.map_err(io::Error::other)?;
        producer_result.map_err(io::Error::other)?;
        http_result
    }
}

pub fn router() -> Router {
    Router::new()
        .route(
            "/horizon.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../web/horizon.js"),
                )
            }),
        )
        .route(
            "/",
            get(|| async { Html(include_str!("../../web/index.html")) }),
        )
        .route(
            "/styles.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../../web/styles.css"),
                )
            }),
        )
        .route("/health", get(|| async { Json(Health { status: "ok" }) }))
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../web/app.js"),
                )
            }),
        )
        .route(
            "/telemetry-client.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../web/telemetry-client.js"),
                )
            }),
        )
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
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        sync::oneshot,
        time::{Duration, timeout},
    };

    #[tokio::test]
    async fn shutdown_closes_active_sse_and_releases_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (publisher, subscription) = telemetry::channel(crate::Source::Demo);
        let server = Server {
            listener,
            publisher,
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
        let (publisher, subscription) = telemetry::channel(crate::Source::Demo);
        let server = Server {
            listener,
            publisher,
            subscription,
        };
        let mut telemetry = server.telemetry();
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
        timeout(Duration::from_secs(5), async {
            while telemetry.changed().await.is_ok() {}
        })
        .await
        .unwrap();
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
        let error = Server::bind(config).await.err().unwrap();
        assert!(matches!(error, ServerError::Bind { .. }));
        assert!(error.to_string().contains(&address.to_string()));
        assert!(matches!(
            Server::bind(Config {
                port: 0,
                ..Config::default()
            })
            .await
            .err()
            .unwrap(),
            ServerError::Config(ConfigError::InvalidPort)
        ));
    }
}
