use super::config::{is_truthy, parse_skip_list};
use super::runtime::{detect_commands, detect_env, detect_files, detect_script, SNIPPET_HEADER};
use super::*;
use serial_test::serial;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::Path,
};
use substrate_common::Platform;
use tempfile::{tempdir, TempDir};

fn make_manifest(managers: &str) -> (ManifestPaths, TempDir) {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("manager_hooks.yaml");
    let payload = format!("version: 2\nmanagers:\n{}", managers);
    fs::write(&manifest_path, payload).unwrap();
    (
        ManifestPaths {
            base: manifest_path,
            overlay: None,
        },
        dir,
    )
}

fn base_config() -> ManagerInitConfig {
    ManagerInitConfig {
        skip_all: false,
        skip_list: HashSet::new(),
        platform: if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        },
        debug: false,
    }
}

#[test]
fn truthy_check_parses_common_values() {
    assert!(is_truthy("1"));
    assert!(is_truthy("true"));
    assert!(is_truthy("YES"));
    assert!(!is_truthy("0"));
    assert!(!is_truthy("nope"));
}

#[test]
fn skip_list_parser_handles_commas_and_spaces() {
    let parsed = parse_skip_list("nvm, pyenv  ,direnv");
    assert!(parsed.contains("nvm"));
    assert!(parsed.contains("pyenv"));
    assert!(parsed.contains("direnv"));
}

#[test]
fn detect_env_matches_expected_values() {
    let mut env_reqs = HashMap::new();
    env_reqs.insert("TEST_ENV_KEY".to_string(), "value".to_string());
    env::set_var("TEST_ENV_KEY", "value");
    assert_eq!(detect_env(&env_reqs), Some("env:TEST_ENV_KEY".to_string()));
    env::remove_var("TEST_ENV_KEY");
}

#[test]
fn detect_files_returns_first_match() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("missing");
    let present = dir.path().join("present");
    fs::write(&present, "ok").unwrap();
    let files = [missing, present.clone()];
    let reason = detect_files(&files);
    assert_eq!(reason, Some(format!("file:{}", present.display())));
}

#[test]
fn detect_commands_handles_absolute_paths() {
    let current = env::current_exe().unwrap();
    let reason = detect_commands(&[current.to_string_lossy().to_string()]);
    assert_eq!(reason, Some(format!("command:{}", current.display())));
}

#[cfg(not(windows))]
#[test]
fn detect_script_runs_shell_commands() {
    assert_eq!(
        detect_script("exit 0", Platform::Linux).unwrap(),
        Some("script".to_string())
    );
    assert_eq!(detect_script("exit 1", Platform::Linux).unwrap(), None);
}

#[cfg(windows)]
#[test]
fn detect_script_runs_powershell_commands() {
    assert_eq!(
        detect_script("exit 0", Platform::Windows).unwrap(),
        Some("script".to_string())
    );
    assert_eq!(detect_script("exit 1", Platform::Windows).unwrap(), None);
}

#[test]
#[serial]
fn config_from_env_respects_skip_and_debug_flags() {
    let prev_skip = env::var("SUBSTRATE_SKIP_MANAGER_INIT").ok();
    let prev_list = env::var("SUBSTRATE_SKIP_MANAGER_INIT_LIST").ok();
    let prev_debug = env::var("SUBSTRATE_MANAGER_INIT_DEBUG").ok();

    env::set_var("SUBSTRATE_SKIP_MANAGER_INIT", "YeS");
    env::set_var("SUBSTRATE_SKIP_MANAGER_INIT_LIST", "NVM, PyEnv");
    env::set_var("SUBSTRATE_MANAGER_INIT_DEBUG", "on");

    let cfg = ManagerInitConfig::from_env(Platform::Linux);
    assert!(cfg.skip_all);
    assert!(cfg.debug);
    assert!(cfg.skip_list.contains("nvm"));
    assert!(cfg.skip_list.contains("pyenv"));

    match prev_skip {
        Some(value) => env::set_var("SUBSTRATE_SKIP_MANAGER_INIT", value),
        None => env::remove_var("SUBSTRATE_SKIP_MANAGER_INIT"),
    }
    match prev_list {
        Some(value) => env::set_var("SUBSTRATE_SKIP_MANAGER_INIT_LIST", value),
        None => env::remove_var("SUBSTRATE_SKIP_MANAGER_INIT_LIST"),
    }
    match prev_debug {
        Some(value) => env::set_var("SUBSTRATE_MANAGER_INIT_DEBUG", value),
        None => env::remove_var("SUBSTRATE_MANAGER_INIT_DEBUG"),
    }
}

