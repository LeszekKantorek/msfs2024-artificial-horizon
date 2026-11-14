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
            Path::new(env!("CARGO_MANIFEST_DIR")).join("default.json"),
            self.0.join("default.json"),
        )
        .unwrap();
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

#[test]
fn missing_mapping_does_not_fall_back_to_working_directory() {
    let executable_dir = Fixture::new();
    let cwd = Fixture::new();
    let binary = executable_dir.binary();
    fs::remove_file(executable_dir.0.join("default.json")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("default.json"),
        cwd.0.join("default.json"),
    )
    .unwrap();
    let scenario = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo-scenarios/demo-scenario.jsonl");
    let result = output(
        &binary,
        &cwd.0,
        &[
            "--source",
            "demo.dll",
            "--scenario",
            scenario.to_str().unwrap(),
        ],
    );
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("default.json"), "{error}");
}

#[test]
fn reordered_mixed_numeric_mapping_reaches_sse() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let mapping_path = fixture.0.join("default.json");
    let mut mapping: serde_json::Value =
        serde_json::from_slice(&fs::read(&mapping_path).unwrap()).unwrap();
    let entries = mapping["mappings"].as_array_mut().unwrap();
    entries.swap(0, 2);
    entries[0]["type"] = "SIMCONNECT_DATATYPE_INT32".into();
    entries[1]["type"] = "SIMCONNECT_DATATYPE_FLOAT32".into();
    entries[2]["type"] = "SIMCONNECT_DATATYPE_INT64".into();
    fs::write(mapping_path, serde_json::to_vec(&mapping).unwrap()).unwrap();
    let scenario = fixture.0.join("mixed.jsonl");
    fs::write(&scenario, concat!(
        "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":10000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"mixed\"}}\n",
        "{\"type\":\"signal\",\"id\":\"pitch\",\"simvar\":\"PLANE PITCH DEGREES\",\"unit\":\"radians\",\"repeat\":true}\n",
        "{\"type\":\"hold\",\"signal\":\"pitch\",\"value\":-1,\"duration_ms\":10000}\n",
        "{\"type\":\"signal\",\"id\":\"bank\",\"simvar\":\"PLANE BANK DEGREES\",\"unit\":\"radians\",\"repeat\":true}\n",
        "{\"type\":\"hold\",\"signal\":\"bank\",\"value\":0.5,\"duration_ms\":10000}\n",
        "{\"type\":\"signal\",\"id\":\"ias\",\"simvar\":\"AIRSPEED INDICATED\",\"unit\":\"knots\",\"repeat\":true}\n",
        "{\"type\":\"hold\",\"signal\":\"ias\",\"value\":123,\"duration_ms\":10000}\n"
    )).unwrap();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &["--scenario", scenario.to_str().unwrap()],
    );
    let snapshot = fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 123.0);
    assert!(
        (snapshot["attitude"]["pitch_deg"].as_f64().unwrap() - 1.0_f64.to_degrees()).abs() < 1e-9
    );
    assert!(
        (snapshot["attitude"]["roll_deg"].as_f64().unwrap() + 0.5_f64.to_degrees()).abs() < 1e-9
    );
    app.shutdown();
}

