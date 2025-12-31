#![cfg(unix)]

mod support;

use std::fs;
use std::path::PathBuf;
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
fn test_redaction_header_values() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export API_TOKEN=secret123 && echo 'test'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        log_content.contains("API_TOKEN=***"),
        "Expected redacted token in log: {}",
        log_content
    );
    assert!(
        !log_content.contains("secret123"),
        "Secret should be redacted in log: {}",
        log_content
    );
}

#[test]
fn test_redaction_user_pass() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export DB_PASSWORD=secretpass && echo 'test'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        log_content.contains("DB_PASSWORD=***"),
        "Expected redacted password in log: {}",
        log_content
    );
    assert!(
        !log_content.contains("secretpass"),
        "Password should be redacted in log: {}",
        log_content
    );
}

#[test]
fn test_raw_mode_no_redaction() {
    let temp = temp_dir("substrate-test-");
    let home = setup_isolated_home(&temp);
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", home.join(".substrate"))
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_LOG_OPTS", "raw")
        .arg("-c")
        .arg("echo 'Authorization: Bearer secret123'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("secret123"));
}
