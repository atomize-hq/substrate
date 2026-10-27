#![cfg(target_os = "linux")]

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const OWNER: &str = "workspace-state.v1.json";

struct Fixture {
    temp: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir_in(
            std::env::var_os("XDG_RUNTIME_DIR").expect("private native fixture parent"),
        )
        .unwrap();
        let home = temp.path().join("product");
        let root = temp.path().join("workspace");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir_all(root.join(".substrate")).unwrap();
        fs::create_dir(root.join("nested")).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(root.join(".substrate"), fs::Permissions::from_mode(0o755)).unwrap();
        for path in [
            home.join("config.yaml"),
            home.join("policy.yaml"),
            root.join(".substrate/workspace.yaml"),
            root.join(".substrate/policy.yaml"),
        ] {
            fs::write(&path, b"{}\n").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        fs::write(root.join(".gitignore"), b"original\n").unwrap();
        Self { temp, home, root }
    }

    fn command(&self, selected: &Path, publish: bool) -> Command {
        let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/substrate");
        let mut cmd = Command::new(binary);
        for name in [
            "SUBSTRATE_HOME",
            "SUBSTRATE_INSTALL_PREFIX",
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
            "SHIM_TRACE_LOG",
            "SUBSTRATE_OVERRIDE_WORLD",
        ] {
            cmd.env_remove(name);
        }
        cmd.current_dir(self.temp.path())
            .arg("--install-prefix")
            .arg(&self.home)
            .arg("workspace");
        if publish {
            cmd.arg("init").arg(selected).arg("--publish-owner");
        } else {
            cmd.arg("status").arg(selected).arg("--json");
        }
        cmd
    }

    fn run(&self, publish: bool) -> Output {
        self.command(&self.root, publish).output().unwrap()
    }

    fn status(&self, expected: &str) -> Value {
        let before = snapshot(self.temp.path());
        let output = self.run(false);
        let value: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|_| panic!("status JSON: {output:?}"));
        assert_eq!(value["state"], expected, "{output:?}");
        assert_eq!(snapshot(self.temp.path()), before, "read side effects");
        if expected == "OwnerStateObserved" {
            assert!(output.status.success(), "{output:?}");
        }
        value
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u32, Vec<u8>)> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, (u32, Vec<u8>)>) {
        let meta = fs::symlink_metadata(path).unwrap();
        let bytes = if meta.is_file() {
            fs::read(path)
                .unwrap_or_else(|error| format!("unreadable:{:?}", error.kind()).into_bytes())
        } else if meta.file_type().is_symlink() {
            fs::read_link(path)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec()
        } else {
            Vec::new()
        };
        result.insert(
            path.strip_prefix(root).unwrap().to_owned(),
            (meta.mode(), bytes),
        );
        if meta.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn test_workspace_owner_production_publisher_fresh_status_nested_idempotence_exact_effects() {
    let f = Fixture::new();
    f.status("OwnerStateMissing");
    let before = snapshot(f.temp.path());
    let output = f.run(true);
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Published"));
    let mut after = snapshot(f.temp.path());
    let record = after
        .remove(&PathBuf::from("workspace/.substrate").join(OWNER))
        .expect("only owner record added");
    assert_eq!(record.0 & 0o7777, 0o600);
    assert_eq!(after, before, "settings, inventory and modes unchanged");
    let value = f.status("OwnerStateObserved");
    for key in [
        "config_revision",
        "policy_revision",
        "lanes",
        "destinations",
        "consumer_read_authorization",
    ] {
        assert_eq!(value[key], "Unsupported");
    }
    let output = f.command(&f.root.join("nested"), false).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        value
    );
    let before = snapshot(f.temp.path());
    let output = f.command(&f.root.join("nested"), true).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("AlreadyPublished"));
    assert_eq!(snapshot(f.temp.path()), before);
    assert_eq!(f.status("OwnerStateObserved"), value);
}