#[test]
fn grouped_distinct_fields_replace_one_invalid_optional() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grouped-distinct.jsonl");
    let log = fixture.0.join("distinct.jsonl");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    let initial = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 101.0 && s["tas_kt"] == 111.0
    });
    assert_eq!(initial["altitude_ft"], 1300.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |records| {
        records.iter().any(|r| {
            r["kind"] == "grouped_sample" && r["data"]["values"][3]["classification"] == "nan"
        })
    });
    let samples: Vec<_> = records
        .iter()
        .filter(|r| r["kind"] == "grouped_sample" && r["data"]["active"] == true)
        .collect();
    let gap = samples
        .iter()
        .position(|r| r["data"]["values"][3]["classification"] == "nan")
        .unwrap();
    assert!(gap > 0);
    let expected = [
        -0.1, 0.2, 101.0, 111.0, 121.0, 1300.0, 29.9, 321.0, 222.0, 32.0, 1.2, 40.0, 50.0, 60.0,
        160.0, 1.0, 270.0, 3000.0, 0.0,
    ];
    for (index, value) in expected.iter().enumerate() {
        assert_eq!(
            samples[gap - 1]["data"]["values"][index]["value"].as_f64(),
            Some(*value),
            "before gap index {index}"
        );
        if index == 3 {
            assert_eq!(
                samples[gap]["data"]["values"][index]["classification"],
                "nan"
            );
        } else {
            assert_eq!(
                samples[gap]["data"]["values"][index]["value"].as_f64(),
                Some(*value),
                "gap index {index}"
            );
        }
    }
    let gap_snapshot = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 101.0 && s.get("tas_kt").is_none()
    });
    let comparable = |mut snapshot: serde_json::Value| {
        let object = snapshot.as_object_mut().unwrap();
        for field in [
            "sequence",
            "sample_age_ms",
            "tas_kt",
            "ias_acceleration_ktps",
        ] {
            object.remove(field);
        }
        snapshot
    };
    assert_eq!(comparable(initial), comparable(gap_snapshot));
    app.shutdown();
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
                .stdin(Stdio::piped())
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
    fn shutdown(&mut self) {
        self.child
            .0
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"stop\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.0.try_wait().unwrap() {
                assert!(status.success(), "shutdown failed: {}", self.messages());
                break;
            }
            assert!(
                Instant::now() < deadline,
                "shutdown timed out: {}",
                self.messages()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
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
    let sibling = fixture.0.join("SimConnect.dll");
    let result = output(&binary, &cwd, &["--source", "SimConnect.dll"]);
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains(sibling.to_str().unwrap()), "{error}");
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Listening on"));
    fs::write(&sibling, "invalid sibling DLL").unwrap();
    let override_path = fixture.0.join("missing").join("SimConnect.dll");
    let result = output(
        &binary,
        &cwd,
        &["--source", override_path.to_str().unwrap()],
    );
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(!result.status.success());
    assert!(error.contains(override_path.to_str().unwrap()), "{error}");
    assert!(!error.contains(sibling.to_str().unwrap()), "{error}");
}

#[test]
fn invalid_library_and_missing_exports_fail_before_http() {
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
        let result = output(&binary, &fixture.0, &["--source", dll.to_str().unwrap()]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains(expected));
        assert!(!String::from_utf8_lossy(&result.stdout).contains("Listening on"));
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
fn missing_library_fails_before_creating_log() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let dll = fixture.0.join("missing").join("SimConnect.dll");
    let log = fixture.0.join("sessions/loader-failure.jsonl");
    let result = output(
        &binary,
        &fixture.0,
        &[
            "--source",
            dll.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    assert!(!result.status.success());
    assert!(!log.exists());
}

#[test]
fn a_log_open_failure_prevents_native_load_but_preserves_http() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo-scenarios/demo-scenario.jsonl");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            fixture.0.to_str().unwrap(),
        ],
    );
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert_eq!(state["source"], "demo");
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
    let binary = fixture.binary();
    let scenario = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo-scenarios/demo-scenario.jsonl");
    let result = output(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
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
        vec![],
        vec!["--source", "demo.dll"],
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
    let scenario = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo-scenarios/demo-scenario.jsonl");
    for arguments in [
        vec!["--scenario", scenario.to_str().unwrap()],
        vec![
            "--source",
            "demo.dll",
            "--scenario",
            scenario.to_str().unwrap(),
        ],
    ] {
        let mut app = Application::start(&binary, &fixture.0, &arguments);
        app.assert_health();
        let (snapshot, _stream) = wait_live_ias(&app);
        assert_eq!(snapshot["source"], "demo");
        let messages = app.messages();
        assert!(messages.contains("Source: DEMO"), "{messages}");
    }
}

