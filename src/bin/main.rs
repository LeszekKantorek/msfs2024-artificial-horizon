use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::ExitCode,
};

use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use msfs2024_artificial_horizon::{
    Config, Server, Source,
    providers::simconnect::{self, DiagnosticStatus, SimConnectConfig},
    telemetry,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Selected source; demo produces synthetic telemetry at 20 Hz.
    #[arg(long, value_enum, default_value = "demo")]
    source: CliSource,
    /// Demo HTTP interface (default: 127.0.0.1); use 0.0.0.0 for LAN access.
    #[arg(long)]
    listen_address: Option<IpAddr>,
    /// Demo HTTP port, 1-65535 (default: 8080).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,
    /// SimConnect only: absolute DLL path; defaults to SimConnect.dll beside this EXE.
    #[arg(long)]
    dll: Option<PathBuf>,
    /// SimConnect only: append diagnostic JSONL to this file (optional).
    #[arg(long)]
    log: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliSource {
    Demo,
    Simconnect,
}

#[derive(Debug)]
enum Mode {
    Demo(Config),
    SimConnect(SimConnectConfig),
}

impl Cli {
    fn into_mode(self) -> Result<Mode, clap::Error> {
        match self.source {
            CliSource::Demo => {
                if self.dll.is_some() || self.log.is_some() {
                    return Err(Self::command().error(
                        ErrorKind::ArgumentConflict,
                        "--dll and --log require --source simconnect",
                    ));
                }
                let defaults = Config::default();
                Ok(Mode::Demo(Config {
                    source: Source::Demo,
                    listen_address: self.listen_address.unwrap_or(defaults.listen_address),
                    port: self.port.unwrap_or(defaults.port),
                }))
            }
            CliSource::Simconnect => {
                if self.listen_address.is_some() || self.port.is_some() {
                    return Err(Self::command().error(
                        ErrorKind::ArgumentConflict,
                        "--listen-address and --port are unavailable for --source simconnect: HTTP/SSE integration belongs to issue #6",
                    ));
                }
                Ok(Mode::SimConnect(SimConnectConfig {
                    dll: self.dll,
                    log: self.log,
                }))
            }
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let mode = Cli::parse()
        .into_mode()
        .unwrap_or_else(|error| error.exit());
    let result = match mode {
        Mode::Demo(config) => start_demo(config).await,
        Mode::SimConnect(config) => start_simconnect(config).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn start_demo(config: Config) -> Result<(), Box<dyn Error>> {
    let server = Server::bind(config).await?;
    print_startup_message(server.local_address()?);
    run_until_ctrl_c(server).await?;
    println!("Server stopped.");
    Ok(())
}

async fn start_simconnect(config: SimConnectConfig) -> Result<(), Box<dyn Error>> {
    println!("Source: SIMCONNECT. HTTP/SSE is not started. Press Ctrl+C to stop.");
    let (publisher, subscription) = telemetry::channel(Source::SimConnect);
    let report = simconnect::run(config, publisher, tokio::signal::ctrl_c()).await;
    // Keep the caller-owned receiver alive for the entire acquisition session.
    drop(subscription);
    match report.diagnostics {
        DiagnosticStatus::Disabled => {}
        DiagnosticStatus::Completed { path } => {
            println!("Diagnostic session completed: {}", path.display());
        }
        DiagnosticStatus::Failed { path, error } => {
            eprintln!(
                "Diagnostic session is incomplete ({}): {error}",
                path.display()
            );
        }
    }
    report.result?;
    println!("SimConnect stopped.");
    Ok(())
}

fn print_startup_message(address: SocketAddr) {
    println!("Listening on {address}");
    if address.ip().is_unspecified() {
        print_lan_url(address);
    } else {
        println!("Open http://{address}");
    }
    println!("Source: DEMO (synthetic telemetry, 20 Hz). Press Ctrl+C to stop.");
}

fn print_lan_url(address: SocketAddr) {
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
        Ok(_) => eprintln!(
            "No LAN interface detected. Use the PC's private interface IP and port {}.",
            address.port()
        ),
        Err(error) => eprintln!(
            "Cannot detect a LAN IP: {error}. Use the PC's private interface IP and port {}.",
            address.port()
        ),
    }
}

async fn run_until_ctrl_c(server: Server) -> Result<(), Box<dyn Error>> {
    let (stop, stopped) = tokio::sync::oneshot::channel();
    // Propagate signal registration errors instead of claiming a clean shutdown.
    let shutdown = tokio::spawn(async {
        let result = tokio::signal::ctrl_c().await;
        let _ = stop.send(());
        result
    });
    server
        .run(async {
            let _ = stopped.await;
        })
        .await?;
    shutdown.await??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_explicit_values_convert_to_library_config() {
        let Mode::Demo(default) = Cli::try_parse_from(["main"]).unwrap().into_mode().unwrap()
        else {
            panic!("default source must be demo");
        };
        assert_eq!(default.source, Source::Demo);
        assert_eq!(default.socket_address(), "127.0.0.1:8080".parse().unwrap());
        let Mode::Demo(explicit) = Cli::try_parse_from([
            "main",
            "--source",
            "demo",
            "--listen-address",
            "0.0.0.0",
            "--port",
            "9000",
        ])
        .unwrap()
        .into_mode()
        .unwrap() else {
            panic!("explicit demo source must be demo")
        };
        assert_eq!(explicit.source, Source::Demo);
        assert_eq!(explicit.socket_address(), "0.0.0.0:9000".parse().unwrap());
    }

    #[test]
    fn simconnect_configuration_has_optional_dll_and_log() {
        let Mode::SimConnect(default) = Cli::try_parse_from(["main", "--source", "simconnect"])
            .unwrap()
            .into_mode()
            .unwrap()
        else {
            panic!("SimConnect source expected")
        };
        assert!(default.dll.is_none());
        assert!(default.log.is_none());
        let Mode::SimConnect(explicit) = Cli::try_parse_from([
            "main",
            "--source",
            "simconnect",
            "--dll",
            "C:\\runtime\\SimConnect.dll",
            "--log",
            "sessions/active-pause.jsonl",
        ])
        .unwrap()
        .into_mode()
        .unwrap() else {
            panic!("SimConnect source expected")
        };
        assert_eq!(
            explicit.dll,
            Some(PathBuf::from("C:\\runtime\\SimConnect.dll"))
        );
        assert_eq!(
            explicit.log,
            Some(PathBuf::from("sessions/active-pause.jsonl"))
        );
    }

    #[test]
    fn source_specific_options_reject_conflicts() {
        for option in ["--dll", "--log"] {
            for source in [None, Some("demo")] {
                let mut args = vec!["main", option, "file"];
                if let Some(source) = source {
                    args.extend(["--source", source]);
                }
                assert!(Cli::try_parse_from(args).unwrap().into_mode().is_err());
            }
        }
        for (option, value) in [("--port", "8080"), ("--listen-address", "127.0.0.1")] {
            assert!(
                Cli::try_parse_from(["main", "--source", "simconnect", option, value])
                    .unwrap()
                    .into_mode()
                    .is_err()
            );
        }
    }

    #[test]
    fn obsolete_probe_arguments_are_rejected() {
        for option in ["--simconnect", "--scenario", "--log-dir"] {
            assert!(Cli::try_parse_from(["main", option]).is_err());
        }
    }
}