#[test]
fn test_workspace_owner_concurrent_cli_publishers_have_one_identity() {
    let f = Fixture::new();
    let first = f
        .command(&f.root, true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let second = f
        .command(&f.root, true)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let a = first.wait_with_output().unwrap();
    let b = second.wait_with_output().unwrap();
    assert!(a.status.success() && b.status.success(), "{a:?} {b:?}");
    let outputs = [
        String::from_utf8(a.stdout).unwrap(),
        String::from_utf8(b.stdout).unwrap(),
    ];
    assert_eq!(
        outputs
            .iter()
            .filter(|s| s.starts_with("substrate: Published "))
            .count(),
        1
    );
    assert_eq!(
        outputs
            .iter()
            .filter(|s| s.starts_with("substrate: AlreadyPublished "))
            .count(),
        1
    );
    let value = f.status("OwnerStateObserved");
    for output in outputs {
        assert!(output.contains(value["workspace_ref"].as_str().unwrap()));
        assert!(output.contains(value["store_incarnation"].as_str().unwrap()));
    }
    assert!(!f
        .root
        .join(".substrate/.workspace-state.v1.pending")
        .exists());
}

#[test]
fn test_workspace_owner_status_during_exclusive_publication_lock_is_read_only() {
    use std::os::fd::AsRawFd;
    let f = Fixture::new();
    let directory = fs::File::open(f.root.join(".substrate")).unwrap();
    // SAFETY: live native-owned fixture directory descriptor.
    assert_eq!(
        unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX) },
        0
    );
    let before = snapshot(f.temp.path());
    let output = f.run(false);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["state"], "PublicationInProgress");
    assert!(value.get("workspace_ref").is_none());
    assert_eq!(before, snapshot(f.temp.path()));
    drop(directory);
    f.status("OwnerStateMissing");
}

#[test]
fn test_workspace_owner_strict_schema_and_native_binding_negative_matrix() {
    for case in [
        "unknown",
        "duplicate",
        "nested_duplicate",
        "version",
        "revision",
        "coverage",
        "empty_workspace",
        "empty_store",
        "malformed_id",
        "principal",
        "home",
        "root",
        "control",
        "record",
        "truncated",
        "oversized",
    ] {
        let f = Fixture::new();
        assert!(f.run(true).status.success());
        let file = f.root.join(".substrate").join(OWNER);
        let original = fs::read_to_string(&file).unwrap();
        let mut value: Value = serde_json::from_str(&original).unwrap();
        match case {
            "unknown" => value["unknown"] = true.into(),
            "version" => value["version"] = 99.into(),
            "revision" => value["state_revision"] = 2.into(),
            "coverage" => value["revision_coverage"]["config"] = "Observed".into(),
            "empty_workspace" => value["workspace_ref"] = "".into(),
            "empty_store" => value["store_incarnation"] = "".into(),
            "malformed_id" => value["workspace_ref"] = "ws_NOT_CANONICAL".into(),
            "principal" => value["native_owner"]["uid"] = u32::MAX.into(),
            "home" => value["product_home"]["inode"] = 0.into(),
            "root" => value["workspace_root"]["inode"] = 0.into(),
            "control" => value["control_root"]["inode"] = 0.into(),
            "record" => value["record_identity"]["inode"] = 0.into(),
            _ => (),
        }
        let bytes = match case {
            "duplicate" => {
                format!("{{\"version\":1,{}", original.strip_prefix('{').unwrap()).into_bytes()
            }
            "nested_duplicate" => original
                .replace("\"native_owner\":{", "\"native_owner\":{\"uid\":0,")
                .into_bytes(),
            "truncated" => b"{".to_vec(),
            "oversized" => vec![b' '; 17000],
            _ => serde_json::to_vec(&value).unwrap(),
        };
        fs::write(&file, bytes).unwrap();
        let expected = match case {
            "version" | "revision" | "coverage" => "UnsupportedOwnerState",
            "principal" | "home" | "root" | "control" | "record" => "BindingMismatch",
            "oversized" => "UnsafeOwnerState",
            _ => "CorruptOwnerState",
        };
        let status = f.status(expected);
        assert!(status.get("workspace_ref").is_none());
        let before = snapshot(f.temp.path());
        let refused = f.run(true);
        assert!(!refused.status.success(), "{case}: {refused:?}");
        assert_eq!(before, snapshot(f.temp.path()), "no replacement for {case}");
        for output in [&refused.stdout, &refused.stderr] {
            let text = String::from_utf8_lossy(output);
            assert!(!text.contains(f.home.to_str().unwrap()));
            assert!(!text.contains(f.root.to_str().unwrap()));
        }
    }
}

