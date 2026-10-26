use super::{
    event::{Event, Wake},
    journal::{Journal, fingerprint, number},
    lifecycle::{self, Lifecycle, candidate_attitude, state_name},
    native::{Connection, Library, Message, NativeError},
};
use crate::{
    Source,
    telemetry::{self, Publisher, Subscription},
};
use serde_json::{Value, json};
use std::{
    ffi::CStr,
    future::Future,
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

/// Explicit diagnostic selection. This does not enable the production provider.
#[derive(Debug)]
pub struct ProbeConfig {
    pub dll: PathBuf,
    pub scenario: String,
    pub log_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("Invalid probe configuration: {0}")]
    Configuration(String),
    #[error("Probe I/O or logging error: {0}")]
    Io(#[from] io::Error),
    #[error("SimConnect error: {0}")]
    Native(String),
    #[error("Telemetry error: {0}")]
    Telemetry(#[from] telemetry::TelemetryError),
    #[error("SimConnect worker panicked")]
    WorkerPanic,
}

impl From<NativeError> for ProbeError {
    fn from(error: NativeError) -> Self {
        Self::Native(error.to_string())
    }
}

/// Run one diagnostic worker until caller-owned cancellation or a fatal error.
/// Native calls and the log writer belong to the worker, never the Tokio runtime.
/// Successful return means controlled shutdown, not live simulator acceptance.
pub async fn run_probe(
    config: ProbeConfig,
    shutdown: impl Future<Output = io::Result<()>>,
) -> Result<PathBuf, ProbeError> {
    if !config.dll.is_absolute() {
        return Err(ProbeError::Configuration(
            "--dll must be an absolute path".into(),
        ));
    }
    if config.scenario.trim().is_empty() {
        return Err(ProbeError::Configuration(
            "--scenario must not be empty".into(),
        ));
    }
    let journal = Journal::create(&config.log_dir, &config.scenario)?;
    let log_path = journal.path().to_owned();
    println!("Session log: {}", log_path.display());
    println!(
        "Diagnostic hypotheses only; flight availability and attitude signs require cockpit verification. Press Ctrl+C to stop."
    );
    let stop = Arc::new(Event::new(true)?);
    let worker_stop = Arc::clone(&stop);
    let worker = thread::Builder::new()
        .name("simconnect-probe".into())
        .spawn(move || worker_entry(config, journal, &worker_stop))?;
    let cancellation = Cancellation(Arc::clone(&stop));
    tokio::pin!(shutdown);
    let signal_result = loop {
        if worker.is_finished() {
            break Ok(());
        }
        tokio::select! {
            result = &mut shutdown => { stop.signal()?; break result; }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    };
    // Joining only after completion keeps native work off this runtime thread.
    while !worker.is_finished() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let result = worker.join().map_err(|_| ProbeError::WorkerPanic)?;
    drop(cancellation);
    result?;
    signal_result?;
    Ok(log_path)
}

struct Cancellation(Arc<Event>);
impl Drop for Cancellation {
    fn drop(&mut self) {
        let _ = self.0.signal();
    }
}

fn worker_entry(config: ProbeConfig, mut journal: Journal, stop: &Event) -> Result<(), ProbeError> {
    let result = worker(&config, &mut journal, stop);
    let ending = journal.record(
        "session_end",
        json!({
            "success": result.is_ok(), "error": result.as_ref().err().map(ToString::to_string),
        }),
        true,
    );
    match (result, ending) {
        (Err(error), Err(log_error)) => Err(ProbeError::Io(io::Error::other(format!(
            "{error}; cannot record session end: {log_error}"
        )))),
        (Err(error), _) => Err(error),
        (Ok(()), ending) => {
            ending?;
            Ok(())
        }
    }
}

fn command_record(program: &str, arguments: &[&str]) -> Value {
    match Command::new(program).args(arguments).output() {
        Ok(output) => {
            json!({ "command": program, "arguments": arguments, "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).trim(), "stderr": String::from_utf8_lossy(&output.stderr).trim() })
        }
        Err(error) => {
            json!({ "command": program, "arguments": arguments, "unavailable": error.to_string() })
        }
    }
}

fn worker(config: &ProbeConfig, journal: &mut Journal, stop: &Event) -> Result<(), ProbeError> {
    let executable = std::env::current_exe()?;
    journal.record("environment", json!({
        "package_version": env!("CARGO_PKG_VERSION"), "target": "x86_64-pc-windows-msvc",
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "current_directory": std::env::current_dir()?, "executable": fingerprint(&executable)?,
        "rustc_at_run": command_record("rustc", &["-Vv"]),
        "git_head_at_run": command_record("git", &["rev-parse", "HEAD"]),
        "git_changes_at_run": command_record("git", &["status", "--porcelain", "--untracked-files=no"]),
        "windows": command_record("cmd", &["/d", "/c", "ver"]),
        "sdk": sdk_version(&config.dll), "scenario": config.scenario,
        "requested_fields": ["pitch radians", "pitch degrees", "bank radians", "bank degrees"],
        "sign_hypothesis": "negate pitch and bank; normalize bank to [-180,180)",
        "note": "runtime toolchain/git metadata is contextual; executable SHA-256 identifies the tested bytes",
    }), true)?;
    match fingerprint(&config.dll) {
        Ok(identity) => journal.record("dll_identity", identity, true)?,
        Err(error) => {
            journal.record(
                "dll_identity_error",
                json!({ "path": config.dll, "error": error.to_string() }),
                true,
            )?;
        }
    }
    journal.record("library_load_attempt", json!({ "path": config.dll }), true)?;
    let library = Library::load(&config.dll).inspect_err(|error| {
        let _ = journal.record("native_error", error, true);
    })?;
    journal.record("library_loaded", json!({}), true)?;
    let result = connections(&library, journal, stop);
    journal.record(
        "library_release",
        json!({ "after_all_connections_closed": result.is_ok(),
        "note": "a failed native close retains the mapping; see native_error records" }),
        true,
    )?;
    drop(library);
    result
}

fn sdk_version(dll: &Path) -> Value {
    // SDK installation metadata is optional. Runtime DLLs may be stored elsewhere.
    for parent in dll.ancestors().skip(1).take(4) {
        let version = parent.join("version.txt");
        if let Ok(contents) = std::fs::read_to_string(&version) {
            return json!({ "metadata_path": version, "contents": contents.trim() });
        }
    }
    json!({ "unavailable": "No nearby SDK version.txt; record the SDK version with observations" })
}

fn retry_seconds(attempt: u32) -> u32 {
    (1_u32 << attempt.min(4)).min(10)
}

fn connections(library: &Library, journal: &mut Journal, stop: &Event) -> Result<(), ProbeError> {
    let (publisher, subscription) = telemetry::channel(Source::SimConnect);
    let mut output = Output {
        publisher,
        subscription,
        transition: None,
    };
    let mut lifecycle = Lifecycle::default();
    output.update(&lifecycle, journal, Instant::now())?;
    let mut generation = 0_u64;
    let mut failures = 0_u32;
    loop {
        if matches!(stop.wait(None, 0)?, Wake::Stop) {
            return Ok(());
        }
        generation += 1;
        let notification = Notification {
            event: Some(Event::new(false)?),
            library,
        };
        journal.record(
            "connection_attempt",
            json!({ "generation": generation }),
            true,
        )?;
        let mut connection = match library.open(notification.event().handle()) {
            Ok(connection) => connection,
            Err(error) => {
                journal.record("native_error", &error, true)?;
                let seconds = retry_seconds(failures);
                failures = failures.saturating_add(1);
                journal.record(
                    "reconnect_wait",
                    json!({ "seconds": seconds, "generation": generation }),
                    true,
                )?;
                if matches!(stop.wait(None, seconds * 1000)?, Wake::Stop) {
                    return Ok(());
                }
                continue;
            }
        };
        let session_start = Instant::now();
        let result = (|| {
            journal.record("api_result", json!({ "operation": "SimConnect_Open", "success": true, "generation": generation }), true)?;
            println!("SimConnect connected (session {generation}).");
            lifecycle.connect();
            output.update(&lifecycle, journal, Instant::now())?;
            session(
                &mut connection,
                notification.event(),
                stop,
                journal,
                &mut lifecycle,
                &mut output,
            )
        })();
        // Explicit cleanup is attempted even when the logger or packet decoder failed.
        let closed = connection.close();
        let close_log = match &closed {
            Ok(()) => journal.record(
                "api_result",
                json!({ "operation": "SimConnect_Close", "success": true }),
                true,
            ),
            Err(error) => journal.record("native_error", error, true),
        };
        closed?;
        close_log?;
        journal.record(
            "notification_release",
            json!({ "generation": generation }),
            true,
        )?;
        drop(notification);
        lifecycle.disconnect();
        output.update(&lifecycle, journal, Instant::now())?;
        match result? {
            SessionEnd::Stop => return Ok(()),
            SessionEnd::Reconnect => {}
        }
        if session_start.elapsed() >= Duration::from_secs(10) {
            failures = 0;
        }
        let seconds = retry_seconds(failures);
        failures = failures.saturating_add(1);
        journal.record(
            "reconnect_wait",
            json!({ "seconds": seconds, "generation": generation }),
            true,
        )?;
        if matches!(stop.wait(None, seconds * 1000)?, Wake::Stop) {
            return Ok(());
        }
    }
}

// The connection may still borrow its event if Close fails, including on a
// panic/unwind path. Preserve the event with the retained native mapping rather
// than recycling its handle while native ownership remains unconfirmed.
struct Notification<'library> {
    event: Option<Event>,
    library: &'library Library,
}
impl Notification<'_> {
    fn event(&self) -> &Event {
        self.event
            .as_ref()
            .expect("notification is owned until drop")
    }
}
impl Drop for Notification<'_> {
    fn drop(&mut self) {
        if self.library.has_unconfirmed_close()
            && let Some(event) = self.event.take()
        {
            std::mem::forget(event);
        }
    }
}

struct Output {
    publisher: Publisher,
    subscription: Subscription,
    transition: Option<(&'static str, &'static str, &'static str)>,
}
impl Output {
    fn update(
        &mut self,
        lifecycle: &Lifecycle,
        journal: &mut Journal,
        now: Instant,
    ) -> Result<(), ProbeError> {
        self.publisher.expire()?;
        let evaluation = lifecycle.evaluate(now);
        let transition = (
            state_name(evaluation.candidate),
            state_name(evaluation.published),
            evaluation.reason,
        );
        if self.transition != Some(transition) {
            self.publisher.publish(evaluation.published)?;
            journal.record(
                "state_transition",
                json!({ "candidate_state": transition.0,
                "candidate_is_hypothesis": true, "published": self.subscription.snapshot(),
                "reason": evaluation.reason,
                "note": "unverified live hypotheses remain waiting in application telemetry" }),
                true,
            )?;
            println!(
                "State: {} (candidate: {}, {})",
                transition.1, transition.0, transition.2
            );
            self.transition = Some(transition);
        }
        Ok(())
    }
}

enum SessionEnd {
    Stop,
    Reconnect,
}

fn api_result(
    connection: &Connection<'_>,
    journal: &mut Journal,
    operation: &str,
    result: Result<(), NativeError>,
    context: Value,
) -> Result<(), ProbeError> {
    match result {
        Ok(()) => {
            let sent = connection.last_sent_packet_id();
            journal.record(
                "api_result",
                json!({ "operation": operation, "success": true,
                "send_id": sent.as_ref().ok(), "context": context }),
                false,
            )?;
            match sent {
                Ok(id) => journal.record("api_result", json!({ "operation": "SimConnect_GetLastSentPacketID", "success": true, "send_id": id }), false)?,
                Err(error) => journal.record("native_error", error, true)?,
            }
            Ok(())
        }
        Err(error) => {
            journal.record("native_error", &error, true)?;
            Err(error.into())
        }
    }
}

fn configure(connection: &mut Connection<'_>, journal: &mut Journal) -> Result<(), ProbeError> {
    for index in 0..4 {
        let result = connection.add_attitude_field(index);
        api_result(
            connection,
            journal,
            "SimConnect_AddToDataDefinition",
            result,
            json!({ "field_index": index }),
        )?;
    }
    for (id, name) in [
        (lifecycle::EVENT_SIM, c"Sim"),
        (lifecycle::EVENT_PAUSE, c"Pause_EX1"),
        (lifecycle::EVENT_FLIGHT, c"FlightLoaded"),
    ] {
        let result = connection.subscribe(id, name);
        api_result(
            connection,
            journal,
            "SimConnect_SubscribeToSystemEvent",
            result,
            json!({ "event_id": id, "name": name.to_string_lossy() }),
        )?;
    }
    query_states(connection, journal)?;
    let result = connection.request_attitude();
    api_result(
        connection,
        journal,
        "SimConnect_RequestDataOnSimObject",
        result,
        json!({ "period": "SIM_FRAME", "flags": 0, "object_id": 0 }),
    )
}

fn query_states(connection: &mut Connection<'_>, journal: &mut Journal) -> Result<(), ProbeError> {
    for (id, name) in [
        (lifecycle::STATE_SIM, c"Sim"),
        (lifecycle::STATE_FLIGHT, c"FlightLoaded"),
    ] {
        request_state(connection, journal, id, name)?;
    }
    Ok(())
}

fn request_state(
    connection: &mut Connection<'_>,
    journal: &mut Journal,
    id: u32,
    name: &CStr,
) -> Result<(), ProbeError> {
    let result = connection.request_state(id, name);
    api_result(
        connection,
        journal,
        "SimConnect_RequestSystemState",
        result,
        json!({ "request_id": id, "name": name.to_string_lossy() }),
    )
}

#[derive(Default, serde::Serialize)]
struct Counters {
    packets: u64,
    empty_dispatch: u64,
    efail_dispatch: u64,
    notification_wakes: u64,
    timeout_wakes: u64,
}

fn session(
    connection: &mut Connection<'_>,
    notification: &Event,
    stop: &Event,
    journal: &mut Journal,
    lifecycle: &mut Lifecycle,
    output: &mut Output,
) -> Result<SessionEnd, ProbeError> {
    configure(connection, journal)?;
    let mut counters = Counters::default();
    let mut last_packet = Instant::now();
    let mut last_sample: Option<Instant> = None;
    let mut summary = Instant::now();
    let mut query = Instant::now();
    loop {
        if matches!(stop.wait(None, 0)?, Wake::Stop) {
            return Ok(SessionEnd::Stop);
        }
        // Bounded batches prevent a continuous native queue from starving stop,
        // expiry, health requests, or periodic journal flushing.
        for _ in 0..128 {
            if matches!(stop.wait(None, 0)?, Wake::Stop) {
                return Ok(SessionEnd::Stop);
            }
            let packet = match connection.next() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    counters.empty_dispatch += 1;
                    if connection.dispatch_hresult() < 0 {
                        counters.efail_dispatch += 1;
                    }
                    break;
                }
                Err(error) => {
                    journal.record("native_error", &error, true)?;
                    // Decoder/ABI errors are fatal, never retried as routine disconnects.
                    if error.operation == "SimConnect_GetNextDispatch" && error.hresult.is_some() {
                        return Ok(SessionEnd::Reconnect);
                    }
                    return Err(error.into());
                }
            };
            counters.packets += 1;
            last_packet = Instant::now();
            let is_attitude = matches!(packet.message, Message::Attitude { .. });
            journal.record("dispatch", &packet, !is_attitude)?;
            match packet.message {
                Message::Quit => {
                    journal.record(
                        "connection_lost",
                        json!({ "reason": "SIMCONNECT_RECV_QUIT" }),
                        true,
                    )?;
                    return Ok(SessionEnd::Reconnect);
                }
                Message::Exception { .. } => {
                    return Err(ProbeError::Native(
                        "Asynchronous SimConnect exception; inspect send_id and api_result records"
                            .into(),
                    ));
                }
                Message::Event {
                    event_id: lifecycle::EVENT_SIM,
                    data,
                } => lifecycle.sim(data != 0),
                Message::Event {
                    event_id: lifecycle::EVENT_PAUSE,
                    data,
                } => lifecycle.pause(data),
                Message::Filename {
                    event_id: lifecycle::EVENT_FLIGHT,
                    filename,
                    ..
                } => lifecycle.flight(&filename, true),
                Message::SystemState {
                    request_id,
                    integer,
                    float,
                    string,
                } => {
                    journal.record(
                        "system_state_float",
                        json!({ "request_id": request_id, "raw": number(f64::from(float)),
                            "native_f32_bits_hex": format!("{:08x}", float.to_bits()) }),
                        false,
                    )?;
                    match request_id {
                        lifecycle::STATE_SIM => lifecycle.sim(integer != 0),
                        lifecycle::STATE_FLIGHT => lifecycle.flight(&string, false),
                        _ => {}
                    }
                }
                Message::Attitude { values, .. } => {
                    let now = Instant::now();
                    let candidate = candidate_attitude(values);
                    journal.record("attitude_sample", json!({
                        "raw": { "pitch_radians": number(values[0]), "pitch_degrees": number(values[1]),
                            "bank_radians": number(values[2]), "bank_degrees": number(values[3]) },
                        "interval_us": last_sample.map(|at| now.saturating_duration_since(at).as_micros().to_string()),
                        "candidate_attitude": candidate.as_ref().ok(), "candidate_is_hypothesis": true,
                        "validation_error": candidate.as_ref().err().map(ToString::to_string),
                    }), false)?;
                    last_sample = Some(now);
                    lifecycle.sample(candidate, now);
                }
                _ => {}
            }
            output.update(lifecycle, journal, Instant::now())?;
        }
        let now = Instant::now();
        output.update(lifecycle, journal, now)?;
        if summary.elapsed() >= Duration::from_secs(1) {
            journal.record(
                "dispatch_summary",
                json!({ "counters": counters,
                "last_dispatch_hresult": format!("0x{:08X}", connection.dispatch_hresult() as u32),
                "silence_ms": last_packet.elapsed().as_millis().to_string() }),
                true,
            )?;
            counters = Counters::default();
            summary = now;
        }
        if query.elapsed() >= Duration::from_secs(5) {
            match query_states(connection, journal) {
                Ok(()) => query = now,
                Err(ProbeError::Native(_)) => return Ok(SessionEnd::Reconnect),
                Err(error) => return Err(error),
            }
        }
        if last_packet.elapsed() >= Duration::from_secs(15) {
            journal.record(
                "connection_lost",
                json!({ "reason": "diagnostic_watchdog_no_packets_for_15_seconds",
                "interpretation_verified": false }),
                true,
            )?;
            return Ok(SessionEnd::Reconnect);
        }
        match stop.wait(Some(notification), 50)? {
            Wake::Stop => return Ok(SessionEnd::Stop),
            Wake::Notification => counters.notification_wakes += 1,
            Wake::Timeout => counters.timeout_wakes += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reconnect_backoff_is_bounded_and_cannot_overflow() {
        assert_eq!(
            (0..7).map(retry_seconds).collect::<Vec<_>>(),
            [1, 2, 4, 8, 10, 10, 10]
        );
        assert_eq!(retry_seconds(u32::MAX), 10);
    }
    #[tokio::test]
    async fn invalid_configuration_does_not_load_native_code() {
        let error = run_probe(
            ProbeConfig {
                dll: "relative.dll".into(),
                scenario: "level".into(),
                log_dir: ".local/probe-tests".into(),
            },
            std::future::pending(),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, ProbeError::Configuration(_)));
    }

    #[tokio::test]
    async fn missing_dll_records_failure_and_joins_worker_without_demo_fallback() {
        let directory = std::env::temp_dir().join(format!(
            "simconnect-worker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = run_probe(
            ProbeConfig {
                dll: directory.join("missing.dll"),
                scenario: "missing-dll".into(),
                log_dir: directory.clone(),
            },
            std::future::pending(),
        )
        .await;
        assert!(matches!(result, Err(ProbeError::Native(_))));
        let path = std::fs::read_dir(&directory)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let records: Vec<Value> = std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(
            records
                .iter()
                .any(|record| record["kind"] == "native_error")
        );
        assert!(
            records
                .iter()
                .all(|record| record["kind"] != "connection_attempt")
        );
        let ending = records.last().unwrap();
        assert_eq!(ending["kind"], "session_end");
        assert_eq!(ending["data"]["success"], false);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