#[test]
fn explicit_scenario_reaches_sse_and_grouped_optional_gap() {
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
    let deadline = started + Duration::from_secs(35);
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
        "IAS did not become unavailable while ALT continued: {}",
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
    let pause_flags: Vec<u64> = records
        .iter()
        .filter(|r| r["kind"] == "dispatch" && r["data"]["message"]["Event"]["event_id"] == 11)
        .filter_map(|r| r["data"]["message"]["Event"]["data"].as_u64())
        .collect();
    assert!(
        pause_flags.windows(4).any(|flags| flags == [8, 0, 4, 0]),
        "expected Escape and Active Pause flags with resumes, got {pause_flags:?}"
    );
    let observed = records
        .iter()
        .filter(|r| r["kind"] == "grouped_sample" && r["data"]["values"][2]["value"] == 100.0)
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
        (r["kind"] == "grouped_sample")
            .then(|| r["data"]["values"][2]["value"].as_f64())
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
        messages.contains("Demo scenario") && messages.contains("line 1"),
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

fn fault_case(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("demo-scenarios/errors")
        .join(name)
}
fn fault_snapshot(
    app: &Application,
    predicate: impl Fn(&serde_json::Value) -> bool,
) -> serde_json::Value {
    let mut stream = app.request("/api/v1/events");
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut pending = String::new();
    while Instant::now() < deadline {
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
            Err(e) => panic!("{e}"),
        }
        while let Some(end) = pending.find('\n') {
            let line = pending[..end].trim_end_matches('\r').to_owned();
            pending.drain(..=end);
            if let Some(data) = line.strip_prefix("data: ")
                && let Ok(snapshot) = serde_json::from_str::<serde_json::Value>(data)
                && predicate(&snapshot)
            {
                return snapshot;
            }
        }
    }
    panic!("expected state not observed: {}", app.messages())
}
fn wait_fault_log(
    path: &Path,
    predicate: impl Fn(&[serde_json::Value]) -> bool,
) -> Vec<serde_json::Value> {
    wait_fault_log_for(path, Duration::from_secs(5), predicate)
}
fn wait_fault_log_for(
    path: &Path,
    timeout: Duration,
    predicate: impl Fn(&[serde_json::Value]) -> bool,
) -> Vec<serde_json::Value> {
    let deadline = Instant::now() + timeout;
    loop {
        let content = fs::read_to_string(path).unwrap_or_default();
        let records: Vec<_> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        if predicate(&records) {
            return records;
        }
        assert!(
            Instant::now() < deadline,
            "expected fault records missing: {content}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
#[test]
fn fault_grouped_request_failure_terminates_session() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "optional-request.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(!records.iter().any(|v| v["kind"] == "grouped_request"));
    assert!(!records.iter().any(|v| v["kind"] == "reconnect_wait"));
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(state["attitude"].is_null());
    app.assert_health();
    app.shutdown();
}

#[test]
fn correlated_grouped_request_exception_identifies_operation() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "optional-exception.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "native_exception")
    });
    let error = records
        .iter()
        .find(|v| v["kind"] == "native_exception")
        .unwrap();
    assert_eq!(
        error["data"]["operation"],
        "SimConnect_RequestDataOnSimObject"
    );
    assert_eq!(error["data"]["mapping"], "no correlated mapping");
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn version_mismatch_reports_dll_and_sdk_without_log() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("version-mismatch.jsonl");
    let input = fs::read_to_string(fault_case("optional-exception.jsonl"))
        .unwrap()
        .replace("\"code\":2", "\"code\":5");
    fs::write(&scenario, input).unwrap();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &["--scenario", scenario.to_str().unwrap()],
    );
    fault_snapshot(&app, |s| s["state"] == "live");
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let messages = app.messages();
        if messages.contains("VERSION_MISMATCH")
            && messages.contains("DLL identity:")
            && messages.contains("SDK metadata:")
        {
            break;
        }
        assert!(Instant::now() < deadline, "{messages}");
        std::thread::sleep(Duration::from_millis(10));
    }
    app.assert_health();
    app.shutdown();
}

#[test]
fn correlated_definition_exception_terminates_group() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "definition-exception.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |records| {
        records.iter().any(|r| r["kind"] == "native_exception")
    });
    let exception = records
        .iter()
        .find(|r| r["kind"] == "native_exception")
        .unwrap();
    assert_eq!(
        exception["data"]["operation"],
        "SimConnect_AddToDataDefinition"
    );
    assert!(
        exception["data"]["mapping"]
            .as_str()
            .unwrap()
            .contains("baro_setting")
    );
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(state.get("ias_kt").is_none());
    assert!(state["attitude"].is_null());
    app.assert_health();
    app.shutdown();
}

