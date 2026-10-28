use super::{
    event::{Event, Wake},
    journal::{DiagnosticStatus, Journal, fingerprint, number},
    lifecycle::{self, Lifecycle, candidate_attitude, state_name},
    native::{Connection, Library, Message, NativeError},
    unit_check,
};
use crate::{
    Source,
    telemetry::{self, Publisher},
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

/// Acquisition options; omission selects SimConnect.dll beside the running executable.
#[derive(Debug, Default)]
pub struct SimConnectConfig {
    pub dll: Option<PathBuf>,
    pub log: Option<PathBuf>,
}

/// Acquisition and diagnostic outcomes remain independent.
#[derive(Debug)]
pub struct RunReport {
    pub result: Result<(), SimConnectError>,
    pub diagnostics: DiagnosticStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum SimConnectError {
    #[error("Invalid SimConnect configuration: {0}")]
    Configuration(String),
    #[error("SimConnect I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("SimConnect error: {0}")]
    Native(String),
    #[error("Telemetry error: {0}")]
    Telemetry(#[from] telemetry::TelemetryError),
    #[error("SimConnect worker panicked")]
    WorkerPanic,
}

impl From<NativeError> for SimConnectError {
    fn from(error: NativeError) -> Self {
        Self::Native(error.to_string())
    }
}

/// Resolve one DLL path without searching PATH, the working directory, or SDK installs.
pub fn resolve_dll(override_path: Option<&Path>) -> Result<PathBuf, SimConnectError> {
    if let Some(path) = override_path {
        return explicit_dll(path);
    }
    let executable = std::env::current_exe().map_err(|error| {
        SimConnectError::Configuration(format!(
            "cannot locate the running executable: {error}; specify an absolute --dll path"
        ))
    })?;
    adjacent_dll(&executable)
}

fn explicit_dll(path: &Path) -> Result<PathBuf, SimConnectError> {
    if !path.is_absolute() {
        return Err(SimConnectError::Configuration(
            "--dll must be an absolute path".into(),
        ));
    }
    Ok(path.to_owned())
}

fn adjacent_dll(executable: &Path) -> Result<PathBuf, SimConnectError> {
    executable
        .parent()
        .map(|parent| parent.join("SimConnect.dll"))
        .ok_or_else(|| {
            SimConnectError::Configuration(
                "cannot locate executable directory; specify --dll".into(),
            )
        })
}

/// Acquire on one native worker into the caller's SimConnect channel; never start HTTP.
/// Caller owns signals. Cancellation signals the worker even if this future is dropped.
pub async fn run(
    config: SimConnectConfig,
    publisher: Publisher,
    shutdown: impl Future<Output = io::Result<()>>,
) -> RunReport {
    let early_error = |error| RunReport {
        result: Err(error),
        diagnostics: DiagnosticStatus::Disabled,
    };
    if publisher.source() != Source::SimConnect {
        return early_error(SimConnectError::Configuration(
            "publisher must identify the SimConnect source".into(),
        ));
    }
    let dll = match resolve_dll(config.dll.as_deref()) {
        Ok(dll) => dll,
        Err(error) => return early_error(error),
    };
    println!("SimConnect DLL: {}", dll.display());
    println!(
        "Live telemetry requires an active, unpaused flight and fresh valid samples. Press Ctrl+C to stop."
    );
    let stop = match Event::new(true) {
        Ok(event) => Arc::new(event),
        Err(error) => return early_error(error.into()),
    };
    let worker_stop = Arc::clone(&stop);
    let log_path = config.log.clone();
    let worker = match thread::Builder::new()
        .name("simconnect".into())
        .spawn(move || worker_entry(dll, config.log, publisher, &worker_stop))
    {
        Ok(worker) => worker,
        Err(error) => return early_error(error.into()),
    };
    let cancellation = Cancellation(Arc::clone(&stop));
    tokio::pin!(shutdown);
    let signal_result = loop {
        if worker.is_finished() {
            break Ok(());
        }
        tokio::select! {
            result = &mut shutdown => {
                let signalled = stop.signal();
                break result.and(signalled);
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    };
    while !worker.is_finished() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut report = match worker.join() {
        Ok(report) => report,
        Err(_) => RunReport {
            result: Err(SimConnectError::WorkerPanic),
            diagnostics: log_path.map_or(DiagnosticStatus::Disabled, |path| {
                DiagnosticStatus::Failed {
                    path,
                    error: "worker panicked; diagnostic completion unconfirmed".into(),
                }
            }),
        },
    };
    drop(cancellation);
    if report.result.is_ok() {
        report.result = signal_result.map_err(Into::into);
    }
    report
}

struct Cancellation(Arc<Event>);
impl Drop for Cancellation {
    fn drop(&mut self) {
        let _ = self.0.signal();
    }
}

fn worker_entry(
    dll: PathBuf,
    log: Option<PathBuf>,
    publisher: Publisher,
    stop: &Event,
) -> RunReport {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return RunReport {
                result: Err(error.into()),
                diagnostics: DiagnosticStatus::Disabled,
            };
        }
    };
    let mut journal = match Journal::open(log.as_deref(), &[&dll, &executable]) {
        Ok(journal) => journal,
        Err(error) => {
            return RunReport {
                diagnostics: log.map_or(DiagnosticStatus::Disabled, |path| {
                    DiagnosticStatus::Failed {
                        path,
                        error: error.to_string(),
                    }
                }),
                result: Err(error.into()),
            };
        }
    };
    if let Some(path) = &log {
        println!("Session log: {}", path.display());
    }
    let result = worker(&dll, publisher, &mut journal, stop);
    journal.record(
        "session_end",
        || {
            json!({
                "success": result.is_ok(), "error": result.as_ref().err().map(ToString::to_string),
            })
        },
        true,
    );
    journal.finish();
    RunReport {
        result,
        diagnostics: journal.status(),
    }
}

fn identity(path: &Path) -> Value {
    fingerprint(path)
        .unwrap_or_else(|error| json!({ "path": path, "unavailable": error.to_string() }))
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

fn worker(
    dll: &Path,
    publisher: Publisher,
    journal: &mut Journal,
    stop: &Event,
) -> Result<(), SimConnectError> {
    let executable = std::env::current_exe()?;
    journal.record("environment", || json!({
        "package_version": env!("CARGO_PKG_VERSION"), "target": "x86_64-pc-windows-msvc",
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "current_directory": std::env::current_dir().ok(), "executable": identity(&executable),
        "rustc_at_run": command_record("rustc", &["-Vv"]),
        "git_head_at_run": command_record("git", &["rev-parse", "HEAD"]),
        "git_changes_at_run": command_record("git", &["status", "--porcelain", "--untracked-files=no"]),
        "windows": command_record("cmd", &["/d", "/c", "ver"]),
        "sdk": sdk_version(dll), "dll": dll,
        "base_requested_fields": ["pitch radians", "bank radians"],
        "angular_input": "native radians; convert to degrees locally",
        "unit_probe_profile": "SDK degrees and repeated radians are diagnostics only; six fields if logging is enabled at connection open",
        "sign_convention": "positive nose up and right wing down; native pitch and bank negated",
        "note": "runtime toolchain/git metadata is contextual; executable SHA-256 identifies the tested bytes",
    }), true);
    journal.record("dll_identity", || identity(dll), true);
    journal.record("library_load_attempt", || json!({ "path": dll }), true);
    let library = Library::load(dll).map_err(|error| {
        journal.record("native_error", || json!(error), true);
        SimConnectError::Native(format!(
            "{error}; selected DLL: {}; use --dll to select another absolute path",
            dll.display()
        ))
    })?;
    journal.record("library_loaded", || json!({}), true);
    let result = connections(&library, publisher, journal, stop);
    journal.record(
        "library_release",
        || {
            json!({ "after_all_connections_closed": result.is_ok(),
        "note": "a failed native close retains the mapping; see native_error records" })
        },
        true,
    );
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

fn connections(
    library: &Library,
    publisher: Publisher,
    journal: &mut Journal,
    stop: &Event,
) -> Result<(), SimConnectError> {
    let mut output = Output {
        publisher,
        transition: None,
        instruments: Default::default(),
    };
    let mut lifecycle = Lifecycle::default();
    output.update(&lifecycle, journal, Instant::now(), None)?;
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
            || json!({ "generation": generation }),
            true,
        );
        let mut connection = match library.open(notification.event().handle(), journal.enabled()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("SimConnect connection unavailable: {error}");
                journal.record("native_error", || json!(&error), true);
                let seconds = retry_seconds(failures);
                failures = failures.saturating_add(1);
                journal.record(
                    "reconnect_wait",
                    || json!({ "seconds": seconds, "generation": generation }),
                    true,
                );
                if matches!(stop.wait(None, seconds * 1000)?, Wake::Stop) {
                    return Ok(());
                }
                continue;
            }
        };
        let session_start = Instant::now();
        let result = (|| {
            journal.record("api_result", || json!({ "operation": "SimConnect_Open", "success": true, "generation": generation }), true);
            println!("SimConnect connected (session {generation}).");
            lifecycle.connect();
            output.update(&lifecycle, journal, Instant::now(), None)?;
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
        match &closed {
            Ok(()) => journal.record(
                "api_result",
                || json!({ "operation": "SimConnect_Close", "success": true }),
                true,
            ),
            Err(error) => journal.record("native_error", || json!(error), true),
        };
        closed?;
        journal.record(
            "notification_release",
            || json!({ "generation": generation }),
            true,
        );
        drop(notification);
        lifecycle.disconnect();
        output.update(&lifecycle, journal, Instant::now(), None)?;
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
            || json!({ "seconds": seconds, "generation": generation }),
            true,
        );
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
    instruments: super::instruments::Instruments,
    publisher: Publisher,
    transition: Option<(&'static str, &'static str)>,
}
impl Output {
    fn update(
        &mut self,
        lifecycle: &Lifecycle,
        journal: &mut Journal,
        now: Instant,
        received_at: Option<Instant>,
    ) -> Result<(), SimConnectError> {
        if !lifecycle.active() {
            self.instruments.clear();
        }
        self.instruments.expire(now);
        let evaluation = lifecycle.evaluate(now);
        let transition = (state_name(evaluation.state), evaluation.reason);
        let changed = self.transition != Some(transition);
        match evaluation.state {
            // Publish each new acquisition, including equal angles. Timer ticks
            // and lifecycle queries must never republish a cached live sample.
            telemetry::State::Live(attitude) => {
                if (received_at.is_some() || self.instruments.dirty)
                    && let Some(at) = lifecycle.sample_time()
                {
                    self.publisher
                        .publish_sample_at(self.instruments.sample(attitude), at.into())?;
                }
            }
            state if changed => self.publisher.publish(state)?,
            _ => {}
        }
        self.instruments.dirty = false;
        if changed {
            journal.record(
                "state_transition",
                || {
                    json!({ "candidate_state": transition.0,
                "candidate_is_hypothesis": false, "published": self.publisher.snapshot(),
                "reason": evaluation.reason,
                "note": "live publication requires a new valid sample from an active, unpaused flight" })
                },
                true,
            );
            println!(
                "State: {} (candidate: {}, {})",
                transition.0, transition.0, transition.1
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
    context: impl FnOnce() -> Value,
) -> Result<(), SimConnectError> {
    match result {
        Ok(()) => {
            if !journal.enabled() {
                return Ok(());
            }
            let sent = connection.last_sent_packet_id();
            journal.record(
                "api_result",
                || {
                    json!({ "operation": operation, "success": true,
                "send_id": sent.as_ref().ok(), "context": context() })
                },
                false,
            );
            match sent {
                Ok(id) => journal.record("api_result", || json!({ "operation": "SimConnect_GetLastSentPacketID", "success": true, "send_id": id }), false),
                Err(error) => journal.record("native_error", || json!(error), true),
            }
            Ok(())
        }
        Err(error) => {
            journal.record("native_error", || json!(&error), true);
            Err(error.into())
        }
    }
}

fn configure(
    connection: &mut Connection<'_>,
    journal: &mut Journal,
) -> Result<(), SimConnectError> {
    let field_count = connection.attitude_field_count();
    journal.record(
        "attitude_definition",
        || {
            json!({
                "field_count": field_count,
                "fields": if field_count == 2 { &[
                    "pitch radians", "bank radians",
                ][..] } else { &[
                    "pitch radians", "pitch degrees", "bank radians", "bank degrees",
                    "pitch radians repeated", "bank radians repeated",
                ][..] },
                "angular_input": "native radians; degree probes never determine sample validity",
                "atomicity_verified": false,
            })
        },
        true,
    );
    for index in 0..field_count {
        let result = connection.add_attitude_field(index);
        api_result(
            connection,
            journal,
            "SimConnect_AddToDataDefinition",
            result,
            || json!({ "field_index": index }),
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
            || json!({ "event_id": id, "name": name.to_string_lossy() }),
        )?;
    }
    query_states(connection, journal)?;
    let result = connection.request_attitude();
    api_result(
        connection,
        journal,
        "SimConnect_RequestDataOnSimObject",
        result,
        || json!({ "period": "SIM_FRAME", "flags": 0, "object_id": 0 }),
    )
}

fn query_states(
    connection: &mut Connection<'_>,
    journal: &mut Journal,
) -> Result<(), SimConnectError> {
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
) -> Result<(), SimConnectError> {
    let result = connection.request_state(id, name);
    api_result(
        connection,
        journal,
        "SimConnect_RequestSystemState",
        result,
        || json!({ "request_id": id, "name": name.to_string_lossy() }),
    )
}

#[derive(Default, serde::Serialize)]
struct Counters {
    packets: u64,
    empty_dispatch: u64,
    efail_dispatch: u64,
    notification_wakes: u64,
    timeout_wakes: u64,
    unit_pair_mismatches: u64,
    repeated_radian_changes: u64,
    repeated_nonfinite: u64,
}

fn session(
    connection: &mut Connection<'_>,
    notification: &Event,
    stop: &Event,
    journal: &mut Journal,
    lifecycle: &mut Lifecycle,
    output: &mut Output,
) -> Result<SessionEnd, SimConnectError> {
    configure(connection, journal)?;
    output.instruments.clear();
    let mut optional_packets = Vec::new();
    for (index, (name, units)) in super::native::INSTRUMENT_FIELDS.iter().enumerate() {
        let added = connection.add_instrument(index);
        if added.is_ok()
            && let Ok(id) = connection.last_sent_packet_id()
        {
            optional_packets.push((id, index));
        }
        let requested = if added.is_ok() {
            connection.request_instrument(index)
        } else {
            added
        };
        if requested.is_ok()
            && let Ok(id) = connection.last_sent_packet_id()
        {
            optional_packets.push((id, index));
        }
        journal.record(
            "instrument_request",
            || {
                json!({"index":index, "name":name.to_string_lossy(),
            "units":units.to_string_lossy(), "error":requested.err().map(|e| e.to_string())})
            },
            true,
        );
    }
    let mut counters = Counters::default();
    let mut last_packet = Instant::now();
    let mut last_sample: Option<Instant> = None;
    let mut sample_index = 0_u64;
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
                    journal.record("native_error", || json!(&error), true);
                    // Decoder/ABI errors are fatal, never retried as routine disconnects.
                    if error.operation == "SimConnect_GetNextDispatch" && error.hresult.is_some() {
                        return Ok(SessionEnd::Reconnect);
                    }
                    return Err(error.into());
                }
            };
            let packet_received_at = Instant::now();
            counters.packets += 1;
            last_packet = Instant::now();
            let is_attitude = matches!(packet.message, Message::Attitude { .. });
            let mut received_at = None;
            journal.record("dispatch", || json!(&packet), !is_attitude);
            match packet.message {
                Message::Quit => {
                    journal.record(
                        "connection_lost",
                        || json!({ "reason": "SIMCONNECT_RECV_QUIT" }),
                        true,
                    );
                    return Ok(SessionEnd::Reconnect);
                }
                Message::Exception {
                    exception,
                    send_id,
                    index,
                } => {
                    if let Some((_, field)) = optional_packets.iter().find(|(id, _)| *id == send_id)
                    {
                        output.instruments.remove(*field);
                        journal.record(
                            "instrument_unavailable",
                            || json!({"index":field,"exception":exception,"send_id":send_id}),
                            true,
                        );
                        continue;
                    }
                    return Err(SimConnectError::Native(format!(
                        "Asynchronous SimConnect exception {exception}, send_id {send_id}, index {index}"
                    )));
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
                } => {
                    output.instruments.clear();
                    lifecycle.flight(&filename, true);
                }
                Message::SystemState {
                    request_id,
                    integer,
                    float,
                    string,
                } => {
                    journal.record(
                        "system_state_float",
                        || {
                            json!({ "request_id": request_id, "raw": number(f64::from(float)),
                            "native_f32_bits_hex": format!("{:08x}", float.to_bits()) })
                        },
                        false,
                    );
                    match request_id {
                        lifecycle::STATE_SIM => lifecycle.sim(integer != 0),
                        lifecycle::STATE_FLIGHT => {
                            if lifecycle.flight_id() != Some(string.as_str()) {
                                output.instruments.clear();
                            }
                            lifecycle.flight(&string, false);
                        }
                        _ => {}
                    }
                }
                Message::Instrument { index, value } => {
                    journal.record(
                        "instrument_sample",
                        || {
                            json!({"index":index,
                        "name":super::native::INSTRUMENT_FIELDS[index].0.to_string_lossy(),
                        "value":number(value),"active":lifecycle.active()})
                        },
                        false,
                    );
                    if lifecycle.active() {
                        output.instruments.receive(index, value, packet_received_at);
                    }
                }
                Message::Attitude {
                    radians,
                    unit_probe,
                    ..
                } => {
                    let now = packet_received_at;
                    sample_index += 1;
                    journal.record("attitude_raw", || json!({
                        "sample_index": sample_index,
                        "raw": { "pitch_radians": number(radians[0]),
                            "pitch_degrees": unit_probe.as_ref().map(|p| number(p.degrees[0])),
                            "bank_radians": number(radians[1]),
                            "bank_degrees": unit_probe.as_ref().map(|p| number(p.degrees[1])) },
                        "repeated_radians": unit_probe.as_ref().map(|p| json!({
                            "pitch": number(p.repeated_radians[0]), "bank": number(p.repeated_radians[1]),
                        })),
                        "interval_us": last_sample.map(|at| now.saturating_duration_since(at).as_micros().to_string()),
                    }), false);
                    if journal.enabled()
                        && let Some(probe) = &unit_probe
                    {
                        let values = [radians[0], probe.degrees[0], radians[1], probe.degrees[1]];
                        let repeated_radians = Some(probe.repeated_radians);
                        let observation = unit_check::observe(values, repeated_radians);
                        counters.unit_pair_mismatches += u64::from(observation.primary_mismatch);
                        counters.repeated_radian_changes += u64::from(observation.repeats_changed);
                        counters.repeated_nonfinite += u64::from(observation.repeats_nonfinite);
                        if observation.anomalous() {
                            journal.record(
                                "attitude_unit_check",
                                || {
                                    json!({
                                        "sample_index": sample_index,
                                        "affects_sample_validity": false,
                                        "comparison": unit_check::details(values, repeated_radians),
                                    })
                                },
                                true,
                            );
                        }
                    }
                    let candidate = candidate_attitude(radians);
                    journal.record("attitude_sample", || json!({
                        "sample_index": sample_index,
                        "candidate_attitude": candidate.as_ref().ok(), "candidate_is_hypothesis": false,
                        "validation_error": candidate.as_ref().err().map(ToString::to_string),
                    }), false);
                    last_sample = Some(now);
                    lifecycle.sample(candidate, now);
                    received_at = Some(now);
                }
                _ => {}
            }
            output.update(lifecycle, journal, Instant::now(), received_at)?;
        }
        let now = Instant::now();
        output.update(lifecycle, journal, now, None)?;
        if summary.elapsed() >= Duration::from_secs(1) {
            journal.record(
                "dispatch_summary",
                || {
                    json!({ "counters": counters,
                "last_dispatch_hresult": format!("0x{:08X}", connection.dispatch_hresult() as u32),
                "silence_ms": last_packet.elapsed().as_millis().to_string() })
                },
                true,
            );
            counters = Counters::default();
            summary = now;
        }
        if query.elapsed() >= Duration::from_secs(5) {
            match query_states(connection, journal) {
                Ok(()) => query = now,
                Err(SimConnectError::Native(_)) => return Ok(SessionEnd::Reconnect),
                Err(error) => return Err(error),
            }
        }
        if last_packet.elapsed() >= Duration::from_secs(15) {
            journal.record(
                "connection_lost",
                || {
                    json!(
                        json!({ "reason": "diagnostic_watchdog_no_packets_for_15_seconds",
                "interpretation_verified": false })
                    )
                },
                true,
            );
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

    fn missing_dll() -> PathBuf {
        std::env::temp_dir().join(format!(
            "missing-simconnect-{}-{}.dll",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn dll_selection_uses_executable_directory_and_never_requires_existing_override() {
        let executable = Path::new(r"C:\application\horizon.exe");
        assert_eq!(
            adjacent_dll(executable).unwrap(),
            Path::new(r"C:\application\SimConnect.dll")
        );
        let explicit = Path::new(r"D:\runtime\chosen.dll");
        assert_eq!(resolve_dll(Some(explicit)).unwrap(), explicit);
        assert!(matches!(
            resolve_dll(Some(Path::new("relative.dll"))),
            Err(SimConnectError::Configuration(_))
        ));
        assert_eq!(
            resolve_dll(None).unwrap(),
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("SimConnect.dll")
        );
    }

    #[test]
    fn reconnect_backoff_is_bounded_and_cannot_overflow() {
        assert_eq!(
            (0..7).map(retry_seconds).collect::<Vec<_>>(),
            [1, 2, 4, 8, 10, 10, 10]
        );
        assert_eq!(retry_seconds(u32::MAX), 10);
    }

    #[tokio::test]
    async fn wrong_channel_source_is_rejected_before_files_or_native_code() {
        let (publisher, subscription) = telemetry::channel(Source::Demo);
        let report = run(
            SimConnectConfig::default(),
            publisher,
            std::future::pending(),
        )
        .await;
        assert!(matches!(
            report.result,
            Err(SimConnectError::Configuration(_))
        ));
        assert_eq!(report.diagnostics, DiagnosticStatus::Disabled);
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["sequence"],
            0
        );
    }

    #[test]
    fn logging_failure_does_not_stop_publication_to_the_callers_channel() {
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut journal = Journal::failing_for_test();
        let mut lifecycle = Lifecycle::default();
        output
            .update(&lifecycle, &mut journal, Instant::now(), None)
            .unwrap();
        assert!(matches!(journal.status(), DiagnosticStatus::Failed { .. }));
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["state"],
            "disconnected"
        );
        lifecycle.connect();
        output
            .update(&lifecycle, &mut journal, Instant::now(), None)
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["state"], "waiting");
        assert_eq!(snapshot["sequence"], 2);
        assert_eq!(snapshot["source"], "simconnect");
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("active.flt", true);
        let now = Instant::now();
        lifecycle.sample(Ok(telemetry::Attitude::new(5.0, -10.0).unwrap()), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["state"], "live");
        assert_eq!(snapshot["attitude"]["pitch_deg"], 5.0);
        assert!(matches!(journal.status(), DiagnosticStatus::Failed { .. }));
    }

    #[test]
    fn every_new_sample_publishes_but_queries_and_ticks_do_not_refresh_cached_values() {
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut journal = Journal::open(None, &[]).unwrap();
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("active.flt", true);
        let now = Instant::now();
        let first_at = now - Duration::from_millis(600);
        let first = telemetry::Attitude::new(5.0, -10.0).unwrap();
        lifecycle.sample(Ok(first), first_at);
        output
            .update(&lifecycle, &mut journal, now, Some(first_at))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["state"], "live");
        assert_eq!(snapshot["sequence"], 1);
        assert!(snapshot["sample_age_ms"].as_u64().unwrap() >= 600);

        // Repeated system state and freshness checks contain no new acquisition.
        lifecycle.sim(true);
        lifecycle.flight("active.flt", false);
        for _ in 0..3 {
            output.update(&lifecycle, &mut journal, now, None).unwrap();
        }
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["sequence"], 1);
        assert!(snapshot["sample_age_ms"].as_u64().unwrap() >= 600);

        let second_at = now - Duration::from_millis(400);
        let second = telemetry::Attitude::new(-7.0, 30.0).unwrap();
        lifecycle.sample(Ok(second), second_at);
        output
            .update(&lifecycle, &mut journal, now, Some(second_at))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["sequence"], 2);
        assert_eq!(snapshot["attitude"]["pitch_deg"], -7.0);
        assert_eq!(snapshot["attitude"]["roll_deg"], 30.0);
        assert!(snapshot["sample_age_ms"].as_u64().unwrap() >= 400);

        // Equal angles, even at the same clock resolution, are a new callback.
        lifecycle.sample(Ok(second), second_at);
        output
            .update(&lifecycle, &mut journal, now, Some(second_at))
            .unwrap();
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["sequence"],
            3
        );
        let equal_at = now - Duration::from_millis(200);
        lifecycle.sample(Ok(second), equal_at);
        output
            .update(&lifecycle, &mut journal, now, Some(equal_at))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["sequence"], 4);
        assert!(snapshot["sample_age_ms"].as_u64().unwrap() >= 200);

        // The lifecycle's independent tick expires data even without callbacks.
        output
            .update(
                &lifecycle,
                &mut journal,
                equal_at + telemetry::STALE_AFTER,
                None,
            )
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["state"], "stale");
        assert!(snapshot["attitude"].is_null());
        assert_eq!(snapshot["sequence"], 5);
    }

    #[test]
    fn pause_modes_menu_new_flight_and_reconnect_require_new_samples_before_live() {
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut journal = Journal::open(None, &[]).unwrap();
        let mut lifecycle = Lifecycle::default();
        let now = Instant::now();
        let attitude = telemetry::Attitude::new(3.0, 12.0).unwrap();
        let state = || serde_json::to_value(subscription.snapshot()).unwrap()["state"].clone();
        lifecycle.connect();
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        assert_eq!(state(), "waiting");
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("first.flt", true);
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert_eq!(state(), "live");
        for flags in [4, 8, 9, 0x8000_0000] {
            lifecycle.pause(flags);
            lifecycle.sample(Ok(attitude), now);
            output
                .update(&lifecycle, &mut journal, now, Some(now))
                .unwrap();
            assert_eq!(state(), "paused");
            assert!(serde_json::to_value(subscription.snapshot()).unwrap()["attitude"].is_null());
            lifecycle.pause(0);
            output.update(&lifecycle, &mut journal, now, None).unwrap();
            assert_eq!(state(), "waiting");
            lifecycle.sample(Ok(attitude), now);
            output
                .update(&lifecycle, &mut journal, now, Some(now))
                .unwrap();
            assert_eq!(state(), "live");
        }
        lifecycle.sim(false);
        lifecycle.flight("second.flt", true);
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert_eq!(state(), "waiting");
        lifecycle.sim(true);
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        assert_eq!(state(), "waiting");
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert_eq!(state(), "live");
        lifecycle.disconnect();
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        assert_eq!(state(), "disconnected");
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("second.flt", true);
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        assert_eq!(state(), "waiting");
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert_eq!(state(), "live");
    }

    #[test]
    fn invalid_or_already_expired_acquisitions_never_publish_live() {
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut journal = Journal::open(None, &[]).unwrap();
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("active.flt", true);
        let now = Instant::now();
        let expired = now - telemetry::STALE_AFTER;
        lifecycle.sample(Ok(telemetry::Attitude::new(1.0, 2.0).unwrap()), expired);
        output
            .update(&lifecycle, &mut journal, now, Some(expired))
            .unwrap();
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["state"],
            "stale"
        );
        lifecycle.sample(candidate_attitude([f64::NAN, 0.0]), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let snapshot = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(snapshot["state"], "invalid");
        assert!(snapshot["attitude"].is_null());
    }

    #[test]
    fn optional_expiry_does_not_renew_attitude_and_pause_clears_extensions() {
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut journal = Journal::open(None, &[]).unwrap();
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("test.flt", true);
        let now = Instant::now();
        output
            .instruments
            .receive(0, 120.0, now - Duration::from_millis(900));
        lifecycle.sample(Ok(telemetry::Attitude::new(0.0, 0.0).unwrap()), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["ias_kt"],
            120.0
        );
        output
            .update(
                &lifecycle,
                &mut journal,
                now + Duration::from_millis(200),
                None,
            )
            .unwrap();
        let state = serde_json::to_value(subscription.snapshot()).unwrap();
        assert!(state.get("ias_kt").is_none());
        assert_eq!(state["state"], "live");
        output.instruments.receive(0, 130.0, now);
        lifecycle.pause(4);
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        lifecycle.pause(0);
        lifecycle.sample(Ok(telemetry::Attitude::new(0.0, 0.0).unwrap()), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        assert!(
            serde_json::to_value(subscription.snapshot())
                .unwrap()
                .get("ias_kt")
                .is_none()
        );
    }

    #[test]
    fn failed_diagnostics_do_not_hide_a_native_failure() {
        let (publisher, _subscription) = telemetry::channel(Source::SimConnect);
        let mut journal = Journal::failing_for_test();
        let stop = Event::new(true).unwrap();
        let error = worker(&missing_dll(), publisher, &mut journal, &stop).unwrap_err();
        assert!(matches!(error, SimConnectError::Native(_)));
        assert!(matches!(journal.status(), DiagnosticStatus::Failed { .. }));
    }

    #[tokio::test]
    async fn missing_dll_keeps_a_complete_diagnostic_failure_session() {
        let dll = missing_dll();
        let log = dll.with_extension("jsonl");
        let (publisher, _subscription) = telemetry::channel(Source::SimConnect);
        let report = run(
            SimConnectConfig {
                dll: Some(dll),
                log: Some(log.clone()),
            },
            publisher,
            std::future::pending(),
        )
        .await;
        assert!(matches!(report.result, Err(SimConnectError::Native(_))));
        assert!(matches!(
            report.diagnostics,
            DiagnosticStatus::Completed { .. }
        ));
        let records: Vec<Value> = std::fs::read_to_string(&log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records[0]["data"]["schema_version"], 4);
        assert_eq!(records[0]["data"]["live_publication_enabled"], true);
        assert!(records[0]["data"].get("attitude_signs_verified").is_none());
        assert!(
            records
                .iter()
                .any(|record| record["kind"] == "native_error")
        );
        assert!(
            !records
                .iter()
                .any(|record| record["kind"] == "connection_attempt")
        );
        assert_eq!(records.last().unwrap()["kind"], "session_end");
        assert_eq!(records.last().unwrap()["data"]["success"], false);
        std::fs::remove_file(log).unwrap();
    }
}
