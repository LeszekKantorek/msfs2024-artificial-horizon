use std::{
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn scenario() -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("demo-scenarios/demo-scenario.jsonl")
        .to_string_lossy()
        .into_owned()
}

fn cli(args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!("CLI did not exit within five seconds: {args:?}; {output:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

#[test]
fn help_and_version_work_without_starting_the_server() {
    let help = cli(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for option in [
        "--source",
        "--listen-address",
        "--port",
        "--scenario",
        "--log",
        "--help",
        "--version",
    ] {
        assert!(text.contains(option));
    }
    let version = cli(&["--version"]);
    assert!(version.status.success());
    assert!(
        String::from_utf8(version.stdout)
            .unwrap()
            .contains(env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_arguments_are_actionable() {
    for (args, expected) in [
        (vec![], "--scenario"),
        (vec!["--source", "demo.dll"], "--scenario"),
        (vec!["--port", "0"], "--port"),
        (vec!["--port", "65536"], "--port"),
        (vec!["--listen-address", "not-an-ip"], "--listen-address"),
        (
            vec!["--scenario", "x", "--listen-address", "224.0.0.1"],
            "multicast",
        ),
        (vec!["--source", "unknown"], "--source"),
    ] {
        let result = cli(&args);
        assert!(!result.status.success());
        assert!(String::from_utf8(result.stderr).unwrap().contains(expected));
    }
}

#[test]
fn occupied_port_exits_with_address_and_actionable_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let result = cli(&[
        "--scenario",
        &scenario(),
        "--port",
        &address.port().to_string(),
    ]);
    assert!(!result.status.success());
    let error = String::from_utf8(result.stderr).unwrap();
    assert!(error.contains(&address.to_string()));
    assert!(error.contains("choose an available"));
}

struct ChildGuard(std::process::Child);

impl std::ops::Deref for ChildGuard {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ChildGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn stop_process(input: &[u8]) {
    use std::io::Write;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"))
            .args(["--scenario", &scenario(), "--port", &port.to_string()])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while TcpListener::bind(("127.0.0.1", port)).is_ok() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"\nunknown\n")
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(child.try_wait().unwrap().is_none());
    child.stdin.as_mut().unwrap().write_all(input).unwrap();
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            panic!("stop did not exit");
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(child.wait().unwrap().success());
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    drop(listener);
    let mut restarted = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"))
            .args(["--scenario", &scenario(), "--port", &port.to_string()])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while TcpListener::bind(("127.0.0.1", port)).is_ok() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    restarted
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"stop\n")
        .unwrap();
    while restarted.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            panic!("restart did not exit");
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(restarted.wait().unwrap().success());
}

#[test]
fn stop_lf_exits_and_releases_port() {
    stop_process(b"stop\n");
}
#[test]
fn stop_crlf_exits_and_releases_port() {
    stop_process(b"stop\r\n");
}
#[test]
fn stdin_eof_does_not_stop_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_msfs2024-artificial-horizon"))
            .args(["--scenario", &scenario(), "--port", &port.to_string()])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    drop(child.stdin.take());
    thread::sleep(Duration::from_millis(500));
    assert!(child.try_wait().unwrap().is_none());
    child.kill().unwrap();
    child.wait().unwrap();
}
#[test]
fn shutdown_file_option_is_removed() {
    assert!(
        !String::from_utf8(cli(&["--help"]).stdout)
            .unwrap()
            .contains("--shutdown-file")
    );
    assert!(!cli(&["--shutdown-file", "anything"]).status.success());
}
