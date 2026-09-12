use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStringExt;

use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use config_projection::{
    CanonicalCgroupIdentityV1, CanonicalDirectoryV1, CodexSetupReadyAttestationV1,
    ConfigProjectionArtifactRoleV1, ConfigProjectionCodecV1, ConfigProjectionRefV1,
    DescriptorPinnedArtifactV1, E3ChildSecurityAttestationV1, E3DeniedControlProbeTargetBindingV1,
    E3DeniedControlProbeV1, E3DerivedWorldFsEnforcementPlanV1, E3ElfExecutionModelV1,
    E3IsolatedChildRoleV1, E3LinuxIdMapExtentV1, E3UserNamespaceAttestationV1,
    E3WorldFsEnforcementInputV1, EffectiveEnvironmentV1, GatewayListenerIdentityV1,
    InWorldGatewayRefV1, LinuxArtifactSourceV1, RuntimeArtifactProvenanceV1, SecretHandoffRefV1,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const MAX_INPUT_BYTES: usize = 65_536;
const USERNS_CREATED: u8 = 0x01;
const USERNS_MAPPED: u8 = 0x02;
const FINAL_EXEC_RELEASE: u8 = 0x01;
const NS_GET_NSTYPE: libc::c_ulong = 0xb703;

const CODEX_KEYS: [&str; 11] = [
    "SUBSTRATE_WORLD_ENTRY_ROLE",
    "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD",
    "SUBSTRATE_E3_NATIVE_REALIZATION_FD",
    "SUBSTRATE_E3_NATIVE_SOURCE_FD",
    "SUBSTRATE_E3_SYSTEM_EMPTY_FD",
    "SUBSTRATE_E3_WORLD_FS_INPUT_FD",
    "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
    "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD",
    "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
    "SUBSTRATE_WORLD_ENTRY_USERNS_FD",
    "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD",
];
const GATEWAY_KEYS: [&str; 11] = [
    "SUBSTRATE_WORLD_ENTRY_ROLE",
    "SUBSTRATE_E3_GATEWAY_LAUNCH_FD",
    "SUBSTRATE_E3_GATEWAY_LISTENER_FD",
    "SUBSTRATE_E3_GATEWAY_SECRET_READY_FD",
    "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
    "SUBSTRATE_E3_WORLD_FS_INPUT_FD",
    "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
    "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD",
    "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
    "SUBSTRATE_WORLD_ENTRY_USERNS_FD",
    "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD",
];
const READINESS_KEYS: [&str; 9] = [
    "SUBSTRATE_WORLD_ENTRY_ROLE",
    "SUBSTRATE_E3_READINESS_PROBE_INPUT_FD",
    "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
    "SUBSTRATE_WORLD_ENTRY_USERNS_FD",
    "SUBSTRATE_E3_READINESS_PROBE_START_FD",
    "SUBSTRATE_E3_READINESS_PROBE_CONNECTED_FD",
    "SUBSTRATE_E3_READINESS_PROBE_REQUEST_RELEASE_FD",
    "SUBSTRATE_E3_READINESS_PROBE_RESULT_FD",
    "SUBSTRATE_WORLD_ENTRY_SELF_ARTIFACT_FD",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WrapperRoleV1 {
    Codex,
    ManagedGateway,
    ManagedGatewayReadinessProbe,
}

#[derive(Debug)]
struct LaunchDescriptorsV1 {
    role: WrapperRoleV1,
    descriptors: BTreeMap<String, RawFd>,
}

impl LaunchDescriptorsV1 {
    fn fd(&self, key: &str) -> Result<RawFd> {
        self.descriptors
            .get(key)
            .copied()
            .with_context(|| format!("missing wrapper descriptor {key}"))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct E3CodexLaunchPlanV1 {
    schema_version: u32,
    authority_store_id: String,
    series_id: String,
    fence_id: String,
    closed_projection_ref: ConfigProjectionRefV1,
    projection_identity_hash: String,
    native_projection_hash: String,
    turn: E3CodexLaunchTurnV1,
    codex_argv: Vec<String>,
    final_environment: EffectiveEnvironmentV1,
    required_absent_codex_home_entries: Vec<String>,
    output_last_message: E3CodexOutputFileV1,
    workspace: CanonicalDirectoryV1,
    native_source: CanonicalDirectoryV1,
    native_realization: CanonicalDirectoryV1,
    system_empty: CanonicalDirectoryV1,
    stdin_fd: u32,
    stdout_fd: u32,
    stderr_fd: u32,
    plan_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
enum E3CodexLaunchTurnV1 {
    Initial {
        bootstrap_run_id: String,
    },
    Resume {
        turn_id: String,
        codex_session_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct E3CodexOutputFileV1 {
    relative_path: String,
    guest_absolute_path: String,
    device_id: u64,
    inode: u64,
    mode: u32,
    initial_byte_length: u64,
    initial_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayLaunchInputRefV1 {
    authority_store_id: String,
    launch_input_id: String,
    launch_input_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayReadinessProbeInputV1 {
    schema_version: u32,
    launch_input_ref: ManagedGatewayLaunchInputRefV1,
    gateway_ref: InWorldGatewayRefV1,
    listener_identity: GatewayListenerIdentityV1,
    readiness_nonce: String,
    readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    target_uid: u64,
    target_gid: u64,
    enforcement_input: E3WorldFsEnforcementInputV1,
    connect_deadline_ms: u32,
    request_release_deadline_ms: u32,
    response_deadline_ms: u32,
    maximum_response_bytes: u32,
    input_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayReadinessProbeConnectedV1 {
    schema_version: u32,
    probe_pid: u32,
    probe_pid_start_time_ticks: u64,
    input_hash: String,
    socket_fd: u32,
    socket_inode: u64,
    local_address: String,
    local_port: u16,
    peer_address: String,
    peer_port: u16,
    readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    connected_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayReadinessProbeResultV1 {
    schema_version: u32,
    probe_pid: u32,
    probe_pid_start_time_ticks: u64,
    input_hash: String,
    connected_hash: String,
    child_security_attestation: E3ChildSecurityAttestationV1,
    http_status: u16,
    response_body_base64: String,
    response_body_sha256: String,
    result_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedGatewayReadinessResponseV1 {
    schema_version: u32,
    readiness_nonce: String,
    launch_input_hash: String,
    gateway_ref: InWorldGatewayRefV1,
    config_projection_identity_hash: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    listener_identity: GatewayListenerIdentityV1,
    secret_handoff_prepared_ref: SecretHandoffRefV1,
    secret_handoff_consumed: bool,
    secret_ready_attestation_hash: String,
}

struct PreparedPrivateNamespaceV1 {
    user_namespace: E3UserNamespaceAttestationV1,
    mount_namespace_inode: u64,
    self_proc_fd: OwnedFd,
    pid_start_time_ticks: u64,
    cap_last_cap: u32,
}

struct FixedBuffer<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

struct StrictReadinessBindingsV1<'a> {
    listener: &'a GatewayListenerIdentityV1,
    projection_identity_hash: &'a str,
    gateway_ref: &'a InWorldGatewayRefV1,
    launch_input_hash: &'a str,
    readiness_nonce: &'a str,
    connect_deadline_ms: u32,
    request_release_deadline_ms: u32,
    response_deadline_ms: u32,
}

impl<const N: usize> FixedBuffer<N> {
    const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn push(&mut self, bytes: &[u8]) -> bool {
        let Some(end) = self.len.checked_add(bytes.len()) else {
            return false;
        };
        if end > N {
            return false;
        }
        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
        true
    }

    fn push_u64(&mut self, mut value: u64) -> bool {
        let mut decimal = [0_u8; 20];
        let mut cursor = decimal.len();
        loop {
            cursor -= 1;
            decimal[cursor] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                return self.push(&decimal[cursor..]);
            }
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

fn main() -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("substrate-world-entry accepts no arguments");
    }
    let environment = collect_exact_environment()?;
    let descriptors = parse_launch_descriptors(&environment)?;
    if descriptors.role == WrapperRoleV1::ManagedGatewayReadinessProbe {
        return run_managed_gateway_readiness_probe(&descriptors);
    }

    let input_bytes = read_bounded_eof(descriptors.fd("SUBSTRATE_E3_WORLD_FS_INPUT_FD")?)?;
    let input: E3WorldFsEnforcementInputV1 = decode_exact_canonical(&input_bytes)?;
    validate_enforcement_input_hash(&input)?;
    validate_role_binding(descriptors.role, input.child_role)?;
    validate_pinned_target_artifact_descriptor(
        descriptors.fd("SUBSTRATE_WORLD_ENTRY_BINARY_FD")?,
        &input.executable_artifact,
        match descriptors.role {
            WrapperRoleV1::Codex => ConfigProjectionArtifactRoleV1::Codex0125,
            WrapperRoleV1::ManagedGateway => ConfigProjectionArtifactRoleV1::ManagedGateway,
            WrapperRoleV1::ManagedGatewayReadinessProbe => unreachable!(),
        },
    )?;
    let codex_plan = if descriptors.role == WrapperRoleV1::Codex {
        let bytes = read_bounded_eof(descriptors.fd("SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD")?)?;
        let plan: E3CodexLaunchPlanV1 = decode_exact_canonical(&bytes)?;
        validate_codex_plan(&plan, &input)?;
        Some(plan)
    } else {
        None
    };

    let namespace = prepare_private_child_namespace(&descriptors, &input)?;
    let derived =
        apply_authenticated_world_fs_enforcement(&descriptors, &input, codex_plan.as_ref())?;
    drop_child_privileges_and_caps(&input, &namespace)?;
    install_child_seccomp()?;
    let denied_control_probe_hash = probe_child_control_path_denials_v1(&input)?;
    let security =
        build_child_security_attestation(&input, &namespace, &derived, denied_control_probe_hash)?;
    write_setup_ready_attestation(
        &descriptors,
        &input,
        codex_plan.as_ref(),
        &security,
        &namespace,
    )?;
    await_final_exec(&descriptors)?;
    exec_pinned_child(&descriptors, codex_plan.as_ref())
}

fn collect_exact_environment() -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for (key, value) in std::env::vars_os() {
        let key =
            String::from_utf8(key.into_vec()).context("non-UTF-8 wrapper environment name")?;
        let value = String::from_utf8(value.into_vec())
            .with_context(|| format!("non-UTF-8 wrapper environment value for {key}"))?;
        if values.insert(key, value).is_some() {
            bail!("duplicate wrapper environment name");
        }
    }
    Ok(values)
}

fn parse_launch_descriptors(environment: &BTreeMap<String, String>) -> Result<LaunchDescriptorsV1> {
    let role = match environment
        .get("SUBSTRATE_WORLD_ENTRY_ROLE")
        .map(String::as_str)
    {
        Some("codex") => WrapperRoleV1::Codex,
        Some("managed_gateway") => WrapperRoleV1::ManagedGateway,
        Some("managed_gateway_readiness_probe") => WrapperRoleV1::ManagedGatewayReadinessProbe,
        _ => bail!("missing or invalid SUBSTRATE_WORLD_ENTRY_ROLE"),
    };
    let expected: BTreeSet<&str> = match role {
        WrapperRoleV1::Codex => CODEX_KEYS.into_iter().collect(),
        WrapperRoleV1::ManagedGateway => GATEWAY_KEYS.into_iter().collect(),
        WrapperRoleV1::ManagedGatewayReadinessProbe => READINESS_KEYS.into_iter().collect(),
    };
    if environment
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != expected
    {
        bail!("wrapper environment does not match the exact role ABI");
    }
    let mut descriptors = BTreeMap::new();
    let mut distinct = BTreeSet::new();
    for (key, value) in environment {
        if key == "SUBSTRATE_WORLD_ENTRY_ROLE" {
            continue;
        }
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            bail!("wrapper descriptor {key} is not shortest unsigned decimal");
        }
        let descriptor: RawFd = value
            .parse()
            .with_context(|| format!("wrapper descriptor {key} is out of range"))?;
        if descriptor < 3 || descriptor.to_string() != *value || !distinct.insert(descriptor) {
            bail!("wrapper descriptor {key} is invalid or aliased");
        }
        if unsafe { libc::fcntl(descriptor, libc::F_GETFD) } < 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("wrapper descriptor {key} is not open"));
        }
        descriptors.insert(key.clone(), descriptor);
    }
    Ok(LaunchDescriptorsV1 { role, descriptors })
}

fn prepare_private_child_namespace(
    descriptors: &LaunchDescriptorsV1,
    input: &E3WorldFsEnforcementInputV1,
) -> Result<PreparedPrivateNamespaceV1> {
    validate_current_process_cgroup(&input.expected_process_cgroup)?;
    validate_current_kernel_boot_id(&input.kernel_boot_id)?;
    if unsafe { libc::unshare(libc::CLONE_NEWUSER) } != 0 {
        return Err(std::io::Error::last_os_error()).context("create private E3-D user namespace");
    }
    let setup = descriptors.fd("SUBSTRATE_WORLD_ENTRY_USERNS_FD")?;
    send_namespace_setup_byte(setup, USERNS_CREATED)?;
    receive_namespace_setup_byte(setup, USERNS_MAPPED)?;
    unsafe { libc::close(setup) };

    let pid = std::process::id();
    let current_user_namespace: OwnedFd = File::open(format!("/proc/{pid}/ns/user"))
        .context("open wrapper user namespace")?
        .into();
    if unsafe { libc::ioctl(current_user_namespace.as_raw_fd(), NS_GET_NSTYPE as _) }
        != libc::CLONE_NEWUSER
    {
        bail!("wrapper user namespace has the wrong type");
    }
    let (namespace_device_id, namespace_inode) =
        descriptor_identity(current_user_namespace.as_raw_fd())?;
    if (namespace_device_id, namespace_inode)
        == (
            input.user_namespace_requirement.parent_namespace_device_id,
            input.user_namespace_requirement.parent_namespace_inode,
        )
    {
        bail!("wrapper remained in the service user namespace");
    }
    validate_current_map("uid_map", &input.user_namespace_requirement.uid_map)?;
    validate_current_map("gid_map", &input.user_namespace_requirement.gid_map)?;

    if unsafe { libc::unshare(libc::CLONE_NEWNS) } != 0 {
        return Err(std::io::Error::last_os_error()).context("create private E3-D mount namespace");
    }
    let root = CString::new("/").unwrap();
    if unsafe {
        libc::mount(
            std::ptr::null(),
            root.as_ptr(),
            std::ptr::null(),
            libc::MS_REC | libc::MS_PRIVATE,
            std::ptr::null(),
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error()).context("make E3-D mounts private");
    }
    let mount_namespace: OwnedFd = File::open(format!("/proc/{pid}/ns/mnt"))
        .context("open wrapper mount namespace")?
        .into();
    let (_, mount_namespace_inode) = descriptor_identity(mount_namespace.as_raw_fd())?;
    let self_proc_fd = open_numeric_self_proc_directory(pid)?;
    let mut proc_statfs: libc::statfs = unsafe { zeroed() };
    if unsafe { libc::fstatfs(self_proc_fd.as_raw_fd(), &mut proc_statfs) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat numeric wrapper procfs");
    }
    if proc_statfs.f_type as i128 != libc::PROC_SUPER_MAGIC as i128 {
        bail!("numeric wrapper process directory is not procfs");
    }
    let pid_start_time_ticks = process_start_time_at(self_proc_fd.as_raw_fd())?;
    validate_numeric_self_proc_identity(self_proc_fd.as_raw_fd(), pid, pid_start_time_ticks)?;
    let cap_last_cap = std::fs::read_to_string("/proc/sys/kernel/cap_last_cap")
        .context("read cap_last_cap before E3-D Landlock")?
        .trim()
        .parse::<u32>()
        .context("parse cap_last_cap")?;
    if cap_last_cap > 63 {
        bail!("E3-D does not support capability numbers above 63");
    }
    Ok(PreparedPrivateNamespaceV1 {
        user_namespace: E3UserNamespaceAttestationV1 {
            namespace_device_id,
            namespace_inode,
            owner_uid: input.user_namespace_requirement.trusted_service_uid,
            parent_namespace_device_id: input.user_namespace_requirement.parent_namespace_device_id,
            parent_namespace_inode: input.user_namespace_requirement.parent_namespace_inode,
            uid_map: input.user_namespace_requirement.uid_map.clone(),
            gid_map: input.user_namespace_requirement.gid_map.clone(),
        },
        mount_namespace_inode,
        self_proc_fd,
        pid_start_time_ticks,
        cap_last_cap,
    })
}

fn open_numeric_self_proc_directory(pid: u32) -> Result<OwnedFd> {
    const RESOLVE_NO_XDEV: u64 = 0x01;
    const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
    const RESOLVE_NO_SYMLINKS: u64 = 0x04;
    const RESOLVE_BENEATH: u64 = 0x08;

    let proc = CString::new("/proc").unwrap();
    let proc_fd = unsafe {
        libc::open(
            proc.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if proc_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open trusted procfs root");
    }
    let proc_fd = unsafe { OwnedFd::from_raw_fd(proc_fd) };
    let mut procfs: libc::statfs = unsafe { zeroed() };
    if unsafe { libc::fstatfs(proc_fd.as_raw_fd(), &mut procfs) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat trusted procfs root");
    }
    if procfs.f_type as i128 != libc::PROC_SUPER_MAGIC as i128 {
        bail!("trusted process root is not procfs");
    }
    let component = CString::new(pid.to_string()).unwrap();
    let how = OpenHowV1 {
        flags: (libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            proc_fd.as_raw_fd(),
            component.as_ptr(),
            &how,
            std::mem::size_of::<OpenHowV1>(),
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open numeric wrapper proc directory without links or mount crossing");
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
}

fn validate_numeric_self_proc_identity(
    self_proc_fd: RawFd,
    pid: u32,
    pid_start_time_ticks: u64,
) -> Result<()> {
    let status = read_proc_file_at(self_proc_fd, "status")?;
    let parsed_pid = status_line(&status, "Pid:")?
        .trim()
        .parse::<u32>()
        .context("parse numeric proc Pid")?;
    let namespace_pids = status_line(&status, "NSpid:")?
        .split_ascii_whitespace()
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parse numeric proc NSpid")?;
    if parsed_pid != pid
        || namespace_pids.first() != Some(&pid)
        || namespace_pids.last() != Some(&pid)
        || process_start_time_at(self_proc_fd)? != pid_start_time_ticks
    {
        bail!("numeric wrapper proc identity does not match the running process");
    }
    Ok(())
}

fn apply_authenticated_world_fs_enforcement(
    descriptors: &LaunchDescriptorsV1,
    input: &E3WorldFsEnforcementInputV1,
    codex_plan: Option<&E3CodexLaunchPlanV1>,
) -> Result<E3DerivedWorldFsEnforcementPlanV1> {
    if let Some(plan) = codex_plan {
        bind_codex_mounts(descriptors, plan)?;
    }
    let workspace = if descriptors.role == WrapperRoleV1::Codex {
        Some(descriptor_path(
            descriptors.fd("SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD")?,
        )?)
    } else {
        None
    };
    let snapshot_bytes = BASE64
        .decode(input.policy_snapshot_bytes_base64.as_bytes())
        .context("decode E3-D policy snapshot bytes")?;
    if snapshot_bytes.len() as u64 != input.policy_snapshot_byte_length
        || format!("{:x}", Sha256::digest(&snapshot_bytes)) != input.policy_snapshot_hash
    {
        bail!("E3-D policy snapshot bytes do not match the authenticated input");
    }
    let snapshot: transport_api_types::PolicySnapshotV3 = decode_exact_canonical(&snapshot_bytes)?;
    let snapshot = snapshot.canonicalize().map_err(anyhow::Error::msg)?;
    if snapshot.world_fs.host_visible {
        bail!("E3-D requires full filesystem isolation");
    }
    let (mut e2_discover_paths, mut e2_execute_paths, mut e2_read_paths, mut e2_write_paths) =
        if let Some(workspace) = workspace.as_deref() {
            let read_patterns = snapshot
                .world_fs
                .read
                .as_ref()
                .map(|dimension| dimension.allow_list.as_slice())
                .unwrap_or(&[]);
            let discover_patterns = snapshot
                .world_fs
                .discover
                .as_ref()
                .map(|dimension| dimension.allow_list.as_slice())
                .unwrap_or(read_patterns);
            let read = resolve_project_allowlist(workspace, read_patterns);
            (
                resolve_project_allowlist(workspace, discover_patterns),
                read.clone(),
                read,
                resolve_project_allowlist(workspace, &snapshot.world_fs.write.allow_list),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };
    for paths in [
        &mut e2_discover_paths,
        &mut e2_execute_paths,
        &mut e2_read_paths,
        &mut e2_write_paths,
    ] {
        paths.sort();
        paths.dedup();
    }

    let numeric_proc = format!("/proc/{}", std::process::id());
    let mut support_discover_paths = vec![numeric_proc.clone()];
    let mut support_execute_paths = Vec::new();
    let mut support_read_paths = vec![numeric_proc];
    let mut support_write_paths = Vec::new();
    if descriptors.role != WrapperRoleV1::ManagedGatewayReadinessProbe {
        support_execute_paths.push(input.executable_artifact.configured_absolute_path.clone());
        support_read_paths.extend(
            [
                "/dev/null",
                "/dev/urandom",
                "/etc/hosts",
                "/etc/nsswitch.conf",
                "/etc/passwd",
                "/etc/group",
                "/etc/resolv.conf",
                "/etc/ssl/certs/ca-certificates.crt",
            ]
            .into_iter()
            .map(str::to_string),
        );
        support_read_paths.push(input.executable_artifact.configured_absolute_path.clone());
        support_write_paths.push("/dev/null".to_string());
    }
    if let Some(directory) = &input.private_realization {
        if descriptors.role == WrapperRoleV1::Codex {
            for child in ["home", "codex-home", "state", "tmp"] {
                let path = format!("{}/{child}", directory.physical_path);
                support_read_paths.push(path.clone());
                support_write_paths.push(path);
            }
        } else {
            support_read_paths.push(directory.physical_path.clone());
            support_write_paths.push(directory.physical_path.clone());
        }
    }
    if descriptors.role == WrapperRoleV1::Codex {
        support_read_paths.push("/etc/codex/config.toml".to_string());
    }
    for paths in [
        &mut support_discover_paths,
        &mut support_execute_paths,
        &mut support_read_paths,
        &mut support_write_paths,
    ] {
        paths.sort();
        paths.dedup();
    }

    let mut derived_policy = world::landlock::LandlockFilesystemPolicy {
        exec_paths: e2_execute_paths.clone(),
        discover_paths: e2_discover_paths.clone(),
        read_paths: e2_read_paths.clone(),
        write_paths: e2_write_paths.clone(),
    };
    derived_policy
        .exec_paths
        .extend(support_execute_paths.clone());
    derived_policy
        .discover_paths
        .extend(support_discover_paths.clone());
    derived_policy.read_paths.extend(support_read_paths.clone());
    derived_policy
        .write_paths
        .extend(support_write_paths.clone());
    for paths in [
        &mut derived_policy.exec_paths,
        &mut derived_policy.discover_paths,
        &mut derived_policy.read_paths,
        &mut derived_policy.write_paths,
    ] {
        paths.sort();
        paths.dedup();
    }
    let derived_policy =
        world_service::internal_exec::resolve_authenticated_world_fs_enforcement_plan_v1(
            derived_policy,
        );
    let report = world_service::internal_exec::apply_authenticated_world_fs_enforcement_plan_v1(
        &derived_policy,
    );
    if !report.support.supported || !report.attempted || !report.applied {
        bail!(
            "E3-D Landlock enforcement is unavailable: {:?}",
            report.reason
        );
    }

    let role_policy = world::landlock::LandlockFilesystemPolicy {
        exec_paths: derived_policy.exec_paths.clone(),
        discover_paths: derived_policy.discover_paths.clone(),
        read_paths: derived_policy.read_paths.clone(),
        write_paths: derived_policy.write_paths.clone(),
    };
    let role_policy =
        world_service::internal_exec::resolve_authenticated_world_fs_enforcement_plan_v1(
            role_policy,
        );
    let role_report =
        world_service::internal_exec::apply_authenticated_world_fs_enforcement_plan_v1(
            &role_policy,
        );
    if !role_report.support.supported || !role_report.attempted || !role_report.applied {
        bail!(
            "E3-D role Landlock enforcement is unavailable: {:?}",
            role_report.reason
        );
    }

    let e2_plan_hash = canonical_value_hash(&serde_json::json!({
        "domain": "substrate.e3.e2-enforcement-plan.v1",
        "discover": e2_discover_paths,
        "execute": e2_execute_paths,
        "policy_snapshot_hash": input.policy_snapshot_hash,
        "read": e2_read_paths,
        "write": e2_write_paths,
    }))?;
    let support_hash = canonical_value_hash(&serde_json::json!({
        "domain": "substrate.e3.derived-support-landlock-layer.v1",
        "e2_enforcement_plan_hash": e2_plan_hash,
        "support_discover": support_discover_paths,
        "support_execute": support_execute_paths,
        "support_read": support_read_paths,
        "support_write": support_write_paths,
    }))?;
    let role_hash = canonical_value_hash(&serde_json::json!({
        "child_role": input.child_role,
        "discover": role_policy.discover_paths,
        "domain": "substrate.e3.role-narrowing-landlock-layer.v1",
        "execute": role_policy.exec_paths,
        "read": role_policy.read_paths,
        "write": role_policy.write_paths,
    }))?;
    let effective_hash = canonical_value_hash(&serde_json::json!({
        "derived_support_ruleset_hash": support_hash,
        "domain": "substrate.e3.effective-landlock-intersection.v1",
        "role_narrowing_ruleset_hash": role_hash,
    }))?;
    Ok(E3DerivedWorldFsEnforcementPlanV1 {
        e2_plan_hash,
        e2_discover_paths,
        e2_execute_paths,
        e2_read_paths,
        e2_write_paths,
        e3_support_discover_paths: support_discover_paths,
        e3_support_execute_paths: support_execute_paths,
        e3_support_read_paths: support_read_paths,
        e3_support_write_paths: support_write_paths,
        derived_support_ruleset_hash: support_hash,
        role_narrowing_ruleset_hash: role_hash,
        effective_landlock_hash: effective_hash,
    })
}

fn drop_child_privileges_and_caps(
    input: &E3WorldFsEnforcementInputV1,
    namespace: &PreparedPrivateNamespaceV1,
) -> Result<()> {
    if input.target_uid == 0 || input.target_gid == 0 {
        bail!("E3-D final identity must be nonroot");
    }
    if unsafe { libc::setgroups(0, std::ptr::null()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("clear wrapper supplementary groups");
    }
    let cap_last = namespace.cap_last_cap;
    for capability in 0..=cap_last {
        if unsafe { libc::prctl(libc::PR_CAPBSET_DROP, capability, 0, 0, 0) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("drop wrapper bounding capability");
        }
    }
    let gid = u32::try_from(input.target_gid).context("target gid exceeds Linux gid_t")?;
    let uid = u32::try_from(input.target_uid).context("target uid exceeds Linux uid_t")?;
    if unsafe { libc::setresgid(gid, gid, gid) } != 0 {
        return Err(std::io::Error::last_os_error()).context("descend wrapper gid");
    }
    unsafe { libc::setfsgid(gid) };
    if unsafe { libc::setresuid(uid, uid, uid) } != 0 {
        return Err(std::io::Error::last_os_error()).context("descend wrapper uid");
    }
    unsafe { libc::setfsuid(uid) };
    let mut header = CapabilityHeader {
        version: 0x2008_0522,
        pid: 0,
    };
    let empty = [CapabilityData::default(); 2];
    if unsafe { libc::syscall(libc::SYS_capset, &mut header, empty.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("clear wrapper capability sets");
    }
    if unsafe {
        libc::prctl(
            libc::PR_CAP_AMBIENT,
            libc::PR_CAP_AMBIENT_CLEAR_ALL,
            0,
            0,
            0,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error()).context("clear wrapper ambient capabilities");
    }
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
        return Err(std::io::Error::last_os_error()).context("set wrapper no-new-privileges");
    }
    if input.child_role == E3IsolatedChildRoleV1::Codex {
        let trusted_parent = unsafe { libc::getppid() };
        if trusted_parent <= 1
            || unsafe {
                libc::prctl(
                    libc::PR_SET_PTRACER,
                    trusted_parent as libc::c_ulong,
                    0,
                    0,
                    0,
                )
            } != 0
        {
            return Err(std::io::Error::last_os_error())
                .context("bind the trusted Codex tracer relationship");
        }
        if unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) } == -1 {
            return Err(std::io::Error::last_os_error())
                .context("establish required trusted Codex tracing");
        }
    }
    if unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) } != 0 {
        return Err(std::io::Error::last_os_error()).context("set wrapper nondumpable");
    }
    if input.child_role == E3IsolatedChildRoleV1::ManagedGateway {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) } != 0 {
            return Err(std::io::Error::last_os_error()).context("disable gateway core dumps");
        }
    }
    validate_zero_capability_status(
        namespace.self_proc_fd.as_raw_fd(),
        input.target_uid,
        input.target_gid,
        cap_last,
    )
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CapabilityHeader {
    version: u32,
    pid: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CapabilityData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

fn install_child_seccomp() -> Result<()> {
    #[cfg(not(target_arch = "x86_64"))]
    bail!("E3-D seccomp profile currently admits only x86_64");
    #[cfg(target_arch = "x86_64")]
    {
        const BPF_LD_W_ABS: u16 = 0x20;
        const BPF_JMP_JEQ_K: u16 = 0x15;
        const BPF_RET_K: u16 = 0x06;
        const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
        const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
        const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
        const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;
        let denied: [i64; 19] = [
            libc::SYS_ptrace,
            libc::SYS_process_vm_readv,
            libc::SYS_process_vm_writev,
            libc::SYS_kcmp,
            libc::SYS_pidfd_getfd,
            libc::SYS_bpf,
            libc::SYS_perf_event_open,
            libc::SYS_mount,
            libc::SYS_umount2,
            libc::SYS_pivot_root,
            libc::SYS_move_mount,
            libc::SYS_open_tree,
            libc::SYS_fsopen,
            libc::SYS_fsconfig,
            libc::SYS_fsmount,
            libc::SYS_fspick,
            libc::SYS_mount_setattr,
            libc::SYS_setns,
            libc::SYS_clone3,
        ];
        let mut filter = vec![
            libc::sock_filter {
                code: BPF_LD_W_ABS,
                jt: 0,
                jf: 0,
                k: 4,
            },
            libc::sock_filter {
                code: BPF_JMP_JEQ_K,
                jt: 1,
                jf: 0,
                k: AUDIT_ARCH_X86_64,
            },
            libc::sock_filter {
                code: BPF_RET_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_KILL_PROCESS,
            },
            libc::sock_filter {
                code: BPF_LD_W_ABS,
                jt: 0,
                jf: 0,
                k: 0,
            },
        ];
        for syscall in denied {
            filter.push(libc::sock_filter {
                code: BPF_JMP_JEQ_K,
                jt: 0,
                jf: 1,
                k: syscall as u32,
            });
            filter.push(libc::sock_filter {
                code: BPF_RET_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_ERRNO | libc::EPERM as u32,
            });
        }
        let namespace_clone_flags = (libc::CLONE_NEWCGROUP
            | libc::CLONE_NEWIPC
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWNS
            | libc::CLONE_NEWPID
            | libc::CLONE_NEWTIME
            | libc::CLONE_NEWUSER
            | libc::CLONE_NEWUTS) as u32;
        filter.extend([
            libc::sock_filter {
                code: BPF_JMP_JEQ_K,
                jt: 0,
                jf: 4,
                k: libc::SYS_clone as u32,
            },
            libc::sock_filter {
                code: BPF_LD_W_ABS,
                jt: 0,
                jf: 0,
                k: 16,
            },
            libc::sock_filter {
                code: 0x54,
                jt: 0,
                jf: 0,
                k: namespace_clone_flags,
            },
            libc::sock_filter {
                code: BPF_JMP_JEQ_K,
                jt: 1,
                jf: 0,
                k: 0,
            },
            libc::sock_filter {
                code: BPF_RET_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_ERRNO | libc::EPERM as u32,
            },
            libc::sock_filter {
                code: BPF_LD_W_ABS,
                jt: 0,
                jf: 0,
                k: 0,
            },
        ]);
        filter.push(libc::sock_filter {
            code: BPF_JMP_JEQ_K,
            jt: 0,
            jf: 1,
            k: libc::SYS_unshare as u32,
        });
        filter.push(libc::sock_filter {
            code: BPF_RET_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ERRNO | libc::EPERM as u32,
        });
        filter.push(libc::sock_filter {
            code: BPF_RET_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ALLOW,
        });
        let program = libc::sock_fprog {
            len: u16::try_from(filter.len()).context("seccomp program too large")?,
            filter: filter.as_mut_ptr(),
        };
        if unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program) } != 0 {
            return Err(std::io::Error::last_os_error()).context("install E3-D seccomp filter");
        }
        if unsafe { libc::prctl(libc::PR_GET_SECCOMP, 0, 0, 0, 0) }
            != libc::SECCOMP_MODE_FILTER as i32
        {
            bail!("E3-D seccomp filter readback mismatch");
        }
        Ok(())
    }
}

fn install_readiness_seccomp() -> Result<()> {
    #[cfg(not(target_arch = "x86_64"))]
    bail!("E3-D readiness seccomp profile currently admits only x86_64");
    #[cfg(target_arch = "x86_64")]
    {
        const BPF_LD_W_ABS: u16 = 0x20;
        const BPF_JMP_JEQ_K: u16 = 0x15;
        const BPF_RET_K: u16 = 0x06;
        const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
        const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
        const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;
        fn load(offset: u32) -> libc::sock_filter {
            libc::sock_filter {
                code: BPF_LD_W_ABS,
                jt: 0,
                jf: 0,
                k: offset,
            }
        }
        fn equal(value: u32, jump_true: u8, jump_false: u8) -> libc::sock_filter {
            libc::sock_filter {
                code: BPF_JMP_JEQ_K,
                jt: jump_true,
                jf: jump_false,
                k: value,
            }
        }
        fn action(value: u32) -> libc::sock_filter {
            libc::sock_filter {
                code: BPF_RET_K,
                jt: 0,
                jf: 0,
                k: value,
            }
        }
        let mut filter = vec![
            load(4),
            equal(AUDIT_ARCH_X86_64, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            load(0),
        ];
        for syscall in [
            libc::SYS_read,
            libc::SYS_write,
            libc::SYS_close,
            libc::SYS_connect,
            libc::SYS_ppoll,
            libc::SYS_getsockname,
            libc::SYS_getpeername,
            libc::SYS_fstat,
            libc::SYS_rt_sigreturn,
            libc::SYS_exit,
            libc::SYS_exit_group,
        ] {
            filter.push(equal(syscall as u32, 0, 1));
            filter.push(action(SECCOMP_RET_ALLOW));
        }
        filter.extend([
            equal(libc::SYS_socket as u32, 0, 10),
            load(16),
            equal(libc::AF_INET as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            load(24),
            equal(
                (libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK) as u32,
                1,
                0,
            ),
            action(SECCOMP_RET_KILL_PROCESS),
            load(32),
            equal(libc::IPPROTO_TCP as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            action(SECCOMP_RET_ALLOW),
            equal(libc::SYS_getsockopt as u32, 0, 7),
            load(24),
            equal(libc::SOL_SOCKET as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            load(32),
            equal(libc::SO_ERROR as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            action(SECCOMP_RET_ALLOW),
            equal(libc::SYS_clock_gettime as u32, 0, 4),
            load(16),
            equal(libc::CLOCK_MONOTONIC as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            action(SECCOMP_RET_ALLOW),
            equal(libc::SYS_prctl as u32, 0, 4),
            load(16),
            equal(libc::PR_GET_SECCOMP as u32, 1, 0),
            action(SECCOMP_RET_KILL_PROCESS),
            action(SECCOMP_RET_ALLOW),
            action(SECCOMP_RET_KILL_PROCESS),
        ]);
        let program = libc::sock_fprog {
            len: u16::try_from(filter.len()).context("readiness seccomp program too large")?,
            filter: filter.as_mut_ptr(),
        };
        if unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("install readiness allow-only seccomp filter");
        }
        std::mem::forget(filter);
        if unsafe { libc::prctl(libc::PR_GET_SECCOMP, 0, 0, 0, 0) }
            != libc::SECCOMP_MODE_FILTER as i32
        {
            unsafe { libc::_exit(126) };
        }
        Ok(())
    }
}

fn write_setup_ready_attestation(
    descriptors: &LaunchDescriptorsV1,
    input: &E3WorldFsEnforcementInputV1,
    codex_plan: Option<&E3CodexLaunchPlanV1>,
    security: &E3ChildSecurityAttestationV1,
    namespace: &PreparedPrivateNamespaceV1,
) -> Result<()> {
    let bytes = if let Some(plan) = codex_plan {
        let mut setup = CodexSetupReadyAttestationV1 {
            schema_version: 1,
            series_id: plan.series_id.clone(),
            fence_id: plan.fence_id.clone(),
            wrapper_pid: std::process::id(),
            wrapper_pid_start_time_ticks: namespace.pid_start_time_ticks,
            mount_namespace_inode: namespace.mount_namespace_inode,
            masked_system_directory: plan.system_empty.clone(),
            native_root: plan.native_realization.clone(),
            codex_launch_plan_hash: plan.plan_hash.clone(),
            validated_loader_input_fingerprint: input.projection_identity_hash.clone(),
            child_security: security.clone(),
            attestation_hash: String::new(),
        };
        setup.attestation_hash = hash_omitting(
            "substrate.e3.codex-setup-ready.v1",
            "attestation",
            &setup,
            "attestation_hash",
        )?;
        ConfigProjectionCodecV1::encode_canonical_json(&setup)?
    } else {
        ConfigProjectionCodecV1::encode_canonical_json(security)?
    };
    if bytes.len() > MAX_INPUT_BYTES {
        bail!("setup-ready attestation exceeds its bounded pipe");
    }
    write_all_fd(
        descriptors.fd("SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD")?,
        &bytes,
    )?;
    unsafe { libc::close(descriptors.fd("SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD")?) };
    Ok(())
}

fn await_final_exec(descriptors: &LaunchDescriptorsV1) -> Result<()> {
    receive_exact_pipe_gate(
        descriptors.fd("SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD")?,
        FINAL_EXEC_RELEASE,
    )?;
    unsafe { libc::close(descriptors.fd("SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD")?) };
    Ok(())
}

fn exec_pinned_child(
    descriptors: &LaunchDescriptorsV1,
    codex_plan: Option<&E3CodexLaunchPlanV1>,
) -> Result<()> {
    if descriptors.role == WrapperRoleV1::Codex
        && (unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 1, 0, 0, 0) } != 0
            || unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) } != 1)
    {
        return Err(std::io::Error::last_os_error())
            .context("enable and read back nonsecret Codex dumpability");
    }
    let (argv, environment) = match (descriptors.role, codex_plan) {
        (WrapperRoleV1::Codex, Some(plan)) => (
            plan.codex_argv.clone(),
            plan.final_environment
                .set
                .iter()
                .map(|entry| format!("{}={}", entry.name, entry.value))
                .collect::<Vec<_>>(),
        ),
        (WrapperRoleV1::ManagedGateway, None) => (
            vec!["substrate-gateway".to_string()],
            [
                "SUBSTRATE_E3_GATEWAY_LAUNCH_FD",
                "SUBSTRATE_E3_GATEWAY_LISTENER_FD",
                "SUBSTRATE_E3_GATEWAY_SECRET_READY_FD",
                "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
            ]
            .into_iter()
            .map(|key| Ok(format!("{key}={}", descriptors.fd(key)?)))
            .collect::<Result<Vec<_>>>()?,
        ),
        _ => bail!("wrong final-exec plan for wrapper role"),
    };
    if descriptors.role != WrapperRoleV1::ManagedGatewayReadinessProbe {
        let working = descriptors.fd("SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD")?;
        if unsafe { libc::fchdir(working) } != 0 {
            return Err(std::io::Error::last_os_error()).context("enter pinned final cwd");
        }
    }
    let argv = c_string_vector(argv)?;
    let environment = c_string_vector(environment)?;
    let mut argv_ptrs = argv.iter().map(|value| value.as_ptr()).collect::<Vec<_>>();
    argv_ptrs.push(std::ptr::null());
    let mut env_ptrs = environment
        .iter()
        .map(|value| value.as_ptr())
        .collect::<Vec<_>>();
    env_ptrs.push(std::ptr::null());
    close_wrapper_descriptors_before_final_exec(descriptors)?;
    let empty = CString::new("").unwrap();
    let binary = descriptors.fd("SUBSTRATE_WORLD_ENTRY_BINARY_FD")?;
    let rc = unsafe {
        libc::syscall(
            libc::SYS_execveat,
            binary,
            empty.as_ptr(),
            argv_ptrs.as_ptr(),
            env_ptrs.as_ptr(),
            libc::AT_EMPTY_PATH,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("descriptor-exec pinned E3-D child");
    }
    unreachable!()
}

fn validate_managed_gateway_readiness_probe_input(
    input: &ManagedGatewayReadinessProbeInputV1,
    self_artifact_fd: RawFd,
) -> Result<()> {
    if input.schema_version != 1
        || input.connect_deadline_ms != 1_000
        || input.request_release_deadline_ms != 1_000
        || input.response_deadline_ms != 1_000
        || input.maximum_response_bytes != MAX_INPUT_BYTES as u32
        || input.target_uid == 0
        || input.target_gid == 0
        || input.target_uid != input.enforcement_input.target_uid
        || input.target_gid != input.enforcement_input.target_gid
        || input.readiness_probe_cgroup != input.enforcement_input.expected_process_cgroup
        || input.enforcement_input.child_role != E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe
        || input.listener_identity.transport != "tcp"
        || input.listener_identity.address != "127.0.0.1"
        || input.listener_identity.port == 0
        || input.listener_identity.socket_inode == 0
        || input.listener_identity.listen_backlog == 0
        || !input
            .listener_identity
            .deny_boundary_effective_before_listen
        || input.listener_identity.responses_base_path != "/v1/responses"
        || input.launch_input_ref.authority_store_id != input.gateway_ref.authority_store_id
        || input.launch_input_ref.launch_input_hash.len() != 64
        || input.launch_input_ref.launch_input_id.is_empty()
    {
        bail!("malformed managed-gateway readiness-probe input");
    }
    if !is_lower_hex_sha256(&input.launch_input_ref.launch_input_hash)
        || input.gateway_ref.validate().is_err()
    {
        bail!("invalid managed-gateway readiness authority reference");
    }
    let nonce = uuid::Uuid::parse_str(&input.readiness_nonce)
        .context("parse managed-gateway readiness nonce")?;
    if nonce.get_version_num() != 7 || nonce.to_string() != input.readiness_nonce {
        bail!("managed-gateway readiness nonce is not canonical UUIDv7");
    }
    let expected_hash = hash_omitting(
        "substrate.e3.managed-gateway-readiness-probe-input.v1",
        "input",
        input,
        "input_hash",
    )?;
    if input.input_hash != expected_hash {
        bail!("managed-gateway readiness-probe input hash mismatch");
    }
    validate_enforcement_input_hash(&input.enforcement_input)?;
    validate_current_process_cgroup(&input.readiness_probe_cgroup)?;
    validate_current_kernel_boot_id(&input.enforcement_input.kernel_boot_id)?;
    let pid = std::process::id();
    let network_namespace = File::open(format!("/proc/{pid}/ns/net"))
        .context("open numeric readiness network namespace")?;
    if descriptor_identity(network_namespace.as_raw_fd())?.1
        != input.listener_identity.network_namespace_inode
    {
        bail!("readiness listener network namespace identity mismatch");
    }
    validate_self_artifact_descriptor(
        self_artifact_fd,
        &input.enforcement_input.executable_artifact,
    )
}

fn validate_self_artifact_descriptor(
    fd: RawFd,
    artifact: &DescriptorPinnedArtifactV1,
) -> Result<()> {
    if artifact.role != ConfigProjectionArtifactRoleV1::WorldEntryWrapper
        || artifact.configured_absolute_path != "/usr/local/lib/substrate/e3/substrate-world-entry"
        || artifact.file_type != "regular"
        || artifact.mode != 0o755
        || artifact.owner_uid != 0
        || artifact.runtime_support.schema_version != 1
        || artifact.runtime_support.support_policy_version != 1
        || artifact.runtime_support.elf_interpreter.is_some()
        || artifact.runtime_support.dynamic_loader_cache.is_some()
        || !artifact.runtime_support.ordered_elf_dependencies.is_empty()
        || !matches!(
            artifact.runtime_support.elf_execution_model,
            E3ElfExecutionModelV1::StaticExec | E3ElfExecutionModelV1::StaticPie(_)
        )
    {
        bail!("readiness probe wrapper artifact is not exact static support-policy V1");
    }
    match &artifact.provenance {
        RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
            component,
            target_triple,
            profile,
            executable_sha256,
            ..
        } if component == "substrate-world-entry"
            && target_triple == "x86_64-unknown-linux-musl"
            && profile == "release"
            && executable_sha256 == &artifact.sha256 => {}
        _ => bail!("readiness probe wrapper artifact provenance mismatch"),
    }
    validate_runtime_support_manifest(artifact)?;
    validate_artifact_descriptor_bytes(fd, artifact)?;
    let pid = std::process::id();
    let executable = File::open(format!("/proc/{pid}/exe"))
        .context("open numeric readiness wrapper executable")?;
    if descriptor_identity(executable.as_raw_fd())? != (artifact.device_id, artifact.inode) {
        bail!("readiness self-artifact does not identify the executing wrapper");
    }
    unsafe { libc::close(fd) };
    Ok(())
}

fn validate_pinned_target_artifact_descriptor(
    fd: RawFd,
    artifact: &DescriptorPinnedArtifactV1,
    expected_role: ConfigProjectionArtifactRoleV1,
) -> Result<()> {
    if artifact.role != expected_role
        || !artifact.configured_absolute_path.starts_with('/')
        || artifact.configured_absolute_path.contains('\0')
    {
        bail!("pinned E3-D target artifact role or path mismatch");
    }
    match (&artifact.role, &artifact.provenance) {
        (
            ConfigProjectionArtifactRoleV1::Codex0125,
            RuntimeArtifactProvenanceV1::OfficialCodexRelease {
                version,
                target_triple,
                extracted_executable_sha256,
                ..
            },
        ) if version == "0.125.0"
            && target_triple == "x86_64-unknown-linux-musl"
            && extracted_executable_sha256 == &artifact.sha256 => {}
        (
            ConfigProjectionArtifactRoleV1::ManagedGateway,
            RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                component,
                target_triple,
                profile,
                executable_sha256,
                ..
            },
        ) if component == "substrate-gateway"
            && target_triple == "x86_64-unknown-linux-musl"
            && profile == "release"
            && executable_sha256 == &artifact.sha256 => {}
        _ => bail!("pinned E3-D target artifact provenance mismatch"),
    }
    validate_runtime_support_manifest(artifact)?;
    validate_artifact_descriptor_bytes(fd, artifact)
}

fn validate_artifact_descriptor_bytes(
    fd: RawFd,
    artifact: &DescriptorPinnedArtifactV1,
) -> Result<()> {
    let installed = open_absolute_beneath_root_no_symlinks(
        &artifact.configured_absolute_path,
        libc::O_RDONLY,
        false,
    )
    .context("open configured E3-D artifact path")?;
    if descriptor_identity(installed.as_raw_fd())? != (artifact.device_id, artifact.inode)
        || descriptor_identity(installed.as_raw_fd())? != descriptor_identity(fd)?
    {
        bail!("configured E3-D artifact path no longer names the held descriptor");
    }
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error()).context("duplicate pinned E3-D artifact");
    }
    let duplicate = unsafe { File::from_raw_fd(duplicate) };
    if LinuxArtifactSourceV1::validate_e3_static_elf_v1(&duplicate)?
        != artifact.runtime_support.elf_execution_model
    {
        bail!("pinned E3-D artifact static ELF model mismatch");
    }
    let mut stat: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat readiness self-artifact");
    }
    if stat.st_mode & libc::S_IFMT != libc::S_IFREG
        || stat.st_mode & 0o7777 != artifact.mode
        || stat.st_mode & (libc::S_ISUID | libc::S_ISGID) != 0
        || stat.st_uid as u64 != artifact.owner_uid
        || stat.st_dev != artifact.device_id
        || stat.st_ino != artifact.inode
        || stat.st_size < 0
        || stat.st_size as u64 != artifact.byte_length
    {
        bail!("pinned E3-D artifact descriptor metadata mismatch");
    }
    let capability_name = b"security.capability\0";
    let capability_length =
        unsafe { libc::fgetxattr(fd, capability_name.as_ptr().cast(), std::ptr::null_mut(), 0) };
    if capability_length >= 0 {
        bail!("pinned E3-D artifact must not carry file capabilities");
    }
    let capability_errno = last_errno()?;
    if capability_errno != libc::ENODATA
        && capability_errno != libc::ENOTSUP
        && capability_errno != libc::EOPNOTSUPP
    {
        return Err(std::io::Error::from_raw_os_error(capability_errno))
            .context("read pinned E3-D artifact file capabilities");
    }
    let mut digest = Sha256::new();
    let mut offset = 0i64;
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let count = unsafe {
            libc::pread(
                fd,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                offset as libc::off_t,
            )
        };
        if count < 0 {
            return Err(std::io::Error::last_os_error()).context("hash pinned E3-D artifact");
        }
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count as usize]);
        offset = offset
            .checked_add(count as i64)
            .context("pinned E3-D artifact byte length overflow")?;
    }
    if offset as u64 != artifact.byte_length
        || format!("{:x}", digest.finalize()) != artifact.sha256
    {
        bail!("pinned E3-D artifact bytes mismatch");
    }
    Ok(())
}