#[test]
fn exception_before_sample_prevents_late_packets_restoring_live() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "exception-before-sample.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "session_end")
    });
    assert!(records.iter().any(|v| v["kind"] == "native_exception"));
    assert!(!records.iter().any(|v| v["kind"] == "reconnect_wait"));
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(state["attitude"].is_null());
    std::thread::sleep(Duration::from_millis(1000));
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_uncorrelated_exception_ends_source_but_preserves_http() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "uncorrelated-exception.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "session_end")
    });
    assert!(
        !records
            .iter()
            .any(|v| v["kind"] == "instrument_unavailable")
    );
    assert_eq!(
        records
            .iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count(),
        1
    );
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn lost_registration_packet_id_stops_before_later_exception() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "packet-id-exception.jsonl");
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "session_end")
    });
    assert!(records.iter().any(|v| {
        v["kind"] == "acquisition_failure"
            && v["data"]["error"]
                .as_str()
                .is_some_and(|s| s.contains("correlate"))
    }));
    assert!(!records.iter().any(|v| v["kind"] == "native_exception"));
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_exception_without_sent_call_is_terminal_scenario_error() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "missing-sent-call.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "session_end")
    });
    assert_eq!(
        records
            .iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "native_error" && v["data"]["hresult"] == -1610350591)
    );
    assert!(app.messages().contains("exception references unavailable"));
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn failed_registration_stops_before_later_exception() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "failed-sent-call.jsonl");
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| v["kind"] == "session_end")
    });
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "native_error" && v["data"]["field"] == "indicated_airspeed")
    );
    assert!(!records.iter().any(|v| v["kind"] == "native_exception"));
    assert!(!records.iter().any(|v| v["kind"] == "grouped_request"));
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_nonfinite_attitude_suppresses_live_until_fresh_recovery() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "invalid-attitude.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let invalid = fault_snapshot(&app, |s| s["state"] == "invalid");
    assert!(invalid["attitude"].is_null());
    let records = wait_fault_log(&log, |r| {
        r.iter()
            .any(|v| v["kind"] == "grouped_sample" && !v["data"]["validation_error"].is_null())
    });
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "grouped_sample" && !v["data"]["validation_error"].is_null())
    );
    fault_snapshot(&app, |s| s["state"] == "live" && s["attitude"].is_object());
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_nonfinite_optional_value_preserves_attitude_and_altitude() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "invalid-optional.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
        r.iter().any(|v| {
            v["kind"] == "grouped_sample" && v["data"]["values"][2]["classification"] == "nan"
        })
    });
    assert!(records.iter().any(
        |v| v["kind"] == "grouped_sample" && v["data"]["values"][2]["classification"] == "nan"
    ));
    let state = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["altitude_ft"] == 1000.0 && s.get("ias_kt").is_none()
    });
    assert!(state["attitude"].is_object());
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    app.assert_health();
    app.shutdown();
}

