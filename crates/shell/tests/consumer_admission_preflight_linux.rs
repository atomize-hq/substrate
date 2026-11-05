#![cfg(target_os = "linux")]

use anyhow::{ensure, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use transport_api_types::{
    WorkspaceCurrentOperationV1 as Operation, WorkspaceCurrentRequestV1 as Request,
    WorkspaceInitializationOfferV1 as Offer,
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofProfile {
    schema_version: u32,
    qualification: String,
    native_binary: PathBuf,
    native_sha256: String,
    writer_binary: PathBuf,
    writer_sha256: String,
    sibling: Request,
    native_home: PathBuf,
    native_runtime: PathBuf,
    native_tmp: PathBuf,
    initialization_effect_path: PathBuf,
    socket: PathBuf,
    selected_location: PathBuf,
    effect_root: PathBuf,
    control: PathBuf,
    native_context: transport_api_types::InstallBootstrapContextCarrierV1,
    current: Request,
    initialization: Offer,
    continuation: String,
}

fn profile_at(path: &Path) -> Result<ProofProfile> {
    ensure!(path.is_absolute(), "absolute proof profile required");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file()
            && meta.uid() == unsafe { libc::geteuid() }
            && meta.mode() & 0o077 == 0
            && meta.nlink() == 1
            && meta.len() <= 32768,
        "protected genuine profile required"
    );
    let profile: ProofProfile = serde_json::from_reader(file)?;
    ensure!(
        profile.schema_version == 1
            && matches!(
                profile.qualification.as_str(),
                "private-source" | "installed"
            ),
        "unsupported proof profile"
    );
    ensure!(
        matches!(
            profile.continuation.as_str(),
            "attach" | "resume" | "owner" | "legacy"
        ),
        "unsupported proof population"
    );
    ensure!(
        [
            &profile.native_binary,
            &profile.writer_binary,
            &profile.native_home,
            &profile.native_runtime,
            &profile.native_tmp,
            &profile.socket,
            &profile.selected_location,
            &profile.effect_root,
            &profile.control
        ]
        .iter()
        .all(|p| p.is_absolute()),
        "absolute native routes required"
    );
    if profile.qualification == "private-source" {
        ensure!(
            std::env::var("TASK22_PRIVATE_NAMESPACE").as_deref() == Ok("joined-v1")
                && std::env::var_os("TASK22_PRIVATE_ROOT")
                    .map(PathBuf::from)
                    .as_ref()
                    == Some(&profile.effect_root),
            "private source fixture identity required"
        );
    }
    ensure!(
        profile.control == profile.effect_root.join("control"),
        "exact harness-only control directory required"
    );
    profile.current.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        matches!(
            profile.current.operation,
            Operation::CurrentActorRead { .. }
        ) && profile.current.native_context == profile.native_context,
        "native context mismatch"
    );
    ensure!(
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(&profile.native_binary)?)
        ) == profile.native_sha256,
        "native source artifact changed"
    );
    ensure!(
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(&profile.writer_binary)?)
        ) == profile.writer_sha256,
        "genuine source writer artifact changed"
    );
    profile.sibling.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        matches!(
            profile.sibling.operation,
            Operation::CurrentActorRead { .. }
        ),
        "genuine sibling read selectors required"
    );
    Ok(profile)
}

fn configured_profile() -> Result<ProofProfile> {
    profile_at(&PathBuf::from(
        std::env::var_os("SUBSTRATE_PREFLIGHT_PROOF_PROFILE").ok_or_else(|| {
            anyhow::anyhow!("SUBSTRATE_PREFLIGHT_PROOF_PROFILE required; no fixture fallback")
        })?,
    ))
}

