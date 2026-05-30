use anyhow as _;
use blake3 as _;
use std::{fs, path::Path};

use agent_session_compactor::discovery::{
    discover_session_artifacts_in_home, resolve_codex_home_from, DiscoveryError,
};
use camino::Utf8Path;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use walkdir as _;

#[test]
fn discovery_resolves_codex_home_in_required_order() {
    let explicit = resolve_codex_home_from(
        Some(camino::Utf8PathBuf::from("/explicit/.codex")),
        Some("/env/.codex".into()),
        Some("/home/user".into()),
    )
    .expect("explicit path wins");
    assert_eq!(explicit, Utf8Path::new("/explicit/.codex"));

    let env_only =
        resolve_codex_home_from(None, Some("/env/.codex".into()), Some("/home/user".into()))
            .expect("env path wins without explicit");
    assert_eq!(env_only, Utf8Path::new("/env/.codex"));

    let home_only = resolve_codex_home_from(None, None, Some("/home/user".into()))
        .expect("home fallback appends .codex");
    assert_eq!(home_only, Utf8Path::new("/home/user/.codex"));
}

#[test]
fn discovery_fails_clearly_when_sessions_dir_is_missing() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path");

    let err = discover_session_artifacts_in_home(codex_home, None).expect_err("missing sessions");
    assert!(matches!(err, DiscoveryError::MissingSessionsDirectory(_)));
}

#[test]
fn discovery_sorts_paths_lexicographically() {
    let temp_dir = seeded_codex_home(&[
        "sessions/2026/05/29/rollout-z.jsonl",
        "sessions/2026/05/28/rollout-a.jsonl",
        "sessions/2026/05/29/session-123-notes.txt",
    ]);
    let codex_home = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path");

    let artifacts = discover_session_artifacts_in_home(codex_home, None).expect("discover files");
    let paths: Vec<_> = artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .collect();

    assert_eq!(
        paths,
        vec![
            codex_home.join("sessions/2026/05/28/rollout-a.jsonl"),
            codex_home.join("sessions/2026/05/29/rollout-z.jsonl"),
            codex_home.join("sessions/2026/05/29/session-123-notes.txt"),
        ]
    );
}

#[test]
fn discovery_filters_by_session_id_substring() {
    let temp_dir = seeded_codex_home(&[
        "sessions/2026/05/29/rollout-session-123.jsonl",
        "sessions/2026/05/29/rollout-session-999.jsonl",
        "sessions/2026/05/29/session-123-audit.txt",
    ]);
    let codex_home = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path");

    let artifacts =
        discover_session_artifacts_in_home(codex_home, Some("session-123")).expect("filtered");
    let paths: Vec<_> = artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .collect();

    assert_eq!(
        paths,
        vec![
            codex_home.join("sessions/2026/05/29/rollout-session-123.jsonl"),
            codex_home.join("sessions/2026/05/29/session-123-audit.txt"),
        ]
    );
}

fn seeded_codex_home(files: &[&str]) -> TempDir {
    let temp_dir = TempDir::new().expect("temp dir");
    for relative_path in files {
        let path = temp_dir.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(path, "{}\n").expect("write test file");
    }
    temp_dir
}

#[allow(dead_code)]
fn _assert_path_exists(path: &Path) {
    assert!(path.exists());
}