#[test]
fn native_bool_minus_one_enables_targets_and_zero_clears_them() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "native-bool.jsonl");
    fault_snapshot(&app, |s| {
        s["state"] == "live"
            && s["selected_heading_deg"] == 270.0
            && s["selected_altitude_ft"] == 3000.0
    });
    let records = wait_fault_log(&log, |r| {
        r.iter()
            .any(|v| v["kind"] == "grouped_sample" && v["data"]["values"][15]["value"] == -1.0)
    });
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "grouped_sample" && v["data"]["values"][15]["value"] == -1.0)
    );
    fault_snapshot(&app, |s| {
        s["state"] == "live"
            && s["heading_deg"] == 180.0
            && s.get("selected_heading_deg").is_none()
            && s.get("selected_altitude_ft").is_none()
    });
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_initial_state_query_ends_source_but_http_stays_available() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "initial-state-query.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(records.iter().any(|v| v["kind"] == "native_error"
        && v["data"]["operation"] == "SimConnect_RequestSystemState"));
    assert_eq!(
        records
            .iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert_eq!(
        fault_snapshot(&app, |s| s["state"] == "disconnected")["attitude"],
        serde_json::Value::Null
    );
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_periodic_state_query_reconnects_and_recovers() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "periodic-state-query.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let records = wait_fault_log_for(&log, Duration::from_secs(15), |r| {
        r.iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count()
            >= 2
    });
    assert!(records.iter().any(|v| v["kind"] == "native_error"
        && v["data"]["operation"] == "SimConnect_RequestSystemState"));
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_packet_id_failure_terminates_setup() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "packet-id.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(records.iter().any(|v| {
        v["kind"] == "acquisition_failure"
            && v["data"]["error"]
                .as_str()
                .is_some_and(|s| s.contains("correlate"))
    }));
    assert!(!records.iter().any(|v| v["kind"] == "grouped_request"));
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(state["attitude"].is_null());
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_failed_close_keeps_http_available() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "close-failure.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "native_error" && v["data"]["operation"] == "SimConnect_Close")
    );
    fault_snapshot(&app, |s| s["state"] == "disconnected");
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_transport_silence_causes_watchdog_reconnect_and_recovery() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "transport-silence.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    fault_snapshot(&app, |s| s["state"] == "stale");
    app.assert_health();
    let records = wait_fault_log_for(&log, Duration::from_secs(25), |r| {
        r.iter().any(|v| {
            v["kind"] == "connection_lost"
                && v["data"]["reason"] == "diagnostic_watchdog_no_packets_for_15_seconds"
        }) && r
            .iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count()
            >= 2
    });
    assert!(
        !records.iter().any(
            |v| v["kind"] == "connection_lost" && v["data"]["reason"] == "SIMCONNECT_RECV_QUIT"
        )
    );
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_stop_remains_responsive_during_transport_silence() {
    let fixture = Fixture::new();
    let (mut app, _) = start_fault(&fixture, "transport-silence.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    fault_snapshot(&app, |s| s["state"] == "stale");
    app.shutdown();
}

#[test]
fn data_only_gap_with_state_replies_does_not_trigger_watchdog() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "data-gap.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    fault_snapshot(&app, |s| s["state"] == "stale");
    let records = wait_fault_log_for(&log, Duration::from_secs(20), |r| {
        r.iter()
            .filter(|v| v["kind"] == "system_state_float")
            .count()
            >= 8
    });
    assert!(!records.iter().any(|v| v["kind"] == "connection_lost"));
    assert_eq!(
        records
            .iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count(),
        1
    );
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    app.assert_health();
    app.shutdown();
}

#[test]
fn stop_remains_responsive_during_data_gap() {
    let fixture = Fixture::new();
    let (mut app, _) = start_fault(&fixture, "data-gap.jsonl");
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    fault_snapshot(&app, |s| s["state"] == "stale");
    app.shutdown();
}

#[test]
fn fault_stop_remains_responsive_under_dense_traffic() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let input = fs::read_to_string(fault_case("optional-definition.jsonl")).unwrap();
    let input = input
        .lines()
        .filter(|line| !line.contains("api_fault"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\"frame_period_ms\":50", "\"frame_period_ms\":1");
    let scenario = fixture.0.join("dense.jsonl");
    fs::write(&scenario, input).unwrap();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &["--scenario", scenario.to_str().unwrap()],
    );
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    app.shutdown();
}

