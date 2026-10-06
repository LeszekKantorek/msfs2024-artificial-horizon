use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    process::ExitCode,
};

use clap::{Parser, ValueEnum};
use msfs2024_artificial_horizon::{Config, Server, Source};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Selected source; demo produces synthetic telemetry at 20 Hz.
    #[arg(long, value_enum, default_value = "demo")]
    source: CliSource,
    /// Local interface IP; use 0.0.0.0 explicitly for LAN access.
    #[arg(long, default_value = "127.0.0.1")]
    listen_address: IpAddr,
    /// HTTP port (1-65535).
    #[arg(long, default_value_t = 8080, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliSource {
    Demo,
    Simconnect,
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        Self {
            source: match cli.source {
                CliSource::Demo => Source::Demo,
                CliSource::Simconnect => Source::SimConnect,
            },
            listen_address: cli.listen_address,
            port: cli.port,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match start(Cli::parse().into()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn start(config: Config) -> Result<(), Box<dyn Error>> {
    let server = Server::bind(config).await?;
    print_startup_message(server.local_address()?);
    run_until_ctrl_c(server).await?;
    println!("Server stopped.");
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
        let default: Config = Cli::try_parse_from(["main"]).unwrap().into();
        assert_eq!(default.source, Source::Demo);
        assert_eq!(default.socket_address(), "127.0.0.1:8080".parse().unwrap());
        let explicit: Config = Cli::try_parse_from([
            "main",
            "--source",
            "simconnect",
            "--listen-address",
            "0.0.0.0",
            "--port",
            "9000",
        ])
        .unwrap()
        .into();
        assert_eq!(explicit.source, Source::SimConnect);
        assert_eq!(explicit.socket_address(), "0.0.0.0:9000".parse().unwrap());
    }
}
