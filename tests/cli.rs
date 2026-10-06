use std::{
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn cli(args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_main"))
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
fn invalid_arguments_and_unavailable_source_are_actionable() {
    for (args, expected) in [
        (vec!["--port", "0"], "--port"),
        (vec!["--port", "65536"], "--port"),
        (vec!["--listen-address", "not-an-ip"], "--listen-address"),
        (vec!["--listen-address", "224.0.0.1"], "multicast"),
        (vec!["--source", "unknown"], "--source"),
        (vec!["--source", "simconnect"], "not implemented"),
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
    let result = cli(&["--port", &address.port().to_string()]);
    assert!(!result.status.success());
    let error = String::from_utf8(result.stderr).unwrap();
    assert!(error.contains(&address.to_string()));
    assert!(error.contains("choose an available"));
}