#[test]
fn stop_interrupts_a_confirmed_queued_packet_burst() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let base = fs::read_to_string(fault_case("optional-definition.jsonl")).unwrap();
    let mut input = base
        .lines()
        .filter(|line| !line.contains("api_fault"))
        .collect::<Vec<_>>()
        .join("\n");
    const FIRST: u32 = 0xf000;
    const COUNT: u32 = 4096;
    // One timestamp queues the whole burst while the emitter holds the state lock.
    for id in FIRST..FIRST + COUNT {
        let b = id.to_le_bytes();
        input.push_str(&format!("\n{{\"type\":\"raw_packet\",\"at_us\":1000000,\"bytes_hex\":\"0c00000000000000{:02x}{:02x}{:02x}{:02x}\",\"returned_length\":12}}", b[0], b[1], b[2], b[3]));
    }
    let scenario = fixture.0.join("queued-burst.jsonl");
    let log = fixture.0.join("queued-burst-log.jsonl");
    fs::write(&scenario, input).unwrap();
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if fs::read_to_string(&log)
            .unwrap_or_default()
            .contains("\"id\":61440")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "burst did not reach the receiver"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    app.shutdown();
    let records: Vec<serde_json::Value> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let received = records
        .iter()
        .filter(|v| {
            v["kind"] == "dispatch"
                && v["data"]["id"]
                    .as_u64()
                    .is_some_and(|id| id >= u64::from(FIRST) && id < u64::from(FIRST + COUNT))
        })
        .count();
    // A partial burst proves Stop interrupted draining rather than following it.
    assert!(
        received > 0 && received < COUNT as usize,
        "stop did not interrupt the burst: {received}"
    );
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "session_end" && v["data"]["success"] == true)
    );
}
fn start_fault(fixture: &Fixture, name: &str) -> (Application, PathBuf) {
    let binary = fixture.binary();
    let scenario = fault_case(name);
    let log = fixture.0.join("fault-log.jsonl");
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
    (app, log)
}
fn report_fault_evidence(fixture: &Fixture, name: &str, records: &[serde_json::Value]) {
    use sha2::{Digest, Sha256};
    let hash = |path: PathBuf| format!("{:x}", Sha256::digest(fs::read(path).unwrap()));
    eprintln!(
        "fault={name} scenario_sha256={} exe_sha256={} dll_sha256={} session_id={}",
        hash(fault_case(name)),
        hash(fixture.0.join("app.exe")),
        hash(fixture.0.join("demo.dll")),
        records[0]["session_id"]
    );
}
#[test]
fn fault_open_retries_once_then_publishes_live_and_keeps_http() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "startup-retry.jsonl");
    let snapshot = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 90.0 && s["altitude_ft"] == 1000.0
    });
    assert_eq!(snapshot["source"], "demo");
    app.assert_health();
    let records = wait_fault_log(&log, |r| {
        r.iter()
            .filter(|v| v["kind"] == "connection_attempt")
            .count()
            >= 2
            && r.iter().any(|v| {
                v["kind"] == "api_result"
                    && v["data"]["operation"] == "SimConnect_Open"
                    && v["data"]["success"] == true
            })
    });
    assert!(
        records
            .iter()
            .any(|r| r["kind"] == "native_error" && r["data"]["operation"] == "SimConnect_Open")
    );
    report_fault_evidence(&fixture, "startup-retry.jsonl", &records);
    app.shutdown();
}
#[test]
fn demo_configuration_hresult_in_injected_open_fault_is_terminal() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let scenario = fixture.0.join("terminal.jsonl");
    fs::write(&scenario, concat!(
        "{\"type\":\"scenario\",\"format\":\"msfs-efi-simconnect-demo\",\"version\":1,\"mode\":\"scripted\",\"frame_period_ms\":50,\"duration_us\":5000000,\"initial\":{\"sim_running\":true,\"pause\":\"off\",\"flight\":\"test\"}}\n",
        "{\"type\":\"api_fault\",\"operation\":\"SimConnect_Open\",\"occurrence\":1,\"hresult\":-1610350591}\n"
    )).unwrap();
    let log = fixture.0.join("terminal-log.jsonl");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            scenario.to_str().unwrap(),
            "--log",
            log.to_str().unwrap(),
        ],
    );
    let records = wait_fault_log(&log, |records| {
        records.iter().any(|r| r["kind"] == "session_end")
    });
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert!(!records.iter().any(|r| r["kind"] == "reconnect_wait"));
    app.assert_health();
    app.shutdown();
}
#[test]
fn failed_middle_definition_terminates_without_request() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "optional-definition.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(
        records
            .iter()
            .any(|v| v["kind"] == "native_error" && v["data"]["field"] == "indicated_airspeed")
    );
    assert!(!records.iter().any(|v| v["kind"] == "grouped_request"));
    assert!(!records.iter().any(|v| v["kind"] == "reconnect_wait"));
    let state = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(state["attitude"].is_null());
    app.assert_health();
    app.shutdown();
}