#[test]
fn test_workspace_owner_copy_move_alias_and_replacement_bindings() {
    for case in [
        "file_copy",
        "root_copy",
        "root_move",
        "control_replacement",
        "different_home",
        "alias",
    ] {
        let f = Fixture::new();
        assert!(f.run(true).status.success());
        let record = f.root.join(".substrate").join(OWNER);
        let mut selected = f.root.clone();
        let mut home = f.home.clone();
        match case {
            "file_copy" => {
                fs::rename(&record, record.with_extension("old")).unwrap();
                fs::copy(record.with_extension("old"), &record).unwrap();
            }
            "root_copy" => {
                selected = f.temp.path().join("copied");
                fs::create_dir_all(selected.join(".substrate")).unwrap();
                for name in ["workspace.yaml", "policy.yaml", OWNER] {
                    fs::copy(
                        f.root.join(".substrate").join(name),
                        selected.join(".substrate").join(name),
                    )
                    .unwrap();
                }
            }
            "root_move" => {
                selected = f.temp.path().join("moved");
                fs::rename(&f.root, &selected).unwrap();
            }
            "control_replacement" => {
                fs::rename(f.root.join(".substrate"), f.root.join(".old")).unwrap();
                fs::create_dir(f.root.join(".substrate")).unwrap();
                for name in ["workspace.yaml", "policy.yaml", OWNER] {
                    fs::copy(
                        f.root.join(".old").join(name),
                        f.root.join(".substrate").join(name),
                    )
                    .unwrap();
                }
            }
            "different_home" => {
                home = f.temp.path().join("other-home");
                fs::create_dir(&home).unwrap();
                fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
                for name in ["config.yaml", "policy.yaml"] {
                    fs::copy(f.home.join(name), home.join(name)).unwrap();
                }
            }
            "alias" => {
                selected = f.temp.path().join("alias");
                std::os::unix::fs::symlink(&f.root, &selected).unwrap();
            }
            _ => unreachable!(),
        }
        let before = snapshot(f.temp.path());
        let mut command = f.command(&selected, false);
        if case == "different_home" {
            command = Command::new(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/substrate"),
            );
            command
                .current_dir(f.temp.path())
                .arg("--install-prefix")
                .arg(home)
                .args(["workspace", "status"])
                .arg(&selected)
                .arg("--json");
            for name in [
                "SUBSTRATE_HOME",
                "SUBSTRATE_INSTALL_PREFIX",
                "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
            ] {
                command.env_remove(name);
            }
        }
        let output = command.output().unwrap();
        assert!(!output.status.success(), "{case}: {output:?}");
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(value.get("workspace_ref").is_none());
        assert_eq!(
            value["state"],
            if case == "alias" {
                "WorkspaceBlocked"
            } else {
                "BindingMismatch"
            },
            "{case}"
        );
        assert_eq!(before, snapshot(f.temp.path()));
    }
}

#[test]
fn test_workspace_owner_partial_special_files_and_modes_refuse_without_repair() {
    for case in [
        "pending",
        "pending_with_final",
        "pending_symlink",
        "pending_directory",
        "pending_fifo",
        "pending_socket",
        "pending_hardlink",
        "owner_symlink",
        "owner_hardlink",
        "owner_fifo",
        "owner_socket",
        "owner_directory",
        "owner_mode",
        "root_mode",
        "control_mode",
    ] {
        let f = Fixture::new();
        if matches!(case, "pending_with_final" | "owner_mode" | "owner_hardlink") {
            assert!(f.run(true).status.success());
        }
        let owner = f.root.join(".substrate").join(OWNER);
        let pending = f.root.join(".substrate/.workspace-state.v1.pending");
        let mut socket = None;
        match case {
            "pending" | "pending_with_final" => fs::write(&pending, b"partial").unwrap(),
            "pending_symlink" => {
                std::os::unix::fs::symlink(f.root.join(".substrate/workspace.yaml"), &pending)
                    .unwrap()
            }
            "pending_directory" => fs::create_dir(&pending).unwrap(),
            "pending_fifo" => {
                let name = std::ffi::CString::new(pending.as_os_str().as_encoded_bytes()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
            }
            "pending_socket" => socket = Some(bind_fixture_socket(&pending)),
            "pending_hardlink" => {
                let other = f.root.join("unrelated-file");
                fs::write(&other, b"partial publication residue").unwrap();
                fs::hard_link(other, &pending).unwrap();
            }
            "owner_symlink" => {
                std::os::unix::fs::symlink(f.root.join(".substrate/workspace.yaml"), &owner)
                    .unwrap()
            }
            "owner_hardlink" => fs::hard_link(&owner, f.root.join("linked-owner")).unwrap(),
            "owner_fifo" => {
                let name = std::ffi::CString::new(owner.as_os_str().as_encoded_bytes()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
            }
            "owner_socket" => socket = Some(bind_fixture_socket(&owner)),
            "owner_directory" => fs::create_dir(&owner).unwrap(),
            "owner_mode" => fs::set_permissions(&owner, fs::Permissions::from_mode(0o644)).unwrap(),
            "root_mode" => fs::set_permissions(&f.root, fs::Permissions::from_mode(0o775)).unwrap(),
            "control_mode" => {
                fs::set_permissions(f.root.join(".substrate"), fs::Permissions::from_mode(0o775))
                    .unwrap()
            }
            _ => unreachable!(),
        }
        let expected = if case.starts_with("pending") {
            "PartialPublication"
        } else {
            "UnsafeOwnerState"
        };
        f.status(expected);
        let before = snapshot(f.temp.path());
        assert!(!f.run(true).status.success(), "{case}");
        assert_eq!(before, snapshot(f.temp.path()), "{case}");
        drop(socket);
    }
}

fn bind_fixture_socket(path: &Path) -> std::os::unix::net::UnixListener {
    use std::os::fd::AsRawFd;
    let parent = fs::File::open(path.parent().unwrap()).unwrap();
    // Keep the fixture address below Linux SUN_LEN even with a long private test parent.
    let short = format!(
        "/proc/self/fd/{}/{}",
        parent.as_raw_fd(),
        path.file_name().unwrap().to_str().unwrap()
    );
    std::os::unix::net::UnixListener::bind(short).unwrap()
}

#[test]
fn test_workspace_owner_spoofed_native_context_precedes_target_disclosure() {
    use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};
    let f = Fixture::new();
    // Negative carrier only: it cannot match this fresh process's actual effective UID.
    let context =
        InstallBootstrapContextV1::new_unix(f.home.to_str().unwrap(), "spenser", u32::MAX).unwrap();
    let carrier = InstallBootstrapContextCarrierV1::from_context(context).unwrap();
    let before = snapshot(f.temp.path());
    let output = f
        .command(&f.root, false)
        .arg("--install-bootstrap-context-v1")
        .arg(carrier.encode().unwrap())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stdout.is_empty(),
        "no target existence/identity disclosed: {output:?}"
    );
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("invalid install bootstrap context"),
        "native binding branch reached: {error}"
    );
    assert!(!error.contains(f.root.to_str().unwrap()));
    assert!(!error.contains(f.home.to_str().unwrap()));
    assert_eq!(before, snapshot(f.temp.path()));
}

