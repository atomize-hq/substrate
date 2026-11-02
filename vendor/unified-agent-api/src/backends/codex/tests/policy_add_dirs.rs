use std::fs;

use serde_json::json;
use tempfile::tempdir;

use super::support::*;

#[cfg(windows)]
use std::path::{Component, Path, Prefix};

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";

#[test]
fn codex_policy_add_dirs_absent_key_returns_empty_vec() {
    let adapter = test_adapter();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert!(policy.add_dirs.is_empty());
}

#[test]
fn codex_policy_add_dirs_supported_extension_allowlist_admits_key() {
    let adapter = test_adapter();

    assert!(adapter
        .supported_extension_keys()
        .contains(&EXT_ADD_DIRS_V1));
}

#[test]
fn codex_policy_add_dirs_normalizes_absolute_and_relative_entries_with_stable_order() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request-root");
    let relative_target = request_root.join("docs");
    let absolute_target = temp.path().join("shared-docs");
    let absolute_target_text = absolute_target.to_string_lossy().into_owned();
    fs::create_dir_all(&relative_target).expect("create relative target");
    fs::create_dir_all(&absolute_target).expect("create absolute target");

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [(
            EXT_ADD_DIRS_V1.to_string(),
            add_dirs_payload(&[
                "docs",
                absolute_target_text.as_str(),
                "./docs",
                absolute_target_text.as_str(),
            ]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let policy = test_adapter()
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![relative_target, absolute_target]);
}

#[test]
fn codex_policy_add_dirs_request_working_dir_beats_default_and_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request_root");
    let default_root = temp.path().join("default_root");
    let run_start_root = temp.path().join("run_start_root");
    let request_target = request_root.join("rel-target");
    let default_target = default_root.join("rel-target");
    let run_start_target = run_start_root.join("rel-target");
    fs::create_dir_all(&request_target).expect("create request target");
    fs::create_dir_all(&default_target).expect("create default target");
    fs::create_dir_all(&run_start_target).expect("create run-start target");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            default_working_dir: Some(default_root),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [(
            EXT_ADD_DIRS_V1.to_string(),
            add_dirs_payload(&["rel-target"]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![request_target]);
}

#[test]
fn codex_policy_add_dirs_default_working_dir_beats_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let default_root = temp.path().join("default_root");
    let run_start_root = temp.path().join("run_start_root");
    let default_target = default_root.join("rel-target");
    let run_start_target = run_start_root.join("rel-target");
    fs::create_dir_all(&default_target).expect("create default target");
    fs::create_dir_all(&run_start_target).expect("create run-start target");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            default_working_dir: Some(default_root),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(
            EXT_ADD_DIRS_V1.to_string(),
            add_dirs_payload(&["rel-target"]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![default_target]);
}

#[test]
fn codex_policy_add_dirs_run_start_cwd_is_final_fallback() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run_start_root");
    let run_start_target = run_start_root.join("rel-target");
    fs::create_dir_all(&run_start_target).expect("create run-start target");

    let adapter = test_adapter_with_run_start_cwd(Some(run_start_root));
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(
            EXT_ADD_DIRS_V1.to_string(),
            add_dirs_payload(&["rel-target"]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![run_start_target]);
}

#[test]
fn codex_policy_add_dirs_accepts_absolute_entries_without_effective_working_dir() {
    let temp = tempdir().expect("tempdir");
    let absolute_docs = temp.path().join("shared-context");
    fs::create_dir_all(&absolute_docs).expect("create absolute add-dir");
    let absolute_docs_text = absolute_docs.to_string_lossy().into_owned();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(
            EXT_ADD_DIRS_V1.to_string(),
            add_dirs_payload(&[absolute_docs_text.as_str()]),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let policy = test_adapter()
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![absolute_docs]);
}

#[test]
fn codex_policy_add_dirs_relative_entries_without_effective_working_dir_fail_safely() {
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let err = adapter_error(test_adapter().validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn codex_policy_add_dirs_invalid_input_beats_fork_handling_and_stays_redacted() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request_root");
    fs::create_dir_all(&request_root).expect("create request root");
    let leaked = "missing-secret-dir";

    let adapter = test_adapter_with_run_start_cwd(Some(temp.path().join("run_start_root")));
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [
            (EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&[leaked])),
            (
                EXT_SESSION_FORK_V1.to_string(),
                json!({
                    "selector": "last",
                }),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let err = adapter_error(adapter.validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
    assert!(
        !err.to_string().contains(leaked),
        "error display leaked raw path text"
    );
}

#[test]
fn codex_policy_add_dirs_resolves_relative_request_working_dir_from_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let request_docs = run_start_root.join("repo").join("docs");
    fs::create_dir_all(&request_docs).expect("create request docs");

    let adapter = test_adapter_with_run_start_cwd(Some(run_start_root));
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(std::path::PathBuf::from("repo")),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![request_docs]);
}

#[test]
fn codex_policy_add_dirs_resolves_relative_default_working_dir_from_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let default_docs = run_start_root.join("repo").join("docs");
    fs::create_dir_all(&default_docs).expect("create default docs");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            default_working_dir: Some(std::path::PathBuf::from("repo")),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![default_docs]);
}

#[cfg(windows)]
#[test]
fn codex_policy_add_dirs_cross_drive_drive_relative_working_dir_rejects_relative_entries_safely() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let adapter = test_adapter_with_run_start_cwd(Some(run_start_root.clone()));
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(windows_drive_relative_on_other_drive(
            "repo",
            &run_start_root,
        )),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let err = adapter_error(adapter.validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

fn adapter_error(
    result: Result<super::super::CodexExecPolicy, AgentWrapperError>,
) -> AgentWrapperError {
    result.expect_err("policy extraction should fail")
}

#[cfg(windows)]
fn windows_drive_relative_on_other_drive(
    relative: &str,
    absolute_path: &Path,
) -> std::path::PathBuf {
    let current_drive = absolute_path
        .components()
        .find_map(|component| match component {
            Component::Prefix(value) => match value.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                    Some(drive.to_ascii_lowercase())
                }
                _ => None,
            },
            _ => None,
        })
        .expect("absolute windows path should include a disk prefix");
    let alternate_drive = if current_drive == b'c' { 'd' } else { 'c' };
    std::path::PathBuf::from(format!("{alternate_drive}:{relative}"))
}
