use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ExplainV1 {
    kind: String,
    keys: std::collections::BTreeMap<String, ExplainKey>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ExplainKey {
    merge_strategy: String,
    sources: Vec<ExplainSource>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ExplainSource {
    layer: String,
    path: Option<String>,
}

fn run_current_show_output(workspace_root: &Path, substrate_home: &Path) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_substrate"));
    cmd.current_dir(workspace_root)
        .env("SUBSTRATE_HOME", substrate_home)
        .env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env_remove("SUBSTRATE_OVERRIDE_ANCHOR_MODE")
        .env_remove("SUBSTRATE_OVERRIDE_ANCHOR_PATH")
        .env_remove("SUBSTRATE_OVERRIDE_CAGED")
        .env_remove("SUBSTRATE_OVERRIDE_POLICY_MODE")
        .env_remove("SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC")
        .env_remove("SUBSTRATE_OVERRIDE_SYNC_DIRECTION")
        .env_remove("SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY")
        .env_remove("SUBSTRATE_OVERRIDE_SYNC_EXCLUDE")
        .args(["config", "current", "show", "--json", "--explain"]);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn run_current_show_json_explain(
    workspace_root: &Path,
    substrate_home: &Path,
) -> (Vec<u8>, Vec<u8>, ExplainV1) {
    let out = run_current_show_output(workspace_root, substrate_home);
    let stderr_text = String::from_utf8_lossy(&out.stderr);
    let json_start = stderr_text.find('{').unwrap_or_else(|| {
        panic!("failed to locate JSON object in --explain stderr:\n{stderr_text}")
    });
    let explain: ExplainV1 = serde_json::from_str(&stderr_text[json_start..]).unwrap();
    (out.stdout, out.stderr, explain)
}

fn json_enabled_list(stdout: &[u8]) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_slice(stdout).unwrap();
    v.pointer("/world/deps/enabled")
        .and_then(|v| v.as_array())
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn test_wcu2_phase_a_explain_contract_global_only() {
    let tmp = TempTree::new("substrate-wcu2-explain-global-only");

    let substrate_home = tmp.path().join(".substrate");
    fs::create_dir_all(&substrate_home).unwrap();
    write_file(
        &substrate_home.join("config.yaml"),
        r#"
world:
  deps:
    enabled: ["a"]
"#,
    );

    let workspace_root = tmp.path().join("ws");
    write_file(
        &workspace_root.join(".substrate/workspace.yaml"),
        "# workspace patch\n",
    );

    let (stdout, _stderr, explain) =
        run_current_show_json_explain(&workspace_root, &substrate_home);
    assert_eq!(explain.kind, "substrate.config.explain.v1");
    assert_eq!(json_enabled_list(&stdout), vec!["a".to_string()]);

    let enabled = explain.keys.get("world.deps.enabled").unwrap();
    assert_eq!(enabled.merge_strategy, "concat_dedupe_ordered_set");
    assert_eq!(enabled.sources.len(), 1);
    assert_eq!(enabled.sources[0].layer, "global_patch");
    assert!(enabled.sources[0]
        .path
        .as_ref()
        .unwrap()
        .ends_with("config.yaml"));
}

#[test]
fn test_wcu2_phase_a_explain_contract_workspace_only() {
    let tmp = TempTree::new("substrate-wcu2-explain-workspace-only");

    let substrate_home = tmp.path().join(".substrate");
    fs::create_dir_all(&substrate_home).unwrap();

    let workspace_root = tmp.path().join("ws");
    write_file(
        &workspace_root.join(".substrate/workspace.yaml"),
        r#"
world:
  deps:
    enabled: ["b", "c"]
"#,
    );

    let (stdout, _stderr, explain) =
        run_current_show_json_explain(&workspace_root, &substrate_home);
    assert_eq!(
        json_enabled_list(&stdout),
        vec!["b".to_string(), "c".to_string()]
    );

    let enabled = explain.keys.get("world.deps.enabled").unwrap();
    assert_eq!(enabled.sources.len(), 1);
    assert_eq!(enabled.sources[0].layer, "workspace_patch");
    assert!(enabled.sources[0]
        .path
        .as_ref()
        .unwrap()
        .ends_with("workspace.yaml"));
}

#[test]
fn test_wcu2_phase_a_explain_contract_workspace_disabled_ignores_workspace_patch() {
    let tmp = TempTree::new("substrate-wcu2-explain-workspace-disabled");

    let substrate_home = tmp.path().join(".substrate");
    fs::create_dir_all(&substrate_home).unwrap();
    write_file(
        &substrate_home.join("config.yaml"),
        r#"
world:
  deps:
    enabled: ["a"]
"#,
    );

    let workspace_root = tmp.path().join("ws");
    write_file(
        &workspace_root.join(".substrate/workspace.yaml"),
        r#"
world:
  deps:
    enabled: ["b"]
"#,
    );
    write_file(
        &workspace_root.join(".substrate/workspace.disabled"),
        "disabled\n",
    );

    let (stdout, _stderr, explain) =
        run_current_show_json_explain(&workspace_root, &substrate_home);
    assert_eq!(json_enabled_list(&stdout), vec!["a".to_string()]);

    let enabled = explain.keys.get("world.deps.enabled").unwrap();
    assert_eq!(enabled.sources.len(), 1);
    assert_eq!(enabled.sources[0].layer, "global_patch");
}

#[test]
fn test_wcu2_phase_a_explain_stderr_is_deterministic_bytes() {
    let tmp = TempTree::new("substrate-wcu2-explain");

    let substrate_home = tmp.path().join(".substrate");
    fs::create_dir_all(&substrate_home).unwrap();
    write_file(
        &substrate_home.join("config.yaml"),
        r#"
world:
  deps:
    enabled: ["a", "b"]
"#,
    );

    let workspace_root = tmp.path().join("ws");
    write_file(
        &workspace_root.join(".substrate/workspace.yaml"),
        r#"
world:
  deps:
    enabled: ["b", "c"]
"#,
    );

    let out1 = run_current_show_output(&workspace_root, &substrate_home);
    let out2 = run_current_show_output(&workspace_root, &substrate_home);

    assert!(out1
        .stderr
        .windows(b"substrate.config.explain.v1".len())
        .any(|w| w == b"substrate.config.explain.v1"));
    assert_eq!(out1.stderr, out2.stderr);
    assert_eq!(out1.stdout, out2.stdout);
}
