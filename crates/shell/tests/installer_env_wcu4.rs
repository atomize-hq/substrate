#![cfg(unix)]

mod common;

use common::{binary_path, ensure_substrate_built, shared_tmpdir, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn read_repo_file(relative_path: &str) -> String {
    fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|err| panic!("read {relative_path}: {err}"))
}

#[test]
fn installer_scripts_do_not_export_substrate_override_by_default() {
    for script in [
        "scripts/substrate/install-substrate.sh",
        "scripts/substrate/dev-install-substrate.sh",
    ] {
        let contents = read_repo_file(script);
        assert!(
            !contents.contains("export SUBSTRATE_OVERRIDE_"),
            "{script} must not export SUBSTRATE_OVERRIDE_* by default (WCU4)"
        );
    }
}

#[test]
fn dev_install_scripts_explicitly_select_gateway_package() {
    let unix_installer = read_repo_file("scripts/substrate/dev-install-substrate.sh");
    assert!(
        unix_installer.contains(
            "BUILD_FLAGS=(build -p substrate --bin substrate --bin substrate-shim -p substrate-gateway --bin substrate-gateway)"
        ),
        "unix dev installer must explicitly select substrate and substrate-gateway packages"
    );

    let windows_installer = read_repo_file("scripts/windows/dev-install-substrate.ps1");
    assert!(
        windows_installer.contains(
            "$buildArgs = @('build', '-p', 'substrate', '--bin', 'substrate', '--bin', 'substrate-shim', '-p', 'substrate-gateway', '--bin', 'substrate-gateway')"
        ),
        "windows dev installer must explicitly select substrate and substrate-gateway packages"
    );
}

#[test]
fn macos_dev_install_does_not_force_skip_guest_build_during_lima_warm() {
    let unix_installer = read_repo_file("scripts/substrate/dev-install-substrate.sh");
    assert!(
        unix_installer.contains("scripts/mac/lima-warm.sh"),
        "unix dev installer should still invoke the macOS Lima warm helper"
    );
    assert!(
        !unix_installer.contains(
            "(cd \"${REPO_ROOT}\" && SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 \"${LIMA_WARM}\" \"${REPO_ROOT}\")"
        ) && !unix_installer.contains(
            "(cd \"${REPO_ROOT}\" && SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 SUBSTRATE_WORLD_NETFILTER_ENABLE=1 \"${LIMA_WARM}\" \"${REPO_ROOT}\")"
        ),
        "macOS dev-install must not unconditionally force SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 for lima-warm"
    );
}

#[test]
fn config_current_show_is_not_affected_without_override_inputs() {
    ensure_substrate_built();
    let temp = temp_dir("substrate-wcu4-clean-env-");
    let home = temp.path().join("home");
    let substrate_home = temp.path().join("substrate-home");
    let cwd = temp.path().join("cwd");
    fs::create_dir_all(&home).expect("create HOME");
    fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME");
    fs::create_dir_all(&cwd).expect("create cwd");

    fs::write(
        substrate_home.join("config.yaml"),
        "policy:\n  mode: disabled\n",
    )
    .expect("write config.yaml");

    let host_path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
    let output = Command::new(binary_path())
        .env_clear()
        .env("PATH", host_path)
        .env("TMPDIR", shared_tmpdir())
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .current_dir(&cwd)
        .arg("--no-world")
        .args(["config", "current", "show", "--json"])
        .output()
        .expect("run substrate config current show --json");
    assert!(
        output.status.success(),
        "config current show should succeed: {output:?}"
    );

    let json: JsonValue = serde_json::from_slice(&output.stdout).expect("current JSON parse");
    assert_eq!(
        json.pointer("/policy/mode").and_then(|v| v.as_str()),
        Some("disabled"),
        "policy.mode mismatch: {json}"
    );
}
