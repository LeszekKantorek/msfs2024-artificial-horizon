//! Application composition: source selection, channel lifetime, and coordinated shutdown.
use std::{future::Future, io, net::SocketAddr, panic::AssertUnwindSafe};

use futures_util::FutureExt;
use tokio::sync::watch;

use crate::{
    Config, ConfigError, Server, ServerError, Source,
    providers::{
        demo,
        simconnect::{self, DiagnosticStatus, SimConnectConfig},
    },
    telemetry::{self, ChannelLifetime, Publisher, State, TelemetryError},
};

#[derive(Debug)]
pub enum SourceSelection {
    Demo,
    SimConnect(SimConnectConfig),
}
impl SourceSelection {
    fn source(&self) -> Source {
        match self {
            Self::Demo => Source::Demo,
            Self::SimConnect(_) => Source::SimConnect,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    SourceConfiguration(#[from] simconnect::SimConnectError),
    #[error(transparent)]
    Server(#[from] ServerError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Telemetry(#[from] TelemetryError),
    #[error("application task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

struct Shutdown(watch::Sender<bool>);
impl Drop for Shutdown {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
async fn stopped(mut shutdown: watch::Receiver<bool>) {
    let _ = shutdown.wait_for(|stop| *stop).await;
}

/// Start HTTP before acquisition, retaining service through source failures.
/// The caller owns process signals. Dropping this future signals both tasks;
/// normal shutdown additionally waits for their results and native cleanup.
pub async fn run(
    config: Config,
    selection: SourceSelection,
    shutdown: impl Future<Output = io::Result<()>> + Send + 'static,
) -> Result<(), ApplicationError> {
    config.validate()?;
    if let SourceSelection::SimConnect(options) = &selection {
        // Resolve path syntax only: existence, exports, and native availability
        // belong to acquisition and cannot prevent HTTP from starting.
        simconnect::resolve_dll(options.dll.as_deref())?;
    }
    let source = selection.source();
    let (publisher, subscription) = telemetry::channel(source);
    let lifetime = publisher.retain_channel();
    let server = Server::bind(config, subscription).await?;
    print_startup(server.local_address()?, source);
    let (stop, stopping) = watch::channel(false);
    let guard = Shutdown(stop);
    let http = tokio::spawn(server.run(stopped(stopping.clone())));
    let producer = tokio::spawn(supervise(selection, publisher, lifetime, stopping));
    coordinate(http, producer, guard, shutdown).await
}

async fn coordinate(
    mut http: tokio::task::JoinHandle<io::Result<()>>,
    mut producer: tokio::task::JoinHandle<Result<(), TelemetryError>>,
    guard: Shutdown,
    shutdown: impl Future<Output = io::Result<()>>,
) -> Result<(), ApplicationError> {
    let mut http_result = None;
    let mut producer_result = None;
    let trigger = tokio::select! {
        result = shutdown => result.map_err(ApplicationError::from),
        result = &mut http => {
            http_result = Some(result);
            Err(ApplicationError::Io(io::Error::other("HTTP service ended unexpectedly")))
        },
        result = &mut producer => {
            producer_result = Some(result);
            Err(ApplicationError::Io(io::Error::other("telemetry supervision ended unexpectedly")))
        },
    };
    guard.0.send_replace(true);
    let http_result = match http_result {
        Some(result) => result,
        None => http.await,
    };
    let producer_result = match producer_result {
        Some(result) => result,
        None => producer.await,
    };
    // Collect both outcomes before propagating any error.
    http_result??;
    producer_result??;
    trigger?;
    println!("Server stopped.");
    Ok(())
}

async fn supervise(
    selection: SourceSelection,
    publisher: Publisher,
    lifetime: ChannelLifetime,
    shutdown: watch::Receiver<bool>,
) -> Result<(), TelemetryError> {
    let acquisition = async {
        match selection {
            SourceSelection::Demo => {
                if let Err(error) = demo::run(publisher, stopped(shutdown.clone())).await {
                    eprintln!("Demo acquisition failed: {error}");
                }
            }
            SourceSelection::SimConnect(config) => {
                let report = simconnect::run(config, publisher, async {
                    stopped(shutdown.clone()).await;
                    Ok(())
                })
                .await;
                if let Err(error) = report.result {
                    eprintln!("SimConnect acquisition failed: {error}");
                }
                match report.diagnostics {
                    DiagnosticStatus::Disabled => {}
                    DiagnosticStatus::Completed { path } => {
                        println!("Diagnostic session completed: {}", path.display())
                    }
                    DiagnosticStatus::Failed { path, error } => eprintln!(
                        "Diagnostic session is incomplete ({}): {error}",
                        path.display()
                    ),
                }
            }
        }
    };
    supervise_acquisition(acquisition, lifetime, shutdown.clone()).await
}

async fn supervise_acquisition(
    acquisition: impl Future<Output = ()>,
    lifetime: ChannelLifetime,
    shutdown: watch::Receiver<bool>,
) -> Result<(), TelemetryError> {
    if AssertUnwindSafe(acquisition).catch_unwind().await.is_err() {
        eprintln!("Telemetry producer panicked; HTTP remains available.");
    }
    let mut publisher = lifetime.recover().await;
    if !*shutdown.borrow() {
        eprintln!("Telemetry producer stopped; data unavailable. HTTP remains available.");
        publisher.publish(State::Disconnected)?;
        stopped(shutdown).await;
    }
    // Retain the final sender until global shutdown, including after a failure.
    drop(publisher);
    Ok(())
}

fn print_startup(address: SocketAddr, source: Source) {
    println!("Listening on {address}");
    if address.ip().is_unspecified() {
        let detected = if address.is_ipv4() {
            local_ip_address::local_ip()
        } else {
            local_ip_address::local_ipv6()
        };
        match detected {
            Ok(ip) if !ip.is_loopback() && !ip.is_unspecified() => println!(
                "Open http://{} on the same private network (check the interface if using VPN).",
                SocketAddr::new(ip, address.port())
            ),
            _ => eprintln!(
                "Cannot detect a LAN IP. Use the PC's private interface IP and port {}.",
                address.port()
            ),
        }
    } else {
        println!("Open http://{address}");
    }
    match source {
        Source::Demo => {
            println!("Source: DEMO (synthetic telemetry, 20 Hz). Press Ctrl+C to stop.")
        }
        Source::SimConnect => println!("Source: SIMCONNECT. Press Ctrl+C to stop."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::Attitude;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn caller_shutdown_closes_real_sse_and_releases_port_in_both_modes() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::{TcpListener, TcpStream},
        };
        for simconnect in [false, true] {
            // Public configuration rejects port zero; reserve an ephemeral port
            // first and release it immediately before application startup.
            let reservation = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = reservation.local_addr().unwrap();
            drop(reservation);
            let selection = if simconnect {
                SourceSelection::SimConnect(SimConnectConfig {
                    dll: Some(std::env::temp_dir().join(format!(
                        "missing-shutdown-{}-{}.dll",
                        std::process::id(),
                        address.port()
                    ))),
                    log: None,
                })
            } else {
                SourceSelection::Demo
            };
            let (stop, stopping) = tokio::sync::oneshot::channel();
            let app = tokio::spawn(run(
                Config {
                    listen_address: address.ip(),
                    port: address.port(),
                },
                selection,
                async { stopping.await.map_err(io::Error::other) },
            ));
            let mut clients = Vec::new();
            for _ in 0..2 {
                let mut client = timeout(Duration::from_secs(5), async {
                    loop {
                        match TcpStream::connect(address).await {
                            Ok(client) => break client,
                            Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
                        }
                    }
                })
                .await
                .unwrap();
                client
                    .write_all(b"GET /api/v1/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .await
                    .unwrap();
                timeout(Duration::from_secs(5), async {
                    let mut received = String::new();
                    loop {
                        let mut bytes = [0; 4096];
                        let count = client.read(&mut bytes).await.unwrap();
                        assert!(count > 0, "SSE ended before expected source state");
                        received.push_str(&String::from_utf8_lossy(&bytes[..count]));
                        if received.contains("event: telemetry")
                            && received.contains(if simconnect {
                                "\"state\":\"disconnected\""
                            } else {
                                "\"state\":\"live\""
                            })
                        {
                            break;
                        }
                    }
                    assert!(received.starts_with("HTTP/1.1 200 OK"));
                })
                .await
                .unwrap();
                clients.push(client);
            }
            stop.send(()).unwrap();
            timeout(Duration::from_secs(7), app)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            for mut client in clients {
                timeout(Duration::from_secs(2), client.read_to_end(&mut Vec::new()))
                    .await
                    .unwrap()
                    .unwrap();
            }
            let rebound = TcpListener::bind(address).await.unwrap();
            assert_eq!(rebound.local_addr().unwrap(), address);
        }
    }

    #[tokio::test]
    async fn http_failure_and_signal_error_stop_and_join_the_producer() {
        for http_fails in [false, true] {
            let (stop, stopping) = watch::channel(false);
            let guard = Shutdown(stop);
            let (joined, observed) = tokio::sync::oneshot::channel();
            let producer = tokio::spawn(async move {
                stopped(stopping).await;
                joined.send(()).unwrap();
                Ok(())
            });
            let http_stopping = guard.0.subscribe();
            let http = tokio::spawn(async move {
                if http_fails {
                    Err(io::Error::other("controlled HTTP failure"))
                } else {
                    stopped(http_stopping).await;
                    Ok(())
                }
            });
            let signal = async move {
                if http_fails {
                    std::future::pending().await
                } else {
                    Err(io::Error::other("controlled signal failure"))
                }
            };
            assert!(coordinate(http, producer, guard, signal).await.is_err());
            observed.await.unwrap();
        }
    }

    #[tokio::test]
    async fn cancelling_coordination_signals_both_components() {
        let (stop, stopping) = watch::channel(false);
        let guard = Shutdown(stop);
        let (http_done, http_observed) = tokio::sync::oneshot::channel();
        let (producer_done, producer_observed) = tokio::sync::oneshot::channel();
        let http_stopping = stopping.clone();
        let http = tokio::spawn(async move {
            stopped(http_stopping).await;
            http_done.send(()).unwrap();
            Ok(())
        });
        let producer = tokio::spawn(async move {
            stopped(stopping).await;
            producer_done.send(()).unwrap();
            Ok(())
        });
        let task = tokio::spawn(coordinate(http, producer, guard, std::future::pending()));
        tokio::task::yield_now().await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        timeout(Duration::from_secs(2), http_observed)
            .await
            .unwrap()
            .unwrap();
        timeout(Duration::from_secs(2), producer_observed)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn completed_and_panicked_producers_invalidate_live_without_closing_channel() {
        for panic in [false, true] {
            let (mut publisher, mut subscription) = telemetry::channel(Source::SimConnect);
            let lifetime = publisher.retain_channel();
            let (stop, stopping) = watch::channel(false);
            let (release, released) = tokio::sync::oneshot::channel();
            let task = tokio::spawn(supervise_acquisition(
                async move {
                    publisher
                        .publish(State::Live(Attitude::new(15.0, 30.0).unwrap()))
                        .unwrap();
                    let _ = released.await;
                    drop(publisher);
                    assert!(!panic, "controlled producer panic");
                },
                lifetime,
                stopping,
            ));
            assert_eq!(
                serde_json::to_value(subscription.changed().await.unwrap()).unwrap()["state"],
                "live"
            );
            release.send(()).unwrap();
            let unavailable = timeout(Duration::from_secs(2), subscription.changed())
                .await
                .unwrap()
                .unwrap();
            let unavailable = serde_json::to_value(unavailable).unwrap();
            assert_eq!(unavailable["state"], "disconnected");
            assert!(unavailable["attitude"].is_null());
            assert!(!task.is_finished());
            assert!(
                timeout(Duration::from_millis(20), subscription.changed())
                    .await
                    .is_err()
            );
            stop.send_replace(true);
            task.await.unwrap().unwrap();
            assert!(subscription.changed().await.is_err());
        }
    }
}