#[test]
fn fault_required_subscription_ends_source_but_http_stays_available() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "primary-subscribe.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(records.iter().any(|r| r["kind"] == "native_error"
        && r["data"]["operation"] == "SimConnect_SubscribeToSystemEvent"));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert_eq!(
        records.iter().find(|r| r["kind"] == "session_end").unwrap()["data"]["success"],
        false
    );
    let snapshot = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert_eq!(snapshot["source"], "demo");
    assert!(snapshot["attitude"].is_null());
    app.assert_health();
    report_fault_evidence(&fixture, "primary-subscribe.jsonl", &records);
    app.shutdown();
}
#[test]
fn fault_truncated_owned_packet_ends_source_after_live_but_not_http() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "truncated-packet.jsonl");
    let live = fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    assert_eq!(live["source"], "demo");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert!(records.iter().any(|r| {
        r["kind"] == "native_error"
            && r["data"]["detail"]
                .as_str()
                .is_some_and(|s| s.contains("truncated receive header"))
    }));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert_eq!(
        records.iter().find(|r| r["kind"] == "session_end").unwrap()["data"]["success"],
        false
    );
    let snapshot = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(snapshot["attitude"].is_null());
    app.assert_health();
    report_fault_evidence(&fixture, "truncated-packet.jsonl", &records);
    app.shutdown();
}

#[test]
fn malformed_owned_packet_shapes_end_only_the_source() {
    for name in [
        "invalid-header-size.jsonl",
        "truncated-payload.jsonl",
        "invalid-count.jsonl",
    ] {
        let fixture = Fixture::new();
        let (mut app, log) = start_fault(&fixture, name);
        fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
        let records = wait_fault_log_for(&log, Duration::from_secs(8), |r| {
            r.iter().any(|v| v["kind"] == "session_end")
        });
        assert!(
            records.iter().any(|v| v["kind"] == "native_error"
                && v["data"]["operation"] == "decode receive packet"),
            "{name}"
        );
        assert_eq!(
            records
                .iter()
                .filter(|v| v["kind"] == "connection_attempt")
                .count(),
            1,
            "{name}"
        );
        fault_snapshot(&app, |s| s["state"] == "disconnected");
        app.assert_health();
        app.shutdown();
    }
}

#[test]
fn fault_open_retry_wait_is_interruptible_by_shutdown() {
    let fixture = Fixture::new();
    let (mut app, log) = start_fault(&fixture, "startup-retry.jsonl");
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "reconnect_wait"));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "connection_attempt")
            .count(),
        1
    );
    app.shutdown();
    let records = wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "session_end"));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "connection_attempt")
            .count(),
        1
    );
    assert_eq!(
        records.iter().find(|r| r["kind"] == "session_end").unwrap()["data"]["success"],
        true
    );
}

fn suite_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("demo-scenarios")
        .join(name)
}
fn assert_sample_cleared(live: &serde_json::Value, stale: &serde_json::Value) {
    assert!(stale["attitude"].is_null());
    for key in live.as_object().unwrap().keys() {
        if !["sequence", "source", "state", "sample_age_ms", "attitude"].contains(&key.as_str()) {
            assert!(stale.get(key).is_none(), "stale retained {key}: {stale}");
        }
    }
}

#[test]
fn slow_interval_expires_complete_sample_and_recovers() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grouped-distinct.jsonl");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            path.to_str().unwrap(),
            "--simconnect-interval",
            "24",
        ],
    );
    let live = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 101.0 && s["altitude_ft"] == 1300.0
    });
    let stale = fault_snapshot(&app, |s| s["state"] == "stale");
    assert_sample_cleared(&live, &stale);
    let recovered = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 101.0 && s["altitude_ft"] == 1300.0
    });
    assert!(recovered["attitude"].is_object());
    assert!(recovered.get("ias_acceleration_ktps").is_none());
    app.shutdown();
}