fn validate_runtime_support_manifest(artifact: &DescriptorPinnedArtifactV1) -> Result<()> {
    let support = &artifact.runtime_support;
    if artifact.file_type != "regular"
        || artifact.mode != 0o755
        || artifact.owner_uid != 0
        || !is_lower_hex_sha256(&artifact.sha256)
        || support.schema_version != 1
        || support.support_policy_version != 1
        || support.elf_interpreter.is_some()
        || support.dynamic_loader_cache.is_some()
        || !support.ordered_elf_dependencies.is_empty()
        || !matches!(
            support.elf_execution_model,
            E3ElfExecutionModelV1::StaticExec | E3ElfExecutionModelV1::StaticPie(_)
        )
    {
        bail!("pinned E3-D artifact runtime support manifest is malformed");
    }
    validate_fixed_support_device("/dev/null", 1, 3)?;
    validate_fixed_support_device("/dev/urandom", 1, 9)?;
    let expected_paths = [
        "/etc/hosts",
        "/etc/nsswitch.conf",
        "/etc/passwd",
        "/etc/group",
        "/etc/resolv.conf",
        "/etc/ssl/certs/ca-certificates.crt",
    ];
    if support.ordered_present_common_files.len() != expected_paths.len() {
        bail!("pinned E3-D artifact common support closure is incomplete");
    }
    for (expected_path, expected) in expected_paths
        .into_iter()
        .zip(&support.ordered_present_common_files)
    {
        if expected.absolute_path != expected_path || !is_lower_hex_sha256(&expected.sha256) {
            bail!("pinned E3-D artifact common support path mismatch");
        }
        let file = open_absolute_beneath_root_no_symlinks(expected_path, libc::O_RDONLY, false)
            .with_context(|| format!("open E3-D support file {expected_path}"))?;
        let mut metadata: libc::stat = unsafe { zeroed() };
        if unsafe { libc::fstat(file.as_raw_fd(), &mut metadata) } != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("stat E3-D support file {expected_path}"));
        }
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG
            || metadata.st_nlink != 1
            || metadata.st_dev != expected.device_id
            || metadata.st_ino != expected.inode
            || metadata.st_mode & 0o7777 != expected.mode
            || metadata.st_size < 0
            || metadata.st_size as u64 != expected.byte_length
            || hash_descriptor(file.as_raw_fd())? != expected.sha256
        {
            bail!("pinned E3-D artifact common support file drifted");
        }
    }
    let system = &support.system_config_mount_target;
    let system_directory =
        open_absolute_beneath_root_no_symlinks("/etc/codex", libc::O_RDONLY, true)
            .context("open E3-D system configuration mount target")?;
    let mut system_metadata: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(system_directory.as_raw_fd(), &mut system_metadata) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("stat E3-D system configuration mount target");
    }
    let enumeration =
        unsafe { libc::fcntl(system_directory.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
    if enumeration < 0 {
        return Err(std::io::Error::last_os_error())
            .context("duplicate E3-D system configuration directory");
    }
    let stream = unsafe { libc::fdopendir(enumeration) };
    if stream.is_null() {
        unsafe { libc::close(enumeration) };
        return Err(std::io::Error::last_os_error())
            .context("enumerate E3-D system configuration directory");
    }
    let mut entry_names = Vec::new();
    loop {
        let entry = unsafe { libc::readdir(stream) };
        if entry.is_null() {
            break;
        }
        let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
        if name != b"." && name != b".." {
            entry_names.push(name.to_vec());
        }
    }
    if unsafe { libc::closedir(stream) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("close E3-D system configuration enumeration");
    }
    entry_names.sort();
    if system.absolute_path != "/etc/codex"
        || system_metadata.st_mode & libc::S_IFMT != libc::S_IFDIR
        || system_metadata.st_dev != system.device_id
        || system_metadata.st_ino != system.inode
        || system_metadata.st_mode & 0o7777 != system.mode
        || u64::from(system_metadata.st_uid) != system.owner_uid
        || u64::from(system_metadata.st_gid) != system.owner_gid
        || !system.ordered_entry_names.is_empty()
        || !entry_names.is_empty()
    {
        bail!("E3-D system configuration mount target drifted");
    }
    let expected_hash = hash_omitting(
        "substrate.e3.runtime-support-manifest.v1",
        "manifest",
        support,
        "manifest_hash",
    )?;
    if support.manifest_hash != expected_hash {
        bail!("pinned E3-D runtime support manifest hash mismatch");
    }
    Ok(())
}

fn validate_fixed_support_device(
    path: &str,
    expected_major: u32,
    expected_minor: u32,
) -> Result<()> {
    let component = path
        .strip_prefix("/dev/")
        .filter(|component| !component.is_empty() && !component.contains('/'))
        .context("E3-D support device path is not a fixed /dev component")?;
    let root = CString::new("/dev").unwrap();
    let root_fd = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if root_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open fixed E3-D device root");
    }
    let root_fd = unsafe { OwnedFd::from_raw_fd(root_fd) };
    let component = CString::new(component).unwrap();
    let how = OpenHowV1 {
        flags: (libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
        mode: 0,
        resolve: 0x01 | 0x02 | 0x04 | 0x08,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root_fd.as_raw_fd(),
            component.as_ptr(),
            &how,
            std::mem::size_of::<OpenHowV1>(),
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open fixed E3-D support device");
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd as RawFd) };
    let mut metadata: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(fd.as_raw_fd(), &mut metadata) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat fixed E3-D support device");
    }
    if metadata.st_mode & libc::S_IFMT != libc::S_IFCHR
        || libc::major(metadata.st_rdev) != expected_major
        || libc::minor(metadata.st_rdev) != expected_minor
    {
        bail!("fixed E3-D support device identity mismatch");
    }
    Ok(())
}

