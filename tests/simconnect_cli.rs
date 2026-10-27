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

#[test]
fn default_dll_is_beside_executable_and_explicit_missing_dll_does_not_fall_back() {
    let fixture = Fixture::new();
    let cwd = fixture.0.join("working-directory");
    fs::create_dir(&cwd).unwrap();
    // A file in the working directory must not influence default selection.
    fs::write(
        cwd.join("SimConnect.dll"),
        "invalid DLL in working directory",
    )
    .unwrap();
    let binary = fixture.binary();
    let expected = fixture.0.join("SimConnect.dll");
    let result = output(&binary, &cwd, &["--source", "simconnect"]);
    assert!(!result.status.success());
    let messages = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(messages.contains(expected.to_str().unwrap()), "{messages}");
    assert!(messages.contains("--dll"), "{messages}");
    assert!(!messages.contains("Listening on"));
    assert_eq!(
        fs::read_dir(&cwd).unwrap().count(),
        1,
        "no implicit diagnostic files"
    );

    fs::write(&expected, "invalid DLL beside executable").unwrap();
    let override_path = fixture.0.join("missing-override.dll");
    let result = output(
        &binary,
        &cwd,
        &[
            "--source",
            "simconnect",
            "--dll",
            override_path.to_str().unwrap(),
        ],
    );
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains(override_path.to_str().unwrap()), "{error}");
    assert!(
        !error.contains(expected.to_str().unwrap()),
        "must not try sibling DLL: {error}"
    );
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
        let result = output(
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
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stdout).contains("Diagnostic session completed"));
    }
    let records: Vec<serde_json::Value> = fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let starts: Vec<_> = records
        .iter()
        .filter(|record| record["kind"] == "session_start")
        .collect();
    let ends: Vec<_> = records
        .iter()
        .filter(|record| record["kind"] == "session_end")
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(ends.len(), 2);
    assert_ne!(starts[0]["session_id"], starts[1]["session_id"]);
    assert!(
        starts
            .iter()
            .all(|record| record["data"].get("scenario").is_none())
    );
    assert!(ends.iter().all(|record| record["data"]["success"] == false));
}

#[test]
fn a_log_open_failure_prevents_the_native_load_attempt() {
    let fixture = Fixture::new();
    let dll = fixture.0.join("must-not-load.dll");
    let result = output(
        Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon")),
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
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        error.contains("log") || error.contains("diagnostic"),
        "{error}"
    );
    assert!(!error.contains("LoadLibrary"), "{error}");
}

#[test]
fn simconnect_rejects_http_options_even_when_they_equal_demo_defaults() {
    let fixture = Fixture::new();
    for (option, value) in [("--port", "8080"), ("--listen-address", "127.0.0.1")] {
        let result = output(
            Path::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon")),
            &fixture.0,
            &["--source", "simconnect", option, value],
        );
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("issue #6"), "{error}");
        assert!(!error.contains("LoadLibrary"));
    }
}

#[test]
fn demo_serves_http_with_an_invalid_sibling_dll() {
    let fixture = Fixture::new();
    let binary = fixture.binary();
    fs::write(fixture.0.join("SimConnect.dll"), "not a native library").unwrap();
    for explicit_source in [false, true] {
        let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reservation.local_addr().unwrap();
        drop(reservation);
        let mut command = Command::new(&binary);
        command
            .current_dir(&fixture.0)
            .arg("--port")
            .arg(address.port().to_string());
        if explicit_source {
            command.args(["--source", "demo"]);
        }
        let mut child = RunningChild(
            command
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            if let Ok(stream) = TcpStream::connect_timeout(&address, Duration::from_millis(100)) {
                break stream;
            }
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "demo exited before serving HTTP"
            );
            assert!(Instant::now() < deadline, "demo HTTP did not start");
            std::thread::sleep(Duration::from_millis(20));
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("{\"status\":\"ok\"}"), "{response}");
    }
}
