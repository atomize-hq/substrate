#![cfg(unix)]

mod support;

use std::fs;
use support::{
    dedupe_path, path_str, payload_lines, substrate_command_for_home, ShellEnvFixture,
    PAYLOAD_MARKER,
};

#[test]
fn shell_env_injects_manager_snippets() {
    let fixture = ShellEnvFixture::new();
    let manifest = fixture.write_manifest(
        r#"version: 1
managers:
  - name: DemoManager
    detect:
      script: "exit 0"
    init:
      shell: |
        export MANAGER_MARKER="manager_init_loaded"
  - name: Volta
    detect:
      script: "exit 0"
    init:
      shell: |
        export VOLTA_MARKER="volta_loaded"
"#,
    );
    let host_bash_env = fixture.home().join("host_bash_env.sh");
    fs::write(&host_bash_env, "export HOST_BE_VALUE=\"host_env\"\n").unwrap();
    let legacy_bashenv = fixture.home().join(".substrate_bashenv");
    fs::write(&legacy_bashenv, "export LEGACY_MARKER=\"legacy_env\"\n").unwrap();
    let parent_path_before = String::new();
    let host_path = fixture.home().join("host-bin");
    fs::create_dir_all(&host_path).unwrap();
    let host_segment = path_str(&host_path);
    let host_path_str = host_segment.clone();

    let script = format!(
        "printf '%s\\n' \"{marker}\" \"$PATH\" \"$MANAGER_MARKER\" \"$LEGACY_MARKER\" \
         \"$HOST_BE_VALUE\" \"${{BASH_ENV:-}}\" \"${{SUBSTRATE_MANAGER_ENV:-}}\" \
         \"${{SUBSTRATE_MANAGER_INIT:-}}\" \"${{SUBSTRATE_ORIGINAL_BASH_ENV:-}}\"",
        marker = PAYLOAD_MARKER
    );
    let output = substrate_command_for_home(&fixture)
        .env("PATH", &host_path_str)
        .env("BASH_ENV", &host_bash_env)
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env("SUBSTRATE_SHIM_PATH", path_str(&fixture.shim_dir()))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_MANAGER_INIT", path_str(&fixture.manager_init_path()))
        .env("SUBSTRATE_MANAGER_ENV", path_str(&fixture.manager_env_path()))
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_MANAGER_MANIFEST", path_str(&manifest))
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate -c for shell env test");

    assert!(
        output.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        8,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let path_line = &lines[0];
    let shim_prefix = format!("{}:", fixture.shim_dir().display());
    assert!(
        path_line.starts_with(&shim_prefix),
        "PATH did not start with shims: {}",
        path_line
    );
    let remainder = &path_line[shim_prefix.len()..];
    assert_eq!(remainder, dedupe_path(&host_path_str));
    assert_eq!(lines[1], "manager_init_loaded");
    assert_eq!(lines[2], "legacy_env");
    assert_eq!(lines[3], "host_env");
    assert_eq!(lines[4], fixture.preexec_path().display().to_string());
    assert_eq!(lines[5], fixture.manager_env_path().display().to_string());
    assert_eq!(lines[6], fixture.manager_init_path().display().to_string());
    assert_eq!(lines[7], host_bash_env.display().to_string());
    let manager_env_contents =
        fs::read_to_string(fixture.manager_env_path()).expect("manager env contents");
    assert!(
        manager_env_contents.contains("SUBSTRATE_MANAGER_INIT"),
        "manager_env missing manager init sourcing"
    );
    assert!(
        manager_env_contents.contains("SUBSTRATE_ORIGINAL_BASH_ENV"),
        "manager_env missing original BASH_ENV sourcing"
    );
    assert!(
        manager_env_contents.contains(".substrate_bashenv"),
        "manager_env missing legacy bashenv sourcing"
    );
    let manager_init_contents =
        fs::read_to_string(fixture.manager_init_path()).expect("manager init contents");
    assert!(
        manager_init_contents.contains("VOLTA_MARKER"),
        "manager init snippet missing Tier-2 manager content"
    );
}

#[test]
fn shell_env_no_world_skips_manager_env() {
    let fixture = ShellEnvFixture::new();
    let host_path = fixture.home().join("host-only");
    fs::create_dir_all(&host_path).unwrap();
    let host_path_str = path_str(&host_path);
    let host_bash_env = fixture.home().join("host_env.sh");
    fs::write(&host_bash_env, "export HOST_ONLY=1\n").unwrap();

    let script = format!(
        "printf '%s\\n' \"{marker}\" \"$PATH\" \"${{BASH_ENV:-}}\" \
         \"${{SUBSTRATE_MANAGER_ENV:-none}}\" \"${{SUBSTRATE_MANAGER_INIT:-none}}\" \
         \"${{SUBSTRATE_ORIGINAL_BASH_ENV:-none}}\"",
        marker = PAYLOAD_MARKER
    );
    let output = substrate_command_for_home(&fixture)
        .env("PATH", &host_path_str)
        .env("BASH_ENV", &host_bash_env)
        .arg("--no-world")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate --no-world");

    assert!(
        output.status.success(),
        "substrate --no-world failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        5,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(lines[0], host_path_str);
    assert_eq!(lines[1], host_bash_env.display().to_string());
    assert_eq!(lines[2], "none");
    assert_eq!(lines[3], "none");
    assert_eq!(lines[4], "none");
}

#[test]
fn shell_env_applies_overlay_manifest() {
    let fixture = ShellEnvFixture::new();
    let missing_path = fixture.home().join("missing-tool");
    let script = format!(
        "source \"{manager_env}\"; printf '%s\\n' \"{marker}\" \"$OVERLAY_VALUE\"",
        marker = PAYLOAD_MARKER,
        manager_env = fixture.manager_env_path().display()
    );
    let output = substrate_command_for_home(&fixture)
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env("SUBSTRATE_SHIM_PATH", path_str(&fixture.shim_dir()))
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("OVERLAY_VALUE", "overlay-active")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate for overlay manifest");
    assert!(
        output.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        1,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(lines[0], "overlay-active");
}
