//! Process-boundary checks remain independent of a simulator and Microsoft SDK.
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "simconnect-cli-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn binary(&self) -> PathBuf {
        let binary = self.0.join("app.exe");
        let original = Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"));
        fs::copy(original, &binary).unwrap();
        fs::copy(
            original.parent().unwrap().join("demo.dll"),
            self.0.join("demo.dll"),
        )
        .unwrap();
        binary
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct RunningChild(Child);
impl Drop for RunningChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn output(binary: &Path, cwd: &Path, arguments: &[&str]) -> Output {
    let mut child = RunningChild(
        Command::new(binary)
            .current_dir(cwd)
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            child
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_end(&mut stdout)
                .unwrap();
            child
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_end(&mut stderr)
                .unwrap();
            return Output {
                status,
                stdout,
                stderr,
            };
        }
        assert!(Instant::now() < deadline, "CLI did not exit promptly");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Cleanup uses process termination only as a test fallback. These process tests
/// establish continued service after a source fault, not graceful shutdown.
struct Application {
    child: RunningChild,
    address: std::net::SocketAddr,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl Application {
    fn start(binary: &Path, cwd: &Path, arguments: &[&str]) -> Self {
        let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reservation.local_addr().unwrap();
        drop(reservation);
        let tag = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let stdout = binary.parent().unwrap().join(format!("stdout-{tag}.txt"));
        let stderr = binary.parent().unwrap().join(format!("stderr-{tag}.txt"));
        let child = RunningChild(
            Command::new(binary)
                .current_dir(cwd)
                .args(arguments)
                .args([
                    "--listen-address",
                    "127.0.0.1",
                    "--port",
                    &address.port().to_string(),
                ])
                .stdout(fs::File::create(&stdout).unwrap())
                .stderr(fs::File::create(&stderr).unwrap())
                .spawn()
                .unwrap(),
        );
        let mut app = Self {
            child,
            address,
            stdout,
            stderr,
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
                break;
            }
            assert!(
                app.child.0.try_wait().unwrap().is_none(),
                "app exited: {}",
                app.messages()
            );
            assert!(
                Instant::now() < deadline,
                "HTTP did not start: {}",
                app.messages()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        app
    }
    fn messages(&self) -> String {
        format!(
            "{}{}",
            fs::read_to_string(&self.stdout).unwrap_or_default(),
            fs::read_to_string(&self.stderr).unwrap_or_default()
        )
    }
    fn request(&self, path: &str) -> TcpStream {
        let mut socket = TcpStream::connect_timeout(&self.address, Duration::from_secs(2)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            socket,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        socket
    }
    fn assert_health(&mut self) {
        assert!(self.child.0.try_wait().unwrap().is_none());
        let mut response = String::new();
        self.request("/health")
            .read_to_string(&mut response)
            .unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("{\"status\":\"ok\"}"), "{response}");
    }
    fn unavailable_stream(&self) -> TcpStream {
        self.unavailable_stream_for("simconnect")
    }
    fn unavailable_stream_for(&self, source: &str) -> TcpStream {
        let mut socket = self.request("/api/v1/events");
        let mut response = String::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut bytes = [0; 4096];
            let count = socket.read(&mut bytes).expect("SSE must remain readable");
            assert!(count > 0, "SSE closed after source failure: {response}");
            response.push_str(&String::from_utf8_lossy(&bytes[..count]));
            for line in response.lines() {
                if let Some(json) = line.strip_prefix("data: ")
                    && let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(json)
                    && snapshot["state"] == "disconnected"
                {
                    assert_eq!(snapshot["source"], source);
                    assert!(snapshot["attitude"].is_null());
                    assert!(snapshot["sample_age_ms"].is_null());
                    assert!(snapshot["sequence"].as_u64().unwrap() > 0);
                    return socket;
                }
            }
            assert!(
                Instant::now() < deadline,
                "no disconnected snapshot: {response}"
            );
        }
    }
}

#[test]
fn default_dll_is_beside_executable_and_explicit_missing_dll_does_not_fall_back() {
    let fixture = Fixture::new();
    let cwd = fixture.0.join("working-directory");
    fs::create_dir(&cwd).unwrap();
    fs::write(
        cwd.join("SimConnect.dll"),
        "invalid DLL in working directory",
    )
    .unwrap();
    let binary = fixture.binary();
    let expected = fixture.0.join("SimConnect.dll");
    {
        let mut app = Application::start(&binary, &cwd, &["--source", "SimConnect.dll"]);
        let mut first = app.unavailable_stream();
        app.assert_health();
        let mut second = app.unavailable_stream();
        for stream in [&mut first, &mut second] {
            stream
                .set_read_timeout(Some(Duration::from_millis(150)))
                .unwrap();
            let mut bytes = [0; 4096];
            loop {
                match stream.read(&mut bytes) {
                    Ok(0) => panic!("existing SSE closed after producer failure"),
                    Ok(_) => continue, // Permit trailing chunk framing already in flight.
                    Err(error) => {
                        assert!(matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ));
                        break;
                    }
                }
            }
        }
        app.assert_health();
        let messages = app.messages();
        assert!(messages.contains(expected.to_str().unwrap()), "{messages}");
        assert!(messages.contains("--source"), "{messages}");
        assert!(messages.contains("Listening on"), "{messages}");
        assert_eq!(
            fs::read_dir(&cwd).unwrap().count(),
            1,
            "no implicit diagnostics"
        );
    }
    fs::write(&expected, "invalid sibling DLL").unwrap();
    let override_path = fixture.0.join("missing").join("SimConnect.dll");
    let mut app = Application::start(
        &binary,
        &cwd,
        &["--source", override_path.to_str().unwrap()],
    );
    let _stream = app.unavailable_stream();
    app.assert_health();
    let messages = app.messages();
    assert!(
        messages.contains(override_path.to_str().unwrap()),
        "{messages}"
    );
    assert!(
        !messages.contains(expected.to_str().unwrap()),
        "must not try sibling DLL: {messages}"
    );
}

#[test]
fn invalid_library_and_missing_exports_preserve_http_and_report_distinct_errors() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let invalid = fixture.0.join("SimConnect.dll");
    fs::write(&invalid, "not a native library").unwrap();
    let system = fixture.0.join("system").join("SimConnect.dll");
    fs::create_dir_all(system.parent().unwrap()).unwrap();
    fs::copy(
        PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/kernel32.dll"),
        &system,
    )
    .unwrap();
    for (dll, expected) in [
        (&invalid, "LoadLibrary"),
        (&system, "missing export SimConnect_Open"),
    ] {
        let mut app = Application::start(&binary, &fixture.0, &["--source", dll.to_str().unwrap()]);
        let _stream = app.unavailable_stream();
        app.assert_health();
        assert!(app.messages().contains(expected), "{}", app.messages());
    }
}

