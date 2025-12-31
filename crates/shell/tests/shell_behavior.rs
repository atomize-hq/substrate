#![cfg(unix)]

mod support;

use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
#[cfg(not(target_os = "macos"))]
use std::process::{Command as StdCommand, Stdio};
#[cfg(not(target_os = "macos"))]
use std::time::Duration;
use substrate_shell::needs_shell;
#[cfg(not(target_os = "macos"))]
use support::{binary_path, ensure_substrate_built};
use support::{get_substrate_binary, temp_dir};

fn setup_isolated_home(temp: &tempfile::TempDir) -> PathBuf {
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).expect("create shims directory");
    fs::write(
        substrate_home.join("config.yaml"),
        "world:\n  enabled: true\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    )
    .expect("write default config");
    home
}

#[test]
fn test_builtin_cd_side_effects() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let target_dir = temp.path().join("test_dir");
    fs::create_dir(&target_dir).unwrap();

    let canonical_dir = target_dir.canonicalize().unwrap();
    let script = format!("cd {} && pwd", canonical_dir.display());

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            canonical_dir.to_string_lossy().to_string(),
        ));
}

#[test]
fn test_ci_flag_strict_mode_ordering() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("--no-world")
        .arg("--shell")
        .arg("/bin/bash")
        .arg("--ci")
        .arg("-c")
        .arg("echo $UNDEFINED_VAR")
        .assert()
        .failure();

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("--no-world")
        .arg("--shell")
        .arg("/bin/bash")
        .arg("-c")
        .arg("echo $UNDEFINED_VAR")
        .assert()
        .success();
}

#[test]
fn test_script_mode_single_process() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let script_file = temp.path().join("test.sh");

    fs::write(&script_file, "cd /tmp\npwd\nexport FOO=bar\necho $FOO").unwrap();

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .arg("-f")
        .arg(&script_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"))
        .stdout(predicate::str::contains("bar"));
}

#[test]
fn test_needs_shell_redirections() {
    assert!(needs_shell("echo hi 2>&1"));
    assert!(needs_shell("echo hi 1>/dev/null"));
    assert!(needs_shell("cat file 2>/dev/null"));
    assert!(needs_shell("cmd 1>&2"));
    assert!(needs_shell("echo test &>/dev/null"));

    assert!(!needs_shell("echo hello world"));
    assert!(!needs_shell("git status"));
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn test_sigterm_exit_code() {
    use std::thread;

    ensure_substrate_built();
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let substrate_bin = std::path::PathBuf::from(binary_path());

    let mut child = StdCommand::new(substrate_bin)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .arg("--no-world")
        .arg("-c")
        .arg("sleep 5")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(200));

    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM).unwrap();

    let status = child.wait().unwrap();
    let code = status.code();
    if let Some(code) = code {
        assert!(
            code == 143 || code == 0,
            "expected SIGTERM exit (143) or graceful shutdown (0), got {}",
            code
        );
    } else {
        use std::os::unix::process::ExitStatusExt;
        let signal = status.signal().unwrap_or_default();
        assert_eq!(
            signal,
            Signal::SIGTERM as i32,
            "expected SIGTERM signal ({}) but got {}",
            Signal::SIGTERM as i32,
            signal
        );
    }
}

#[test]
fn test_cd_minus_behavior() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SUBSTRATE_CAGED", "0")
        .arg("--uncaged")
        .arg("-c")
        .arg("cd /tmp && pwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"));

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        log_content.contains("cd /tmp"),
        "cd command should be logged"
    );
}

#[test]
fn test_export_complex_values_deferred() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export FOO=\"bar baz\" && echo $FOO")
        .assert()
        .success()
        .stdout(predicate::str::contains("bar baz"));
}
