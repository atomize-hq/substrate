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

    fs::write(project.join(".substrate-profile"), "not: [valid: yaml")
        .expect("write invalid profile");

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
        stderr.contains("failed to load Substrate profile"),
        "expected profile load error, got: {stderr}"
    );
    assert!(
        stderr.contains(".substrate-profile") || stderr.contains("profile"),
        "expected stderr to mention profile context, got: {stderr}"
    );
}