#[derive(Debug, PartialEq, Eq)]
struct EntrySnapshot {
    device: u64,
    inode: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    links: u64,
    length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
    bytes: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    entries: BTreeMap<PathBuf, EntrySnapshot>,
    mounts: String,
    cgroups: String,
    namespaces: Vec<PathBuf>,
    children: Vec<String>,
}
fn snapshot(profile: &ProofProfile) -> Snapshot {
    fn walk(root: &Path, path: &Path, control: &Path, out: &mut BTreeMap<PathBuf, EntrySnapshot>) {
        if path == control {
            return;
        }
        let m = std::fs::symlink_metadata(path).unwrap();
        out.insert(
            path.strip_prefix(root).unwrap().to_owned(),
            EntrySnapshot {
                device: m.dev(),
                inode: m.ino(),
                mode: m.mode(),
                uid: m.uid(),
                gid: m.gid(),
                links: m.nlink(),
                length: m.len(),
                modified_seconds: m.mtime(),
                modified_nanoseconds: m.mtime_nsec(),
                changed_seconds: m.ctime(),
                changed_nanoseconds: m.ctime_nsec(),
                bytes: if m.is_file() {
                    std::fs::read(path).unwrap()
                } else {
                    Vec::new()
                },
            },
        );
        if m.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                walk(root, &entry.unwrap().path(), control, out);
            }
        }
    }
    let mut entries = BTreeMap::new();
    walk(
        &profile.effect_root,
        &profile.effect_root,
        &profile.control,
        &mut entries,
    );
    let mut children = std::fs::read_dir("/proc/self/task")
        .unwrap()
        .map(|task| {
            std::fs::read_to_string(task.unwrap().path().join("children")).unwrap_or_default()
        })
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>();
    children.sort();
    Snapshot {
        entries,
        mounts: std::fs::read_to_string("/proc/self/mountinfo").unwrap(),
        cgroups: std::fs::read_to_string("/proc/self/cgroup").unwrap(),
        namespaces: ["mnt", "net", "pid", "user", "cgroup"]
            .iter()
            .map(|n| std::fs::read_link(format!("/proc/self/ns/{n}")).unwrap())
            .collect(),
        children,
    }
}