#[repr(C)]
struct OpenHowV1 {
    flags: u64,
    mode: u64,
    resolve: u64,
}

fn open_absolute_beneath_root_no_symlinks(
    absolute_path: &str,
    flags: libc::c_int,
    directory: bool,
) -> Result<OwnedFd> {
    const RESOLVE_NO_XDEV: u64 = 0x01;
    const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
    const RESOLVE_NO_SYMLINKS: u64 = 0x04;
    const RESOLVE_BENEATH: u64 = 0x08;

    let relative = absolute_path
        .strip_prefix('/')
        .filter(|path| {
            !path.is_empty()
                && path
                    .split('/')
                    .all(|component| !component.is_empty() && component != "." && component != "..")
        })
        .context("E3-D support path is not canonical absolute form")?;
    let root = CString::new("/").unwrap();
    let root_fd = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if root_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open E3-D filesystem root");
    }
    let root_fd = unsafe { OwnedFd::from_raw_fd(root_fd) };
    let relative = CString::new(relative).context("E3-D support path contains NUL")?;
    let how = OpenHowV1 {
        flags: u64::try_from(
            flags
                | libc::O_CLOEXEC
                | libc::O_NOFOLLOW
                | if directory { libc::O_DIRECTORY } else { 0 },
        )
        .context("E3-D support open flags are invalid")?,
        mode: 0,
        resolve: RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root_fd.as_raw_fd(),
            relative.as_ptr(),
            &how,
            std::mem::size_of::<OpenHowV1>(),
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open {absolute_path} beneath the held root"));
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd as RawFd) })
}

