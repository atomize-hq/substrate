use std::{ffi::OsStr, fs, io, process::Command};

use super::support::McpTestSandbox;

#[test]
fn sandboxes_use_distinct_paths() {
    let first = McpTestSandbox::new("support_smoke_first").expect("first sandbox");
    let second = McpTestSandbox::new("support_smoke_second").expect("second sandbox");

    assert_ne!(first.root(), second.root());
    assert_ne!(first.record_path(), second.record_path());
    assert_ne!(first.bin_dir(), second.bin_dir());
    assert_ne!(first.codex_home(), second.codex_home());
    assert_ne!(first.claude_home(), second.claude_home());

    assert!(first.bin_dir().is_dir());
    assert!(first.codex_home().is_dir());
    assert!(first.claude_home().is_dir());
    assert!(second.bin_dir().is_dir());
    assert!(second.codex_home().is_dir());
    assert!(second.claude_home().is_dir());
}

#[test]
fn installed_fake_binaries_use_platform_names_and_are_runnable() {
    let sandbox = McpTestSandbox::new("support_smoke_install").expect("sandbox");

    let fake_codex = sandbox.install_fake_codex().expect("install codex");
    let fake_claude = sandbox.install_fake_claude().expect("install claude");

    assert_eq!(
        fake_codex.file_name(),
        Some(OsStr::new(expected_binary_name("codex")))
    );
    assert_eq!(
        fake_claude.file_name(),
        Some(OsStr::new(expected_binary_name("claude")))
    );
    assert_eq!(fake_codex.parent(), Some(sandbox.bin_dir()));
    assert_eq!(fake_claude.parent(), Some(sandbox.bin_dir()));
    assert!(fake_codex.is_file());
    assert!(fake_claude.is_file());

    let codex_status = Command::new(&fake_codex)
        .env("FAKE_CODEX_MCP_RECORD_PATH", sandbox.record_path())
        .status()
        .expect("spawn fake codex");
    assert!(
        codex_status.success(),
        "fake codex should exit successfully"
    );

    let claude_status = Command::new(&fake_claude)
        .env("FAKE_CLAUDE_MCP_RECORD_PATH", sandbox.record_path())
        .status()
        .expect("spawn fake claude");
    assert!(
        claude_status.success(),
        "fake claude should exit successfully"
    );

    let records = sandbox.read_records().expect("read invocation records");
    assert_eq!(records.len(), 2, "expected one record per spawned binary");
}

#[test]
fn read_records_skips_blank_lines_and_preserves_order() {
    let sandbox = McpTestSandbox::new("support_smoke_parse").expect("sandbox");
    fs::write(
        sandbox.record_path(),
        concat!(
            "\n",
            "{\"args\":[\"mcp\",\"list\"],\"cwd\":\"/tmp/one\",\"env\":{\"ALPHA\":\"1\"}}\n",
            "\n",
            "{\"args\":[\"mcp\",\"get\",\"demo\"],\"cwd\":\"/tmp/two\",\"env\":{\"BETA\":\"2\",\"GAMMA\":\"3\"}}\n",
        ),
    )
    .expect("write records");

    let records = sandbox.read_records().expect("parse records");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].args, vec!["mcp", "list"]);
    assert_eq!(records[0].cwd, std::path::PathBuf::from("/tmp/one"));
    assert_eq!(records[0].env.get("ALPHA").map(String::as_str), Some("1"));
    assert_eq!(records[1].args, vec!["mcp", "get", "demo"]);
    assert_eq!(records[1].cwd, std::path::PathBuf::from("/tmp/two"));
    assert_eq!(records[1].env.get("BETA").map(String::as_str), Some("2"));
    assert_eq!(records[1].env.get("GAMMA").map(String::as_str), Some("3"));
}

#[test]
fn read_records_rejects_missing_required_fields() {
    let sandbox = McpTestSandbox::new("support_smoke_invalid").expect("sandbox");
    fs::write(
        sandbox.record_path(),
        "{\"args\":[\"mcp\",\"list\"],\"cwd\":\"/tmp/one\"}\n",
    )
    .expect("write invalid record");

    let err = sandbox
        .read_records()
        .expect_err("missing env should be rejected");

    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(
        err.to_string().contains("missing env"),
        "unexpected error: {err}"
    );
}

fn expected_binary_name(base: &str) -> &'static str {
    match base {
        "codex" if cfg!(windows) => "codex.exe",
        "claude" if cfg!(windows) => "claude.exe",
        "codex" => "codex",
        "claude" => "claude",
        _ => unreachable!("unexpected binary base name"),
    }
}