#[test]
fn help_and_version_do_not_open_requested_files() {
    let fixture = Fixture::new();
    let log = fixture.0.join("must-not-exist/session.jsonl");
    for flag in ["--help", "--version"] {
        let result = output(
            Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon")),
            &fixture.0,
            &[
                "--source",
                "missing.dll",
                "--log",
                log.to_str().unwrap(),
                flag,
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!log.parent().unwrap().exists());
    }
}

#[test]
fn explicit_log_appends_distinct_complete_sessions_when_native_loading_fails() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let dll = fixture.0.join("missing").join("SimConnect.dll");
    let log = fixture.0.join("sessions/loader-failure.jsonl");
    for _ in 0..2 {
        let mut app = Application::start(
            &binary,
            &fixture.0,
            &[
                "--source",
                dll.to_str().unwrap(),
                "--log",
                "sessions/loader-failure.jsonl",
            ],
        );
        let _stream = app.unavailable_stream();
        app.assert_health();
        assert!(app.messages().contains("Diagnostic session completed"));
    }
    let records: Vec<serde_json::Value> = fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let starts: Vec<_> = records
        .iter()
        .filter(|r| r["kind"] == "session_start")
        .collect();
    let ends: Vec<_> = records
        .iter()
        .filter(|r| r["kind"] == "session_end")
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(ends.len(), 2);
    assert_ne!(starts[0]["session_id"], starts[1]["session_id"]);
    assert!(starts.iter().all(|r| r["data"].get("scenario").is_none()));
    assert!(ends.iter().all(|r| r["data"]["success"] == false));
}

