use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::Duration,
};

use crate::{
    backends::opencode::{OpencodeBackend, OpencodeBackendConfig},
    AgentWrapperRunRequest,
};

static FAKE_BINARY: OnceLock<PathBuf> = OnceLock::new();

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn target_dir() -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("target"))
}

pub(super) fn target_debug_binary(name: &str) -> PathBuf {
    let binary_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    target_dir().join("debug").join(binary_name)
}

pub(super) fn fake_opencode_run_json_binary() -> PathBuf {
    FAKE_BINARY
        .get_or_init(|| {
            let binary = target_debug_binary("fake_opencode_run_json");
            if binary.exists() {
                return binary;
            }

            let output = Command::new("cargo")
                .args([
                    "build",
                    "-p",
                    "unified-agent-api-opencode",
                    "--bin",
                    "fake_opencode_run_json",
                    "--all-features",
                ])
                .current_dir(repo_root())
                .output()
                .expect("spawn cargo build for fake opencode binary");

            assert!(
                output.status.success(),
                "cargo build failed: status={:?}, stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                binary.exists(),
                "fake opencode binary should exist after cargo build"
            );
            binary
        })
        .clone()
}

pub(super) fn backend_with_env(env: BTreeMap<String, String>) -> OpencodeBackend {
    backend_with_config(OpencodeBackendConfig {
        binary: Some(fake_opencode_run_json_binary()),
        default_timeout: None,
        env,
    })
}

pub(super) fn backend_with_timeout(
    env: BTreeMap<String, String>,
    timeout: Duration,
) -> OpencodeBackend {
    backend_with_config(OpencodeBackendConfig {
        binary: Some(fake_opencode_run_json_binary()),
        default_timeout: Some(timeout),
        env,
    })
}

pub(super) fn backend_with_config(config: OpencodeBackendConfig) -> OpencodeBackend {
    OpencodeBackend::new(config)
}

pub(super) fn request(prompt: &str, working_dir: Option<&Path>) -> AgentWrapperRunRequest {
    AgentWrapperRunRequest {
        prompt: prompt.to_string(),
        working_dir: working_dir.map(Path::to_path_buf),
        ..Default::default()
    }
}

pub(super) fn capture_json(path: &Path) -> serde_json::Value {
    let bytes = fs::read(path).expect("read capture file");
    serde_json::from_slice(&bytes).expect("parse capture json")
}
