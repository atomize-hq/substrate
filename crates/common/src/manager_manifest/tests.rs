use super::*;
use std::{env, fs, path::Path};
use tempfile::tempdir;

#[test]
fn loads_manifest_with_overlay_and_sorting() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    let overlay_path = dir.path().join("overlay.yaml");

    let base = r#"
version: 1
managers:
  - name: nvm
    priority: 20
    detect:
      files:
        - "$SUBSTRATE_TEST_HOME/.nvm/nvm.sh"
    init:
      shell: "echo source nvm"
    errors:
      - "nvm: .*"
  - name: pyenv
    priority: 5
    detect:
      commands: ["pyenv --version"]
    init:
      shell: "echo pyenv"
    errors: []
"#;

    let overlay = r#"
version: 1
managers:
  nvm:
    priority: 1
  asdf:
    detect:
      files: ["~/custom/.asdf"]
    init:
      shell: "echo asdf"
    errors:
      - "asdf: .*"
"#;

    fs::write(&base_path, base).unwrap();
    fs::write(&overlay_path, overlay).unwrap();
    env::set_var(
        "SUBSTRATE_TEST_HOME",
        dir.path().to_string_lossy().to_string(),
    );

    let manifest = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap();
    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.managers.len(), 3);
    assert_eq!(manifest.managers[0].name, "nvm");
    assert_eq!(manifest.managers[0].priority, 1);
    assert_eq!(manifest.managers[1].name, "pyenv");
    assert_eq!(manifest.managers[2].name, "asdf");

    assert!(manifest.managers[0].detect.files[0]
        .display()
        .to_string()
        .contains(dir.path().to_str().unwrap()));

    env::remove_var("SUBSTRATE_TEST_HOME");
}

#[test]
fn expands_env_and_tilde_in_detect_fields() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  envy:
    detect:
      files:
        - "~/.substrate/envy.sh"
        - "$MANAGER_TEST_HOME/scripts/envy.sh"
      env:
        ENVY_ROOT: "~/.substrate/envy"
        MANAGER_HOME: "$MANAGER_TEST_HOME/manager"
    init: {}
    errors: []
"#,
    )
    .unwrap();

    env::set_var("MANAGER_TEST_HOME", dir.path());
    let manifest = ManagerManifest::load(&base_path, None).unwrap();
    env::remove_var("MANAGER_TEST_HOME");

    let detect = &manifest.managers[0].detect;
    let home = dirs::home_dir().expect("home directory must be set for tests");
    assert_eq!(detect.files[0], home.join(".substrate/envy.sh"));
    assert_eq!(detect.files[1], dir.path().join("scripts/envy.sh"));
    let expected_env = format!("{}/manager", dir.path().display());
    assert_eq!(
        detect.env.get("MANAGER_HOME").map(String::as_str),
        Some(expected_env.as_str())
    );
    assert!(detect.env.contains_key("ENVY_ROOT"));
    assert!(detect
        .env
        .get("ENVY_ROOT")
        .unwrap()
        .contains(".substrate/envy"));
}

#[test]
fn overlay_merges_detect_env_and_install_spec() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    let overlay_path = dir.path().join("overlay.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  direnv:
    detect:
      files: ["~/.direnv"]
      commands: ["direnv version"]
      env:
        DIRENV_DIR: "/base/path"
    init: {}
    errors: []
    guest_detect:
      command: "direnv version"
    guest_install:
      apt: "sudo apt install direnv"
"#,
    )
    .unwrap();
    fs::write(
        &overlay_path,
        r#"
version: 1
managers:
  direnv:
    detect:
      commands: ["direnv --help"]
      env:
        DIRENV_HOME: "/overlay/home"
    guest_install:
      custom: "run-me.sh"
"#,
    )
    .unwrap();

    let manifest = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap();
    let direnv = manifest
        .managers
        .iter()
        .find(|spec| spec.name == "direnv")
        .unwrap();

    assert_eq!(direnv.detect.commands, vec!["direnv --help"]);
    let home = dirs::home_dir().expect("home directory must be set for tests");
    assert_eq!(direnv.detect.files[0], home.join(".direnv"));
    assert_eq!(
        direnv.detect.env.get("DIRENV_DIR").map(String::as_str),
        Some("/base/path")
    );
    assert_eq!(
        direnv.detect.env.get("DIRENV_HOME").map(String::as_str),
        Some("/overlay/home")
    );
    let install = direnv.guest.install.as_ref().expect("install spec");
    assert_eq!(install.apt.as_deref(), Some("sudo apt install direnv"));
    assert_eq!(install.custom.as_deref(), Some("run-me.sh"));
}

