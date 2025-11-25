#![cfg(unix)]

mod support;

use std::fs;
use support::{get_substrate_binary, temp_dir};

#[test]
fn test_redaction_header_values() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
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
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
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
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_LOG_OPTS", "raw")
        .arg("-c")
        .arg("echo 'Authorization: Bearer secret123'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("secret123"));
}
