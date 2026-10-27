//! SDK-independent public-module checks and an explicit opt-in official DLL smoke test.
use msfs2024_artificial_horizon::{
    Source,
    providers::simconnect::{DiagnosticStatus, SimConnectConfig, run},
    telemetry,
};
use std::{path::PathBuf, time::Duration};

#[tokio::test]
async fn missing_explicit_dll_returns_controlled_error_without_diagnostics() {
    let dll = std::env::temp_dir().join(format!(
        "simconnect-missing-{}-{}.dll",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let (publisher, mut subscription) = telemetry::channel(Source::SimConnect);
    let report = run(
        SimConnectConfig {
            dll: Some(dll),
            log: None,
        },
        publisher,
        std::future::pending(),
    )
    .await;
    assert!(report.result.is_err());
    assert!(matches!(report.diagnostics, DiagnosticStatus::Disabled));
    assert_eq!(
        serde_json::to_value(subscription.snapshot()).unwrap()["source"],
        "simconnect"
    );
    // The failed acquisition releases its publisher rather than leaving a worker alive.
    while subscription.changed().await.is_ok() {}
}

#[tokio::test]
#[ignore = "requires SIMCONNECT_TEST_DLL pointing to a separately installed official DLL"]
async fn official_dll_stops_on_requested_shutdown_and_uses_caller_channel() {
    let dll = PathBuf::from(
        std::env::var_os("SIMCONNECT_TEST_DLL").expect("Set SIMCONNECT_TEST_DLL explicitly"),
    );
    let log = std::env::current_dir()
        .unwrap()
        .join(".local/simconnect-sessions/automated-cancellation-smoke.jsonl");
    let (publisher, mut subscription) = telemetry::channel(Source::SimConnect);
    let report = run(
        SimConnectConfig {
            dll: Some(dll),
            log: Some(log.clone()),
        },
        publisher,
        async {
            tokio::time::sleep(Duration::from_millis(750)).await;
            Ok(())
        },
    )
    .await;
    report
        .result
        .expect("must stop cleanly; this does not establish live simulator acceptance");
    assert!(matches!(
        report.diagnostics,
        DiagnosticStatus::Completed { .. }
    ));
    let records: Vec<serde_json::Value> = std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let session = &records.last().expect("session outcome must be recorded")["session_id"];
    let records: Vec<_> = records
        .iter()
        .filter(|record| &record["session_id"] == session)
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
    // At least the initial connection attempt must be visible in the caller's watch.
    let snapshot = serde_json::to_value(subscription.snapshot_and_update()).unwrap();
    assert_eq!(snapshot["source"], "simconnect");
    assert!(snapshot["sequence"].as_u64().unwrap() > 0);
    assert!(subscription.changed().await.is_err());
}
