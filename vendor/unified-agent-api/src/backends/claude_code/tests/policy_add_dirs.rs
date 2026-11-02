use std::fs;

use tempfile::tempdir;

use super::support::*;

#[cfg(windows)]
use std::path::{Component, Path, Prefix};

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";

#[test]
fn claude_policy_add_dirs_absent_key_returns_empty_vec() {
    let adapter = new_adapter();
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
fn claude_policy_add_dirs_supported_extension_allowlist_admits_key() {
    let adapter = new_adapter();
    assert!(adapter
        .supported_extension_keys()
        .contains(&EXT_ADD_DIRS_V1));

    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request-root");
    let request_docs = request_root.join("docs");
    fs::create_dir_all(&request_docs).expect("create request docs");

    let defaults = crate::backend_harness::BackendDefaults::default();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    crate::backend_harness::normalize_request(&adapter, &defaults, request)
        .expect("add-dir extension key should pass Claude allowlist gating");
}

#[test]
fn claude_policy_add_dirs_request_working_dir_beats_default_and_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request-root");
    let default_root = temp.path().join("default-root");
    let run_start_root = temp.path().join("run-start-root");
    let request_docs = request_root.join("docs");
    let default_docs = default_root.join("docs");
    let run_start_docs = run_start_root.join("docs");
    fs::create_dir_all(&request_docs).expect("create request docs");
    fs::create_dir_all(&default_docs).expect("create default docs");
    fs::create_dir_all(&run_start_docs).expect("create run-start docs");

    let adapter = new_adapter_with_config_and_run_start_cwd(
        ClaudeCodeBackendConfig {
            default_working_dir: Some(default_root),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
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
fn claude_policy_add_dirs_backend_default_beats_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let default_root = temp.path().join("default-root");
    let run_start_root = temp.path().join("run-start-root");
    let default_docs = default_root.join("docs");
    let run_start_docs = run_start_root.join("docs");
    fs::create_dir_all(&default_docs).expect("create default docs");
    fs::create_dir_all(&run_start_docs).expect("create run-start docs");

    let adapter = new_adapter_with_config_and_run_start_cwd(
        ClaudeCodeBackendConfig {
            default_working_dir: Some(default_root),
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

#[test]
fn claude_policy_add_dirs_run_start_cwd_is_final_fallback() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start-root");
    let run_start_docs = run_start_root.join("docs");
    fs::create_dir_all(&run_start_docs).expect("create run-start docs");

    let adapter = new_adapter_with_run_start_cwd(Some(run_start_root));
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

    assert_eq!(policy.add_dirs, vec![run_start_docs]);
}

#[test]
fn claude_policy_add_dirs_accepts_absolute_entries_without_effective_working_dir() {
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

    let policy = new_adapter()
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![absolute_docs]);
    assert_eq!(policy.resolved_working_dir, None);
}

#[test]
fn claude_policy_add_dirs_relative_entries_without_effective_working_dir_fail_safely() {
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let err = adapter_error(new_adapter().validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn claude_policy_add_dirs_invalid_input_uses_safe_message_without_leakage() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request-root");
    fs::create_dir_all(&request_root).expect("create request root");
    let leaked = "missing-secret-dir";
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&[leaked]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let err = adapter_error(new_adapter().validate_and_extract_policy(&request));
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
fn claude_policy_add_dirs_resolves_relative_request_working_dir_from_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let request_docs = run_start_root.join("repo").join("docs");
    fs::create_dir_all(&request_docs).expect("create request docs");

    let adapter = new_adapter_with_run_start_cwd(Some(run_start_root));
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
fn claude_policy_add_dirs_resolves_relative_default_working_dir_from_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let default_docs = run_start_root.join("repo").join("docs");
    fs::create_dir_all(&default_docs).expect("create default docs");

    let adapter = new_adapter_with_config_and_run_start_cwd(
        ClaudeCodeBackendConfig {
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

#[test]
fn claude_policy_relative_working_dir_without_run_start_cwd_fails_safely() {
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(std::path::PathBuf::from("repo")),
        ..Default::default()
    };

    let err = adapter_error(new_adapter().validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message,
                super::super::util::PINNED_WORKING_DIR_RESOLUTION_FAILURE
            );
        }
        other => panic!("expected Backend, got: {other:?}"),
    }
}

#[cfg(windows)]
#[test]
fn claude_policy_cross_drive_drive_relative_working_dir_rejects_before_add_dir_resolution() {
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let adapter = new_adapter_with_run_start_cwd(Some(run_start_root.clone()));
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
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message,
                super::super::util::PINNED_WORKING_DIR_RESOLUTION_FAILURE
            );
        }
        other => panic!("expected Backend, got: {other:?}"),
    }
}

fn adapter_error(
    result: Result<super::super::harness::ClaudeExecPolicy, AgentWrapperError>,
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
