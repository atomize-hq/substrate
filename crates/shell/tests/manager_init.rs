#![cfg(unix)]

use std::collections::HashSet;
use std::fs;

use substrate_common::manager_manifest::Platform;
use substrate_shell::manager_init::{self, ManagerInitConfig, ManifestPaths};
use tempfile::tempdir;

#[test]
fn tier2_managers_generate_snippets() {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("manager_hooks.yaml");
    fs::write(
        &manifest_path,
        r#"version: 1
managers:
  - name: mise
    priority: 9
    detect:
      script: "exit 0"
    init:
      shell: |
        export MISE_MARKER="mise-loaded"
  - name: rtx
    priority: 10
    detect:
      script: "exit 0"
    init:
      shell: |
        export RTX_MARKER="rtx-loaded"
  - name: rbenv
    priority: 12
    detect:
      script: "exit 0"
    init:
      shell: |
        export RBENV_MARKER="rbenv-loaded"
  - name: sdkman
    priority: 14
    detect:
      script: "exit 0"
    init:
      shell: |
        export SDKMAN_MARKER="sdkman-loaded"
  - name: bun
    priority: 16
    detect:
      script: "exit 0"
    init:
      shell: |
        export BUN_MARKER="bun-loaded"
  - name: volta
    priority: 18
    detect:
      script: "exit 0"
    init:
      shell: |
        export VOLTA_MARKER="volta-loaded"
  - name: goenv
    priority: 20
    detect:
      script: "exit 0"
    init:
      shell: |
        export GOENV_MARKER="goenv-loaded"
"#,
    )
    .unwrap();

    let config = ManagerInitConfig {
        skip_all: false,
        skip_list: HashSet::new(),
        platform: Platform::Linux,
        debug: true,
    };
    let result = manager_init::detect_and_generate(
        ManifestPaths {
            base: manifest_path,
            overlay: None,
        },
        config,
    )
    .expect("manager init should succeed");

    let names: HashSet<_> = result
        .states
        .iter()
        .map(|state| state.name.as_str())
        .collect();
    for expected in ["mise", "rtx", "rbenv", "sdkman", "bun", "volta", "goenv"] {
        assert!(
            names.contains(expected),
            "missing state for manager {expected}"
        );
    }

    for state in &result.states {
        assert!(
            state.detected,
            "manager {} should be detected via script",
            state.name
        );
        assert!(
            state
                .reason
                .as_deref()
                .unwrap_or_default()
                .starts_with("script"),
            "expected script detection reason for {}",
            state.name
        );
        assert!(
            state
                .snippet
                .as_deref()
                .unwrap()
                .contains(&format!("{}_MARKER", state.name.to_ascii_uppercase())),
            "snippet for {} missing unique marker",
            state.name
        );
    }

    assert!(result.snippet.contains("MISE_MARKER"));
    assert!(result.snippet.contains("GOENV_MARKER"));
    assert!(
        result.snippet.contains("# manager: volta"),
        "snippet should include manager headers"
    );
}
