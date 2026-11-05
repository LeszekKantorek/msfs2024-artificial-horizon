use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use msfs2024_artificial_horizon::{
    Config,
    application::{self, SourceSelection},
    providers::simconnect::SimConnectConfig,
};
use std::{
    io,
    net::IpAddr,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Selected source; demo produces synthetic telemetry at 20 Hz.
    #[arg(long, value_enum, default_value = "demo")]
    source: CliSource,
    /// HTTP interface (default: 127.0.0.1); use 0.0.0.0 for LAN access.
    #[arg(long)]
    listen_address: Option<IpAddr>,
    /// HTTP port, 1-65535 (default: 8080).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,
    /// Stop when this absolute, initially absent file is created.
    #[arg(long)]
    shutdown_file: Option<PathBuf>,
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

impl Cli {
    fn into_configuration(self) -> Result<(Config, SourceSelection), clap::Error> {
        let defaults = Config::default();
        let config = Config {
            listen_address: self.listen_address.unwrap_or(defaults.listen_address),
            port: self.port.unwrap_or(defaults.port),
        };
        let source = match self.source {
            CliSource::Demo => {
                if self.dll.is_some() || self.log.is_some() {
                    return Err(Self::command().error(
                        ErrorKind::ArgumentConflict,
                        "--dll and --log require --source simconnect",
                    ));
                }
                SourceSelection::Demo
            }
            CliSource::Simconnect => SourceSelection::SimConnect(SimConnectConfig {
                dll: self.dll,
                log: self.log,
            }),
        };
        Ok((config, source))
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(path) = &cli.shutdown_file {
        validate_shutdown_file(path).unwrap_or_else(|error| {
            Cli::command()
                .error(ErrorKind::ValueValidation, error)
                .exit()
        });
    }
    let shutdown_file = cli.shutdown_file.clone();
    let (config, source) = cli
        .into_configuration()
        .unwrap_or_else(|error| error.exit());
    match application::run(config, source, shutdown(shutdown_file)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn validate_shutdown_file(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("--shutdown-file must be an absolute path".into());
    }
    match std::fs::metadata(path) {
        Ok(_) => Err("--shutdown-file already exists".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot inspect --shutdown-file: {error}")),
    }
}

async fn shutdown(path: Option<PathBuf>) -> io::Result<()> {
    let Some(path) = path else {
        return tokio::signal::ctrl_c().await;
    };
    tokio::select! {
        result = tokio::signal::ctrl_c() => result,
        result = async {
            loop {
                match std::fs::metadata(&path) {
                    Ok(_) => return Ok(()),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {},
                    Err(error) => return Err(io::Error::new(error.kind(), format!("cannot inspect shutdown file: {error}"))),
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        } => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_demo_and_explicit_simconnect_share_http_configuration() {
        let (config, source) = Cli::try_parse_from(["main"])
            .unwrap()
            .into_configuration()
            .unwrap();
        assert!(matches!(source, SourceSelection::Demo));
        assert_eq!(config.socket_address(), "127.0.0.1:8080".parse().unwrap());
        let (config, source) = Cli::try_parse_from([
            "main",
            "--source",
            "simconnect",
            "--listen-address",
            "0.0.0.0",
            "--port",
            "9000",
            "--dll",
            "C:\\runtime\\SimConnect.dll",
            "--log",
            "sessions/test.jsonl",
        ])
        .unwrap()
        .into_configuration()
        .unwrap();
        assert_eq!(config.socket_address(), "0.0.0.0:9000".parse().unwrap());
        let SourceSelection::SimConnect(options) = source else {
            panic!("expected SimConnect")
        };
        assert!(options.dll.unwrap().is_absolute());
        assert_eq!(options.log, Some(PathBuf::from("sessions/test.jsonl")));
    }
    #[test]
    fn removed_diagnostic_limit_is_absent_and_rejected() {
        let help = Cli::command().render_long_help().to_string();
        assert!(!help.contains("--log-max-bytes"));
        let error = Cli::try_parse_from([
            "main",
            "--source",
            "simconnect",
            "--log",
            "log.jsonl",
            "--log-max-bytes",
            "10",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }
    #[test]
    fn diagnostic_options_require_simconnect_and_obsolete_options_are_rejected() {
        for option in ["--dll", "--log"] {
            assert!(
                Cli::try_parse_from(["main", option, "file"])
                    .unwrap()
                    .into_configuration()
                    .is_err()
            );
        }
        for option in ["--simconnect", "--scenario", "--log-dir"] {
            assert!(Cli::try_parse_from(["main", option]).is_err());
        }
    }
}
