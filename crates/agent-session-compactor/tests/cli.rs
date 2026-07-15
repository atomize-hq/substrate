use agent_session_compactor::cli::Cli;
use anyhow as _;
use blake3 as _;
use camino as _;
use clap::Parser;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile as _;
use thiserror as _;
use time as _;
use walkdir as _;

#[test]
fn cli_linked_children_requires_an_explicit_session_id() {
    let error = Cli::try_parse_from([
        "agent-session-compactor",
        "--include-linked-children",
        "--output-dir",
        "/tmp/output",
    ])
    .expect_err("linked-child mode must require --session-id");

    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn cli_linked_children_maps_to_the_library_config() {
    let cli = Cli::try_parse_from([
        "agent-session-compactor",
        "--session-id",
        "session-parent-alpha",
        "--include-linked-children",
        "--output-dir",
        "/tmp/output",
    ])
    .expect("valid linked-child arguments");

    let config = cli.run_config();
    assert_eq!(config.session_id.as_deref(), Some("session-parent-alpha"));
    assert!(config.include_linked_children);
}
