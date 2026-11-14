use super::{
    event::{Event, Wake},
    journal::{DiagnosticStatus, Journal, fingerprint, number},
    lifecycle::{self, Lifecycle, candidate_attitude, state_name},
    native::{Connection, Library, Message, NativeError},
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

/// Acquisition options; omission selects demo.dll beside the running executable.
#[derive(Debug, Default)]
pub struct SimConnectConfig {
    pub dll: Option<PathBuf>,
    pub log: Option<PathBuf>,
    pub scenario: Option<PathBuf>,
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
    #[error("SimConnect send-ID correlation failed: {0}")]
    Correlation(String),
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
pub fn resolve_dll(selected: Option<&Path>) -> Result<PathBuf, SimConnectError> {
    let path = selected.unwrap_or_else(|| Path::new("demo.dll"));
    let name = path.file_name().and_then(|v| v.to_str()).ok_or_else(|| {
        SimConnectError::Configuration("--source must name demo.dll or SimConnect.dll".into())
    })?;
    if !name.eq_ignore_ascii_case("demo.dll") && !name.eq_ignore_ascii_case("SimConnect.dll") {
        return Err(SimConnectError::Configuration(
            "--source must name demo.dll or SimConnect.dll".into(),
        ));
    }
    if path.is_absolute() {
        return Ok(path.to_owned());
    }
    if path.components().count() != 1 || path.as_os_str() != std::ffi::OsStr::new(name) {
        return Err(SimConnectError::Configuration(
            "--source must be a supported bare filename or an absolute path".into(),
        ));
    }
    let executable = std::env::current_exe()?;
    executable
        .parent()
        .map(|parent| parent.join(path))
        .ok_or_else(|| SimConnectError::Configuration("cannot locate executable directory".into()))
}
pub fn source_kind(path: &Path) -> Source {
    if path
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("demo.dll"))
    {
        Source::Demo
    } else {
        Source::SimConnect
    }
}

/// Validate the scenario selection for the chosen source.
pub fn validate_scenario(source: Source, scenario: Option<&Path>) -> Result<(), SimConnectError> {
    match (source, scenario) {
        (Source::Demo, None) => Err(SimConnectError::Configuration(
            "demo.dll requires --scenario <file>".into(),
        )),
        (Source::SimConnect, Some(_)) => Err(SimConnectError::Configuration(
            "--scenario requires demo.dll".into(),
        )),
        _ => Ok(()),
    }
}

pub fn load_definition() -> Result<super::Definition, SimConnectError> {
    let exe = std::env::current_exe()?;
    let directory = exe.parent().ok_or_else(|| {
        SimConnectError::Configuration("cannot locate executable directory".into())
    })?;
    super::Definition::load(&directory.join("default.json")).map_err(SimConnectError::Configuration)
}

pub fn validate_library(path: &Path) -> Result<(), SimConnectError> {
    Library::load(path).map(|_| ()).map_err(|error| {
        SimConnectError::Native(format!("{error}; selected DLL: {}", path.display()))
    })
}

/// Acquire on one native worker into the caller's SimConnect channel; never start HTTP.
/// Caller owns signals. Cancellation signals the worker even if this future is dropped.
pub async fn run(
    config: SimConnectConfig,
    publisher: Publisher,
    shutdown: impl Future<Output = io::Result<()>>,
) -> RunReport {
    let definition = match load_definition() {
        Ok(definition) => definition,
        Err(error) => {
            return RunReport {
                result: Err(error),
                diagnostics: DiagnosticStatus::Disabled,
            };
        }
    };
    run_with_definition(config, definition, publisher, shutdown).await
}

pub async fn run_with_definition(
    config: SimConnectConfig,
    definition: super::Definition,
    publisher: Publisher,
    shutdown: impl Future<Output = io::Result<()>>,
) -> RunReport {
    let early_error = |error| RunReport {
        result: Err(error),
        diagnostics: DiagnosticStatus::Disabled,
    };
    let dll = match resolve_dll(config.dll.as_deref()) {
        Ok(dll) => dll,
        Err(error) => return early_error(error),
    };
    let source = source_kind(&dll);
    if publisher.source() != source {
        return early_error(SimConnectError::Configuration(
            "publisher source must match selected DLL".into(),
        ));
    }
    if let Err(error) = validate_scenario(source, config.scenario.as_deref()) {
        return early_error(error);
    }
    println!("Selected DLL: {}", dll.display());
    println!(
        "Live telemetry requires an active, unpaused flight and fresh valid samples. Press Ctrl+C to stop."
    );
    let stop = match Event::new(true) {
        Ok(event) => Arc::new(event),
        Err(error) => return early_error(error.into()),
    };
    let worker_stop = Arc::clone(&stop);
    let log_path = config.log.clone();
    let scenario = match config.scenario {
        Some(path) if !path.is_absolute() => match std::env::current_dir() {
            Ok(cwd) => Some(cwd.join(path)),
            Err(e) => return early_error(e.into()),
        },
        other => other,
    };
    let worker = match thread::Builder::new()
        .name("simconnect".into())
        .spawn(move || {
            worker_entry(
                dll,
                config.log,
                scenario,
                source,
                publisher,
                &worker_stop,
                definition,
            )
        }) {
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
    scenario: Option<PathBuf>,
    source: Source,
    publisher: Publisher,
    stop: &Event,
    definition: super::Definition,
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
    let mapping_path = executable
        .parent()
        .expect("executable has a directory")
        .join("default.json");
    let mut protected = vec![dll.as_path(), executable.as_path(), mapping_path.as_path()];
    if let Some(path) = scenario.as_deref() {
        protected.push(path);
    }
    let mut journal = match Journal::open(log.as_deref(), &protected) {
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
    let result = worker(&dll, source, publisher, &mut journal, stop, &definition);
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
    source: Source,
    publisher: Publisher,
    journal: &mut Journal,
    stop: &Event,
    definition: &super::Definition,
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
        "sdk": if source == Source::SimConnect { sdk_version(dll) } else { json!({"synthetic": true}) }, "dll": dll, "source": source,
        "requested_fields": definition.mappings.iter().map(|mapping| &mapping.field_name).collect::<Vec<_>>(),
        "angular_input": "native radians; convert to degrees locally",
        "payload_profile": "one ordered untagged grouped definition",
        "sign_convention": "positive nose up and right wing down; native pitch and bank negated",
        "note": "runtime toolchain/git metadata is contextual; executable SHA-256 identifies the tested bytes",
    }), true);
    journal.record("dll_identity", || identity(dll), true);
    journal.record("library_load_attempt", || json!({ "path": dll }), true);
    let library = Library::load(dll).map_err(|error| {
        journal.record("native_error", || json!(error), true);
        SimConnectError::Native(format!(
            "{error}; selected DLL: {}; use --source to select another DLL",
            dll.display()
        ))
    })?;
    journal.record("library_loaded", || json!({}), true);
    let result = connections(&library, source, publisher, journal, stop, definition, dll);
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

fn exception_name(code: u32) -> Option<&'static str> {
    Some(match code {
        0 => "NONE",
        1 => "ERROR",
        2 => "SIZE_MISMATCH",
        3 => "UNRECOGNIZED_ID",
        4 => "UNOPENED",
        5 => "VERSION_MISMATCH",
        6 => "TOO_MANY_GROUPS",
        7 => "NAME_UNRECOGNIZED",
        8 => "TOO_MANY_EVENT_NAMES",
        9 => "EVENT_ID_DUPLICATE",
        10 => "TOO_MANY_MAPS",
        11 => "TOO_MANY_OBJECTS",
        12 => "TOO_MANY_REQUESTS",
        18 => "INVALID_DATA_TYPE",
        19 => "INVALID_DATA_SIZE",
        20 => "DATA_ERROR",
        21 => "INVALID_ARRAY",
        24 => "OPERATION_INVALID_FOR_OBJECT_TYPE",
        25 => "ILLEGAL_OPERATION",
        26 => "ALREADY_SUBSCRIBED",
        27 => "INVALID_ENUM",
        28 => "DEFINITION_ERROR",
        29 => "DUPLICATE_ID",
        30 => "DATUM_ID",
        31 => "OUT_OF_BOUNDS",
        44 => "EVENT_NAME_RESERVED",
        45 => "INTERNAL",
        _ => return None,
    })
}

