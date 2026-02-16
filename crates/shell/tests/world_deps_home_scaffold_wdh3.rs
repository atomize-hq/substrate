#![cfg(unix)]

mod support;

use support::{substrate_shell_driver, temp_dir};

use std::fs;
use std::path::Path;

fn assert_is_dir(path: &Path) {
    assert!(
        path.is_dir(),
        "expected directory at {}, but it was missing or not a directory",
        path.display()
    );
}

fn assert_is_file(path: &Path) {
    assert!(
        path.is_file(),
        "expected file at {}, but it was missing or not a file",
        path.display()
    );
}

#[test]
fn test_bootstrap_scaffolds_deps_on_version() {
    let tmp = temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");

    substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .current_dir(&home)
        .arg("--version")
        .assert()
        .success();

    let deps_root = substrate_home.join("deps");
    assert_is_dir(&deps_root);
    assert_is_dir(&deps_root.join("packages"));
    assert_is_dir(&deps_root.join("bundles"));
    assert_is_dir(&deps_root.join("scripts"));

    assert_is_file(&deps_root.join("README.md"));
    assert_is_file(&deps_root.join("packages/example-manual.yaml"));
    assert_is_file(&deps_root.join("packages/example-script.yaml"));
    assert_is_file(&deps_root.join("packages/example-apt.yaml"));
    assert_is_file(&deps_root.join("bundles/example-bundle.yaml"));
    assert_is_file(&deps_root.join("scripts/example-install.sh"));
}

#[test]
fn test_bootstrap_is_idempotent_and_does_not_overwrite() {
    let tmp = temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");

    substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .current_dir(&home)
        .arg("--version")
        .assert()
        .success();

    let readme = substrate_home.join("deps/README.md");
    assert_is_file(&readme);

    let custom = "user-modified README\n";
    fs::write(&readme, custom).expect("overwrite README in fixture");

    substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .current_dir(&home)
        .arg("--version")
        .assert()
        .success();

    let after = fs::read_to_string(&readme).expect("read README after");
    assert_eq!(after, custom);
}

#[test]
fn test_bootstrap_wrong_type_fails_with_exit_1() {
    let tmp = temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");
    let deps_root = substrate_home.join("deps");
    fs::create_dir_all(&deps_root).expect("create deps root");
    fs::write(deps_root.join("packages"), "not a directory\n").expect("seed wrong type");

    substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .current_dir(&home)
        .arg("--version")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("deps/packages"))
        .stderr(predicates::str::contains("expected directory"));
}
