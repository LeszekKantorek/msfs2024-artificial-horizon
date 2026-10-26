use clap::Parser;
use msfs2024_artificial_horizon::providers::simconnect::{ProbeConfig, run_probe};
use std::{path::PathBuf, process::ExitCode};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Record diagnostic MSFS2024 SimConnect sessions (issue #5)"
)]
struct Cli {
    /// Absolute path to the official x64 SimConnect.dll; never searched on PATH.
    #[arg(long)]
    dll: PathBuf,
    /// Scenario label stored in this session's JSONL log.
    #[arg(long)]
    scenario: String,
    /// Local diagnostic output directory. Each run creates a new file.
    #[arg(long, default_value = ".local/simconnect-sessions")]
    log_dir: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run_probe(
        ProbeConfig {
            dll: cli.dll,
            scenario: cli.scenario,
            log_dir: cli.log_dir,
        },
        tokio::signal::ctrl_c(),
    )
    .await
    {
        Ok(path) => {
            println!("Probe stopped. Session log: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_dll_and_scenario_are_required() {
        assert!(Cli::try_parse_from(["simconnect-probe"]).is_err());
        assert!(
            Cli::try_parse_from(["simconnect-probe", "--dll", "C:\\runtime\\SimConnect.dll"])
                .is_err()
        );
        let cli = Cli::try_parse_from([
            "simconnect-probe",
            "--dll",
            "C:\\runtime\\SimConnect.dll",
            "--scenario",
            "active-pause",
        ])
        .unwrap();
        assert_eq!(cli.scenario, "active-pause");
        assert_eq!(cli.log_dir, PathBuf::from(".local/simconnect-sessions"));
    }
}
