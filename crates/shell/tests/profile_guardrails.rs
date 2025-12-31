#![cfg(unix)]

#[path = "common.rs"]
mod common;

use common::{substrate_shell_driver, temp_dir};
use std::fs;

#[test]
fn invalid_substrate_profile_fails_fast() {
    let temp = temp_dir("substrate-profile-invalid-");
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&home).expect("create home");
    fs::create_dir_all(&project).expect("create project");

    let substrate_dir = project.join(".substrate");
    fs::create_dir_all(&substrate_dir).expect("create .substrate");
    fs::write(
        substrate_dir.join("workspace.yaml"),
        r#"world:
  enabled: false
  anchor_mode: workspace
  anchor_path: ""
  caged: true
policy:
  mode: observe
sync:
  auto_sync: false
  direction: from_world
  conflict_policy: prefer_host
  exclude: []
"#,
    )
    .expect("write workspace.yaml marker");
    fs::write(substrate_dir.join("policy.yaml"), "not: [valid: yaml")
        .expect("write invalid policy.yaml");

    let assert = substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .current_dir(&project)
        .arg("-c")
        .arg("true")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("Failed to parse policy")
            || stderr.contains("failed to load Substrate profile"),
        "expected policy load error, got: {stderr}"
    );
    assert!(
        stderr.contains("policy.yaml") || stderr.contains("policy"),
        "expected stderr to mention policy context, got: {stderr}"
    );
}