#[test]
fn a_log_open_failure_prevents_native_load_but_preserves_http() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let dll = fixture.0.join("missing").join("SimConnect.dll");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--source",
            dll.to_str().unwrap(),
            "--log",
            fixture.0.to_str().unwrap(),
        ],
    );
    let _stream = app.unavailable_stream();
    app.assert_health();
    let messages = app.messages();
    assert!(
        messages.contains("log") || messages.contains("diagnostic"),
        "{messages}"
    );
    assert!(!messages.contains("LoadLibrary"), "{messages}");
}

#[test]
fn occupied_http_port_prevents_producer_and_diagnostics_start() {
    let fixture = Fixture::new();
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reservation.local_addr().unwrap().port().to_string();
    let log = fixture.0.join("must-not-exist/session.jsonl");
    let dll = fixture.0.join("missing").join("SimConnect.dll");
    let result = output(
        Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon")),
        &fixture.0,
        &[
            "--source",
            dll.to_str().unwrap(),
            "--port",
            &port,
            "--log",
            log.to_str().unwrap(),
        ],
    );
    assert!(!result.status.success());
    assert!(!log.parent().unwrap().exists());
    let messages = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(messages.contains("cannot listen"), "{messages}");
    assert!(!messages.contains("Selected DLL:"), "{messages}");
}

#[test]
fn invalid_source_specific_arguments_have_no_startup_side_effects() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    for arguments in [
        vec!["--source", "relative/SimConnect.dll"],
        vec!["--source", "demo"],
        vec![
            "--source",
            "SimConnect.dll",
            "--scenario",
            "must-not-exist/session.jsonl",
        ],
    ] {
        let result = output(&binary, &fixture.0, &arguments);
        assert!(!result.status.success());
        assert!(!String::from_utf8_lossy(&result.stdout).contains("Listening on"));
        assert!(!fixture.0.join("must-not-exist").exists());
    }
}

#[test]
fn demo_serves_http_with_an_invalid_sibling_dll() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    fs::write(fixture.0.join("SimConnect.dll"), "not a native library").unwrap();
    for arguments in [&[][..], &["--source", "demo.dll"][..]] {
        let mut app = Application::start(&binary, &fixture.0, arguments);
        app.assert_health();
        let (snapshot, _stream) = wait_live_ias(&app);
        assert_eq!(snapshot["source"], "demo");
        let messages = app.messages();
        assert!(messages.contains("Source: DEMO"), "{messages}");
    }
}

