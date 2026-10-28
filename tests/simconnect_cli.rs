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
        fs::copy(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"), &binary).unwrap();
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
                    assert_eq!(snapshot["source"], "simconnect");
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
        let mut app = Application::start(&binary, &cwd, &["--source", "simconnect"]);
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
        assert!(messages.contains("--dll"), "{messages}");
        assert!(messages.contains("Listening on"), "{messages}");
        assert_eq!(
            fs::read_dir(&cwd).unwrap().count(),
            1,
            "no implicit diagnostics"
        );
    }
    fs::write(&expected, "invalid sibling DLL").unwrap();
    let override_path = fixture.0.join("missing-override.dll");
    let mut app = Application::start(
        &binary,
        &cwd,
        &[
            "--source",
            "simconnect",
            "--dll",
            override_path.to_str().unwrap(),
        ],
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
    let invalid = fixture.0.join("invalid.dll");
    fs::write(&invalid, "not a native library").unwrap();
    let system =
        PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/kernel32.dll");
    for (dll, expected) in [
        (&invalid, "LoadLibrary"),
        (&system, "missing export SimConnect_Open"),
    ] {
        let mut app = Application::start(
            &binary,
            &fixture.0,
            &["--source", "simconnect", "--dll", dll.to_str().unwrap()],
        );
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
                "simconnect",
                "--dll",
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
    let dll = fixture.0.join("missing.dll");
    let log = fixture.0.join("sessions/loader-failure.jsonl");
    for _ in 0..2 {
        let mut app = Application::start(
            &binary,
            &fixture.0,
            &[
                "--source",
                "simconnect",
                "--dll",
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
    let dll = fixture.0.join("must-not-load.dll");
    let mut app = Application::start(
        &binary,
        &fixture.0,
        &[
            "--source",
            "simconnect",
            "--dll",
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
    let dll = fixture.0.join("must-not-load.dll");
    let result = output(
        Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon")),
        &fixture.0,
        &[
            "--source",
            "simconnect",
            "--port",
            &port,
            "--dll",
            dll.to_str().unwrap(),
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
    assert!(!messages.contains("SimConnect DLL:"), "{messages}");
}

#[test]
fn invalid_source_specific_arguments_have_no_startup_side_effects() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    for arguments in [
        vec!["--source", "simconnect", "--dll", "relative.dll"],
        vec!["--source", "demo", "--dll", "file.dll"],
        vec!["--source", "demo", "--log", "must-not-exist/session.jsonl"],
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
    for arguments in [&[][..], &["--source", "demo"][..]] {
        let mut app = Application::start(&binary, &fixture.0, arguments);
        app.assert_health();
        let messages = app.messages();
        assert!(!messages.contains("SimConnect DLL:"), "{messages}");
    }
}