fn native(
    profile: &ProofProfile,
    operation: &str,
    body: Option<&Value>,
    path: Option<&Path>,
) -> Value {
    let transport_api_types::PlatformPrincipalV1::Unix { uid, account } =
        &profile.native_context.context.intended_host_principal
    else {
        panic!("native Unix caller required")
    };
    let request_file = profile.control.join("integration-native-request.json");
    if let Some(body) = body {
        std::fs::write(&request_file, serde_json::to_vec(body).unwrap()).unwrap();
        std::fs::set_permissions(&request_file, std::fs::Permissions::from_mode(0o600)).unwrap();
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(request_file.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(name.as_ptr(), *uid, *uid) }, 0);
    }
    let before = snapshot(profile);
    let mut command = std::process::Command::new("/usr/bin/setpriv");
    command
        .args([
            format!("--reuid={uid}"),
            format!("--regid={uid}"),
            "--clear-groups".into(),
        ])
        .arg(&profile.native_binary)
        .arg("--install-prefix")
        .arg(&profile.native_context.context.host_substrate_home)
        .args(["workspace", operation])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("HOME", &profile.native_home)
        .env("USER", account)
        .env("LOGNAME", account)
        .env("XDG_CONFIG_HOME", profile.native_home.join(".config"))
        .env("XDG_CACHE_HOME", profile.native_home.join(".cache"))
        .env("XDG_DATA_HOME", profile.native_home.join(".local/share"))
        .env("XDG_RUNTIME_DIR", &profile.native_runtime)
        .env("TMPDIR", &profile.native_tmp)
        .env("SUBSTRATE_WORLD_SOCKET", &profile.socket)
        .current_dir(&profile.selected_location);
    if body.is_some() {
        command.arg("--request").arg(&request_file);
    } else if let Some(path) = path {
        command.arg(path);
    }
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command.spawn().expect("actual native adapter");
    let witness = child.id();
    let race = operation == "preflight" && profile.control.join("integration-race-mode").exists();
    let after_writer = if race {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !profile.control.join("preflight-gate-arrived").exists()
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            profile.control.join("preflight-gate-arrived").exists(),
            "native C1 reached actual service before writer"
        );
        let mode = std::fs::read_to_string(profile.control.join("integration-race-mode")).unwrap();
        if mode == "sibling" {
            update(profile, &profile.sibling, false);
            update(profile, &profile.sibling, true);
        } else {
            native_writer(profile, &mode);
        }
        let mut observed = snapshot(profile);
        for children in &mut observed.children {
            *children = children
                .split_whitespace()
                .filter(|pid| *pid != witness.to_string())
                .collect::<Vec<_>>()
                .join(" ");
        }
        observed.children.retain(|s| !s.trim().is_empty());
        std::fs::write(
            profile.control.join("preflight-gate-release"),
            b"actual owner writer ended",
        )
        .unwrap();
        Some(observed)
    } else {
        None
    };
    let output = child
        .wait_with_output()
        .expect("actual native child reaped");
    let reply: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "{operation}: {e}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    if matches!(operation, "open" | "preflight")
        || operation == "initialize-approved" && reply["state"] != "Initialized"
    {
        assert_eq!(
            snapshot(profile),
            after_writer.unwrap_or(before),
            "PF-ZERO actual native {operation}, writer effects measured separately; reply={reply}"
        );
    }
    println!(
        "PF_NATIVE_OPERATION={} state={} reason={}",
        operation,
        reply["state"],
        reply.get("reason").unwrap_or(&Value::Null)
    );
    reply
}
fn native_writer(profile: &ProofProfile, mode: &str) -> String {
    let transport_api_types::PlatformPrincipalV1::Unix { uid, account } =
        &profile.native_context.context.intended_host_principal
    else {
        panic!("Unix")
    };
    let output=std::process::Command::new("/usr/bin/setpriv")
        .args([format!("--reuid={uid}"),format!("--regid={uid}"),"--clear-groups".into()]).arg(&profile.writer_binary)
        .args(["execution::agent_runtime::host_session_authority::facade::workspace_current_private_driver_v1::test_workspace_current_private_producer_driver","--ignored","--exact","--nocapture"])
        .env_clear().env("PATH","/usr/bin:/bin").env("LANG","C").env("LC_ALL","C").env("HOME",&profile.native_home)
        .env("USER",account).env("LOGNAME",account).env("XDG_RUNTIME_DIR",&profile.native_runtime).env("TMPDIR",&profile.native_tmp)
        .env("TASK22_PRIVATE_ROOT",&profile.effect_root).env("TASK22_PRIVATE_NAMESPACE","joined-v1").env("TASK22_DRIVER_MODE",mode)
        .env("TASK22_WORLD_ID",&profile.current.resource.world_id).env("TASK22_WORLD_GENERATION",profile.current.resource.world_generation.to_string())
        .output().expect("actual native source-owner writer");
    assert!(
        output.status.success(),
        "actual source writer {mode}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("PF-REV actual native writer {mode} ended");
    String::from_utf8(output.stdout).unwrap()
}
fn direct_preflight(profile: &ProofProfile, request: &Request) -> Value {
    let path = profile.control.join("transport-request.json");
    std::fs::write(&path, serde_json::to_vec(request).unwrap()).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let before = snapshot(profile);
    let output = native_writer(profile, "preflight-transport");
    assert_eq!(
        snapshot(profile),
        before,
        "direct typed-service read PF-ZERO"
    );
    serde_json::from_str(
        output
            .lines()
            .find_map(|line| line.strip_prefix("TASK22_PREFLIGHT_RESPONSE="))
            .expect("actual direct typed response"),
    )
    .unwrap()
}
fn refresh_current(profile: &ProofProfile, current: &mut Request) {
    native_writer(profile, "observe");
    let fresh: Request =
        serde_json::from_slice(&std::fs::read(profile.control.join("request.json")).unwrap())
            .unwrap();
    current.observation = fresh.observation;
    if let Operation::CurrentActorRead { expected_actor, .. } = &mut current.operation {
        *expected_actor =
            std::fs::read_to_string(profile.control.join("expected-actor.txt")).unwrap();
    }
}
fn raced_preflight(profile: &ProofProfile, current: &Request, mode: &str) -> Value {
    for name in ["preflight-gate-arrived", "preflight-gate-release"] {
        let _ = std::fs::remove_file(profile.control.join(name));
    }
    std::fs::write(profile.control.join("preflight-gate"), b"one bounded race").unwrap();
    std::fs::write(profile.control.join("integration-race-mode"), mode).unwrap();
    let reply = native(profile, "preflight", Some(&json!(current)), None);
    for name in [
        "preflight-gate",
        "preflight-gate-arrived",
        "preflight-gate-release",
        "integration-race-mode",
    ] {
        std::fs::remove_file(profile.control.join(name)).unwrap();
    }
    reply
}
fn prove_actual_settings_cli_writers(profile: &ProofProfile) {
    let folder = Path::new("/run/task22-owner/settings-writer-cli");
    std::fs::create_dir(folder).unwrap();
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o700)).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(folder.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
    let before = snapshot(profile);
    for (kind, key) in [
        ("config", "sync.auto_sync"),
        ("policy", "workspace_authority.workspace_load"),
    ] {
        let revision = folder.join(format!(".{kind}.yaml.owner-revision.v1.json"));
        for (index, action) in ["init", "set", "reset"].into_iter().enumerate() {
            let mut command = std::process::Command::new("/usr/bin/setpriv");
            command
                .args(["--reuid=1000", "--regid=1000", "--clear-groups"])
                .arg(&profile.native_binary)
                .arg("--install-prefix")
                .arg(folder)
                .args([kind, "global", action])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("LANG", "C")
                .env("LC_ALL", "C")
                .env("HOME", &profile.native_home)
                .env("USER", "task22-owner")
                .env("LOGNAME", "task22-owner")
                .env("XDG_CONFIG_HOME", profile.native_home.join(".config"))
                .env("XDG_CACHE_HOME", profile.native_home.join(".cache"))
                .env("XDG_DATA_HOME", profile.native_home.join(".local/share"))
                .env("XDG_RUNTIME_DIR", &profile.native_runtime)
                .env("TMPDIR", &profile.native_tmp)
                .current_dir(folder);
            if action == "set" {
                command.arg(format!(
                    "{key}={}",
                    if kind == "policy" { "deny" } else { "false" }
                ));
            } else if action == "reset" {
                command.arg(key);
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "actual {kind} {action}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let record: Value = serde_json::from_slice(&std::fs::read(&revision).unwrap()).unwrap();
            assert_eq!(
                record["revision"],
                (index + 1) as u64,
                "actual {kind} {action} monotonic owner publication"
            );
            println!(
                "PF-REV actual native CLI {kind} global {action}: revision {}",
                index + 1
            );
        }
    }
    let after = snapshot(profile);
    let relative = Path::new("run/task22-owner/settings-writer-cli");
    for (path, entry) in &after.entries {
        if !path.starts_with(relative) {
            assert_eq!(
                before.entries.get(path),
                Some(entry),
                "settings CLI escaped selected private home {path:?}"
            );
        }
    }
}

fn prove_sticky_tmp_legacy_writers(profile: &ProofProfile, current: &Request) {
    let temp = tempfile::tempdir_in("/tmp").unwrap();
    let root = temp.path();
    assert_eq!(std::fs::metadata("/tmp").unwrap().mode() & 0o7777, 0o1777);
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(root.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
    let commands: Vec<Vec<&str>> = vec![
        vec!["workspace", "init", root.to_str().unwrap()],
        vec!["config", "workspace", "set", "sync.auto_sync=false"],
        vec!["config", "workspace", "reset", "sync.auto_sync"],
        vec![
            "policy",
            "workspace",
            "set",
            "workspace_authority.current_actor_read=deny",
        ],
        vec![
            "policy",
            "workspace",
            "reset",
            "workspace_authority.current_actor_read",
        ],
        vec!["workspace", "disable", root.to_str().unwrap()],
        vec!["workspace", "enable", root.to_str().unwrap()],
    ];
    for arguments in commands {
        let output = std::process::Command::new("/usr/bin/setpriv")
            .args(["--reuid=1000", "--regid=1000", "--clear-groups"])
            .arg(&profile.native_binary)
            .arg("--install-prefix")
            .arg(&profile.native_home)
            .args(&arguments)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &profile.native_home)
            .env("USER", "task22-owner")
            .env("LOGNAME", "task22-owner")
            .env("XDG_RUNTIME_DIR", &profile.native_runtime)
            .env("TMPDIR", &profile.native_tmp)
            .env("SUBSTRATE_OVERRIDE_WORLD", "disabled")
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "legacy sticky /tmp {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let control = root.join(".substrate");
    assert!(control.join("workspace.yaml").is_file() && control.join("policy.yaml").is_file());
    assert!(control.join("git/repo.git").is_dir() && root.join(".gitignore").is_file());
    for entry in std::fs::read_dir(&control).unwrap() {
        assert!(
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("owner-revision"),
            "unsupported legacy writer must not advertise coherent coverage"
        );
    }
    assert!(
        matches!(
            native(profile, "open", None, Some(root))["state"].as_str(),
            Some("Refused" | "Invalid")
        ),
        "unsupported ancestor must not load or offer native initialization"
    );
    let mut unsupported = current.clone();
    unsupported.resource.workspace_path = root.to_str().unwrap().into();
    assert_ne!(
        native(profile, "preflight", Some(&json!(unsupported)), None)["state"],
        "EligibleObservation"
    );
    let empty = root.join("empty");
    std::fs::create_dir(&empty).unwrap();
    let name = std::ffi::CString::new(empty.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
    let mut offer = profile.initialization.clone();
    let meta = std::fs::metadata(&empty).unwrap();
    offer.selected_location = empty.to_str().unwrap().into();
    offer.root_device = meta.dev();
    offer.root_inode = meta.ino();
    let before = snapshot(profile);
    assert_eq!(
        native(
            profile,
            "initialize-approved",
            Some(&json!({"decision":"Approved","offer":offer})),
            None
        )["state"],
        "InitializationFailed"
    );
    assert_eq!(
        snapshot(profile),
        before,
        "strict new initializer never falls back to legacy unsupported custody"
    );
    println!("R22-R0910-IMPL-01 actual legacy init/config/policy/disable/enable at sticky /tmp pass; no counter coverage; new open/preflight/init refuse with zero read/init effects");
}

fn eligible(value: &Value) {
    assert_eq!(value["state"], "EligibleObservation", "{value}");
    assert!(value.get("request_id").is_none() && value.get("may_send").is_none());
}
fn selectors(request: &mut Request, publication: &Value) {
    let actor = match &request.operation {
        Operation::CurrentActorRead { expected_actor, .. } => expected_actor.clone(),
        _ => panic!("read selectors"),
    };
    request.operation = Operation::CurrentActorRead {
        lane_ref: publication["lane_ref"].as_str().unwrap().into(),
        association_incarnation: publication["association_incarnation"]
            .as_str()
            .unwrap()
            .into(),
        association_revision: publication["association_revision"].as_u64().unwrap(),
        backend_custody_ref: publication["backend_custody_ref"].as_str().unwrap().into(),
        expected_actor: actor,
    };
}
fn update(profile: &ProofProfile, request: &Request, active: bool) -> Value {
    let mut current = request.clone();
    current.operation = Operation::PublishAssociation;
    let reply = native(
        profile,
        "publish-world-association",
        Some(&json!({"current":current,"active":active})),
        None,
    );
    assert_eq!(reply["state"], "AssociationPublished", "{reply}");
    reply
}

#[test]
fn test_workspace_proof_missing_profile_fails_explicitly() {
    assert!(profile_at(Path::new("relative.json")).is_err());
    assert!(profile_at(Path::new("/nonexistent/substrate-preflight-profile.json")).is_err());
}

#[test]
#[ignore = "requires an explicit protected genuine no-send profile; never seeds owner state"]
fn test_real_workspace_open_init_and_existing_world_no_send() {
    let profile = configured_profile().expect("genuine profile required");
    let raw =
        std::fs::read_to_string(std::env::var_os("SUBSTRATE_PREFLIGHT_PROOF_PROFILE").unwrap())
            .unwrap();
    assert!(
        serde_json::from_str::<ProofProfile>(&raw.replacen('{', "{\"schema_version\":1,", 1))
            .is_err()
    );
    assert!(
        serde_json::from_str::<ProofProfile>(&raw.replacen('{', "{\"unexpected\":true,", 1))
            .is_err()
    );
    let loaded = native(&profile, "open", None, Some(&profile.selected_location));
    assert_eq!(loaded["state"], "Loaded", "{loaded}");
    let mut current = profile.current.clone();
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    if profile.continuation == "legacy" {
        prove_sticky_tmp_legacy_writers(&profile, &current);
        prove_actual_settings_cli_writers(&profile);
        eligible(&native(&profile, "preflight", Some(&json!(current)), None));
        return;
    }
    if profile.continuation == "owner" {
        native_writer(&profile, "unmediated-workspace-owner-restore");
        assert_eq!(
            native(&profile, "preflight", Some(&json!(current)), None)["reason"],
            "Changed"
        );
        assert_eq!(direct_preflight(&profile, &current)["reason"], "Changed");
        let loaded = native(&profile, "open", None, Some(&profile.selected_location));
        assert_eq!(loaded["state"], "Loaded");
        println!("PF-REV/PF-ZERO exact Workspace owner same-inode pre-cut restoration refuses via selected source receipt; local Loaded remains separate");
        return;
    }
    prove_actual_settings_cli_writers(&profile);
    native_writer(&profile, "actor-only");
    let actor_only_open = native(&profile, "open", None, Some(&profile.selected_location));
    assert_eq!(actor_only_open["state"], "Refused");
    assert_eq!(
        actor_only_open["reason"], "Denied",
        "actor projection grant never grants enumeration/config load"
    );
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "load-only");
    assert_eq!(
        native(&profile, "open", None, Some(&profile.selected_location))["state"],
        "Loaded"
    );
    assert_eq!(
        native(&profile, "preflight", Some(&json!(current)), None)["reason"],
        "Denied"
    );
    assert_eq!(direct_preflight(&profile, &current)["reason"], "Denied");
    let mutation_before = snapshot(&profile);
    let publication = native(
        &profile,
        "publish-world-association",
        Some(&json!({"current": current, "active":false})),
        None,
    );
    assert_ne!(publication["state"], "AssociationPublished");
    assert_eq!(
        snapshot(&profile),
        mutation_before,
        "load grant never permits association mutation"
    );
    let denied_init = native(
        &profile,
        "initialize-approved",
        Some(&json!({"decision":"Approved","offer":profile.initialization})),
        None,
    );
    assert_eq!(
        denied_init["state"], "InitializationFailed",
        "load grant never permits initialization"
    );
    native_writer(&profile, "policy-restore");
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    println!("PF-AUTH/PF-ZERO distinct actor-only/load-only grants; no implicit mutation or service grant");
    // A genuine same-principal local context is not a Root installed-home claim.
    native_writer(&profile, "alternate-native-context");
    let mut alternate = profile.clone();
    alternate.native_context = serde_json::from_slice(
        &std::fs::read(profile.control.join("alternate-native-context.json")).unwrap(),
    )
    .unwrap();
    alternate.selected_location = PathBuf::from("/run/task22-owner/alternate-local-workspace");
    let offered = native(&alternate, "open", None, Some(&alternate.selected_location));
    assert_eq!(offered["state"], "InitializationOffered", "{offered}");
    let alternative_offer = offered["offer"].clone();
    let wrong_context = native(
        &profile,
        "initialize-approved",
        Some(&json!({
            "decision":"Approved", "offer":alternative_offer,
        })),
        None,
    );
    assert_eq!(
        wrong_context["state"], "InitializationFailed",
        "approval cannot cross native contexts"
    );
    let before_alternate = snapshot(&profile);
    let initialized = native(
        &alternate,
        "initialize-approved",
        Some(&json!({
            "decision":"Approved", "offer":alternative_offer,
        })),
        None,
    );
    assert_eq!(initialized["state"], "Initialized", "{initialized}");
    let after_alternate = snapshot(&profile);
    let selected_relative = profile
        .effect_root
        .join("run/task22-owner/alternate-local-workspace");
    let selected_relative = selected_relative
        .strip_prefix(&profile.effect_root)
        .unwrap();
    for (path, entry) in &after_alternate.entries {
        if !path.starts_with(selected_relative) {
            assert_eq!(
                before_alternate.entries.get(path),
                Some(entry),
                "alternate init write {path:?}"
            );
        }
    }
    assert_eq!(
        native(&alternate, "open", None, Some(&alternate.selected_location))["state"],
        "Loaded"
    );
    let mut alternate_request = current.clone();
    alternate_request.native_context = alternate.native_context.clone();
    alternate_request.resource.workspace_path =
        alternate.selected_location.to_str().unwrap().into();
    assert_eq!(
        direct_preflight(&profile, &alternate_request)["reason"],
        "Denied",
        "local context never elevates into Root accepted-home authority"
    );
    println!("PF-AUTH/PF-OPEN/PF-INIT/PF-ZERO alternate genuine native-local context loads and initializes; Root service independently denies");
    for mismatch in [
        "actor",
        "lane",
        "association",
        "generation",
        "principal",
        "root",
    ] {
        let mut bad = current.clone();
        match mismatch {
            "actor" => {
                if let Operation::CurrentActorRead { expected_actor, .. } = &mut bad.operation {
                    *expected_actor = "wrong-actor".into()
                }
            }
            "lane" => {
                if let Operation::CurrentActorRead { lane_ref, .. } = &mut bad.operation {
                    *lane_ref = "wrong-lane".into()
                }
            }
            "association" => {
                if let Operation::CurrentActorRead {
                    association_revision,
                    ..
                } = &mut bad.operation
                {
                    *association_revision += 1
                }
            }
            "generation" => bad.resource.world_generation += 1,
            "principal" => {
                bad.native_context.context.intended_host_principal =
                    transport_api_types::PlatformPrincipalV1::Unix {
                        uid: 99999,
                        account: "spoofed".into(),
                    }
            }
            _ => bad.resource.workspace_path.push_str("/missing"),
        }
        let refused = native(&profile, "preflight", Some(&json!(bad)), None);
        assert_ne!(refused["state"], "EligibleObservation", "{mismatch}");
        println!(
            "PF-AUTH/PF-EXACT/PF-ZERO native mismatch {mismatch}: {}",
            refused["reason"]
        );
    }
    let before_sibling = native(&profile, "preflight", Some(&json!(current)), None);
    eligible(&before_sibling);
    let sibling = raced_preflight(&profile, &current, "sibling");
    eligible(&sibling);
    assert_eq!(
        before_sibling, sibling,
        "PF-REV independent sibling does not change selected vector"
    );
    let mut sibling_request = profile.sibling.clone();
    let sibling_publication = update(&profile, &sibling_request, true);
    selectors(&mut sibling_request, &sibling_publication);
    eligible(&native(
        &profile,
        "preflight",
        Some(&json!(sibling_request)),
        None,
    ));
    native_writer(&profile, "unmediated-sibling-association-restore");
    assert_eq!(
        native(&profile, "preflight", Some(&json!(sibling_request)), None)["reason"],
        "Changed",
        "pre-cut same-inode association restoration invalidates owner snapshot receipt"
    );
    assert_eq!(
        direct_preflight(&profile, &sibling_request)["reason"],
        "Changed"
    );
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    println!("PF-REV/PF-ZERO unmediated pre-cut selected record restoration refuses; unaffected World remains eligible");
    for mode in [
        "config-aba",
        "workspace-config-aba",
        "policy-aba",
        "workspace-policy-aba",
        "disabled-aba",
        "root-attributes-aba",
        "home-attributes-aba",
        "control-attributes-aba",
        "ancestor-attributes-aba",
    ] {
        let reply = raced_preflight(&profile, &current, mode);
        assert_ne!(
            reply["state"], "EligibleObservation",
            "PF-REV {mode} across C1/C2"
        );
        eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    }
    let reply = raced_preflight(&profile, &current, "policy-revoke");
    assert_ne!(reply["state"], "EligibleObservation");
    assert_ne!(
        native(&profile, "preflight", Some(&json!(current)), None)["state"],
        "EligibleObservation",
        "PF-LATER current read revoked"
    );
    for target in [
        &profile.selected_location,
        Path::new(&profile.initialization.selected_location),
        &profile.selected_location.join("missing-target"),
    ] {
        let mut denied = current.clone();
        denied.resource.workspace_path = target.to_string_lossy().into_owned();
        assert_eq!(
            native(&profile, "preflight", Some(&json!(denied)), None),
            json!({"state":"Refused","reason":"Denied"}),
            "uniform native denied privacy"
        );
        assert_eq!(
            direct_preflight(&profile, &denied),
            json!({"state":"Refused","reason":"Denied"}),
            "uniform direct service denied privacy"
        );
    }
    native_writer(&profile, "workspace-read-override");
    assert_eq!(
        native(&profile, "open", None, Some(&profile.selected_location))["state"],
        "Loaded",
        "Workspace allow overrides global deny"
    );
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "workspace-policy-remove");
    assert_eq!(
        native(&profile, "preflight", Some(&json!(current)), None),
        json!({"state":"Refused","reason":"Denied"})
    );
    native_writer(&profile, "policy-restore");
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "global-config-absence");
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "global-config-present");
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "absence-coverage-missing");
    assert_eq!(
        native(&profile, "preflight", Some(&json!(current)), None)["reason"],
        "Unsupported",
        "uncovered absent policy is never eligible"
    );
    native_writer(&profile, "publish-absence-coverage");
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    let old = current.clone();
    update(&profile, &current, false);
    assert_ne!(
        native(&profile, "preflight", Some(&json!(old)), None)["state"],
        "EligibleObservation",
        "PF-LATER revoked destination"
    );
    let publication = update(&profile, &current, true);
    selectors(&mut current, &publication);
    assert_ne!(
        native(&profile, "preflight", Some(&json!(old)), None)["state"],
        "EligibleObservation",
        "PF-REV same-state association ABA retains new revision"
    );
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    let init_path = Path::new(&profile.initialization.selected_location);
    let offered = native(&profile, "open", None, Some(init_path));
    assert_eq!(offered["state"], "InitializationOffered", "{offered}");
    assert_eq!(
        offered["offer"],
        json!(profile.initialization),
        "exact genuine offer/settings/effects"
    );
    for decision in ["Denied", "Cancelled"] {
        let reply = native(
            &profile,
            "initialize-approved",
            Some(&json!({"decision":decision})),
            None,
        );
        assert_eq!(reply["state"], "NotInitialized");
    }
    let before = snapshot(&profile);
    let initialized = native(
        &profile,
        "initialize-approved",
        Some(&json!({"decision":"Approved","offer":profile.initialization})),
        None,
    );
    assert_eq!(initialized["state"], "Initialized", "{initialized}");
    let after = snapshot(&profile);
    println!(
        "PF_INIT_APPROVED_CREATED_ENTRIES={}",
        after
            .entries
            .keys()
            .filter(|path| !before.entries.contains_key(*path))
            .count()
    );
    let relative = profile
        .initialization_effect_path
        .strip_prefix(&profile.effect_root)
        .expect("approved initialization root within measured effect root");
    for (path, value) in &after.entries {
        if !path.starts_with(relative) {
            assert_eq!(
                before.entries.get(path),
                Some(value),
                "approved init changed outside declared root {path:?}"
            );
        }
    }
    assert_eq!(
        native(&profile, "open", None, Some(init_path))["state"],
        "Loaded"
    );
    assert_eq!(
        native(&profile, "open", None, Some(init_path))["state"],
        "Loaded"
    );
    let retry = native(
        &profile,
        "initialize-approved",
        Some(&json!({"decision":"Approved","offer":profile.initialization})),
        None,
    );
    assert_eq!(
        retry["state"], "InitializationFailed",
        "no second approval or repair"
    );
    native_writer(&profile, "registered");
    assert_ne!(
        native(&profile, "preflight", Some(&json!(current)), None)["state"],
        "EligibleObservation",
        "PF-REV genuine HSA root/authority publication"
    );
    refresh_current(&profile, &mut current);
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    let modes: &[&str] = if profile.continuation == "resume" {
        &["retained", "settled", "resume", "retained"]
    } else {
        &[
            "retained", "settled", "attach", "detach", "attach", "retained",
        ]
    };
    for mode in modes {
        native_writer(&profile, mode);
        refresh_current(&profile, &mut current);
        eligible(&native(&profile, "preflight", Some(&json!(current)), None));
        println!("PF-EXACT/PF-ZERO genuine native continuation {mode}");
    }
    if profile.continuation == "attach" {
        let before = snapshot(&profile);
        let witness = native_writer(&profile, "stop-retained-refusal");
        assert!(witness.contains("TASK22_STOP_SUPPORT_CEILING=trailing-retained"));
        assert_eq!(
            snapshot(&profile),
            before,
            "genuine unsupported Stop is read-only"
        );
        println!("{witness}");
    }
    std::fs::write(
        profile.control.join("proof-current-read.json"),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(
        profile.control.join("proof-current-read.json"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let witness = native_writer(&profile, "world-fork-ceiling");
    assert!(witness.contains("TASK22_WORLD_FORK_SUPPORT_CEILING="));
    println!("{witness}");
    if profile.control.join("world-fork-read.json").exists() {
        let fork: Request = serde_json::from_slice(
            &std::fs::read(profile.control.join("world-fork-read.json")).unwrap(),
        )
        .unwrap();
        assert_ne!(
            native(&profile, "preflight", Some(&json!(fork)), None)["state"],
            "EligibleObservation",
            "genuine unsupported World Fork has no invented Lane/World custody"
        );
        assert_ne!(
            direct_preflight(&profile, &fork)["state"],
            "EligibleObservation"
        );
    }
    refresh_current(&profile, &mut current);
    eligible(&native(&profile, "preflight", Some(&json!(current)), None));
    native_writer(&profile, "unmediated-config-restore");
    assert_ne!(
        native(&profile, "preflight", Some(&json!(current)), None)["state"],
        "EligibleObservation",
        "PF-REV unmediated same-byte restoration refuses, no ledger repair"
    );

    std::fs::write(
        profile.control.join("proof-current-read.json"),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(
        profile.control.join("proof-current-read.json"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let before_failure = snapshot(&profile);
    let failure = native_writer(&profile, "selected-receipt-failure");
    let effect: Value = serde_json::from_str(
        failure
            .lines()
            .find_map(|l| l.strip_prefix("TASK22_SELECTED_UPDATE_FAILURE="))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(effect["association"]["published_visible"], true);
    assert_eq!(effect["association"]["directory_sync_confirmed"], true);
    assert_eq!(effect["receipt"]["staging_created"], false);
    assert_eq!(effect["receipt"]["published_visible"], false);
    let after_failure = snapshot(&profile);
    let association_name = format!(
        "workspace-world-{:x}.v1.json",
        Sha256::digest(current.resource.orchestration_session_id.as_bytes())
    );
    let control_relative = Path::new("run/task22-owner/workspace/.substrate");
    for (path, entry) in &after_failure.entries {
        if path != control_relative && path != &control_relative.join(&association_name) {
            assert_eq!(
                before_failure.entries.get(path),
                Some(entry),
                "selected publication failure escaped exact effects {path:?}"
            );
        }
    }
    println!("PF-REV selected owner component fault after durable association/before receipt: {effect}; exact writer delta measured");
    println!("PF-OPEN/INIT/AUTH/EXACT/REV/ZERO/LATER genuine native source profile completed; qualification={}",profile.qualification);
}