#[test]
fn overlay_version_mismatch_returns_error() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    let overlay_path = dir.path().join("overlay.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  base:
    detect: {}
    init: {}
    errors: []
"#,
    )
    .unwrap();
    fs::write(
        &overlay_path,
        r#"
version: 2
managers:
  base:
    detect: {}
    init: {}
    errors: []
"#,
    )
    .unwrap();

    let err = ManagerManifest::load(&base_path, Some(&overlay_path)).unwrap_err();
    assert!(err
        .to_string()
        .contains("overlay manifest version 2 does not match base 1"));
}

#[test]
fn missing_overlay_is_ignored() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  - name: only
    detect:
      files: ["~/file"]
    init:
      shell: "echo"
    errors: []
"#,
    )
    .unwrap();

    let missing = dir.path().join("missing.yaml");
    let manifest = ManagerManifest::load(&base_path, Some(&missing)).unwrap();
    assert_eq!(manifest.managers.len(), 1);
}

#[test]
fn duplicate_names_return_error() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  - name: dup
    detect: {}
    init: {}
    errors: []
  - name: dup
    detect: {}
    init: {}
    errors: []
"#,
    )
    .unwrap();

    let err = ManagerManifest::load(&base_path, None).unwrap_err();
    assert!(err.to_string().contains("duplicate manager"));
}

#[test]
fn invalid_regex_is_reported() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  broken:
    detect: {}
    init: {}
    errors:
      - "[unclosed"
"#,
    )
    .unwrap();

    let err = ManagerManifest::load(&base_path, None).unwrap_err();
    assert!(err.to_string().contains("invalid regex"));
}

#[test]
fn resolve_for_platform_trims_init_snippets() {
    let dir = tempdir().unwrap();
    let base_path = dir.path().join("base.yaml");
    fs::write(
        &base_path,
        r#"
version: 1
managers:
  shellish:
    detect: {}
    init:
      shell: "echo shell"
      powershell: "Write-Host shell"
    errors: []
"#,
    )
    .unwrap();

    let manifest = ManagerManifest::load(&base_path, None).unwrap();
    let windows = manifest.resolve_for_platform(Platform::Windows);
    assert!(windows[0].init.shell.is_none());
    assert!(windows[0].init.powershell.is_some());

    let linux = manifest.resolve_for_platform(Platform::Linux);
    assert!(linux[0].init.shell.is_some());
    assert!(linux[0].init.powershell.is_none());
}

