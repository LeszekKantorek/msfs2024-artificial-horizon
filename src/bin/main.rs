use clap::{CommandFactory, Parser, error::ErrorKind};
use msfs2024_artificial_horizon::{
    Config,
    application::{self, SourceSelection},
    providers::simconnect::SimConnectConfig,
};
use std::{io, net::IpAddr, path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// DLL source: demo.dll or SimConnect.dll, beside the EXE or an absolute path.
    #[arg(long, default_value = "demo.dll")]
    source: PathBuf,
    /// Required for Demo: JSONL scenario file.
    #[arg(long)]
    scenario: Option<PathBuf>,
    /// HTTP interface (default: 127.0.0.1); use 0.0.0.0 for LAN access.
    #[arg(long)]
    listen_address: Option<IpAddr>,
    /// HTTP port, 1-65535 (default: 8080).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,
    /// SIM_FRAME interval in frame events (0-4294967295; default: 0).
    #[arg(long, default_value_t = 0)]
    simconnect_interval: u32,
    /// Append diagnostic JSONL to this file (optional).
    #[arg(long)]
    log: Option<PathBuf>,
}

impl Cli {
    fn into_configuration(self) -> Result<(Config, SourceSelection), clap::Error> {
        let defaults = Config::default();
        let config = Config {
            listen_address: self.listen_address.unwrap_or(defaults.listen_address),
            port: self.port.unwrap_or(defaults.port),
        };
        let dll =
            msfs2024_artificial_horizon::providers::simconnect::resolve_dll(Some(&self.source))
                .map_err(|error| Self::command().error(ErrorKind::ValueValidation, error))?;
        msfs2024_artificial_horizon::providers::simconnect::validate_scenario(
            msfs2024_artificial_horizon::providers::simconnect::source_kind(&dll),
            self.scenario.as_deref(),
        )
        .map_err(|error| Self::command().error(ErrorKind::ValueValidation, error))?;
        let scenario = self
            .scenario
            .map(|path| {
                if path.is_absolute() {
                    Ok(path)
                } else {
                    std::env::current_dir().map(|cwd| cwd.join(path))
                }
            })
            .transpose()
            .map_err(|error| Self::command().error(ErrorKind::ValueValidation, error))?;
        let source = SourceSelection(SimConnectConfig {
            dll: Some(dll),
            log: self.log,
            scenario,
            interval: self.simconnect_interval,
        });
        Ok((config, source))
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let (config, source) = cli
        .into_configuration()
        .unwrap_or_else(|error| error.exit());
    match application::run(config, source, shutdown()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn read_stop_command(mut reader: impl io::BufRead) -> io::Result<bool> {
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(false);
        }
        if line.trim_end_matches(['\r', '\n']) == "stop" {
            return Ok(true);
        }
    }
}

async fn shutdown() -> io::Result<()> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || match read_stop_command(io::stdin().lock()) {
        Ok(true) => {
            let _ = sender.send(());
        }
        Ok(false) => {}
        Err(error) => eprintln!("Error reading stdin: {error}"),
    });
    tokio::select! {
        result = tokio::signal::ctrl_c() => result,
        _ = async { if receiver.await.is_err() { std::future::pending::<()>().await; } } => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interval_cli_accepts_full_dword_and_rejects_invalid_values() {
        assert_eq!(
            Cli::try_parse_from(["main"]).unwrap().simconnect_interval,
            0
        );
        for value in ["0", "1", "4294967294", "4294967295"] {
            let cli = Cli::try_parse_from(["main", "--simconnect-interval", value]).unwrap();
            assert_eq!(cli.simconnect_interval.to_string(), value);
        }
        for value in ["4294967296", "-1", "1.5"] {
            assert!(Cli::try_parse_from(["main", "--simconnect-interval", value]).is_err());
        }
    }
    #[test]
    fn reads_stop_command_and_eof() {
        assert!(read_stop_command(std::io::Cursor::new(b"ignored\r\nstop\r\n")).unwrap());
        assert!(!read_stop_command(std::io::Cursor::new(b"ignored\n")).unwrap());
    }
    #[test]
    fn source_validation_and_legacy_rejection() {
        let (_, selected) = Cli::try_parse_from(["main", "--scenario", "x"])
            .unwrap()
            .into_configuration()
            .unwrap();
        assert_eq!(
            msfs2024_artificial_horizon::providers::simconnect::source_kind(
                selected.0.dll.as_ref().unwrap()
            ),
            msfs2024_artificial_horizon::Source::Demo
        );
        for value in ["demo", "simconnect", "foo.dll", "folder/demo.dll"] {
            assert!(
                Cli::try_parse_from(["main", "--source", value])
                    .unwrap()
                    .into_configuration()
                    .is_err()
            );
        }
        assert!(Cli::try_parse_from(["main", "--dll", "x"]).is_err());
        assert!(
            Cli::try_parse_from(["main", "--source", "SimConnect.dll", "--scenario", "x"])
                .unwrap()
                .into_configuration()
                .is_err()
        );
        assert!(
            Cli::try_parse_from(["main", "--source", "DEMO.dll", "--scenario", "x"])
                .unwrap()
                .into_configuration()
                .is_ok()
        );
    }
}
