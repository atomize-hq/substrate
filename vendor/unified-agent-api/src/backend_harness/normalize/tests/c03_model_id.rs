use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::json;

use super::super::{normalize_model_id_v1, normalize_request};
use super::support::PolicyFnAdapter;
use crate::backend_harness::BackendDefaults;
use crate::{AgentWrapperError, AgentWrapperRunRequest, EXT_AGENT_API_CONFIG_MODEL_V1};

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn normalize_model_id_v1_absent_returns_none() {
    let normalized = normalize_model_id_v1(None).expect("absent model id is allowed");
    assert_eq!(normalized, None);
}

#[test]
fn normalize_model_id_v1_rejects_non_string_without_echoing_value() {
    let raw = json!(false);
    let err = normalize_model_id_v1(Some(&raw)).expect_err("expected string parse failure");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.config.model.v1");
            assert!(!message.contains("false"));
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn normalize_model_id_v1_rejects_whitespace_only_after_trim() {
    let err = normalize_model_id_v1(Some(&json!("  \t \n  ")))
        .expect_err("expected whitespace-only failure");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.config.model.v1");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn normalize_model_id_v1_rejects_oversize_after_trim_without_echoing_value() {
    let raw = format!("  {}  ", "a".repeat(129));
    let err =
        normalize_model_id_v1(Some(&json!(raw.clone()))).expect_err("expected oversize failure");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.config.model.v1");
            assert!(!message.contains(&raw));
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn normalize_model_id_v1_trims_and_returns_success() {
    let normalized =
        normalize_model_id_v1(Some(&json!("  agent-model-1  "))).expect("expected trimmed success");
    assert_eq!(normalized, Some("agent-model-1".to_string()));
}

#[test]
fn bh_c03_agent_api_config_model_v1_invalid_values_use_safe_template_via_normalize_request() {
    const SUPPORTED: [&str; 1] = [EXT_AGENT_API_CONFIG_MODEL_V1];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();
    let secret = "SECRET_MODEL_ID_SHOULD_NOT_LEAK";
    let invalid_cases = vec![
        ("null", json!(null), Some("null".to_string())),
        ("bool", json!(false), Some("false".to_string())),
        ("number", json!(123), Some("123".to_string())),
        (
            "object",
            json!({ "model": secret }),
            Some(secret.to_string()),
        ),
        (
            "array",
            json!(["agent-model", secret]),
            Some(secret.to_string()),
        ),
        ("whitespace_only", json!("  \t \n  "), None),
        (
            "oversize_after_trim",
            json!(format!("  {}  ", "x".repeat(129))),
            Some("x".repeat(129)),
        ),
    ];

    for (name, raw, leak_probe) in invalid_cases {
        let mut request = AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        };
        request
            .extensions
            .insert(EXT_AGENT_API_CONFIG_MODEL_V1.to_string(), raw);

        let err = match normalize_request(&adapter, &defaults, request) {
            Ok(_) => panic!("expected invalid model id for case {name}"),
            Err(err) => err,
        };

        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, "invalid agent_api.config.model.v1");
                if let Some(leak_probe) = leak_probe {
                    assert!(
                        !message.contains(&leak_probe),
                        "case {name} leaked raw value into InvalidRequest message"
                    );
                }
            }
            other => panic!("expected InvalidRequest for case {name}, got: {other:?}"),
        }
    }
}

#[test]
fn bh_c03_agent_api_config_model_v1_trims_before_mapping_via_normalize_request() {
    const SUPPORTED: [&str; 1] = [EXT_AGENT_API_CONFIG_MODEL_V1];
    let adapter = PolicyFnAdapter::ok_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        EXT_AGENT_API_CONFIG_MODEL_V1.to_string(),
        json!("  agent-model-1  "),
    );

    let normalized =
        normalize_request(&adapter, &defaults, request).expect("expected normalized request");
    assert_eq!(normalized.model_id, Some("agent-model-1".to_string()));
}

#[test]
fn bh_r0_agent_api_config_model_v1_is_rejected_before_value_shape_validation_via_normalize_request()
{
    const SUPPORTED: [&str; 1] = ["backend.toy.example"];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();

    for raw in [json!(false), json!("  \t \n  "), json!("x".repeat(256))] {
        let mut request = AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        };
        request
            .extensions
            .insert(EXT_AGENT_API_CONFIG_MODEL_V1.to_string(), raw.clone());

        let err = match normalize_request(&adapter, &defaults, request) {
            Ok(_) => panic!("unsupported model key must fail closed"),
            Err(err) => err,
        };
        match err {
            AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability,
            } => {
                assert_eq!(agent_kind, "toy");
                assert_eq!(capability, EXT_AGENT_API_CONFIG_MODEL_V1);
            }
            other => panic!("expected UnsupportedCapability, got: {other:?}"),
        }
    }
}

#[test]
fn bh_r0_agent_api_config_model_v1_is_confined_to_normalize_rs_in_production_code() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_normalize = src_root.join("backend_harness/normalize.rs");
    let allowed_constant_home = src_root.join("lib.rs");

    let mut files = Vec::new();
    collect_rs_files(&src_root, &mut files);

    let mut literal_offenders = Vec::new();
    let mut raw_access_offenders = Vec::new();
    for path in files {
        let is_test_source = path
            .components()
            .any(|component| component.as_os_str() == "tests")
            || path.file_name().and_then(|name| name.to_str()) == Some("tests.rs");

        if is_test_source {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("read source file");
        if path != allowed_normalize
            && path != allowed_constant_home
            && contents.contains(EXT_AGENT_API_CONFIG_MODEL_V1)
        {
            literal_offenders.push(
                path.strip_prefix(&src_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
            );
        }

        if path != allowed_normalize
            && (contents.contains("extensions.get(EXT_AGENT_API_CONFIG_MODEL_V1)")
                || contents.contains("extensions.get(crate::EXT_AGENT_API_CONFIG_MODEL_V1)")
                || contents.contains("extensions.contains_key(EXT_AGENT_API_CONFIG_MODEL_V1)")
                || contents
                    .contains("extensions.contains_key(crate::EXT_AGENT_API_CONFIG_MODEL_V1)")
                || contents.contains("[EXT_AGENT_API_CONFIG_MODEL_V1]")
                || contents.contains("[crate::EXT_AGENT_API_CONFIG_MODEL_V1]"))
        {
            raw_access_offenders.push(
                path.strip_prefix(&src_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        literal_offenders.is_empty(),
        "agent_api.config.model.v1 literal leaked outside normalize.rs/lib.rs and tests: {literal_offenders:?}"
    );
    assert!(
        raw_access_offenders.is_empty(),
        "raw model-selection extension access escaped normalize.rs: {raw_access_offenders:?}"
    );
}