fn exception_is_terminal(code: u32) -> bool {
    code != 0
}

fn connections(
    library: &Library,
    source: Source,
    publisher: Publisher,
    journal: &mut Journal,
    stop: &Event,
    definition: &super::Definition,
    dll: &Path,
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
        let mut connection = match library.open(notification.event().handle()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("SimConnect connection unavailable: {error}");
                journal.record("native_error", || json!(&error), true);
                if source == Source::Demo && error.hresult == Some(0xA004_0001u32 as i32) {
                    return Err(SimConnectError::Configuration(format!(
                        "Demo scenario configuration failed: {error}"
                    )));
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
                SessionContext {
                    source,
                    definition,
                    generation,
                    dll,
                },
            )
        })();
        // Invalidate the published state before native cleanup can block.
        lifecycle.disconnect();
        let publication = output.update(&lifecycle, journal, Instant::now(), None);
        if let Err(error) = &result {
            eprintln!("SimConnect acquisition failed in generation {generation}: {error}");
            journal.record(
                "acquisition_failure",
                || json!({"generation":generation,"error":error.to_string()}),
                true,
            );
        }
        if let Err(error) = &publication {
            eprintln!("SimConnect disconnect publication failed: {error}");
        }
        // Explicit cleanup is attempted even when publication or acquisition failed.
        let closed = connection.close();
        match &closed {
            Ok(()) => journal.record(
                "api_result",
                || json!({ "operation": "SimConnect_Close", "success": true }),
                true,
            ),
            Err(error) => journal.record("native_error", || json!(error), true),
        };
        if let Err(error) = &closed {
            eprintln!("SimConnect cleanup failed: {error}");
        }
        let session_end = result?;
        publication?;
        closed?;
        journal.record(
            "notification_release",
            || json!({ "generation": generation }),
            true,
        );
        drop(notification);
        match session_end {
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

#[derive(Debug)]
struct SentOperation {
    generation: u64,
    id: u32,
    operation: &'static str,
    mapping: Option<usize>,
    query_id: Option<u32>,
}

fn find_sent(sends: &[SentOperation], generation: u64, id: u32) -> Option<&SentOperation> {
    sends
        .iter()
        .find(|sent| sent.generation == generation && sent.id == id)
}

fn api_result(
    connection: &Connection<'_>,
    journal: &mut Journal,
    operation: &str,
    result: Result<(), NativeError>,
    context: impl FnOnce() -> Value,
) -> Result<u32, SimConnectError> {
    match result {
        Ok(()) => {
            let sent = connection.last_sent_packet_id().map_err(|error| {
                journal.record("native_error", || json!(&error), true);
                SimConnectError::Correlation(format!("{error}; cannot correlate {operation}"))
            })?;
            journal.record(
                "api_result",
                || {
                    json!({ "operation": operation, "success": true,
                "send_id": sent, "context": context() })
                },
                false,
            );
            Ok(sent)
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
    definition: &super::Definition,
    generation: u64,
) -> Result<Vec<SentOperation>, SimConnectError> {
    let mut sends = Vec::new();
    for (index, mapping) in definition.mappings.iter().enumerate() {
        let result = connection.add_mapping(mapping);
        if let Err(error) = result {
            journal.record("native_error", || json!({"generation":generation,"error":error,"field":mapping.field_name,"simvar":mapping.simvar.to_string_lossy(),"unit":mapping.unit.to_string_lossy(),"type":mapping.type_name}), true);
            return Err(SimConnectError::Native(format!(
                "{error}; generation {generation}, field={}, SimVar={}, unit={}, type={}",
                mapping.field_name,
                mapping.simvar.to_string_lossy(),
                mapping.unit.to_string_lossy(),
                mapping.type_name
            )));
        }
        let sent = connection.last_sent_packet_id().map_err(|error| {
            SimConnectError::Correlation(format!(
                "{error}; generation {generation}, cannot correlate registration for field {}",
                mapping.field_name
            ))
        })?;
        sends.push(SentOperation {
            generation,
            id: sent,
            operation: "SimConnect_AddToDataDefinition",
            mapping: Some(index),
            query_id: None,
        });
        journal.record("definition_registration", || json!({"generation":generation,"send_id":sent,"field":mapping.field_name,"simvar":mapping.simvar.to_string_lossy(),"unit":mapping.unit.to_string_lossy(),"type":mapping.type_name}), true);
    }
    for (id, name) in [
        (lifecycle::EVENT_SIM, c"Sim"),
        (lifecycle::EVENT_PAUSE, c"Pause_EX1"),
        (lifecycle::EVENT_FLIGHT, c"FlightLoaded"),
    ] {
        let result = connection.subscribe(id, name);
        let sent = api_result(
            connection,
            journal,
            "SimConnect_SubscribeToSystemEvent",
            result,
            || json!({ "event_id": id, "name": name.to_string_lossy() }),
        )?;
        sends.push(SentOperation {
            generation,
            id: sent,
            operation: "SimConnect_SubscribeToSystemEvent",
            mapping: None,
            query_id: None,
        });
    }
    query_states(connection, journal, &mut sends, generation)?;
    connection.request_attitude()?;
    let sent = connection.last_sent_packet_id().map_err(|error| {
        SimConnectError::Correlation(format!("{error}; cannot correlate grouped request"))
    })?;
    sends.push(SentOperation {
        generation,
        id: sent,
        operation: "SimConnect_RequestDataOnSimObject",
        mapping: None,
        query_id: None,
    });
    journal.record(
        "grouped_request",
        || json!({"generation":generation,"send_id":sent,"period":"SIM_FRAME","flags":0,"interval":0}),
        true,
    );
    Ok(sends)
}
fn query_states(
    connection: &mut Connection<'_>,
    journal: &mut Journal,
    sends: &mut Vec<SentOperation>,
    generation: u64,
) -> Result<(), SimConnectError> {
    for (id, name) in [
        (lifecycle::STATE_SIM, c"Sim"),
        (lifecycle::STATE_FLIGHT, c"FlightLoaded"),
    ] {
        let sent = request_state(connection, journal, id, name)?;
        sends.retain(|entry| entry.query_id != Some(id));
        sends.push(SentOperation {
            generation,
            id: sent,
            operation: "SimConnect_RequestSystemState",
            mapping: None,
            query_id: Some(id),
        });
    }
    Ok(())
}

fn request_state(
    connection: &mut Connection<'_>,
    journal: &mut Journal,
    id: u32,
    name: &CStr,
) -> Result<u32, SimConnectError> {
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
}

struct SessionContext<'a> {
    source: Source,
    definition: &'a super::Definition,
    generation: u64,
    dll: &'a Path,
}

fn session(
    connection: &mut Connection<'_>,
    notification: &Event,
    stop: &Event,
    journal: &mut Journal,
    lifecycle: &mut Lifecycle,
    output: &mut Output,
    context: SessionContext<'_>,
) -> Result<SessionEnd, SimConnectError> {
    let SessionContext {
        source,
        definition,
        generation,
        dll,
    } = context;
    let mut sends = configure(connection, journal, definition, generation)?;
    output.instruments.clear();
    let mut counters = Counters::default();
    let mut last_packet = Instant::now();
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
            let packet = match connection.next_grouped(definition) {
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
                    if error.operation == "SimConnect_GetNextDispatch"
                        && error.hresult.is_some()
                        && !(source == Source::Demo && error.hresult == Some(0xA004_0001u32 as i32))
                    {
                        return Ok(SessionEnd::Reconnect);
                    }
                    return Err(error.into());
                }
            };
            let packet_received_at = Instant::now();
            counters.packets += 1;
            last_packet = Instant::now();
            let is_attitude = matches!(packet.message, Message::Grouped { .. });
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
                    if !exception_is_terminal(exception) {
                        continue;
                    }
                    let sent = find_sent(&sends, generation, send_id);
                    let mapping =
                        sent.and_then(|sent| sent.mapping.map(|entry| &definition.mappings[entry]));
                    let operation = sent
                        .map(|sent| sent.operation)
                        .unwrap_or("unknown operation");
                    let context = mapping
                        .map(|m| {
                            format!(
                                "field={}, SimVar={}, unit={}, type={}",
                                m.field_name,
                                m.simvar.to_string_lossy(),
                                m.unit.to_string_lossy(),
                                m.type_name
                            )
                        })
                        .unwrap_or_else(|| "no correlated mapping".into());
                    let compatibility = if exception == 5 {
                        format!(
                            "; DLL identity: {}; SDK metadata: {}",
                            identity(dll),
                            sdk_version(dll)
                        )
                    } else {
                        String::new()
                    };
                    journal.record("native_exception", || json!({"generation":generation,"exception":exception,"name":exception_name(exception),"send_id":send_id,"parameter_index":index,"operation":operation,"mapping":context}), true);
                    return Err(SimConnectError::Native(format!(
                        "Asynchronous SimConnect exception {} ({exception}), generation {generation}, send_id {send_id}, parameter index {index}, operation {operation}; {context}{compatibility}",
                        exception_name(exception).unwrap_or("unknown")
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
                Message::Grouped { values } => {
                    let now = packet_received_at;
                    sample_index += 1;
                    let radians = [values[0].unwrap_or(f64::NAN), values[1].unwrap_or(f64::NAN)];
                    let candidate = candidate_attitude(radians);
                    journal.record("grouped_sample", || json!({"sample_index":sample_index,"values":values.iter().map(|v| v.map(number)).collect::<Vec<_>>(),"active":lifecycle.active(),"validation_error":candidate.as_ref().err().map(ToString::to_string)}), false);
                    if lifecycle.active() && candidate.is_ok() {
                        for (index, value) in values[2..].iter().enumerate() {
                            output
                                .instruments
                                .receive(index, value.unwrap_or(f64::NAN), now);
                        }
                    } else {
                        output.instruments.clear();
                    }
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
            match query_states(connection, journal, &mut sends, generation) {
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
    #[test]
    fn exception_policy_is_terminal_except_none() {
        for code in [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 18, 19, 20, 21, 24, 25, 26, 27, 28, 29, 30, 31,
            44, 45,
        ] {
            assert!(exception_is_terminal(code));
            assert!(exception_name(code).is_some(), "code {code}");
        }
        assert!(!exception_is_terminal(0));
        assert!(exception_is_terminal(u32::MAX));
        assert!(exception_name(u32::MAX).is_none());
    }
    #[test]
    fn correlation_checks_generation_and_preserves_operation() {
        let sends = vec![
            SentOperation {
                generation: 7,
                id: 11,
                operation: "SimConnect_AddToDataDefinition",
                mapping: Some(3),
                query_id: None,
            },
            SentOperation {
                generation: 7,
                id: 12,
                operation: "SimConnect_RequestDataOnSimObject",
                mapping: None,
                query_id: None,
            },
        ];
        let entry = find_sent(&sends, 7, 11).unwrap();
        assert_eq!(entry.mapping, Some(3));
        let request = find_sent(&sends, 7, 12).unwrap();
        assert_eq!(request.operation, "SimConnect_RequestDataOnSimObject");
        assert_eq!(request.mapping, None);
        assert!(find_sent(&sends, 8, 11).is_none());
        assert!(find_sent(&sends, 7, 99).is_none());
    }

    fn missing_dll() -> PathBuf {
        std::env::temp_dir()
            .join(format!("missing-simconnect-{}", std::process::id()))
            .join("SimConnect.dll")
    }
    #[test]
    fn dll_selection_uses_executable_directory_and_validates_basename() {
        let explicit = Path::new(r"D:\runtime\SimConnect.dll");
        assert_eq!(resolve_dll(Some(explicit)).unwrap(), explicit);
        for path in ["relative/demo.dll", "relative.dll", "demo"] {
            assert!(resolve_dll(Some(Path::new(path))).is_err());
        }
        assert_eq!(
            resolve_dll(None).unwrap(),
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("demo.dll")
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
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
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
    fn subscription_skips_ticks() {
        let mut journal = Journal::open(None, &[]).unwrap();
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut lifecycle = Lifecycle::default();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("test.flt", true);
        let now = Instant::now();
        lifecycle.sample(Ok(telemetry::Attitude::new(5.0, 2.0).unwrap()), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let actual = serde_json::to_value(subscription.snapshot()).unwrap();
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        journal.finish();
        assert_eq!(actual["state"], "live");
        assert_eq!(
            serde_json::to_value(subscription.snapshot()).unwrap()["sequence"],
            actual["sequence"]
        );
    }

    #[test]
    fn reconnect_and_optional_changes_reach_subscription() {
        let path = missing_dll().with_extension("jsonl");
        let mut journal = Journal::open(Some(&path), &[]).unwrap();
        let (publisher, subscription) = telemetry::channel(Source::SimConnect);
        let mut output = Output {
            publisher,
            transition: None,
            instruments: Default::default(),
        };
        let mut lifecycle = Lifecycle::default();
        let now = Instant::now();
        let attitude = telemetry::Attitude::new(5.0, 2.0).unwrap();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("first.flt", true);
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let first = serde_json::to_value(subscription.snapshot()).unwrap();
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let equal = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(equal["sequence"], first["sequence"].as_u64().unwrap() + 1);
        output
            .instruments
            .receive(0, f64::NAN, now - Duration::from_millis(900));
        output
            .instruments
            .receive(3, 1234.0, now - Duration::from_millis(900));
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        let optional = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(optional["state"], "live");
        assert!(optional.get("ias_kt").is_none());
        assert_eq!(optional["altitude_ft"], 1234.0);
        output
            .update(
                &lifecycle,
                &mut journal,
                now + Duration::from_millis(200),
                None,
            )
            .unwrap();
        let expired = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(expired["state"], "live");
        assert_eq!(expired["altitude_ft"], 1234.0);
        lifecycle.disconnect();
        output.update(&lifecycle, &mut journal, now, None).unwrap();
        let disconnected = serde_json::to_value(subscription.snapshot()).unwrap();
        lifecycle.connect();
        lifecycle.sim(true);
        lifecycle.pause(0);
        lifecycle.flight("second.flt", true);
        lifecycle.sample(Ok(attitude), now);
        output
            .update(&lifecycle, &mut journal, now, Some(now))
            .unwrap();
        let reconnected = serde_json::to_value(subscription.snapshot()).unwrap();
        journal.finish();
        assert_eq!(disconnected["state"], "disconnected");
        assert_eq!(reconnected["state"], "live");
        std::fs::remove_file(path).unwrap();
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
    fn sample_expiry_and_pause_clear_all_indications() {
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
        assert_eq!(state["ias_kt"], 120.0);
        assert_eq!(state["state"], "live");
        output
            .update(
                &lifecycle,
                &mut journal,
                now + Duration::from_millis(999),
                None,
            )
            .unwrap();
        let just_fresh = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(just_fresh["state"], "live");
        assert_eq!(just_fresh["ias_kt"], 120.0);
        output
            .update(&lifecycle, &mut journal, now + telemetry::STALE_AFTER, None)
            .unwrap();
        let stale = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(stale["state"], "stale");
        assert!(stale.get("ias_kt").is_none());
        output
            .update(
                &lifecycle,
                &mut journal,
                now + Duration::from_millis(1001),
                None,
            )
            .unwrap();
        let still_stale = serde_json::to_value(subscription.snapshot()).unwrap();
        assert_eq!(still_stale["state"], "stale");
        assert!(still_stale.get("ias_kt").is_none());
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
        let error = worker(
            &missing_dll(),
            Source::SimConnect,
            publisher,
            &mut journal,
            &stop,
            &super::super::mapping::Definition::parse(include_str!("../../../default.json"))
                .unwrap(),
        )
        .unwrap_err();
        assert!(matches!(error, SimConnectError::Native(_)));
        assert!(matches!(journal.status(), DiagnosticStatus::Failed { .. }));
    }

    #[tokio::test]
    async fn missing_dll_keeps_a_complete_diagnostic_failure_session() {
        let dll = missing_dll();
        let log =
            std::env::temp_dir().join(format!("missing-simconnect-{}.jsonl", std::process::id()));
        let (publisher, _subscription) = telemetry::channel(Source::SimConnect);
        let report = run_with_definition(
            SimConnectConfig {
                dll: Some(dll),
                log: Some(log.clone()),
                scenario: None,
            },
            super::super::mapping::Definition::parse(include_str!("../../../default.json"))
                .unwrap(),
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
        assert_eq!(records[0]["data"]["schema_version"], 6);
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
