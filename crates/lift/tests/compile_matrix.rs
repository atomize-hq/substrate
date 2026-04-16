use assert_cmd as _;
use clap as _;
use jsonschema as _;
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror as _;
use toml as _;

#[cfg(feature = "cli")]
#[test]
fn run_cli_is_exposed_when_cli_feature_is_enabled() {
    let _entrypoint: fn() -> Result<(), substrate_lift::error::LiftError> = substrate_lift::run_cli;
}

#[test]
fn crate_compiles_without_default_features() {
    let workspace_root = workspace_root();
    let target_dir = workspace_root.join("target/lift-seam0-no-default-features");
    let output = Command::new(env!("CARGO"))
        .current_dir(&workspace_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["check", "-p", "substrate-lift", "--no-default-features"])
        .output()
        .expect("cargo check should run");

    assert!(
        output.status.success(),
        "cargo check -p substrate-lift --no-default-features failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn binary_is_gated_behind_cli_feature_in_manifest() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("crate Cargo.toml should be readable");

    assert!(
        manifest.contains("[[bin]]"),
        "expected a binary target declaration in {}",
        manifest_path.display(),
    );
    assert!(
        manifest.contains("name = \"lift\""),
        "expected lift binary declaration in {}",
        manifest_path.display(),
    );
    assert!(
        manifest.contains("required-features = [\"cli\"]"),
        "expected lift binary to be gated by required-features = [\"cli\"] in {}",
        manifest_path.display(),
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate should live under crates/<name>")
        .to_path_buf()
}
