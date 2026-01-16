use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

    let run = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_substrate"));
        cmd.current_dir(&workspace_root)
            .env("SUBSTRATE_HOME", &substrate_home)
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
    };

    let out1 = run();
    let out2 = run();

    assert!(out1
        .stderr
        .windows(b"substrate.config.explain.v1".len())
        .any(|w| w == b"substrate.config.explain.v1"));
    assert_eq!(out1.stderr, out2.stderr);
}