fn hash_descriptor(fd: RawFd) -> Result<String> {
    let mut digest = Sha256::new();
    let mut offset = 0i64;
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let count = unsafe {
            libc::pread(
                fd,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                offset as libc::off_t,
            )
        };
        if count < 0 {
            return Err(std::io::Error::last_os_error()).context("hash E3-D support descriptor");
        }
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count as usize]);
        offset = offset
            .checked_add(count as i64)
            .context("E3-D support byte length overflow")?;
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn validate_current_kernel_boot_id(expected: &str) -> Result<()> {
    let observed = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .context("read readiness kernel boot ID")?;
    if observed.trim() != expected {
        bail!("readiness kernel boot identity drifted");
    }
    Ok(())
}

fn validate_current_process_cgroup(expected: &CanonicalCgroupIdentityV1) -> Result<()> {
    if expected.cgroup_relative_path.is_empty()
        || expected.cgroup_relative_path.starts_with('/')
        || expected
            .cgroup_relative_path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        bail!("readiness cgroup path is not canonical relative form");
    }
    let mut mount_stat: libc::stat = unsafe { zeroed() };
    let mount = CString::new("/sys/fs/cgroup").unwrap();
    if unsafe { libc::stat(mount.as_ptr(), &mut mount_stat) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat cgroup v2 mount");
    }
    let mut mount_fs: libc::statfs = unsafe { zeroed() };
    if unsafe { libc::statfs(mount.as_ptr(), &mut mount_fs) } != 0 {
        return Err(std::io::Error::last_os_error()).context("statfs cgroup v2 mount");
    }
    if mount_fs.f_type as i128 != 0x6367_7270i128
        || mount_stat.st_dev != expected.cgroup_v2_mount_device_id
        || mount_stat.st_ino != expected.cgroup_v2_mount_inode
    {
        bail!("readiness cgroup v2 mount identity mismatch");
    }
    let directory = format!("/sys/fs/cgroup/{}", expected.cgroup_relative_path);
    let directory = CString::new(directory).context("readiness cgroup path contains NUL")?;
    let mut directory_stat: libc::stat = unsafe { zeroed() };
    if unsafe { libc::stat(directory.as_ptr(), &mut directory_stat) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat readiness cgroup");
    }
    if directory_stat.st_dev != expected.cgroup_v2_mount_device_id
        || directory_stat.st_ino != expected.cgroup_directory_inode
    {
        bail!("readiness cgroup directory identity mismatch");
    }
    let pid = std::process::id();
    let membership = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .context("read numeric readiness cgroup membership")?;
    let expected_line = format!("0::/{}\n", expected.cgroup_relative_path);
    if membership != expected_line {
        bail!("readiness process is not in the exact expected cgroup");
    }
    Ok(())
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn run_managed_gateway_readiness_probe(descriptors: &LaunchDescriptorsV1) -> Result<()> {
    let bytes = read_bounded_eof(descriptors.fd("SUBSTRATE_E3_READINESS_PROBE_INPUT_FD")?)?;
    let probe: ManagedGatewayReadinessProbeInputV1 = decode_exact_canonical(&bytes)?;
    validate_managed_gateway_readiness_probe_input(
        &probe,
        descriptors.fd("SUBSTRATE_WORLD_ENTRY_SELF_ARTIFACT_FD")?,
    )?;
    let input = &probe.enforcement_input;
    validate_role_binding(descriptors.role, input.child_role)?;
    let namespace = prepare_private_child_namespace(descriptors, input)?;
    let derived = apply_authenticated_world_fs_enforcement(descriptors, input, None)?;
    drop_child_privileges_and_caps(input, &namespace)?;
    install_child_seccomp()?;
    let denied_control_probe_hash = probe_child_control_path_denials_v1(input)?;
    let security =
        build_child_security_attestation(input, &namespace, &derived, denied_control_probe_hash)?;
    let probe_pid = std::process::id();
    let request = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Substrate-E3-Readiness-Nonce: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
        probe.listener_identity.port, probe.readiness_nonce
    )
    .into_bytes();
    let setup_ready = ConfigProjectionCodecV1::encode_canonical_json(&security)?;
    let input_hash_json = ConfigProjectionCodecV1::encode_canonical_json(&probe.input_hash)?;
    let cgroup_json =
        ConfigProjectionCodecV1::encode_canonical_json(&probe.readiness_probe_cgroup)?;
    if setup_ready.len() > MAX_INPUT_BYTES
        || input_hash_json.len() != 66
        || cgroup_json.len() > MAX_INPUT_BYTES
    {
        bail!("readiness prebuilt output exceeds its fixed buffer");
    }
    let strict_fds = [
        descriptors.fd("SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD")?,
        descriptors.fd("SUBSTRATE_E3_READINESS_PROBE_START_FD")?,
        descriptors.fd("SUBSTRATE_E3_READINESS_PROBE_CONNECTED_FD")?,
        descriptors.fd("SUBSTRATE_E3_READINESS_PROBE_REQUEST_RELEASE_FD")?,
        descriptors.fd("SUBSTRATE_E3_READINESS_PROBE_RESULT_FD")?,
    ];
    let mut connected_bytes = FixedBuffer::<MAX_INPUT_BYTES>::new();
    let mut result_bytes = FixedBuffer::<MAX_INPUT_BYTES>::new();
    let mut hash_preimage = FixedBuffer::<MAX_INPUT_BYTES>::new();
    let mut response_bytes = [0_u8; MAX_INPUT_BYTES];
    let mut response_body_base64 = [0_u8; MAX_INPUT_BYTES];
    let strict_bindings = StrictReadinessBindingsV1 {
        listener: &probe.listener_identity,
        projection_identity_hash: &probe.enforcement_input.projection_identity_hash,
        gateway_ref: &probe.gateway_ref,
        launch_input_hash: &probe.launch_input_ref.launch_input_hash,
        readiness_nonce: &probe.readiness_nonce,
        connect_deadline_ms: probe.connect_deadline_ms,
        request_release_deadline_ms: probe.request_release_deadline_ms,
        response_deadline_ms: probe.response_deadline_ms,
    };
    install_readiness_seccomp()?;
    run_readiness_probe_after_allow_only_filter(
        strict_fds,
        &strict_bindings,
        probe_pid,
        namespace.pid_start_time_ticks,
        &request,
        &setup_ready,
        &input_hash_json,
        &cgroup_json,
        &mut connected_bytes,
        &mut result_bytes,
        &mut hash_preimage,
        &mut response_bytes,
        &mut response_body_base64,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_readiness_probe_after_allow_only_filter(
    fds: [RawFd; 5],
    bindings: &StrictReadinessBindingsV1<'_>,
    probe_pid: u32,
    probe_start: u64,
    request: &[u8],
    setup_ready: &[u8],
    input_hash_json: &[u8],
    cgroup_json: &[u8],
    connected_bytes: &mut FixedBuffer<MAX_INPUT_BYTES>,
    result_bytes: &mut FixedBuffer<MAX_INPUT_BYTES>,
    hash_preimage: &mut FixedBuffer<MAX_INPUT_BYTES>,
    response_bytes: &mut [u8; MAX_INPUT_BYTES],
    response_body_base64: &mut [u8; MAX_INPUT_BYTES],
) -> ! {
    let [setup_fd, start_fd, connected_fd, request_release_fd, result_fd] = fds;
    if !strict_write_all(setup_fd, setup_ready) || unsafe { libc::close(setup_fd) } != 0 {
        strict_exit_failure();
    }
    if !strict_read_exact_gate(start_fd, 0x01) || unsafe { libc::close(start_fd) } != 0 {
        strict_exit_failure();
    }
    let Some(connect_deadline) = strict_deadline_after(bindings.connect_deadline_ms) else {
        strict_exit_failure();
    };
    let socket = strict_connect_once(bindings.listener, connect_deadline);
    if socket < 0 {
        strict_exit_failure();
    }
    let Some((local_port, socket_inode)) =
        strict_validate_connected_socket(socket, bindings.listener)
    else {
        strict_exit_failure();
    };
    let socket_fd = socket as u32;

    hash_preimage.clear();
    if !hash_preimage.push(b"{\"connected\":{")
        || !append_connected_fields(
            hash_preimage,
            None,
            input_hash_json,
            local_port,
            bindings.listener.port,
            probe_pid,
            probe_start,
            cgroup_json,
            socket_fd,
            socket_inode,
        )
        || !hash_preimage
            .push(b"},\"domain\":\"substrate.e3.managed-gateway-readiness-probe-connected.v1\"}")
    {
        strict_exit_failure();
    }
    let connected_hash = sha256_lower_hex(hash_preimage.as_slice());
    connected_bytes.clear();
    if !connected_bytes.push(b"{")
        || !append_connected_fields(
            connected_bytes,
            Some(&connected_hash),
            input_hash_json,
            local_port,
            bindings.listener.port,
            probe_pid,
            probe_start,
            cgroup_json,
            socket_fd,
            socket_inode,
        )
        || !connected_bytes.push(b"}")
        || !strict_write_all(connected_fd, connected_bytes.as_slice())
        || unsafe { libc::close(connected_fd) } != 0
    {
        strict_exit_failure();
    }

    let Some(request_release_deadline) =
        strict_deadline_after(bindings.request_release_deadline_ms)
    else {
        strict_exit_failure();
    };
    if !strict_read_exact_gate_before(request_release_fd, 0x01, request_release_deadline)
        || unsafe { libc::close(request_release_fd) } != 0
    {
        strict_exit_failure();
    }

    let Some(response_deadline) = strict_deadline_after(bindings.response_deadline_ms) else {
        strict_exit_failure();
    };
    if !strict_write_socket_before(socket, request, response_deadline) {
        strict_exit_failure();
    }
    let Some(response_length) =
        strict_read_socket_before(socket, response_bytes, response_deadline)
    else {
        strict_exit_failure();
    };
    let Some(body) = strict_readiness_body(&response_bytes[..response_length]) else {
        strict_exit_failure();
    };
    if !strict_validate_readiness_response(bindings, body) {
        strict_exit_failure();
    }
    let Ok(base64_length) = BASE64.encode_slice(body, response_body_base64) else {
        strict_exit_failure();
    };
    let body_base64 = &response_body_base64[..base64_length];
    let body_sha256 = sha256_lower_hex(body);

    hash_preimage.clear();
    if !hash_preimage.push(
        b"{\"domain\":\"substrate.e3.managed-gateway-readiness-probe-result.v1\",\"result\":{",
    ) || !append_result_fields(
        hash_preimage,
        None,
        setup_ready,
        &connected_hash,
        input_hash_json,
        probe_pid,
        probe_start,
        body_base64,
        &body_sha256,
    ) || !hash_preimage.push(b"}}")
    {
        strict_exit_failure();
    }
    let result_hash = sha256_lower_hex(hash_preimage.as_slice());
    result_bytes.clear();
    if !result_bytes.push(b"{")
        || !append_result_fields(
            result_bytes,
            Some(&result_hash),
            setup_ready,
            &connected_hash,
            input_hash_json,
            probe_pid,
            probe_start,
            body_base64,
            &body_sha256,
        )
        || !result_bytes.push(b"}")
        || unsafe { libc::close(socket) } != 0
        || !strict_write_all(result_fd, result_bytes.as_slice())
        || unsafe { libc::close(result_fd) } != 0
    {
        strict_exit_failure();
    }
    unsafe { libc::_exit(0) }
}

#[allow(clippy::too_many_arguments)]
fn append_connected_fields<const N: usize>(
    output: &mut FixedBuffer<N>,
    connected_hash: Option<&[u8; 64]>,
    input_hash_json: &[u8],
    local_port: u16,
    peer_port: u16,
    probe_pid: u32,
    probe_start: u64,
    cgroup_json: &[u8],
    socket_fd: u32,
    socket_inode: u64,
) -> bool {
    if let Some(hash) = connected_hash {
        if !output.push(b"\"connected_hash\":\"") || !output.push(hash) || !output.push(b"\",") {
            return false;
        }
    }
    output.push(b"\"input_hash\":")
        && output.push(input_hash_json)
        && output.push(b",\"local_address\":\"127.0.0.1\",\"local_port\":")
        && output.push_u64(u64::from(local_port))
        && output.push(b",\"peer_address\":\"127.0.0.1\",\"peer_port\":")
        && output.push_u64(u64::from(peer_port))
        && output.push(b",\"probe_pid\":")
        && output.push_u64(u64::from(probe_pid))
        && output.push(b",\"probe_pid_start_time_ticks\":")
        && output.push_u64(probe_start)
        && output.push(b",\"readiness_probe_cgroup\":")
        && output.push(cgroup_json)
        && output.push(b",\"schema_version\":1,\"socket_fd\":")
        && output.push_u64(u64::from(socket_fd))
        && output.push(b",\"socket_inode\":")
        && output.push_u64(socket_inode)
}

#[allow(clippy::too_many_arguments)]
fn append_result_fields<const N: usize>(
    output: &mut FixedBuffer<N>,
    result_hash: Option<&[u8; 64]>,
    child_security: &[u8],
    connected_hash: &[u8; 64],
    input_hash_json: &[u8],
    probe_pid: u32,
    probe_start: u64,
    response_body_base64: &[u8],
    response_body_sha256: &[u8; 64],
) -> bool {
    if !output.push(b"\"child_security_attestation\":")
        || !output.push(child_security)
        || !output.push(b",\"connected_hash\":\"")
        || !output.push(connected_hash)
        || !output.push(b"\",\"http_status\":200,\"input_hash\":")
        || !output.push(input_hash_json)
        || !output.push(b",\"probe_pid\":")
        || !output.push_u64(u64::from(probe_pid))
        || !output.push(b",\"probe_pid_start_time_ticks\":")
        || !output.push_u64(probe_start)
        || !output.push(b",\"response_body_base64\":\"")
        || !output.push(response_body_base64)
        || !output.push(b"\",\"response_body_sha256\":\"")
        || !output.push(response_body_sha256)
        || !output.push(b"\",")
    {
        return false;
    }
    if let Some(hash) = result_hash {
        if !output.push(b"\"result_hash\":\"") || !output.push(hash) || !output.push(b"\",") {
            return false;
        }
    }
    output.push(b"\"schema_version\":1")
}

fn sha256_lower_hex(bytes: &[u8]) -> [u8; 64] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = [0_u8; 64];
    for (index, byte) in digest.iter().copied().enumerate() {
        output[index * 2] = HEX[(byte >> 4) as usize];
        output[index * 2 + 1] = HEX[(byte & 0x0f) as usize];
    }
    output
}