fn set_acl(path: &Path, default: bool, masked: bool) {
    use std::os::fd::AsRawFd;
    let file = fs::File::open(path).unwrap();
    let mut bytes = 2u32.to_le_bytes().to_vec();
    // POSIX ACL USER_OBJ, named USER, GROUP_OBJ, MASK, OTHER. Named UID is foreign;
    // a zero mask still must be refused by the conservative authority posture.
    let regular = file.metadata().unwrap().is_file();
    for (tag, permissions, id) in [
        (1u16, if regular { 6u16 } else { 7u16 }, u32::MAX),
        (2, 4, 65534),
        (4, if regular { 0 } else { 5 }, u32::MAX),
        (16, if masked || regular { 0 } else { 5 }, u32::MAX),
        (32, if regular { 0 } else { 5 }, u32::MAX),
    ] {
        bytes.extend(tag.to_le_bytes());
        bytes.extend(permissions.to_le_bytes());
        bytes.extend(id.to_le_bytes());
    }
    let name = if default {
        c"system.posix_acl_default"
    } else {
        c"system.posix_acl_access"
    };
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len(),
                0,
            )
        },
        0,
        "native fixture ACL support required: {}",
        std::io::Error::last_os_error()
    );
}

#[test]
fn test_workspace_owner_actual_named_masked_default_acls_are_rejected() {
    for case in [
        "root_access",
        "root_masked",
        "root_default",
        "control_access",
        "control_default",
        "owner_access",
        "ancestor_access",
    ] {
        let f = Fixture::new();
        assert!(f.run(true).status.success());
        let path = match case {
            "root_access" | "root_masked" | "root_default" => f.root.clone(),
            "control_access" | "control_default" => f.root.join(".substrate"),
            "owner_access" => f.root.join(".substrate").join(OWNER),
            _ => f.temp.path().to_path_buf(),
        };
        set_acl(&path, case.ends_with("default"), case == "root_masked");
        f.status("UnsafeOwnerState");
        let before = snapshot(f.temp.path());
        assert!(!f.run(true).status.success(), "{case}");
        assert_eq!(before, snapshot(f.temp.path()));
    }
}

#[test]
fn test_workspace_owner_force_examples_missing_home_and_invalid_boundary_no_effects() {
    for flag in ["--force", "--examples"] {
        let f = Fixture::new();
        let before = snapshot(f.temp.path());
        let output = f.command(&f.root, true).arg(flag).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(before, snapshot(f.temp.path()));
    }
    let f = Fixture::new();
    fs::remove_dir_all(&f.home).unwrap();
    let before = snapshot(f.temp.path());
    let output = f.run(false);
    assert!(!output.status.success());
    assert_eq!(before, snapshot(f.temp.path()));
    assert!(!f.home.exists());
    for file in ["disabled", "workspace.yaml", "policy.yaml"] {
        let f = Fixture::new();
        if file == "disabled" {
            fs::write(f.root.join(".substrate/workspace.disabled"), b"").unwrap();
        } else {
            fs::write(f.root.join(".substrate").join(file), b"[invalid: patch").unwrap();
        }
        let before = snapshot(f.temp.path());
        assert!(!f.run(true).status.success(), "{file}");
        assert!(!f.run(false).status.success(), "{file}");
        assert_eq!(before, snapshot(f.temp.path()));
    }
}
