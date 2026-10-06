//! Embedded web assets, HTTP liveness, listener ownership and graceful shutdown.

use std::{future::Future, io, net::SocketAddr};

use axum::{Json, Router, http::header, response::Html, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::telemetry::{self, Publisher, Subscription};
use crate::{Config, ConfigError};

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
        let http_result = axum::serve(self.listener, router())
            .with_graceful_shutdown(async move {
                tokio::select! { _ = shutdown => {}, _ = completion => {} }
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