fn strict_exit_failure() -> ! {
    unsafe { libc::_exit(127) }
}

fn strict_errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

fn strict_write_all(fd: RawFd, mut bytes: &[u8]) -> bool {
    while !bytes.is_empty() {
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written > 0 {
            bytes = &bytes[written as usize..];
        } else if written < 0 && strict_errno() == libc::EINTR {
            continue;
        } else {
            return false;
        }
    }
    true
}

fn strict_read_exact_gate(fd: RawFd, expected: u8) -> bool {
    let mut bytes = [0_u8; 2];
    let mut length = 0usize;
    loop {
        let read = unsafe {
            libc::read(
                fd,
                bytes[length..].as_mut_ptr().cast(),
                bytes.len() - length,
            )
        };
        if read == 0 {
            return length == 1 && bytes[0] == expected;
        }
        if read > 0 {
            length += read as usize;
            if length == bytes.len() {
                return false;
            }
        } else if strict_errno() != libc::EINTR {
            return false;
        }
    }
}

fn strict_monotonic_ns() -> Option<u64> {
    let mut now: libc::timespec = unsafe { zeroed() };
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) } != 0
        || now.tv_sec < 0
        || now.tv_nsec < 0
        || now.tv_nsec >= 1_000_000_000
    {
        return None;
    }
    u64::try_from(now.tv_sec)
        .ok()?
        .checked_mul(1_000_000_000)?
        .checked_add(now.tv_nsec as u64)
}

fn strict_deadline_after(milliseconds: u32) -> Option<u64> {
    strict_monotonic_ns()?.checked_add(u64::from(milliseconds).checked_mul(1_000_000)?)
}

fn strict_poll_before(fd: RawFd, events: i16, deadline_ns: u64) -> bool {
    loop {
        let Some(now) = strict_monotonic_ns() else {
            return false;
        };
        let Some(remaining) = deadline_ns.checked_sub(now) else {
            return false;
        };
        let timeout = libc::timespec {
            tv_sec: (remaining / 1_000_000_000).try_into().unwrap_or(i64::MAX),
            tv_nsec: (remaining % 1_000_000_000) as libc::c_long,
        };
        let mut poll = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let rc = unsafe { libc::ppoll(&mut poll, 1, &timeout, std::ptr::null()) };
        if rc > 0 {
            if poll.revents & libc::POLLNVAL != 0 {
                return false;
            }
            return poll.revents & events != 0 || poll.revents & libc::POLLHUP != 0;
        }
        if rc == 0 || strict_errno() != libc::EINTR {
            return false;
        }
    }
}

fn strict_read_exact_gate_before(fd: RawFd, expected: u8, deadline_ns: u64) -> bool {
    let mut bytes = [0_u8; 2];
    let mut length = 0usize;
    loop {
        if !strict_poll_before(fd, libc::POLLIN, deadline_ns) {
            return false;
        }
        let read = unsafe {
            libc::read(
                fd,
                bytes[length..].as_mut_ptr().cast(),
                bytes.len() - length,
            )
        };
        if read == 0 {
            return length == 1 && bytes[0] == expected;
        }
        if read > 0 {
            length += read as usize;
            if length == bytes.len() {
                return false;
            }
        } else if strict_errno() != libc::EINTR {
            return false;
        }
    }
}

fn strict_connect_once(listener: &GatewayListenerIdentityV1, deadline_ns: u64) -> RawFd {
    let socket = unsafe {
        libc::socket(
            libc::AF_INET,
            libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
            libc::IPPROTO_TCP,
        )
    };
    if socket < 0 {
        return -1;
    }
    let address = libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: listener.port.to_be(),
        sin_addr: libc::in_addr {
            s_addr: u32::from_ne_bytes([127, 0, 0, 1]),
        },
        sin_zero: [0; 8],
    };
    let rc = unsafe {
        libc::connect(
            socket,
            (&address as *const libc::sockaddr_in).cast(),
            std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        )
    };
    if rc != 0 {
        if strict_errno() != libc::EINPROGRESS
            || !strict_poll_before(socket, libc::POLLOUT, deadline_ns)
        {
            unsafe { libc::close(socket) };
            return -1;
        }
        let mut socket_error: libc::c_int = -1;
        let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                socket,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut socket_error as *mut libc::c_int).cast(),
                &mut length,
            )
        } != 0
            || length as usize != std::mem::size_of::<libc::c_int>()
            || socket_error != 0
        {
            unsafe { libc::close(socket) };
            return -1;
        }
    }
    if strict_monotonic_ns().is_none_or(|now| now > deadline_ns) {
        unsafe { libc::close(socket) };
        return -1;
    }
    socket
}

fn strict_validate_connected_socket(
    fd: RawFd,
    listener: &GatewayListenerIdentityV1,
) -> Option<(u16, u64)> {
    let mut local: libc::sockaddr_in = unsafe { zeroed() };
    let mut local_length = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    let mut peer: libc::sockaddr_in = unsafe { zeroed() };
    let mut peer_length = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    if unsafe {
        libc::getsockname(
            fd,
            (&mut local as *mut libc::sockaddr_in).cast(),
            &mut local_length,
        )
    } != 0
        || unsafe {
            libc::getpeername(
                fd,
                (&mut peer as *mut libc::sockaddr_in).cast(),
                &mut peer_length,
            )
        } != 0
        || local_length as usize != std::mem::size_of::<libc::sockaddr_in>()
        || peer_length as usize != std::mem::size_of::<libc::sockaddr_in>()
    {
        return None;
    }
    let loopback = u32::from_ne_bytes([127, 0, 0, 1]);
    let local_port = u16::from_be(local.sin_port);
    if local.sin_family as i32 != libc::AF_INET
        || peer.sin_family as i32 != libc::AF_INET
        || local.sin_addr.s_addr != loopback
        || peer.sin_addr.s_addr != loopback
        || local_port == 0
        || local_port == listener.port
        || u16::from_be(peer.sin_port) != listener.port
    {
        return None;
    }
    let mut stat: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0
        || stat.st_mode & libc::S_IFMT != libc::S_IFSOCK
        || stat.st_ino == 0
    {
        return None;
    }
    Some((local_port, stat.st_ino))
}

fn strict_write_socket_before(fd: RawFd, mut bytes: &[u8], deadline_ns: u64) -> bool {
    while !bytes.is_empty() {
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written > 0 {
            bytes = &bytes[written as usize..];
            continue;
        }
        let errno = strict_errno();
        if errno == libc::EINTR {
            continue;
        }
        if (errno == libc::EAGAIN || errno == libc::EWOULDBLOCK)
            && strict_poll_before(fd, libc::POLLOUT, deadline_ns)
        {
            continue;
        }
        return false;
    }
    strict_monotonic_ns().is_some_and(|now| now <= deadline_ns)
}

fn strict_read_socket_before(fd: RawFd, bytes: &mut [u8], deadline_ns: u64) -> Option<usize> {
    let mut length = 0usize;
    loop {
        if length == bytes.len() {
            return None;
        }
        let read = unsafe {
            libc::read(
                fd,
                bytes[length..].as_mut_ptr().cast(),
                bytes.len() - length,
            )
        };
        if read > 0 {
            length += read as usize;
            continue;
        }
        if read == 0 {
            return strict_monotonic_ns()
                .filter(|now| *now <= deadline_ns)
                .map(|_| length);
        }
        let errno = strict_errno();
        if errno == libc::EINTR {
            continue;
        }
        if (errno == libc::EAGAIN || errno == libc::EWOULDBLOCK)
            && strict_poll_before(fd, libc::POLLIN, deadline_ns)
        {
            continue;
        }
        return None;
    }
}

fn strict_readiness_body(response: &[u8]) -> Option<&[u8]> {
    const PREFIX: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ";
    const SUFFIX: &[u8] = b"\r\nConnection: close\r\n\r\n";
    let remaining = response.strip_prefix(PREFIX)?;
    let separator = remaining
        .windows(SUFFIX.len())
        .position(|window| window == SUFFIX)?;
    let length_bytes = &remaining[..separator];
    if length_bytes.is_empty()
        || !length_bytes.iter().all(u8::is_ascii_digit)
        || (length_bytes[0] == b'0' && length_bytes.len() != 1)
    {
        return None;
    }
    let declared = strict_decimal(length_bytes)?;
    let body = &remaining[separator + SUFFIX.len()..];
    (declared == body.len() as u64).then_some(body)
}