#[test]
fn explicit_scenario_reaches_sse_and_independent_gap_expiry() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo-evidence.jsonl");
    let scenario = fixture.0.join("bounded-evidence.jsonl");
    let mut input = fs::read_to_string(source).unwrap();
    // This no-op event marks the midpoint without changing the S6 signal tracks.
    input.push_str("{\"type\":\"pause\",\"at_us\":7500000,\"mode\":\"off\"}\n");
    let scenario_sha = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(input.as_bytes()))
    };
    fs::write(&scenario, input).unwrap();
    let log = fixture.0.join("demo-log.jsonl");
    let started_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let started = Instant::now();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--source",
            "demo.dll",
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    let mut stream = app.request("/api/v1/events");
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut pending = String::new();
    let mut hold_first = None;
    let mut hold_late = false;
    let mut ramp_snapshot = None;
    let mut gap = false;
    let mut normal_pause = false;
    let mut normal_resume = false;
    let mut active_pause = false;
    let mut loop_resume = false;
    let deadline = started + Duration::from_secs(25);
    while Instant::now() < deadline && !loop_resume {
        let mut bytes = [0; 8192];
        match stream.read(&mut bytes) {
            Ok(0) => panic!("SSE closed: {}", app.messages()),
            Ok(n) => pending.push_str(&String::from_utf8_lossy(&bytes[..n])),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                continue;
            }
            Err(e) => panic!("SSE read failed: {e}"),
        }
        while let Some(pos) = pending.find('\n') {
            let line = pending[..pos].trim_end_matches('\r').to_owned();
            pending.drain(..=pos);
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(data) else {
                continue;
            };
            assert_eq!(snapshot["source"], "demo");
            if gap && snapshot["state"] == "paused" {
                assert!(snapshot["attitude"].is_null());
                assert!(snapshot.get("ias_kt").is_none());
                if normal_resume {
                    active_pause = true;
                } else {
                    normal_pause = true;
                }
            }
            if snapshot["state"] != "live" {
                continue;
            }
            if active_pause {
                loop_resume = true;
            } else if normal_pause {
                normal_resume = true;
            }
            if snapshot["ias_kt"] == 100.0 && snapshot["altitude_ft"] == 1000.0 {
                let first = hold_first.get_or_insert_with(Instant::now);
                if first.elapsed() > Duration::from_millis(1100) {
                    hold_late = true;
                }
                assert!((snapshot["attitude"]["pitch_deg"].as_f64().unwrap() - 15.0).abs() < 1e-9);
                assert!((snapshot["attitude"]["roll_deg"].as_f64().unwrap() + 30.0).abs() < 1e-9);
            }
            if let Some(value) = snapshot["ias_kt"].as_f64()
                && (124.0..=126.0).contains(&value)
            {
                ramp_snapshot = Some(value);
            }
            if hold_late
                && started.elapsed() > Duration::from_secs(11)
                && snapshot.get("ias_kt").is_none()
                && snapshot["altitude_ft"] == 1000.0
            {
                gap = true;
            }
        }
    }
    assert!(
        hold_late,
        "equal IAS did not remain live past one second: {}",
        app.messages()
    );
    assert!(
        gap,
        "IAS did not expire independently while ALT continued: {}",
        app.messages()
    );
    assert!(
        normal_pause && normal_resume && active_pause && loop_resume,
        "pause/resume across the next pass was not observed: {}",
        app.messages()
    );
    assert!(
        ramp_snapshot.is_some(),
        "no SSE snapshot near the independent 125-kt midpoint"
    );
    app.assert_health();
    let records = fs::read_to_string(log).unwrap();
    let records: Vec<serde_json::Value> = records
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect();
    let observed = records
        .iter()
        .filter(|r| {
            r["kind"] == "instrument_sample"
                && r["data"]["name"] == "AIRSPEED INDICATED"
                && r["data"]["value"]["value"] == 100.0
        })
        .count();
    assert!(observed > 2, "raw diagnostic IAS receipts: {observed}");
    let marker = records
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r["kind"] == "dispatch"
                && r["data"]["message"]["Event"]["event_id"] == 11
                && r["data"]["message"]["Event"]["data"] == 0
        })
        .nth(2)
        .map(|(i, _)| i)
        .expect("initial, 2-second, and 7.5-second pause-off events");
    let ias = |r: &serde_json::Value| {
        (r["kind"] == "instrument_sample" && r["data"]["name"] == "AIRSPEED INDICATED")
            .then(|| r["data"]["value"]["value"].as_f64())
            .flatten()
    };
    let before = records[..marker].iter().rev().find_map(ias).unwrap();
    let after = records[marker + 1..].iter().find_map(ias).unwrap();
    assert!(
        (124.0..=125.0).contains(&before),
        "last IAS before 7.5s marker: {before}"
    );
    assert!(
        (125.0..=126.0).contains(&after),
        "first IAS after 7.5s marker: {after}"
    );
    eprintln!(
        "S6 run start_unix_ms={started_unix_ms}, scenario_sha256={scenario_sha}, session_id={}, marker_elapsed_us={}, IAS before={before}, after={after}, SSE near midpoint={}",
        records[marker]["session_id"],
        records[marker]["elapsed_us"],
        ramp_snapshot.unwrap()
    );
}