#[test]
fn focused_availability_reaches_live_gap_stale_and_fresh_recovery() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let path = suite_path("availability-gaps.jsonl");
    let mut app = Application::start(&binary, &fixture.0, &["--scenario", path.to_str().unwrap()]);
    let initial = fault_snapshot(&app, |s| {
        s["state"] == "live" && s["ias_kt"] == 90.0 && s["tas_kt"] == 105.0
    });
    assert_eq!(initial["source"], "demo");
    let gap = fault_snapshot(&app, |s| {
        s["state"] == "live" && s.get("ias_kt").is_none() && s["tas_kt"] == 105.0
    });
    assert!(gap["attitude"].is_object());
    let stale = fault_snapshot(&app, |s| s["state"] == "stale");
    assert_sample_cleared(&initial, &stale);
    let fresh = fault_snapshot(&app, |s| {
        s["state"] == "live"
            && s["ias_kt"] == 100.0
            && s["attitude"]["pitch_deg"]
                .as_f64()
                .is_some_and(|p| (p - 0.1_f64.to_degrees()).abs() < 1e-9)
    });
    assert_eq!(fresh["tas_kt"], 105.0);
    app.assert_health();
    app.shutdown();
}
#[test]
fn focused_lifecycle_retries_pauses_inactivates_and_reconnects() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    let path = suite_path("simulator-lifecycle.jsonl");
    let log = fixture.0.join("interval-reconnect.jsonl");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--scenario",
            path.to_str().unwrap(),
            "--simconnect-interval",
            "1",
            "--log",
            log.to_str().unwrap(),
        ],
    );
    let waiting = fault_snapshot(&app, |s| s["state"] == "waiting");
    assert!(waiting["attitude"].is_null());
    let live = fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    assert_eq!(live["source"], "demo");
    for _ in 0..2 {
        let paused = fault_snapshot(&app, |s| s["state"] == "paused");
        assert!(paused["attitude"].is_null());
        let recovered = fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
        assert!(recovered["attitude"].is_object());
    }
    let inactive = fault_snapshot(&app, |s| s["state"] == "waiting");
    assert!(inactive["attitude"].is_null());
    fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    std::thread::sleep(Duration::from_secs(2));
    let disconnected = fault_snapshot(&app, |s| s["state"] == "disconnected");
    assert!(disconnected["attitude"].is_null());
    let reconnected = fault_snapshot(&app, |s| s["state"] == "live" && s["ias_kt"] == 90.0);
    assert!(reconnected["attitude"].is_object());
    let records = wait_fault_log(&log, |records| {
        records
            .iter()
            .filter(|v| v["kind"] == "grouped_request")
            .count()
            >= 2
    });
    assert!(
        records
            .iter()
            .filter(|v| v["kind"] == "grouped_request")
            .all(|v| v["data"]["interval"] == 1)
    );
    app.assert_health();
    app.shutdown();
}
#[test]
fn focused_sparse_scenarios_never_fabricate_live_attitude() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    for name in ["ias-only.jsonl", "event-only.jsonl", "no-signal.jsonl"] {
        let path = suite_path(name);
        let log = fixture.0.join(format!("{name}.log"));
        let mut app = Application::start(
            &binary,
            &fixture.0,
            &[
                "--scenario",
                path.to_str().unwrap(),
                "--log",
                log.to_str().unwrap(),
            ],
        );
        let mut stream = app.request("/api/v1/events");
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(6);
        let mut pending = String::new();
        let mut saw_waiting = false;
        let mut saw_paused = false;
        let mut saw_resume = false;
        while Instant::now() < deadline {
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
                Err(e) => panic!("{e}"),
            }
            while let Some(end) = pending.find('\n') {
                let line = pending[..end].trim_end_matches('\r').to_owned();
                pending.drain(..=end);
                if let Some(data) = line.strip_prefix("data: ")
                    && let Ok(s) = serde_json::from_str::<serde_json::Value>(data)
                {
                    assert_ne!(s["state"], "live", "{name}: {s}");
                    assert!(s["attitude"].is_null(), "{name}: {s}");
                    assert!(s.get("ias_kt").is_none(), "{name}: {s}");
                    saw_waiting |= s["state"] == "waiting";
                    saw_resume |= saw_paused && s["state"] == "waiting";
                    saw_paused |= s["state"] == "paused";
                }
            }
        }
        assert!(saw_waiting, "{name}");
        if name == "event-only.jsonl" {
            assert!(saw_paused && saw_resume, "{name}");
        }
        if name == "ias-only.jsonl" {
            let records =
                wait_fault_log(&log, |r| r.iter().any(|v| v["kind"] == "grouped_request"));
            assert!(!records.iter().any(|v| v["kind"] == "grouped_sample"));
        }
        app.assert_health();
        app.shutdown();
    }
}