fn strict_decimal(bytes: &[u8]) -> Option<u64> {
    let mut value = 0_u64;
    for byte in bytes {
        value = value
            .checked_mul(10)?
            .checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

struct StrictJsonCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> StrictJsonCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn expect(&mut self, literal: &[u8]) -> bool {
        let Some(end) = self.position.checked_add(literal.len()) else {
            return false;
        };
        if self.bytes.get(self.position..end) != Some(literal) {
            return false;
        }
        self.position = end;
        true
    }

    fn string(&mut self) -> Option<&'a [u8]> {
        if !self.expect(b"\"") {
            return None;
        }
        let start = self.position;
        while let Some(byte) = self.bytes.get(self.position).copied() {
            if byte == b'"' {
                let value = &self.bytes[start..self.position];
                self.position += 1;
                return Some(value);
            }
            if byte == b'\\' || byte < 0x20 || !byte.is_ascii() {
                return None;
            }
            self.position += 1;
        }
        None
    }

    fn number(&mut self) -> Option<u64> {
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .is_some_and(u8::is_ascii_digit)
        {
            self.position += 1;
        }
        let bytes = &self.bytes[start..self.position];
        if bytes.is_empty() || (bytes[0] == b'0' && bytes.len() != 1) {
            return None;
        }
        strict_decimal(bytes)
    }

    fn finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

fn strict_lower_hex_sha256(bytes: &[u8]) -> bool {
    bytes.len() == 64
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn strict_identifier(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
}

fn strict_validate_readiness_response(
    bindings: &StrictReadinessBindingsV1<'_>,
    body: &[u8],
) -> bool {
    let mut json = StrictJsonCursor::new(body);
    if !json.expect(b"{\"backend_id\":") {
        return false;
    }
    let Some(backend_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"config_projection_identity_hash\":") {
        return false;
    }
    let Some(projection_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"gateway_ref\":{\"authority_store_id\":") {
        return false;
    }
    let Some(gateway_authority_store_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"gateway_identity_hash\":") {
        return false;
    }
    let Some(gateway_identity_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"gateway_instance_id\":") {
        return false;
    }
    let Some(gateway_instance_id) = json.string() else {
        return false;
    };
    if !json.expect(b"},\"launch_input_hash\":") {
        return false;
    }
    let Some(launch_input_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"listener_identity\":{\"address\":") {
        return false;
    }
    let Some(listener_address) = json.string() else {
        return false;
    };
    if !json.expect(b",\"deny_boundary_effective_before_listen\":true,\"listen_backlog\":") {
        return false;
    }
    let Some(listen_backlog) = json.number() else {
        return false;
    };
    if !json.expect(b",\"network_namespace_inode\":") {
        return false;
    }
    let Some(network_namespace_inode) = json.number() else {
        return false;
    };
    if !json.expect(b",\"port\":") {
        return false;
    }
    let Some(port) = json.number() else {
        return false;
    };
    if !json.expect(b",\"responses_base_path\":") {
        return false;
    }
    let Some(responses_base_path) = json.string() else {
        return false;
    };
    if !json.expect(b",\"socket_inode\":") {
        return false;
    }
    let Some(listener_socket_inode) = json.number() else {
        return false;
    };
    if !json.expect(b",\"transport\":") {
        return false;
    }
    let Some(listener_transport) = json.string() else {
        return false;
    };
    if !json.expect(b"},\"orchestration_session_id\":") {
        return false;
    }
    let Some(orchestration_session_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"readiness_nonce\":") {
        return false;
    }
    let Some(readiness_nonce) = json.string() else {
        return false;
    };
    if !json.expect(b",\"retained_participant_id\":") {
        return false;
    }
    let Some(retained_participant_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"schema_version\":1,\"secret_handoff_consumed\":true,\"secret_handoff_prepared_ref\":{\"authority_store_id\":") {
        return false;
    }
    let Some(handoff_authority_store_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"handoff_hash\":") {
        return false;
    }
    let Some(handoff_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"handoff_id\":") {
        return false;
    }
    let Some(handoff_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"handoff_state_revision\":") {
        return false;
    }
    let Some(handoff_state_revision) = json.number() else {
        return false;
    };
    if !json.expect(b",\"orchestration_session_id\":") {
        return false;
    }
    let Some(handoff_orchestration_session_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"receiving_gateway_identity_hash\":") {
        return false;
    }
    let Some(receiving_gateway_identity_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"retained_participant_id\":") {
        return false;
    }
    let Some(handoff_retained_participant_id) = json.string() else {
        return false;
    };
    if !json.expect(b",\"runtime_family\":") {
        return false;
    }
    let Some(runtime_family) = json.string() else {
        return false;
    };
    if !json.expect(b",\"world_generation\":") {
        return false;
    }
    let Some(handoff_world_generation) = json.number() else {
        return false;
    };
    if !json.expect(b",\"world_id\":") {
        return false;
    }
    let Some(handoff_world_id) = json.string() else {
        return false;
    };
    if !json.expect(b"},\"secret_ready_attestation_hash\":") {
        return false;
    }
    let Some(secret_ready_attestation_hash) = json.string() else {
        return false;
    };
    if !json.expect(b",\"world_generation\":") {
        return false;
    }
    let Some(world_generation) = json.number() else {
        return false;
    };
    if !json.expect(b",\"world_id\":") {
        return false;
    }
    let Some(world_id) = json.string() else {
        return false;
    };
    if !json.expect(b"}") || !json.finished() {
        return false;
    }

    projection_hash == bindings.projection_identity_hash.as_bytes()
        && gateway_authority_store_id == bindings.gateway_ref.authority_store_id.as_bytes()
        && gateway_identity_hash == bindings.gateway_ref.gateway_identity_hash.as_bytes()
        && gateway_instance_id == bindings.gateway_ref.gateway_instance_id.as_bytes()
        && launch_input_hash == bindings.launch_input_hash.as_bytes()
        && listener_address == bindings.listener.address.as_bytes()
        && listen_backlog == u64::from(bindings.listener.listen_backlog)
        && network_namespace_inode == bindings.listener.network_namespace_inode
        && port == u64::from(bindings.listener.port)
        && responses_base_path == bindings.listener.responses_base_path.as_bytes()
        && listener_socket_inode == bindings.listener.socket_inode
        && listener_transport == bindings.listener.transport.as_bytes()
        && readiness_nonce == bindings.readiness_nonce.as_bytes()
        && handoff_authority_store_id == bindings.gateway_ref.authority_store_id.as_bytes()
        && receiving_gateway_identity_hash == bindings.gateway_ref.gateway_identity_hash.as_bytes()
        && orchestration_session_id == handoff_orchestration_session_id
        && retained_participant_id == handoff_retained_participant_id
        && world_generation == handoff_world_generation
        && world_id == handoff_world_id
        && handoff_state_revision > 0
        && strict_lower_hex_sha256(handoff_hash)
        && strict_lower_hex_sha256(secret_ready_attestation_hash)
        && strict_identifier(backend_id)
        && strict_identifier(orchestration_session_id)
        && strict_identifier(retained_participant_id)
        && strict_identifier(handoff_id)
        && strict_identifier(runtime_family)
        && strict_identifier(world_id)
        && world_generation > 0
}

fn probe_child_control_path_denials_v1(input: &E3WorldFsEnforcementInputV1) -> Result<String> {
    let mut probes = Vec::with_capacity(input.denied_control_probe_targets.len());
    for target in &input.denied_control_probe_targets {
        let (operation, observed_errno) = match &target.binding {
            E3DeniedControlProbeTargetBindingV1::AcceptedHomeRegistry { directory }
            | E3DeniedControlProbeTargetBindingV1::SiblingNativeRealization { directory }
            | E3DeniedControlProbeTargetBindingV1::WorldServiceState { directory }
            | E3DeniedControlProbeTargetBindingV1::OtherRolePrivateRoot { directory } => (
                "open_read_directory",
                denied_open_errno(&directory.physical_path, libc::O_RDONLY | libc::O_DIRECTORY)?,
            ),
            E3DeniedControlProbeTargetBindingV1::CgroupControl {
                cgroup,
                control_file,
            } => {
                if control_file != "cgroup.procs" {
                    bail!("invalid E3-D cgroup control probe target");
                }
                let path = format!(
                    "/sys/fs/cgroup/{}/{}",
                    cgroup.cgroup_relative_path.trim_start_matches('/'),
                    control_file
                );
                (
                    "open_write_cgroup_procs",
                    denied_open_errno(&path, libc::O_WRONLY)?,
                )
            }
            E3DeniedControlProbeTargetBindingV1::NftablesControl { .. } => {
                ("send_noop_nft_batch", send_noop_nft_batch_errno()?)
            }
            E3DeniedControlProbeTargetBindingV1::OtherRoleProcessState { pid, .. } => {
                let path = format!("/proc/{pid}/status");
                (
                    "open_read_proc_status",
                    denied_open_errno(&path, libc::O_RDONLY)?,
                )
            }
        };
        let expected_errno = if matches!(
            &target.binding,
            E3DeniedControlProbeTargetBindingV1::NftablesControl { .. }
        ) {
            libc::EPERM
        } else {
            libc::EACCES
        };
        if observed_errno != expected_errno {
            bail!(
                "E3-D control probe {operation} returned errno {observed_errno}, expected {expected_errno}"
            );
        }
        probes.push(E3DeniedControlProbeV1 {
            target: target.clone(),
            operation: operation.to_string(),
            result_errno: observed_errno,
        });
    }
    canonical_hash("substrate.e3.denied-control-probes.v1", "probes", &probes)
}

fn send_noop_nft_batch_errno() -> Result<i32> {
    const NLMSG_HDRLEN: usize = 16;
    const NFGENMSG_LEN: usize = 4;
    const NFNL_MSG_BATCH_BEGIN: u16 = 0x10;
    const NFNL_MSG_BATCH_END: u16 = 0x11;
    const NFNL_SUBSYS_NFTABLES: u16 = 10;
    const NFT_MSG_GETGEN: u16 = 16;
    const NLM_F_REQUEST: u16 = 0x01;
    const NLM_F_ACK: u16 = 0x04;
    const NLMSG_ERROR: u16 = 0x02;

    fn push_message(bytes: &mut Vec<u8>, message_type: u16, flags: u16, sequence: u32) {
        let length = (NLMSG_HDRLEN + NFGENMSG_LEN) as u32;
        bytes.extend_from_slice(&length.to_ne_bytes());
        bytes.extend_from_slice(&message_type.to_ne_bytes());
        bytes.extend_from_slice(&flags.to_ne_bytes());
        bytes.extend_from_slice(&sequence.to_ne_bytes());
        bytes.extend_from_slice(&0u32.to_ne_bytes());
        bytes.push(libc::AF_UNSPEC as u8);
        bytes.push(0);
        bytes.extend_from_slice(&0u16.to_be_bytes());
    }

    let fd = unsafe {
        libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_RAW | libc::SOCK_CLOEXEC,
            libc::NETLINK_NETFILTER,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open nftables denial probe socket");
    }
    let socket = unsafe { OwnedFd::from_raw_fd(fd) };
    let mut request = Vec::with_capacity(3 * (NLMSG_HDRLEN + NFGENMSG_LEN));
    push_message(&mut request, NFNL_MSG_BATCH_BEGIN, NLM_F_REQUEST, 1);
    push_message(
        &mut request,
        (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_GETGEN,
        NLM_F_REQUEST | NLM_F_ACK,
        1,
    );
    push_message(&mut request, NFNL_MSG_BATCH_END, NLM_F_REQUEST, 1);
    let mut kernel: libc::sockaddr_nl = unsafe { zeroed() };
    kernel.nl_family = libc::AF_NETLINK as libc::sa_family_t;
    kernel.nl_pid = 0;
    kernel.nl_groups = 0;
    let sent = unsafe {
        libc::sendto(
            socket.as_raw_fd(),
            request.as_ptr().cast(),
            request.len(),
            libc::MSG_NOSIGNAL,
            (&kernel as *const libc::sockaddr_nl).cast(),
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    if sent < 0 {
        return last_errno();
    }
    if sent as usize != request.len() {
        bail!("E3-D no-op nftables batch was only partially sent");
    }

    let mut poll = libc::pollfd {
        fd: socket.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let polled = unsafe { libc::poll(&mut poll, 1, 1_000) };
    if polled <= 0 {
        if polled == 0 {
            bail!("E3-D no-op nftables batch returned no conclusive response");
        }
        return Err(std::io::Error::last_os_error()).context("poll nftables denial probe response");
    }
    let mut response = [0u8; 4096];
    let received = unsafe {
        libc::recv(
            socket.as_raw_fd(),
            response.as_mut_ptr().cast(),
            response.len(),
            0,
        )
    };
    if received < 0 {
        return Err(std::io::Error::last_os_error()).context("read nftables denial probe response");
    }
    let mut offset = 0usize;
    let received = received as usize;
    while received.saturating_sub(offset) >= NLMSG_HDRLEN {
        let length = u32::from_ne_bytes(response[offset..offset + 4].try_into().unwrap()) as usize;
        if length < NLMSG_HDRLEN || length > received - offset {
            bail!("E3-D nftables denial response has malformed framing");
        }
        let message_type = u16::from_ne_bytes(response[offset + 4..offset + 6].try_into().unwrap());
        if message_type == NLMSG_ERROR && length >= NLMSG_HDRLEN + 4 {
            let error = i32::from_ne_bytes(
                response[offset + NLMSG_HDRLEN..offset + NLMSG_HDRLEN + 4]
                    .try_into()
                    .unwrap(),
            );
            if error < 0 {
                return Ok(-error);
            }
            if error == 0 {
                bail!("E3-D no-op nftables batch unexpectedly succeeded");
            }
        }
        offset = offset
            .checked_add((length + 3) & !3)
            .context("E3-D nftables response length overflow")?;
    }
    bail!("E3-D no-op nftables batch lacked a conclusive kernel error")
}

fn denied_open_errno(path: &str, flags: libc::c_int) -> Result<i32> {
    let path = CString::new(path).context("E3-D control probe path contains NUL")?;
    let fd = unsafe { libc::open(path.as_ptr(), flags | libc::O_CLOEXEC | libc::O_NOFOLLOW) };
    if fd >= 0 {
        unsafe { libc::close(fd) };
        bail!("E3-D control-path denial probe unexpectedly succeeded");
    }
    last_errno()
}

fn last_errno() -> Result<i32> {
    std::io::Error::last_os_error()
        .raw_os_error()
        .context("kernel failure did not report an errno")
}

fn build_child_security_attestation(
    input: &E3WorldFsEnforcementInputV1,
    namespace: &PreparedPrivateNamespaceV1,
    derived: &E3DerivedWorldFsEnforcementPlanV1,
    denied_control_probe_hash: String,
) -> Result<E3ChildSecurityAttestationV1> {
    let status = read_proc_file_at(namespace.self_proc_fd.as_raw_fd(), "status")?;
    let cap_last = namespace.cap_last_cap;
    let (ruid, euid, suid, fsuid) = parse_status_quad(&status, "Uid:")?;
    let (rgid, egid, sgid, fsgid) = parse_status_quad(&status, "Gid:")?;
    let mut attestation = E3ChildSecurityAttestationV1 {
        schema_version: 1,
        child_role: input.child_role,
        projection_identity_hash: input.projection_identity_hash.clone(),
        pid: std::process::id(),
        pid_start_time_ticks: namespace.pid_start_time_ticks,
        real_uid: ruid,
        effective_uid: euid,
        saved_uid: suid,
        filesystem_uid: fsuid,
        real_gid: rgid,
        effective_gid: egid,
        saved_gid: sgid,
        filesystem_gid: fsgid,
        supplementary_group_count: supplementary_group_count()?,
        cap_inheritable: status_hex(&status, "CapInh:")?,
        cap_permitted: status_hex(&status, "CapPrm:")?,
        cap_effective: status_hex(&status, "CapEff:")?,
        cap_bounding: status_hex(&status, "CapBnd:")?,
        cap_ambient: status_hex(&status, "CapAmb:")?,
        cap_last_cap: cap_last,
        no_new_privs: status_number(&status, "NoNewPrivs:")? == 1,
        dumpable: unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) } as u32,
        tracer_pid: status_number(&status, "TracerPid:")? as u32,
        kernel_boot_id: input.kernel_boot_id.clone(),
        user_namespace: namespace.user_namespace.clone(),
        seccomp_mode: status_number(&status, "Seccomp:")? as u32,
        landlock_abi: world::landlock::detect_support().abi.unwrap_or(0),
        e2_enforcement_plan_hash: derived.e2_plan_hash.clone(),
        derived_support_ruleset_hash: derived.derived_support_ruleset_hash.clone(),
        role_narrowing_ruleset_hash: derived.role_narrowing_ruleset_hash.clone(),
        effective_landlock_hash: derived.effective_landlock_hash.clone(),
        policy_snapshot_ref: input.policy_snapshot_ref.clone(),
        policy_snapshot_hash: input.policy_snapshot_hash.clone(),
        policy_snapshot_revision: input.policy_snapshot_revision.clone(),
        enforcement_input_hash: input.enforcement_input_hash.clone(),
        denied_control_probe_hash,
        attestation_hash: String::new(),
    };
    attestation.attestation_hash = hash_omitting(
        "substrate.e3.child-security-attestation.v1",
        "attestation",
        &attestation,
        "attestation_hash",
    )?;
    Ok(attestation)
}

fn validate_enforcement_input_hash(input: &E3WorldFsEnforcementInputV1) -> Result<()> {
    if input.schema_version != 1
        || input.support_policy_version != 1
        || input.target_uid == 0
        || input.target_gid == 0
        || input.user_namespace_requirement.trusted_service_uid != 0
        || input.user_namespace_requirement.uid_map
            != (E3LinuxIdMapExtentV1 {
                inside_id: input.target_uid,
                outside_id: input.target_uid,
                length: 1,
            })
        || input.user_namespace_requirement.gid_map
            != (E3LinuxIdMapExtentV1 {
                inside_id: input.target_gid,
                outside_id: input.target_gid,
                length: 1,
            })
    {
        bail!("malformed E3-D enforcement input");
    }
    let expected = hash_omitting(
        "substrate.e3.world-fs-enforcement-input.v1",
        "input",
        input,
        "enforcement_input_hash",
    )?;
    if input.enforcement_input_hash != expected {
        bail!("E3-D enforcement input hash mismatch");
    }
    validate_denied_control_probe_targets(&input.denied_control_probe_targets)?;
    match input.child_role {
        E3IsolatedChildRoleV1::Codex => {
            if input.immutable_config_source.is_none()
                || input.private_realization.is_none()
                || input.codex_launch_plan_hash.is_none()
            {
                bail!("Codex E3-D enforcement input has invalid role-specific nullability");
            }
        }
        E3IsolatedChildRoleV1::ManagedGateway => {
            if input.immutable_config_source.is_some()
                || input.private_realization.is_none()
                || input.codex_launch_plan_hash.is_some()
            {
                bail!("gateway E3-D enforcement input has invalid role-specific nullability");
            }
        }
        E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe => {
            if input.immutable_config_source.is_some()
                || input.private_realization.is_some()
                || input.codex_launch_plan_hash.is_some()
            {
                bail!("readiness E3-D enforcement input has invalid role-specific nullability");
            }
        }
    }
    Ok(())
}

fn validate_denied_control_probe_targets(
    targets: &[config_projection::E3DeniedControlProbeTargetV1],
) -> Result<()> {
    let mut prior_key: Option<(u8, Vec<u8>)> = None;
    let mut hashes = BTreeSet::new();
    for target in targets {
        let expected_hash = hash_omitting(
            "substrate.e3.denied-control-probe-target.v1",
            "target",
            target,
            "target_hash",
        )?;
        if target.target_hash != expected_hash || !hashes.insert(target.target_hash.clone()) {
            bail!("invalid or duplicate E3-D denied-control target hash");
        }
        let key = denied_control_target_order_key(&target.binding)?;
        if prior_key.as_ref().is_some_and(|prior| prior >= &key) {
            bail!("E3-D denied-control targets are not in exact canonical order");
        }
        prior_key = Some(key);
    }
    Ok(())
}

fn denied_control_target_order_key(
    binding: &E3DeniedControlProbeTargetBindingV1,
) -> Result<(u8, Vec<u8>)> {
    let key = match binding {
        E3DeniedControlProbeTargetBindingV1::AcceptedHomeRegistry { directory } => {
            (0, directory.physical_path.as_bytes().to_vec())
        }
        E3DeniedControlProbeTargetBindingV1::SiblingNativeRealization { directory } => {
            (1, directory.physical_path.as_bytes().to_vec())
        }
        E3DeniedControlProbeTargetBindingV1::CgroupControl {
            cgroup,
            control_file,
        } => {
            if control_file != "cgroup.procs" {
                bail!("E3-D cgroup target has the wrong control file");
            }
            let mut bytes = cgroup.cgroup_relative_path.as_bytes().to_vec();
            bytes.push(0);
            bytes.extend_from_slice(control_file.as_bytes());
            (2, bytes)
        }
        E3DeniedControlProbeTargetBindingV1::NftablesControl {
            network_namespace_inode,
        } => (3, network_namespace_inode.to_be_bytes().to_vec()),
        E3DeniedControlProbeTargetBindingV1::WorldServiceState { directory } => {
            (4, directory.physical_path.as_bytes().to_vec())
        }
        E3DeniedControlProbeTargetBindingV1::OtherRolePrivateRoot { directory } => {
            (5, directory.physical_path.as_bytes().to_vec())
        }
        E3DeniedControlProbeTargetBindingV1::OtherRoleProcessState {
            pid,
            pid_start_time_ticks,
            ..
        } => {
            let mut bytes = pid.to_be_bytes().to_vec();
            bytes.extend_from_slice(&pid_start_time_ticks.to_be_bytes());
            (6, bytes)
        }
    };
    Ok(key)
}

fn validate_role_binding(role: WrapperRoleV1, input_role: E3IsolatedChildRoleV1) -> Result<()> {
    if matches!(
        (role, input_role),
        (WrapperRoleV1::Codex, E3IsolatedChildRoleV1::Codex)
            | (
                WrapperRoleV1::ManagedGateway,
                E3IsolatedChildRoleV1::ManagedGateway
            )
            | (
                WrapperRoleV1::ManagedGatewayReadinessProbe,
                E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe
            )
    ) {
        Ok(())
    } else {
        bail!("wrapper role differs from the authenticated enforcement input")
    }
}

fn validate_codex_plan(
    plan: &E3CodexLaunchPlanV1,
    input: &E3WorldFsEnforcementInputV1,
) -> Result<()> {
    if plan.schema_version != 1
        || plan.series_id != plan.closed_projection_ref.series_id
        || plan.projection_identity_hash != input.projection_identity_hash
        || plan.stdin_fd != 0
        || plan.stdout_fd != 1
        || plan.stderr_fd != 2
        || plan.codex_argv.first().map(String::as_str) != Some("codex")
        || plan.final_environment.inherited_names != Vec::<String>::new()
        || plan.required_absent_codex_home_entries
            != [
                ".credentials.json".to_string(),
                "auth.json".to_string(),
                "cloud-requirements-cache.json".to_string(),
            ]
        || plan.output_last_message.mode != 0o600
        || plan.output_last_message.initial_byte_length != 0
        || plan.output_last_message.initial_sha256
            != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    {
        bail!("malformed E3-D Codex launch plan");
    }
    let expected = hash_omitting(
        "substrate.e3.codex-launch-plan.v1",
        "plan",
        plan,
        "plan_hash",
    )?;
    if plan.plan_hash != expected || input.codex_launch_plan_hash.as_ref() != Some(&expected) {
        bail!("E3-D Codex launch-plan hash mismatch");
    }
    Ok(())
}

fn bind_codex_mounts(descriptors: &LaunchDescriptorsV1, plan: &E3CodexLaunchPlanV1) -> Result<()> {
    let realization = descriptors.fd("SUBSTRATE_E3_NATIVE_REALIZATION_FD")?;
    let system_empty = descriptors.fd("SUBSTRATE_E3_SYSTEM_EMPTY_FD")?;
    let pid = std::process::id();
    let source = format!("/proc/{pid}/fd/{realization}/immutable/codex-home/config.toml");
    let target = format!("/proc/{pid}/fd/{realization}/codex-home/config.toml");
    bind_mount(&source, &target, false)?;
    bind_mount(&source, &target, true)?;
    bind_mount(
        &format!("/proc/{pid}/fd/{system_empty}"),
        "/etc/codex",
        true,
    )?;
    for name in &plan.required_absent_codex_home_entries {
        let path = format!("/proc/{pid}/fd/{realization}/codex-home/{name}");
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => bail!("forbidden Codex home entry exists"),
        }
    }
    Ok(())
}

fn bind_mount(source: &str, target: &str, read_only: bool) -> Result<()> {
    let source = CString::new(source)?;
    let target = CString::new(target)?;
    let flags = if read_only {
        libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY | libc::MS_NOSUID | libc::MS_NODEV
    } else {
        libc::MS_BIND
    };
    if unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            flags,
            std::ptr::null(),
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error())
            .context("install E3-D descriptor-rooted bind mount");
    }
    Ok(())
}

fn resolve_project_allowlist(project: &str, patterns: &[String]) -> Vec<String> {
    let mut paths = Vec::new();
    for raw in patterns {
        let pattern = raw.trim();
        if pattern.is_empty() {
            continue;
        }
        let pattern = pattern.trim_start_matches("./");
        let relative = if pattern.starts_with('/') {
            if pattern == project {
                "*"
            } else if let Some(relative) = pattern.strip_prefix(&format!("{project}/")) {
                relative
            } else {
                continue;
            }
        } else {
            pattern
        };
        let relative = relative.trim_start_matches("./");
        let prefix = relative
            .find(['*', '?', '['])
            .map(|index| &relative[..index])
            .unwrap_or(relative)
            .trim_matches('/');
        if prefix.split('/').any(|part| part == "..") {
            continue;
        }
        if prefix.is_empty() || prefix == "." {
            paths.push("/project".to_string());
            paths.push(project.to_string());
        } else {
            paths.push(format!("/project/{prefix}"));
            paths.push(format!("{project}/{prefix}"));
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn validate_current_map(name: &str, expected: &E3LinuxIdMapExtentV1) -> Result<()> {
    let bytes = std::fs::read_to_string(format!("/proc/{}/{name}", std::process::id()))?;
    let values = bytes
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if values != [expected.inside_id, expected.outside_id, expected.length] {
        bail!("wrapper {name} differs from the authenticated map");
    }
    Ok(())
}

fn validate_zero_capability_status(
    self_proc_fd: RawFd,
    uid: u64,
    gid: u64,
    cap_last: u32,
) -> Result<()> {
    let status = read_proc_file_at(self_proc_fd, "status")?;
    if parse_status_quad(&status, "Uid:")? != (uid, uid, uid, uid)
        || parse_status_quad(&status, "Gid:")? != (gid, gid, gid, gid)
        || supplementary_group_count()? != 0
        || ["CapInh:", "CapPrm:", "CapEff:", "CapBnd:", "CapAmb:"]
            .into_iter()
            .any(|key| status_hex(&status, key).ok().as_deref() != Some("0000000000000000"))
        || cap_last > 63
        || status_number(&status, "NoNewPrivs:")? != 1
    {
        bail!("wrapper privilege descent readback mismatch");
    }
    Ok(())
}

fn supplementary_group_count() -> Result<u32> {
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count < 0 {
        return Err(std::io::Error::last_os_error()).context("read supplementary groups");
    }
    Ok(count as u32)
}

fn read_proc_file_at(self_proc_fd: RawFd, name: &str) -> Result<String> {
    let name = CString::new(name)?;
    let fd = unsafe {
        libc::openat(
            self_proc_fd,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open numeric proc child");
    }
    let mut value = String::new();
    unsafe { File::from_raw_fd(fd) }
        .read_to_string(&mut value)
        .context("read numeric proc child")?;
    Ok(value)
}

fn parse_status_quad(status: &str, key: &str) -> Result<(u64, u64, u64, u64)> {
    let values = status_line(status, key)?
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if let [a, b, c, d] = values.as_slice() {
        Ok((*a, *b, *c, *d))
    } else {
        bail!("invalid {key} proc status shape")
    }
}

fn status_hex(status: &str, key: &str) -> Result<String> {
    let value = status_line(status, key)?.trim();
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid {key} proc status value");
    }
    Ok(value.to_ascii_lowercase())
}

fn status_number(status: &str, key: &str) -> Result<u64> {
    status_line(status, key)?
        .trim()
        .parse()
        .with_context(|| format!("parse {key}"))
}

fn status_line<'a>(status: &'a str, key: &str) -> Result<&'a str> {
    status
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .with_context(|| format!("missing {key} in proc status"))
}

fn process_start_time_at(self_proc_fd: RawFd) -> Result<u64> {
    let stat = read_proc_file_at(self_proc_fd, "stat")?;
    stat.rfind(") ")
        .and_then(|index| stat.get(index + 2..))
        .and_then(|tail| tail.split_ascii_whitespace().nth(19))
        .context("parse wrapper process start time")?
        .parse()
        .context("parse numeric wrapper process start time")
}

fn descriptor_identity(fd: RawFd) -> Result<(u64, u64)> {
    let mut stat: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat wrapper descriptor");
    }
    Ok((stat.st_dev, stat.st_ino))
}

fn descriptor_path(fd: RawFd) -> Result<String> {
    let bytes = std::fs::read_link(format!("/proc/{}/fd/{fd}", std::process::id()))?;
    bytes
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow::anyhow!("wrapper descriptor path is not UTF-8"))
}