fn wait_live_ias(app: &Application) -> (serde_json::Value, TcpStream) {
    let mut stream = app.request("/api/v1/events");
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut response = String::new();
    while Instant::now() < deadline {
        let mut bytes = [0; 8192];
        let n = match stream.read(&mut bytes) {
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                continue;
            }
            Err(e) => panic!("SSE read failed: {e}"),
        };
        assert!(n > 0);
        response.push_str(&String::from_utf8_lossy(&bytes[..n]));
        for line in response.lines() {
            if let Some(data) = line.strip_prefix("data: ")
                && let Ok(value) = serde_json::from_str::<serde_json::Value>(data)
                && value["state"] == "live"
                && value.get("ias_kt").is_some()
            {
                return (value, stream);
            }
        }
    }
    panic!("no live IAS: {}", app.messages())
}

#[test]
fn scenario_changes_apply_after_restart_and_two_clients_share_source() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("selected.jsonl");
    for expected in [40, 90] {
        let text = format!(
            concat!(
                "{{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":10000000,\"initial\":{{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}}}\n",
                "{{\"type\":\"signal\",\"id\":\"p\",\"simvar\":\"PLANE PITCH DEGREES\",\"unit\":\"radians\",\"repeat\":false}}\n{{\"type\":\"hold\",\"signal\":\"p\",\"value\":0,\"duration_ms\":10000}}\n",
                "{{\"type\":\"signal\",\"id\":\"b\",\"simvar\":\"PLANE BANK DEGREES\",\"unit\":\"radians\",\"repeat\":false}}\n{{\"type\":\"hold\",\"signal\":\"b\",\"value\":0,\"duration_ms\":10000}}\n",
                "{{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":false}}\n{{\"type\":\"hold\",\"signal\":\"ias\",\"value\":{},\"duration_ms\":10000}}\n"
            ),
            expected
        );
        fs::write(&scenario, text).unwrap();
        let app = Application::start(
            &binary,
            &fixture.0,
            &[
                "--source",
                "DEMO.dll",
                "--scenario",
                scenario.to_str().unwrap(),
            ],
        );
        let (a, _first_stream) = wait_live_ias(&app);
        let (b, _second_stream) = wait_live_ias(&app);
        assert_eq!(a["source"], "demo");
        assert_eq!(a["ias_kt"].as_f64(), Some(expected as f64));
        assert_eq!(b["ias_kt"].as_f64(), Some(expected as f64));
        assert!(b["sequence"].as_u64().unwrap() >= a["sequence"].as_u64().unwrap());
    }
}

#[test]
fn invalid_demo_scenario_keeps_http_and_reports_setup_error() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("bad.jsonl");
    fs::write(&scenario, "bad json").unwrap();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &["--scenario", scenario.to_str().unwrap()],
    );
    let _stream = app.unavailable_stream_for("demo");
    app.assert_health();
    let messages = app.messages();
    assert!(
        messages.contains("EfiDemo_ConfigureV1") && messages.contains("line 1"),
        "{messages}"
    );
}

#[test]
fn sparse_ias_scenario_does_not_fabricate_attitude_or_publish_optional_values() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("sparse.jsonl");
    fs::write(&scenario,concat!(
        "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":5000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
        "{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":false}\n",
        "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":100,\"duration_ms\":5000}\n"
    )).unwrap();
    let app = Application::start(
        &binary,
        &fixture.0,
        &["--scenario", scenario.to_str().unwrap()],
    );
    let mut stream = app.request("/api/v1/events");
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut response = String::new();
    let deadline = Instant::now() + Duration::from_millis(800);
    let mut saw = false;
    while Instant::now() < deadline {
        let mut bytes = [0; 4096];
        let n = match stream.read(&mut bytes) {
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                continue;
            }
            Err(e) => panic!("SSE read failed: {e}"),
        };
        assert!(n > 0);
        response.push_str(&String::from_utf8_lossy(&bytes[..n]));
        for line in response.lines() {
            if let Some(data) = line.strip_prefix("data: ")
                && let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(data)
            {
                assert_ne!(snapshot["state"], "live");
                assert!(snapshot["attitude"].is_null());
                assert!(snapshot.get("ias_kt").is_none());
                saw = true;
            }
        }
    }
    assert!(saw);
}

