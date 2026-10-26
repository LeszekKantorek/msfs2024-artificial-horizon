//! Explicit opt-in SDK smoke check. Default tests remain SDK-independent.
use msfs2024_artificial_horizon::providers::simconnect::{ProbeConfig, run_probe};
use std::{path::PathBuf, time::Duration};

#[tokio::test]
#[ignore = "requires SIMCONNECT_TEST_DLL pointing to a separately installed official DLL"]
async fn official_dll_probe_stops_on_requested_shutdown() {
    let dll = PathBuf::from(
        std::env::var_os("SIMCONNECT_TEST_DLL").expect("Set SIMCONNECT_TEST_DLL explicitly"),
    );
    let log_dir = std::env::current_dir()
        .unwrap()
        .join(".local/simconnect-sessions");
    let path = run_probe(
        ProbeConfig {
            dll,
            scenario: "automated-cancellation-smoke".into(),
            log_dir,
        },
        async {
            tokio::time::sleep(Duration::from_millis(750)).await;
            Ok(())
        },
    )
    .await
    .expect("probe must stop cleanly; this does not establish live simulator acceptance");
    let records: Vec<serde_json::Value> = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        records
            .iter()
            .any(|record| record["kind"] == "library_loaded")
    );
    assert!(
        records
            .iter()
            .any(|record| record["kind"] == "connection_attempt")
    );
    assert_eq!(records.last().unwrap()["data"]["success"], true);
}