fn read_bounded_eof(fd: RawFd) -> Result<Vec<u8>> {
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error()).context("duplicate bounded input descriptor");
    }
    let file = unsafe { File::from_raw_fd(duplicate) };
    let mut bytes = Vec::new();
    file.take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() > MAX_INPUT_BYTES {
        bail!("bounded wrapper input is empty or oversized");
    }
    unsafe { libc::close(fd) };
    Ok(bytes)
}

fn decode_exact_canonical<T>(bytes: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    let value = ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
    if ConfigProjectionCodecV1::encode_canonical_json(&value)? != bytes {
        bail!("wrapper input is not unique canonical JSON");
    }
    Ok(value)
}

fn hash_omitting<T: Serialize>(domain: &str, key: &str, value: &T, field: &str) -> Result<String> {
    let mut value = serde_json::to_value(value)?;
    value
        .as_object_mut()
        .context("hashed wrapper value must be an object")?
        .remove(field)
        .context("hashed wrapper value lacks its hash field")?;
    canonical_hash(domain, key, &value)
}

fn canonical_hash<T: Serialize>(domain: &str, key: &str, value: &T) -> Result<String> {
    let mut preimage = serde_json::Map::new();
    preimage.insert("domain".to_string(), Value::String(domain.to_string()));
    preimage.insert(key.to_string(), serde_json::to_value(value)?);
    ConfigProjectionCodecV1::domain_sha256("", &Value::Object(preimage)).map_err(Into::into)
}

fn canonical_value_hash(value: &Value) -> Result<String> {
    ConfigProjectionCodecV1::domain_sha256("", value).map_err(Into::into)
}

fn send_namespace_setup_byte(fd: RawFd, byte: u8) -> Result<()> {
    if unsafe { libc::send(fd, (&byte as *const u8).cast(), 1, libc::MSG_NOSIGNAL) } != 1 {
        return Err(std::io::Error::last_os_error()).context("send wrapper setup byte");
    }
    Ok(())
}

fn receive_namespace_setup_byte(fd: RawFd, expected: u8) -> Result<()> {
    let mut byte = 0u8;
    let mut control = [0u8; 64];
    let mut iovec = libc::iovec {
        iov_base: (&mut byte as *mut u8).cast(),
        iov_len: 1,
    };
    let mut message: libc::msghdr = unsafe { zeroed() };
    message.msg_iov = &mut iovec;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = control.len() as _;
    let received = unsafe { libc::recvmsg(fd, &mut message, libc::MSG_CMSG_CLOEXEC) };
    if received != 1
        || byte != expected
        || message.msg_controllen != 0
        || message.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0
    {
        bail!("missing or malformed wrapper release byte");
    }
    let trailing = unsafe {
        libc::recv(
            fd,
            (&mut byte as *mut u8).cast(),
            1,
            libc::MSG_DONTWAIT | libc::MSG_PEEK,
        )
    };
    if trailing >= 0 || std::io::Error::last_os_error().kind() != std::io::ErrorKind::WouldBlock {
        bail!("duplicate, trailing, or closed wrapper release channel");
    }
    Ok(())
}