#[test]
fn inactive_flight_waits_for_new_attitude_after_resume() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("inactive.jsonl");
    let log = fixture.0.join("inactive-log.jsonl");
    fs::write(&scenario,concat!(
        "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":10000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
        "{\"type\":\"flight\",\"at_us\":500000,\"active\":false}\n",
        "{\"type\":\"flight\",\"at_us\":1500000,\"active\":true}\n",
        "{\"type\":\"signal\",\"id\":\"pitch\",\"simvar\":\"PLANE PITCH DEGREES\",\"unit\":\"radians\",\"repeat\":false}\n",
        "{\"type\":\"hold\",\"signal\":\"pitch\",\"value\":0,\"duration_ms\":1200}\n",
        "{\"type\":\"gap\",\"signal\":\"pitch\",\"duration_ms\":1500}\n",
        "{\"type\":\"hold\",\"signal\":\"pitch\",\"value\":0.1,\"duration_ms\":7300}\n",
        "{\"type\":\"signal\",\"id\":\"bank\",\"simvar\":\"PLANE BANK DEGREES\",\"unit\":\"radians\",\"repeat\":false}\n",
        "{\"type\":\"hold\",\"signal\":\"bank\",\"value\":0,\"duration_ms\":10000}\n"
    )).unwrap();
    let app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    let mut stream = app.request("/api/v1/events");
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(7);
    let mut pending = String::new();
    let mut saw_live = false;
    let mut saw_waiting = false;
    let mut resumed_at = None;
    let mut held_waiting = false;
    let mut recovered = false;
    while Instant::now() < deadline && !recovered {
        if saw_waiting && resumed_at.is_none() {
            let content = fs::read_to_string(&log).unwrap_or_default();
            let active = content
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .any(|r| {
                    r["kind"] == "dispatch"
                        && r["data"]["message"]["Event"]["event_id"] == 10
                        && r["data"]["message"]["Event"]["data"] == 1
                });
            if active {
                resumed_at = Some(Instant::now());
            }
        }
        if let Some(t) = resumed_at
            && t.elapsed() > Duration::from_millis(400)
        {
            held_waiting = true;
        }
        let mut bytes = [0; 8192];
        match stream.read(&mut bytes) {
            Ok(0) => panic!("SSE closed"),
            Ok(n) => pending.push_str(&String::from_utf8_lossy(&bytes[..n])),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(e) => panic!("{e}"),
        }
        while let Some(pos) = pending.find('\n') {
            let line = pending[..pos].trim_end_matches('\r').to_owned();
            pending.drain(..=pos);
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(data) else {
                continue;
            };
            if snapshot["state"] == "live" {
                if !saw_waiting {
                    saw_live = true;
                } else if resumed_at.is_some() {
                    assert!(
                        held_waiting,
                        "live resumed before a fresh attitude was eligible"
                    );
                    assert!(
                        (snapshot["attitude"]["pitch_deg"].as_f64().unwrap()
                            + 0.1_f64.to_degrees())
                        .abs()
                            < 1e-9
                    );
                    recovered = true;
                }
            } else if saw_live && snapshot["state"] == "waiting" {
                assert!(snapshot["attitude"].is_null());
                saw_waiting = true;
            }
        }
    }
    assert!(
        saw_live && saw_waiting && resumed_at.is_some() && held_waiting && recovered,
        "incomplete inactive/resume sequence: {}",
        app.messages()
    );
}
