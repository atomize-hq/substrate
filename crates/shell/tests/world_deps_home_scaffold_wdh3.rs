#![cfg(unix)]

mod support;

use support::substrate_shell_driver;

use assert_cmd::prelude::*;
use std::fs;
#[cfg(target_os = "linux")]
use std::io::Write as _;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::{Output, Stdio};
use tempfile::{Builder, TempDir};
use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

fn private_temp_dir(prefix: &str) -> TempDir {
    let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(format!("/run/user/{}", unsafe { libc::geteuid() }))
        });
    Builder::new()
        .prefix(prefix)
        .tempdir_in(safe_parent)
        .expect("allocate private-home test root")
}

fn install_bootstrap_context(substrate_home: &Path) -> (String, String, String, String) {
    let uid = unsafe { libc::geteuid() };
    let account_output = Command::new("id")
        .args(["-nu", &uid.to_string()])
        .output()
        .expect("resolve current Unix account");
    assert!(account_output.status.success());
    let account = String::from_utf8(account_output.stdout)
        .expect("account is UTF-8")
        .trim()
        .to_string();
    let context = InstallBootstrapContextV1::new_unix(
        substrate_home.to_str().expect("selected home is UTF-8"),
        &account,
        uid,
    )
    .expect("construct install bootstrap context");
    let carrier = InstallBootstrapContextCarrierV1::from_context(context)
        .expect("commit install bootstrap context");
    (
        carrier.encode().expect("encode install bootstrap context"),
        carrier.host_context_commitment,
        account,
        uid.to_string(),
    )
}

fn install_bootstrap_driver(home: &Path, substrate_home: &Path) -> assert_cmd::Command {
    let (carrier, commitment, account, uid) = install_bootstrap_context(substrate_home);
    let mut command = substrate_shell_driver();
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("SUBSTRATE_HOME", substrate_home)
        .env("SUBSTRATE_ROOT", substrate_home)
        .env("SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT", commitment)
        .env("SUBSTRATE_INSTALL_PRIMARY_USER", account)
        .env("SUBSTRATE_INSTALL_PRIMARY_UID", uid)
        .env("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", &carrier)
        .current_dir(home)
        .arg("--install-bootstrap-context-v1")
        .arg(carrier)
        .arg("--install-bootstrap-home-v1");
    command
}

fn assert_install_bootstrap_with_umask(home: &Path, substrate_home: &Path, umask: &str) {
    support::ensure_substrate_built();
    let binary = support::binary_path();
    let (carrier, commitment, account, uid) = install_bootstrap_context(substrate_home);
    Command::new("bash")
        .args([
            "-c",
            "umask \"$1\"; shift; exec \"$@\"",
            "substrate-private-home-test",
            umask,
        ])
        .arg(binary)
        .arg("--install-bootstrap-context-v1")
        .arg(&carrier)
        .arg("--install-bootstrap-home-v1")
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("SUBSTRATE_HOME", substrate_home)
        .env("SUBSTRATE_ROOT", substrate_home)
        .env("SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT", commitment)
        .env("SUBSTRATE_INSTALL_PRIMARY_USER", account)
        .env("SUBSTRATE_INSTALL_PRIMARY_UID", uid)
        .env("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", carrier)
        .current_dir(home)
        .assert()
        .success();
}

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

#[cfg(target_os = "linux")]
fn lima_private_home_fallback() -> String {
    let script = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/mac/lima-warm.sh"),
    )
    .expect("read Lima provisioner");
    let marker = "python3 - \"${SUBSTRATE_GUEST_HOME}\" <<'PY'\n";
    let (_, after_marker) = script
        .split_once(marker)
        .expect("find embedded Lima private-home provisioner");
    let (program, _) = after_marker
        .split_once("\nPY\nEOF")
        .expect("find end of embedded Lima private-home provisioner");
    program.to_string()
}