fn receive_exact_pipe_gate(fd: RawFd, expected: u8) -> Result<()> {
    let bytes = read_bounded_eof(fd)?;
    if bytes.as_slice() != [expected] {
        bail!("missing or malformed wrapper release byte");
    }
    Ok(())
}

fn write_all_fd(fd: RawFd, mut bytes: &[u8]) -> Result<()> {
    while !bytes.is_empty() {
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written <= 0 {
            return Err(std::io::Error::last_os_error()).context("write wrapper attestation");
        }
        bytes = &bytes[written as usize..];
    }
    Ok(())
}

fn c_string_vector(values: Vec<String>) -> Result<Vec<CString>> {
    values
        .into_iter()
        .map(|value| CString::new(value).context("final exec value contains NUL"))
        .collect()
}

fn close_wrapper_descriptors_before_final_exec(descriptors: &LaunchDescriptorsV1) -> Result<()> {
    let retained: BTreeSet<RawFd> = if descriptors.role == WrapperRoleV1::ManagedGateway {
        [
            "SUBSTRATE_E3_GATEWAY_LAUNCH_FD",
            "SUBSTRATE_E3_GATEWAY_LISTENER_FD",
            "SUBSTRATE_E3_GATEWAY_SECRET_READY_FD",
            "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
            "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
        ]
        .into_iter()
        .map(|key| descriptors.fd(key))
        .collect::<Result<_>>()?
    } else {
        [descriptors.fd("SUBSTRATE_WORLD_ENTRY_BINARY_FD")?]
            .into_iter()
            .collect()
    };
    for descriptor in descriptors.descriptors.values().copied() {
        if !retained.contains(&descriptor) {
            unsafe { libc::close(descriptor) };
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::IntoRawFd;

    fn open_test_fds(count: usize) -> Vec<RawFd> {
        (0..count)
            .map(|_| File::open("/dev/null").unwrap().into_raw_fd())
            .collect()
    }

    #[test]
    fn descriptor_parser_accepts_only_exact_role_abi_and_distinct_decimal_fds() {
        let fds = open_test_fds(10);
        let mut env = BTreeMap::from([
            (
                "SUBSTRATE_WORLD_ENTRY_ROLE".to_string(),
                "codex".to_string(),
            ),
            (
                "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD".to_string(),
                fds[0].to_string(),
            ),
            (
                "SUBSTRATE_E3_NATIVE_REALIZATION_FD".to_string(),
                fds[1].to_string(),
            ),
            (
                "SUBSTRATE_E3_NATIVE_SOURCE_FD".to_string(),
                fds[2].to_string(),
            ),
            (
                "SUBSTRATE_E3_SYSTEM_EMPTY_FD".to_string(),
                fds[3].to_string(),
            ),
            (
                "SUBSTRATE_E3_WORLD_FS_INPUT_FD".to_string(),
                fds[4].to_string(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_BINARY_FD".to_string(),
                fds[5].to_string(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD".to_string(),
                fds[6].to_string(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD".to_string(),
                fds[7].to_string(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_USERNS_FD".to_string(),
                fds[8].to_string(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD".to_string(),
                fds[9].to_string(),
            ),
        ]);
        parse_launch_descriptors(&env).unwrap();
        env.insert("UNKNOWN".to_string(), "99".to_string());
        assert!(parse_launch_descriptors(&env).is_err());
        env.remove("UNKNOWN");
        env.insert(
            "SUBSTRATE_WORLD_ENTRY_BINARY_FD".to_string(),
            fds[4].to_string(),
        );
        assert!(parse_launch_descriptors(&env).is_err());
        env.insert(
            "SUBSTRATE_WORLD_ENTRY_BINARY_FD".to_string(),
            format!("+{}", fds[5]),
        );
        assert!(parse_launch_descriptors(&env).is_err());
        for fd in fds {
            unsafe { libc::close(fd) };
        }
    }

    #[test]
    fn setup_release_parser_rejects_wrong_trailing_and_eof_messages() {
        for malformed in [vec![0x00], vec![USERNS_MAPPED, 0xff], Vec::new()] {
            let mut sockets = [-1; 2];
            assert_eq!(
                unsafe {
                    libc::socketpair(
                        libc::AF_UNIX,
                        libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
                        0,
                        sockets.as_mut_ptr(),
                    )
                },
                0
            );
            if malformed.is_empty() {
                unsafe { libc::close(sockets[1]) };
            } else {
                assert_eq!(
                    unsafe {
                        libc::send(
                            sockets[1],
                            malformed.as_ptr().cast(),
                            malformed.len(),
                            libc::MSG_NOSIGNAL,
                        )
                    },
                    malformed.len() as isize
                );
            }
            assert!(receive_namespace_setup_byte(sockets[0], USERNS_MAPPED).is_err());
            unsafe {
                libc::close(sockets[0]);
                if !malformed.is_empty() {
                    libc::close(sockets[1]);
                }
            }
        }
    }

    #[test]
    fn pipe_gate_requires_one_byte_followed_by_eof() {
        for (bytes, accepted) in [
            (vec![FINAL_EXEC_RELEASE], true),
            (Vec::new(), false),
            (vec![0x00], false),
            (vec![FINAL_EXEC_RELEASE, 0xff], false),
        ] {
            let mut pipe = [-1; 2];
            assert_eq!(
                unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) },
                0
            );
            if !bytes.is_empty() {
                assert_eq!(
                    unsafe { libc::write(pipe[1], bytes.as_ptr().cast(), bytes.len()) },
                    bytes.len() as isize
                );
            }
            unsafe { libc::close(pipe[1]) };
            assert_eq!(
                receive_exact_pipe_gate(pipe[0], FINAL_EXEC_RELEASE).is_ok(),
                accepted
            );
            unsafe { libc::close(pipe[0]) };
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn readiness_seccomp_is_allow_only_and_argument_filters_socket() {
        fn run_child(operation: u8) -> libc::c_int {
            let pid = unsafe { libc::fork() };
            assert!(pid >= 0);
            if pid == 0 {
                if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0
                    || install_readiness_seccomp().is_err()
                {
                    unsafe { libc::_exit(120) };
                }
                match operation {
                    0 => {
                        if unsafe { libc::prctl(libc::PR_GET_SECCOMP, 0, 0, 0, 0) }
                            != libc::SECCOMP_MODE_FILTER as i32
                        {
                            unsafe { libc::_exit(121) };
                        }
                    }
                    1 => {
                        unsafe { libc::syscall(libc::SYS_getpid) };
                    }
                    2 => {
                        unsafe {
                            libc::socket(
                                libc::AF_UNIX,
                                libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                                0,
                            )
                        };
                    }
                    _ => unsafe { libc::_exit(122) },
                }
                unsafe { libc::_exit(0) };
            }
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
            status
        }

        assert!(libc::WIFEXITED(run_child(0)));
        for operation in [1, 2] {
            let status = run_child(operation);
            assert!(libc::WIFSIGNALED(status));
            assert_eq!(libc::WTERMSIG(status), libc::SIGSYS);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn readiness_state_machine_completes_without_post_filter_allocation_syscalls() {
        use std::io::{Read as _, Write as _};
        use std::net::{Shutdown, TcpListener};

        fn test_pipe() -> [RawFd; 2] {
            let mut pipe = [-1; 2];
            assert_eq!(
                unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) },
                0
            );
            pipe
        }

        fn read_pipe_to_eof(fd: RawFd) -> Vec<u8> {
            let mut file = unsafe { File::from_raw_fd(fd) };
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            bytes
        }

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut listener_stat: libc::stat = unsafe { zeroed() };
        assert_eq!(
            unsafe { libc::fstat(listener.as_raw_fd(), &mut listener_stat) },
            0
        );
        let listener_identity = GatewayListenerIdentityV1 {
            transport: "tcp".to_string(),
            network_namespace_inode: 7,
            address: "127.0.0.1".to_string(),
            port,
            socket_inode: listener_stat.st_ino,
            listen_backlog: 16,
            deny_boundary_effective_before_listen: true,
            responses_base_path: "/v1/responses".to_string(),
        };
        let gateway_ref = InWorldGatewayRefV1 {
            authority_store_id: "cpa_01991f65-7800-7000-8000-000000000001".to_string(),
            gateway_instance_id: "cgi_01991f65-7800-7000-8000-000000000002".to_string(),
            gateway_identity_hash: "b".repeat(64),
        };
        let projection_hash = "c".repeat(64);
        let launch_input_hash = "d".repeat(64);
        let readiness_nonce = "01991f65-7800-7000-8000-000000000003".to_string();
        let input_hash = "a".repeat(64);
        let input_hash_json = ConfigProjectionCodecV1::encode_canonical_json(&input_hash).unwrap();
        let cgroup = CanonicalCgroupIdentityV1 {
            cgroup_v2_mount_device_id: 1,
            cgroup_v2_mount_inode: 2,
            cgroup_directory_inode: 3,
            cgroup_relative_path: "substrate/test/readiness".to_string(),
        };
        let cgroup_json = ConfigProjectionCodecV1::encode_canonical_json(&cgroup).unwrap();
        let setup_ready = br#"{"schema_version":1}"#;
        let request = format!(
            "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Substrate-E3-Readiness-Nonce: {readiness_nonce}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
        )
        .into_bytes();
        let bindings = StrictReadinessBindingsV1 {
            listener: &listener_identity,
            projection_identity_hash: &projection_hash,
            gateway_ref: &gateway_ref,
            launch_input_hash: &launch_input_hash,
            readiness_nonce: &readiness_nonce,
            connect_deadline_ms: 1_000,
            request_release_deadline_ms: 1_000,
            response_deadline_ms: 1_000,
        };
        let setup = test_pipe();
        let start = test_pipe();
        let connected = test_pipe();
        let release = test_pipe();
        let result = test_pipe();
        let child = unsafe { libc::fork() };
        assert!(child >= 0);
        if child == 0 {
            unsafe {
                libc::close(setup[0]);
                libc::close(start[1]);
                libc::close(connected[0]);
                libc::close(release[1]);
                libc::close(result[0]);
            }
            let mut connected_bytes = FixedBuffer::<MAX_INPUT_BYTES>::new();
            let mut result_bytes = FixedBuffer::<MAX_INPUT_BYTES>::new();
            let mut hash_preimage = FixedBuffer::<MAX_INPUT_BYTES>::new();
            let mut response_bytes = [0_u8; MAX_INPUT_BYTES];
            let mut response_body_base64 = [0_u8; MAX_INPUT_BYTES];
            let probe_pid = std::process::id();
            if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0
                || install_readiness_seccomp().is_err()
            {
                unsafe { libc::_exit(120) };
            }
            run_readiness_probe_after_allow_only_filter(
                [setup[1], start[0], connected[1], release[0], result[1]],
                &bindings,
                probe_pid,
                41,
                &request,
                setup_ready,
                &input_hash_json,
                &cgroup_json,
                &mut connected_bytes,
                &mut result_bytes,
                &mut hash_preimage,
                &mut response_bytes,
                &mut response_body_base64,
            );
        }
        unsafe {
            libc::close(setup[1]);
            libc::close(start[0]);
            libc::close(connected[1]);
            libc::close(release[0]);
            libc::close(result[1]);
        }
        let observed_setup = read_pipe_to_eof(setup[0]);
        if observed_setup.is_empty() {
            let mut early_status = 0;
            assert_eq!(unsafe { libc::waitpid(child, &mut early_status, 0) }, child);
            panic!("strict readiness child exited before setup: {early_status:#x}");
        }
        assert_eq!(observed_setup, setup_ready);
        assert_eq!(
            unsafe { libc::write(start[1], [0x01_u8].as_ptr().cast(), 1) },
            1
        );
        unsafe { libc::close(start[1]) };
        let (mut stream, _) = listener.accept().unwrap();
        let connected_bytes = read_pipe_to_eof(connected[0]);
        let connected_attestation: ManagedGatewayReadinessProbeConnectedV1 =
            ConfigProjectionCodecV1::decode_canonical_json(&connected_bytes).unwrap();
        assert_eq!(connected_attestation.probe_pid, child as u32);
        assert_eq!(connected_attestation.probe_pid_start_time_ticks, 41);
        assert_eq!(connected_attestation.input_hash, input_hash);
        assert_eq!(connected_attestation.readiness_probe_cgroup, cgroup);
        assert_eq!(
            connected_attestation.connected_hash,
            hash_omitting(
                "substrate.e3.managed-gateway-readiness-probe-connected.v1",
                "connected",
                &connected_attestation,
                "connected_hash",
            )
            .unwrap()
        );
        assert_eq!(
            unsafe { libc::write(release[1], [0x01_u8].as_ptr().cast(), 1) },
            1
        );
        unsafe { libc::close(release[1]) };
        let mut received_request = Vec::new();
        let mut chunk = [0_u8; 256];
        while !received_request.ends_with(b"\r\n\r\n") {
            let count = stream.read(&mut chunk).unwrap();
            assert!(count > 0);
            received_request.extend_from_slice(&chunk[..count]);
        }
        assert_eq!(received_request, request);
        let handoff_ref = SecretHandoffRefV1 {
            authority_store_id: gateway_ref.authority_store_id.clone(),
            handoff_id: "hnd_01991f65-7800-7000-8000-000000000004".to_string(),
            orchestration_session_id: "orch_01991f65-7800-7000-8000-000000000005".to_string(),
            retained_participant_id: "part_01991f65-7800-7000-8000-000000000006".to_string(),
            runtime_family: "codex".to_string(),
            world_id: "wld_01991f65-7800-7000-8000-000000000007".to_string(),
            world_generation: 1,
            receiving_gateway_identity_hash: gateway_ref.gateway_identity_hash.clone(),
            handoff_state_revision: 1,
            handoff_hash: "e".repeat(64),
        };
        let body = ConfigProjectionCodecV1::encode_canonical_json(&serde_json::json!({
            "backend_id": "api:openai",
            "config_projection_identity_hash": projection_hash,
            "gateway_ref": gateway_ref,
            "launch_input_hash": launch_input_hash,
            "listener_identity": listener_identity,
            "orchestration_session_id": handoff_ref.orchestration_session_id,
            "readiness_nonce": readiness_nonce,
            "retained_participant_id": handoff_ref.retained_participant_id,
            "schema_version": 1,
            "secret_handoff_consumed": true,
            "secret_handoff_prepared_ref": handoff_ref,
            "secret_ready_attestation_hash": "f".repeat(64),
            "world_generation": 1,
            "world_id": "wld_01991f65-7800-7000-8000-000000000007",
        }))
        .unwrap();
        assert!(strict_validate_readiness_response(&bindings, &body));
        let mut malformed_body = body.clone();
        malformed_body.push(b' ');
        assert!(!strict_validate_readiness_response(
            &bindings,
            &malformed_body
        ));
        let response = [
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes(),
            body,
        ]
        .concat();
        stream.write_all(&response).unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
        let result_bytes = read_pipe_to_eof(result[0]);
        let result_value: Value =
            ConfigProjectionCodecV1::decode_canonical_json(&result_bytes).unwrap();
        assert_eq!(
            result_value["connected_hash"],
            connected_attestation.connected_hash
        );
        assert_eq!(result_value["input_hash"], input_hash);
        assert_eq!(
            result_value["child_security_attestation"]["schema_version"],
            1
        );
        let result_hash = result_value["result_hash"].as_str().unwrap().to_string();
        assert_eq!(
            result_hash,
            hash_omitting(
                "substrate.e3.managed-gateway-readiness-probe-result.v1",
                "result",
                &result_value,
                "result_hash",
            )
            .unwrap()
        );
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}