#[test]
fn detect_and_generate_skips_all_when_flag_set() {
    let (paths, _tmp) = make_manifest(
        r#"
  - name: Sample
    priority: 10
    detect:
      files: ["/tmp/nonexistent"]
    init:
      shell: |
        export SAMPLE=1
"#,
    );

    let mut cfg = base_config();
    cfg.skip_all = true;

    let result = detect_and_generate(paths, cfg).unwrap();
    assert!(result.skipped);
    assert!(result.states.is_empty());
    assert!(result.snippet.contains("SUBSTRATE_SKIP_MANAGER_INIT"));
}

#[test]
fn detect_and_generate_collects_detected_managers_and_snippet() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("nvm.sh");
    fs::write(&file_path, "echo nvm").unwrap();
    let exe = env::current_exe().unwrap();
    let manifest_body = format!(
        r#"
  - name: FileMgr
    priority: 10
    detect:
      files: ['{}']
    init:
      shell: |
        export FILE=1
  - name: CommandMgr
    priority: 20
    detect:
      commands: ['{}']
    init:
      shell: |
        export CMD=1
"#,
        yaml_path(&file_path),
        yaml_path(&exe)
    );
    let (paths, _tmp) = make_manifest(&manifest_body);

    let result = detect_and_generate(paths, base_config()).unwrap();
    assert_eq!(result.states.len(), 2);
    assert!(!result.skipped);
    let file_state = result
        .states
        .iter()
        .find(|state| state.name == "FileMgr")
        .unwrap();
    assert!(file_state.detected);
    assert!(file_state.reason.as_ref().unwrap().starts_with("file:"));
    let command_state = result
        .states
        .iter()
        .find(|state| state.name == "CommandMgr")
        .unwrap();
    assert!(command_state.detected);
    assert!(command_state
        .reason
        .as_ref()
        .unwrap()
        .starts_with("command:"));

    assert!(result.snippet.starts_with(SNIPPET_HEADER));
    let first_index = result.snippet.find("manager: FileMgr").unwrap();
    let second_index = result.snippet.find("manager: CommandMgr").unwrap();
    assert!(first_index < second_index);
}

#[test]
fn detect_and_generate_respects_skip_list_entries() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("skipme");
    fs::write(&file_path, "skip").unwrap();
    let manifest_body = format!(
        r#"
  - name: SkipMe
    priority: 5
    detect:
      files: ['{}']
    init:
      shell: |
        export SKIP=1
"#,
        yaml_path(&file_path)
    );
    let (paths, _tmp) = make_manifest(&manifest_body);

    let mut cfg = base_config();
    cfg.skip_list.insert("skipme".to_string());
    cfg.debug = true;

    let result = detect_and_generate(paths, cfg).unwrap();
    assert_eq!(result.states.len(), 1);
    let state = &result.states[0];
    assert!(!state.detected);
    assert_eq!(
        state.reason.as_deref(),
        Some("skipped via SUBSTRATE_SKIP_MANAGER_INIT_LIST")
    );
    assert!(state.snippet.is_none());
    assert!(result.snippet.contains("# No managers detected"));
}
#[test]
fn write_snippet_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("subdir").join("manager_init.sh");
    let snippet = "# test";
    write_snippet(&path, snippet).unwrap();
    assert!(path.exists());
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, snippet);
}

#[test]
fn write_snippet_replaces_existing_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manager_init.sh");
    write_snippet(&path, "first").unwrap();
    write_snippet(&path, "second").unwrap();
    let written = std::fs::read_to_string(path).unwrap();
    assert_eq!(written, "second");
}

#[test]
fn telemetry_payload_marks_snippet_presence() {
    let states = vec![
        ManagerState {
            name: "nvm".into(),
            detected: true,
            reason: Some("file:/nvm".into()),
            snippet: Some("export NVM=1".into()),
        },
        ManagerState {
            name: "pyenv".into(),
            detected: false,
            reason: None,
            snippet: Some("".into()),
        },
        ManagerState {
            name: "direnv".into(),
            detected: false,
            reason: None,
            snippet: None,
        },
    ];
    let payload = telemetry_payload(&states);
    let array = payload.as_array().unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array[0]["name"], "nvm");
    assert_eq!(array[0]["has_snippet"], true);
    assert_eq!(array[1]["has_snippet"], false);
    assert_eq!(array[2]["has_snippet"], false);
}
fn yaml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