#[cfg(target_os = "linux")]
fn run_lima_private_home_fallback(program: &str, target: &Path) -> Output {
    let mut child = Command::new("python3")
        .arg("-")
        .arg(target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start embedded Lima private-home provisioner");
    child
        .stdin
        .take()
        .expect("open provisioner stdin")
        .write_all(program.as_bytes())
        .expect("write provisioner program");
    child.wait_with_output().expect("wait for provisioner")
}

#[test]
fn test_install_bootstrap_action_scaffolds_deps() {
    let tmp = private_temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");

    install_bootstrap_driver(&home, &substrate_home)
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
fn test_version_is_non_mutating() {
    let tmp = private_temp_dir("substrate-wdh3-version-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
    let substrate_home = home.join(".substrate");

    substrate_shell_driver()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("SUBSTRATE_HOME", &substrate_home)
        .env("SUBSTRATE_ROOT", &substrate_home)
        .current_dir(&home)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("substrate"));

    assert!(!substrate_home.exists());
    for relative in [
        "deps",
        "manager_env.sh",
        "manager_init.sh",
        "manager_hooks.yaml",
        "shims",
        "trace.jsonl",
        "env.sh",
        "dev-shim-env.sh",
        "config.yaml",
        "version.json",
    ] {
        assert!(
            !substrate_home.join(relative).exists(),
            "version created {relative}"
        );
    }
}

#[test]
fn test_bootstrap_is_idempotent_and_does_not_overwrite() {
    let tmp = private_temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .success();

    let readme = substrate_home.join("deps/README.md");
    assert_is_file(&readme);

    let custom = "user-modified README\n";
    fs::write(&readme, custom).expect("overwrite README in fixture");

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .success();

    let after = fs::read_to_string(&readme).expect("read README after");
    assert_eq!(after, custom);
}

#[test]
fn test_bootstrap_wrong_type_fails_with_exit_1() {
    let tmp = private_temp_dir("substrate-wdh3-");
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).expect("create HOME");

    let substrate_home = home.join(".substrate");
    let deps_root = substrate_home.join("deps");
    fs::create_dir_all(&deps_root).expect("create deps root");
    fs::set_permissions(&substrate_home, fs::Permissions::from_mode(0o700))
        .expect("secure SUBSTRATE_HOME fixture");
    fs::write(deps_root.join("packages"), "not a directory\n").expect("seed wrong type");

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .code(1)
        .stderr(predicates::str::contains("deps/packages"))
        .stderr(predicates::str::contains("expected directory"));
}

#[test]
fn test_bootstrap_creation_is_exact_0700_across_umasks() {
    for umask in ["000", "022", "027", "077", "777"] {
        let tmp = private_temp_dir(&format!("substrate-wdh3-umask-{umask}-"));
        let home = tmp.path().join("home");
        fs::create_dir(&home).expect("create HOME");
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
        let substrate_home = home.join(".substrate");

        assert_install_bootstrap_with_umask(&home, &substrate_home, umask);

        let metadata = fs::symlink_metadata(&substrate_home).expect("stat SUBSTRATE_HOME");
        assert_eq!(metadata.mode() & 0o7777, 0o700, "umask {umask}");
        assert_eq!(metadata.uid(), unsafe { libc::geteuid() }, "umask {umask}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_lima_fallback_creation_and_idempotence() {
    let program = lima_private_home_fallback();
    let tmp = private_temp_dir("substrate-lima-home-");
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let target = tmp.path().join("home");

    assert!(run_lima_private_home_fallback(&program, &target)
        .status
        .success());
    assert!(run_lima_private_home_fallback(&program, &target)
        .status
        .success());
    let metadata = fs::symlink_metadata(&target).unwrap();
    assert_eq!(metadata.mode() & 0o7777, 0o700);
    assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
}

#[cfg(target_os = "linux")]
#[test]
fn test_lima_fallback_rejects_invalid_existing_home_without_mutation() {
    let program = lima_private_home_fallback();
    let tmp = private_temp_dir("substrate-lima-invalid-");
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let target = tmp.path().join("home");
    fs::create_dir(&target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();

    let output = run_lima_private_home_fallback(&program, &target);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("wrong-mode"));
    assert_eq!(
        fs::symlink_metadata(&target).unwrap().mode() & 0o7777,
        0o755
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_lima_fallback_rejects_unsafe_parent_without_creation() {
    let program = lima_private_home_fallback();
    let tmp = private_temp_dir("substrate-lima-parent-");
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o770)).unwrap();
    let target = tmp.path().join("home");

    let output = run_lima_private_home_fallback(&program, &target);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsafe"));
    assert!(!target.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn test_lima_fallback_accepts_non_grant_parent_acl() {
    let program = lima_private_home_fallback();
    let tmp = private_temp_dir("substrate-lima-parent-acl-");
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let foreign_uid = unsafe { libc::geteuid() }.saturating_add(1);
    Command::new("setfacl")
        .args(["-m", &format!("u:{foreign_uid}:---")])
        .arg(tmp.path())
        .assert()
        .success();
    let target = tmp.path().join("home");

    let output = run_lima_private_home_fallback(&program, &target);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::symlink_metadata(&target).unwrap().mode() & 0o7777,
        0o700
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_lima_fallback_rejects_parent_acl_grant() {
    let program = lima_private_home_fallback();
    let tmp = private_temp_dir("substrate-lima-parent-acl-grant-");
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let foreign_uid = unsafe { libc::geteuid() }.saturating_add(1);
    Command::new("setfacl")
        .args(["-m", &format!("u:{foreign_uid}:--x")])
        .arg(tmp.path())
        .assert()
        .success();
    let target = tmp.path().join("home");

    let output = run_lima_private_home_fallback(&program, &target);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("foreign-acl"));
    assert!(!target.exists());
}

#[test]
fn test_existing_invalid_home_modes_fail_without_mutation_or_authority_state() {
    for mode in [0o000, 0o755, 0o750, 0o770, 0o777, 0o1700, 0o2700, 0o4700] {
        let tmp = private_temp_dir(&format!("substrate-wdh3-mode-{mode:o}-"));
        let home = tmp.path().join("home");
        let substrate_home = home.join(".substrate");
        fs::create_dir(&home).expect("create HOME");
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
        fs::create_dir(&substrate_home).expect("create existing SUBSTRATE_HOME");
        fs::set_permissions(&substrate_home, fs::Permissions::from_mode(mode))
            .expect("set invalid mode");

        install_bootstrap_driver(&home, &substrate_home)
            .assert()
            .code(5)
            .stderr(predicates::str::contains("wrong-mode"));

        assert_eq!(
            fs::symlink_metadata(&substrate_home).unwrap().mode() & 0o7777,
            mode
        );
        assert!(!substrate_home.join("deps").exists());
        assert!(!substrate_home.join("authority-v1").exists());
    }
}

#[test]
fn test_existing_symlink_home_fails_without_touching_target() {
    let tmp = private_temp_dir("substrate-wdh3-symlink-");
    let home = tmp.path().join("home");
    let target = tmp.path().join("target");
    let substrate_home = home.join(".substrate");
    fs::create_dir(&home).expect("create HOME");
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
    fs::create_dir(&target).expect("create target");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).expect("secure target");
    std::os::unix::fs::symlink(&target, &substrate_home).expect("create home symlink");

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .code(5)
        .stderr(predicates::str::contains("symlink"));

    assert!(!target.join("deps").exists());
    assert!(!target.join("authority-v1").exists());
}

#[test]
fn test_existing_non_directory_home_fails_without_authority_state() {
    let tmp = private_temp_dir("substrate-wdh3-file-");
    let home = tmp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir(&home).expect("create HOME");
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
    fs::write(&substrate_home, b"not a directory\n").expect("create wrong-type home");

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .code(5)
        .stderr(predicates::str::contains("wrong-type"));

    assert_eq!(fs::read(&substrate_home).unwrap(), b"not a directory\n");
    assert!(!home.join("authority-v1").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn test_existing_masked_named_acl_fails_without_descendant_writes() {
    let tmp = private_temp_dir("substrate-wdh3-acl-");
    let home = tmp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir(&home).expect("create HOME");
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
    fs::create_dir(&substrate_home).expect("create SUBSTRATE_HOME");
    fs::set_permissions(&substrate_home, fs::Permissions::from_mode(0o700))
        .expect("secure SUBSTRATE_HOME");
    let foreign_uid = unsafe { libc::geteuid() }.saturating_add(1);
    Command::new("setfacl")
        .args(["-m", &format!("u:{foreign_uid}:---")])
        .arg(&substrate_home)
        .assert()
        .success();

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .code(5)
        .stderr(predicates::str::contains("foreign-acl"));

    assert!(!substrate_home.join("deps").exists());
    assert!(!substrate_home.join("authority-v1").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn test_existing_default_acl_fails_without_descendant_writes() {
    let tmp = private_temp_dir("substrate-wdh3-default-acl-");
    let home = tmp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir(&home).expect("create HOME");
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).expect("secure HOME");
    fs::create_dir(&substrate_home).expect("create SUBSTRATE_HOME");
    fs::set_permissions(&substrate_home, fs::Permissions::from_mode(0o700))
        .expect("secure SUBSTRATE_HOME");
    Command::new("setfacl")
        .args(["-d", "-m", "u::rwx,g::---,o::---"])
        .arg(&substrate_home)
        .assert()
        .success();

    install_bootstrap_driver(&home, &substrate_home)
        .assert()
        .code(5)
        .stderr(predicates::str::contains("foreign-acl"));

    assert!(!substrate_home.join("deps").exists());
    assert!(!substrate_home.join("authority-v1").exists());
}