#[test]
fn tier2_managers_include_complete_metadata() {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("tier2.yaml");
    let tier2_root = dir.path().join("tier2-home");
    fs::create_dir_all(&tier2_root).unwrap();
    env::set_var("TIER2_HOME", &tier2_root);

    let tier2_str = tier2_root.to_string_lossy().replace('\\', "\\\\");
    let manifest_body = format!(
        r#"
version: 1
managers:
  - name: mise
    priority: 9
    detect:
      commands: ["mise --version"]
      env:
        MISE_DATA_DIR: "$TIER2_HOME/state/mise"
      script: |
        test -x "$MISE_DATA_DIR/bin/mise"
    init:
      shell: |
        export MISE_DATA_DIR="${{MISE_DATA_DIR:-$HOME/.local/share/mise}}"
        eval "$(mise activate bash)"
    errors:
      - "mise: command not found"
    repair_hint: |
      eval "$(mise activate bash)"
    guest_detect:
      command: "mise --version"
    guest_install:
      apt: "sudo apt install mise"
  - name: rtx
    priority: 12
    detect:
      files:
        - "~/.local/share/rtx/bin/rtx"
      commands: ["rtx --version"]
    init:
      shell: |
        eval "$(rtx activate bash)"
    errors:
      - "rtx: command not found"
    repair_hint: |
      eval "$(rtx activate bash)"
    guest_detect:
      command: "rtx --version"
    guest_install:
      custom: "curl https://rtx.pub/install.sh | sh"
  - name: rbenv
    priority: 14
    detect:
      files:
        - '{tier2}/.rbenv/bin/rbenv'
      env:
        RBENV_ROOT: '{tier2}/.rbenv'
    init:
      shell: |
        export RBENV_ROOT="${{RBENV_ROOT:-$HOME/.rbenv}}"
        eval "$(rbenv init - bash)"
    errors:
      - "rbenv: command not found"
    repair_hint: |
      eval "$(rbenv init - bash)"
    guest_detect:
      command: "rbenv --version"
    guest_install:
      apt: "sudo apt install rbenv"
  - name: sdkman
    priority: 18
    detect:
      files:
        - "~/.sdkman/bin/sdkman-init.sh"
      env:
        SDKMAN_DIR: "~/.sdkman"
      script: |
        test -d "$SDKMAN_DIR/candidates"
    init:
      shell: |
        export SDKMAN_DIR="${{SDKMAN_DIR:-$HOME/.sdkman}}"
        source "$SDKMAN_DIR/bin/sdkman-init.sh"
    errors:
      - "sdk: command not found"
    repair_hint: |
      source "$SDKMAN_DIR/bin/sdkman-init.sh"
    guest_detect:
      command: "sdk version"
    guest_install:
      custom: "curl -s https://get.sdkman.io | bash"
  - name: bun
    priority: 30
    detect:
      commands: ["bun --version"]
      files:
        - "~/.bun/bin/bun"
    init:
      shell: |
        export BUN_INSTALL="${{BUN_INSTALL:-$HOME/.bun}}"
        export PATH="$BUN_INSTALL/bin:$PATH"
    errors:
      - "bun: command not found"
    repair_hint: |
      curl https://bun.sh/install | bash
    guest_detect:
      command: "bun --version"
    guest_install:
      custom: "curl https://bun.sh/install | bash"
  - name: volta
    priority: 22
    detect:
      env:
        VOLTA_HOME: "$TIER2_HOME/.volta"
      script: |
        test -x "$VOLTA_HOME/bin/volta"
    init:
      shell: |
        export VOLTA_HOME="${{VOLTA_HOME:-$HOME/.volta}}"
        export PATH="$VOLTA_HOME/bin:$PATH"
    errors:
      - "volta: command not found"
    repair_hint: |
      export VOLTA_HOME="${{VOLTA_HOME:-$HOME/.volta}}"
    guest_detect:
      command: "volta --version"
    guest_install:
      apt: "sudo apt install volta"
  - name: goenv
    priority: 35
    detect:
      files:
        - '{tier2}/.goenv/bin/goenv'
      env:
        GOENV_ROOT: '{tier2}/.goenv'
    init:
      shell: |
        export GOENV_ROOT="${{GOENV_ROOT:-$HOME/.goenv}}"
        eval "$(goenv init -)"
    errors:
      - "goenv: command not found"
    repair_hint: |
      eval "$(goenv init -)"
    guest_detect:
      command: "goenv --version"
    guest_install:
      apt: "sudo apt install goenv"
      custom: "brew install goenv"
  - name: asdf-node
    priority: 40
    detect:
      commands:
        - "asdf current nodejs"
    init:
      shell: |
        asdf exec node --version >/dev/null 2>&1
    errors:
      - "asdf: .*"
    repair_hint: |
      asdf plugin add nodejs
    guest_detect:
      command: "asdf current nodejs"
    guest_install:
      custom: "asdf plugin add nodejs && asdf install nodejs latest"
"#,
        tier2 = tier2_str
    );
    fs::write(&manifest_path, manifest_body).unwrap();

    let manifest = ManagerManifest::load(&manifest_path, None).unwrap();
    env::remove_var("TIER2_HOME");

    let expected = [
        "mise",
        "rtx",
        "rbenv",
        "sdkman",
        "bun",
        "volta",
        "goenv",
        "asdf-node",
    ];
    for name in expected {
        assert!(
            manifest.managers.iter().any(|spec| spec.name == name),
            "expected {name} entry in manifest"
        );
    }

    let find = |name: &str| -> &ManagerSpec {
        manifest
            .managers
            .iter()
            .find(|spec| spec.name == name)
            .unwrap_or_else(|| panic!("missing {name} spec"))
    };

    let tier2_mise_dir = tier2_root.join("state").join("mise");
    let mise = find("mise");
    let mise_data = mise
        .detect
        .env
        .get("MISE_DATA_DIR")
        .expect("MISE_DATA_DIR env");
    assert_eq!(Path::new(mise_data), tier2_mise_dir);
    assert!(
        mise.repair_hint
            .as_deref()
            .map(|hint| hint.contains("mise activate bash"))
            .unwrap_or(false),
        "expected mise repair hint to mention activation",
    );
    assert!(mise
        .detect
        .script
        .as_deref()
        .unwrap()
        .contains("$MISE_DATA_DIR/bin/mise"));
    assert_eq!(mise.guest.detect_cmd.as_deref(), Some("mise --version"));
    let mise_install = mise.guest.install.as_ref().expect("mise install spec");
    assert_eq!(mise_install.apt.as_deref(), Some("sudo apt install mise"));

    let home = dirs::home_dir().expect("home directory");
    let rtx = find("rtx");
    assert_eq!(rtx.detect.files[0], home.join(".local/share/rtx/bin/rtx"));
    let rtx_install = rtx.guest.install.as_ref().expect("rtx install spec");
    assert_eq!(
        rtx_install.custom.as_deref(),
        Some("curl https://rtx.pub/install.sh | sh")
    );

    let rbenv = find("rbenv");
    assert_eq!(
        rbenv.detect.files[0],
        tier2_root.join(".rbenv").join("bin").join("rbenv")
    );
    let expected_rbenv_root = tier2_root.join(".rbenv");
    assert_eq!(
        rbenv.detect.env.get("RBENV_ROOT").map(Path::new),
        Some(expected_rbenv_root.as_path())
    );
    assert_eq!(rbenv.guest.detect_cmd.as_deref(), Some("rbenv --version"));

    let sdkman = find("sdkman");
    let sdkman_dir = sdkman.detect.env.get("SDKMAN_DIR").expect("SDKMAN_DIR env");
    assert_eq!(Path::new(sdkman_dir), home.join(".sdkman"));
    assert!(sdkman
        .detect
        .script
        .as_deref()
        .unwrap()
        .contains("candidates"));

    let bun = find("bun");
    assert_eq!(bun.detect.files[0], home.join(".bun/bin/bun"));
    assert_eq!(bun.errors[0].pattern, "bun: command not found".to_string());

    let volta = find("volta");
    let volta_home = volta.detect.env.get("VOLTA_HOME").expect("VOLTA_HOME env");
    assert_eq!(Path::new(volta_home), tier2_root.join(".volta"));

    let goenv = find("goenv");
    let expected_goenv_root = tier2_root.join(".goenv");
    assert_eq!(
        goenv.detect.env.get("GOENV_ROOT").map(Path::new),
        Some(expected_goenv_root.as_path())
    );
    let install = goenv.guest.install.as_ref().expect("goenv install");
    assert_eq!(install.apt.as_deref(), Some("sudo apt install goenv"));
    assert_eq!(install.custom.as_deref(), Some("brew install goenv"));

    let asdf_node = find("asdf-node");
    assert_eq!(
        asdf_node.guest.detect_cmd.as_deref(),
        Some("asdf current nodejs")
    );
    assert!(asdf_node.repair_hint.as_ref().unwrap().contains("plugin"));
}
