use anyhow::{anyhow, Context, Result};
#[cfg(not(test))]
use config_projection::CanonicalCgroupIdentityV1;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener as StdTcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use substrate_common::{
    GatewayAuthBundleV1, GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION, SUBSTRATE_LLM_AUTH_BUNDLE_FD,
    SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
    SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
};
use tokio::sync::Mutex as AsyncMutex;
use transport_api_types::{
    validate_gateway_backend_id_selector, GatewayCliCodexIntegratedAuthV1, GatewayClientWiringV1,
    GatewayIntegratedAuthPayloadV1, GatewayLifecycleResponseV1, GatewayStatusV1,
};

use crate::e3_child_security::{E3PrivilegedChildExclusionV1, HeldNonE3PrivilegedChildLeaseV1};

/// Process-local ownership of an E3 gateway attempt. Dropping a pre-bound attempt closes its
/// socket before relinquishing process-wide exclusion. It never invokes a compatibility launcher.
pub(crate) struct E3GatewayRuntimeAuthorityV1 {
    launch: Option<ManagedGatewayLaunchCapabilityV1>,
    listener: Option<OwnedFd>,
    network_namespace: Option<std::fs::File>,
    bound_listener: Option<(u16, u64, u64)>,
    listening: bool,
    revoked: bool,
    terminal: bool,
    terminal_evidence: Option<config_projection::E3TerminalChildQuiescenceEvidenceV1>,
    cleanup_boundary: Option<config_projection::GatewayAccessBoundaryV1>,
    removed_cgroups: BTreeMap<String, bool>,
    cgroups: Vec<(
        std::fs::File,
        std::fs::File,
        config_projection::E3ChildCgroupRegistrationV1,
        config_projection::E3KernelEffectIntentV1,
    )>,
    pending_cgroup: Option<(
        std::fs::File,
        std::ffi::CString,
        config_projection::E3KernelEffectIntentV1,
    )>,
    boundary: Option<(
        config_projection::E3KernelEffectIntentV1,
        u64,
        config_projection::NftablesChainIdentityV1,
        Vec<config_projection::NftablesRuleIdentityV1>,
        config_projection::GatewayAccessPostureV1,
    )>,
    pending_boundary: Option<config_projection::E3KernelEffectIntentV1>,
    boundary_deleted: bool,
    boundary_removal_attempted: bool,
    cleanup_resolutions: Vec<config_projection::E3KernelEffectResolutionV1>,
    unresolved_intents: Vec<config_projection::E3KernelEffectIntentV1>,
    // Parent/name are retained immediately after mkdir, even if opening or writing fails.
    // The optional descriptors remain owned until exact child-free terminal cleanup.
    // Keep partial construction in this admitted owner; the E3-E catalog admits no new
    // config capability/accessor type. Optional descriptors record exact cleanup progress.
    #[allow(clippy::type_complexity)]
    gateway_config: Option<(
        std::fs::File,
        std::ffi::CString,
        Option<std::fs::File>,
        Option<std::fs::File>,
        Option<config_projection::GatewayRuntimeConfigIdentityV1>,
    )>,
    gateway_config_owner: Option<(u32, u32)>,
    gateway_config_bytes: Vec<u8>,
    // Only our successful unlink operations may justify absence on a cleanup retry.
    gateway_config_removed: (bool, bool),
    // Explicitly released only after socket closure and complete kernel resolution. An
    // unexpected drop with unresolved effects must keep process-wide child admission closed.
    _exclusion:
        std::mem::ManuallyDrop<crate::e3_child_security::HeldE3PrivilegedChildExclusionLeaseV1>,
}

// Non-cloneable, process-local launch permission and partial-resource owner. The original
// exclusion and listener remain in E3GatewayRuntimeAuthorityV1 until verified cleanup.
#[allow(dead_code)] // E3-E composition is called only by the bounded preparation boundary.
struct ManagedGatewayLaunchCapabilityV1 {
    deadline: libc::timespec,
    identity: config_projection::ConfigProjectionIdentityV1,
    launch_input: config_projection::ManagedGatewayLaunchInputV1,
    enforcement: config_projection::E3WorldFsEnforcementInputV1,
    exclusion: Arc<E3PrivilegedChildExclusionV1>,
    wrapper: fs::File,
    executable: fs::File,
    control_directories: Vec<(fs::File, config_projection::CanonicalDirectoryV1)>,
    // pid and pidfd are retained before fallible registration/gates. None registration means
    // cleanup must still kill/reap that child, but must not invent durable registration evidence.
    gateway_pid: Option<u32>,
    gateway_pidfd: Option<OwnedFd>,
    gateway_registration: Option<config_projection::E3ChildProcessRegistrationV1>,
    gateway_wait_status: Option<i32>,
    namespace_setup_started: bool,
    start_gate: Option<fs::File>,
    final_gate: Option<fs::File>,
    auth_writer: Option<fs::File>,
    secret_ready_reader: Option<OwnedFd>,
    security: Option<config_projection::E3ChildSecurityAttestationV1>,
    secret_ready: Option<config_projection::E3GatewaySecretReadyAttestationV1>,
    delivery_started: bool,
    delivered_at: Option<config_projection::Timestamp>,
    probe: Option<Box<ManagedGatewayLaunchCapabilityV1>>,
    probe_input: Option<serde_json::Value>,
    probe_start_gate: Option<fs::File>,
    probe_request_gate: Option<fs::File>,
    probe_connected_reader: Option<OwnedFd>,
    probe_result_reader: Option<OwnedFd>,
    readiness: Option<config_projection::E3ManagedGatewayActivationObservationV1>,
}

#[allow(dead_code)]
impl ManagedGatewayLaunchCapabilityV1 {
    fn e3_remaining_ms(deadline: &libc::timespec) -> Result<i32> {
        let mut now = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        anyhow::ensure!(
            unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut now) } == 0,
            "read E3 activation deadline"
        );
        let ns = (i128::from(deadline.tv_sec) - i128::from(now.tv_sec)) * 1_000_000_000
            + i128::from(deadline.tv_nsec)
            - i128::from(now.tv_nsec);
        anyhow::ensure!(ns > 0, "E3 activation deadline elapsed");
        Ok(((ns + 999_999) / 1_000_000).min(i128::from(i32::MAX)) as i32)
    }

    fn e3_read_bounded(fd: i32, deadline: &libc::timespec) -> Result<Vec<u8>> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        anyhow::ensure!(
            flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
            "bound E3 protocol read"
        );
        let mut bytes = Vec::with_capacity(65_536);
        loop {
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let rc = unsafe { libc::poll(&mut poll, 1, Self::e3_remaining_ms(deadline)?) };
            if rc < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            anyhow::ensure!(
                rc > 0 && poll.revents & (libc::POLLNVAL | libc::POLLERR) == 0,
                "E3 pipe read deadline or failure"
            );
            let mut buffer = [0u8; 4096];
            let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
            if count == 0 {
                return Ok(bytes);
            }
            if count < 0 {
                let error = std::io::Error::last_os_error();
                if matches!(error.raw_os_error(), Some(libc::EINTR | libc::EAGAIN)) {
                    continue;
                }
                return Err(error.into());
            }
            anyhow::ensure!(
                count as usize <= 65_536 - bytes.len(),
                "E3 pipe exceeded bounded protocol"
            );
            bytes.extend_from_slice(&buffer[..count as usize]);
        }
    }

    // Called once per one-shot channel. Taking the writer before this operation prevents a
    // partial write/error from becoming a resend. Auth bytes remain under a scrub owner.
    fn e3_write_once(writer: fs::File, bytes: &[u8], deadline: &libc::timespec) -> Result<()> {
        // A receiver can close between poll and write. Block SIGPIPE only on this
        // thread, preserving its previous mask and any already pending signal.
        struct PipeSignalMask {
            previous: libc::sigset_t,
            pipe: libc::sigset_t,
            was_pending: bool,
        }
        impl Drop for PipeSignalMask {
            fn drop(&mut self) {
                unsafe {
                    if !self.was_pending {
                        let timeout = libc::timespec {
                            tv_sec: 0,
                            tv_nsec: 0,
                        };
                        while libc::sigtimedwait(&self.pipe, std::ptr::null_mut(), &timeout) < 0
                            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                        {
                        }
                    }
                    libc::pthread_sigmask(libc::SIG_SETMASK, &self.previous, std::ptr::null_mut());
                }
            }
        }
        let mut pipe: libc::sigset_t = unsafe { std::mem::zeroed() };
        let mut previous: libc::sigset_t = unsafe { std::mem::zeroed() };
        unsafe {
            libc::sigemptyset(&mut pipe);
            libc::sigaddset(&mut pipe, libc::SIGPIPE);
        }
        let rc = unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, &pipe, &mut previous) };
        anyhow::ensure!(rc == 0, "block E3 pipe signal: {rc}");
        let mut pending: libc::sigset_t = unsafe { std::mem::zeroed() };
        let pending_result = unsafe { libc::sigpending(&mut pending) };
        let mask = PipeSignalMask {
            previous,
            pipe,
            was_pending: pending_result != 0
                || unsafe { libc::sigismember(&pending, libc::SIGPIPE) } == 1,
        };
        anyhow::ensure!(pending_result == 0, "read E3 pending pipe signal");
        let _mask = mask;
        let fd = writer.as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        anyhow::ensure!(
            flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
            "bound E3 protocol write"
        );
        let mut offset = 0;
        while offset < bytes.len() {
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLOUT,
                revents: 0,
            };
            let rc = unsafe { libc::poll(&mut poll, 1, Self::e3_remaining_ms(deadline)?) };
            if rc < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            anyhow::ensure!(
                rc > 0 && poll.revents & (libc::POLLNVAL | libc::POLLERR | libc::POLLHUP) == 0,
                "E3 pipe write deadline or receiver loss"
            );
            let count =
                unsafe { libc::write(fd, bytes[offset..].as_ptr().cast(), bytes.len() - offset) };
            if count < 0 {
                let error = std::io::Error::last_os_error();
                if matches!(error.raw_os_error(), Some(libc::EINTR | libc::EAGAIN)) {
                    continue;
                }
                return Err(error.into());
            }
            anyhow::ensure!(count > 0, "E3 pipe write made no progress");
            offset += count as usize;
        }
        drop(writer);
        Ok(())
    }

    fn e3_open_pinned(path: &str, directory: bool) -> Result<fs::File> {
        anyhow::ensure!(
            path.starts_with('/') && path != "/",
            "invalid E3 pinned coordinate"
        );
        let mut parent = fs::File::open("/")?;
        let parts: Vec<_> = path.split('/').skip(1).collect();
        for (index, part) in parts.iter().enumerate() {
            anyhow::ensure!(
                !part.is_empty() && !matches!(*part, "." | ".."),
                "invalid E3 pinned component"
            );
            let component = std::ffi::CString::new(*part)?;
            let fd = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    libc::O_RDONLY
                        | libc::O_CLOEXEC
                        | libc::O_NOFOLLOW
                        | if directory || index + 1 < parts.len() {
                            libc::O_DIRECTORY
                        } else {
                            0
                        },
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error()).context("open E3 pinned coordinate");
            }
            parent = unsafe { fs::File::from_raw_fd(fd) };
        }
        Ok(parent)
    }

    fn e3_validate_artifact(
        file: &fs::File,
        expected: &config_projection::DescriptorPinnedArtifactV1,
    ) -> Result<()> {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::{FileExt, MetadataExt};
        let m = file.metadata()?;
        anyhow::ensure!(
            m.is_file()
                && m.dev() == expected.device_id
                && m.ino() == expected.inode
                && m.uid() as u64 == expected.owner_uid
                && m.mode() & 0o7777 == expected.mode
                && m.len() == expected.byte_length
                && m.mode() & 0o6022 == 0
                && m.mode() & 0o111 != 0,
            "E3 executable identity drifted"
        );
        let mut hash = Sha256::new();
        let mut offset = 0;
        let mut buffer = [0u8; 65536];
        while offset < m.len() {
            let length = (m.len() - offset).min(buffer.len() as u64) as usize;
            file.read_exact_at(&mut buffer[..length], offset)?;
            hash.update(&buffer[..length]);
            offset += length as u64;
        }
        anyhow::ensure!(
            format!("{:x}", hash.finalize()) == expected.sha256,
            "E3 executable bytes changed"
        );
        let after = file.metadata()?;
        anyhow::ensure!(
            after.len() == m.len()
                && after.ctime() == m.ctime()
                && after.ctime_nsec() == m.ctime_nsec(),
            "E3 executable changed during hash"
        );
        Ok(())
    }

    fn e3_process_start(pid: u32) -> Result<u64> {
        let bytes = fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let (_, tail) = bytes.rsplit_once(") ").context("invalid E3 process stat")?;
        Ok(tail
            .split_whitespace()
            .nth(19)
            .context("missing E3 start time")?
            .parse()?)
    }

    fn e3_seal<T: Serialize>(domain: &str, key: &str, value: &T, omitted: &str) -> Result<String> {
        let mut value = serde_json::to_value(value)?;
        value
            .as_object_mut()
            .context("invalid E3 protocol object")?
            .remove(omitted);
        Ok(config_projection::ConfigProjectionCodecV1::domain_sha256(
            domain,
            &serde_json::json!({key:value}),
        )?)
    }
}

pub(crate) enum E3GatewayRevocationTargetV1<'a> {
    Live(&'a mut E3GatewayRuntimeAuthorityV1),
    Recovered {
        effects: &'a [config_projection::E3KernelEffectRecoveryV1],
        service_instance_id: &'a str,
        exclusion: &'a E3PrivilegedChildExclusionV1,
    },
}

impl Drop for E3GatewayRuntimeAuthorityV1 {
    fn drop(&mut self) {
        // The lease destructor is inert after explicit release; otherwise it poisons admission
        // without decrementing or releasing shared namespaces. Drop is never the cleanup worker.
        unsafe { std::mem::ManuallyDrop::drop(&mut self._exclusion) };
    }
}

impl E3GatewayRuntimeAuthorityV1 {
    pub(crate) fn release_exclusion_after_cleanup_v1(&mut self) -> Result<()> {
        anyhow::ensure!(
            self.revoked
                && self.terminal
                && self.listener.is_none()
                && self.cgroups.is_empty()
                && self.boundary.is_none()
                && self.pending_cgroup.is_none()
                && self.pending_boundary.is_none()
                && self.unresolved_intents.is_empty()
                && self.gateway_config.is_none(),
            "E3 runtime cleanup is incomplete"
        );
        self._exclusion.release_after_cleanup_v1()
    }

    pub(crate) fn new(
        exclusion: crate::e3_child_security::HeldE3PrivilegedChildExclusionLeaseV1,
    ) -> Self {
        Self {
            launch: None,
            listener: None,
            network_namespace: None,
            bound_listener: None,
            listening: false,
            revoked: false,
            terminal: false,
            terminal_evidence: None,
            cleanup_boundary: None,
            removed_cgroups: BTreeMap::new(),
            cgroups: Vec::with_capacity(3),
            pending_cgroup: None,
            boundary: None,
            pending_boundary: None,
            boundary_deleted: false,
            boundary_removal_attempted: false,
            cleanup_resolutions: Vec::with_capacity(4),
            unresolved_intents: Vec::with_capacity(4),
            gateway_config: None,
            gateway_config_owner: None,
            gateway_config_bytes: Vec::new(),
            gateway_config_removed: (false, false),
            _exclusion: std::mem::ManuallyDrop::new(exclusion),
        }
    }

    #[allow(dead_code)] // E3-E's bounded manager/harness owns the only caller.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn spawn_descriptor_pinned(
        &mut self,
        identity: &config_projection::ConfigProjectionIdentityV1,
        input: &config_projection::ManagedGatewayLaunchInputV1,
        e2: &transport_api_types::E2MemberLaunchActivationCarrierV1,
        exclusion: Arc<E3PrivilegedChildExclusionV1>,
        registry: &config_projection::ConfigProjectionRegistryV1,
        service_instance_id: &str,
        deadline: libc::timespec,
        authenticate: &mut dyn FnMut() -> Result<()>,
    ) -> Result<()> {
        anyhow::ensure!(!self.terminal, "E3 attempt is terminal");
        use config_projection::*;
        use std::os::fd::AsFd;
        type Launch = ManagedGatewayLaunchCapabilityV1;
        anyhow::ensure!(
            self.launch.is_none() && self.listening && !self.revoked,
            "E3 launch permission unavailable"
        );
        Launch::e3_remaining_ms(&deadline)?;
        authenticate()?;
        self.validate_listener_inventory()?;
        let (port, inode, namespace) = self.bound_listener.context("missing E3 listener")?;
        anyhow::ensure!(
            input.config_projection_identity_hash == identity.identity_hash
                && input.dormant_projection_ref.series_id == identity.series_id
                && input.listener_identity.port == port
                && input.listener_identity.socket_inode == inode
                && input.listener_identity.network_namespace_inode == namespace
                && e2.activation_id == identity.immutable_launch_cap.e2_activation_id
                && e2.immutable_worker_cap_ref
                    == identity.immutable_launch_cap.immutable_worker_cap_ref
                && e2.policy_snapshot_ref == identity.immutable_launch_cap.policy_snapshot_ref
                && e2.policy_snapshot_hash == identity.immutable_launch_cap.policy_snapshot_hash
                && e2.policy_snapshot_revision
                    == identity.immutable_launch_cap.policy_snapshot_revision,
            "E3 retained launch input mismatch"
        );
        let (uid, gid) = self
            .gateway_config_owner
            .context("missing E3 accepted owner")?;
        anyhow::ensure!(
            uid != 0 && gid != 0 && self.cgroups.len() == 3,
            "E3 launch lacks child identity/groups"
        );
        let (_, _, root, config, config_identity) = self
            .gateway_config
            .as_ref()
            .context("missing retained config")?;
        let root = root.as_ref().context("missing retained config root")?;
        let config = config.as_ref().context("missing retained config file")?;
        anyhow::ensure!(
            config_identity.as_ref() == Some(&input.gateway_config),
            "E3 config identity changed"
        );
        input
            .gateway_config
            .root
            .revalidate_linux_from_fd(root.as_fd())?;
        anyhow::ensure!(
            config.metadata()?.len() == input.gateway_config.byte_length,
            "E3 config length changed"
        );
        let wrapper = Launch::e3_open_pinned(
            &identity
                .runtime_artifacts
                .world_entry_wrapper
                .configured_absolute_path,
            false,
        )?;
        let executable = Launch::e3_open_pinned(
            &identity
                .runtime_artifacts
                .managed_gateway
                .configured_absolute_path,
            false,
        )?;
        Launch::e3_validate_artifact(&wrapper, &identity.runtime_artifacts.world_entry_wrapper)?;
        Launch::e3_validate_artifact(&executable, &identity.runtime_artifacts.managed_gateway)?;
        let mut directories = Vec::new();
        let accepted = Launch::e3_open_pinned(&identity.accepted_home.physical_path, true)?;
        identity
            .accepted_home
            .revalidate_linux_from_fd(accepted.as_fd())?;
        directories.push((accepted, identity.accepted_home.clone()));
        let state = Launch::e3_open_pinned("/run/substrate", true)?;
        let state_identity = CanonicalDirectoryV1::capture_linux_from_fd(state.as_fd())?;
        directories.push((state, state_identity.clone()));
        let mut targets = vec![E3DeniedControlProbeTargetBindingV1::AcceptedHomeRegistry {
            directory: identity.accepted_home.clone(),
        }];
        let mut groups = self
            .cgroups
            .iter()
            .map(|(_, _, r, _)| r.cgroup.clone())
            .collect::<Vec<_>>();
        groups.sort_by(|a, b| a.cgroup_relative_path.cmp(&b.cgroup_relative_path));
        targets.extend(groups.into_iter().map(|cgroup| {
            E3DeniedControlProbeTargetBindingV1::CgroupControl {
                cgroup,
                control_file: "cgroup.procs".into(),
            }
        }));
        targets.push(E3DeniedControlProbeTargetBindingV1::NftablesControl {
            network_namespace_inode: namespace,
        });
        targets.push(E3DeniedControlProbeTargetBindingV1::WorldServiceState {
            directory: state_identity,
        });
        let targets = targets
            .into_iter()
            .map(|binding| {
                let mut target = E3DeniedControlProbeTargetV1 {
                    binding,
                    target_hash: String::new(),
                };
                target.target_hash = Launch::e3_seal(
                    "substrate.e3.denied-control-probe-target.v1",
                    "target",
                    &target,
                    "target_hash",
                )?;
                Ok(target)
            })
            .collect::<Result<Vec<_>>>()?;
        let group = &self.cgroups[0].2;
        anyhow::ensure!(
            group.role == E3TerminalProcessRoleV1::ManagedGateway,
            "wrong E3 gateway group role"
        );
        let mut enforcement = E3WorldFsEnforcementInputV1 {
            schema_version: 1,
            child_role: E3IsolatedChildRoleV1::ManagedGateway,
            projection_identity_hash: identity.identity_hash.clone(),
            policy_authority: E3PolicyAuthoritySourceV1::InitialLaunch {
                e2_activation_id: e2.activation_id.clone(),
                commitment_ref: e2.commitment_ref.clone(),
            },
            immutable_worker_cap_ref: e2.immutable_worker_cap_ref.clone(),
            policy_snapshot_bytes_base64: e2.policy_snapshot_bytes_base64.clone(),
            policy_snapshot_byte_length: e2.policy_snapshot_byte_length,
            policy_snapshot_ref: e2.policy_snapshot_ref.clone(),
            policy_snapshot_hash: e2.policy_snapshot_hash.clone(),
            policy_snapshot_revision: e2.policy_snapshot_revision.clone(),
            expected_process_cgroup: group.cgroup.clone(),
            kernel_boot_id: group.kernel_boot_id.clone(),
            user_namespace_requirement:
                crate::e3_child_security::bind_e3_service_user_namespace_v1(
                    &exclusion,
                    uid.into(),
                    gid.into(),
                )?,
            target_uid: uid.into(),
            target_gid: gid.into(),
            immutable_config_source: None,
            private_realization: Some(input.gateway_config.root.clone()),
            codex_launch_plan_hash: None,
            executable_artifact: identity.runtime_artifacts.managed_gateway.clone(),
            denied_control_probe_targets: targets,
            support_policy_version: 1,
            enforcement_input_hash: String::new(),
        };
        enforcement.enforcement_input_hash = Launch::e3_seal(
            "substrate.e3.world-fs-enforcement-input.v1",
            "input",
            &enforcement,
            "enforcement_input_hash",
        )?;
        let enforcement =
            crate::e3_child_security::build_authenticated_world_fs_enforcement_input_v1(
                &exclusion,
                &enforcement,
            )?;
        let (auth_reader, auth_writer) = create_inherited_auth_bundle_pipe()?;
        let (secret_reader, secret_writer) = create_e3_gateway_secret_ready_pipe()?;
        // Gateway validates the receiver's pipe ownership after dropping privileges.
        for fd in [auth_reader.as_raw_fd(), secret_reader.as_raw_fd()] {
            anyhow::ensure!(
                unsafe { libc::fchown(fd, uid, gid) } == 0,
                "bind E3 gateway pipe ownership"
            );
        }
        let (fs_reader, fs_writer) = create_inherited_auth_bundle_pipe()?;
        let (launch_reader, launch_writer) = create_inherited_auth_bundle_pipe()?;
        anyhow::ensure!(
            unsafe { libc::fchown(launch_reader.as_raw_fd(), uid, gid) } == 0,
            "bind E3 launch-input pipe ownership"
        );
        let (setup_reader, setup_writer) = create_inherited_auth_bundle_pipe()?;
        let (final_reader, final_writer) = create_inherited_auth_bundle_pipe()?;
        let (start_reader, start_writer) = create_inherited_auth_bundle_pipe()?;
        let (userns_parent, userns_child) =
            crate::e3_child_security::create_e3_child_user_namespace_channel_v1()?;
        let entries = [
            ("SUBSTRATE_E3_GATEWAY_LAUNCH_FD", launch_reader.as_raw_fd()),
            (
                "SUBSTRATE_E3_GATEWAY_LISTENER_FD",
                self.listener
                    .as_ref()
                    .context("missing listener")?
                    .as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_GATEWAY_SECRET_READY_FD",
                secret_writer.as_raw_fd(),
            ),
            ("SUBSTRATE_LLM_AUTH_BUNDLE_FD", auth_reader.as_raw_fd()),
            ("SUBSTRATE_E3_WORLD_FS_INPUT_FD", fs_reader.as_raw_fd()),
            ("SUBSTRATE_WORLD_ENTRY_BINARY_FD", executable.as_raw_fd()),
            (
                "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD",
                final_reader.as_raw_fd(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                setup_writer.as_raw_fd(),
            ),
            ("SUBSTRATE_WORLD_ENTRY_USERNS_FD", userns_child.as_raw_fd()),
            ("SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD", root.as_raw_fd()),
        ];
        let mut env = entries
            .iter()
            .map(|(key, fd)| std::ffi::CString::new(format!("{key}={fd}")))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        env.push(std::ffi::CString::new(
            "SUBSTRATE_WORLD_ENTRY_ROLE=managed_gateway",
        )?);
        let mut env_ptrs = env.iter().map(|v| v.as_ptr()).collect::<Vec<_>>();
        env_ptrs.push(std::ptr::null());
        let argv = [c"substrate-world-entry".as_ptr(), std::ptr::null()];
        let wrapper_fd = wrapper.as_raw_fd();
        let null = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;
        self.launch = Some(ManagedGatewayLaunchCapabilityV1 {
            deadline,
            identity: identity.clone(),
            launch_input: input.clone(),
            enforcement,
            exclusion,
            wrapper,
            executable,
            control_directories: directories,
            gateway_pid: None,
            gateway_pidfd: None,
            gateway_registration: None,
            gateway_wait_status: None,
            namespace_setup_started: false,
            start_gate: Some(start_writer),
            final_gate: Some(final_writer),
            auth_writer: Some(auth_writer),
            secret_ready_reader: Some(secret_reader),
            security: None,
            secret_ready: None,
            delivery_started: false,
            delivered_at: None,
            probe: None,
            probe_input: None,
            probe_start_gate: None,
            probe_request_gate: None,
            probe_connected_reader: None,
            probe_result_reader: None,
            readiness: None,
        });
        // All allocation, argv, descriptors and cleanup ownership precede fork. This trusted
        // child performs only syscalls until descriptor-exec; no Rust runtime runs after fork.
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(std::io::Error::last_os_error()).context("fork E3 wrapper");
        }
        if pid == 0 {
            unsafe {
                if libc::syscall(
                    libc::SYS_close_range,
                    3u32,
                    u32::MAX,
                    libc::CLOSE_RANGE_CLOEXEC,
                ) != 0
                {
                    libc::_exit(120);
                }
                let mut gate = 0u8;
                if libc::read(start_reader.as_raw_fd(), (&mut gate as *mut u8).cast(), 1) != 1
                    || gate != 1
                {
                    libc::_exit(121);
                }
                if libc::setpgid(0, 0) != 0 {
                    libc::_exit(122);
                }
                for stdio in 0..3 {
                    if libc::dup2(null.as_raw_fd(), stdio) != stdio {
                        libc::_exit(123);
                    }
                }
                for (_, fd) in &entries {
                    if libc::fcntl(*fd, libc::F_SETFD, 0) != 0 {
                        libc::_exit(124);
                    }
                }
                libc::syscall(
                    libc::SYS_execveat,
                    wrapper_fd,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    env_ptrs.as_ptr(),
                    libc::AT_EMPTY_PATH,
                );
                libc::_exit(125);
            }
        }
        let owned = self.launch.as_mut().context("lost E3 launch owner")?;
        owned.gateway_pid = Some(pid as u32);
        let pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if pidfd < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 gateway pidfd");
        }
        owned.gateway_pidfd = Some(unsafe { OwnedFd::from_raw_fd(pidfd as i32) });
        let start = Launch::e3_process_start(pid as u32)?;
        let group_fd = self.cgroups[0].1.as_raw_fd();
        let procs = unsafe {
            libc::openat(
                group_fd,
                c"cgroup.procs".as_ptr(),
                libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if procs < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 gateway cgroup");
        }
        let mut procs = unsafe { fs::File::from_raw_fd(procs) };
        write!(procs, "{pid}")?;
        drop(procs);
        let mut registration = E3ChildProcessRegistrationV1 {
            schema_version: 1,
            authority_store_id: identity.authority_store_id.clone(),
            series_id: identity.series_id.clone(),
            registration_id: format!("ecp_{}", uuid::Uuid::now_v7()),
            cgroup_registration_id: group.cgroup_registration_id.clone(),
            cgroup_registration_hash: group.cgroup_registration_hash.clone(),
            fence_id: group.fence_id.clone(),
            role: group.role,
            pid: pid as u32,
            pid_start_time_ticks: start,
            process_cgroup: group.cgroup.clone(),
            kernel_boot_id: group.kernel_boot_id.clone(),
            parent_service_instance_id: service_instance_id.into(),
            registered_at: Timestamp(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            ),
            registration_hash: String::new(),
        };
        registration.registration_hash = Launch::e3_seal(
            "substrate.e3.child-process-registration.v1",
            "registration",
            &registration,
            "registration_hash",
        )?;
        owned.gateway_registration = Some(registration.clone());
        registry.publish_child_process_registration(&registration)?;
        drop(start_reader);
        drop(auth_reader);
        drop(secret_writer);
        drop(fs_reader);
        drop(launch_reader);
        drop(setup_writer);
        drop(final_reader);
        drop(userns_child);
        authenticate()?;
        Launch::e3_remaining_ms(&deadline)?;
        for (file, expected) in &owned.control_directories {
            expected.revalidate_linux_from_fd(file.as_fd())?;
        }
        Launch::e3_validate_artifact(
            &owned.wrapper,
            &identity.runtime_artifacts.world_entry_wrapper,
        )?;
        Launch::e3_write_once(
            owned.start_gate.take().context("E3 start gate consumed")?,
            &[1],
            &deadline,
        )?;
        Launch::e3_write_once(
            fs_writer,
            &ConfigProjectionCodecV1::encode_canonical_json(&owned.enforcement)?,
            &deadline,
        )?;
        let mut poll = libc::pollfd {
            fd: userns_parent.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        anyhow::ensure!(
            unsafe { libc::poll(&mut poll, 1, Launch::e3_remaining_ms(&deadline)?) } == 1
                && poll.revents & libc::POLLIN != 0,
            "E3 namespace setup deadline"
        );
        authenticate()?;
        owned.namespace_setup_started = true;
        crate::e3_child_security::install_and_validate_e3_child_user_namespace_v1(
            &mut self._exclusion,
            &userns_parent,
            &owned.identity,
            owned
                .gateway_registration
                .as_ref()
                .context("E3 setup lacks retained registration")?,
            &owned.enforcement.user_namespace_requirement,
        )?;
        let bytes = Launch::e3_read_bounded(setup_reader.as_raw_fd(), &deadline)?;
        drop(setup_reader);
        owned.security = Some(ConfigProjectionCodecV1::decode_canonical_json(&bytes)?);
        // Setup-ready proves the child has passed the mapped-release trailing-data check.
        drop(userns_parent);
        self.validate_child_security()?;
        authenticate()?;
        Launch::e3_remaining_ms(&deadline)?;
        let owned = self.launch.as_mut().context("lost E3 owner")?;
        Launch::e3_write_once(
            owned.final_gate.take().context("E3 final gate consumed")?,
            &[1],
            &deadline,
        )?;
        Launch::e3_write_once(
            launch_writer,
            &ConfigProjectionCodecV1::encode_canonical_json(input)?,
            &deadline,
        )?;
        self.validate_gateway_secret_ready()
    }

    #[allow(dead_code)]
    pub(crate) fn validate_child_security(&self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        let owned = self.launch.as_ref().context("E3 launch owner missing")?;
        ManagedGatewayLaunchCapabilityV1::e3_remaining_ms(&owned.deadline)?;
        let security = owned
            .security
            .as_ref()
            .context("E3 wrapper attestation missing")?;
        let registration = owned
            .gateway_registration
            .as_ref()
            .context("E3 child registration missing")?;
        anyhow::ensure!(
            security.pid == registration.pid
                && security.pid_start_time_ticks == registration.pid_start_time_ticks
                && owned.gateway_pid == Some(registration.pid),
            "E3 child attestation has wrong process"
        );
        crate::e3_child_security::validate_child_security_attestation_v1(
            &owned.exclusion,
            &owned.enforcement,
            security,
        )?;
        let executable = fs::File::open(format!("/proc/{}/exe", registration.pid))?;
        let expected = if owned.secret_ready.is_some() {
            &owned.identity.runtime_artifacts.managed_gateway
        } else {
            &owned.identity.runtime_artifacts.world_entry_wrapper
        };
        ManagedGatewayLaunchCapabilityV1::e3_validate_artifact(&executable, expected)?;
        let pidfd = owned.gateway_pidfd.as_ref().context("E3 pidfd missing")?;
        let info = fs::read_to_string(format!("/proc/self/fdinfo/{}", pidfd.as_raw_fd()))?;
        anyhow::ensure!(
            info.lines().any(|line| line
                .strip_prefix("Pid:")
                .is_some_and(|v| v.trim() == registration.pid.to_string())),
            "E3 pidfd identity changed"
        );
        let mut poll = libc::pollfd {
            fd: pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        anyhow::ensure!(
            unsafe { libc::poll(&mut poll, 1, 0) } == 0
                && executable.metadata()?.ino() == expected.inode,
            "E3 gateway died"
        );
        self.validate_listener_inventory()
    }

    #[allow(dead_code)]
    pub(crate) fn validate_gateway_secret_ready(&mut self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        type Launch = ManagedGatewayLaunchCapabilityV1;
        let owned = self.launch.as_mut().context("E3 launch owner missing")?;
        if owned.secret_ready.is_none() {
            let reader = owned
                .secret_ready_reader
                .take()
                .context("E3 secret-ready read already attempted")?;
            let bytes = Launch::e3_read_bounded(reader.as_raw_fd(), &owned.deadline)?;
            let ready: config_projection::E3GatewaySecretReadyAttestationV1 =
                config_projection::ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
            let security = owned.security.as_ref().context("E3 security missing")?;
            anyhow::ensure!(
                ready.schema_version == 1
                    && ready.gateway_instance_id
                        == owned.launch_input.gateway_ref.gateway_instance_id
                    && ready.launch_input_hash == owned.launch_input.launch_input_hash
                    && ready.gateway_pid == security.pid
                    && ready.gateway_pid_start_time_ticks == security.pid_start_time_ticks
                    && ready.user_namespace_device_id
                        == security.user_namespace.namespace_device_id
                    && ready.user_namespace_inode == security.user_namespace.namespace_inode
                    && ready.dumpable == 0
                    && ready.rlimit_core_soft == 0
                    && ready.rlimit_core_hard == 0
                    && ready.tracer_pid == 0
                    && ready.attestation_hash
                        == Launch::e3_seal(
                            "substrate.e3.gateway-secret-ready-attestation.v1",
                            "attestation",
                            &ready,
                            "attestation_hash"
                        )?,
                "E3 post-exec secret-ready attestation mismatch"
            );
            owned.secret_ready = Some(ready);
        }
        let ready = owned
            .secret_ready
            .as_ref()
            .context("E3 secret-ready evidence missing")?;
        {
            let proc = fs::File::open(format!("/proc/{}", ready.gateway_pid))?;
            anyhow::ensure!(
                proc.metadata()?.uid() == 0,
                "E3 gateway remains dumpable after exec"
            );
            let mut limits = libc::rlimit {
                rlim_cur: 1,
                rlim_max: 1,
            };
            anyhow::ensure!(
                unsafe {
                    libc::prlimit(
                        ready.gateway_pid as i32,
                        libc::RLIMIT_CORE,
                        std::ptr::null(),
                        &mut limits,
                    )
                } == 0
                    && limits.rlim_cur == 0
                    && limits.rlim_max == 0,
                "E3 core limit drifted"
            );
        }
        self.validate_child_security()
    }

    #[allow(dead_code)]
    pub(crate) fn deliver_secret_after_privilege_drop(
        &mut self,
        auth: &mut GatewayIntegratedAuthPayloadV1,
    ) -> Result<config_projection::Timestamp> {
        // Retry belongs exclusively to durable publication. A delivery attempt, including a
        // partial/error return, consumes the writer and can never be entered a second time.
        self.validate_gateway_secret_ready()?;
        let owned = self.launch.as_mut().context("E3 launch owner missing")?;
        anyhow::ensure!(
            !owned.delivery_started && owned.delivered_at.is_none(),
            "E3 auth delivery already attempted"
        );
        auth.validate_for_selected_backend(&owned.identity.backend_id)
            .map_err(|_| anyhow!("invalid E3 integrated auth"))?;
        let binding = resolve_gateway_backend_binding(&owned.identity.backend_id)
            .context("missing E3 backend binding")?;
        struct AuthScrub {
            bundle: GatewayAuthBundleV1,
            bytes: Vec<u8>,
        }
        impl Drop for AuthScrub {
            fn drop(&mut self) {
                for value in self.bundle.fields.values_mut() {
                    // SAFETY: replacing an exclusively borrowed UTF-8 allocation with NUL is
                    // valid UTF-8; volatile stores precede destruction on success/error/unwind.
                    for byte in unsafe { value.as_bytes_mut() } {
                        unsafe { std::ptr::write_volatile(byte, 0) };
                    }
                }
                for byte in &mut self.bytes {
                    unsafe { std::ptr::write_volatile(byte, 0) };
                }
                std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            }
        }
        let mut secret = AuthScrub {
            bundle: GatewayAuthBundleV1 {
                schema_version: GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
                backend_id: binding.auth_bundle_backend_id.into(),
                fields: HashMap::new(),
            },
            // JSON escaping expands each byte by at most six; reserve before accepting any
            // secret so serialization never abandons a reallocated credential buffer.
            bytes: Vec::with_capacity(65_536 * 6 + 4096),
        };
        match binding.auth_kind {
            GatewayIntegratedAuthKind::CliCodex => {
                let cli = auth.cli_codex.as_mut().context("missing E3 Codex auth")?;
                secret.bundle.fields.insert(
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                    std::mem::take(&mut cli.access_token),
                );
                if let Some(account) = cli.account_id.take() {
                    secret.bundle.fields.insert(
                        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.into(),
                        account,
                    );
                }
            }
            GatewayIntegratedAuthKind::ApiEnv => {
                let GatewayProviderAuthConfig::ApiKey {
                    env_var,
                    bundle_field,
                } = binding.provider_auth
                else {
                    anyhow::bail!("wrong E3 backend auth binding")
                };
                let value = auth
                    .api_env
                    .as_mut()
                    .context("missing E3 API auth")?
                    .env
                    .remove(env_var)
                    .context("missing E3 API credential")?;
                secret.bundle.fields.insert(bundle_field.into(), value);
            }
        }
        secret
            .bundle
            .validate()
            .map_err(|_| anyhow!("invalid E3 auth bundle"))?;
        serde_json::to_writer(&mut secret.bytes, &secret.bundle)
            .context("encode E3 auth bundle")?;
        anyhow::ensure!(
            secret.bytes.len() <= 65_536,
            "E3 auth bundle exceeds its receiver bound"
        );
        owned.delivery_started = true;
        let writer = owned
            .auth_writer
            .take()
            .context("E3 auth writer already consumed")?;
        ManagedGatewayLaunchCapabilityV1::e3_write_once(writer, &secret.bytes, &owned.deadline)?;
        drop(secret);
        let now = config_projection::Timestamp(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        );
        owned.delivered_at = Some(now.clone());
        self.validate_child_security()?;
        Ok(now)
    }

    #[allow(dead_code)]
    pub(crate) fn spawn_readiness_probe(
        &mut self,
        registry: &config_projection::ConfigProjectionRegistryV1,
        authenticate: &mut dyn FnMut() -> Result<()>,
    ) -> Result<()> {
        anyhow::ensure!(!self.terminal, "E3 attempt is terminal");
        use config_projection::*;
        use std::os::fd::AsFd;
        use std::os::unix::fs::MetadataExt;
        type Launch = ManagedGatewayLaunchCapabilityV1;
        self.validate_gateway_secret_ready()?;
        let main = self.launch.as_mut().context("E3 gateway owner missing")?;
        anyhow::ensure!(
            main.delivered_at.is_some() && main.probe.is_none() && main.readiness.is_none(),
            "E3 readiness launch already consumed or auth not delivered"
        );
        let deadline = main.deadline;
        authenticate()?;
        Launch::e3_remaining_ms(&deadline)?;
        let group = &self.cgroups[1].2;
        anyhow::ensure!(
            group.role == E3TerminalProcessRoleV1::ReadinessProbe,
            "wrong E3 probe cgroup role"
        );
        let mut enforcement = main.enforcement.clone();
        enforcement.child_role = E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe;
        enforcement.expected_process_cgroup = group.cgroup.clone();
        enforcement.executable_artifact =
            main.identity.runtime_artifacts.world_entry_wrapper.clone();
        enforcement.private_realization = None;
        let root =
            Launch::e3_open_pinned(&main.launch_input.gateway_config.root.physical_path, true)?;
        main.launch_input
            .gateway_config
            .root
            .revalidate_linux_from_fd(root.as_fd())?;
        let procfs = Launch::e3_open_pinned("/proc", true)?;
        let proc_meta = procfs.metadata()?;
        let process = main
            .gateway_registration
            .as_ref()
            .context("E3 gateway registration missing")?;
        let extra = [
            E3DeniedControlProbeTargetBindingV1::OtherRolePrivateRoot {
                directory: main.launch_input.gateway_config.root.clone(),
            },
            E3DeniedControlProbeTargetBindingV1::OtherRoleProcessState {
                pid: process.pid,
                pid_start_time_ticks: process.pid_start_time_ticks,
                procfs_mount_device_id: proc_meta.dev(),
                procfs_mount_inode: proc_meta.ino(),
            },
        ];
        for binding in extra {
            let mut target = E3DeniedControlProbeTargetV1 {
                binding,
                target_hash: String::new(),
            };
            target.target_hash = Launch::e3_seal(
                "substrate.e3.denied-control-probe-target.v1",
                "target",
                &target,
                "target_hash",
            )?;
            enforcement.denied_control_probe_targets.push(target);
        }
        enforcement.enforcement_input_hash = Launch::e3_seal(
            "substrate.e3.world-fs-enforcement-input.v1",
            "input",
            &enforcement,
            "enforcement_input_hash",
        )?;
        let enforcement =
            crate::e3_child_security::build_authenticated_world_fs_enforcement_input_v1(
                &main.exclusion,
                &enforcement,
            )?;
        let input = &main.launch_input;
        let millis = Launch::e3_remaining_ms(&deadline)?.min(5000);
        let mut probe_input = serde_json::json!({
            "schema_version":1,"launch_input_ref":{"authority_store_id": input.authority_store_id,
                "launch_input_id":input.launch_input_id,"launch_input_hash":input.launch_input_hash},
            "gateway_ref":input.gateway_ref,"listener_identity":input.listener_identity,"readiness_nonce":input.readiness_nonce,
            "readiness_probe_cgroup":group.cgroup,"target_uid":enforcement.target_uid,"target_gid":enforcement.target_gid,
            "enforcement_input":enforcement,"connect_deadline_ms":millis,"request_release_deadline_ms":millis,
            "response_deadline_ms":millis,"maximum_response_bytes":65536,"input_hash":""
        });
        probe_input["input_hash"] = serde_json::Value::String(Launch::e3_seal(
            "substrate.e3.managed-gateway-readiness-probe-input.v1",
            "input",
            &probe_input,
            "input_hash",
        )?);
        let wrapper = Launch::e3_open_pinned(
            &enforcement.executable_artifact.configured_absolute_path,
            false,
        )?;
        Launch::e3_validate_artifact(&wrapper, &enforcement.executable_artifact)?;
        let executable = wrapper.try_clone()?;
        let mut directories = main
            .control_directories
            .iter()
            .map(|(fd, id)| Ok((fd.try_clone()?, id.clone())))
            .collect::<Result<Vec<_>>>()?;
        directories.push((root, input.gateway_config.root.clone()));
        directories.push((
            procfs,
            CanonicalDirectoryV1 {
                physical_path: "/proc".into(),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: proc_meta.dev(),
                    inode: proc_meta.ino(),
                },
            },
        ));
        let (input_reader, input_writer) = create_inherited_auth_bundle_pipe()?;
        let (setup_reader, setup_writer) = create_inherited_auth_bundle_pipe()?;
        let (start_reader, start_writer) = create_inherited_auth_bundle_pipe()?;
        let (probe_start_reader, probe_start_writer) = create_inherited_auth_bundle_pipe()?;
        let (connected_reader, connected_writer) = create_inherited_auth_bundle_pipe()?;
        let (request_reader, request_writer) = create_inherited_auth_bundle_pipe()?;
        let (result_reader, result_writer) = create_inherited_auth_bundle_pipe()?;
        let (userns_parent, userns_child) =
            crate::e3_child_security::create_e3_child_user_namespace_channel_v1()?;
        let entries = [
            (
                "SUBSTRATE_E3_READINESS_PROBE_INPUT_FD",
                input_reader.as_raw_fd(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                setup_writer.as_raw_fd(),
            ),
            ("SUBSTRATE_WORLD_ENTRY_USERNS_FD", userns_child.as_raw_fd()),
            (
                "SUBSTRATE_E3_READINESS_PROBE_START_FD",
                probe_start_reader.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_CONNECTED_FD",
                connected_writer.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_REQUEST_RELEASE_FD",
                request_reader.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_RESULT_FD",
                result_writer.as_raw_fd(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SELF_ARTIFACT_FD",
                executable.as_raw_fd(),
            ),
        ];
        let mut env = entries
            .iter()
            .map(|(key, fd)| std::ffi::CString::new(format!("{key}={fd}")))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        env.push(std::ffi::CString::new(
            "SUBSTRATE_WORLD_ENTRY_ROLE=managed_gateway_readiness_probe",
        )?);
        let mut env_ptrs = env.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        env_ptrs.push(std::ptr::null());
        let argv = [c"substrate-world-entry".as_ptr(), std::ptr::null()];
        let wrapper_fd = wrapper.as_raw_fd();
        let null = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;
        main.probe = Some(Box::new(ManagedGatewayLaunchCapabilityV1 {
            deadline,
            identity: main.identity.clone(),
            launch_input: input.clone(),
            enforcement,
            exclusion: Arc::clone(&main.exclusion),
            wrapper,
            executable,
            control_directories: directories,
            gateway_pid: None,
            gateway_pidfd: None,
            gateway_registration: None,
            gateway_wait_status: None,
            namespace_setup_started: false,
            start_gate: Some(start_writer),
            final_gate: None,
            auth_writer: None,
            secret_ready_reader: None,
            security: None,
            secret_ready: None,
            delivery_started: false,
            delivered_at: None,
            probe: None,
            probe_input: Some(probe_input),
            probe_start_gate: Some(probe_start_writer),
            probe_request_gate: Some(request_writer),
            probe_connected_reader: Some(connected_reader),
            probe_result_reader: Some(result_reader),
            readiness: None,
        }));
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(std::io::Error::last_os_error()).context("fork E3 readiness wrapper");
        }
        if pid == 0 {
            unsafe {
                if libc::syscall(
                    libc::SYS_close_range,
                    3u32,
                    u32::MAX,
                    libc::CLOSE_RANGE_CLOEXEC,
                ) != 0
                {
                    libc::_exit(120);
                }
                let mut gate = 0u8;
                if libc::read(start_reader.as_raw_fd(), (&mut gate as *mut u8).cast(), 1) != 1
                    || gate != 1
                {
                    libc::_exit(121);
                }
                if libc::setpgid(0, 0) != 0 {
                    libc::_exit(122);
                }
                for stdio in 0..3 {
                    if libc::dup2(null.as_raw_fd(), stdio) != stdio {
                        libc::_exit(123);
                    }
                }
                for (_, fd) in &entries {
                    if libc::fcntl(*fd, libc::F_SETFD, 0) != 0 {
                        libc::_exit(124);
                    }
                }
                libc::syscall(
                    libc::SYS_execveat,
                    wrapper_fd,
                    c"".as_ptr(),
                    argv.as_ptr(),
                    env_ptrs.as_ptr(),
                    libc::AT_EMPTY_PATH,
                );
                libc::_exit(125);
            }
        }
        let probe = main.probe.as_mut().context("E3 probe owner lost")?;
        probe.gateway_pid = Some(pid as u32);
        let pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if pidfd < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 probe pidfd");
        }
        probe.gateway_pidfd = Some(unsafe { OwnedFd::from_raw_fd(pidfd as i32) });
        let start = Launch::e3_process_start(pid as u32)?;
        let procs = unsafe {
            libc::openat(
                self.cgroups[1].1.as_raw_fd(),
                c"cgroup.procs".as_ptr(),
                libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if procs < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 probe cgroup");
        }
        let mut procs = unsafe { fs::File::from_raw_fd(procs) };
        write!(procs, "{pid}")?;
        drop(procs);
        let mut registration = E3ChildProcessRegistrationV1 {
            schema_version: 1,
            authority_store_id: probe.identity.authority_store_id.clone(),
            series_id: probe.identity.series_id.clone(),
            registration_id: format!("ecp_{}", uuid::Uuid::now_v7()),
            cgroup_registration_id: group.cgroup_registration_id.clone(),
            cgroup_registration_hash: group.cgroup_registration_hash.clone(),
            fence_id: group.fence_id.clone(),
            role: group.role,
            pid: pid as u32,
            pid_start_time_ticks: start,
            process_cgroup: group.cgroup.clone(),
            kernel_boot_id: group.kernel_boot_id.clone(),
            parent_service_instance_id: process.parent_service_instance_id.clone(),
            registered_at: Timestamp(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            ),
            registration_hash: String::new(),
        };
        registration.registration_hash = Launch::e3_seal(
            "substrate.e3.child-process-registration.v1",
            "registration",
            &registration,
            "registration_hash",
        )?;
        probe.gateway_registration = Some(registration.clone());
        registry.publish_child_process_registration(&registration)?;
        drop(input_reader);
        drop(setup_writer);
        drop(start_reader);
        drop(probe_start_reader);
        drop(connected_writer);
        drop(request_reader);
        drop(result_writer);
        drop(userns_child);
        authenticate()?;
        for (file, expected) in &probe.control_directories {
            expected.revalidate_linux_from_fd(file.as_fd())?;
        }
        Launch::e3_write_once(
            probe
                .start_gate
                .take()
                .context("E3 probe exec gate consumed")?,
            &[1],
            &deadline,
        )?;
        Launch::e3_write_once(
            input_writer,
            &ConfigProjectionCodecV1::encode_canonical_json(
                probe.probe_input.as_ref().context("E3 probe input lost")?,
            )?,
            &deadline,
        )?;
        let mut poll = libc::pollfd {
            fd: userns_parent.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        anyhow::ensure!(
            unsafe { libc::poll(&mut poll, 1, Launch::e3_remaining_ms(&deadline)?) } == 1
                && poll.revents & libc::POLLIN != 0,
            "E3 probe namespace setup deadline"
        );
        authenticate()?;
        probe.namespace_setup_started = true;
        crate::e3_child_security::install_and_validate_e3_child_user_namespace_v1(
            &mut self._exclusion,
            &userns_parent,
            &probe.identity,
            probe
                .gateway_registration
                .as_ref()
                .context("E3 setup lacks retained registration")?,
            &probe.enforcement.user_namespace_requirement,
        )?;
        let bytes = Launch::e3_read_bounded(setup_reader.as_raw_fd(), &deadline)?;
        drop(setup_reader);
        probe.security = Some(ConfigProjectionCodecV1::decode_canonical_json(&bytes)?);
        // Setup-ready proves the child has passed the mapped-release trailing-data check.
        drop(userns_parent);
        self.validate_readiness_probe()
    }

    #[allow(dead_code)]
    pub(crate) fn validate_readiness_probe(&self) -> Result<()> {
        type Launch = ManagedGatewayLaunchCapabilityV1;
        self.validate_child_security()?;
        let probe = self
            .launch
            .as_ref()
            .and_then(|o| o.probe.as_ref())
            .context("E3 probe owner missing")?;
        Launch::e3_remaining_ms(&probe.deadline)?;
        let registration = probe
            .gateway_registration
            .as_ref()
            .context("E3 probe registration missing")?;
        let security = probe
            .security
            .as_ref()
            .context("E3 probe security missing")?;
        anyhow::ensure!(
            registration.pid == security.pid
                && registration.pid_start_time_ticks == security.pid_start_time_ticks,
            "E3 probe identity mismatch"
        );
        crate::e3_child_security::validate_child_security_attestation_v1(
            &probe.exclusion,
            &probe.enforcement,
            security,
        )?;
        let executable = fs::File::open(format!("/proc/{}/exe", registration.pid))?;
        Launch::e3_validate_artifact(&executable, &probe.enforcement.executable_artifact)?;
        let pidfd = probe
            .gateway_pidfd
            .as_ref()
            .context("E3 probe pidfd missing")?;
        let mut poll = libc::pollfd {
            fd: pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        anyhow::ensure!(
            unsafe { libc::poll(&mut poll, 1, 0) } == 0,
            "E3 probe died before barrier"
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn probe_readiness(
        &mut self,
        authenticate: &mut dyn FnMut() -> Result<()>,
    ) -> Result<config_projection::E3ManagedGatewayActivationObservationV1> {
        use base64::Engine;
        use config_projection::*;
        use sha2::{Digest, Sha256};
        type Launch = ManagedGatewayLaunchCapabilityV1;
        self.validate_child_security()?;
        if let Some(observation) = self.launch.as_ref().and_then(|o| o.readiness.as_ref()) {
            authenticate()?;
            return Ok(observation.clone());
        }
        self.validate_readiness_probe()?;
        authenticate()?;
        let probe = self
            .launch
            .as_mut()
            .and_then(|o| o.probe.as_mut())
            .context("E3 probe owner missing")?;
        let deadline = probe.deadline;
        Launch::e3_write_once(
            probe
                .probe_start_gate
                .take()
                .context("E3 probe connect gate already consumed")?,
            &[1],
            &deadline,
        )?;
        let reader = probe
            .probe_connected_reader
            .take()
            .context("E3 connected read already consumed")?;
        let bytes = Launch::e3_read_bounded(reader.as_raw_fd(), &deadline)?;
        drop(reader);
        let connected: serde_json::Value = ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
        let child = probe
            .gateway_registration
            .as_ref()
            .context("E3 probe registration missing")?;
        let input = probe
            .probe_input
            .as_ref()
            .context("E3 probe input missing")?;
        let socket_fd = connected["socket_fd"]
            .as_u64()
            .filter(|fd| *fd >= 3 && *fd <= i32::MAX as u64)
            .context("invalid E3 connected socket fd")?;
        let inode = connected["socket_inode"]
            .as_u64()
            .filter(|inode| *inode != 0)
            .context("invalid E3 connected socket inode")?;
        let local_port = connected["local_port"]
            .as_u64()
            .filter(|p| *p > 0 && *p <= u16::MAX as u64)
            .context("invalid E3 probe local port")?;
        let peer_port = probe.launch_input.listener_identity.port;
        anyhow::ensure!(
            connected.as_object().is_some_and(|o| o.len() == 12)
                && connected["schema_version"] == 1
                && connected["probe_pid"] == child.pid
                && connected["probe_pid_start_time_ticks"] == child.pid_start_time_ticks
                && connected["input_hash"] == input["input_hash"]
                && connected["local_address"] == "127.0.0.1"
                && connected["peer_address"] == "127.0.0.1"
                && connected["peer_port"] == peer_port
                && connected["readiness_probe_cgroup"]
                    == serde_json::to_value(&child.process_cgroup)?
                && connected["connected_hash"]
                    == Launch::e3_seal(
                        "substrate.e3.managed-gateway-readiness-probe-connected.v1",
                        "connected",
                        &connected,
                        "connected_hash"
                    )?,
            "E3 connected observation mismatch"
        );
        let observed = fs::read_link(format!("/proc/{}/fd/{socket_fd}", child.pid))?;
        anyhow::ensure!(
            observed == PathBuf::from(format!("socket:[{inode}]")),
            "E3 probe socket descriptor substituted"
        );
        let tcp = fs::read_to_string(format!("/proc/{}/net/tcp", child.pid))?;
        let socket_rows = tcp
            .lines()
            .skip(1)
            .filter_map(|line| {
                let fields = line.split_whitespace().collect::<Vec<_>>();
                (fields.len() >= 10 && fields[9].parse::<u64>().ok() == Some(inode))
                    .then_some(fields)
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            socket_rows.len() == 1
                && socket_rows[0][1] == format!("0100007F:{local_port:04X}")
                && socket_rows[0][2] == format!("0100007F:{peer_port:04X}")
                && socket_rows[0][3] == "01",
            "E3 probe socket tuple/state mismatch"
        );
        self.validate_readiness_probe()?;
        authenticate()?;
        let probe = self
            .launch
            .as_mut()
            .and_then(|o| o.probe.as_mut())
            .context("E3 probe owner missing")?;
        Launch::e3_write_once(
            probe
                .probe_request_gate
                .take()
                .context("E3 request gate already consumed")?,
            &[1],
            &deadline,
        )?;
        let reader = probe
            .probe_result_reader
            .take()
            .context("E3 result read already consumed")?;
        let bytes = Launch::e3_read_bounded(reader.as_raw_fd(), &deadline)?;
        drop(reader);
        let result: serde_json::Value = ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
        let child = probe
            .gateway_registration
            .as_ref()
            .context("E3 probe registration missing")?;
        anyhow::ensure!(
            result.as_object().is_some_and(|o| o.len() == 10)
                && result["schema_version"] == 1
                && result["probe_pid"] == child.pid
                && result["probe_pid_start_time_ticks"] == child.pid_start_time_ticks
                && result["input_hash"] == connected["input_hash"]
                && result["connected_hash"] == connected["connected_hash"]
                && result["child_security_attestation"]
                    == serde_json::to_value(
                        probe
                            .security
                            .as_ref()
                            .context("E3 probe security missing")?
                    )?
                && result["http_status"] == 200
                && result["result_hash"]
                    == Launch::e3_seal(
                        "substrate.e3.managed-gateway-readiness-probe-result.v1",
                        "result",
                        &result,
                        "result_hash"
                    )?,
            "E3 readiness result mismatch"
        );
        let encoded = result["response_body_base64"]
            .as_str()
            .context("E3 response encoding missing")?;
        let body = base64::engine::general_purpose::STANDARD.decode(encoded)?;
        anyhow::ensure!(
            body.len() <= 65536
                && base64::engine::general_purpose::STANDARD.encode(&body) == encoded
                && result["response_body_sha256"] == format!("{:x}", Sha256::digest(&body)),
            "E3 response bytes mismatch"
        );
        let response: serde_json::Value = ConfigProjectionCodecV1::decode_canonical_json(&body)?;
        let main = self.launch.as_ref().context("E3 gateway owner missing")?;
        let input = &main.launch_input;
        let expected = serde_json::json!({"schema_version":1,"readiness_nonce":input.readiness_nonce,
            "launch_input_hash":input.launch_input_hash,"gateway_ref":input.gateway_ref,
            "config_projection_identity_hash":input.config_projection_identity_hash,"orchestration_session_id":input.orchestration_session_id,
            "retained_participant_id":input.retained_participant_id,"backend_id":input.backend_id,"world_id":input.world_id,
            "world_generation":input.world_generation,"listener_identity":input.listener_identity,
            "secret_handoff_prepared_ref":input.secret_handoff_prepared_ref,"secret_handoff_consumed":true,
            "secret_ready_attestation_hash":main.secret_ready.as_ref().context("E3 secret-ready missing")?.attestation_hash});
        anyhow::ensure!(response == expected, "E3 readiness body binding mismatch");
        // The exact child must exit successfully and be reaped before ownership can transfer.
        let probe = self
            .launch
            .as_mut()
            .and_then(|o| o.probe.as_mut())
            .context("E3 probe owner missing")?;
        let pidfd = probe
            .gateway_pidfd
            .as_ref()
            .context("E3 probe pidfd missing")?;
        let mut poll = libc::pollfd {
            fd: pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        anyhow::ensure!(
            unsafe { libc::poll(&mut poll, 1, Launch::e3_remaining_ms(&deadline)?) } == 1
                && poll.revents & libc::POLLIN != 0,
            "E3 probe exit deadline"
        );
        let pid = probe.gateway_pid.context("E3 probe pid missing")?;
        let mut status = 0;
        anyhow::ensure!(
            unsafe { libc::waitpid(pid as i32, &mut status, libc::WNOHANG) } == pid as i32,
            "E3 probe wait identity mismatch"
        );
        probe.gateway_wait_status = Some(status);
        anyhow::ensure!(
            libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
            "E3 readiness probe failed"
        );
        self._exclusion.release_terminal_child_user_namespace_v1(
            probe
                .gateway_registration
                .as_ref()
                .context("E3 probe registration missing")?,
        )?;
        self.validate_gateway_secret_ready()?;
        authenticate()?;
        let main = self.launch.as_mut().context("E3 gateway owner missing")?;
        let security = main
            .security
            .as_ref()
            .context("E3 security missing")?
            .clone();
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        anyhow::ensure!(
            unsafe {
                libc::fstat(
                    main.gateway_pidfd
                        .as_ref()
                        .context("E3 pidfd missing")?
                        .as_raw_fd(),
                    &mut stat,
                )
            } == 0,
            "E3 pidfd metadata missing"
        );
        let artifact = &main.identity.runtime_artifacts.managed_gateway;
        let observation = E3ManagedGatewayActivationObservationV1::Ready {
            gateway_process_identity: GatewayProcessIdentityV1 {
                pid: security.pid,
                pid_start_time_ticks: security.pid_start_time_ticks,
                pidfd_inode: stat.st_ino,
                executable_device_id: artifact.device_id,
                executable_inode: artifact.inode,
                executable_sha256: artifact.sha256.clone(),
                process_cgroup: main.enforcement.expected_process_cgroup.clone(),
                child_security_attestation_hash: security.attestation_hash.clone(),
                secret_ready_attestation_hash: main
                    .secret_ready
                    .as_ref()
                    .context("E3 secret-ready missing")?
                    .attestation_hash
                    .clone(),
            },
            child_security_attestation: security,
            observed_at: Timestamp(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            ),
        };
        main.readiness = Some(observation.clone());
        Ok(observation)
    }

    pub(crate) fn bind_dormant_listener(&mut self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        anyhow::ensure!(
            !self.revoked
                && self.listener.is_none()
                && self.network_namespace.is_none()
                && self.bound_listener.is_none(),
            "E3 listener is already reserved"
        );
        let namespace = std::fs::File::open("/proc/self/ns/net")?;
        let namespace_inode = namespace.metadata()?.ino();
        anyhow::ensure!(namespace_inode != 0, "E3 network namespace has no identity");
        // SAFETY: constant protocol arguments; the returned descriptor is owned immediately.
        let raw = unsafe {
            libc::socket(
                libc::AF_INET,
                libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                libc::IPPROTO_TCP,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("reserve E3 gateway socket");
        }
        let socket = unsafe { OwnedFd::from_raw_fd(raw) };
        let address = libc::sockaddr_in {
            sin_family: libc::AF_INET as libc::sa_family_t,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: u32::from_ne_bytes([127, 0, 0, 1]),
            },
            sin_zero: [0; 8],
        };
        // SAFETY: address points to a complete IPv4 sockaddr for the duration of bind.
        if unsafe {
            libc::bind(
                socket.as_raw_fd(),
                (&address as *const libc::sockaddr_in).cast(),
                std::mem::size_of_val(&address) as libc::socklen_t,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error()).context("bind E3 gateway socket");
        }
        let mut observed: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut length = std::mem::size_of_val(&observed) as libc::socklen_t;
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: both outputs are writable allocations of the lengths supplied to the kernel.
        anyhow::ensure!(
            unsafe {
                libc::getsockname(
                    socket.as_raw_fd(),
                    (&mut observed as *mut libc::sockaddr_in).cast(),
                    &mut length,
                )
            } == 0
                && length as usize == std::mem::size_of_val(&observed)
                && observed.sin_family == libc::AF_INET as libc::sa_family_t
                && observed.sin_addr.s_addr == address.sin_addr.s_addr
                && observed.sin_port != 0
                && unsafe { libc::fstat(socket.as_raw_fd(), &mut stat) } == 0
                && stat.st_mode & libc::S_IFMT == libc::S_IFSOCK
                && stat.st_ino != 0
                && std::fs::metadata("/proc/self/ns/net")?.ino() == namespace_inode,
            "E3 bound listener identity mismatch"
        );
        self.bound_listener = Some((
            u16::from_be(observed.sin_port),
            stat.st_ino,
            namespace_inode,
        ));
        self.network_namespace = Some(namespace);
        self.listener = Some(socket);
        self.validate_listener_inventory()
    }

    /// Called only inside the registry's intent/readback transaction while this owner retains
    /// exclusive admission. No helper process or caller-selected filesystem root is involved.
    pub(crate) fn install_dormant_boundary(
        &mut self,
        intent: &config_projection::E3KernelEffectIntentV1,
        reference: &config_projection::E3KernelEffectIntentRefV1,
        posture: config_projection::GatewayAccessPostureV1,
        remove_after_revoke: bool,
    ) -> Result<Option<config_projection::E3ChildCgroupRegistrationV1>> {
        use config_projection::{
            CanonicalCgroupIdentityV1, E3ChildCgroupRegistrationV1, E3KernelEffectKindV1,
        };
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::MetadataExt;

        anyhow::ensure!(
            !self.revoked
                && reference.authority_store_id == intent.authority_store_id
                && reference.effect_intent_id == intent.effect_intent_id
                && reference.intent_hash == intent.intent_hash
                && self.pending_cgroup.is_none(),
            "E3 kernel effect has the wrong intent or an unresolved earlier effect"
        );
        if let Some(existing) = self
            .unresolved_intents
            .iter()
            .find(|existing| existing.effect_intent_id == intent.effect_intent_id)
        {
            anyhow::ensure!(existing == intent, "E3 owned kernel intent changed");
        } else {
            anyhow::ensure!(
                posture == config_projection::GatewayAccessPostureV1::DenyAllDormant
                    && self.unresolved_intents.len() < 4,
                "E3 cannot adopt an unowned intent during revocation"
            );
            // The registry has already made this intent durable before calling us. Retain it
            // even if the very first syscall fails, or an existing fixed name collides.
            self.unresolved_intents.push(intent.clone());
        }
        if posture != config_projection::GatewayAccessPostureV1::Revoked {
            self.validate_listener_inventory()?;
        }
        if let Some(namespace) = &self.network_namespace {
            anyhow::ensure!(
                std::fs::metadata("/proc/self/ns/net")?.ino() == namespace.metadata()?.ino(),
                "E3 kernel operation left its held network namespace"
            );
        }
        anyhow::ensure!(
            !remove_after_revoke || posture == config_projection::GatewayAccessPostureV1::Revoked,
            "E3 boundary removal requires revocation"
        );
        match &intent.effect {
            E3KernelEffectKindV1::CreateChildCgroup {
                cgroup_registration_id,
                role,
                parent_cgroup,
                child_component,
                expected_relative_path,
            } => {
                anyhow::ensure!(
                    posture == config_projection::GatewayAccessPostureV1::DenyAllDormant
                        && self.boundary.is_none()
                        && self.pending_boundary.is_none(),
                    "E3 cgroups can only be created before the dormant boundary"
                );
                anyhow::ensure!(
                    self.cgroups.len() < 3
                        && !self
                            .cgroups
                            .iter()
                            .any(|(_, _, registration, _)| registration.role == *role),
                    "E3 cgroup role is already reserved"
                );
                anyhow::ensure!(
                    child_component
                        == &format!(
                            "substrate-e3-{}",
                            &format!("{:x}", Sha256::digest(cgroup_registration_id.as_bytes()))
                                [..24]
                        )
                        && parent_cgroup.cgroup_relative_path.starts_with("substrate/")
                        && parent_cgroup
                            .cgroup_relative_path
                            .split('/')
                            .all(|part| !part.is_empty() && part != "." && part != "..")
                        && expected_relative_path
                            == &format!(
                                "{}/{}",
                                parent_cgroup.cgroup_relative_path, child_component
                            ),
                    "E3 cgroup name does not match its fixed intent"
                );
                for (_, _, registration, previous) in &self.cgroups {
                    let E3KernelEffectKindV1::CreateChildCgroup {
                        parent_cgroup: previous_parent,
                        ..
                    } = &previous.effect
                    else {
                        anyhow::bail!("E3 retained cgroup intent changed")
                    };
                    anyhow::ensure!(
                        previous_parent == parent_cgroup
                            && previous.authority_store_id == intent.authority_store_id
                            && previous.series_id == intent.series_id
                            && previous.preparation_id == intent.preparation_id
                            && previous.fence_id == intent.fence_id
                            && registration.cgroup_registration_id != *cgroup_registration_id,
                        "E3 cgroup attempts do not share an exact parent and preparation"
                    );
                }
                let mount = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW)
                    .open("/sys/fs/cgroup")?;
                let mount_metadata = mount.metadata()?;
                let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
                anyhow::ensure!(
                    unsafe { libc::fstatfs(mount.as_raw_fd(), &mut filesystem) } == 0
                        && filesystem.f_type as u64 == 0x6367_7270
                        && mount_metadata.dev() == parent_cgroup.cgroup_v2_mount_device_id
                        && mount_metadata.ino() == parent_cgroup.cgroup_v2_mount_inode,
                    "E3 cgroup v2 mount identity changed"
                );
                let parent_name =
                    std::ffi::CString::new(parent_cgroup.cgroup_relative_path.as_str())?;
                let mut how: libc::open_how = unsafe { std::mem::zeroed() };
                how.flags =
                    (libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                        as u64;
                how.resolve = libc::RESOLVE_BENEATH
                    | libc::RESOLVE_NO_MAGICLINKS
                    | libc::RESOLVE_NO_SYMLINKS
                    | libc::RESOLVE_NO_XDEV;
                let raw = unsafe {
                    libc::syscall(
                        libc::SYS_openat2,
                        mount.as_raw_fd(),
                        parent_name.as_ptr(),
                        &how,
                        std::mem::size_of_val(&how),
                    )
                };
                if raw < 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("open exact E3 parent cgroup");
                }
                let parent = unsafe { std::fs::File::from_raw_fd(raw as i32) };
                let metadata = parent.metadata()?;
                anyhow::ensure!(
                    metadata.dev() == mount_metadata.dev()
                        && metadata.ino() == parent_cgroup.cgroup_directory_inode
                        && metadata.uid() == 0
                        && metadata.mode() & 0o022 == 0,
                    "E3 parent cgroup identity or control ownership changed"
                );
                let component = std::ffi::CString::new(child_component.as_str())?;
                // Retain the complete recovery coordinate before the first kernel mutation.
                // On EEXIST it is cleared: this attempt never owns an existing same-name object.
                self.pending_cgroup = Some((parent, component, intent.clone()));
                let (parent, component, _) = self.pending_cgroup.as_ref().unwrap();
                if unsafe { libc::mkdirat(parent.as_raw_fd(), component.as_ptr(), 0o700) } != 0 {
                    let error = std::io::Error::last_os_error();
                    self.pending_cgroup = None;
                    return Err(error).context("create exclusive E3 cgroup");
                }
                let raw = unsafe {
                    libc::syscall(
                        libc::SYS_openat2,
                        parent.as_raw_fd(),
                        component.as_ptr(),
                        &how,
                        std::mem::size_of_val(&how),
                    )
                };
                if raw < 0 {
                    return Err(std::io::Error::last_os_error()).context("open created E3 cgroup");
                }
                let child = unsafe { std::fs::File::from_raw_fd(raw as i32) };
                let metadata = child.metadata()?;
                anyhow::ensure!(
                    metadata.uid() == 0
                        && metadata.gid() == 0
                        && metadata.mode() & 0o7777 == 0o700
                        && metadata.dev() == mount_metadata.dev()
                        && metadata.ino() != parent_cgroup.cgroup_directory_inode,
                    "E3 created cgroup control ownership mismatch"
                );
                for control in [c"cgroup.procs", c"cgroup.threads", c"cgroup.events"] {
                    let raw = unsafe {
                        libc::openat(
                            child.as_raw_fd(),
                            control.as_ptr(),
                            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                        )
                    };
                    if raw < 0 {
                        return Err(std::io::Error::last_os_error())
                            .context("open E3 cgroup control readback");
                    }
                    let mut file = unsafe { std::fs::File::from_raw_fd(raw) };
                    let meta = file.metadata()?;
                    anyhow::ensure!(
                        meta.is_file()
                            && meta.uid() == 0
                            && meta.mode() & 0o022 == 0
                            && meta.dev() == metadata.dev(),
                        "E3 cgroup control file ownership mismatch"
                    );
                    let mut bytes = String::new();
                    std::io::Read::read_to_string(
                        &mut std::io::Read::take(&mut file, 4096),
                        &mut bytes,
                    )?;
                    anyhow::ensure!(
                        bytes.len() < 4096
                            && if control == c"cgroup.events" {
                                bytes
                                    .lines()
                                    .filter(|line| line.starts_with("populated "))
                                    .collect::<Vec<_>>()
                                    == ["populated 0"]
                            } else {
                                bytes.is_empty()
                            },
                        "E3 new cgroup is not empty"
                    );
                }
                let mut registration = E3ChildCgroupRegistrationV1 {
                    schema_version: 1,
                    authority_store_id: intent.authority_store_id.clone(),
                    series_id: intent.series_id.clone(),
                    cgroup_registration_id: cgroup_registration_id.clone(),
                    kernel_effect_intent_ref: reference.clone(),
                    fence_id: intent.fence_id.clone(),
                    turn_id: None,
                    role: *role,
                    cgroup: CanonicalCgroupIdentityV1 {
                        cgroup_v2_mount_device_id: mount_metadata.dev(),
                        cgroup_v2_mount_inode: mount_metadata.ino(),
                        cgroup_directory_inode: metadata.ino(),
                        cgroup_relative_path: expected_relative_path.clone(),
                    },
                    kernel_boot_id: std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?
                        .trim()
                        .to_string(),
                    registered_at: config_projection::Timestamp(
                        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                    ),
                    cgroup_registration_hash: String::new(),
                };
                let mut value = serde_json::to_value(&registration)?;
                value
                    .as_object_mut()
                    .context("E3 cgroup registration is not an object")?
                    .remove("cgroup_registration_hash");
                registration.cgroup_registration_hash =
                    config_projection::ConfigProjectionCodecV1::domain_sha256(
                        "substrate.e3.child-cgroup-registration.v1",
                        &serde_json::json!({"cgroup_registration":value}),
                    )?;
                let (parent, _, owned_intent) = self.pending_cgroup.take().unwrap();
                self.cgroups
                    .push((parent, child, registration.clone(), owned_intent));
                Ok(Some(registration))
            }
            E3KernelEffectKindV1::InstallGatewayBoundary {
                access_boundary_id,
                network_namespace_inode,
                table_name,
                chain_name,
            } => {
                use config_projection::{
                    GatewayAccessPostureV1 as Posture, NftablesRuleRoleV1 as Role,
                };
                use std::collections::BTreeSet;

                // These codecs are confined to this fixed E3 inet/output boundary. They expose
                // neither arbitrary commands nor a general netfilter/security API.
                let (port, _, namespace_inode) = self
                    .bound_listener
                    .context("E3 listener binding is absent")?;
                anyhow::ensure!(
                    *network_namespace_inode == namespace_inode
                        && table_name
                            == &format!(
                                "substrate_e3_{}",
                                &format!("{:x}", Sha256::digest(access_boundary_id.as_bytes()))
                                    [..24]
                            )
                        && chain_name == "gateway_output"
                        && self.cgroups.len() == 3,
                    "E3 boundary intent does not bind the held listener and cgroups"
                );
                for (_, _, _, cgroup_intent) in &self.cgroups {
                    anyhow::ensure!(
                        cgroup_intent.authority_store_id == intent.authority_store_id
                            && cgroup_intent.series_id == intent.series_id
                            && cgroup_intent.preparation_id == intent.preparation_id
                            && cgroup_intent.fence_id == intent.fence_id,
                        "E3 boundary and cgroup attempts differ"
                    );
                }
                let readiness = &self
                    .cgroups
                    .iter()
                    .find(|(_, _, r, _)| {
                        r.role == config_projection::E3TerminalProcessRoleV1::ReadinessProbe
                    })
                    .context("E3 readiness cgroup missing")?
                    .2
                    .cgroup;
                let member = &self
                    .cgroups
                    .iter()
                    .find(|(_, _, r, _)| {
                        r.role == config_projection::E3TerminalProcessRoleV1::Codex
                    })
                    .context("E3 member cgroup missing")?
                    .2
                    .cgroup;
                let rule = |role: Role| -> Result<(Vec<u8>, String)> {
                    let mut wire = [
                        expression("meta", &[number(1, 1), number(2, 15)].concat()),
                        compare(&[2]),
                        expression("meta", &[number(1, 1), number(2, 16)].concat()),
                        compare(&[6]),
                        expression(
                            "payload",
                            &[number(1, 1), number(2, 1), number(3, 16), number(4, 4)].concat(),
                        ),
                        compare(&[127, 0, 0, 1]),
                        expression(
                            "payload",
                            &[number(1, 1), number(2, 2), number(3, 2), number(4, 2)].concat(),
                        ),
                        compare(&port.to_be_bytes()),
                    ]
                    .concat();
                    let mut json = vec![
                        serde_json::json!({"match":{"op":"==","left":{"meta":{"key":"nfproto"}},"right":"ipv4"}}),
                        serde_json::json!({"match":{"op":"==","left":{"meta":{"key":"l4proto"}},"right":"tcp"}}),
                        serde_json::json!({"match":{"op":"==","left":{"payload":{"protocol":"ip","field":"daddr"}},"right":"127.0.0.1"}}),
                        serde_json::json!({"match":{"op":"==","left":{"payload":{"protocol":"tcp","field":"dport"}},"right":port}}),
                    ];
                    if role == Role::RejectRemainder {
                        wire.extend(expression("reject", &number(1, 1)));
                        json.push(serde_json::json!({"reject":{"type":"tcp reset"}}));
                    } else {
                        let cgroup = if role == Role::ReadinessProbeAccept {
                            readiness
                        } else {
                            member
                        };
                        let level = u32::try_from(cgroup.cgroup_relative_path.split('/').count())?;
                        wire.extend(expression(
                            "socket",
                            &[number(1, 3), number(2, 1), number(3, level)].concat(),
                        ));
                        wire.extend(compare(&cgroup.cgroup_directory_inode.to_ne_bytes()));
                        wire.extend(expression(
                            "immediate",
                            &[
                                number(1, 0),
                                attribute(0x8002, &attribute(0x8002, &number(1, 1))),
                            ]
                            .concat(),
                        ));
                        json.push(serde_json::json!({"match":{"op":"==","left":{"socket":{"key":"cgroupv2","level":level}},"right":cgroup.cgroup_directory_inode}}));
                        json.push(serde_json::json!({"accept":null}));
                    }
                    Ok((
                        wire,
                        String::from_utf8(
                            config_projection::ConfigProjectionCodecV1::encode_canonical_json(
                                &json,
                            )?,
                        )?,
                    ))
                };
                let roles = |state| match state {
                    Posture::DenyAllDormant => {
                        vec![Role::ReadinessProbeAccept, Role::RejectRemainder]
                    }
                    Posture::AllowExactMember => vec![
                        Role::ReadinessProbeAccept,
                        Role::ExactMemberAccept,
                        Role::RejectRemainder,
                    ],
                    Posture::Revoked => vec![Role::RejectRemainder],
                };
                let raw = unsafe {
                    libc::socket(
                        libc::AF_NETLINK,
                        libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                        libc::NETLINK_NETFILTER,
                    )
                };
                if raw < 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("open E3 netfilter socket");
                }
                let netlink = unsafe { OwnedFd::from_raw_fd(raw) };
                let mut local: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
                local.nl_family = libc::AF_NETLINK as u16;
                anyhow::ensure!(
                    unsafe {
                        libc::bind(
                            raw,
                            (&local as *const libc::sockaddr_nl).cast(),
                            std::mem::size_of_val(&local) as libc::socklen_t,
                        )
                    } == 0,
                    "bind E3 netfilter socket"
                );
                let mut sequence = 10_u32;
                let mut request = |kind: u16,
                                   family: u8,
                                   payload: &[u8],
                                   dump: bool|
                 -> Result<Vec<(u16, Vec<u8>)>> {
                    sequence += 1;
                    exchange(
                        netlink.as_raw_fd(),
                        &message(
                            0x0a00 | kind,
                            if dump { 0x301 } else { 5 },
                            sequence,
                            family,
                            payload,
                        ),
                        &[sequence],
                        dump,
                    )
                };
                let name = string(1, table_name);
                let chain_query = [string(1, table_name), string(3, chain_name)].concat();
                let rules_query = [string(1, table_name), string(2, chain_name)].concat();
                let generation = |response: Vec<(u16, Vec<u8>)>| -> Result<u32> {
                    anyhow::ensure!(
                        response.len() == 1 && response[0].0 == 0x0a0f,
                        "E3 nftables generation response is not unique"
                    );
                    let fields = map(&response[0].1[4..])?;
                    Ok(u32::from_be_bytes(
                        (*fields
                            .get(&1)
                            .context("E3 nftables generation is missing")?)
                        .try_into()?,
                    ))
                };
                let mut observe = |state: Posture| -> Result<(
                    u64,
                    config_projection::NftablesChainIdentityV1,
                    Vec<config_projection::NftablesRuleIdentityV1>,
                )> {
                    let before = generation(request(16, 0, &[], false)?)?;
                    let table = request(1, 1, &name, false)?;
                    anyhow::ensure!(
                        table.len() == 1 && table[0].0 == 0x0a00 && table[0].1[0] == 1,
                        "E3 table readback is not unique"
                    );
                    let fields = map(&table[0].1[4..])?;
                    anyhow::ensure!(
                        fields.get(&1) == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                            && fields.get(&2) == Some(&0_u32.to_be_bytes().as_slice())
                            && !fields.contains_key(&6)
                            && !fields.contains_key(&7),
                        "E3 table flags or identity changed"
                    );
                    let table_handle = u64::from_be_bytes(
                        (*fields.get(&4).context("E3 table handle missing")?).try_into()?,
                    );
                    anyhow::ensure!(table_handle != 0, "E3 table handle is zero");
                    let chain = request(4, 1, &chain_query, false)?;
                    anyhow::ensure!(
                        chain.len() == 1 && chain[0].0 == 0x0a03 && chain[0].1[0] == 1,
                        "E3 chain readback is not unique"
                    );
                    let fields = map(&chain[0].1[4..])?;
                    let hook = map(fields.get(&4).context("E3 chain hook missing")?)?;
                    anyhow::ensure!(
                        fields.get(&1) == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                            && fields.get(&3) == Some(&b"gateway_output\0".as_slice())
                            && fields.get(&5) == Some(&1_u32.to_be_bytes().as_slice())
                            && fields.get(&7) == Some(&b"filter\0".as_slice())
                            && hook.len() == 2
                            && hook.get(&1) == Some(&3_u32.to_be_bytes().as_slice())
                            && hook.get(&2) == Some(&(-100_i32).to_be_bytes().as_slice()),
                        "E3 chain hook, priority or policy changed"
                    );
                    let chain_handle = u64::from_be_bytes(
                        (*fields.get(&2).context("E3 chain handle missing")?).try_into()?,
                    );
                    anyhow::ensure!(chain_handle != 0, "E3 chain handle is zero");
                    let rules = request(7, 1, &rules_query, true)?;
                    let expected_roles = roles(state);
                    anyhow::ensure!(
                        rules.len() == expected_roles.len(),
                        "E3 ordered rule inventory changed"
                    );
                    let mut observed = Vec::with_capacity(rules.len());
                    let mut handles = BTreeSet::new();
                    for ((kind, bytes), role) in rules.iter().zip(expected_roles) {
                        anyhow::ensure!(
                            *kind == 0x0a06 && bytes[0] == 1,
                            "E3 rule response kind changed"
                        );
                        let fields = map(&bytes[4..])?;
                        let (expected_wire, canonical_expression) = rule(role)?;
                        anyhow::ensure!(
                            fields.get(&1)
                                == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                                && fields.get(&2) == Some(&b"gateway_output\0".as_slice())
                                && matches_expression(
                                    fields.get(&4).context("E3 rule expressions missing")?,
                                    &expected_wire,
                                    0
                                )?,
                            "E3 rule expression or chain changed"
                        );
                        let handle = u64::from_be_bytes(
                            (*fields.get(&3).context("E3 rule handle missing")?).try_into()?,
                        );
                        anyhow::ensure!(
                            handle != 0 && handles.insert(handle),
                            "E3 rule handle is zero or repeated"
                        );
                        observed.push(config_projection::NftablesRuleIdentityV1 {
                            role,
                            family: "inet".to_string(),
                            table: table_name.clone(),
                            chain: chain_name.clone(),
                            rule_handle: handle,
                            canonical_expression,
                        });
                    }
                    anyhow::ensure!(
                        before == generation(request(16, 0, &[], false)?)?,
                        "E3 nftables generation changed across exact boundary readback"
                    );
                    Ok((
                        table_handle,
                        config_projection::NftablesChainIdentityV1 {
                            family: "inet".to_string(),
                            table: table_name.clone(),
                            chain: chain_name.clone(),
                            chain_handle,
                            chain_type: "filter".to_string(),
                            hook: "output".to_string(),
                            priority: -100,
                            policy: "accept".to_string(),
                        },
                        observed,
                    ))
                };
                let previous = self.boundary.as_ref();
                if self.boundary_deleted {
                    anyhow::ensure!(
                        posture == Posture::Revoked,
                        "a removed E3 boundary cannot be reopened"
                    );
                    let absent = request(1, 1, &name, false);
                    anyhow::ensure!(
                        absent
                            .as_ref()
                            .err()
                            .and_then(|error| error.downcast_ref::<std::io::Error>())
                            .and_then(|error| error.raw_os_error())
                            == Some(libc::ENOENT),
                        "E3 removed boundary is no longer absent"
                    );
                    return Ok(None);
                }
                if let Some((owned_intent, table, chain, rules, state)) = previous {
                    anyhow::ensure!(owned_intent == intent, "E3 boundary intent changed");
                    let observed = observe(*state)?;
                    anyhow::ensure!(
                        observed == (*table, chain.clone(), rules.clone()),
                        "E3 boundary handles or rules changed"
                    );
                    if *state == posture {
                        if remove_after_revoke {
                            anyhow::ensure!(
                                self.listener.is_none(),
                                "E3 removal requires a closed listener"
                            );
                            for (parent, child, registration, cgroup_intent) in &self.cgroups {
                                let E3KernelEffectKindV1::CreateChildCgroup {
                                    child_component, ..
                                } = &cgroup_intent.effect
                                else {
                                    anyhow::bail!("E3 cleanup cgroup kind changed")
                                };
                                let component = std::ffi::CString::new(child_component.as_str())?;
                                let metadata = child.metadata()?;
                                let mut found: libc::stat = unsafe { std::mem::zeroed() };
                                anyhow::ensure!(
                                    metadata.dev() == registration.cgroup.cgroup_v2_mount_device_id
                                        && metadata.ino()
                                            == registration.cgroup.cgroup_directory_inode
                                        && unsafe {
                                            libc::fstatat(
                                                parent.as_raw_fd(),
                                                component.as_ptr(),
                                                &mut found,
                                                libc::AT_SYMLINK_NOFOLLOW,
                                            )
                                        } == -1
                                        && std::io::Error::last_os_error().raw_os_error()
                                            == Some(libc::ENOENT),
                                    "E3 removal requires each exact cgroup name to be absent"
                                );
                            }
                            let delete = [
                                message(0x10, 1, 100, 0, &[]),
                                message(0x0a02, 5, 101, 1, &attribute(4, &table.to_be_bytes())),
                                message(0x11, 1, 102, 0, &[]),
                            ]
                            .concat();
                            exchange(netlink.as_raw_fd(), &delete, &[101], false)?;
                            let absent = request(1, 1, &name, false);
                            anyhow::ensure!(
                                absent
                                    .as_ref()
                                    .err()
                                    .and_then(|e| e.downcast_ref::<std::io::Error>())
                                    .and_then(|e| e.raw_os_error())
                                    == Some(libc::ENOENT),
                                "E3 deleted boundary was not observed absent"
                            );
                            self.boundary_deleted = true;
                        }
                        return Ok(None);
                    }
                    anyhow::ensure!(
                        (*state == Posture::DenyAllDormant
                            && matches!(posture, Posture::AllowExactMember | Posture::Revoked))
                            || (*state == Posture::AllowExactMember && posture == Posture::Revoked),
                        "illegal E3 boundary successor"
                    );
                } else {
                    anyhow::ensure!(
                        posture == Posture::DenyAllDormant && self.pending_boundary.is_none(),
                        "E3 boundary must begin dormant"
                    );
                }
                let mut batch = message(0x10, 1, 1, 0, &[]);
                let mut acknowledgements = Vec::new();
                let mut append = |kind: u16, flags: u16, payload: Vec<u8>| {
                    let sequence = 2 + acknowledgements.len() as u32;
                    batch.extend(message(0x0a00 | kind, flags, sequence, 1, &payload));
                    acknowledgements.push(sequence);
                };
                if let Some((_, _, _, current_rules, _)) = previous {
                    if posture == Posture::Revoked {
                        for current in current_rules
                            .iter()
                            .filter(|entry| entry.role != Role::RejectRemainder)
                        {
                            append(
                                8,
                                5,
                                [
                                    rules_query.clone(),
                                    attribute(3, &current.rule_handle.to_be_bytes()),
                                ]
                                .concat(),
                            );
                        }
                    } else {
                        append(
                            6,
                            0xc05,
                            [
                                rules_query.clone(),
                                attribute(6, &current_rules[0].rule_handle.to_be_bytes()),
                                attribute(0x8004, &rule(Role::ExactMemberAccept)?.0),
                            ]
                            .concat(),
                        );
                    }
                } else {
                    append(0, 0x605, name.clone());
                    append(
                        3,
                        0x605,
                        [
                            chain_query.clone(),
                            attribute(
                                0x8004,
                                &[number(1, 3), number(2, (-100_i32) as u32)].concat(),
                            ),
                            number(5, 1),
                            string(7, "filter"),
                        ]
                        .concat(),
                    );
                    for role in roles(Posture::DenyAllDormant) {
                        append(
                            6,
                            0xc05,
                            [rules_query.clone(), attribute(0x8004, &rule(role)?.0)].concat(),
                        );
                    }
                }
                batch.extend(message(0x11, 1, 9, 0, &[]));
                self.pending_boundary = Some(intent.clone());
                if let Err(error) = exchange(netlink.as_raw_fd(), &batch, &acknowledgements, false)
                {
                    // A negative kernel ACK aborts the atomic batch. A timeout or malformed
                    // response has no such proof and keeps the unresolved intent owned.
                    if error.downcast_ref::<std::io::Error>().is_some() {
                        self.pending_boundary = None;
                    }
                    return Err(error);
                }
                let (table, chain, rules) = observe(posture)?;
                if let Some((_, old_table, old_chain, old_rules, _)) = previous {
                    anyhow::ensure!(
                        table == *old_table
                            && chain == *old_chain
                            && rules
                                .iter()
                                .filter(|r| r.role != Role::ExactMemberAccept)
                                .all(|r| old_rules.contains(r)),
                        "E3 boundary successor replaced an existing handle"
                    );
                }
                self.boundary = Some((intent.clone(), table, chain, rules, posture));
                self.pending_boundary = None;
                Ok(None)
            }
        }
    }

    pub(crate) fn listen_after_dormant_boundary(
        &mut self,
        gateway: &config_projection::InWorldGatewayIdentityV1,
        series_id: &str,
        fence_id: &str,
        owner_uid: u64,
        owner_gid: u64,
    ) -> Result<(
        config_projection::GatewayAccessBoundaryV1,
        [config_projection::E3ChildCgroupRegistrationV1; 3],
        config_projection::GatewayRuntimeConfigIdentityV1,
    )> {
        use config_projection::{
            E3KernelEffectKindV1, E3TerminalProcessRoleV1, GatewayAccessPostureV1,
        };
        anyhow::ensure!(
            !self.listening
                && self.gateway_config.is_none()
                && !self.boundary_deleted
                && self.pending_boundary.is_none()
                && self.pending_cgroup.is_none(),
            "E3 listener cannot be listened twice or over an unresolved effect"
        );
        let (intent, _, _, _, posture) = self
            .boundary
            .clone()
            .context("E3 dormant deny boundary is absent")?;
        let E3KernelEffectKindV1::InstallGatewayBoundary {
            access_boundary_id, ..
        } = &intent.effect
        else {
            anyhow::bail!("E3 boundary intent kind changed")
        };
        anyhow::ensure!(posture == GatewayAccessPostureV1::DenyAllDormant && gateway.authority_store_id == intent.authority_store_id && gateway.access_boundary_id == *access_boundary_id && series_id == intent.series_id && fence_id == intent.fence_id && self.cgroups.iter().all(|(_, _, _, intent)| matches!(&intent.effect, E3KernelEffectKindV1::CreateChildCgroup { parent_cgroup, .. } if parent_cgroup.cgroup_relative_path == format!("substrate/{}", gateway.world_id))), "E3 listener gateway binding changed");
        let reference = config_projection::E3KernelEffectIntentRefV1 {
            authority_store_id: intent.authority_store_id.clone(),
            effect_intent_id: intent.effect_intent_id.clone(),
            intent_hash: intent.intent_hash.clone(),
        };
        self.install_dormant_boundary(
            &intent,
            &reference,
            GatewayAccessPostureV1::DenyAllDormant,
            false,
        )?;
        for (_, child, _, _) in &self.cgroups {
            let raw = unsafe {
                libc::openat(
                    child.as_raw_fd(),
                    c"cgroup.events".as_ptr(),
                    libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            if raw < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("read E3 cgroup before listen");
            }
            let file = unsafe { std::fs::File::from_raw_fd(raw) };
            let mut state = String::new();
            std::io::Read::read_to_string(&mut std::io::Read::take(file, 4096), &mut state)?;
            anyhow::ensure!(
                state.len() < 4096
                    && state
                        .lines()
                        .filter(|line| line.starts_with("populated "))
                        .collect::<Vec<_>>()
                        == ["populated 0"],
                "E3 cgroup populated before listen"
            );
        }
        let fd = self
            .listener
            .as_ref()
            .context("E3 listener is absent")?
            .as_raw_fd();
        if unsafe { libc::listen(fd, 16) } != 0 {
            return Err(std::io::Error::last_os_error()).context("listen on E3 deny-bound socket");
        }
        self.listening = true;
        self.validate_listener_inventory()?;
        let mut info: libc::tcp_info = unsafe { std::mem::zeroed() };
        let mut length = std::mem::size_of_val(&info) as libc::socklen_t;
        anyhow::ensure!(
            unsafe {
                libc::getsockopt(
                    fd,
                    libc::IPPROTO_TCP,
                    libc::TCP_INFO,
                    (&mut info as *mut libc::tcp_info).cast(),
                    &mut length,
                )
            } == 0
                && length > 0
                && info.tcpi_state == 10,
            "E3 accepting socket is not TCP LISTEN"
        );
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let zero = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        anyhow::ensure!(
            unsafe { libc::ppoll(&mut poll, 1, &zero, std::ptr::null()) } == 0 && poll.revents == 0,
            "E3 pre-gateway listener queue is not empty"
        );
        let (port, socket_inode, network_namespace_inode) = self
            .bound_listener
            .context("E3 listener identity disappeared")?;
        let (_, _, chain, rules, _) = self.boundary.as_ref().context("E3 boundary disappeared")?;
        let mut boundary = config_projection::GatewayAccessBoundaryV1 {
            schema_version: 1,
            authority_store_id: gateway.authority_store_id.clone(),
            kernel_effect_intent_ref: reference,
            access_boundary_id: gateway.access_boundary_id.clone(),
            gateway_instance_id: gateway.gateway_instance_id.clone(),
            config_projection_identity_hash: gateway.config_projection_identity_hash.clone(),
            orchestration_session_id: gateway.orchestration_session_id.clone(),
            retained_participant_id: gateway.retained_participant_id.clone(),
            backend_id: gateway.backend_id.clone(),
            world_id: gateway.world_id.clone(),
            world_generation: gateway.world_generation,
            gateway_listener: config_projection::GatewayListenerIdentityV1 {
                transport: "tcp".to_string(),
                network_namespace_inode,
                address: "127.0.0.1".to_string(),
                port,
                socket_inode,
                listen_backlog: 16,
                deny_boundary_effective_before_listen: true,
                responses_base_path: "/v1".to_string(),
            },
            readiness_probe_cgroup: self
                .cgroups
                .iter()
                .find(|(_, _, r, _)| r.role == E3TerminalProcessRoleV1::ReadinessProbe)
                .context("E3 readiness cgroup missing")?
                .2
                .cgroup
                .clone(),
            allowed_member_cgroup: self
                .cgroups
                .iter()
                .find(|(_, _, r, _)| r.role == E3TerminalProcessRoleV1::Codex)
                .context("E3 member cgroup missing")?
                .2
                .cgroup
                .clone(),
            nftables_chain: chain.clone(),
            nftables_rules: rules.clone(),
            posture: GatewayAccessPostureV1::DenyAllDormant,
            revision: 1,
            predecessor_ref: None,
            boundary_hash: String::new(),
        };
        let mut value = serde_json::to_value(&boundary)?;
        value
            .as_object_mut()
            .context("E3 boundary is not an object")?
            .remove("boundary_hash");
        boundary.boundary_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.gateway-access-boundary.v1",
            &serde_json::json!({"boundary":value}),
        )?;
        let registrations = [
            E3TerminalProcessRoleV1::ManagedGateway,
            E3TerminalProcessRoleV1::ReadinessProbe,
            E3TerminalProcessRoleV1::Codex,
        ]
        .map(|role| {
            self.cgroups
                .iter()
                .find(|(_, _, r, _)| r.role == role)
                .map(|(_, _, r, _)| r.clone())
        });
        let [Some(gateway_registration), Some(readiness), Some(codex)] = registrations else {
            anyhow::bail!("E3 ordered registrations disappeared");
        };
        // Build only the fixed, nonsecret cli:codex-world configuration. The parent is never
        // caller-selected, and every path component is opened without following links.
        use sha2::{Digest, Sha256};
        use std::os::fd::AsFd;
        use std::os::unix::fs::MetadataExt;
        for (id, prefix) in [(series_id, "cps_"), (fence_id, "cpf_")] {
            let suffix = id
                .strip_prefix(prefix)
                .context("invalid E3 config coordinate")?;
            let parsed = uuid::Uuid::parse_str(suffix)?;
            anyhow::ensure!(
                parsed.get_version_num() == 7 && parsed.to_string() == suffix,
                "invalid E3 config coordinate"
            );
        }
        let uid = u32::try_from(owner_uid)?;
        let gid = u32::try_from(owner_gid)?;
        let mut parent = fs::File::open("/")?;
        for component in ["run", "substrate", "e3-gateway", series_id] {
            let name = std::ffi::CString::new(component)?;
            if component == "e3-gateway" || component == series_id {
                let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o711) };
                anyhow::ensure!(
                    result == 0
                        || std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST),
                    "create E3 config parent: {}",
                    std::io::Error::last_os_error()
                );
            }
            let raw = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if raw < 0 {
                return Err(std::io::Error::last_os_error()).context("open E3 config parent");
            }
            let next = unsafe { fs::File::from_raw_fd(raw) };
            let metadata = next.metadata()?;
            anyhow::ensure!(
                metadata.uid() == 0 && metadata.mode() & 0o022 == 0,
                "E3 config ancestor is not protected"
            );
            if component == "e3-gateway" || component == series_id {
                anyhow::ensure!(
                    metadata.mode() & 0o7777 == 0o711,
                    "E3 config parent mode changed"
                );
            }
            parent.sync_all()?;
            parent = next;
        }
        let temporary = std::ffi::CString::new(format!(
            ".e3-gateway-realization-tmp.{}",
            uuid::Uuid::now_v7()
        ))?;
        let binding = resolve_gateway_backend_binding("cli:codex-world")
            .context("E3 gateway binding missing")?;
        let bytes = render_integrated_config(port, binding).into_bytes();
        if unsafe { libc::mkdirat(parent.as_raw_fd(), temporary.as_ptr(), 0o700) } != 0 {
            return Err(std::io::Error::last_os_error()).context("create E3 config temporary");
        }
        self.gateway_config = Some((parent, temporary, None, None, None));
        self.gateway_config_owner = Some((uid, gid));
        self.gateway_config_bytes = bytes.clone();
        let (parent, component, root, config, identity) = self.gateway_config.as_mut().unwrap();
        let raw = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 config temporary");
        }
        *root = Some(unsafe { fs::File::from_raw_fd(raw) });
        let root = root.as_ref().unwrap();
        // Use O_EXCL before handing the directory to the authenticated target principal.
        let raw = unsafe {
            libc::openat(
                root.as_raw_fd(),
                c"config.toml".as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("create E3 config file");
        }
        *config = Some(unsafe { fs::File::from_raw_fd(raw) });
        let config = config.as_mut().unwrap();
        config.write_all(&bytes)?;
        anyhow::ensure!(
            unsafe { libc::fchown(config.as_raw_fd(), uid, gid) } == 0
                && unsafe { libc::fchmod(config.as_raw_fd(), 0o600) } == 0
                && unsafe { libc::fchown(root.as_raw_fd(), uid, gid) } == 0
                && unsafe { libc::fchmod(root.as_raw_fd(), 0o700) } == 0,
            "set E3 config ownership: {}",
            std::io::Error::last_os_error()
        );
        config.sync_all()?;
        root.sync_all()?;
        use std::os::unix::fs::FileExt;
        let mut observed = vec![0; bytes.len()];
        config.read_exact_at(&mut observed, 0)?;
        let file_meta = config.metadata()?;
        let root_meta = root.metadata()?;
        anyhow::ensure!(
            observed == bytes
                && file_meta.len() == bytes.len() as u64
                && file_meta.is_file()
                && file_meta.nlink() == 1
                && file_meta.uid() == uid
                && file_meta.gid() == gid
                && file_meta.mode() & 0o7777 == 0o600
                && root_meta.uid() == uid
                && root_meta.gid() == gid
                && root_meta.mode() & 0o7777 == 0o700,
            "E3 config readback changed"
        );
        let final_name = std::ffi::CString::new(fence_id)?;
        if unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                parent.as_raw_fd(),
                component.as_ptr(),
                parent.as_raw_fd(),
                final_name.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error())
                .context("install E3 config without replacement");
        }
        *component = final_name;
        parent.sync_all()?;
        // Ownership is already the authenticated user's: bind the installed names back to
        // the held descriptors and re-read the exact bytes after the no-replace publication.
        let root_meta = root.metadata()?;
        let file_meta = config.metadata()?;
        let mut named_root = std::mem::MaybeUninit::<libc::stat>::uninit();
        let mut named_config = std::mem::MaybeUninit::<libc::stat>::uninit();
        anyhow::ensure!(
            unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    named_root.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } == 0
                && unsafe {
                    libc::fstatat(
                        root.as_raw_fd(),
                        c"config.toml".as_ptr(),
                        named_config.as_mut_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } == 0,
            "E3 installed config name disappeared"
        );
        let named_root = unsafe { named_root.assume_init() };
        let named_config = unsafe { named_config.assume_init() };
        let entries = fs::read_dir(format!("/proc/self/fd/{}", root.as_raw_fd()))?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<std::io::Result<Vec<_>>>()?;
        config.read_exact_at(&mut observed, 0)?;
        let second_file_meta = config.metadata()?;
        let second_root_meta = root.metadata()?;
        anyhow::ensure!(
            entries == [std::ffi::OsString::from("config.toml")]
                && (
                    second_file_meta.dev(),
                    second_file_meta.ino(),
                    second_file_meta.mode(),
                    second_file_meta.uid(),
                    second_file_meta.gid(),
                    second_file_meta.nlink(),
                    second_file_meta.len(),
                    second_file_meta.ctime(),
                    second_file_meta.ctime_nsec()
                ) == (
                    file_meta.dev(),
                    file_meta.ino(),
                    file_meta.mode(),
                    file_meta.uid(),
                    file_meta.gid(),
                    file_meta.nlink(),
                    file_meta.len(),
                    file_meta.ctime(),
                    file_meta.ctime_nsec()
                )
                && (
                    second_root_meta.dev(),
                    second_root_meta.ino(),
                    second_root_meta.mode(),
                    second_root_meta.uid(),
                    second_root_meta.gid(),
                    second_root_meta.nlink(),
                    second_root_meta.ctime(),
                    second_root_meta.ctime_nsec()
                ) == (
                    root_meta.dev(),
                    root_meta.ino(),
                    root_meta.mode(),
                    root_meta.uid(),
                    root_meta.gid(),
                    root_meta.nlink(),
                    root_meta.ctime(),
                    root_meta.ctime_nsec()
                )
                && named_root.st_dev == root_meta.dev()
                && named_root.st_ino == root_meta.ino()
                && named_root.st_mode & libc::S_IFMT == libc::S_IFDIR
                && root_meta.uid() == uid
                && root_meta.gid() == gid
                && root_meta.mode() & 0o7777 == 0o700
                && named_config.st_dev == file_meta.dev()
                && named_config.st_ino == file_meta.ino()
                && named_config.st_mode & libc::S_IFMT == libc::S_IFREG
                && named_config.st_nlink == 1
                && file_meta.uid() == uid
                && file_meta.gid() == gid
                && file_meta.mode() & 0o7777 == 0o600
                && file_meta.nlink() == 1
                && file_meta.len() == bytes.len() as u64
                && observed == bytes,
            "E3 installed config readback changed"
        );
        let directory =
            config_projection::CanonicalDirectoryV1::capture_linux_from_fd(root.as_fd())?;
        anyhow::ensure!(
            directory.physical_path == format!("/run/substrate/e3-gateway/{series_id}/{fence_id}"),
            "E3 config root changed"
        );
        let observed_identity = config_projection::GatewayRuntimeConfigIdentityV1 {
            root: directory,
            relative_path: "config.toml".into(),
            mode: 0o600,
            byte_length: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
        };
        *identity = Some(observed_identity.clone());
        Ok((
            boundary,
            [gateway_registration, readiness, codex],
            observed_identity,
        ))
    }

    /// Resolve a child-free attempt without releasing its owner until every kernel object has
    /// been reobserved absent and each original intent has a durable exact resolution.
    pub(crate) fn revoke(
        target: E3GatewayRevocationTargetV1<'_>,
        registry: &config_projection::ConfigProjectionRegistryV1,
    ) -> Result<()> {
        match target {
            E3GatewayRevocationTargetV1::Live(owner) => {
                owner.terminal = true;
                if owner.revoked {
                    return Ok(());
                }
                // Shut every authority-bearing channel before changing network policy. The
                // retained descriptors identify this attempt even when its head has advanced.
                if let Some(launch) = owner.launch.as_mut() {
                    launch.start_gate.take();
                    launch.final_gate.take();
                    launch.auth_writer.take();
                    launch.secret_ready_reader.take();
                    if let Some(probe) = launch.probe.as_deref_mut() {
                        probe.start_gate.take();
                        probe.final_gate.take();
                        probe.probe_start_gate.take();
                        probe.probe_request_gate.take();
                        probe.probe_connected_reader.take();
                        probe.probe_result_reader.take();
                    }
                }
                if let Some((intent, _, _, _, _)) = owner.boundary.clone() {
                    let reference = config_projection::E3KernelEffectIntentRefV1 {
                        authority_store_id: intent.authority_store_id.clone(),
                        effect_intent_id: intent.effect_intent_id.clone(),
                        intent_hash: intent.intent_hash.clone(),
                    };
                    if owner.boundary_removal_attempted && !owner.boundary_deleted {
                        // An interrupted netlink return may follow deletion. Use the existing
                        // absence-only readback; restore the unresolved state if it cannot prove it.
                        owner.boundary_deleted = true;
                        if owner
                            .install_dormant_boundary(
                                &intent,
                                &reference,
                                config_projection::GatewayAccessPostureV1::Revoked,
                                true,
                            )
                            .is_err()
                        {
                            owner.boundary_deleted = false;
                        }
                    }
                    // This operation checks the held namespace and exact table/rule handles,
                    // installs denial and reads it back. Reaping follows verified denial;
                    // the existing resolution transaction later records Revoked durably.
                    owner.install_dormant_boundary(
                        &intent,
                        &reference,
                        config_projection::GatewayAccessPostureV1::Revoked,
                        false,
                    )?;
                }
                if let Some(launch) = owner.launch.as_mut() {
                    let mut deadline = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    anyhow::ensure!(
                        unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut deadline) } == 0,
                        "read E3 cleanup clock"
                    );
                    deadline.tv_sec += 5;
                    let mut children = Vec::new();
                    if let Some(probe) = launch.probe.as_deref_mut() {
                        children.push(probe);
                    }
                    // Split borrowing leaves the gateway owner and optional probe independently held.
                    for child in children {
                        child.start_gate.take();
                        child.final_gate.take();
                        child.auth_writer.take();
                        child.secret_ready_reader.take();
                        child.probe_start_gate.take();
                        child.probe_request_gate.take();
                        child.probe_connected_reader.take();
                        child.probe_result_reader.take();
                        if let Some(pid) = child
                            .gateway_pid
                            .filter(|_| child.gateway_wait_status.is_none())
                        {
                            let result = if let Some(fd) = &child.gateway_pidfd {
                                unsafe {
                                    libc::syscall(
                                        libc::SYS_pidfd_send_signal,
                                        fd.as_raw_fd(),
                                        libc::SIGKILL,
                                        std::ptr::null::<libc::siginfo_t>(),
                                        0,
                                    )
                                }
                            } else {
                                unsafe { libc::kill(pid as i32, libc::SIGKILL) as libc::c_long }
                            };
                            anyhow::ensure!(
                                result == 0
                                    || std::io::Error::last_os_error().raw_os_error()
                                        == Some(libc::ESRCH),
                                "kill exact E3 probe"
                            );
                            loop {
                                ManagedGatewayLaunchCapabilityV1::e3_remaining_ms(&deadline)?;
                                let mut status = 0;
                                let observed = unsafe {
                                    libc::waitpid(pid as i32, &mut status, libc::WNOHANG)
                                };
                                if observed == pid as i32 {
                                    child.gateway_wait_status = Some(status);
                                    break;
                                }
                                anyhow::ensure!(observed == 0, "E3 probe reap identity lost");
                                std::thread::sleep(Duration::from_millis(1));
                            }
                        }
                    }
                    launch.start_gate.take();
                    launch.final_gate.take();
                    launch.auth_writer.take();
                    launch.secret_ready_reader.take();
                    if let Some(pid) = launch
                        .gateway_pid
                        .filter(|_| launch.gateway_wait_status.is_none())
                    {
                        let result = if let Some(fd) = &launch.gateway_pidfd {
                            unsafe {
                                libc::syscall(
                                    libc::SYS_pidfd_send_signal,
                                    fd.as_raw_fd(),
                                    libc::SIGKILL,
                                    std::ptr::null::<libc::siginfo_t>(),
                                    0,
                                )
                            }
                        } else {
                            unsafe { libc::kill(pid as i32, libc::SIGKILL) as libc::c_long }
                        };
                        anyhow::ensure!(
                            result == 0
                                || std::io::Error::last_os_error().raw_os_error()
                                    == Some(libc::ESRCH),
                            "kill exact E3 gateway"
                        );
                        loop {
                            ManagedGatewayLaunchCapabilityV1::e3_remaining_ms(&deadline)?;
                            let mut status = 0;
                            let observed =
                                unsafe { libc::waitpid(pid as i32, &mut status, libc::WNOHANG) };
                            if observed == pid as i32 {
                                launch.gateway_wait_status = Some(status);
                                break;
                            }
                            anyhow::ensure!(observed == 0, "E3 gateway reap identity lost");
                            std::thread::sleep(Duration::from_millis(1));
                        }
                    }
                }
                owner.revoke_live_v1(registry)
            }
            E3GatewayRevocationTargetV1::Recovered {
                effects,
                service_instance_id,
                exclusion,
            } => Self::revoke_recovered_v1(effects, service_instance_id, exclusion, registry),
        }
    }

    fn revoke_recovered_v1(
        effects: &[config_projection::E3KernelEffectRecoveryV1],
        service_instance_id: &str,
        exclusion: &E3PrivilegedChildExclusionV1,
        registry: &config_projection::ConfigProjectionRegistryV1,
    ) -> Result<()> {
        use config_projection::*;
        use sha2::{Digest, Sha256};
        use std::os::fd::AsFd;
        use std::os::unix::fs::{FileExt, MetadataExt};

        fn seal<T: Serialize>(
            domain: &str,
            key: &str,
            omitted: &str,
            object: &T,
        ) -> Result<String> {
            let mut value = serde_json::to_value(object)?;
            value
                .as_object_mut()
                .context("invalid recovered object")?
                .remove(omitted);
            Ok(ConfigProjectionCodecV1::domain_sha256(
                domain,
                &serde_json::json!({key:value}),
            )?)
        }
        fn open_directory(path: &str) -> Result<fs::File> {
            anyhow::ensure!(path.starts_with('/'), "recovery coordinate is not absolute");
            let mut directory = fs::File::open("/")?;
            for component in path.split('/').skip(1) {
                anyhow::ensure!(
                    !component.is_empty() && !matches!(component, "." | ".."),
                    "invalid recovery coordinate"
                );
                let name = std::ffi::CString::new(component)?;
                let raw = unsafe {
                    libc::openat(
                        directory.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    )
                };
                if raw < 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("open recorded recovery directory");
                }
                directory = unsafe { fs::File::from_raw_fd(raw) };
            }
            Ok(directory)
        }
        fn named(parent: &fs::File, name: &std::ffi::CStr) -> Result<Option<libc::stat>> {
            let mut stat: libc::stat = unsafe { std::mem::zeroed() };
            if unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    &mut stat,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } == 0
            {
                return Ok(Some(stat));
            }
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                Ok(None)
            } else {
                Err(error.into())
            }
        }
        fn empty_group(file: &fs::File) -> Result<(String, String)> {
            let metadata = file.metadata()?;
            let mut previous = None;
            for _ in 0..2 {
                for entry in fs::read_dir(format!("/proc/self/fd/{}", file.as_raw_fd()))? {
                    anyhow::ensure!(
                        !entry?.file_type()?.is_dir(),
                        "unclassified recovered cgroup descendant"
                    );
                }
                let mut contents = Vec::new();
                for control in [c"cgroup.events", c"cgroup.procs", c"cgroup.threads"] {
                    let raw = unsafe {
                        libc::openat(
                            file.as_raw_fd(),
                            control.as_ptr(),
                            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                        )
                    };
                    if raw < 0 {
                        return Err(std::io::Error::last_os_error())
                            .context("open recovered cgroup control");
                    }
                    let control_file = unsafe { fs::File::from_raw_fd(raw) };
                    let current = control_file.metadata()?;
                    anyhow::ensure!(
                        current.uid() == 0
                            && current.mode() & 0o022 == 0
                            && current.dev() == metadata.dev(),
                        "recovered cgroup control changed"
                    );
                    let mut bytes = Vec::new();
                    std::io::Read::read_to_end(
                        &mut std::io::Read::take(control_file, 4096),
                        &mut bytes,
                    )?;
                    anyhow::ensure!(bytes.len() < 4096, "recovered cgroup control exceeds bound");
                    contents.push(bytes);
                }
                anyhow::ensure!(
                    contents[1].is_empty()
                        && contents[2].is_empty()
                        && std::str::from_utf8(&contents[0])?
                            .lines()
                            .filter(|line| line.starts_with("populated "))
                            .collect::<Vec<_>>()
                            == ["populated 0"],
                    "recovered cgroup has live or unclassified members"
                );
                if let Some(prior) = &previous {
                    anyhow::ensure!(
                        prior == &contents,
                        "recovered cgroup changed between observations"
                    );
                }
                previous = Some(contents);
            }
            let contents = previous.context("missing cgroup observations")?;
            Ok((
                format!("{:x}", Sha256::digest(&contents[0])),
                format!("{:x}", Sha256::digest(&contents[1])),
            ))
        }
        // Intent-only cleanup observes an attributable tree; it does not synthesize registrations.
        fn open_cgroup_at(
            parent: &fs::File,
            name: &std::ffi::CStr,
            flags: i32,
        ) -> Result<fs::File> {
            let mut how: libc::open_how = unsafe { std::mem::zeroed() };
            how.flags = (flags | libc::O_NOFOLLOW | libc::O_CLOEXEC) as u64;
            how.resolve = libc::RESOLVE_BENEATH
                | libc::RESOLVE_NO_MAGICLINKS
                | libc::RESOLVE_NO_SYMLINKS
                | libc::RESOLVE_NO_XDEV;
            let raw = unsafe {
                libc::syscall(
                    libc::SYS_openat2,
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    &how,
                    std::mem::size_of_val(&how),
                )
            };
            if raw < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("open non-mounted recovered cgroup entry");
            }
            let file = unsafe { fs::File::from_raw_fd(raw as i32) };
            let metadata = file.metadata()?;
            anyhow::ensure!(
                metadata.dev() == parent.metadata()?.dev()
                    && metadata.uid() == 0
                    && metadata.mode() & 0o022 == 0,
                "unprotected recovered cgroup entry"
            );
            let stat = named(parent, name)?.context("recovered cgroup entry disappeared")?;
            anyhow::ensure!(
                stat.st_dev == metadata.dev() && stat.st_ino == metadata.ino(),
                "recovered cgroup entry substituted"
            );
            Ok(file)
        }
        fn cgroup_tree(root: &fs::File) -> Result<BTreeMap<String, fs::File>> {
            let mut tree = BTreeMap::new();
            let mut pending = vec![(String::new(), root.try_clone()?)];
            while let Some((path, directory)) = pending.pop() {
                anyhow::ensure!(
                    tree.len() + pending.len() < 256,
                    "recovered cgroup tree exceeds bound"
                );
                for entry in fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))? {
                    let entry = entry?;
                    let name = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow::anyhow!("noncanonical cgroup descendant"))?;
                    let component = std::ffi::CString::new(name.as_str())?;
                    let kind = entry.file_type()?;
                    let file = open_cgroup_at(
                        &directory,
                        &component,
                        libc::O_RDONLY | if kind.is_dir() { libc::O_DIRECTORY } else { 0 },
                    )?;
                    if kind.is_dir() {
                        pending.push((format!("{path}/{name}"), file));
                    } else {
                        anyhow::ensure!(
                            kind.is_file() && file.metadata()?.is_file(),
                            "unknown recovered cgroup entry"
                        );
                    }
                }
                anyhow::ensure!(
                    tree.insert(path, directory).is_none(),
                    "duplicate cgroup descendant"
                );
            }
            Ok(tree)
        }
        fn same_tree(
            expected: &BTreeMap<String, fs::File>,
            actual: &BTreeMap<String, fs::File>,
        ) -> Result<()> {
            anyhow::ensure!(
                expected.len() == actual.len(),
                "recovered descendant inventory changed"
            );
            for (name, file) in expected {
                let next = actual
                    .get(name)
                    .context("recovered descendant disappeared")?
                    .metadata()?;
                let prior = file.metadata()?;
                anyhow::ensure!(
                    prior.dev() == next.dev() && prior.ino() == next.ino(),
                    "recovered descendant substituted"
                );
            }
            Ok(())
        }
        fn quiescent_tree(tree: &BTreeMap<String, fs::File>) -> Result<bool> {
            let mut empty = true;
            for file in tree.values() {
                for control in [c"cgroup.events", c"cgroup.procs", c"cgroup.threads"] {
                    let mut file = open_cgroup_at(file, control, libc::O_RDONLY)?;
                    let mut bytes = String::new();
                    std::io::Read::read_to_string(
                        &mut std::io::Read::take(&mut file, 65536),
                        &mut bytes,
                    )?;
                    anyhow::ensure!(bytes.len() < 65536, "cgroup membership exceeds bound");
                    if control == c"cgroup.events" {
                        let populated: Vec<_> = bytes
                            .lines()
                            .filter(|line| line.starts_with("populated "))
                            .collect();
                        anyhow::ensure!(
                            populated == ["populated 0"] || populated == ["populated 1"],
                            "invalid cgroup population"
                        );
                        empty &= populated == ["populated 0"];
                    } else {
                        for pid in bytes.lines() {
                            anyhow::ensure!(pid.parse::<u32>()? > 0, "invalid cgroup member");
                        }
                        empty &= bytes.is_empty();
                    }
                }
            }
            Ok(empty)
        }
        let suffix = service_instance_id
            .strip_prefix("wsi_")
            .context("invalid recovery service instance")?;
        let instance = uuid::Uuid::parse_str(suffix)?;
        anyhow::ensure!(
            instance.get_version_num() == 7 && instance.to_string() == suffix,
            "noncanonical recovery service instance"
        );
        if effects.iter().any(|effect| effect.resolution.is_none()) {
            exclusion.require_recovering_v1()?;
        }
        // Re-read under the registry's parent/child locks. Supplied snapshots never bypass its joins.
        let current = registry.recover(None)?;
        for effect in effects {
            let exact = current
                .kernel_effects
                .iter()
                .find(|candidate| candidate.intent == effect.intent)
                .context("recovered intent changed")?;
            anyhow::ensure!(
                exact.resolution == effect.resolution
                    && exact.child_cgroup == effect.child_cgroup
                    && exact.child_processes == effect.child_processes
                    && exact.boundary == effect.boundary
                    && exact.projection == effect.projection
                    && exact.terminal_child_evidence == effect.terminal_child_evidence
                    && exact.gateway_config == effect.gateway_config,
                "recovered effect snapshot changed"
            );
        }
        let mut attempts = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for effect in effects {
            anyhow::ensure!(
                ids.insert(&effect.intent.effect_intent_id),
                "duplicate recovered intent"
            );
            attempts
                .entry((
                    &effect.intent.authority_store_id,
                    &effect.intent.series_id,
                    &effect.intent.fence_id,
                    &effect.intent.preparation_id,
                ))
                .or_insert_with(Vec::new)
                .push(effect);
        }
        let boot = fs::read_to_string("/proc/sys/kernel/random/boot_id")?
            .trim()
            .to_owned();
        // Terminal Active eligibility requires complete same-series headless cleanup. Resolve
        // those intents first while retaining Recovering admission; keep the existing order
        // within each class and all exact identity/observation checks for every attempt.
        let mut attempts: Vec<_> = attempts.into_iter().collect();
        attempts
            .sort_by_key(|(_, attempt)| attempt.iter().any(|effect| effect.projection.is_some()));
        for ((store, series, fence, preparation), attempt) in attempts {
            anyhow::ensure!(
                current
                    .kernel_effects
                    .iter()
                    .filter(|effect| &effect.intent.authority_store_id == store
                        && &effect.intent.series_id == series
                        && &effect.intent.fence_id == fence
                        && &effect.intent.preparation_id == preparation)
                    .count()
                    == attempt.len(),
                "recovered attempt omits retained effects"
            );
            let record = attempt.iter().find_map(|effect| effect.projection.as_ref());
            let expected = record.map(|record| ConfigProjectionRefV1 {
                authority_store_id: record.identity.authority_store_id.clone(),
                series_id: record.identity.series_id.clone(),
                record_id: record.record_id.clone(),
                revision: record.revision,
                record_hash: record.record_hash.clone(),
            });
            anyhow::ensure!(
                attempt
                    .iter()
                    .all(|effect| effect.projection.as_ref() == record),
                "incomplete recovered projection join"
            );
            let active_history = record.is_some_and(|record| {
                record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active
            });
            let mut retained: Option<&E3TerminalChildQuiescenceEvidenceV1> = None;
            for evidence in attempt
                .iter()
                .flat_map(|effect| &effect.terminal_child_evidence)
                .filter(|evidence| Some(&evidence.final_projection_ref) == expected.as_ref())
            {
                if let Some(prior) = retained {
                    anyhow::ensure!(
                        ConfigProjectionCodecV1::encode_canonical_json(prior)?
                            == ConfigProjectionCodecV1::encode_canonical_json(evidence)?,
                        "ambiguous recovered terminal evidence"
                    );
                } else {
                    retained = Some(evidence);
                }
            }
            anyhow::ensure!(
                !active_history
                    || (retained.is_some()
                        && attempt.iter().all(|effect| effect.resolution.is_some())),
                "Active history lacks completed terminal runtime authority"
            );
            let historical = if let Some(record) = record {
                let readback = registry.recover(Some(&record.identity))?;
                let Some(ConfigProjectionSubjectReadbackV1::Bound(metadata)) = readback.subject
                else {
                    anyhow::bail!("cleanup subject missing");
                };
                if metadata.record() != record {
                    anyhow::ensure!(
                        metadata.record().identity == record.identity
                            && metadata.record().revision > record.revision
                            && retained.is_some()
                            && attempt.iter().all(|effect| effect.resolution.is_some()),
                        "unresolved or unproved historical cleanup"
                    );
                    true
                } else {
                    active_history
                }
            } else {
                false
            };
            let covered: Vec<_> = current
                .kernel_effects
                .iter()
                .filter(|effect| {
                    &effect.intent.authority_store_id == store
                        && &effect.intent.series_id == series
                        && (attempt.iter().any(|own| own.intent == effect.intent)
                            || record.is_some_and(|record| {
                                effect.projection.as_ref().is_some_and(|prior| {
                                    prior.identity == record.identity
                                        && prior.revision < record.revision
                                })
                            }))
                })
                .collect();
            anyhow::ensure!(
                covered.iter().all(
                    |effect| attempt.iter().any(|own| own.intent == effect.intent)
                        || effect.resolution.is_some()
                ),
                "historical cleanup remains unresolved"
            );
            let mut observations = Vec::new();
            for process in covered.iter().flat_map(|effect| &effect.child_processes) {
                anyhow::ensure!(
                    process.kernel_boot_id == boot
                        && (process.parent_service_instance_id != service_instance_id
                            || (retained.is_some()
                                && covered.iter().all(|effect| effect.resolution.is_some()))),
                    "recovered process boot or original service identity is not restart eligible"
                );
                let mut killed_observation = None;
                let observed = match read_pid_start_time_ticks(process.pid) {
                    Ok(start) if start != process.pid_start_time_ticks => {
                        E3RecoveryProcIdentityV1::PidReused {
                            observed_pid_start_time_ticks: start,
                        }
                    }
                    Ok(_) => {
                        exclusion.require_recovering_v1()?;
                        let effect = covered
                            .iter()
                            .find(|effect| effect.child_processes.contains(process))
                            .context("live remnant registration join missing")?;
                        anyhow::ensure!(
                            effect.resolution.is_none(),
                            "resolved process is still live"
                        );
                        let projection = effect
                            .projection
                            .as_ref()
                            .context("live remnant has no recorded owner binding")?;
                        let boundary = covered
                            .iter()
                            .find(|other| {
                                other.intent.fence_id == effect.intent.fence_id
                                    && other.intent.preparation_id == effect.intent.preparation_id
                                    && other.boundary.is_some()
                            })
                            .and_then(|other| other.boundary.as_ref())
                            .context("live remnant namespace binding absent")?;
                        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, process.pid, 0) };
                        if raw < 0 {
                            return Err(std::io::Error::last_os_error())
                                .context("open bound live remnant pidfd");
                        }
                        let pidfd = unsafe { OwnedFd::from_raw_fd(raw as i32) };
                        let proc_root = open_directory("/proc")?;
                        let mut procfs: libc::statfs = unsafe { std::mem::zeroed() };
                        anyhow::ensure!(
                            unsafe { libc::fstatfs(proc_root.as_raw_fd(), &mut procfs) } == 0
                                && procfs.f_type as i128 == libc::PROC_SUPER_MAGIC as i128,
                            "live remnant procfs changed"
                        );
                        let directory = open_directory(&format!("/proc/{}", process.pid))?;
                        let open_namespace = |name: &std::ffi::CStr| -> Result<fs::File> {
                            let raw = unsafe {
                                libc::openat(
                                    directory.as_raw_fd(),
                                    name.as_ptr(),
                                    libc::O_RDONLY | libc::O_CLOEXEC,
                                )
                            };
                            if raw < 0 {
                                return Err(std::io::Error::last_os_error())
                                    .context("open bound remnant namespace");
                            }
                            Ok(unsafe { fs::File::from_raw_fd(raw) })
                        };
                        let user = open_namespace(c"ns/user")?;
                        let network = open_namespace(c"ns/net")?;
                        let service_user = fs::File::open("/proc/self/ns/user")?;
                        let user_meta = user.metadata()?;
                        let service_meta = service_user.metadata()?;
                        let mut namespace_owner: libc::uid_t = u32::MAX;
                        anyhow::ensure!(
                            unsafe { libc::ioctl(user.as_raw_fd(), 0xb703 as _) }
                                == libc::CLONE_NEWUSER
                                && unsafe {
                                    libc::ioctl(user.as_raw_fd(), 0xb704 as _, &mut namespace_owner)
                                } == 0
                                && namespace_owner == unsafe { libc::geteuid() }
                                && (user_meta.dev(), user_meta.ino())
                                    != (service_meta.dev(), service_meta.ino())
                                && network.metadata()?.ino()
                                    == boundary.gateway_listener.network_namespace_inode,
                            "live remnant namespace type, owner or network identity changed"
                        );
                        let parent_raw = unsafe { libc::ioctl(user.as_raw_fd(), 0xb702 as _) };
                        if parent_raw < 0 {
                            return Err(std::io::Error::last_os_error())
                                .context("read remnant namespace parent");
                        }
                        let namespace_parent = unsafe { fs::File::from_raw_fd(parent_raw) };
                        anyhow::ensure!(
                            (
                                namespace_parent.metadata()?.dev(),
                                namespace_parent.metadata()?.ino()
                            ) == (service_meta.dev(), service_meta.ino()),
                            "live remnant user namespace parent changed"
                        );
                        let uid = projection.native.root.owner_uid;
                        let gid = projection.native.root.owner_gid;
                        anyhow::ensure!(uid != 0 && gid != 0, "live remnant maps host root");
                        let uid_map = fs::read_to_string(format!("/proc/{}/uid_map", process.pid))?;
                        let gid_map = fs::read_to_string(format!("/proc/{}/gid_map", process.pid))?;
                        anyhow::ensure!(
                            uid_map.split_whitespace().collect::<Vec<_>>()
                                == [uid.to_string(), uid.to_string(), "1".into()]
                                && gid_map.split_whitespace().collect::<Vec<_>>()
                                    == [gid.to_string(), gid.to_string(), "1".into()],
                            "live remnant namespace maps changed"
                        );
                        let membership =
                            fs::read_to_string(format!("/proc/{}/cgroup", process.pid))?;
                        anyhow::ensure!(
                            membership.trim()
                                == format!("0::/{}", process.process_cgroup.cgroup_relative_path),
                            "live remnant cgroup membership changed"
                        );
                        let mount = open_directory("/sys/fs/cgroup")?;
                        let cgroup = open_directory(&format!(
                            "/sys/fs/cgroup/{}",
                            process.process_cgroup.cgroup_relative_path
                        ))?;
                        let mut cgroupfs: libc::statfs = unsafe { std::mem::zeroed() };
                        anyhow::ensure!(
                            unsafe { libc::fstatfs(mount.as_raw_fd(), &mut cgroupfs) } == 0
                                && cgroupfs.f_type as i128 == libc::CGROUP2_SUPER_MAGIC as i128
                                && mount.metadata()?.dev()
                                    == process.process_cgroup.cgroup_v2_mount_device_id
                                && mount.metadata()?.ino()
                                    == process.process_cgroup.cgroup_v2_mount_inode
                                && cgroup.metadata()?.dev()
                                    == process.process_cgroup.cgroup_v2_mount_device_id
                                && cgroup.metadata()?.ino()
                                    == process.process_cgroup.cgroup_directory_inode,
                            "live remnant cgroup identity changed"
                        );
                        anyhow::ensure!(
                            read_pid_start_time_ticks(process.pid)? == process.pid_start_time_ticks
                                && fs::read_to_string(format!("/proc/{}/cgroup", process.pid))?
                                    == membership
                                && open_namespace(c"ns/user")?.metadata()?.ino() == user_meta.ino()
                                && open_namespace(c"ns/net")?.metadata()?.ino()
                                    == network.metadata()?.ino(),
                            "live remnant changed before pidfd termination"
                        );
                        let mut poll = libc::pollfd {
                            fd: pidfd.as_raw_fd(),
                            events: libc::POLLIN,
                            revents: 0,
                        };
                        anyhow::ensure!(
                            unsafe { libc::poll(&mut poll, 1, 0) } == 0,
                            "live remnant exited before identity validation"
                        );
                        let killed = unsafe {
                            libc::syscall(
                                libc::SYS_pidfd_send_signal,
                                pidfd.as_raw_fd(),
                                libc::SIGKILL,
                                std::ptr::null::<libc::siginfo_t>(),
                                0,
                            )
                        };
                        let kill_errno = if killed == 0 {
                            None
                        } else {
                            Some(
                                std::io::Error::last_os_error()
                                    .raw_os_error()
                                    .context("pidfd kill errno")?,
                            )
                        };
                        anyhow::ensure!(
                            kill_errno.is_none_or(|errno| errno == libc::ESRCH),
                            "bound remnant pidfd termination failed"
                        );
                        anyhow::ensure!(
                            unsafe { libc::poll(&mut poll, 1, 5000) } == 1
                                && poll.revents & libc::POLLIN != 0,
                            "bound remnant did not become terminal"
                        );
                        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
                        anyhow::ensure!(
                            unsafe {
                                libc::waitid(
                                    libc::P_PIDFD,
                                    pidfd.as_raw_fd() as libc::id_t,
                                    &mut info,
                                    libc::WEXITED | libc::WNOHANG,
                                )
                            } == -1
                                && std::io::Error::last_os_error().raw_os_error()
                                    == Some(libc::ECHILD),
                            "recovered remnant is not an orphaned terminal child"
                        );
                        let deadline = Instant::now() + Duration::from_secs(5);
                        let observed = loop {
                            match read_pid_start_time_ticks(process.pid) {
                                Ok(start) if start != process.pid_start_time_ticks => {
                                    break E3RecoveryProcIdentityV1::PidReused {
                                        observed_pid_start_time_ticks: start,
                                    }
                                }
                                Ok(_) => {
                                    anyhow::ensure!(
                                        Instant::now() < deadline,
                                        "terminal remnant remains present in procfs"
                                    );
                                    std::thread::sleep(Duration::from_millis(10));
                                }
                                Err(error)
                                    if error.downcast_ref::<std::io::Error>().is_some_and(
                                        |error| error.raw_os_error() == Some(libc::ENOENT),
                                    ) =>
                                {
                                    break E3RecoveryProcIdentityV1::Absent
                                }
                                Err(error) => {
                                    return Err(error).context("read terminated remnant identity")
                                }
                            }
                        };
                        killed_observation = Some(kill_errno);
                        observed
                    }
                    Err(error)
                        if error
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|error| error.raw_os_error() == Some(libc::ENOENT)) =>
                    {
                        E3RecoveryProcIdentityV1::Absent
                    }
                    Err(error) => return Err(error).context("read recovered PID identity"),
                };
                let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, process.pid, 0) };
                let errno = if raw < 0 {
                    Some(
                        std::io::Error::last_os_error()
                            .raw_os_error()
                            .context("pidfd errno")?,
                    )
                } else {
                    drop(unsafe { OwnedFd::from_raw_fd(raw as i32) });
                    None
                };
                anyhow::ensure!(
                    match &observed {
                        E3RecoveryProcIdentityV1::Absent => errno == Some(libc::ESRCH),
                        E3RecoveryProcIdentityV1::PidReused {
                            observed_pid_start_time_ticks,
                        } =>
                            errno.is_none()
                                && read_pid_start_time_ticks(process.pid)?
                                    == *observed_pid_start_time_ticks,
                    },
                    "recovered PID changed across pidfd observation"
                );
                observations.push(E3TerminalProcessObservationV1 {
                    registration_id: process.registration_id.clone(),
                    registration_hash: process.registration_hash.clone(),
                    role: process.role,
                    pid: process.pid,
                    pid_start_time_ticks: process.pid_start_time_ticks,
                    observation: E3TerminalProcessObservationKindV1::RecoveryObservedTerminal {
                        original_service_instance_id: process.parent_service_instance_id.clone(),
                        recovery_service_instance_id: service_instance_id.to_owned(),
                        pidfd_open_errno: if killed_observation.is_some() {
                            None
                        } else {
                            errno
                        },
                        pidfd_kill_errno: killed_observation.flatten(),
                        pidfd_became_readable: killed_observation.map(|_| true),
                        waitid_errno: killed_observation.map(|_| libc::ECHILD),
                        proc_identity: observed,
                    },
                });
            }
            let mut groups = Vec::new();
            let mut group_evidence = Vec::new();
            for effect in &covered {
                let E3KernelEffectKindV1::CreateChildCgroup {
                    parent_cgroup,
                    child_component,
                    expected_relative_path,
                    ..
                } = &effect.intent.effect
                else {
                    continue;
                };
                let mount = open_directory("/sys/fs/cgroup")?;
                let mount_meta = mount.metadata()?;
                let mut filesystem: libc::statfs = unsafe { std::mem::zeroed() };
                anyhow::ensure!(
                    unsafe { libc::fstatfs(mount.as_raw_fd(), &mut filesystem) } == 0
                        && filesystem.f_type as i128 == libc::CGROUP2_SUPER_MAGIC as i128
                        && mount_meta.dev() == parent_cgroup.cgroup_v2_mount_device_id
                        && mount_meta.ino() == parent_cgroup.cgroup_v2_mount_inode,
                    "recovered cgroup mount identity changed"
                );
                let mut parent = mount.try_clone()?;
                let mut absent_parent = false;
                for part in parent_cgroup.cgroup_relative_path.split('/') {
                    let component = std::ffi::CString::new(part)?;
                    let meta = parent.metadata()?;
                    anyhow::ensure!(
                        meta.uid() == 0
                            && meta.mode() & 0o022 == 0
                            && meta.dev() == mount_meta.dev(),
                        "recovered cgroup ancestor changed"
                    );
                    if named(&parent, &component)?.is_none() {
                        anyhow::ensure!(
                            named(&parent, &component)?.is_none(),
                            "absent cgroup ancestor reappeared"
                        );
                        absent_parent = true;
                        break;
                    }
                    parent =
                        open_cgroup_at(&parent, &component, libc::O_RDONLY | libc::O_DIRECTORY)?;
                }
                if absent_parent {
                    let resolution = effect
                        .resolution
                        .as_ref()
                        .context("missing cgroup parent has no completed resolution")?;
                    if let Some(registration) = &effect.child_cgroup {
                        anyhow::ensure!(
                            registration.kernel_boot_id == boot
                                && resolution.observed_cgroup.as_ref()
                                    == Some(&registration.cgroup)
                                && resolution.disposition
                                    == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                            "missing parent does not bind the resolved registration"
                        );
                        if let Some(projection) = &effect.projection {
                            let proofs: Vec<_> = effect
                                .terminal_child_evidence
                                .iter()
                                .filter(|proof| {
                                    proof.final_projection_ref.record_id == projection.record_id
                                        && proof.final_projection_ref.record_hash
                                            == projection.record_hash
                                        && proof.final_projection_ref.revision
                                            == projection.revision
                                })
                                .collect();
                            anyhow::ensure!(
                                proofs.len() == 1,
                                "missing parent has no unique terminal proof"
                            );
                            let proof = proofs[0]
                                .ordered_empty_cgroups
                                .iter()
                                .find(|entry| {
                                    entry.cgroup_registration_id
                                        == registration.cgroup_registration_id
                                        && entry.cgroup_registration_hash
                                            == registration.cgroup_registration_hash
                                        && entry.cgroup == registration.cgroup
                                })
                                .context("missing parent proof omits registration")?;
                            group_evidence.push(proof.clone());
                        }
                    } else {
                        anyhow::ensure!(
                            (resolution.disposition
                                == E3KernelEffectResolutionDispositionV1::NoEffectObserved
                                && resolution.observed_cgroup.is_none())
                                || (resolution.disposition
                                    == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent
                                    && resolution.observed_cgroup.as_ref().is_some_and(
                                        |observed| observed.cgroup_relative_path
                                            == *expected_relative_path
                                            && observed.cgroup_v2_mount_device_id
                                                == mount_meta.dev()
                                            && observed.cgroup_v2_mount_inode == mount_meta.ino()
                                    )),
                            "headless missing parent has no absent-effect resolution"
                        );
                    }
                    continue;
                }
                let metadata = parent.metadata()?;
                anyhow::ensure!(
                    metadata.dev() == parent_cgroup.cgroup_v2_mount_device_id
                        && metadata.ino() == parent_cgroup.cgroup_directory_inode
                        && metadata.uid() == 0
                        && metadata.mode() & 0o022 == 0,
                    "recovered cgroup parent changed"
                );
                let component = std::ffi::CString::new(child_component.as_str())?;
                let stat = named(&parent, &component)?;
                if stat.is_some() && effect.child_cgroup.is_none() {
                    exclusion.require_recovering_v1()?;
                    anyhow::ensure!(
                        effect.resolution.is_none()
                            && effect.projection.is_none()
                            && effect.child_processes.is_empty()
                            && effect.terminal_child_evidence.is_empty(),
                        "unregistered cgroup has conflicting retained authority"
                    );
                    // No other intent may own a node within the effect being reversed.
                    for other in &current.kernel_effects {
                        if other.intent.effect_intent_id == effect.intent.effect_intent_id {
                            continue;
                        }
                        if let E3KernelEffectKindV1::CreateChildCgroup {
                            expected_relative_path: other_path,
                            ..
                        } = &other.intent.effect
                        {
                            anyhow::ensure!(
                                other_path != expected_relative_path
                                    && !other_path
                                        .starts_with(&format!("{expected_relative_path}/")),
                                "recovered subtree overlaps another intent"
                            );
                        }
                    }
                    let child =
                        open_cgroup_at(&parent, &component, libc::O_RDONLY | libc::O_DIRECTORY)?;
                    let observed = CanonicalCgroupIdentityV1 {
                        cgroup_v2_mount_device_id: mount_meta.dev(),
                        cgroup_v2_mount_inode: mount_meta.ino(),
                        cgroup_directory_inode: child.metadata()?.ino(),
                        cgroup_relative_path: expected_relative_path.clone(),
                    };
                    let tree = cgroup_tree(&child)?;
                    quiescent_tree(&tree)?;
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: effect.intent.authority_store_id.clone(),
                        resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                        effect_intent_ref: E3KernelEffectIntentRefV1 {
                            authority_store_id: effect.intent.authority_store_id.clone(),
                            effect_intent_id: effect.intent.effect_intent_id.clone(),
                            intent_hash: effect.intent.intent_hash.clone(),
                        },
                        disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                        observed_cgroup: Some(observed),
                        observed_nftables_table_handle: None,
                        resolved_at: Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        resolution_hash: String::new(),
                    };
                    resolution.resolution_hash = seal(
                        "substrate.e3.kernel-effect-resolution.v1",
                        "resolution",
                        "resolution_hash",
                        &resolution,
                    )?;
                    let mut cleanup_error = None;
                    let published = registry.publish_kernel_effect_resolution(
                        &resolution,
                        Some(&mut || {
                            (|| -> Result<_> {
                                exclusion.require_recovering_v1()?;
                                let parent_name = std::ffi::CString::new(
                                    parent_cgroup.cgroup_relative_path.as_str(),
                                )?;
                                let fresh_parent = open_cgroup_at(
                                    &mount,
                                    &parent_name,
                                    libc::O_RDONLY | libc::O_DIRECTORY,
                                )?;
                                anyhow::ensure!(
                                    fresh_parent.metadata()?.ino() == parent.metadata()?.ino(),
                                    "recovered parent substituted before mutation"
                                );
                                let reopened = open_cgroup_at(
                                    &parent,
                                    &component,
                                    libc::O_RDONLY | libc::O_DIRECTORY,
                                )?;
                                anyhow::ensure!(
                                    reopened.metadata()?.ino() == child.metadata()?.ino(),
                                    "unregistered cgroup substituted before cleanup"
                                );
                                same_tree(&tree, &cgroup_tree(&reopened)?)?;
                                // cgroup.kill targets the validated subtree atomically, including forks racing termination.
                                let mut kill =
                                    open_cgroup_at(&child, c"cgroup.kill", libc::O_WRONLY)?;
                                std::io::Write::write_all(&mut kill, b"1")?;
                                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                                loop {
                                    let scanned = cgroup_tree(&child)?;
                                    same_tree(&tree, &scanned)?;
                                    if quiescent_tree(&scanned)? {
                                        let second = cgroup_tree(&child)?;
                                        same_tree(&tree, &second)?;
                                        if quiescent_tree(&second)? {
                                            break;
                                        }
                                    }
                                    anyhow::ensure!(
                                        std::time::Instant::now() < deadline,
                                        "unregistered cgroup did not become quiescent"
                                    );
                                    std::thread::sleep(Duration::from_millis(10));
                                }
                                for (path, file) in tree.iter().rev() {
                                    if path.is_empty() {
                                        continue;
                                    }
                                    let (parent_path, name) = path
                                        .rsplit_once('/')
                                        .context("cgroup descendant coordinate")?;
                                    let directory = tree
                                        .get(parent_path)
                                        .context("cgroup descendant parent")?;
                                    let name = std::ffi::CString::new(name)?;
                                    let current = open_cgroup_at(
                                        directory,
                                        &name,
                                        libc::O_RDONLY | libc::O_DIRECTORY,
                                    )?;
                                    anyhow::ensure!(
                                        current.metadata()?.ino() == file.metadata()?.ino(),
                                        "descendant changed before removal"
                                    );
                                    empty_group(&current)?;
                                    anyhow::ensure!(
                                        unsafe {
                                            libc::unlinkat(
                                                directory.as_raw_fd(),
                                                name.as_ptr(),
                                                libc::AT_REMOVEDIR,
                                            )
                                        } == 0,
                                        "remove recovered descendant"
                                    );
                                    anyhow::ensure!(
                                        named(directory, &name)?.is_none(),
                                        "recovered descendant remains"
                                    );
                                }
                                empty_group(&child)?;
                                let current = open_cgroup_at(
                                    &parent,
                                    &component,
                                    libc::O_RDONLY | libc::O_DIRECTORY,
                                )?;
                                anyhow::ensure!(
                                    current.metadata()?.ino() == child.metadata()?.ino(),
                                    "cgroup changed before removal"
                                );
                                anyhow::ensure!(
                                    unsafe {
                                        libc::unlinkat(
                                            parent.as_raw_fd(),
                                            component.as_ptr(),
                                            libc::AT_REMOVEDIR,
                                        )
                                    } == 0,
                                    "remove unregistered recovered cgroup"
                                );
                                anyhow::ensure!(
                                    named(&parent, &component)?.is_none()
                                        && named(&parent, &component)?.is_none(),
                                    "unregistered cgroup remains"
                                );
                                Ok(None)
                            })()
                            .map_err(|error| {
                                cleanup_error = Some(error);
                                ConfigProjectionFailureV1::UnsupportedSecurityPosture
                            })
                        }),
                        None,
                    );
                    if let Some(error) = cleanup_error {
                        return Err(error).context("unregistered cgroup reversal");
                    }
                    anyhow::ensure!(
                        published? == resolution,
                        "unregistered cgroup resolution readback changed"
                    );
                    continue;
                }
                let child = if let Some(stat) = stat {
                    let registration = effect
                        .child_cgroup
                        .as_ref()
                        .context("unregistered recovered cgroup cannot be adopted")?;
                    anyhow::ensure!(
                        registration.kernel_boot_id == boot
                            && stat.st_dev == registration.cgroup.cgroup_v2_mount_device_id
                            && stat.st_ino == registration.cgroup.cgroup_directory_inode
                            && stat.st_mode & libc::S_IFMT == libc::S_IFDIR,
                        "recovered cgroup boot or identity changed"
                    );
                    let raw = unsafe {
                        libc::openat(
                            parent.as_raw_fd(),
                            component.as_ptr(),
                            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                        )
                    };
                    if raw < 0 {
                        return Err(std::io::Error::last_os_error())
                            .context("open recovered child cgroup");
                    }
                    let file = unsafe { fs::File::from_raw_fd(raw) };
                    let opened = file.metadata()?;
                    anyhow::ensure!(
                        opened.dev() == stat.st_dev
                            && opened.ino() == stat.st_ino
                            && opened.uid() == 0
                            && opened.mode() & 0o022 == 0,
                        "recovered child changed during open"
                    );
                    let (events, procs) = empty_group(&file)?;
                    group_evidence.push(E3TerminalCgroupQuiescenceV1 {
                        cgroup_registration_id: registration.cgroup_registration_id.clone(),
                        cgroup_registration_hash: registration.cgroup_registration_hash.clone(),
                        role: registration.role,
                        cgroup: registration.cgroup.clone(),
                        cgroup_events_sha256: events,
                        cgroup_procs_sha256: procs,
                        populated: false,
                        ordered_live_pids: Vec::new(),
                    });
                    Some(file)
                } else {
                    if let Some(registration) = &effect.child_cgroup {
                        anyhow::ensure!(
                            registration.kernel_boot_id == boot,
                            "recovered absent cgroup boot changed"
                        );
                        let Some(final_record) = effect.projection.as_ref() else {
                            anyhow::ensure!(effect.resolution.as_ref().is_some_and(|resolution| resolution.disposition == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent && resolution.observed_cgroup.as_ref() == Some(&registration.cgroup)), "absent headless registration has no cleanup proof");
                            anyhow::ensure!(
                                named(&parent, &component)?.is_none(),
                                "headless cgroup reappeared"
                            );
                            continue;
                        };
                        let proofs: Vec<_> = effect
                            .terminal_child_evidence
                            .iter()
                            .filter(|proof| {
                                proof.final_projection_ref.record_id == final_record.record_id
                                    && proof.final_projection_ref.record_hash
                                        == final_record.record_hash
                                    && proof.final_projection_ref.revision == final_record.revision
                            })
                            .collect();
                        anyhow::ensure!(
                            proofs.len() == 1,
                            "absent historical cgroup has ambiguous or missing proof"
                        );
                        let proof = proofs
                            .first()
                            .copied()
                            .and_then(|evidence| {
                                evidence.ordered_empty_cgroups.iter().find(|entry| {
                                    entry.cgroup_registration_id
                                        == registration.cgroup_registration_id
                                        && entry.cgroup_registration_hash
                                            == registration.cgroup_registration_hash
                                        && entry.cgroup == registration.cgroup
                                })
                            })
                            .context("absent recovered cgroup lacks original terminal proof")?;
                        group_evidence.push(proof.clone());
                    }
                    None
                };
                if attempt.iter().any(|own| own.intent == effect.intent) {
                    groups.push((parent, component, child, *effect));
                } else {
                    anyhow::ensure!(child.is_none(), "resolved historical cgroup reappeared");
                }
            }
            if let Some(record) = record {
                let rank = |role| match role {
                    E3TerminalProcessRoleV1::ManagedGateway => 0,
                    E3TerminalProcessRoleV1::Codex => 1,
                    E3TerminalProcessRoleV1::ReadinessProbe => 2,
                };
                observations.sort_by_key(|entry| {
                    (
                        rank(entry.role),
                        entry.pid,
                        entry.pid_start_time_ticks,
                        entry.registration_id.clone(),
                    )
                });
                group_evidence.sort_by_key(|entry| {
                    (
                        rank(entry.role),
                        entry.cgroup.cgroup_v2_mount_device_id,
                        entry.cgroup.cgroup_v2_mount_inode,
                        entry.cgroup.cgroup_directory_inode,
                        entry.cgroup.cgroup_relative_path.clone(),
                        entry.cgroup_registration_id.clone(),
                    )
                });
                let evidence = if let Some(prior) = retained {
                    anyhow::ensure!(
                        observations.iter().all(|entry| prior
                            .ordered_terminal_processes
                            .iter()
                            .any(|old| old.registration_id == entry.registration_id
                                && old.registration_hash == entry.registration_hash))
                            && group_evidence.iter().all(|entry| prior
                                .ordered_empty_cgroups
                                .iter()
                                .any(|old| old.cgroup_registration_id
                                    == entry.cgroup_registration_id
                                    && old.cgroup_registration_hash
                                        == entry.cgroup_registration_hash
                                    && old.cgroup == entry.cgroup)),
                        "retained evidence omits recovered registrations"
                    );
                    prior.clone()
                } else {
                    let mut evidence = E3TerminalChildQuiescenceEvidenceV1 {
                        schema_version: 1,
                        authority_store_id: record.identity.authority_store_id.clone(),
                        series_id: series.clone(),
                        evidence_id: format!("tce_{}", uuid::Uuid::now_v7()),
                        final_projection_ref: expected
                            .clone()
                            .context("missing recovered final ref")?,
                        world_id: record.identity.world_id.clone(),
                        world_generation: record.identity.world_generation,
                        ordered_terminal_processes: observations,
                        ordered_empty_cgroups: group_evidence,
                        observed_at: Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        evidence_hash: String::new(),
                    };
                    evidence.evidence_hash = seal(
                        "substrate.e3.terminal-child-quiescence.v1",
                        "evidence",
                        "evidence_hash",
                        &evidence,
                    )?;
                    evidence
                };
                // Active accepts only the retained exact object; its retry also validates
                // terminal handoff, released leases and the original Allow -> Revoked chain.
                registry.publish_terminal_child_evidence(&evidence)?;
            }
            let mut pending_tables = Vec::new();
            // Boundary revocation uses the same fixed wire codec as installation, with no listener/owner reconstruction.
            for effect in &attempt {
                let E3KernelEffectKindV1::InstallGatewayBoundary {
                    network_namespace_inode,
                    table_name,
                    chain_name,
                    ..
                } = &effect.intent.effect
                else {
                    continue;
                };
                let namespace = fs::File::open("/proc/self/ns/net")?;
                anyhow::ensure!(
                    namespace.metadata()?.ino() == *network_namespace_inode,
                    "recovered network namespace unavailable or changed"
                );
                if let Some(original) = &effect.boundary {
                    anyhow::ensure!(
                        original.gateway_listener.network_namespace_inode
                            == *network_namespace_inode,
                        "recovered listener namespace changed"
                    );
                    for row in fs::read_to_string("/proc/self/net/tcp")?.lines().skip(1) {
                        anyhow::ensure!(
                            row.split_whitespace()
                                .nth(9)
                                .context("malformed TCP inode row")?
                                .parse::<u64>()?
                                != original.gateway_listener.socket_inode,
                            "original recovered listener is still live"
                        );
                    }
                }
                let raw = unsafe {
                    libc::socket(
                        libc::AF_NETLINK,
                        libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                        libc::NETLINK_NETFILTER,
                    )
                };
                if raw < 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("open recovered netfilter socket");
                }
                let socket = unsafe { OwnedFd::from_raw_fd(raw) };
                let mut sequence = 1000u32;
                let mut request =
                    |kind: u16, payload: &[u8], dump: bool| -> Result<Vec<(u16, Vec<u8>)>> {
                        sequence += 1;
                        exchange(
                            socket.as_raw_fd(),
                            &message(
                                0x0a00 | kind,
                                if dump { 0x301 } else { 5 },
                                sequence,
                                1,
                                payload,
                            ),
                            &[sequence],
                            dump,
                        )
                    };
                let table_query = string(1, table_name);
                let table = request(1, &table_query, false);
                let absent = table
                    .as_ref()
                    .err()
                    .and_then(|error| error.downcast_ref::<std::io::Error>())
                    .is_some_and(|error| error.raw_os_error() == Some(libc::ENOENT));
                let Some(original) = effect.boundary.as_ref() else {
                    if !absent {
                        exclusion.require_recovering_v1()?;
                        anyhow::ensure!(
                            record.is_none()
                                && effect.gateway_config.is_none()
                                && effect.resolution.is_none(),
                            "unregistered boundary has conflicting retained authority"
                        );
                        let table_rows = table?;
                        anyhow::ensure!(
                            table_rows.len() == 1
                                && table_rows[0].0 == 0x0a00
                                && table_rows[0].1[0] == 1,
                            "unregistered boundary table is not unique inet"
                        );
                        let table_fields = map(&table_rows[0].1[4..])?;
                        anyhow::ensure!(
                            table_fields.get(&1)
                                == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                                && table_fields
                                    .get(&2)
                                    .is_none_or(|flags| *flags == 0u32.to_be_bytes()),
                            "unregistered table identity changed"
                        );
                        let handle = u64::from_be_bytes(
                            (*table_fields
                                .get(&4)
                                .context("unregistered table handle missing")?)
                            .try_into()?,
                        );
                        anyhow::ensure!(handle != 0, "zero unregistered table handle");
                        let chain_query = [string(1, table_name), string(3, chain_name)].concat();
                        let rule_query = [string(1, table_name), string(2, chain_name)].concat();
                        let chains = request(4, &table_query, true)?;
                        anyhow::ensure!(
                            chains.len() == 1 && chains[0].0 == 0x0a03 && chains[0].1[0] == 1,
                            "unclassified unregistered boundary chains"
                        );
                        let fields = map(&chains[0].1[4..])?;
                        let hook =
                            map(fields.get(&4).context("unregistered chain hook missing")?)?;
                        let chain_handle = u64::from_be_bytes(
                            (*fields
                                .get(&2)
                                .context("unregistered chain handle missing")?)
                            .try_into()?,
                        );
                        anyhow::ensure!(
                            chain_handle != 0
                                && fields.get(&1)
                                    == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                                && fields.get(&3)
                                    == Some(&[chain_name.as_bytes(), &[0]].concat().as_slice())
                                && fields.get(&5) == Some(&1u32.to_be_bytes().as_slice())
                                && fields.get(&7) == Some(&b"filter\0".as_slice())
                                && hook.len() == 2
                                && hook.get(&1) == Some(&3u32.to_be_bytes().as_slice())
                                && hook.get(&2) == Some(&(-100i32).to_be_bytes().as_slice()),
                            "unregistered chain identity or hook changed"
                        );
                        for kind in [10, 19, 23] {
                            anyhow::ensure!(
                                request(kind, &table_query, true)?.is_empty(),
                                "unknown object in unregistered boundary"
                            );
                        }
                        let rules = request(7, &rule_query, true)?;
                        let mut handles = BTreeSet::new();
                        for (kind, bytes) in &rules {
                            anyhow::ensure!(
                                *kind == 0x0a06 && bytes[0] == 1,
                                "unregistered rule type changed"
                            );
                            let fields = map(&bytes[4..])?;
                            let rule_handle = u64::from_be_bytes(
                                (*fields.get(&3).context("unregistered rule handle missing")?)
                                    .try_into()?,
                            );
                            anyhow::ensure!(
                                fields.get(&1)
                                    == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                                    && fields.get(&2)
                                        == Some(&[chain_name.as_bytes(), &[0]].concat().as_slice())
                                    && rule_handle != 0
                                    && handles.insert(rule_handle),
                                "unregistered rule identity changed"
                            );
                        }
                        let mut resolution = E3KernelEffectResolutionV1 {
                            schema_version: 1,
                            authority_store_id: effect.intent.authority_store_id.clone(),
                            resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                            effect_intent_ref: E3KernelEffectIntentRefV1 {
                                authority_store_id: effect.intent.authority_store_id.clone(),
                                effect_intent_id: effect.intent.effect_intent_id.clone(),
                                intent_hash: effect.intent.intent_hash.clone(),
                            },
                            disposition:
                                E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                            observed_cgroup: None,
                            observed_nftables_table_handle: Some(handle),
                            resolved_at: Timestamp(
                                chrono::Utc::now()
                                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                            ),
                            resolution_hash: String::new(),
                        };
                        resolution.resolution_hash = seal(
                            "substrate.e3.kernel-effect-resolution.v1",
                            "resolution",
                            "resolution_hash",
                            &resolution,
                        )?;
                        let mut cleanup_error = None;
                        let published = registry.publish_kernel_effect_resolution(
                            &resolution,
                            Some(&mut || {
                                (|| -> Result<_> {
                                    exclusion.require_recovering_v1()?;
                                    anyhow::ensure!(
                                        fs::metadata("/proc/self/ns/net")?.ino()
                                            == *network_namespace_inode
                                            && request(1, &table_query, false)? == table_rows
                                            && request(4, &table_query, true)? == chains
                                            && request(7, &rule_query, true)? == rules,
                                        "unregistered boundary changed before reversal"
                                    );
                                    for kind in [10, 19, 23] {
                                        anyhow::ensure!(
                                            request(kind, &table_query, true)?.is_empty(),
                                            "unknown boundary object appeared"
                                        );
                                    }
                                    // Replace the exact chain's complete rule set atomically with inet reject-all.
                                    let reject = expression(
                                        "reject",
                                        &[number(1, 2), attribute(2, &[3])].concat(),
                                    );
                                    let batch = [
                                        message(0x10, 1, 7000, 0, &[]),
                                        message(0x0a08, 5, 7001, 1, &rule_query),
                                        message(
                                            0x0a06,
                                            0xc05,
                                            7002,
                                            1,
                                            &[rule_query.clone(), attribute(0x8004, &reject)]
                                                .concat(),
                                        ),
                                        message(0x11, 1, 7003, 0, &[]),
                                    ]
                                    .concat();
                                    exchange(socket.as_raw_fd(), &batch, &[7001, 7002], false)?;
                                    let rejected = request(7, &rule_query, true)?;
                                    anyhow::ensure!(
                                        rejected.len() == 1 && rejected[0].0 == 0x0a06,
                                        "unregistered reject-all readback incomplete"
                                    );
                                    let fields = map(&rejected[0].1[4..])?;
                                    anyhow::ensure!(
                                        fields.get(&1)
                                            == Some(
                                                &[table_name.as_bytes(), &[0]].concat().as_slice()
                                            )
                                            && fields.get(&2)
                                                == Some(
                                                    &[chain_name.as_bytes(), &[0]]
                                                        .concat()
                                                        .as_slice()
                                                )
                                            && matches_expression(
                                                fields
                                                    .get(&4)
                                                    .context("reject-all expression missing")?,
                                                &reject,
                                                0
                                            )?,
                                        "unregistered reject-all expression changed"
                                    );
                                    let current_table = request(1, &table_query, false)?;
                                    let current_chains = request(4, &chain_query, false)?;
                                    anyhow::ensure!(current_table.len() == 1 && current_chains.len() == 1
                                        && current_table[0].0 == 0x0a00 && current_chains[0].0 == 0x0a03
                                        && current_table[0].1[0] == 1 && current_chains[0].1[0] == 1,
                                        "unregistered boundary changed message identity");
                                    let mut before_chain = map(&chains[0].1[4..])?;
                                    let mut after_chain = map(&current_chains[0].1[4..])?;
                                    // NFTA_CHAIN_USE counts rules: our complete replacement leaves exactly one.
                                    anyhow::ensure!(before_chain.remove(&6) == Some((rules.len() as u32).to_be_bytes().as_slice())
                                        && after_chain.remove(&6) == Some(1u32.to_be_bytes().as_slice())
                                        && before_chain == after_chain
                                        && map(&current_table[0].1[4..])? == map(&table_rows[0].1[4..])?,
                                        "unregistered table or chain identity changed before deletion");
                                    let delete = [
                                        message(0x10, 1, 7010, 0, &[]),
                                        message(
                                            0x0a02,
                                            5,
                                            7011,
                                            1,
                                            &attribute(4, &handle.to_be_bytes()),
                                        ),
                                        message(0x11, 1, 7012, 0, &[]),
                                    ]
                                    .concat();
                                    exchange(socket.as_raw_fd(), &delete, &[7011], false)?;
                                    for _ in 0..2 {
                                        anyhow::ensure!(
                                            request(1, &table_query, false)
                                                .as_ref()
                                                .err()
                                                .and_then(
                                                    |error| error.downcast_ref::<std::io::Error>()
                                                )
                                                .is_some_and(|error| error.raw_os_error()
                                                    == Some(libc::ENOENT)),
                                            "unregistered boundary remains after deletion"
                                        );
                                    }
                                    Ok(None)
                                })()
                                .map_err(|error| {
                                    cleanup_error = Some(error);
                                    ConfigProjectionFailureV1::UnsupportedSecurityPosture
                                })
                            }),
                            None,
                        );
                        if let Some(error) = cleanup_error {
                            return Err(error).context("unregistered boundary reversal");
                        }
                        anyhow::ensure!(
                            published? == resolution,
                            "unregistered boundary resolution changed"
                        );
                        continue;
                    }
                    anyhow::ensure!(
                        record.is_none() && effect.gateway_config.is_none() && absent,
                        "headless boundary lacks an exact absent-effect classification"
                    );
                    let resolution = if let Some(existing) = &effect.resolution {
                        existing.clone()
                    } else {
                        let mut resolution = E3KernelEffectResolutionV1 {
                            schema_version: 1,
                            authority_store_id: effect.intent.authority_store_id.clone(),
                            resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                            effect_intent_ref: E3KernelEffectIntentRefV1 {
                                authority_store_id: effect.intent.authority_store_id.clone(),
                                effect_intent_id: effect.intent.effect_intent_id.clone(),
                                intent_hash: effect.intent.intent_hash.clone(),
                            },
                            disposition: E3KernelEffectResolutionDispositionV1::NoEffectObserved,
                            observed_cgroup: None,
                            observed_nftables_table_handle: None,
                            resolved_at: Timestamp(
                                chrono::Utc::now()
                                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                            ),
                            resolution_hash: String::new(),
                        };
                        resolution.resolution_hash = seal(
                            "substrate.e3.kernel-effect-resolution.v1",
                            "resolution",
                            "resolution_hash",
                            &resolution,
                        )?;
                        resolution
                    };
                    let mut verify_absent = || -> std::result::Result<Option<GatewayAccessBoundaryV1>, ConfigProjectionFailureV1> {
                        let result = request(1, &table_query, false);
                        if namespace.metadata().is_ok_and(|meta| meta.ino() == *network_namespace_inode)
                            && result.as_ref().err().and_then(|error| error.downcast_ref::<std::io::Error>()).is_some_and(|error| error.raw_os_error() == Some(libc::ENOENT)) {
                            Ok(None)
                        } else { Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture) }
                    };
                    verify_absent()?;
                    registry.publish_kernel_effect_resolution(
                        &resolution,
                        Some(&mut verify_absent),
                        None,
                    )?;
                    continue;
                };
                let mut revoked = original.clone();
                let mut revoke_batch = None;
                let mut observed_rules = Vec::new();
                let mut reject_rules = Vec::new();
                let table_handle = if absent {
                    anyhow::ensure!(
                        original.posture == GatewayAccessPostureV1::Revoked,
                        "absent boundary has no durable Revoked revision"
                    );
                    effect
                        .resolution
                        .as_ref()
                        .and_then(|resolution| resolution.observed_nftables_table_handle)
                        .context("absent table has no durable cleanup identity")?
                } else {
                    let table = table?;
                    anyhow::ensure!(
                        table.len() == 1 && table[0].0 == 0x0a00,
                        "recovered table is not unique"
                    );
                    let fields = map(&table[0].1[4..])?;
                    anyhow::ensure!(
                        fields.get(&1) == Some(&[table_name.as_bytes(), &[0]].concat().as_slice()),
                        "recovered table name changed"
                    );
                    let handle = u64::from_be_bytes(
                        (*fields.get(&4).context("recovered table handle absent")?).try_into()?,
                    );
                    anyhow::ensure!(
                        handle != 0
                            && effect
                                .resolution
                                .as_ref()
                                .is_none_or(|resolution| resolution.observed_nftables_table_handle
                                    == Some(handle)),
                        "recovered table handle changed"
                    );
                    let chains = request(
                        4,
                        &[string(1, table_name), string(3, chain_name)].concat(),
                        false,
                    )?;
                    anyhow::ensure!(
                        chains.len() == 1 && chains[0].0 == 0x0a03,
                        "recovered chain is not unique"
                    );
                    let fields = map(&chains[0].1[4..])?;
                    let hook = map(fields.get(&4).context("recovered chain hook absent")?)?;
                    anyhow::ensure!(
                        fields.get(&1) == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                            && fields.get(&3)
                                == Some(&[chain_name.as_bytes(), &[0]].concat().as_slice())
                            && fields.get(&2)
                                == Some(
                                    &original
                                        .nftables_chain
                                        .chain_handle
                                        .to_be_bytes()
                                        .as_slice()
                                )
                            && fields.get(&5) == Some(&1u32.to_be_bytes().as_slice())
                            && fields.get(&7) == Some(&b"filter\0".as_slice())
                            && hook.len() == 2
                            && hook.get(&1) == Some(&3u32.to_be_bytes().as_slice())
                            && hook.get(&2) == Some(&(-100i32).to_be_bytes().as_slice()),
                        "recovered chain identity or hook changed"
                    );
                    let query = [string(1, table_name), string(2, chain_name)].concat();
                    let rules = request(7, &query, true)?;
                    let retained_rules: Vec<_> = if rules.len() == 1 {
                        original
                            .nftables_rules
                            .iter()
                            .filter(|rule| rule.role == NftablesRuleRoleV1::RejectRemainder)
                            .collect()
                    } else {
                        original.nftables_rules.iter().collect()
                    };
                    anyhow::ensure!(
                        rules.len() == retained_rules.len(),
                        "recovered rule inventory changed"
                    );
                    for ((kind, bytes), rule) in rules.iter().zip(&retained_rules) {
                        anyhow::ensure!(*kind == 0x0a06, "recovered rule type changed");
                        let fields = map(&bytes[4..])?;
                        let port = original.gateway_listener.port;
                        let mut wire = [
                            expression("meta", &[number(1, 1), number(2, 15)].concat()),
                            compare(&[2]),
                            expression("meta", &[number(1, 1), number(2, 16)].concat()),
                            compare(&[6]),
                            expression(
                                "payload",
                                &[number(1, 1), number(2, 1), number(3, 16), number(4, 4)].concat(),
                            ),
                            compare(&[127, 0, 0, 1]),
                            expression(
                                "payload",
                                &[number(1, 1), number(2, 2), number(3, 2), number(4, 2)].concat(),
                            ),
                            compare(&port.to_be_bytes()),
                        ]
                        .concat();
                        if rule.role == NftablesRuleRoleV1::RejectRemainder {
                            wire.extend(expression("reject", &number(1, 1)));
                        } else {
                            let cgroup = if rule.role == NftablesRuleRoleV1::ReadinessProbeAccept {
                                &original.readiness_probe_cgroup
                            } else {
                                &original.allowed_member_cgroup
                            };
                            let level =
                                u32::try_from(cgroup.cgroup_relative_path.split('/').count())?;
                            wire.extend(expression(
                                "socket",
                                &[number(1, 3), number(2, 1), number(3, level)].concat(),
                            ));
                            wire.extend(compare(&cgroup.cgroup_directory_inode.to_ne_bytes()));
                            wire.extend(expression(
                                "immediate",
                                &[
                                    number(1, 0),
                                    attribute(0x8002, &attribute(0x8002, &number(1, 1))),
                                ]
                                .concat(),
                            ));
                        }
                        anyhow::ensure!(
                            fields.get(&1)
                                == Some(&[table_name.as_bytes(), &[0]].concat().as_slice())
                                && fields.get(&2)
                                    == Some(&[chain_name.as_bytes(), &[0]].concat().as_slice())
                                && fields.get(&3)
                                    == Some(&rule.rule_handle.to_be_bytes().as_slice())
                                && matches_expression(
                                    fields.get(&4).context("recovered rule expression absent")?,
                                    &wire,
                                    0
                                )?,
                            "recovered rule identity or expression changed"
                        );
                    }
                    observed_rules = rules.clone();
                    reject_rules = rules
                        .iter()
                        .zip(&retained_rules)
                        .filter(|(_, rule)| rule.role == NftablesRuleRoleV1::RejectRemainder)
                        .map(|(wire, _)| wire.clone())
                        .collect();
                    anyhow::ensure!(
                        reject_rules.len() == 1,
                        "recovered reject rule is not unique"
                    );
                    if original.posture != GatewayAccessPostureV1::Revoked {
                        let mut batch = message(0x10, 1, 2000, 0, &[]);
                        let mut ack = Vec::new();
                        for rule in original
                            .nftables_rules
                            .iter()
                            .filter(|rule| rule.role != NftablesRuleRoleV1::RejectRemainder)
                        {
                            let sequence = 2001 + ack.len() as u32;
                            batch.extend(message(
                                0x0a08,
                                5,
                                sequence,
                                1,
                                &[query.clone(), attribute(3, &rule.rule_handle.to_be_bytes())]
                                    .concat(),
                            ));
                            ack.push(sequence);
                        }
                        batch.extend(message(0x11, 1, 2099, 0, &[]));
                        if rules.len() != 1 {
                            revoke_batch = Some((batch, ack));
                        }
                        let reject = original
                            .nftables_rules
                            .iter()
                            .find(|rule| rule.role == NftablesRuleRoleV1::RejectRemainder)
                            .context("missing reject rule")?;
                        revoked.revision = original
                            .revision
                            .checked_add(1)
                            .context("boundary revision overflow")?;
                        revoked.predecessor_ref = Some(GatewayAccessBoundaryRefV1 {
                            authority_store_id: original.authority_store_id.clone(),
                            access_boundary_id: original.access_boundary_id.clone(),
                            revision: original.revision,
                            boundary_hash: original.boundary_hash.clone(),
                        });
                        revoked.posture = GatewayAccessPostureV1::Revoked;
                        revoked.nftables_rules = vec![reject.clone()];
                        revoked.boundary_hash = seal(
                            "substrate.e3.gateway-access-boundary.v1",
                            "boundary",
                            "boundary_hash",
                            &revoked,
                        )?;
                    }
                    handle
                };
                let resolution = if let Some(existing) = &effect.resolution {
                    existing.clone()
                } else {
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: effect.intent.authority_store_id.clone(),
                        resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                        effect_intent_ref: original.kernel_effect_intent_ref.clone(),
                        disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                        observed_cgroup: None,
                        observed_nftables_table_handle: Some(table_handle),
                        resolved_at: Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        resolution_hash: String::new(),
                    };
                    resolution.resolution_hash = seal(
                        "substrate.e3.kernel-effect-resolution.v1",
                        "resolution",
                        "resolution_hash",
                        &resolution,
                    )?;
                    resolution
                };
                if historical {
                    anyhow::ensure!(
                        effect.resolution.as_ref() == Some(&resolution) && absent,
                        "resolved historical boundary reappeared"
                    );
                    pending_tables.push((socket, table_handle, absent, table_name, namespace));
                    continue;
                }
                let mut cleanup_error = None;
                let publication = registry.publish_kernel_effect_resolution(
                    &resolution,
                    Some(&mut || {
                        (|| -> Result<_> {
                            anyhow::ensure!(
                                namespace.metadata()?.ino() == *network_namespace_inode,
                                "recovered namespace changed before revocation"
                            );
                            if !absent {
                                let query = [string(1, table_name), string(2, chain_name)].concat();
                                anyhow::ensure!(
                                    request(7, &query, true)? == observed_rules,
                                    "recovered rules changed before revocation"
                                );
                                if let Some((batch, ack)) = &revoke_batch {
                                    exchange(socket.as_raw_fd(), batch, ack, false)?;
                                }
                                let remaining = request(7, &query, true)?;
                                anyhow::ensure!(
                                    remaining.len() == 1 && remaining[0].0 == reject_rules[0].0,
                                    "recovered reject inventory changed"
                                );
                                let observed = map(&remaining[0].1[4..])?;
                                let retained = map(&reject_rules[0].1[4..])?;
                                anyhow::ensure!(
                                    [1, 2, 3, 4]
                                        .iter()
                                        .all(|key| observed.get(key) == retained.get(key)),
                                    "recovered reject proof changed"
                                );
                            }
                            Ok(Some(revoked.clone()))
                        })()
                        .map_err(|error| {
                            cleanup_error = Some(error);
                            ConfigProjectionFailureV1::UnsupportedSecurityPosture
                        })
                    }),
                    expected.as_ref(),
                );
                if let Some(error) = cleanup_error {
                    return Err(error).context("recovered boundary revocation");
                }
                publication?;
                pending_tables.push((socket, table_handle, absent, table_name, namespace));
            }
            for (parent, component, child, effect) in &groups {
                if effect.resolution.is_some() {
                    anyhow::ensure!(
                        named(parent, component)?.is_none(),
                        "resolved recovered cgroup reappeared"
                    );
                    if historical {
                        continue;
                    }
                }
                let resolution = if let Some(existing) = &effect.resolution {
                    existing.clone()
                } else {
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: effect.intent.authority_store_id.clone(),
                        resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                        effect_intent_ref: E3KernelEffectIntentRefV1 {
                            authority_store_id: effect.intent.authority_store_id.clone(),
                            effect_intent_id: effect.intent.effect_intent_id.clone(),
                            intent_hash: effect.intent.intent_hash.clone(),
                        },
                        disposition: if effect.child_cgroup.is_some() {
                            E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent
                        } else {
                            E3KernelEffectResolutionDispositionV1::NoEffectObserved
                        },
                        observed_cgroup: effect
                            .child_cgroup
                            .as_ref()
                            .map(|registration| registration.cgroup.clone()),
                        observed_nftables_table_handle: None,
                        resolved_at: Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        resolution_hash: String::new(),
                    };
                    resolution.resolution_hash = seal(
                        "substrate.e3.kernel-effect-resolution.v1",
                        "resolution",
                        "resolution_hash",
                        &resolution,
                    )?;
                    resolution
                };
                let mut cleanup_error = None;
                let publication = registry.publish_kernel_effect_resolution(
                    &resolution,
                    Some(&mut || {
                        (|| -> Result<_> {
                            if let Some(stat) = named(parent, component)? {
                                let child = child
                                    .as_ref()
                                    .context("unclassified recovered cgroup appeared")?;
                                anyhow::ensure!(
                                    stat.st_dev == child.metadata()?.dev()
                                        && stat.st_ino == child.metadata()?.ino(),
                                    "recovered cgroup substituted"
                                );
                                empty_group(child)?;
                                anyhow::ensure!(
                                    unsafe {
                                        libc::unlinkat(
                                            parent.as_raw_fd(),
                                            component.as_ptr(),
                                            libc::AT_REMOVEDIR,
                                        )
                                    } == 0,
                                    "remove recovered cgroup"
                                );
                            }
                            anyhow::ensure!(
                                named(parent, component)?.is_none(),
                                "recovered cgroup not absent"
                            );
                            Ok(None)
                        })()
                        .map_err(|error| {
                            cleanup_error = Some(error);
                            ConfigProjectionFailureV1::UnsupportedSecurityPosture
                        })
                    }),
                    expected.as_ref(),
                );
                if let Some(error) = cleanup_error {
                    return Err(error).context("recovered cgroup cleanup");
                }
                publication?;
            }
            for (socket, handle, absent, table_name, _namespace) in pending_tables {
                let query = message(0x0a01, 5, 3000, 1, &string(1, table_name));
                if !absent {
                    exclusion.require_recovering_v1()?;
                    let table = exchange(socket.as_raw_fd(), &query, &[3000], false)?;
                    anyhow::ensure!(
                        table.len() == 1
                            && map(&table[0].1[4..])?.get(&4)
                                == Some(&handle.to_be_bytes().as_slice()),
                        "recovered table changed before deletion"
                    );
                    let delete = [
                        message(0x10, 1, 3001, 0, &[]),
                        message(0x0a02, 5, 3002, 1, &attribute(4, &handle.to_be_bytes())),
                        message(0x11, 1, 3003, 0, &[]),
                    ]
                    .concat();
                    exchange(socket.as_raw_fd(), &delete, &[3002], false)?;
                }
                let result = exchange(
                    socket.as_raw_fd(),
                    &message(0x0a01, 5, 3004, 1, &string(1, table_name)),
                    &[3004],
                    false,
                );
                anyhow::ensure!(
                    result
                        .as_ref()
                        .err()
                        .and_then(|error| error.downcast_ref::<std::io::Error>())
                        .is_some_and(|error| error.raw_os_error() == Some(libc::ENOENT)),
                    "recovered table not absent"
                );
            }
            if let Some(record) = record {
                let subject_readback = registry.recover(Some(&record.identity))?;
                let durable = registry.recover(None)?;
                let subject = subject_readback
                    .subject
                    .as_ref()
                    .context("recovered cleanup subject absent")?;
                let ConfigProjectionSubjectReadbackV1::Bound(metadata) = subject else {
                    anyhow::bail!("recovered cleanup subject is unbound");
                };
                anyhow::ensure!(
                    (metadata.projection_ref()
                        == expected
                            .as_ref()
                            .context("missing expected recovered head")?
                        && metadata.record() == record)
                        || (metadata.record().identity == record.identity
                            && metadata.record().revision > record.revision
                            && retained.is_some()
                            && attempt.iter().all(|effect| effect.resolution.is_some())),
                    "recovered cleanup head changed"
                );
                anyhow::ensure!(
                    attempt
                        .iter()
                        .all(
                            |effect| durable.kernel_effects.iter().any(|current| current.intent
                                == effect.intent
                                && current.resolution.is_some()
                                && current
                                    .boundary
                                    .as_ref()
                                    .is_none_or(|boundary| boundary.posture
                                        == GatewayAccessPostureV1::Revoked)
                                && current
                                    .terminal_child_evidence
                                    .iter()
                                    .any(|proof| Some(&proof.final_projection_ref)
                                        == expected.as_ref()))
                        ),
                    "recorded config cleanup lacks complete durable resolution proof"
                );
                let config = attempt
                    .iter()
                    .find_map(|effect| effect.gateway_config.as_ref())
                    .context("recovered config binding absent")?;
                anyhow::ensure!(
                    attempt
                        .iter()
                        .all(|effect| effect.gateway_config.as_ref() == Some(config)),
                    "recovered config copies disagree"
                );
                let boundary = attempt
                    .iter()
                    .find_map(|effect| effect.boundary.as_ref())
                    .context("recovered listener binding absent")?;
                let binding = resolve_gateway_backend_binding(&record.identity.backend_id)
                    .context("recovered backend binding absent")?;
                let bytes =
                    render_integrated_config(boundary.gateway_listener.port, binding).into_bytes();
                anyhow::ensure!(
                    config.root.physical_path
                        == format!("/run/substrate/e3-gateway/{series}/{fence}")
                        && config.relative_path == "config.toml"
                        && config.mode == 0o600
                        && config.byte_length == bytes.len() as u64
                        && config.sha256 == format!("{:x}", Sha256::digest(&bytes)),
                    "recorded runtime config binding changed"
                );
                let home = open_directory(&record.identity.accepted_home.physical_path)?;
                record
                    .identity
                    .accepted_home
                    .revalidate_linux_from_fd(home.as_fd())?;
                let home_meta = home.metadata()?;
                anyhow::ensure!(
                    u64::from(home_meta.uid()) == record.native.root.owner_uid
                        && u64::from(home_meta.gid()) == record.native.root.owner_gid,
                    "recovered config owner differs from accepted home"
                );
                for path in [
                    "/run".to_owned(),
                    "/run/substrate".to_owned(),
                    "/run/substrate/e3-gateway".to_owned(),
                    format!("/run/substrate/e3-gateway/{series}"),
                ] {
                    let file = open_directory(&path)?;
                    let meta = file.metadata()?;
                    anyhow::ensure!(
                        meta.uid() == 0 && meta.mode() & 0o022 == 0,
                        "recovered config parent is unprotected"
                    );
                    if path.starts_with("/run/substrate/e3-gateway") {
                        anyhow::ensure!(
                            meta.mode() & 0o7777 == 0o711,
                            "recovered config parent mode changed"
                        );
                    }
                }
                let parent = open_directory(&format!("/run/substrate/e3-gateway/{series}"))?;
                let component = std::ffi::CString::new(fence.as_str())?;
                if let Some(stat) = named(&parent, &component)? {
                    anyhow::ensure!(
                        !active_history && metadata.record() == record,
                        "terminal or historical runtime config reappeared"
                    );
                    exclusion.require_recovering_v1()?;
                    let root = open_directory(&config.root.physical_path)?;
                    config.root.revalidate_linux_from_fd(root.as_fd())?;
                    let meta = root.metadata()?;
                    anyhow::ensure!(
                        stat.st_dev == meta.dev()
                            && stat.st_ino == meta.ino()
                            && meta.uid() == home_meta.uid()
                            && meta.gid() == home_meta.gid()
                            && meta.mode() & 0o7777 == 0o700,
                        "recovered config root changed"
                    );
                    let names = fs::read_dir(format!("/proc/self/fd/{}", root.as_raw_fd()))?
                        .map(|entry| entry.map(|entry| entry.file_name()))
                        .collect::<std::io::Result<Vec<_>>>()?;
                    anyhow::ensure!(
                        names.is_empty() || names == [std::ffi::OsString::from("config.toml")],
                        "unexpected recovered config entries"
                    );
                    if !names.is_empty() {
                        let raw = unsafe {
                            libc::openat(
                                root.as_raw_fd(),
                                c"config.toml".as_ptr(),
                                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                            )
                        };
                        if raw < 0 {
                            return Err(std::io::Error::last_os_error())
                                .context("open recorded runtime config");
                        }
                        let file = unsafe { fs::File::from_raw_fd(raw) };
                        let meta = file.metadata()?;
                        anyhow::ensure!(
                            meta.is_file()
                                && meta.nlink() == 1
                                && meta.uid() == home_meta.uid()
                                && meta.gid() == home_meta.gid()
                                && meta.mode() & 0o7777 == config.mode
                                && meta.len() == config.byte_length
                                && meta.len() <= 65536,
                            "recovered config file metadata changed"
                        );
                        let mut observed = vec![0; bytes.len()];
                        file.read_exact_at(&mut observed, 0)?;
                        anyhow::ensure!(observed == bytes, "recovered config bytes changed");
                        let current = named(&root, c"config.toml")?
                            .context("recovered config disappeared during read")?;
                        anyhow::ensure!(
                            current.st_dev == meta.dev()
                                && current.st_ino == meta.ino()
                                && current.st_nlink == 1
                                && current.st_mode & libc::S_IFMT == libc::S_IFREG,
                            "recovered config substituted during read"
                        );
                        anyhow::ensure!(
                            unsafe { libc::unlinkat(root.as_raw_fd(), c"config.toml".as_ptr(), 0) }
                                == 0,
                            "unlink recorded runtime config"
                        );
                    }
                    root.sync_all()?;
                    anyhow::ensure!(
                        unsafe {
                            libc::unlinkat(
                                parent.as_raw_fd(),
                                component.as_ptr(),
                                libc::AT_REMOVEDIR,
                            )
                        } == 0,
                        "unlink recorded runtime config root"
                    );
                }
                anyhow::ensure!(
                    named(&parent, &component)?.is_none(),
                    "recorded runtime config root remains"
                );
                parent.sync_all()?;
            }
        }
        Ok(())
    }

    fn revoke_live_v1(
        &mut self,
        registry: &config_projection::ConfigProjectionRegistryV1,
    ) -> Result<()> {
        use config_projection::{
            ConfigProjectionFailureV1 as Failure, E3KernelEffectKindV1,
            E3KernelEffectResolutionDispositionV1, E3KernelEffectResolutionV1,
            GatewayAccessPostureV1,
        };
        use std::os::unix::fs::MetadataExt;
        fn remove_empty(
            parent: &std::fs::File,
            child: &std::fs::File,
            registration: &config_projection::E3ChildCgroupRegistrationV1,
            intent: &config_projection::E3KernelEffectIntentV1,
            remove: bool,
            removed: &mut bool,
        ) -> Result<()> {
            let E3KernelEffectKindV1::CreateChildCgroup {
                child_component,
                parent_cgroup,
                ..
            } = &intent.effect
            else {
                anyhow::bail!("invalid E3 cleanup cgroup intent")
            };
            let component = std::ffi::CString::new(child_component.as_str())?;
            let parent_meta = parent.metadata()?;
            anyhow::ensure!(
                std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim()
                    == registration.kernel_boot_id,
                "E3 cleanup boot identity changed"
            );
            anyhow::ensure!(
                parent_meta.dev() == parent_cgroup.cgroup_v2_mount_device_id
                    && parent_meta.ino() == parent_cgroup.cgroup_directory_inode
                    && parent_meta.uid() == 0
                    && parent_meta.mode() & 0o022 == 0,
                "E3 cleanup parent identity changed"
            );
            let mount = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                .open("/sys/fs/cgroup")?;
            let mount_metadata = mount.metadata()?;
            let mut mount_fs: libc::statfs = unsafe { std::mem::zeroed() };
            anyhow::ensure!(
                unsafe { libc::fstatfs(mount.as_raw_fd(), &mut mount_fs) } == 0
                    && mount_fs.f_type as i128 == 0x6367_7270i128
                    && mount_metadata.dev() == parent_cgroup.cgroup_v2_mount_device_id
                    && mount_metadata.ino() == parent_cgroup.cgroup_v2_mount_inode
                    && registration.cgroup.cgroup_v2_mount_device_id
                        == parent_cgroup.cgroup_v2_mount_device_id
                    && registration.cgroup.cgroup_v2_mount_inode
                        == parent_cgroup.cgroup_v2_mount_inode,
                "E3 cleanup cgroup mount changed"
            );
            let mut named_parent = mount;
            for name in parent_cgroup.cgroup_relative_path.split('/') {
                anyhow::ensure!(
                    !name.is_empty() && name != "." && name != "..",
                    "E3 cleanup parent path is noncanonical"
                );
                let name = std::ffi::CString::new(name)?;
                let raw = unsafe {
                    libc::openat(
                        named_parent.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                    )
                };
                anyhow::ensure!(raw >= 0, "E3 cleanup parent path disappeared");
                named_parent = unsafe { fs::File::from_raw_fd(raw) };
            }
            let named_parent_metadata = named_parent.metadata()?;
            anyhow::ensure!(
                named_parent_metadata.dev() == parent_meta.dev()
                    && named_parent_metadata.ino() == parent_meta.ino(),
                "E3 cleanup parent pathname was substituted"
            );
            let child_meta = child.metadata()?;
            anyhow::ensure!(
                child_meta.dev() == registration.cgroup.cgroup_v2_mount_device_id
                    && child_meta.ino() == registration.cgroup.cgroup_directory_inode,
                "E3 cleanup child descriptor changed"
            );
            let mut current: libc::stat = unsafe { std::mem::zeroed() };
            let found = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    &mut current,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if found == 0 {
                anyhow::ensure!(!*removed, "E3 removed cgroup reappeared");
                anyhow::ensure!(
                    current.st_dev == child_meta.dev()
                        && current.st_ino == child_meta.ino()
                        && current.st_mode & libc::S_IFMT == libc::S_IFDIR,
                    "E3 cleanup cgroup name was substituted"
                );
                let mut observations = Vec::with_capacity(2);
                for _ in 0..2 {
                    for entry in fs::read_dir(format!("/proc/self/fd/{}", child.as_raw_fd()))? {
                        anyhow::ensure!(
                            !entry?.file_type()?.is_dir(),
                            "E3 cleanup has unclassified descendant cgroups"
                        );
                    }
                    let mut observation = Vec::new();
                    for control in [c"cgroup.events", c"cgroup.procs", c"cgroup.threads"] {
                        let fd = unsafe {
                            libc::openat(
                                child.as_raw_fd(),
                                control.as_ptr(),
                                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                            )
                        };
                        if fd < 0 {
                            return Err(std::io::Error::last_os_error())
                                .context("read E3 cleanup cgroup state");
                        }
                        let file = unsafe { std::fs::File::from_raw_fd(fd) };
                        let metadata = file.metadata()?;
                        anyhow::ensure!(
                            metadata.uid() == 0
                                && metadata.mode() & 0o022 == 0
                                && metadata.dev() == child_meta.dev(),
                            "E3 cleanup control ownership changed"
                        );
                        let mut bytes = String::new();
                        std::io::Read::read_to_string(
                            &mut std::io::Read::take(file, 4096),
                            &mut bytes,
                        )?;
                        anyhow::ensure!(
                            bytes.len() < 4096
                                && if control == c"cgroup.events" {
                                    bytes
                                        .lines()
                                        .filter(|line| line.starts_with("populated "))
                                        .collect::<Vec<_>>()
                                        == ["populated 0"]
                                } else {
                                    bytes.is_empty()
                                },
                            "E3 cleanup cgroup is populated"
                        );
                        observation.push(bytes);
                    }
                    observations.push(observation);
                }
                anyhow::ensure!(
                    observations[0] == observations[1],
                    "E3 cgroup changed across empty readback"
                );
                if !remove {
                    return Ok(());
                }
                // rmdir is also the atomic no-descendants check: a nonempty subtree is never
                // recursively removed by child-free preparation cleanup.
                if unsafe {
                    libc::unlinkat(parent.as_raw_fd(), component.as_ptr(), libc::AT_REMOVEDIR)
                } != 0
                {
                    return Err(std::io::Error::last_os_error()).context("remove empty E3 cgroup");
                }
                *removed = true;
            } else {
                anyhow::ensure!(
                    *removed
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT),
                    "E3 cleanup cannot prove cgroup absence"
                );
            }
            let found = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    &mut current,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            anyhow::ensure!(
                found == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT),
                "E3 removed cgroup was not observed absent"
            );
            Ok(())
        }
        anyhow::ensure!(
            self.pending_cgroup.is_none() && self.pending_boundary.is_none(),
            "E3 unresolved kernel effect requires retained recovery ownership"
        );
        anyhow::ensure!(
            self.unresolved_intents.iter().all(|intent| {
                self.cgroups.iter().any(|(_, _, _, owned)| owned == intent)
                    || self
                        .boundary
                        .as_ref()
                        .is_some_and(|(owned, _, _, _, _)| owned == intent)
            }),
            "E3 intent without complete effect readback requires retained recovery ownership"
        );
        // Kill/reap is complete. Independently verify all preallocated groups before releasing
        // namespace ownership, including groups whose child never reached namespace retention.
        for (parent, child, registration, intent) in &self.cgroups {
            remove_empty(
                parent,
                child,
                registration,
                intent,
                false,
                self.removed_cgroups
                    .entry(registration.cgroup_registration_id.clone())
                    .or_default(),
            )?;
        }
        if let Some(launch) = &self.launch {
            for child in [launch.probe.as_deref(), Some(launch)]
                .into_iter()
                .flatten()
            {
                if child.namespace_setup_started {
                    self._exclusion.release_terminal_child_user_namespace_v1(
                        child
                            .gateway_registration
                            .as_ref()
                            .context("lost E3 setup registration")?,
                    )?;
                }
            }
        }
        let effects = if self.unresolved_intents.is_empty() {
            Vec::new()
        } else {
            registry.recover(None)?.kernel_effects
        };
        let attempt: Vec<_> = effects
            .iter()
            .filter(|effect| {
                self.unresolved_intents
                    .iter()
                    .any(|intent| intent == &effect.intent)
            })
            .collect();
        let record = attempt.iter().find_map(|effect| effect.projection.as_ref());
        let expected_head = record.map(|record| config_projection::ConfigProjectionRefV1 {
            authority_store_id: record.identity.authority_store_id.clone(),
            series_id: record.identity.series_id.clone(),
            record_id: record.record_id.clone(),
            revision: record.revision,
            record_hash: record.record_hash.clone(),
        });
        if self.terminal_evidence.is_none() {
            if let (Some(record), Some(expected)) = (record, expected_head.as_ref()) {
                let mut current_processes = Vec::new();
                for registration in attempt.iter().flat_map(|effect| &effect.child_processes) {
                    let main = self
                        .launch
                        .as_ref()
                        .context("registered E3 child lacks retained cleanup owner")?;
                    let owner = [Some(main), main.probe.as_deref()]
                        .into_iter()
                        .flatten()
                        .find(|owner| owner.gateway_registration.as_ref() == Some(registration))
                        .context("registered E3 child differs from exact retained owner")?;
                    let status = owner
                        .gateway_wait_status
                        .context("registered E3 child was not reaped")?;
                    current_processes.push(config_projection::E3TerminalProcessObservationV1 {
                        registration_id: registration.registration_id.clone(),
                        registration_hash: registration.registration_hash.clone(),
                        role: registration.role,
                        pid: registration.pid,
                        pid_start_time_ticks: registration.pid_start_time_ticks,
                        observation:
                            config_projection::E3TerminalProcessObservationKindV1::ParentWaitid {
                                terminal_wait_status: status,
                            },
                    });
                }
                let retained: Vec<_> = attempt
                    .iter()
                    .flat_map(|effect| effect.terminal_child_evidence.iter())
                    .filter(|evidence| evidence.final_projection_ref == *expected)
                    .collect();
                let mut groups = Vec::new();
                for (parent, child, registration, intent) in &self.cgroups {
                    let E3KernelEffectKindV1::CreateChildCgroup {
                        child_component, ..
                    } = &intent.effect
                    else {
                        anyhow::bail!("invalid cgroup cleanup intent");
                    };
                    let component = std::ffi::CString::new(child_component.as_str())?;
                    let mut named: libc::stat = unsafe { std::mem::zeroed() };
                    if unsafe {
                        libc::fstatat(
                            parent.as_raw_fd(),
                            component.as_ptr(),
                            &mut named,
                            libc::AT_SYMLINK_NOFOLLOW,
                        )
                    } == -1
                    {
                        anyhow::ensure!(
                            std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT)
                                && retained.first().is_some_and(|proof| proof
                                    .ordered_empty_cgroups
                                    .iter()
                                    .any(|entry| entry.cgroup_registration_id
                                        == registration.cgroup_registration_id
                                        && entry.cgroup_registration_hash
                                            == registration.cgroup_registration_hash)),
                            "cgroup absence has no retained terminal proof"
                        );
                        remove_empty(
                            parent,
                            child,
                            registration,
                            intent,
                            false,
                            self.removed_cgroups
                                .entry(registration.cgroup_registration_id.clone())
                                .or_default(),
                        )?;
                        continue;
                    }
                    anyhow::ensure!(
                        named.st_dev == registration.cgroup.cgroup_v2_mount_device_id
                            && named.st_ino == registration.cgroup.cgroup_directory_inode
                            && named.st_mode & libc::S_IFMT == libc::S_IFDIR,
                        "cgroup terminal observation name changed"
                    );
                    let mut observed = None;
                    for _ in 0..2 {
                        let root = format!("/proc/self/fd/{}", child.as_raw_fd());
                        let directories = fs::read_dir(&root)?
                            .map(|entry| entry.and_then(|entry| entry.file_type()))
                            .collect::<std::io::Result<Vec<_>>>()?;
                        anyhow::ensure!(
                            directories.iter().all(|kind| !kind.is_dir()),
                            "live cleanup has descendant cgroups"
                        );
                        let read_control = |name: &std::ffi::CStr| -> Result<Vec<u8>> {
                            let raw = unsafe {
                                libc::openat(
                                    child.as_raw_fd(),
                                    name.as_ptr(),
                                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                                )
                            };
                            if raw == -1 {
                                return Err(std::io::Error::last_os_error())
                                    .context("read terminal cgroup control");
                            }
                            let file = unsafe { fs::File::from_raw_fd(raw) };
                            let metadata = file.metadata()?;
                            anyhow::ensure!(
                                metadata.dev() == named.st_dev
                                    && metadata.uid() == 0
                                    && metadata.mode() & 0o022 == 0,
                                "terminal cgroup control changed"
                            );
                            let mut bytes = Vec::new();
                            std::io::Read::read_to_end(
                                &mut std::io::Read::take(file, 4096),
                                &mut bytes,
                            )?;
                            anyhow::ensure!(
                                bytes.len() < 4096,
                                "terminal cgroup control exceeds bound"
                            );
                            Ok(bytes)
                        };
                        let events = read_control(c"cgroup.events")?;
                        let procs = read_control(c"cgroup.procs")?;
                        let threads = read_control(c"cgroup.threads")?;
                        anyhow::ensure!(
                            procs.is_empty()
                                && threads.is_empty()
                                && std::str::from_utf8(&events)?
                                    .lines()
                                    .filter(|line| line.starts_with("populated "))
                                    .collect::<Vec<_>>()
                                    == ["populated 0"],
                            "live cleanup is not quiescent"
                        );
                        let current = (events, procs);
                        if let Some(previous) = &observed {
                            anyhow::ensure!(previous == &current, "cgroup quiescence changed");
                        }
                        observed = Some(current);
                    }
                    let (events, procs) = observed.context("missing cgroup observations")?;
                    use sha2::{Digest, Sha256};
                    groups.push(config_projection::E3TerminalCgroupQuiescenceV1 {
                        cgroup_registration_id: registration.cgroup_registration_id.clone(),
                        cgroup_registration_hash: registration.cgroup_registration_hash.clone(),
                        role: registration.role,
                        cgroup: registration.cgroup.clone(),
                        cgroup_events_sha256: format!("{:x}", Sha256::digest(events)),
                        cgroup_procs_sha256: format!("{:x}", Sha256::digest(procs)),
                        populated: false,
                        ordered_live_pids: Vec::new(),
                    });
                }
                let mut historical_processes = current_processes;
                // ReadyClosed advances the same attempt; its Dormant predecessor is not a
                // previously cleaned attempt and cannot supply terminal cleanup evidence.
                if let Some(predecessor) = record.predecessor_ref.as_ref().filter(|_| {
                    record.managed_gateway.posture
                        == config_projection::ManagedGatewayProjectionPostureV1::Dormant
                }) {
                    anyhow::ensure!(
                        effects
                            .iter()
                            .filter(|effect| effect
                                .projection
                                .as_ref()
                                .is_some_and(|prior| prior.identity == record.identity
                                    && prior.revision <= predecessor.revision))
                            .all(|effect| effect.resolution.is_some()),
                        "live successor has unresolved history"
                    );
                    let proofs: Vec<_> = effects
                        .iter()
                        .flat_map(|effect| &effect.terminal_child_evidence)
                        .filter(|proof| proof.final_projection_ref == *predecessor)
                        .collect();
                    let proof = proofs
                        .first()
                        .context("live successor lacks its exact predecessor cleanup proof")?;
                    anyhow::ensure!(
                        proofs.iter().all(|other| other == proof),
                        "live successor has competing predecessor evidence"
                    );
                    // The predecessor object already covers its complete historical registration set.
                    // Different historical final refs may legitimately retain different observations;
                    // never splice those older objects into or rewrite this exact complete proof.
                    historical_processes.extend(proof.ordered_terminal_processes.clone());
                    groups.extend(proof.ordered_empty_cgroups.clone());
                }
                let rank = |role| match role {
                    config_projection::E3TerminalProcessRoleV1::ManagedGateway => 0,
                    config_projection::E3TerminalProcessRoleV1::Codex => 1,
                    config_projection::E3TerminalProcessRoleV1::ReadinessProbe => 2,
                };
                groups.sort_by_key(|group| {
                    (
                        rank(group.role),
                        group.cgroup.cgroup_v2_mount_device_id,
                        group.cgroup.cgroup_v2_mount_inode,
                        group.cgroup.cgroup_directory_inode,
                        group.cgroup.cgroup_relative_path.clone(),
                        group.cgroup_registration_id.clone(),
                    )
                });
                historical_processes.sort_by_key(|process| {
                    (
                        rank(process.role),
                        process.pid,
                        process.pid_start_time_ticks,
                        process.registration_id.clone(),
                    )
                });
                let evidence = if let Some(original) = retained.first() {
                    anyhow::ensure!(
                        retained.iter().all(|other| other == original),
                        "ambiguous terminal evidence"
                    );
                    (*original).clone()
                } else {
                    let mut evidence = config_projection::E3TerminalChildQuiescenceEvidenceV1 {
                        schema_version: 1,
                        authority_store_id: record.identity.authority_store_id.clone(),
                        series_id: record.identity.series_id.clone(),
                        evidence_id: format!("tce_{}", uuid::Uuid::now_v7()),
                        final_projection_ref: expected.clone(),
                        world_id: record.identity.world_id.clone(),
                        world_generation: record.identity.world_generation,
                        ordered_terminal_processes: historical_processes,
                        ordered_empty_cgroups: groups,
                        observed_at: config_projection::Timestamp(
                            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        ),
                        evidence_hash: String::new(),
                    };
                    let mut value = serde_json::to_value(&evidence)?;
                    value
                        .as_object_mut()
                        .context("invalid evidence object")?
                        .remove("evidence_hash");
                    evidence.evidence_hash =
                        config_projection::ConfigProjectionCodecV1::domain_sha256(
                            "substrate.e3.terminal-child-quiescence.v1",
                            &serde_json::json!({"evidence": value}),
                        )?;
                    evidence
                };
                self.terminal_evidence = Some(evidence);
            }
        }
        if let Some(evidence) = &self.terminal_evidence {
            anyhow::ensure!(
                Some(&evidence.final_projection_ref) == expected_head.as_ref(),
                "E3 terminal cleanup head changed"
            );
            registry.publish_terminal_child_evidence(evidence)?;
        }
        if self.cleanup_resolutions.is_empty() {
            let created_at = config_projection::Timestamp(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            );
            let mut resolutions = Vec::new();
            let mut allocate =
                |intent: &config_projection::E3KernelEffectIntentV1, cgroup, table| -> Result<()> {
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: intent.authority_store_id.clone(),
                        resolution_id: format!("ekr_{}", uuid::Uuid::now_v7()),
                        effect_intent_ref: config_projection::E3KernelEffectIntentRefV1 {
                            authority_store_id: intent.authority_store_id.clone(),
                            effect_intent_id: intent.effect_intent_id.clone(),
                            intent_hash: intent.intent_hash.clone(),
                        },
                        disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                        observed_cgroup: cgroup,
                        observed_nftables_table_handle: table,
                        resolved_at: created_at.clone(),
                        resolution_hash: String::new(),
                    };
                    let mut value = serde_json::to_value(&resolution)?;
                    value
                        .as_object_mut()
                        .context("E3 resolution is not an object")?
                        .remove("resolution_hash");
                    resolution.resolution_hash =
                        config_projection::ConfigProjectionCodecV1::domain_sha256(
                            "substrate.e3.kernel-effect-resolution.v1",
                            &serde_json::json!({"resolution":value}),
                        )?;
                    resolutions.push(resolution);
                    Ok(())
                };
            if let Some((intent, table, _, _, _)) = &self.boundary {
                allocate(intent, None, Some(*table))?;
            }
            for (_, _, registration, intent) in &self.cgroups {
                allocate(intent, Some(registration.cgroup.clone()), None)?;
            }
            self.cleanup_resolutions = resolutions;
        }
        if let Some((intent, _, _, _, _)) = self.boundary.clone() {
            let resolution = self
                .cleanup_resolutions
                .iter()
                .find(|r| r.effect_intent_ref.effect_intent_id == intent.effect_intent_id)
                .context("E3 boundary resolution is absent")?
                .clone();
            if self.cleanup_boundary.is_none() {
                self.cleanup_boundary = attempt
                    .iter()
                    .find_map(|effect| effect.boundary.as_ref())
                    .map(|prior| -> Result<_> {
                        let mut revoked = prior.clone();
                        if revoked.posture != GatewayAccessPostureV1::Revoked {
                            revoked.revision = revoked
                                .revision
                                .checked_add(1)
                                .context("boundary revision overflow")?;
                            revoked.predecessor_ref =
                                Some(config_projection::GatewayAccessBoundaryRefV1 {
                                    authority_store_id: prior.authority_store_id.clone(),
                                    access_boundary_id: prior.access_boundary_id.clone(),
                                    revision: prior.revision,
                                    boundary_hash: prior.boundary_hash.clone(),
                                });
                            let (_, _, chain, rules, posture) = self
                                .boundary
                                .as_ref()
                                .context("missing observed revoked boundary")?;
                            anyhow::ensure!(
                                *posture == GatewayAccessPostureV1::Revoked,
                                "boundary was not revoked"
                            );
                            revoked.posture = *posture;
                            revoked.nftables_chain = chain.clone();
                            revoked.nftables_rules = rules.clone();
                            let mut value = serde_json::to_value(&revoked)?;
                            value
                                .as_object_mut()
                                .context("invalid boundary")?
                                .remove("boundary_hash");
                            revoked.boundary_hash =
                                config_projection::ConfigProjectionCodecV1::domain_sha256(
                                    "substrate.e3.gateway-access-boundary.v1",
                                    &serde_json::json!({"boundary": value}),
                                )?;
                        }
                        Ok(revoked)
                    })
                    .transpose()?;
            }
            let mut cleanup_error = None;
            let publication = registry.publish_kernel_effect_resolution(
                &resolution,
                Some(&mut || {
                    (|| -> Result<Option<config_projection::GatewayAccessBoundaryV1>> {
                        self.install_dormant_boundary(
                            &intent,
                            &resolution.effect_intent_ref,
                            GatewayAccessPostureV1::Revoked,
                            false,
                        )?;
                        let revoked = self.cleanup_boundary.clone();
                        for (parent, child, registration, cgroup_intent) in &self.cgroups {
                            remove_empty(
                                parent,
                                child,
                                registration,
                                cgroup_intent,
                                false,
                                self.removed_cgroups
                                    .entry(registration.cgroup_registration_id.clone())
                                    .or_default(),
                            )?;
                        }
                        Ok(revoked)
                    })()
                    .map_err(|error| {
                        cleanup_error = Some(error);
                        Failure::UnsupportedSecurityPosture
                    })
                }),
                expected_head.as_ref(),
            );
            if let Some(error) = cleanup_error {
                return Err(error).context("E3 boundary cleanup");
            }
            publication?;
            // The observed Revoked revision is now durable. Retain its handles and the
            // empty cgroups through any lost publication return; a retry observes them again.
            self.listener.take();
            for (parent, child, registration, cgroup_intent) in &self.cgroups {
                remove_empty(
                    parent,
                    child,
                    registration,
                    cgroup_intent,
                    true,
                    self.removed_cgroups
                        .entry(registration.cgroup_registration_id.clone())
                        .or_default(),
                )?;
            }
            self.boundary_removal_attempted = true;
            self.install_dormant_boundary(
                &intent,
                &resolution.effect_intent_ref,
                GatewayAccessPostureV1::Revoked,
                true,
            )?;
        } else {
            self.listener.take();
        }
        for (parent, child, registration, intent) in &self.cgroups {
            let resolution = self
                .cleanup_resolutions
                .iter()
                .find(|r| r.effect_intent_ref.effect_intent_id == intent.effect_intent_id)
                .context("E3 cgroup resolution is absent")?;
            registry.publish_kernel_effect_resolution(
                resolution,
                Some(&mut || {
                    remove_empty(
                        parent,
                        child,
                        registration,
                        intent,
                        true,
                        self.removed_cgroups
                            .entry(registration.cgroup_registration_id.clone())
                            .or_default(),
                    )
                    .map(|()| None)
                    .map_err(|_| Failure::UnsupportedSecurityPosture)
                }),
                expected_head.as_ref(),
            )?;
        }
        if let Some((parent, component, root, config, identity)) = &self.gateway_config {
            use sha2::{Digest, Sha256};
            use std::os::fd::AsFd;
            use std::os::unix::fs::FileExt;
            let root = root
                .as_ref()
                .context("E3 config directory lacks retained identity")?;
            let root_meta = root.metadata()?;
            let (uid, gid) = self
                .gateway_config_owner
                .context("E3 config owner binding missing")?;
            anyhow::ensure!(
                root_meta.uid() == uid
                    && root_meta.gid() == gid
                    && root_meta.mode() & 0o7777 == 0o700,
                "E3 config root ownership changed"
            );
            let mut current: libc::stat = unsafe { std::mem::zeroed() };
            let found = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    component.as_ptr(),
                    &mut current,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if found == 0 {
                anyhow::ensure!(
                    !self.gateway_config_removed.1,
                    "E3 removed config directory reappeared"
                );
                anyhow::ensure!(
                    current.st_dev == root_meta.dev()
                        && current.st_ino == root_meta.ino()
                        && current.st_mode & libc::S_IFMT == libc::S_IFDIR,
                    "E3 config directory was substituted"
                );
                if let Some(identity) = identity {
                    identity.root.revalidate_linux_from_fd(root.as_fd())?;
                }
                let entries = fs::read_dir(format!("/proc/self/fd/{}", root.as_raw_fd()))?
                    .map(|entry| entry.map(|entry| entry.file_name()))
                    .collect::<std::io::Result<Vec<_>>>()?;
                anyhow::ensure!(
                    entries.is_empty() || entries == [std::ffi::OsString::from("config.toml")],
                    "E3 config directory has unknown entries"
                );
                anyhow::ensure!(
                    !entries.is_empty()
                        || self.gateway_config_removed.0
                        || (config.is_none() && identity.is_none()),
                    "E3 config file disappeared before cleanup"
                );
                if !entries.is_empty() {
                    anyhow::ensure!(
                        !self.gateway_config_removed.0,
                        "E3 removed config file reappeared"
                    );
                    let file = config
                        .as_ref()
                        .context("E3 config file lacks retained identity")?;
                    let metadata = file.metadata()?;
                    anyhow::ensure!(
                        metadata.uid() == uid
                            && metadata.gid() == gid
                            && metadata.mode() & 0o7777 == 0o600
                            && metadata.len() == self.gateway_config_bytes.len() as u64
                            && metadata.len() <= 65_536,
                        "E3 config metadata changed"
                    );
                    let mut bytes = vec![0; self.gateway_config_bytes.len()];
                    file.read_exact_at(&mut bytes, 0)?;
                    anyhow::ensure!(
                        bytes == self.gateway_config_bytes,
                        "E3 config bytes changed"
                    );
                    if let Some(identity) = identity {
                        anyhow::ensure!(
                            identity.relative_path == "config.toml"
                                && identity.mode == 0o600
                                && identity.byte_length == metadata.len()
                                && identity.sha256 == format!("{:x}", Sha256::digest(&bytes)),
                            "E3 config immutable identity changed"
                        );
                    }
                    anyhow::ensure!(
                        unsafe {
                            libc::fstatat(
                                root.as_raw_fd(),
                                c"config.toml".as_ptr(),
                                &mut current,
                                libc::AT_SYMLINK_NOFOLLOW,
                            )
                        } == 0
                            && current.st_dev == metadata.dev()
                            && current.st_ino == metadata.ino()
                            && current.st_mode & libc::S_IFMT == libc::S_IFREG
                            && current.st_nlink == 1,
                        "E3 config file was substituted"
                    );
                    if unsafe { libc::unlinkat(root.as_raw_fd(), c"config.toml".as_ptr(), 0) } != 0
                    {
                        return Err(std::io::Error::last_os_error())
                            .context("remove E3 config file");
                    }
                    self.gateway_config_removed.0 = true;
                }
                root.sync_all()?;
                if unsafe {
                    libc::unlinkat(parent.as_raw_fd(), component.as_ptr(), libc::AT_REMOVEDIR)
                } != 0
                {
                    return Err(std::io::Error::last_os_error())
                        .context("remove E3 config directory");
                }
                self.gateway_config_removed.1 = true;
            } else {
                anyhow::ensure!(
                    std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT)
                        && self.gateway_config_removed.1,
                    "E3 config directory absence is unproven"
                );
            }
            anyhow::ensure!(
                unsafe {
                    libc::fstatat(
                        parent.as_raw_fd(),
                        component.as_ptr(),
                        &mut current,
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT),
                "E3 config cleanup did not remove the exact realization"
            );
            parent.sync_all()?;
        }
        self.gateway_config = None;
        self.gateway_config_owner = None;
        self.gateway_config_bytes.clear();
        self.cgroups.clear();
        self.boundary = None;
        self.bound_listener = None;
        self.network_namespace = None;
        self.unresolved_intents.clear();
        self.launch.take();
        self.revoked = true;
        Ok(())
    }

    pub(crate) fn validate_listener_inventory(&self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        let socket = self
            .listener
            .as_ref()
            .context("E3 listener is not reserved")?;
        let namespace = self
            .network_namespace
            .as_ref()
            .context("E3 network namespace is not retained")?;
        let (port, inode, namespace_inode) = self.bound_listener.context("E3 binding is absent")?;
        for (option, expected) in [
            (libc::SO_DOMAIN, libc::AF_INET),
            (libc::SO_TYPE, libc::SOCK_STREAM),
            (libc::SO_PROTOCOL, libc::IPPROTO_TCP),
            (libc::SO_ACCEPTCONN, i32::from(self.listening)),
            (libc::SO_REUSEADDR, 0),
            (libc::SO_REUSEPORT, 0),
        ] {
            let mut value: i32 = -1;
            let mut length = std::mem::size_of_val(&value) as libc::socklen_t;
            anyhow::ensure!(
                unsafe {
                    libc::getsockopt(
                        socket.as_raw_fd(),
                        libc::SOL_SOCKET,
                        option,
                        (&mut value as *mut i32).cast(),
                        &mut length,
                    )
                } == 0
                    && length as usize == std::mem::size_of_val(&value)
                    && value == expected,
                "E3 pre-listen socket state mismatch"
            );
        }
        let mut address: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut length = std::mem::size_of_val(&address) as libc::socklen_t;
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        let fd_flags = unsafe { libc::fcntl(socket.as_raw_fd(), libc::F_GETFD) };
        let status_flags = unsafe { libc::fcntl(socket.as_raw_fd(), libc::F_GETFL) };
        anyhow::ensure!(
            fd_flags >= 0
                && fd_flags & libc::FD_CLOEXEC != 0
                && status_flags >= 0
                && status_flags & libc::O_NONBLOCK != 0
                && unsafe { libc::fstat(socket.as_raw_fd(), &mut stat) } == 0
                && stat.st_mode & libc::S_IFMT == libc::S_IFSOCK
                && stat.st_ino == inode
                && namespace.metadata()?.ino() == namespace_inode
                && std::fs::metadata("/proc/self/ns/net")?.ino() == namespace_inode
                && unsafe {
                    libc::getsockname(
                        socket.as_raw_fd(),
                        (&mut address as *mut libc::sockaddr_in).cast(),
                        &mut length,
                    )
                } == 0
                && length as usize == std::mem::size_of_val(&address)
                && address.sin_family == libc::AF_INET as libc::sa_family_t
                && address.sin_addr.s_addr == u32::from_ne_bytes([127, 0, 0, 1])
                && u16::from_be(address.sin_port) == port,
            "E3 retained listener changed"
        );
        if let Some(launch) = &self.launch {
            if let Some(registration) = &launch.gateway_registration {
                if launch.security.is_some() && launch.gateway_wait_status.is_none() {
                    anyhow::ensure!(
                        ManagedGatewayLaunchCapabilityV1::e3_process_start(registration.pid)?
                            == registration.pid_start_time_ticks,
                        "E3 listener process identity changed"
                    );
                    let mut process_sockets = BTreeSet::new();
                    for entry in fs::read_dir(format!("/proc/{}/fd", registration.pid))? {
                        let target = fs::read_link(entry?.path())?;
                        if let Some(value) = target
                            .to_str()
                            .and_then(|v| v.strip_prefix("socket:["))
                            .and_then(|v| v.strip_suffix(']'))
                        {
                            process_sockets.insert(value.parse::<u64>()?);
                        }
                    }
                    anyhow::ensure!(
                        process_sockets.contains(&inode),
                        "E3 gateway lost inherited listener"
                    );
                    let mut listening = Vec::new();
                    for table in ["tcp", "tcp6"] {
                        let bytes =
                            fs::read_to_string(format!("/proc/{}/net/{table}", registration.pid))?;
                        for line in bytes.lines().skip(1) {
                            let fields = line.split_whitespace().collect::<Vec<_>>();
                            anyhow::ensure!(fields.len() >= 10, "malformed E3 listener inventory");
                            let candidate = fields[9].parse::<u64>()?;
                            if fields[3] == "0A" && process_sockets.contains(&candidate) {
                                listening.push(candidate);
                            }
                        }
                    }
                    anyhow::ensure!(
                        listening == [inode],
                        "E3 gateway has an unexpected listening socket"
                    );
                    anyhow::ensure!(
                        fs::metadata(format!("/proc/{}/ns/net", registration.pid))?.ino()
                            == namespace_inode,
                        "E3 gateway network namespace changed"
                    );
                }
            }
        }
        Ok(())
    }
}

/// A separate, nonsecret post-exec attestation channel; never the auth-bundle pipe.
#[allow(dead_code)] // Consumed by the E3 descriptor launch primitive below.
pub(crate) fn create_e3_gateway_secret_ready_pipe() -> Result<(OwnedFd, std::fs::File)> {
    let mut descriptors = [-1; 2];
    // Both ends remain close-on-exec until the exact launch descriptor inventory installs them.
    if unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(std::io::Error::last_os_error()).context("create E3 secret-ready pipe");
    }
    Ok(unsafe {
        (
            OwnedFd::from_raw_fd(descriptors[0]),
            std::fs::File::from_raw_fd(descriptors[1]),
        )
    })
}

pub(crate) const GATEWAY_REQUEST_ENABLED_ENV: &str = "SUBSTRATE_LLM_GATEWAY_ENABLED";
pub(crate) const GATEWAY_REQUEST_MODE_ENV: &str = "SUBSTRATE_LLM_GATEWAY_MODE";
pub(crate) const GATEWAY_REQUEST_DEFAULT_BACKEND_ENV: &str = "SUBSTRATE_LLM_DEFAULT_BACKEND";

const GATEWAY_LAUNCH_MODE_ENV: &str = "SUBSTRATE_LLM_GATEWAY_MODE";
const GATEWAY_LAUNCH_CONFIG_PATH_ENV: &str = "SUBSTRATE_LLM_GATEWAY_CONFIG_PATH";
const GATEWAY_LAUNCH_DISABLE_TOKEN_PERSISTENCE_ENV: &str =
    "SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE";
const GATEWAY_MODE_IN_WORLD: &str = "in_world";
const GATEWAY_MODE_HOST_ONLY: &str = "host_only";

const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
const GATEWAY_BINARY_OVERRIDE_ENV: &str = "SUBSTRATE_GATEWAY_BINARY";
const HEALTH_PATH: &str = "/health";
const DEFAULT_BACKEND: &str = "cli:codex";
const CLI_CODEX_HOST_BACKEND: &str = "cli:codex-host";
const CLI_CODEX_WORLD_BACKEND: &str = "cli:codex-world";
const CLI_CLAUDE_CODE_BACKEND: &str = "cli:claude_code";
const CLI_CLAUDE_CODE_HOST_BACKEND: &str = "cli:claude_code-host";
const CLI_CLAUDE_CODE_WORLD_BACKEND: &str = "cli:claude_code-world";
const API_OPENAI_BACKEND: &str = "api:openai";
const DEFAULT_ROUTED_MODEL: &str = "codex";
const DEFAULT_ACTUAL_MODEL: &str = "codex-mini-latest";
const DEFAULT_PROVIDER_NAME: &str = "openai-codex";
const CLAUDE_ROUTED_MODEL: &str = "claude-sonnet-4.5";
const CLAUDE_ACTUAL_MODEL: &str = "claude-sonnet-4-5-20250929";
const CLAUDE_PROVIDER_NAME: &str = "anthropic-api";
const OPENAI_ROUTED_MODEL: &str = "gpt-4.1-mini";
const OPENAI_ACTUAL_MODEL: &str = "gpt-4.1-mini";
const OPENAI_PROVIDER_NAME: &str = "openai-api";
const GATEWAY_RUNTIME_ROOT_DIR: &str = "substrate-gateway-runtime";
const GATEWAY_RUNTIME_MANIFEST_NAME: &str = "runtime.json";
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(8);
const GATEWAY_RUNTIME_DIR_MODE: u32 = 0o750;
const GATEWAY_RUNTIME_FILE_MODE: u32 = 0o640;
const WORLD_ENTRY_WRAPPER_MODE: u32 = 0o755;
const KNOWN_GATEWAY_AUTH_ENV_VARS: &[&str] = &[
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
    SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
    SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
    ANTHROPIC_API_KEY_ENV,
    OPENAI_API_KEY_ENV,
];
const REQUIRED_GATEWAY_CAPABILITIES: &[&str] =
    &["agent_api.run", "agent_api.events", "agent_api.events.live"];
const WORLD_ENTRY_BINARY_ENV: &str = "SUBSTRATE_WORLD_ENTRY_BINARY";
const WORLD_ENTRY_WORKING_DIR_ENV: &str = "SUBSTRATE_WORLD_ENTRY_WORKING_DIR";
const WORLD_ENTRY_CGROUP_PROCS_ENV: &str = "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH";
const WORLD_ENTRY_REQUIRE_CGROUP_ATTACH_ENV: &str = "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GatewayRuntimeState {
    AbsentComponent,
    ProvisionedStopped,
    Starting,
    Ready,
    RestartInProgress,
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayControlSettings {
    pub default_backend: String,
}

impl GatewayControlSettings {
    pub(crate) fn from_request_env(
        env: Option<&HashMap<String, String>>,
    ) -> Result<Self, GatewayRuntimeFailure> {
        let enabled = parse_bool_env(env, GATEWAY_REQUEST_ENABLED_ENV)?.unwrap_or(true);
        if !enabled {
            return Err(GatewayRuntimeFailure::policy(
                "gateway lifecycle is disabled by effective config",
            ));
        }

        let mode = env
            .and_then(|values| values.get(GATEWAY_REQUEST_MODE_ENV))
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| GATEWAY_MODE_IN_WORLD.to_string());

        match mode.as_str() {
            GATEWAY_MODE_IN_WORLD => {}
            GATEWAY_MODE_HOST_ONLY => {
                return Err(GatewayRuntimeFailure::policy(
                    "gateway lifecycle is unavailable while llm.gateway.mode=host_only",
                ));
            }
            other => {
                return Err(GatewayRuntimeFailure::invalid_integration(format!(
                    "unsupported gateway mode '{}'",
                    other
                )));
            }
        }

        let default_backend = env
            .and_then(|values| values.get(GATEWAY_REQUEST_DEFAULT_BACKEND_ENV))
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| DEFAULT_BACKEND.to_string());

        validate_gateway_backend_id_selector(&default_backend).map_err(|err| {
            if default_backend.is_empty() {
                GatewayRuntimeFailure::invalid_integration(format!(
                    "{} must be a non-empty backend id",
                    GATEWAY_REQUEST_DEFAULT_BACKEND_ENV
                ))
            } else {
                GatewayRuntimeFailure::invalid_integration(err)
            }
        })?;

        Ok(Self { default_backend })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewayIntegratedAuthKind {
    CliCodex,
    ApiEnv,
}

#[derive(Debug, Clone, Copy)]
enum GatewayProviderAuthConfig {
    OAuth {
        oauth_provider: &'static str,
    },
    ApiKey {
        env_var: &'static str,
        bundle_field: &'static str,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GatewayBackendBinding {
    pub(crate) backend_id: &'static str,
    auth_bundle_backend_id: &'static str,
    pub(crate) routed_model: &'static str,
    pub(crate) actual_model: &'static str,
    pub(crate) provider_name: &'static str,
    pub(crate) provider_type: &'static str,
    pub(crate) advertised_capabilities: &'static [&'static str],
    pub(crate) required_capabilities: &'static [&'static str],
    provider_auth: GatewayProviderAuthConfig,
    auth_kind: GatewayIntegratedAuthKind,
}

const CLI_CODEX_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: DEFAULT_BACKEND,
    auth_bundle_backend_id: DEFAULT_BACKEND,
    routed_model: DEFAULT_ROUTED_MODEL,
    actual_model: DEFAULT_ACTUAL_MODEL,
    provider_name: DEFAULT_PROVIDER_NAME,
    provider_type: "openai",
    advertised_capabilities: &[
        "agent_api.run",
        "agent_api.events",
        "agent_api.events.live",
        "agent_api.exec.non_interactive",
        "agent_api.exec.add_dirs.v1",
        "agent_api.session.resume.v1",
        "agent_api.session.fork.v1",
        "agent_api.session.handle.v1",
        "agent_api.control.cancel.v1",
        "agent_api.tools.structured.v1",
        "agent_api.tools.results.v1",
        "agent_api.artifacts.final_text.v1",
    ],
    required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
    provider_auth: GatewayProviderAuthConfig::OAuth {
        oauth_provider: DEFAULT_PROVIDER_NAME,
    },
    auth_kind: GatewayIntegratedAuthKind::CliCodex,
};

const CLI_CODEX_HOST_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: CLI_CODEX_HOST_BACKEND,
    ..CLI_CODEX_BACKEND_BINDING
};

const CLI_CODEX_WORLD_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: CLI_CODEX_WORLD_BACKEND,
    ..CLI_CODEX_BACKEND_BINDING
};

const API_OPENAI_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: API_OPENAI_BACKEND,
    auth_bundle_backend_id: API_OPENAI_BACKEND,
    routed_model: OPENAI_ROUTED_MODEL,
    actual_model: OPENAI_ACTUAL_MODEL,
    provider_name: OPENAI_PROVIDER_NAME,
    provider_type: "openai",
    advertised_capabilities: CLI_CODEX_BACKEND_BINDING.advertised_capabilities,
    required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
    provider_auth: GatewayProviderAuthConfig::ApiKey {
        env_var: OPENAI_API_KEY_ENV,
        bundle_field: SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
    },
    auth_kind: GatewayIntegratedAuthKind::ApiEnv,
};

const CLI_CLAUDE_CODE_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: CLI_CLAUDE_CODE_BACKEND,
    auth_bundle_backend_id: CLI_CLAUDE_CODE_BACKEND,
    routed_model: CLAUDE_ROUTED_MODEL,
    actual_model: CLAUDE_ACTUAL_MODEL,
    provider_name: CLAUDE_PROVIDER_NAME,
    provider_type: "anthropic",
    advertised_capabilities: CLI_CODEX_BACKEND_BINDING.advertised_capabilities,
    required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
    provider_auth: GatewayProviderAuthConfig::ApiKey {
        env_var: ANTHROPIC_API_KEY_ENV,
        bundle_field: SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
    },
    auth_kind: GatewayIntegratedAuthKind::ApiEnv,
};

const CLI_CLAUDE_CODE_HOST_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: CLI_CLAUDE_CODE_HOST_BACKEND,
    ..CLI_CLAUDE_CODE_BACKEND_BINDING
};

const CLI_CLAUDE_CODE_WORLD_BACKEND_BINDING: GatewayBackendBinding = GatewayBackendBinding {
    backend_id: CLI_CLAUDE_CODE_WORLD_BACKEND,
    ..CLI_CLAUDE_CODE_BACKEND_BINDING
};

const GATEWAY_BACKEND_BINDINGS: &[GatewayBackendBinding] = &[
    CLI_CODEX_BACKEND_BINDING,
    CLI_CODEX_HOST_BACKEND_BINDING,
    CLI_CODEX_WORLD_BACKEND_BINDING,
    CLI_CLAUDE_CODE_BACKEND_BINDING,
    CLI_CLAUDE_CODE_HOST_BACKEND_BINDING,
    CLI_CLAUDE_CODE_WORLD_BACKEND_BINDING,
    API_OPENAI_BACKEND_BINDING,
];

pub(crate) fn resolve_gateway_backend_binding(
    backend_id: &str,
) -> Option<&'static GatewayBackendBinding> {
    GATEWAY_BACKEND_BINDINGS
        .iter()
        .find(|binding| binding.backend_id == backend_id)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GatewayRuntimeFailure {
    #[error("gateway_invalid_integration: {0}")]
    InvalidIntegration(String),
    #[error("gateway_transient_failure: {0}")]
    Transient(String),
    #[error("gateway_policy_blocked: {0}")]
    PolicyBlocked(String),
}

impl GatewayRuntimeFailure {
    pub(crate) fn invalid_integration(message: impl Into<String>) -> Self {
        Self::InvalidIntegration(message.into())
    }

    pub(crate) fn transient(message: impl Into<String>) -> Self {
        Self::Transient(message.into())
    }

    pub(crate) fn policy(message: impl Into<String>) -> Self {
        Self::PolicyBlocked(message.into())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayRuntimeStartContext {
    pub world_id: String,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub require_cgroup_attach: bool,
    pub binding: &'static GatewayBackendBinding,
    pub integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
}

#[derive(Debug, Clone)]
pub(crate) struct LinuxWorldPlacementContext {
    pub working_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub require_cgroup_attach: bool,
}

impl LinuxWorldPlacementContext {
    pub(crate) fn cgroup_procs_path(&self) -> PathBuf {
        self.cgroup_path.join("cgroup.procs")
    }
}

impl From<&GatewayRuntimeStartContext> for LinuxWorldPlacementContext {
    fn from(value: &GatewayRuntimeStartContext) -> Self {
        Self {
            working_dir: value.project_dir.clone(),
            cgroup_path: value.cgroup_path.clone(),
            require_cgroup_attach: value.require_cgroup_attach,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedLinuxWorldEntryLauncher {
    pub launcher_path: PathBuf,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRuntimeManifest {
    world_id: String,
    backend_id: String,
    pid: u32,
    pid_start_time_ticks: u64,
    port: u16,
    runtime_dir: PathBuf,
    config_path: PathBuf,
    state: GatewayRuntimeState,
}

#[derive(Debug)]
enum ManagedGatewayProcess {
    Child(Child),
    RediscoveredPid(u32),
}

#[derive(Debug, Clone)]
struct ManagedGatewayRuntime {
    world_id: String,
    backend_id: String,
    port: u16,
    runtime_dir: PathBuf,
    config_path: PathBuf,
    manifest_path: PathBuf,
    pid_start_time_ticks: u64,
    process: Arc<Mutex<ManagedGatewayProcess>>,
    state: Arc<RwLock<GatewayRuntimeState>>,
    #[allow(dead_code)]
    privileged_child_exclusion_lease: Arc<HeldNonE3PrivilegedChildLeaseV1>,
}

impl ManagedGatewayRuntime {
    fn set_state(&self, state: GatewayRuntimeState) {
        if let Ok(mut guard) = self.state.write() {
            *guard = state;
        }
        let _ = self.persist_manifest();
    }

    fn state(&self) -> GatewayRuntimeState {
        self.state
            .read()
            .map(|guard| *guard)
            .unwrap_or(GatewayRuntimeState::ProvisionedStopped)
    }

    fn pid(&self) -> Result<u32> {
        let guard = self
            .process
            .lock()
            .map_err(|_| anyhow!("gateway child lock poisoned"))?;
        Ok(match &*guard {
            ManagedGatewayProcess::Child(child) => child.id(),
            ManagedGatewayProcess::RediscoveredPid(pid) => *pid,
        })
    }

    fn persist_manifest(&self) -> Result<()> {
        let manifest = GatewayRuntimeManifest {
            world_id: self.world_id.clone(),
            backend_id: self.backend_id.clone(),
            pid: self.pid()?,
            pid_start_time_ticks: self.pid_start_time_ticks,
            port: self.port,
            runtime_dir: self.runtime_dir.clone(),
            config_path: self.config_path.clone(),
            state: self.state(),
        };
        write_runtime_manifest(&self.manifest_path, &manifest)
    }
}

#[derive(Default)]
struct GatewayLifecycleWorldState {
    op_lock: AsyncMutex<()>,
    state: RwLock<Option<GatewayRuntimeState>>,
}

impl GatewayLifecycleWorldState {
    fn state(&self) -> Option<GatewayRuntimeState> {
        self.state.read().ok().and_then(|guard| *guard)
    }

    fn replace_state(&self, state: Option<GatewayRuntimeState>) -> Option<GatewayRuntimeState> {
        match self.state.write() {
            Ok(mut guard) => std::mem::replace(&mut *guard, state),
            Err(_) => None,
        }
    }

    fn scoped_state(self: &Arc<Self>, state: GatewayRuntimeState) -> GatewayLifecycleStateGuard {
        let previous = self.replace_state(Some(state));
        GatewayLifecycleStateGuard {
            state: Arc::clone(self),
            previous,
        }
    }
}

struct GatewayLifecycleStateGuard {
    state: Arc<GatewayLifecycleWorldState>,
    previous: Option<GatewayRuntimeState>,
}

impl Drop for GatewayLifecycleStateGuard {
    fn drop(&mut self) {
        let _ = self.state.replace_state(self.previous);
    }
}

pub(crate) struct GatewayRuntimeManager {
    runtimes: Mutex<HashMap<String, ManagedGatewayRuntime>>,
    lifecycle: Mutex<HashMap<String, Arc<GatewayLifecycleWorldState>>>,
    privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
}

impl GatewayRuntimeManager {
    pub(crate) fn new(privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>) -> Self {
        Self {
            runtimes: Mutex::default(),
            lifecycle: Mutex::default(),
            privileged_child_exclusion,
        }
    }

    #[cfg(not(test))]
    pub(crate) fn recover_existing_non_e3_gateways_for_exclusion(&self) -> Result<()> {
        let root = gateway_runtime_root_dir();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("scan gateway runtime root {}", root.display()))
            }
        };
        for backend_entry in entries {
            let backend_entry = backend_entry.context("read gateway backend runtime entry")?;
            if !backend_entry
                .file_type()
                .context("inspect gateway backend runtime entry")?
                .is_dir()
            {
                bail_unclassified_gateway_runtime(&backend_entry.path())?;
            }
            for world_entry in
                fs::read_dir(backend_entry.path()).context("scan gateway world runtime entries")?
            {
                let world_entry = world_entry.context("read gateway world runtime entry")?;
                if !world_entry
                    .file_type()
                    .context("inspect gateway world runtime entry")?
                    .is_dir()
                {
                    bail_unclassified_gateway_runtime(&world_entry.path())?;
                }
                let manifest_path = world_entry.path().join(GATEWAY_RUNTIME_MANIFEST_NAME);
                if !manifest_path.is_file() {
                    continue;
                }
                let manifest = read_runtime_manifest(&manifest_path).with_context(|| {
                    format!(
                        "classify recovered gateway manifest {}",
                        manifest_path.display()
                    )
                })?;
                if manifest_path
                    != manifest_path_for_world(&manifest.world_id, &manifest.backend_id)
                {
                    anyhow::bail!(
                        "unclassified recovered gateway manifest path {}",
                        manifest_path.display()
                    );
                }
                if !pid_is_running(manifest.pid) {
                    delete_runtime_manifest(&manifest_path);
                    continue;
                }
                if read_pid_start_time_ticks(manifest.pid).ok()
                    != Some(manifest.pid_start_time_ticks)
                {
                    anyhow::bail!(
                        "live recovered gateway PID/start identity is unclassified: {}",
                        manifest.pid
                    );
                }
                let cgroup = canonical_cgroup_identity_for_pid(manifest.pid)?;
                let lease = Arc::new(
                    self.privileged_child_exclusion
                        .register_recovered_non_e3_child(
                            manifest.pid,
                            manifest.pid_start_time_ticks,
                            cgroup,
                        )?,
                );
                if self
                    .recover_runtime(&manifest.world_id, &manifest.backend_id, lease)
                    .is_none()
                {
                    anyhow::bail!(
                        "live recovered gateway failed exact pre-adoption validation: {}",
                        manifest.pid
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn status(
        &self,
        world_id: &str,
        backend_id: &str,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(state) = self.lifecycle_state_for_world(world_id) {
            return Err(lifecycle_status_transient_failure(world_id, state));
        }

        let Some(runtime) = self.runtime_for_world_or_manifest(world_id, backend_id)? else {
            return Ok(unavailable_response());
        };

        let state = self.observe_runtime_state(&runtime).await?;
        Ok(match state {
            GatewayRuntimeState::Ready => available_response(runtime.port),
            GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                return Err(lifecycle_status_transient_failure(world_id, state));
            }
            GatewayRuntimeState::ProvisionedStopped | GatewayRuntimeState::AbsentComponent => {
                self.remove_runtime(world_id, backend_id);
                unavailable_response()
            }
        })
    }

    pub(crate) async fn sync(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        self.sync_with_timeout(ctx, DEFAULT_READY_TIMEOUT).await
    }

    async fn sync_with_timeout(
        &self,
        ctx: GatewayRuntimeStartContext,
        ready_timeout: Duration,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let lifecycle = self.lifecycle_for_world(&ctx.world_id);
        let _world_guard = lifecycle.op_lock.lock().await;
        self.sync_with_timeout_locked(ctx, ready_timeout, Some(&lifecycle))
            .await
    }

    async fn sync_with_timeout_locked(
        &self,
        ctx: GatewayRuntimeStartContext,
        ready_timeout: Duration,
        lifecycle: Option<&Arc<GatewayLifecycleWorldState>>,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        if let Some(runtime) =
            self.runtime_for_world_or_manifest(&ctx.world_id, ctx.binding.backend_id)?
        {
            match self.observe_runtime_state(&runtime).await? {
                GatewayRuntimeState::Ready => return Ok(available_response(runtime.port)),
                GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                    return self.wait_until_ready(runtime, ready_timeout).await;
                }
                GatewayRuntimeState::ProvisionedStopped | GatewayRuntimeState::AbsentComponent => {
                    self.remove_runtime(&ctx.world_id, ctx.binding.backend_id);
                }
            }
        }

        let _state_guard = lifecycle.map(|state| state.scoped_state(GatewayRuntimeState::Starting));
        let Some(binary_path) = resolve_gateway_binary()? else {
            return Ok(unavailable_response());
        };

        let backend_id = ctx.binding.backend_id;
        let privileged_child_exclusion_lease = Arc::new(
            self.privileged_child_exclusion
                .acquire_non_e3_child()
                .map_err(|error| GatewayRuntimeFailure::policy(error.to_string()))?,
        );
        let runtime = start_runtime(binary_path, ctx, privileged_child_exclusion_lease)?;
        let world_id = runtime.world_id.clone();
        self.insert_runtime(runtime.clone());
        match self.wait_until_ready(runtime, ready_timeout).await {
            Ok(response) => Ok(response),
            Err(err) => {
                if let Some(runtime) = self.take_runtime(&world_id) {
                    let _ = stop_runtime(runtime);
                }
                delete_runtime_manifest(&manifest_path_for_world(&world_id, backend_id));
                Err(err)
            }
        }
    }

    pub(crate) async fn restart(
        &self,
        ctx: GatewayRuntimeStartContext,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let lifecycle = self.lifecycle_for_world(&ctx.world_id);
        let _world_guard = lifecycle.op_lock.lock().await;
        let _restart_state = lifecycle.scoped_state(GatewayRuntimeState::RestartInProgress);

        if let Some(existing) =
            self.runtime_for_world_or_manifest(&ctx.world_id, ctx.binding.backend_id)?
        {
            existing.set_state(GatewayRuntimeState::RestartInProgress);
            self.take_runtime(&ctx.world_id);
            delete_runtime_manifest(&manifest_path_for_world(
                &ctx.world_id,
                ctx.binding.backend_id,
            ));
            stop_runtime(existing)
                .with_context(|| format!("failed to stop gateway runtime for {}", ctx.world_id))
                .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
        }

        self.sync_with_timeout_locked(ctx, DEFAULT_READY_TIMEOUT, None)
            .await
    }

    pub(crate) fn pid_for_world(&self, world_id: &str) -> Option<u32> {
        let runtime = self.runtime_for_world(world_id)?;
        runtime.pid().ok()
    }

    pub(crate) fn forget_runtime_for_test(&self, world_id: &str) {
        let _ = self.take_runtime(world_id);
    }

    fn runtime_for_world(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|guard| guard.get(world_id).cloned())
    }

    fn lifecycle_for_world(&self, world_id: &str) -> Arc<GatewayLifecycleWorldState> {
        match self.lifecycle.lock() {
            Ok(mut guard) => Arc::clone(
                guard
                    .entry(world_id.to_string())
                    .or_insert_with(|| Arc::new(GatewayLifecycleWorldState::default())),
            ),
            Err(_) => Arc::new(GatewayLifecycleWorldState::default()),
        }
    }

    fn lifecycle_state_for_world(&self, world_id: &str) -> Option<GatewayRuntimeState> {
        self.lifecycle
            .lock()
            .ok()
            .and_then(|guard| guard.get(world_id).cloned())
            .and_then(|state| state.state())
    }

    fn runtime_for_world_or_manifest(
        &self,
        world_id: &str,
        backend_id: &str,
    ) -> Result<Option<ManagedGatewayRuntime>, GatewayRuntimeFailure> {
        if let Some(runtime) = self.runtime_for_world(world_id) {
            return Ok(Some(runtime));
        }
        let lease = Arc::new(
            self.privileged_child_exclusion
                .acquire_non_e3_child()
                .map_err(|error| GatewayRuntimeFailure::policy(error.to_string()))?,
        );
        Ok(self.recover_runtime(world_id, backend_id, lease))
    }

    fn insert_runtime(&self, runtime: ManagedGatewayRuntime) {
        if let Ok(mut guard) = self.runtimes.lock() {
            guard.insert(runtime.world_id.clone(), runtime);
        }
    }

    fn remove_runtime(&self, world_id: &str, backend_id: &str) -> Option<ManagedGatewayRuntime> {
        let runtime = self.take_runtime(world_id);
        delete_runtime_manifest(&manifest_path_for_world(world_id, backend_id));
        runtime
    }

    fn take_runtime(&self, world_id: &str) -> Option<ManagedGatewayRuntime> {
        self.runtimes
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(world_id))
    }

    fn recover_runtime(
        &self,
        world_id: &str,
        backend_id: &str,
        privileged_child_exclusion_lease: Arc<HeldNonE3PrivilegedChildLeaseV1>,
    ) -> Option<ManagedGatewayRuntime> {
        let manifest_path = manifest_path_for_world(world_id, backend_id);
        let manifest = match read_runtime_manifest(&manifest_path) {
            Ok(manifest) => manifest,
            Err(_) => {
                delete_runtime_manifest(&manifest_path);
                return None;
            }
        };
        let expected_runtime_dir = runtime_dir_for_world(world_id, backend_id);
        let artifacts_exist = manifest.runtime_dir.is_dir() && manifest.config_path.is_file();
        let start_time_matches = read_pid_start_time_ticks(manifest.pid)
            .map(|start_time| start_time == manifest.pid_start_time_ticks)
            .unwrap_or(false);
        if manifest.world_id != world_id
            || manifest.backend_id != backend_id
            || manifest.runtime_dir != expected_runtime_dir
            || !artifacts_exist
            || !pid_is_running(manifest.pid)
            || !start_time_matches
            || !gateway_health_ready_blocking(manifest.port)
        {
            delete_runtime_manifest(&manifest_path);
            return None;
        }

        let runtime = ManagedGatewayRuntime {
            world_id: manifest.world_id,
            backend_id: manifest.backend_id,
            port: manifest.port,
            runtime_dir: manifest.runtime_dir,
            config_path: manifest.config_path,
            manifest_path,
            pid_start_time_ticks: manifest.pid_start_time_ticks,
            process: Arc::new(Mutex::new(ManagedGatewayProcess::RediscoveredPid(
                manifest.pid,
            ))),
            state: Arc::new(RwLock::new(manifest.state)),
            privileged_child_exclusion_lease,
        };
        self.insert_runtime(runtime.clone());
        Some(runtime)
    }

    async fn observe_runtime_state(
        &self,
        runtime: &ManagedGatewayRuntime,
    ) -> Result<GatewayRuntimeState, GatewayRuntimeFailure> {
        let is_running = {
            let mut process = runtime
                .process
                .lock()
                .map_err(|_| GatewayRuntimeFailure::transient("gateway child lock poisoned"))?;
            process_is_running(&mut process, &runtime.runtime_dir)
        }
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

        if !is_running {
            runtime.set_state(GatewayRuntimeState::ProvisionedStopped);
            return Ok(GatewayRuntimeState::ProvisionedStopped);
        }

        if gateway_health_ready(runtime.port).await {
            runtime.set_state(GatewayRuntimeState::Ready);
            return Ok(GatewayRuntimeState::Ready);
        }

        Ok(runtime.state())
    }

    async fn wait_until_ready(
        &self,
        runtime: ManagedGatewayRuntime,
        timeout: Duration,
    ) -> Result<GatewayLifecycleResponseV1, GatewayRuntimeFailure> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.observe_runtime_state(&runtime).await? {
                GatewayRuntimeState::Ready => return Ok(available_response(runtime.port)),
                GatewayRuntimeState::ProvisionedStopped => {
                    return Err(GatewayRuntimeFailure::transient(format!(
                        "gateway process exited before it became ready; inspect {} and {}",
                        runtime.runtime_dir.join("stdout.log").display(),
                        runtime.runtime_dir.join("stderr.log").display()
                    )));
                }
                GatewayRuntimeState::Starting | GatewayRuntimeState::RestartInProgress => {
                    if Instant::now() >= deadline {
                        return Err(GatewayRuntimeFailure::transient(format!(
                            "gateway did not become ready before timeout; inspect {} and {}",
                            runtime.runtime_dir.join("stdout.log").display(),
                            runtime.runtime_dir.join("stderr.log").display()
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(125)).await;
                }
                GatewayRuntimeState::AbsentComponent => return Ok(unavailable_response()),
            }
        }
    }
}

#[cfg(not(test))]
fn bail_unclassified_gateway_runtime(path: &Path) -> Result<()> {
    anyhow::bail!(
        "unclassified entry in recovered gateway runtime tree: {}",
        path.display()
    )
}

#[cfg(not(test))]
fn canonical_cgroup_identity_for_pid(pid: u32) -> Result<CanonicalCgroupIdentityV1> {
    use std::os::unix::fs::MetadataExt;

    let membership = fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .context("read recovered gateway cgroup membership")?;
    let relative = membership
        .strip_prefix("0::/")
        .and_then(|value| value.strip_suffix('\n'))
        .context("recovered gateway has noncanonical cgroup v2 membership")?;
    if relative.is_empty()
        || relative
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        anyhow::bail!("recovered gateway cgroup path is not canonical");
    }
    let mount = fs::metadata("/sys/fs/cgroup").context("stat cgroup v2 mount")?;
    let directory = fs::metadata(Path::new("/sys/fs/cgroup").join(relative))
        .context("stat recovered gateway cgroup")?;
    Ok(CanonicalCgroupIdentityV1 {
        cgroup_v2_mount_device_id: mount.dev(),
        cgroup_v2_mount_inode: mount.ino(),
        cgroup_directory_inode: directory.ino(),
        cgroup_relative_path: relative.to_string(),
    })
}

fn start_runtime(
    binary_path: PathBuf,
    ctx: GatewayRuntimeStartContext,
    privileged_child_exclusion_lease: Arc<HeldNonE3PrivilegedChildLeaseV1>,
) -> Result<ManagedGatewayRuntime, GatewayRuntimeFailure> {
    validate_binding_capabilities(ctx.binding)?;
    let placement = LinuxWorldPlacementContext::from(&ctx);
    validate_linux_world_placement_context(&placement)?;
    let port = pick_free_port().map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let runtime_dir = runtime_dir_for_world(&ctx.world_id, ctx.binding.backend_id);
    let home_dir = runtime_dir.join("home");
    ensure_directory_with_mode(&runtime_dir, GATEWAY_RUNTIME_DIR_MODE)
        .with_context(|| {
            format!(
                "failed to create gateway runtime directory {}",
                runtime_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    ensure_directory_with_mode(&home_dir, GATEWAY_RUNTIME_DIR_MODE)
        .with_context(|| {
            format!(
                "failed to create gateway runtime directory {}",
                home_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let config_path = runtime_dir.join("config.toml");
    let config = render_integrated_config(port, ctx.binding);
    write_file_with_mode(&config_path, config.as_bytes(), GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to write {}", config_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let stdout_log = runtime_dir.join("stdout.log");
    let stderr_log = runtime_dir.join("stderr.log");
    let stdout = create_file_with_mode(&stdout_log, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to create {}", stdout_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let stderr = create_file_with_mode(&stderr_log, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to create {}", stderr_log.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let auth_bundle_handoff =
        prepare_gateway_auth_bundle_handoff(ctx.binding, ctx.integrated_auth)?;

    let mut command = Command::new(&binary_path);
    command
        .current_dir(&ctx.project_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .env("HOME", &home_dir)
        .env(GATEWAY_LAUNCH_MODE_ENV, GATEWAY_MODE_IN_WORLD)
        .env(GATEWAY_LAUNCH_CONFIG_PATH_ENV, &config_path)
        .env(GATEWAY_LAUNCH_DISABLE_TOKEN_PERSISTENCE_ENV, "1")
        .env_remove(SUBSTRATE_LLM_AUTH_BUNDLE_FD)
        .env(
            SUBSTRATE_LLM_AUTH_BUNDLE_FD,
            &auth_bundle_handoff.fd_env_value,
        );

    for env_key in KNOWN_GATEWAY_AUTH_ENV_VARS {
        command.env_remove(env_key);
    }
    append_gateway_start_args(&mut command, &config_path);

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", binary_path.display()))
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    drop(auth_bundle_handoff);

    if let Err(err) = attach_pid_to_linux_world_cgroup(child.id(), &placement) {
        let _ = kill_child_process(&mut child);
        return Err(err);
    }
    let pid_start_time_ticks = read_pid_start_time_ticks(child.id()).map_err(|err| {
        let _ = kill_child_process(&mut child);
        GatewayRuntimeFailure::transient(err.to_string())
    })?;

    let world_id = ctx.world_id;
    let runtime = ManagedGatewayRuntime {
        world_id: world_id.clone(),
        backend_id: ctx.binding.backend_id.to_string(),
        port,
        runtime_dir,
        config_path,
        manifest_path: manifest_path_for_world(&world_id, ctx.binding.backend_id),
        pid_start_time_ticks,
        process: Arc::new(Mutex::new(ManagedGatewayProcess::Child(child))),
        state: Arc::new(RwLock::new(GatewayRuntimeState::Starting)),
        privileged_child_exclusion_lease,
    };
    if let Err(err) = runtime.persist_manifest() {
        if let Ok(mut process) = runtime.process.lock() {
            let _ = stop_process(&mut process);
        }
        delete_runtime_manifest(&runtime.manifest_path);
        return Err(GatewayRuntimeFailure::transient(err.to_string()));
    }

    Ok(runtime)
}

fn stop_runtime(runtime: ManagedGatewayRuntime) -> Result<()> {
    let mut process = runtime
        .process
        .lock()
        .map_err(|_| anyhow!("gateway child lock poisoned"))?;
    stop_process(&mut process)
}

pub(crate) fn validate_linux_world_placement_context(
    placement: &LinuxWorldPlacementContext,
) -> Result<(), GatewayRuntimeFailure> {
    if !placement.working_dir.is_dir() {
        return Err(GatewayRuntimeFailure::transient(format!(
            "world entry working_dir is missing or not a directory: {}",
            placement.working_dir.display()
        )));
    }

    let cgroup_procs = placement.cgroup_procs_path();
    if placement.require_cgroup_attach && !cgroup_procs.is_file() {
        return Err(GatewayRuntimeFailure::transient(format!(
            "world entry cgroup attach target is missing: {}",
            cgroup_procs.display()
        )));
    }

    Ok(())
}

pub(crate) fn attach_pid_to_linux_world_cgroup(
    pid: u32,
    placement: &LinuxWorldPlacementContext,
) -> Result<(), GatewayRuntimeFailure> {
    let cgroup_procs = placement.cgroup_procs_path();
    match fs::write(&cgroup_procs, pid.to_string()) {
        Ok(()) => Ok(()),
        Err(_err) if !placement.require_cgroup_attach => Ok(()),
        Err(err) => Err(GatewayRuntimeFailure::transient(format!(
            "failed to attach gateway pid {} to {}: {}",
            pid,
            cgroup_procs.display(),
            err
        ))),
    }
}

pub(crate) fn prepare_linux_world_entry_launcher(
    launcher_dir: &Path,
    actual_binary_path: &Path,
    placement: &LinuxWorldPlacementContext,
) -> Result<PreparedLinuxWorldEntryLauncher, GatewayRuntimeFailure> {
    validate_linux_world_placement_context(placement)?;

    if !actual_binary_path.is_file() {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "world entry binary does not exist or is not a file: {}",
            actual_binary_path.display()
        )));
    }

    ensure_directory_with_mode(launcher_dir, GATEWAY_RUNTIME_DIR_MODE)
        .with_context(|| {
            format!(
                "failed to create world entry launcher dir {}",
                launcher_dir.display()
            )
        })
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    let launcher_path = launcher_dir.join("world-entry.sh");
    write_file_with_mode(
        &launcher_path,
        render_linux_world_entry_wrapper().as_bytes(),
        WORLD_ENTRY_WRAPPER_MODE,
    )
    .with_context(|| format!("failed to write {}", launcher_path.display()))
    .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;

    Ok(PreparedLinuxWorldEntryLauncher {
        launcher_path,
        env: vec![
            (
                WORLD_ENTRY_BINARY_ENV.to_string(),
                actual_binary_path.display().to_string(),
            ),
            (
                WORLD_ENTRY_WORKING_DIR_ENV.to_string(),
                placement.working_dir.display().to_string(),
            ),
            (
                WORLD_ENTRY_CGROUP_PROCS_ENV.to_string(),
                placement.cgroup_procs_path().display().to_string(),
            ),
            (
                WORLD_ENTRY_REQUIRE_CGROUP_ATTACH_ENV.to_string(),
                if placement.require_cgroup_attach {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            ),
        ],
    })
}

fn render_linux_world_entry_wrapper() -> &'static str {
    r#"#!/bin/sh
set -eu

binary="${SUBSTRATE_WORLD_ENTRY_BINARY:-}"
working_dir="${SUBSTRATE_WORLD_ENTRY_WORKING_DIR:-}"
cgroup_procs_path="${SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH:-}"
require_cgroup_attach="${SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH:-1}"

if [ -z "$binary" ]; then
  echo "substrate: error: world entry missing binary path" >&2
  exit 125
fi
if [ ! -f "$binary" ]; then
  echo "substrate: error: world entry binary missing: $binary" >&2
  exit 125
fi
if [ -z "$working_dir" ]; then
  echo "substrate: error: world entry missing working_dir" >&2
  exit 125
fi
if [ ! -d "$working_dir" ]; then
  echo "substrate: error: world entry working_dir missing: $working_dir" >&2
  exit 125
fi
cd "$working_dir"

if [ "$require_cgroup_attach" = "1" ]; then
  if [ -z "$cgroup_procs_path" ]; then
    echo "substrate: error: world entry missing cgroup attach target" >&2
    exit 125
  fi
  if [ ! -e "$cgroup_procs_path" ]; then
    echo "substrate: error: world entry cgroup attach target does not exist: $cgroup_procs_path" >&2
    exit 125
  fi
  if ! printf '%s\n' "$$" > "$cgroup_procs_path"; then
    echo "substrate: error: world entry cgroup attach failed: $cgroup_procs_path" >&2
    exit 125
  fi
elif [ -n "$cgroup_procs_path" ] && [ -e "$cgroup_procs_path" ]; then
  printf '%s\n' "$$" > "$cgroup_procs_path" 2>/dev/null || true
fi

exec "$binary" "$@"
"#
}

fn append_gateway_start_args(command: &mut Command, config_path: &Path) {
    command.arg("--config").arg(config_path).arg("start");
}

fn resolve_gateway_binary() -> Result<Option<PathBuf>, GatewayRuntimeFailure> {
    if let Some(path) = std::env::var_os(GATEWAY_BINARY_OVERRIDE_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "{} points at a missing binary: {}",
            GATEWAY_BINARY_OVERRIDE_ENV,
            path.display()
        )));
    }

    for candidate in [
        PathBuf::from("/usr/local/bin/substrate-gateway"),
        PathBuf::from("/usr/bin/substrate-gateway"),
    ] {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join("substrate-gateway");
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
    }

    Ok(None)
}

fn runtime_dir_for_world(world_id: &str, backend_id: &str) -> PathBuf {
    backend_runtime_root_dir(backend_id).join(world_id)
}

fn gateway_runtime_root_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("SUBSTRATE_GATEWAY_RUNTIME_ROOT") {
        let path = PathBuf::from(path);
        let _ = ensure_directory_with_mode(&path, GATEWAY_RUNTIME_DIR_MODE);
        return path;
    }

    let run_dir = PathBuf::from("/run/substrate").join(GATEWAY_RUNTIME_ROOT_DIR);
    if ensure_directory_with_mode(&run_dir, GATEWAY_RUNTIME_DIR_MODE).is_ok() {
        return run_dir;
    }

    let temp_dir = std::env::temp_dir().join(GATEWAY_RUNTIME_ROOT_DIR);
    let _ = ensure_directory_with_mode(&temp_dir, GATEWAY_RUNTIME_DIR_MODE);
    temp_dir
}

fn backend_runtime_root_dir(backend_id: &str) -> PathBuf {
    let path = gateway_runtime_root_dir().join(runtime_backend_dir_name(backend_id));
    let _ = ensure_directory_with_mode(&path, GATEWAY_RUNTIME_DIR_MODE);
    path
}

fn runtime_backend_dir_name(backend_id: &str) -> String {
    backend_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' | ':' => ch,
            _ => '_',
        })
        .collect()
}

fn manifest_path_for_world(world_id: &str, backend_id: &str) -> PathBuf {
    runtime_dir_for_world(world_id, backend_id).join(GATEWAY_RUNTIME_MANIFEST_NAME)
}

fn write_runtime_manifest(path: &Path, manifest: &GatewayRuntimeManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory_with_mode(parent, GATEWAY_RUNTIME_DIR_MODE).with_context(|| {
            format!("failed to create gateway manifest dir {}", parent.display())
        })?;
    }
    let encoded =
        serde_json::to_vec_pretty(manifest).context("failed to encode gateway manifest")?;
    write_file_with_mode(path, &encoded, GATEWAY_RUNTIME_FILE_MODE)
        .with_context(|| format!("failed to write gateway manifest {}", path.display()))
}

fn ensure_directory_with_mode(path: &Path, mode: u32) -> Result<()> {
    fs::create_dir_all(path)?;
    set_path_mode(path, mode)
}

fn create_file_with_mode(path: &Path, mode: u32) -> Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    options.mode(mode);
    let file = options.open(path)?;
    set_path_mode(path, mode)?;
    Ok(file)
}

fn write_file_with_mode(path: &Path, content: &[u8], mode: u32) -> Result<()> {
    let mut file = create_file_with_mode(path, mode)?;
    file.write_all(content)?;
    Ok(())
}

fn set_path_mode(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn read_runtime_manifest(path: &Path) -> Result<GatewayRuntimeManifest> {
    let content = fs::read(path)
        .with_context(|| format!("failed to read gateway manifest {}", path.display()))?;
    serde_json::from_slice(&content)
        .with_context(|| format!("failed to parse gateway manifest {}", path.display()))
}

fn delete_runtime_manifest(path: &Path) {
    if let Err(err) = fs::remove_file(path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(error = %err, manifest = %path.display(), "failed to remove gateway runtime manifest");
        }
    }
}

fn render_integrated_config(port: u16, binding: &GatewayBackendBinding) -> String {
    let provider_auth = render_provider_auth_config(binding);
    format!(
        r#"[server]
host = "127.0.0.1"
port = {port}
log_level = "info"

[router]
default = "{routed_model}"

[[providers]]
name = "{provider_name}"
provider_type = "{provider_type}"
{provider_auth}
models = ["{actual_model}"]
enabled = true

[[models]]
name = "{routed_model}"

[[models.mappings]]
priority = 1
provider = "{provider_name}"
actual_model = "{actual_model}"
"#,
        routed_model = binding.routed_model,
        provider_name = binding.provider_name,
        provider_type = binding.provider_type,
        provider_auth = provider_auth,
        actual_model = binding.actual_model,
    )
}

fn render_provider_auth_config(binding: &GatewayBackendBinding) -> String {
    match binding.provider_auth {
        GatewayProviderAuthConfig::OAuth { oauth_provider } => {
            format!("auth_type = \"oauth\"\noauth_provider = \"{oauth_provider}\"")
        }
        GatewayProviderAuthConfig::ApiKey { env_var, .. } => {
            format!("auth_type = \"apikey\"\napi_key = \"${env_var}\"")
        }
    }
}

struct ResolvedGatewayAuthHandoff {
    bundle: GatewayAuthBundleV1,
}

struct GatewayAuthBundleHandoff {
    _read_fd: OwnedFd,
    fd_env_value: String,
}

fn prepare_gateway_auth_bundle_handoff(
    binding: &GatewayBackendBinding,
    auth: Option<GatewayIntegratedAuthPayloadV1>,
) -> Result<GatewayAuthBundleHandoff, GatewayRuntimeFailure> {
    let bundle = resolve_integrated_auth_handoff(binding, auth)?.bundle;
    bundle.validate().map_err(|err| {
        GatewayRuntimeFailure::transient(format!("invalid gateway auth bundle: {err}"))
    })?;
    let encoded = serde_json::to_vec(&bundle)
        .context("failed to encode gateway auth bundle")
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    let (read_fd, mut write_file) = create_inherited_auth_bundle_pipe()
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    write_file
        .write_all(&encoded)
        .context("failed to write gateway auth bundle")
        .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
    drop(write_file);

    Ok(GatewayAuthBundleHandoff {
        fd_env_value: read_fd.as_raw_fd().to_string(),
        _read_fd: read_fd,
    })
}

fn create_inherited_auth_bundle_pipe() -> Result<(OwnedFd, fs::File)> {
    let mut fds = [-1; 2];
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("failed to create auth bundle pipe");
    }

    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    clear_close_on_exec(read_fd.as_raw_fd())?;

    Ok((read_fd, fs::File::from(write_fd)))
}

fn clear_close_on_exec(fd: i32) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error())
            .context("failed to inspect auth bundle fd flags");
    }

    let rc = unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .context("failed to mark auth bundle fd inheritable");
    }

    Ok(())
}

fn resolve_integrated_auth_handoff(
    binding: &GatewayBackendBinding,
    auth: Option<GatewayIntegratedAuthPayloadV1>,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let Some(auth) = auth else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "missing request-provided integrated auth handoff for {}",
            binding.backend_id
        )));
    };

    if auth.backend_id.trim() != binding.backend_id {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided integrated auth payload for '{}' does not match selected backend '{}'",
            auth.backend_id.trim(),
            binding.backend_id
        )));
    }

    match binding.auth_kind {
        GatewayIntegratedAuthKind::CliCodex => resolve_codex_auth_handoff(auth.cli_codex.clone()),
        GatewayIntegratedAuthKind::ApiEnv => resolve_api_env_auth_handoff(binding, &auth),
    }
}

fn resolve_codex_auth_handoff(
    auth: Option<GatewayCliCodexIntegratedAuthV1>,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let Some(auth) = auth else {
        return Err(GatewayRuntimeFailure::invalid_integration(
            "missing request-provided integrated auth handoff for cli:codex",
        ));
    };

    let access_token = auth.access_token.trim().to_string();
    if access_token.is_empty() {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided {} is empty",
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN
        )));
    }

    let account_id = auth
        .account_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut fields = HashMap::from([(
        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
        access_token,
    )]);
    if let Some(account_id) = account_id {
        fields.insert(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.to_string(),
            account_id,
        );
    }

    Ok(ResolvedGatewayAuthHandoff {
        bundle: GatewayAuthBundleV1 {
            schema_version: GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
            backend_id: DEFAULT_BACKEND.to_string(),
            fields,
        },
    })
}

fn resolve_api_env_auth_handoff(
    binding: &GatewayBackendBinding,
    auth: &GatewayIntegratedAuthPayloadV1,
) -> Result<ResolvedGatewayAuthHandoff, GatewayRuntimeFailure> {
    let GatewayProviderAuthConfig::ApiKey {
        env_var,
        bundle_field,
    } = binding.provider_auth
    else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "backend '{}' is not configured for api env auth",
            binding.backend_id
        )));
    };

    let Some(api_env) = auth.api_env.as_ref() else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "missing request-provided integrated auth handoff for {} (expected api_env '{}')",
            binding.backend_id, env_var
        )));
    };

    let Some(value) = api_env.env.get(env_var) else {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided integrated auth payload for '{}' is missing api_env '{}'",
            auth.backend_id, env_var
        )));
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(GatewayRuntimeFailure::invalid_integration(format!(
            "request-provided api_env '{}' is empty",
            env_var
        )));
    }

    Ok(ResolvedGatewayAuthHandoff {
        bundle: GatewayAuthBundleV1 {
            schema_version: GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
            backend_id: binding.auth_bundle_backend_id.to_string(),
            fields: HashMap::from([(bundle_field.to_string(), value)]),
        },
    })
}

fn validate_binding_capabilities(
    binding: &GatewayBackendBinding,
) -> Result<(), GatewayRuntimeFailure> {
    for capability in binding.required_capabilities {
        if !binding
            .advertised_capabilities
            .iter()
            .any(|advertised| advertised == capability)
        {
            return Err(GatewayRuntimeFailure::transient(format!(
                "backend '{}' does not advertise required capability '{}'",
                binding.backend_id, capability
            )));
        }
    }

    Ok(())
}

fn parse_bool_env(
    env: Option<&HashMap<String, String>>,
    key: &str,
) -> Result<Option<bool>, GatewayRuntimeFailure> {
    let Some(raw) = env.and_then(|values| values.get(key)) else {
        return Ok(None);
    };

    let raw_trimmed = raw.trim().to_ascii_lowercase();
    match raw_trimmed.as_str() {
        "" => Ok(None),
        "1" | "true" | "yes" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "off" => Ok(Some(false)),
        _ => Err(GatewayRuntimeFailure::invalid_integration(format!(
            "invalid boolean value '{}' for {}",
            raw, key
        ))),
    }
}

fn pick_free_port() -> Result<u16> {
    let listener =
        StdTcpListener::bind(("127.0.0.1", 0)).context("failed to allocate gateway port")?;
    let port = listener
        .local_addr()
        .context("failed to inspect allocated gateway port")?
        .port();
    drop(listener);
    Ok(port)
}

fn process_is_running(process: &mut ManagedGatewayProcess, runtime_dir: &Path) -> Result<bool> {
    match process {
        ManagedGatewayProcess::Child(child) => child
            .try_wait()
            .with_context(|| {
                format!(
                    "failed to inspect gateway child status for {}",
                    runtime_dir.display()
                )
            })
            .map(|status| status.is_none()),
        ManagedGatewayProcess::RediscoveredPid(pid) => Ok(pid_is_running(*pid)),
    }
}

fn stop_process(process: &mut ManagedGatewayProcess) -> Result<()> {
    match process {
        ManagedGatewayProcess::Child(child) => kill_child_process(child),
        ManagedGatewayProcess::RediscoveredPid(pid) => kill_pid(*pid),
    }
}

fn kill_child_process(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_none() {
        child.kill().context("failed to kill gateway child")?;
        let _ = child.wait();
    }
    Ok(())
}

fn kill_pid(pid: u32) -> Result<()> {
    if process_has_exited(pid) {
        return Ok(());
    }

    let rc = unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::ESRCH) {
            return Err(err).context(format!("failed to kill gateway pid {pid}"));
        }
        return Ok(());
    }

    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if process_has_exited(pid) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    Err(anyhow!("gateway pid {pid} did not exit after SIGKILL"))
}

fn pid_is_running(pid: u32) -> bool {
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if rc == 0 {
        return true;
    }

    match std::io::Error::last_os_error().raw_os_error() {
        Some(code) if code == libc::EPERM => true,
        Some(code) if code == libc::ESRCH => false,
        _ => false,
    }
}

fn process_has_exited(pid: u32) -> bool {
    reap_pid_if_possible(pid) || !pid_is_running(pid)
}

fn reap_pid_if_possible(pid: u32) -> bool {
    let mut status = 0;
    let rc = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
    if rc > 0 {
        return true;
    }
    false
}

fn read_pid_start_time_ticks(pid: u32) -> Result<u64> {
    let stat_path = PathBuf::from("/proc").join(pid.to_string()).join("stat");
    let stat = fs::read_to_string(&stat_path)
        .with_context(|| format!("failed to read {}", stat_path.display()))?;
    parse_pid_start_time_ticks(&stat)
        .with_context(|| format!("failed to parse {}", stat_path.display()))
}

fn parse_pid_start_time_ticks(stat: &str) -> Result<u64> {
    let (_, rest) = stat
        .rsplit_once(") ")
        .ok_or_else(|| anyhow!("missing comm terminator in /proc stat"))?;
    let field = rest
        .split_whitespace()
        .nth(19)
        .ok_or_else(|| anyhow!("missing start time field in /proc stat"))?;
    field
        .parse::<u64>()
        .context("invalid start time field in /proc stat")
}

async fn gateway_health_ready(port: u16) -> bool {
    tokio::task::spawn_blocking(move || gateway_health_ready_blocking(port))
        .await
        .ok()
        .unwrap_or(false)
}

fn gateway_health_ready_blocking(port: u16) -> bool {
    let mut stream = match TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}")
            .parse()
            .expect("health socket address"),
        Duration::from_millis(250),
    ) {
        Ok(stream) => stream,
        Err(_) => return false,
    };

    if stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .is_err()
        || stream
            .set_write_timeout(Some(Duration::from_millis(250)))
            .is_err()
    {
        return false;
    }

    let request =
        format!("GET {HEALTH_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut status_line = String::new();
    let mut reader = BufReader::new(stream);
    if reader.read_line(&mut status_line).is_err() {
        return false;
    }
    status_line.starts_with("HTTP/1.1 200") || status_line.starts_with("HTTP/1.0 200")
}

fn available_response(port: u16) -> GatewayLifecycleResponseV1 {
    let base_url = format!("http://127.0.0.1:{port}");
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Available,
        client_wiring: Some(GatewayClientWiringV1 {
            openai_base_url: base_url.clone(),
            anthropic_base_url: base_url,
        }),
        identity_tuple: None,
        placement_posture: None,
    }
}

pub(crate) fn unavailable_response() -> GatewayLifecycleResponseV1 {
    GatewayLifecycleResponseV1 {
        status: GatewayStatusV1::Unavailable,
        client_wiring: None,
        identity_tuple: None,
        placement_posture: None,
    }
}

fn lifecycle_status_transient_failure(
    world_id: &str,
    state: GatewayRuntimeState,
) -> GatewayRuntimeFailure {
    let action = match state {
        GatewayRuntimeState::Starting => "starting",
        GatewayRuntimeState::RestartInProgress => "restarting",
        GatewayRuntimeState::Ready => "ready",
        GatewayRuntimeState::ProvisionedStopped => "stopped",
        GatewayRuntimeState::AbsentComponent => "absent",
    };
    GatewayRuntimeFailure::transient(format!(
        "gateway runtime for world {world_id} is still {action}"
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::io::{Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;
    use transport_api_types::{
        GatewayApiEnvIntegratedAuthV1, GatewayCliCodexIntegratedAuthV1,
        GatewayIntegratedAuthPayloadV1,
    };

    static ENV_LOCK: Lazy<AsyncMutex<()>> = Lazy::new(|| AsyncMutex::new(()));
    const MISSING_CAPABILITY_BINDING: GatewayBackendBinding = GatewayBackendBinding {
        backend_id: "test:missing-capability",
        auth_bundle_backend_id: DEFAULT_BACKEND,
        routed_model: "broken",
        actual_model: "broken",
        provider_name: "broken-provider",
        provider_type: "openai",
        advertised_capabilities: &["agent_api.events"],
        required_capabilities: REQUIRED_GATEWAY_CAPABILITIES,
        provider_auth: GatewayProviderAuthConfig::OAuth {
            oauth_provider: "broken-provider",
        },
        auth_kind: GatewayIntegratedAuthKind::CliCodex,
    };

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl Into<std::ffi::OsString>) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value.into());
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn finished_child_exclusion() -> Arc<E3PrivilegedChildExclusionV1> {
        let exclusion = E3PrivilegedChildExclusionV1::new_recovering()
            .expect("bind gateway fixture child exclusion");
        exclusion
            .finish_recovery()
            .expect("finish gateway fixture child-exclusion recovery");
        exclusion
    }

    #[test]
    fn e3_e_config_cleanup_rejects_substitution_and_retains_exclusion() -> Result<()> {
        use config_projection::{
            ConfigProjectionFailureV1, ConfigProjectionHsaAuthorityV1, ConfigProjectionRegistryV1,
        };
        use std::os::fd::{AsFd, BorrowedFd};
        use std::os::unix::fs::MetadataExt;
        struct Parent(PathBuf);
        impl ConfigProjectionHsaAuthorityV1 for Parent {
            fn with_locked_parent(
                &self,
                operation: &mut dyn for<'fd> FnMut(
                    BorrowedFd<'fd>,
                ) -> std::result::Result<
                    (),
                    ConfigProjectionFailureV1,
                >,
            ) -> std::result::Result<(), ConfigProjectionFailureV1> {
                let root = fs::File::open(&self.0)
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                operation(root.as_fd())
            }
        }
        let temp = tempfile::tempdir()?;
        let registry = ConfigProjectionRegistryV1::open(Arc::new(Parent(temp.path().into())))?;
        let root_path = temp.path().join("attempt");
        fs::create_dir(&root_path)?;
        fs::write(
            root_path.join("config.toml"),
            b"nonsecret test configuration\n",
        )?;
        fs::set_permissions(&root_path, fs::Permissions::from_mode(0o700))?;
        fs::set_permissions(
            root_path.join("config.toml"),
            fs::Permissions::from_mode(0o600),
        )?;
        // Exercise the Linux no-replace ABI without the privileged preparation path.
        let parent = fs::File::open(temp.path())?;
        let source = fs::File::open(&root_path)?;
        assert_eq!(
            unsafe {
                libc::syscall(
                    libc::SYS_renameat2,
                    parent.as_raw_fd(),
                    c"attempt".as_ptr(),
                    parent.as_raw_fd(),
                    c"published".as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            },
            0
        );
        assert!(!root_path.exists());
        let root_path = temp.path().join("published");
        parent.sync_all()?;
        assert_eq!(fs::metadata(&root_path)?.ino(), source.metadata()?.ino());
        assert_eq!(
            fs::read(root_path.join("config.toml"))?,
            b"nonsecret test configuration\n"
        );
        let exclusion = finished_child_exclusion();
        let lease = exclusion.acquire_e3_exclusive("config-cleanup-test", 1)?;
        let mut owner = E3GatewayRuntimeAuthorityV1::new(lease);
        let identity = config_projection::GatewayRuntimeConfigIdentityV1 {
            root: config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
                fs::File::open(&root_path)?.as_fd(),
            )?,
            relative_path: "config.toml".into(),
            mode: 0o600,
            byte_length: b"nonsecret test configuration\n".len() as u64,
            sha256: {
                use sha2::{Digest, Sha256};
                format!("{:x}", Sha256::digest(b"nonsecret test configuration\n"))
            },
        };
        owner.gateway_config = Some((
            fs::File::open(temp.path())?,
            std::ffi::CString::new("published")?,
            Some(fs::File::open(&root_path)?),
            Some(fs::File::open(root_path.join("config.toml"))?),
            Some(identity),
        ));
        owner.gateway_config_owner = Some((unsafe { libc::geteuid() }, unsafe { libc::getegid() }));
        owner.gateway_config_bytes = b"nonsecret test configuration\n".to_vec();
        let occupied = temp.path().join("occupied");
        fs::create_dir(&occupied)?;
        fs::write(occupied.join("sentinel"), b"existing destination")?;
        let occupied_inode = fs::metadata(&occupied)?.ino();
        let (parent, component, _, _, _) = owner.gateway_config.as_ref().unwrap();
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                parent.as_raw_fd(),
                component.as_ptr(),
                parent.as_raw_fd(),
                c"occupied".as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        let error = std::io::Error::last_os_error();
        assert_eq!(result, -1);
        assert_eq!(error.raw_os_error(), Some(libc::EEXIST));
        assert_eq!(fs::metadata(&occupied)?.ino(), occupied_inode);
        assert_eq!(
            fs::read(occupied.join("sentinel"))?,
            b"existing destination"
        );
        assert_eq!(fs::metadata(&root_path)?.ino(), source.metadata()?.ino());
        assert_eq!(component.as_c_str(), c"published");
        assert!(owner.gateway_config.is_some());
        assert!(exclusion.acquire_non_e3_child().is_err());
        fs::write(root_path.join("unexpected"), b"must remain")?;
        assert!(E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry
        )
        .is_err());
        assert!(owner.gateway_config.is_some());
        assert!(root_path.join("config.toml").exists());
        assert!(exclusion.acquire_non_e3_child().is_err());
        fs::remove_file(root_path.join("unexpected"))?;
        let original = temp.path().join("original.toml");
        fs::rename(root_path.join("config.toml"), &original)?;
        assert!(E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry
        )
        .is_err());
        assert!(root_path.exists());
        fs::write(root_path.join("config.toml"), b"substituted")?;
        assert!(E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry
        )
        .is_err());
        assert_eq!(fs::read(root_path.join("config.toml"))?, b"substituted");
        fs::remove_file(root_path.join("config.toml"))?;
        fs::rename(&original, root_path.join("config.toml"))?;
        // In-place mutation retains the same inode and must still prevent terminal cleanup.
        fs::write(
            root_path.join("config.toml"),
            b"corrupted test configuration\n",
        )?;
        assert!(E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry
        )
        .is_err());
        assert!(owner.gateway_config.is_some());
        fs::write(
            root_path.join("config.toml"),
            b"nonsecret test configuration\n",
        )?;
        fs::set_permissions(
            root_path.join("config.toml"),
            fs::Permissions::from_mode(0o640),
        )?;
        assert!(E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry
        )
        .is_err());
        fs::set_permissions(
            root_path.join("config.toml"),
            fs::Permissions::from_mode(0o600),
        )?;
        E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry,
        )?;
        assert!(!root_path.exists());
        assert!(owner.gateway_config.is_none());
        E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut owner),
            &registry,
        )?;
        // Revocation alone retains exclusion until the manager explicitly completes release.
        assert!(exclusion.acquire_non_e3_child().is_err());
        owner.release_exclusion_after_cleanup_v1()?;
        owner.release_exclusion_after_cleanup_v1()?;
        drop(owner);
        assert!(exclusion.acquire_non_e3_child().is_ok());
        Ok(())
    }

    #[test]
    #[ignore = "requires root and actual cgroup v2/netfilter; run the bounded E3-E kernel harness"]
    fn e3_e_kernel_prepare_boundary_and_cleanup() -> Result<()> {
        use config_projection::*;
        use sha2::{Digest, Sha256};
        use std::os::fd::{AsFd, BorrowedFd};
        use std::os::unix::fs::MetadataExt;
        struct TestParent(PathBuf);
        impl ConfigProjectionHsaAuthorityV1 for TestParent {
            fn with_locked_parent(
                &self,
                operation: &mut dyn for<'fd> FnMut(
                    BorrowedFd<'fd>,
                ) -> std::result::Result<
                    (),
                    ConfigProjectionFailureV1,
                >,
            ) -> std::result::Result<(), ConfigProjectionFailureV1> {
                let lock = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(self.0.join("parent.lock"))
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                let authority = fs::File::open(self.0.join("authority-v1"))
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                operation(authority.as_fd())?;
                if std::env::var_os("E3_E_HEADLESS_LOST_RETURN").is_some()
                    && fs::read_dir(
                        self.0.join(
                            "authority-v1/agent-config-projection-v1/kernel-effects/resolutions",
                        ),
                    )
                    .is_ok_and(|mut entries| entries.next().is_some())
                {
                    unsafe { libc::_exit(78) };
                }
                Ok(())
            }
        }
        fn seal<T: Serialize>(domain: &str, key: &str, omitted: &str, value: &T) -> Result<String> {
            let mut object = serde_json::to_value(value)?;
            object
                .as_object_mut()
                .context("fixture object")?
                .remove(omitted);
            Ok(ConfigProjectionCodecV1::domain_sha256(
                domain,
                &serde_json::json!({key:object}),
            )?)
        }
        anyhow::ensure!(
            unsafe { libc::geteuid() } == 0,
            "kernel harness requires root"
        );
        if let Some(root) = std::env::var_os("E3_E_HEADLESS_REOPEN") {
            let registry =
                ConfigProjectionRegistryV1::open(Arc::new(TestParent(PathBuf::from(root))))?;
            let before = registry.recover(None)?;
            let exclusion = E3PrivilegedChildExclusionV1::new_recovering()?;
            let instance = format!("wsi_{}", uuid::Uuid::now_v7());
            if std::env::var_os("E3_E_HEADLESS_MOUNT_REJECTION").is_some() {
                let child_path = before
                    .kernel_effects
                    .iter()
                    .find_map(|effect| match &effect.intent.effect {
                        E3KernelEffectKindV1::CreateChildCgroup {
                            expected_relative_path,
                            ..
                        } => Some(format!("/sys/fs/cgroup/{expected_relative_path}")),
                        _ => None,
                    })
                    .context("fixture cgroup intent")?;
                let path = std::ffi::CString::new(child_path)?;
                anyhow::ensure!(
                    unsafe {
                        libc::mount(
                            path.as_ptr(),
                            path.as_ptr(),
                            std::ptr::null(),
                            libc::MS_BIND,
                            std::ptr::null(),
                        )
                    } == 0,
                    "fixture bind mount failed"
                );
                let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                    E3GatewayRevocationTargetV1::Recovered {
                        effects: &before.kernel_effects,
                        service_instance_id: &instance,
                        exclusion: &exclusion,
                    },
                    &registry,
                );
                anyhow::ensure!(
                    unsafe { libc::umount2(path.as_ptr(), 0) } == 0,
                    "fixture unmount failed"
                );
                anyhow::ensure!(
                    rejected.is_err()
                        && registry
                            .recover(None)?
                            .kernel_effects
                            .iter()
                            .all(|effect| effect.resolution.is_none()),
                    "mounted unregistered effect was accepted"
                );
                exclusion.require_recovering_v1()?;
                eprintln!("E3_HEADLESS_BIND_MOUNT_REJECTED");
                return Ok(());
            }
            E3GatewayRuntimeAuthorityV1::revoke(
                E3GatewayRevocationTargetV1::Recovered {
                    effects: &before.kernel_effects,
                    service_instance_id: &instance,
                    exclusion: &exclusion,
                },
                &registry,
            )?;
            let after = registry.recover(None)?;
            for old in &before.kernel_effects {
                if let Some(resolution) = &old.resolution {
                    anyhow::ensure!(
                        after
                            .kernel_effects
                            .iter()
                            .any(|effect| effect.resolution.as_ref() == Some(resolution)),
                        "process retry rewrote resolution"
                    );
                }
            }
            anyhow::ensure!(
                after
                    .kernel_effects
                    .iter()
                    .all(|effect| effect.resolution.is_some()
                        && effect.child_cgroup.is_none()
                        && effect.child_processes.is_empty()
                        && effect.projection.is_none()
                        && effect.terminal_child_evidence.is_empty()),
                "headless process recovery incomplete"
            );
            exclusion.require_recovering_v1()?;
            eprintln!("E3_HEADLESS_ACTUAL_PROCESS_REOPEN_EXACT_RESOLUTION");
            return Ok(());
        }
        let root = tempfile::Builder::new().prefix("e3kn-").tempdir()?.keep();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        fs::create_dir(root.join("authority-v1"))?;
        fs::set_permissions(root.join("authority-v1"), fs::Permissions::from_mode(0o700))?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("parent.lock"))?;
        let registry = ConfigProjectionRegistryV1::open(Arc::new(TestParent(root.clone())))?;
        let store = registry.recover(None).map(|readback| readback.store)?;
        let world_id = format!("e3-test-{}", uuid::Uuid::now_v7());
        let parent_path = Path::new("/sys/fs/cgroup/substrate").join(&world_id);
        anyhow::ensure!(
            parent_path
                .parent()
                .context("cgroup fixture parent")?
                .is_dir(),
            "existing substrate cgroup parent is absent"
        );
        fs::create_dir(&parent_path)?;
        fs::set_permissions(&parent_path, fs::Permissions::from_mode(0o700))?;
        eprintln!(
            "E3_KERNEL_FIXTURE authority={} parent={}",
            root.display(),
            parent_path.display()
        );
        let mount_metadata = fs::metadata("/sys/fs/cgroup")?;
        let parent_metadata = fs::metadata(&parent_path)?;
        let parent = CanonicalCgroupIdentityV1 {
            cgroup_v2_mount_device_id: mount_metadata.dev(),
            cgroup_v2_mount_inode: mount_metadata.ino(),
            cgroup_directory_inode: parent_metadata.ino(),
            cgroup_relative_path: format!("substrate/{world_id}"),
        };
        let series_id = format!("cps_{}", uuid::Uuid::now_v7());
        let preparation_id = format!("e3p_{}", uuid::Uuid::now_v7());
        let fence_id = format!("cpf_{}", uuid::Uuid::now_v7());
        let now =
            Timestamp(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
        if std::env::var_os("E3_E_HEADLESS_RECOVERY_FIXTURE").is_some() {
            let registration = format!("ecg_{}", uuid::Uuid::now_v7());
            let component = format!(
                "substrate-e3-{}",
                &format!("{:x}", Sha256::digest(registration.as_bytes()))[..24]
            );
            let boundary = format!("gab_{}", uuid::Uuid::now_v7());
            for effect in [
                E3KernelEffectKindV1::CreateChildCgroup {
                    cgroup_registration_id: registration,
                    role: E3TerminalProcessRoleV1::ManagedGateway,
                    parent_cgroup: parent.clone(),
                    child_component: component.clone(),
                    expected_relative_path: format!("{}/{component}", parent.cgroup_relative_path),
                },
                E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: boundary.clone(),
                    network_namespace_inode: fs::metadata("/proc/self/ns/net")?.ino(),
                    table_name: format!(
                        "substrate_e3_{}",
                        &format!("{:x}", Sha256::digest(boundary.as_bytes()))[..24]
                    ),
                    chain_name: "gateway_output".into(),
                },
            ] {
                let mut intent = E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: series_id.clone(),
                    effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                    preparation_id: preparation_id.clone(),
                    fence_id: fence_id.clone(),
                    effect,
                    created_at: now.clone(),
                    intent_hash: String::new(),
                };
                intent.intent_hash = seal(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    "intent_hash",
                    &intent,
                )?;
                registry.publish_kernel_effect_intent(&intent, None, None)?;
            }
            let boundary_present = std::env::var("E3_E_HEADLESS_RECOVERY_FIXTURE")
                .as_deref()
                .is_ok_and(|mode| matches!(mode, "boundary" | "boundary-restart"));
            let boundary_socket = if boundary_present {
                anyhow::ensure!(
                    fs::metadata("/proc/self/ns/net")?.ino()
                        != fs::metadata("/proc/1/ns/net")?.ino(),
                    "boundary fixture needs an isolated network namespace"
                );
                let table = format!(
                    "substrate_e3_{}",
                    &format!("{:x}", Sha256::digest(boundary.as_bytes()))[..24]
                );
                let raw = unsafe {
                    libc::socket(
                        libc::AF_NETLINK,
                        libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                        libc::NETLINK_NETFILTER,
                    )
                };
                anyhow::ensure!(raw >= 0, "fixture netfilter socket");
                let socket = unsafe { OwnedFd::from_raw_fd(raw) };
                let batch = [
                    message(0x10, 1, 1, 0, &[]),
                    message(0x0a00, 0x605, 2, 1, &string(1, &table)),
                    message(
                        0x0a03,
                        0x605,
                        3,
                        1,
                        &[
                            string(1, &table),
                            string(3, "gateway_output"),
                            attribute(
                                0x8004,
                                &[number(1, 3), number(2, (-100_i32) as u32)].concat(),
                            ),
                            number(5, 1),
                            string(7, "filter"),
                        ]
                        .concat(),
                    ),
                    message(0x11, 1, 4, 0, &[]),
                ]
                .concat();
                exchange(socket.as_raw_fd(), &batch, &[2, 3], false)?;
                eprintln!("E3_HEADLESS_PRESENT_BOUNDARY {table}");
                Some((socket, table))
            } else {
                None
            };
            let present = std::env::var("E3_E_HEADLESS_RECOVERY_FIXTURE")
                .as_deref()
                .is_ok_and(|mode| matches!(mode, "present" | "restart"));
            let child_path = parent_path.join(&component);
            let mut remnant = None;
            if present {
                fs::create_dir(&child_path)?;
                fs::set_permissions(&child_path, fs::Permissions::from_mode(0o700))?;
                fs::create_dir(child_path.join("descendant"))?;
                let child = std::process::Command::new("/bin/sleep").arg("60").spawn()?;
                fs::write(
                    child_path.join("descendant/cgroup.procs"),
                    child.id().to_string(),
                )?;
                remnant = Some(child);
            }
            let observed_inode = present.then(|| fs::metadata(&child_path).unwrap().ino());
            let exclusion = E3PrivilegedChildExclusionV1::new_recovering()?;
            let instance = format!("wsi_{}", uuid::Uuid::now_v7());
            if let Some((socket, table)) = &boundary_socket {
                let extra = [string(1, table), string(3, "unclassified")].concat();
                let create = [
                    message(0x10, 1, 20, 0, &[]),
                    message(0x0a03, 0x605, 21, 1, &extra),
                    message(0x11, 1, 22, 0, &[]),
                ]
                .concat();
                exchange(socket.as_raw_fd(), &create, &[21], false)?;
                let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                    E3GatewayRevocationTargetV1::Recovered {
                        effects: &registry.recover(None)?.kernel_effects,
                        service_instance_id: &instance,
                        exclusion: &exclusion,
                    },
                    &registry,
                );
                anyhow::ensure!(
                    rejected.is_err()
                        && registry
                            .recover(None)?
                            .kernel_effects
                            .iter()
                            .all(|effect| effect.resolution.is_none()),
                    "unknown boundary chain was accepted"
                );
                exclusion.require_recovering_v1()?;
                let delete = [
                    message(0x10, 1, 23, 0, &[]),
                    message(0x0a05, 5, 24, 1, &extra),
                    message(0x11, 1, 25, 0, &[]),
                ]
                .concat();
                exchange(socket.as_raw_fd(), &delete, &[24], false)?;
                eprintln!("E3_HEADLESS_UNKNOWN_CHAIN_REJECTED_WITHOUT_RESOLUTION");
            }
            if present {
                let before = registry.recover(None)?;
                fs::set_permissions(
                    child_path.join("descendant"),
                    fs::Permissions::from_mode(0o777),
                )?;
                let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                    E3GatewayRevocationTargetV1::Recovered {
                        effects: &before.kernel_effects,
                        service_instance_id: &instance,
                        exclusion: &exclusion,
                    },
                    &registry,
                );
                fs::set_permissions(
                    child_path.join("descendant"),
                    fs::Permissions::from_mode(0o755),
                )?;
                anyhow::ensure!(
                    rejected.is_err() && remnant.as_mut().unwrap().try_wait()?.is_none(),
                    "unsafe descendant was acted on"
                );
                anyhow::ensure!(
                    registry
                        .recover(None)?
                        .kernel_effects
                        .iter()
                        .all(|effect| effect.resolution.is_none()),
                    "rejection published cleanup"
                );
                exclusion.require_recovering_v1()?;
                eprintln!("E3_HEADLESS_UNPROTECTED_DESCENDANT_REJECTED_WITH_LIVE_REMNANT");
            }
            if present {
                let status = std::process::Command::new("unshare")
                    .args(["--mount", "--propagation", "private"])
                    .arg(std::env::current_exe()?)
                    .args([
                        "e3_e_kernel_prepare_boundary_and_cleanup",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("E3_E_HEADLESS_REOPEN", &root)
                    .env("E3_E_HEADLESS_MOUNT_REJECTION", "1")
                    .status()?;
                anyhow::ensure!(
                    status.success() && remnant.as_mut().unwrap().try_wait()?.is_none(),
                    "mount substitution rejection failed"
                );
            }
            let restart = std::env::var("E3_E_HEADLESS_RECOVERY_FIXTURE")
                .as_deref()
                .is_ok_and(|mode| matches!(mode, "restart" | "boundary-restart"));
            let cleanup = if restart {
                let mut command = std::process::Command::new(std::env::current_exe()?);
                command
                    .args([
                        "e3_e_kernel_prepare_boundary_and_cleanup",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("E3_E_HEADLESS_REOPEN", &root)
                    .env("E3_E_HEADLESS_LOST_RETURN", "1");
                let interrupted = command.status()?;
                anyhow::ensure!(
                    interrupted.code() == Some(78),
                    "missing resolution lost-return interruption"
                );
                command.env_remove("E3_E_HEADLESS_LOST_RETURN");
                anyhow::ensure!(
                    command.status()?.success(),
                    "actual process resolution retry failed"
                );
                Ok(())
            } else {
                E3GatewayRuntimeAuthorityV1::revoke(
                    E3GatewayRevocationTargetV1::Recovered {
                        effects: &registry.recover(None)?.kernel_effects,
                        service_instance_id: &instance,
                        exclusion: &exclusion,
                    },
                    &registry,
                )
            };
            if let Some(child) = &mut remnant {
                if cleanup.is_err() {
                    let _ = child.kill();
                }
                let status = child.wait()?;
                anyhow::ensure!(!status.success(), "unregistered remnant survived cleanup");
            }
            if cleanup.is_err() && present {
                fs::remove_dir(child_path.join("descendant"))?;
                fs::remove_dir(&child_path)?;
                fs::remove_dir(&parent_path)?;
            }
            cleanup?;
            anyhow::ensure!(!child_path.exists(), "unregistered cgroup survived cleanup");
            let first = registry.recover(None)?;
            anyhow::ensure!(
                first.kernel_effects.len() == 2
                    && first
                        .kernel_effects
                        .iter()
                        .all(|effect| effect.projection.is_none()
                            && effect.terminal_child_evidence.is_empty()
                            && effect.gateway_config.is_none()
                            && effect.child_cgroup.is_none()
                            && effect.child_processes.is_empty()
                            && effect.resolution.as_ref().is_some_and(|resolution| {
                                if present && matches!(effect.intent.effect, E3KernelEffectKindV1::CreateChildCgroup { .. }) {
                                    resolution.disposition == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent
                                        && resolution.observed_cgroup.as_ref().map(|cgroup| cgroup.cgroup_directory_inode) == observed_inode
                                } else if boundary_present && matches!(effect.intent.effect, E3KernelEffectKindV1::InstallGatewayBoundary { .. }) {
                                    resolution.disposition == E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent
                                        && resolution.observed_nftables_table_handle.is_some()
                                } else {
                                    resolution.disposition == E3KernelEffectResolutionDispositionV1::NoEffectObserved
                                }
                            })),
                "headless no-effect cleanup incomplete"
            );
            if present {
                fs::create_dir(&child_path)?;
                let rejected = E3GatewayRuntimeAuthorityV1::revoke(
                    E3GatewayRevocationTargetV1::Recovered {
                        effects: &first.kernel_effects,
                        service_instance_id: &instance,
                        exclusion: &exclusion,
                    },
                    &registry,
                );
                fs::remove_dir(&child_path)?;
                anyhow::ensure!(
                    rejected.is_err(),
                    "resolved cgroup substitution was accepted"
                );
                exclusion.require_recovering_v1()?;
                eprintln!("E3_HEADLESS_RESOLVED_CGROUP_SUBSTITUTION_REJECTED");
            }
            fs::remove_dir(&parent_path)?;
            E3GatewayRuntimeAuthorityV1::revoke(
                E3GatewayRevocationTargetV1::Recovered {
                    effects: &first.kernel_effects,
                    service_instance_id: &instance,
                    exclusion: &exclusion,
                },
                &registry,
            )?;
            let retry = registry.recover(None)?;
            anyhow::ensure!(
                first
                    .kernel_effects
                    .iter()
                    .zip(&retry.kernel_effects)
                    .all(|(old, new)| old.resolution == new.resolution),
                "headless retry changed durable resolution"
            );
            exclusion.finish_recovery()?;
            eprintln!("E3_RECOVERED_HEADLESS_PRESENT={present}_AND_DISAPPEARED_PARENT_EXACT_RETRY");
            return Ok(());
        }
        let exclusion = finished_child_exclusion();
        let mut gateway =
            E3GatewayRuntimeAuthorityV1::new(exclusion.acquire_e3_exclusive(&world_id, 1)?);
        gateway.bind_dormant_listener()?;
        let attempt = (|| -> Result<()> {
            for role in [
                E3TerminalProcessRoleV1::ManagedGateway,
                E3TerminalProcessRoleV1::ReadinessProbe,
                E3TerminalProcessRoleV1::Codex,
            ] {
                let cgroup_registration_id = format!("ecg_{}", uuid::Uuid::now_v7());
                let component = format!(
                    "substrate-e3-{}",
                    &format!("{:x}", Sha256::digest(cgroup_registration_id.as_bytes()))[..24]
                );
                let mut intent = E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: series_id.clone(),
                    effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                    preparation_id: preparation_id.clone(),
                    fence_id: fence_id.clone(),
                    effect: E3KernelEffectKindV1::CreateChildCgroup {
                        cgroup_registration_id,
                        role,
                        parent_cgroup: parent.clone(),
                        child_component: component.clone(),
                        expected_relative_path: format!(
                            "{}/{}",
                            parent.cgroup_relative_path, component
                        ),
                    },
                    created_at: now.clone(),
                    intent_hash: String::new(),
                };
                intent.intent_hash = seal(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    "intent_hash",
                    &intent,
                )?;
                let mut cause = None;
                let result = registry.publish_kernel_effect_intent(
                    &intent,
                    Some(&mut |reference| {
                        gateway
                            .install_dormant_boundary(
                                &intent,
                                reference,
                                GatewayAccessPostureV1::DenyAllDormant,
                                false,
                            )
                            .map_err(|error| {
                                cause = Some(error);
                                ConfigProjectionFailureV1::UnsupportedSecurityPosture
                            })
                    }),
                    None,
                );
                if let Some(error) = cause {
                    return Err(error).context("actual E3 cgroup creation");
                }
                result?;
            }
            let boundary_id = format!("gab_{}", uuid::Uuid::now_v7());
            let table = format!(
                "substrate_e3_{}",
                &format!("{:x}", Sha256::digest(boundary_id.as_bytes()))[..24]
            );
            eprintln!("E3_KERNEL_TABLE {table}");
            let mut intent = E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id,
                effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                preparation_id,
                fence_id,
                effect: E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: boundary_id.clone(),
                    network_namespace_inode: gateway.bound_listener.context("fixture listener")?.2,
                    table_name: table,
                    chain_name: "gateway_output".to_string(),
                },
                created_at: now,
                intent_hash: String::new(),
            };
            intent.intent_hash = seal(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                "intent_hash",
                &intent,
            )?;
            let mut identity = InWorldGatewayIdentityV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                gateway_instance_id: format!("cgi_{}", uuid::Uuid::now_v7()),
                config_projection_identity_hash: "11".repeat(32),
                orchestration_session_id: "kernel-harness".to_string(),
                retained_participant_id: "member".to_string(),
                backend_id: "cli:codex-world".to_string(),
                world_id: world_id.clone(),
                world_generation: 1,
                gateway_artifact_sha256: "22".repeat(32),
                access_boundary_id: boundary_id.clone(),
                gateway_identity_hash: String::new(),
            };
            identity.gateway_identity_hash = seal(
                "substrate.e3.in-world-gateway-identity.v1",
                "gateway",
                "gateway_identity_hash",
                &identity,
            )?;
            let mut cause = None;
            let result = registry.publish_kernel_effect_intent(
                &intent,
                Some(&mut |reference| {
                    gateway
                        .install_dormant_boundary(
                            &intent,
                            reference,
                            GatewayAccessPostureV1::DenyAllDormant,
                            false,
                        )
                        .map_err(|error| {
                            cause = Some(error);
                            ConfigProjectionFailureV1::UnsupportedSecurityPosture
                        })
                }),
                Some(&identity),
            );
            if let Some(error) = cause {
                return Err(error).context("actual E3 dormant boundary installation");
            }
            result?;

            let (boundary, registrations, config) = gateway.listen_after_dormant_boundary(
                &identity,
                &intent.series_id,
                &intent.fence_id,
                u64::from(unsafe { libc::geteuid() }),
                u64::from(unsafe { libc::getegid() }),
            )?;
            assert_eq!(config.relative_path, "config.toml");
            anyhow::ensure!(
                registrations[1].cgroup == boundary.readiness_probe_cgroup
                    && registrations[2].cgroup == boundary.allowed_member_cgroup,
                "ordered registration readback mismatch"
            );
            anyhow::ensure!(
                boundary.gateway_listener.socket_inode
                    == gateway.bound_listener.context("fixture listener")?.1
                    && boundary.nftables_rules.len() == 2
                    && boundary.nftables_rules.iter().all(|r| r.rule_handle != 0),
                "actual boundary readback mismatch"
            );
            let outsider = TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], boundary.gateway_listener.port)),
                Duration::from_secs(1),
            );
            anyhow::ensure!(
                outsider.is_err(),
                "world-service outside the readiness cgroup reached the dormant listener"
            );
            // Move only this test process, never a daemon or unrelated workload. A held control
            // descriptor restores its exact original cgroup even if the positive control fails.
            struct RestoreCgroup(fs::File, String);
            impl Drop for RestoreCgroup {
                fn drop(&mut self) {
                    self.0
                        .write_all(self.1.as_bytes())
                        .expect("restore kernel harness cgroup");
                }
            }
            let membership = fs::read_to_string("/proc/self/cgroup")?;
            let original = membership
                .trim()
                .strip_prefix("0::/")
                .context("fixture cgroup v2 membership")?;
            anyhow::ensure!(
                original
                    .split('/')
                    .all(|part| !part.is_empty() && part != "." && part != ".."),
                "noncanonical fixture membership"
            );
            let restore = RestoreCgroup(
                fs::OpenOptions::new()
                    .write(true)
                    .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
                    .open(
                        Path::new("/sys/fs/cgroup")
                            .join(original)
                            .join("cgroup.procs"),
                    )?,
                std::process::id().to_string(),
            );
            let readiness = &gateway
                .cgroups
                .iter()
                .find(|(_, _, r, _)| r.role == E3TerminalProcessRoleV1::ReadinessProbe)
                .context("fixture readiness cgroup")?
                .1;
            let raw = unsafe {
                libc::openat(
                    readiness.as_raw_fd(),
                    c"cgroup.procs".as_ptr(),
                    libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            anyhow::ensure!(raw >= 0, "open fixture readiness membership writer");
            unsafe { fs::File::from_raw_fd(raw) }
                .write_all(std::process::id().to_string().as_bytes())?;
            let allowed = TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], boundary.gateway_listener.port)),
                Duration::from_secs(1),
            );
            drop(restore);
            let allowed =
                allowed.context("actual readiness cgroup must reach its own dormant listener")?;
            let accepted = unsafe {
                libc::accept4(
                    gateway
                        .listener
                        .as_ref()
                        .context("fixture listener")?
                        .as_raw_fd(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                )
            };
            anyhow::ensure!(accepted >= 0, "accept positive-control readiness socket");
            drop(unsafe { OwnedFd::from_raw_fd(accepted) });
            drop(allowed);
            anyhow::ensure!(
                gateway
                    .listen_after_dormant_boundary(
                        &identity,
                        &intent.series_id,
                        &intent.fence_id,
                        u64::from(unsafe { libc::geteuid() }),
                        u64::from(unsafe { libc::getegid() })
                    )
                    .is_err(),
                "listener admitted a second listen"
            );
            // An occupied child cgroup must leave the revoked attempt owned and exclusion held.
            // Use this harness PID as the occupant; no child or unrelated process is moved.
            let restore = RestoreCgroup(
                fs::OpenOptions::new()
                    .write(true)
                    .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
                    .open(
                        Path::new("/sys/fs/cgroup")
                            .join(original)
                            .join("cgroup.procs"),
                    )?,
                std::process::id().to_string(),
            );
            let member = &gateway
                .cgroups
                .iter()
                .find(|(_, _, r, _)| r.role == E3TerminalProcessRoleV1::Codex)
                .context("fixture member cgroup")?
                .1;
            let raw = unsafe {
                libc::openat(
                    member.as_raw_fd(),
                    c"cgroup.procs".as_ptr(),
                    libc::O_WRONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            anyhow::ensure!(raw >= 0, "open fixture member membership writer");
            unsafe { fs::File::from_raw_fd(raw) }
                .write_all(std::process::id().to_string().as_bytes())?;
            let incomplete_cleanup = E3GatewayRuntimeAuthorityV1::revoke(
                E3GatewayRevocationTargetV1::Live(&mut gateway),
                &registry,
            );
            drop(restore);
            anyhow::ensure!(
                incomplete_cleanup.is_err()
                    && gateway.listener.is_some()
                    && gateway.cgroups.len() == 3
                    && exclusion.acquire_non_e3_child().is_err(),
                "failed cleanup released its resources or process-wide exclusion"
            );
            let (_, _, _, rules, state) = gateway
                .boundary
                .as_ref()
                .context("retained revoked boundary")?;
            anyhow::ensure!(
                *state == GatewayAccessPostureV1::Revoked
                    && rules.len() == 1
                    && rules[0].role == NftablesRuleRoleV1::RejectRemainder,
                "failed cleanup did not retain a reject-only boundary"
            );
            for resolution in &gateway.cleanup_resolutions {
                anyhow::ensure!(
                    !root
                        .join("authority-v1/agent-config-projection-v1/kernel-effects/resolutions")
                        .join(format!("{}.json", resolution.resolution_id))
                        .exists(),
                    "failed cleanup published a terminal resolution"
                );
            }
            eprintln!("E3_KERNEL_FAILED_CLEANUP_RETAINED");
            Ok(())
        })();
        let cleanup = E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut gateway),
            &registry,
        );
        if let Err(error) = &cleanup {
            eprintln!("E3_KERNEL_CLEANUP_RETAINED {error:#}");
        }
        attempt?;
        cleanup?;
        E3GatewayRuntimeAuthorityV1::revoke(
            E3GatewayRevocationTargetV1::Live(&mut gateway),
            &registry,
        )?;
        anyhow::ensure!(
            gateway.bind_dormant_listener().is_err(),
            "a revoked attempt reopened its listener"
        );
        registry.recover(None).map(|readback| readback.store)?;
        anyhow::ensure!(
            gateway.cgroups.is_empty() && gateway.boundary.is_none() && gateway.listener.is_none(),
            "kernel owner retained a cleaned resource"
        );
        gateway.release_exclusion_after_cleanup_v1()?;
        gateway.release_exclusion_after_cleanup_v1()?;
        drop(gateway);
        anyhow::ensure!(
            exclusion.acquire_non_e3_child().is_ok(),
            "cleanup did not release exclusion"
        );
        fs::remove_dir(&parent_path)?;
        // Keep synthetic registry evidence for inspection; it is not installed authority.
        eprintln!("E3_KERNEL_PASS authority={}", root.display());
        Ok(())
    }

    #[test]
    fn e3_e_listener_reservation_cannot_accept_and_holds_exclusion_until_close() {
        let exclusion = finished_child_exclusion();
        let lease = exclusion.acquire_e3_exclusive("e3-bind-test", 1).unwrap();
        let mut gateway = E3GatewayRuntimeAuthorityV1::new(lease);
        gateway.bind_dormant_listener().unwrap();
        gateway.validate_listener_inventory().unwrap();
        let identity = gateway.bound_listener.unwrap();
        assert!(exclusion.acquire_non_e3_child().is_err());
        assert!(exclusion.acquire_e3_exclusive("e3-other", 1).is_err());
        let address = std::net::SocketAddr::from(([127, 0, 0, 1], identity.0));
        let result = TcpStream::connect_timeout(&address, Duration::from_secs(1));
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::ConnectionRefused
        );
        assert!(gateway.bind_dormant_listener().is_err());
        assert_eq!(gateway.bound_listener, Some(identity));
        gateway.validate_listener_inventory().unwrap();
        drop(gateway);
        assert!(exclusion.acquire_non_e3_child().is_err());
        // Descriptor drop closes the listener, but unexpected owner loss keeps admission closed.
        let replacement = StdTcpListener::bind(address).unwrap();
        drop(replacement);
    }

    #[test]
    fn e3_e_listener_owner_loss_with_unresolved_intent_keeps_admission_closed() {
        let exclusion = finished_child_exclusion();
        let mut gateway = E3GatewayRuntimeAuthorityV1::new(
            exclusion.acquire_e3_exclusive("e3-owner-loss", 1).unwrap(),
        );
        gateway.bind_dormant_listener().unwrap();
        let port = gateway.bound_listener.unwrap().0;
        // Model the durable callback boundary before its first syscall. No fake kernel object
        // is used or claimed; this test covers the destructor's admission invariant only.
        gateway
            .unresolved_intents
            .push(config_projection::E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: format!("cpa_{}", uuid::Uuid::now_v7()),
                series_id: format!("cps_{}", uuid::Uuid::now_v7()),
                effect_intent_id: format!("eki_{}", uuid::Uuid::now_v7()),
                preparation_id: format!("e3p_{}", uuid::Uuid::now_v7()),
                fence_id: format!("cpf_{}", uuid::Uuid::now_v7()),
                effect: config_projection::E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: format!("gab_{}", uuid::Uuid::now_v7()),
                    network_namespace_inode: gateway.bound_listener.unwrap().2,
                    table_name: "unused-destructor-fixture".to_string(),
                    chain_name: "gateway_output".to_string(),
                },
                created_at: config_projection::Timestamp(chrono::Utc::now().to_rfc3339()),
                intent_hash: String::new(),
            });
        drop(gateway);
        assert!(exclusion.acquire_non_e3_child().is_err());
        assert!(
            StdTcpListener::bind(std::net::SocketAddr::from(([127, 0, 0, 1], port))).is_ok(),
            "unexpected owner loss must still close the secret-capable listener"
        );
    }

    #[test]
    fn e3_e_listener_rejects_reuse_and_descriptor_flag_drift() {
        for changed in [libc::SO_REUSEADDR, libc::SO_REUSEPORT, -1, -2] {
            let exclusion = finished_child_exclusion();
            let mut gateway = E3GatewayRuntimeAuthorityV1::new(
                exclusion.acquire_e3_exclusive("e3-bind-test", 1).unwrap(),
            );
            gateway.bind_dormant_listener().unwrap();
            let fd = gateway.listener.as_ref().unwrap().as_raw_fd();
            let result = if changed > 0 {
                let value = 1_i32;
                unsafe {
                    libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        changed,
                        (&value as *const i32).cast(),
                        std::mem::size_of_val(&value) as libc::socklen_t,
                    )
                }
            } else if changed == -1 {
                unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }
            } else {
                unsafe { libc::fcntl(fd, libc::F_SETFL, 0) }
            };
            assert_eq!(result, 0);
            assert!(gateway.validate_listener_inventory().is_err(), "{changed}");
        }
    }

    #[test]
    fn e3_e_listener_rejects_premature_listen_and_identity_substitution() {
        let exclusion = finished_child_exclusion();
        let mut gateway = E3GatewayRuntimeAuthorityV1::new(
            exclusion.acquire_e3_exclusive("e3-bind-test", 1).unwrap(),
        );
        gateway.bind_dormant_listener().unwrap();
        let identity = gateway.bound_listener.unwrap();
        for altered in [
            (identity.0, identity.1 + 1, identity.2),
            (identity.0, identity.1, identity.2 + 1),
            (0, identity.1, identity.2),
        ] {
            gateway.bound_listener = Some(altered);
            assert!(gateway.validate_listener_inventory().is_err());
        }
        gateway.bound_listener = Some(identity);
        gateway.validate_listener_inventory().unwrap();
        let fd = gateway.listener.as_ref().unwrap().as_raw_fd();
        assert_eq!(unsafe { libc::listen(fd, 16) }, 0);
        assert!(gateway.validate_listener_inventory().is_err());
    }

    fn ordinary_gateway_child_lease() -> Arc<HeldNonE3PrivilegedChildLeaseV1> {
        Arc::new(
            finished_child_exclusion()
                .acquire_non_e3_child()
                .expect("acquire gateway fixture child lease"),
        )
    }

    fn start_context(project_dir: &Path, world_id: &str) -> GatewayRuntimeStartContext {
        start_context_with_codex_auth(
            project_dir,
            world_id,
            "header.payload.signature",
            Some("acct_test"),
        )
    }

    fn start_context_with_codex_auth(
        project_dir: &Path,
        world_id: &str,
        access_token: &str,
        account_id: Option<&str>,
    ) -> GatewayRuntimeStartContext {
        let binding = resolve_gateway_backend_binding(DEFAULT_BACKEND).expect("codex binding");
        GatewayRuntimeStartContext {
            world_id: world_id.to_string(),
            project_dir: project_dir.to_path_buf(),
            cgroup_path: project_dir.join("missing-cgroup"),
            require_cgroup_attach: false,
            binding,
            integrated_auth: Some(GatewayIntegratedAuthPayloadV1 {
                backend_id: binding.backend_id.to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: account_id.map(str::to_string),
                    access_token: access_token.to_string(),
                }),
                api_env: None,
            }),
        }
    }

    fn start_context_with_binding(
        project_dir: &Path,
        world_id: &str,
        binding: &'static GatewayBackendBinding,
    ) -> GatewayRuntimeStartContext {
        GatewayRuntimeStartContext {
            world_id: world_id.to_string(),
            project_dir: project_dir.to_path_buf(),
            cgroup_path: project_dir.join("missing-cgroup"),
            require_cgroup_attach: false,
            binding,
            integrated_auth: integrated_auth_for_binding(binding),
        }
    }

    fn integrated_auth_for_binding(
        binding: &GatewayBackendBinding,
    ) -> Option<GatewayIntegratedAuthPayloadV1> {
        match binding.backend_id {
            DEFAULT_BACKEND | CLI_CODEX_HOST_BACKEND | CLI_CODEX_WORLD_BACKEND => {
                Some(GatewayIntegratedAuthPayloadV1 {
                    backend_id: binding.backend_id.to_string(),
                    cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                        account_id: Some("acct_test".to_string()),
                        access_token: "header.payload.signature".to_string(),
                    }),
                    api_env: None,
                })
            }
            CLI_CLAUDE_CODE_BACKEND
            | CLI_CLAUDE_CODE_HOST_BACKEND
            | CLI_CLAUDE_CODE_WORLD_BACKEND => Some(anthropic_integrated_auth_payload_for_backend(
                binding.backend_id,
                "sk-ant-proof",
            )),
            API_OPENAI_BACKEND => Some(openai_integrated_auth_payload("sk-openai-test")),
            _ => None,
        }
    }

    fn anthropic_integrated_auth_payload(api_key: &str) -> GatewayIntegratedAuthPayloadV1 {
        anthropic_integrated_auth_payload_for_backend(CLI_CLAUDE_CODE_BACKEND, api_key)
    }

    fn anthropic_integrated_auth_payload_for_backend(
        backend_id: &str,
        api_key: &str,
    ) -> GatewayIntegratedAuthPayloadV1 {
        let mut env = HashMap::new();
        env.insert(ANTHROPIC_API_KEY_ENV.to_string(), api_key.to_string());
        GatewayIntegratedAuthPayloadV1 {
            backend_id: backend_id.to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        }
    }

    fn openai_integrated_auth_payload(api_key: &str) -> GatewayIntegratedAuthPayloadV1 {
        let mut env = HashMap::new();
        env.insert(OPENAI_API_KEY_ENV.to_string(), api_key.to_string());
        GatewayIntegratedAuthPayloadV1 {
            backend_id: API_OPENAI_BACKEND.to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        }
    }

    fn auth_bundle_snapshot_path(temp_dir: &TempDir, launch: u32) -> PathBuf {
        temp_dir
            .path()
            .join("auth-bundles")
            .join(format!("{launch}.json"))
    }

    fn delayed_gateway_binary(temp_dir: &TempDir, delay_ms: u64) -> (PathBuf, PathBuf, PathBuf) {
        let path = temp_dir.path().join("delayed-gateway.sh");
        let pid_dir = temp_dir.path().join("pids");
        let bundle_dir = temp_dir.path().join("auth-bundles");
        let launch_count_path = temp_dir.path().join("launch-count.txt");
        fs::write(
            &path,
            format!(
                r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_AUTH_BUNDLE_FD:-}}" ]; then
  echo "missing gateway auth bundle fd env" >&2
  exit 65
fi

launch="$(python3 - "{launch_count_path}" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
count = int(path.read_text(encoding='utf-8').strip()) if path.exists() else 0
count += 1
path.write_text(str(count), encoding='utf-8')
print(count)
PY
)"
mkdir -p "{pid_dir}"
printf '%s\n' "$$" >"{pid_dir}/$launch.pid"

python3 - "$launch" "{bundle_dir}" <<'PY'
import json
import os
import pathlib
import sys

launch = sys.argv[1]
bundle_dir = pathlib.Path(sys.argv[2])
bundle_dir.mkdir(parents=True, exist_ok=True)
for forbidden in (
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
    "SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY",
    "OPENAI_API_KEY",
):
    if os.environ.get(forbidden):
        raise SystemExit("forbidden secret env present: " + forbidden)
fd_raw = os.environ["SUBSTRATE_LLM_AUTH_BUNDLE_FD"]
with os.fdopen(int(fd_raw), "rb") as handle:
    payload = handle.read()
bundle = json.loads(payload)
(bundle_dir / (launch + ".json")).write_bytes(payload)
if bundle.get("backend_id") != "cli:codex":
    raise SystemExit("unexpected backend_id")
fields = bundle.get("fields") or dict()
if "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN" not in fields:
    raise SystemExit("missing Codex token field")
PY

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

sleep {delay_s}
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
                launch_count_path = launch_count_path.display(),
                bundle_dir = bundle_dir.display(),
                pid_dir = pid_dir.display(),
                delay_s = format_delay_seconds(delay_ms),
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        (path, pid_dir, launch_count_path)
    }

    fn first_launch_hangs_second_ready_binary(temp_dir: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
        let path = temp_dir.path().join("phased-gateway.sh");
        let pid_dir = temp_dir.path().join("pids");
        let bundle_dir = temp_dir.path().join("auth-bundles");
        let launch_count_path = temp_dir.path().join("launch-count.txt");
        fs::write(
            &path,
            format!(
                r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_AUTH_BUNDLE_FD:-}}" ]; then
  echo "missing gateway auth bundle fd env" >&2
  exit 65
fi

launch="$(python3 - "{launch_count_path}" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
count = int(path.read_text(encoding='utf-8').strip()) if path.exists() else 0
count += 1
path.write_text(str(count), encoding='utf-8')
print(count)
PY
)"
mkdir -p "{pid_dir}"
printf '%s\n' "$$" >"{pid_dir}/$launch.pid"

python3 - "$launch" "{bundle_dir}" <<'PY'
import json
import os
import pathlib
import sys

launch = sys.argv[1]
bundle_dir = pathlib.Path(sys.argv[2])
bundle_dir.mkdir(parents=True, exist_ok=True)
for forbidden in (
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
    "SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY",
    "OPENAI_API_KEY",
):
    if os.environ.get(forbidden):
        raise SystemExit("forbidden secret env present: " + forbidden)
fd_raw = os.environ["SUBSTRATE_LLM_AUTH_BUNDLE_FD"]
with os.fdopen(int(fd_raw), "rb") as handle:
    payload = handle.read()
bundle = json.loads(payload)
(bundle_dir / (launch + ".json")).write_bytes(payload)
if bundle.get("backend_id") != "cli:codex":
    raise SystemExit("unexpected backend_id")
fields = bundle.get("fields") or dict()
if "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN" not in fields:
    raise SystemExit("missing Codex token field")
PY

if [ "$launch" = "1" ]; then
  sleep 30
  exit 0
fi

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
                launch_count_path = launch_count_path.display(),
                bundle_dir = bundle_dir.display(),
                pid_dir = pid_dir.display(),
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        (path, pid_dir, launch_count_path)
    }

    fn format_delay_seconds(delay_ms: u64) -> String {
        format!("{:.3}", delay_ms as f64 / 1000.0)
    }

    fn wait_for_pid(pid_dir: &Path, launch: u32) -> u32 {
        let path = pid_dir.join(format!("{launch}.pid"));
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Ok(raw) = fs::read_to_string(&path) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    let pid = trimmed.parse::<u32>().expect("parse pid");
                    if pid > 0 {
                        return pid;
                    }
                }
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    fn wait_for_auth_bundle_snapshot(temp_dir: &TempDir, launch: u32) -> GatewayAuthBundleV1 {
        let path = auth_bundle_snapshot_path(temp_dir, launch);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Ok(raw) = fs::read(&path) {
                return serde_json::from_slice(&raw)
                    .unwrap_or_else(|err| panic!("invalid {}: {err}", path.display()));
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    fn read_launch_count(path: &Path) -> u32 {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| raw.trim().parse::<u32>().ok())
            .unwrap_or(0)
    }

    fn assert_process_exited(pid: u32) {
        let rc = unsafe { libc::kill(pid as i32, 0) };
        assert_eq!(rc, -1, "expected pid {pid} to be gone");
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH),
            "pid {pid} should be gone",
        );
    }

    fn assert_mode(path: &Path, expected: u32) {
        let actual = fs::metadata(path)
            .unwrap_or_else(|_| panic!("missing {}", path.display()))
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(actual, expected, "unexpected mode for {}", path.display());
    }

    fn start_strict_health_server() -> u16 {
        let listener = StdTcpListener::bind(("127.0.0.1", 0)).expect("bind strict health server");
        let port = listener
            .local_addr()
            .expect("strict health server addr")
            .port();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept strict health connection");
            stream
                .set_read_timeout(Some(Duration::from_millis(150)))
                .expect("set strict health read timeout");

            let mut request = Vec::new();
            let mut buf = [0_u8; 256];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => return,
                    Ok(read) => {
                        request.extend_from_slice(&buf[..read]);
                        if request.windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(err) => panic!("failed reading strict health request: {err}"),
                }
            }

            let mut probe = [0_u8; 1];
            match stream.read(&mut probe) {
                Ok(0) => {}
                Ok(_) => panic!("unexpected extra bytes after strict health request"),
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        || err.kind() == std::io::ErrorKind::TimedOut =>
                {
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                        )
                        .expect("write strict health response");
                }
                Err(err) => panic!("failed probing strict health client state: {err}"),
            }
        });
        port
    }

    fn legacy_gateway_health_probe_blocking(port: u16) -> bool {
        let mut stream = match TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}")
                .parse()
                .expect("legacy health socket address"),
            Duration::from_millis(250),
        ) {
            Ok(stream) => stream,
            Err(_) => return false,
        };

        if stream
            .set_read_timeout(Some(Duration::from_millis(250)))
            .is_err()
            || stream
                .set_write_timeout(Some(Duration::from_millis(250)))
                .is_err()
        {
            return false;
        }

        let request =
            format!("GET {HEALTH_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
        if stream.write_all(request.as_bytes()).is_err() {
            return false;
        }
        let _ = stream.shutdown(std::net::Shutdown::Write);

        let mut response = String::new();
        if stream.read_to_string(&mut response).is_err() {
            return false;
        }
        response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
    }

    #[test]
    fn append_gateway_start_args_uses_global_config_flag_position() {
        let mut command = Command::new("/bin/true");
        let config_path = Path::new("/tmp/config.toml");
        append_gateway_start_args(&mut command, config_path);

        let args: Vec<_> = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["--config", "/tmp/config.toml", "start"]);
    }

    #[test]
    fn empty_default_backend_stays_invalid() {
        let mut env = HashMap::new();
        env.insert(
            GATEWAY_REQUEST_DEFAULT_BACKEND_ENV.to_string(),
            "   ".to_string(),
        );

        let err = GatewayControlSettings::from_request_env(Some(&env)).expect_err("empty backend");
        assert!(
            matches!(err, GatewayRuntimeFailure::InvalidIntegration(_)),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn nonempty_unbound_backend_is_accepted_as_selected_input() {
        let mut env = HashMap::new();
        env.insert(
            GATEWAY_REQUEST_DEFAULT_BACKEND_ENV.to_string(),
            "api:anthropic".to_string(),
        );

        let control =
            GatewayControlSettings::from_request_env(Some(&env)).expect("selected backend");
        assert_eq!(control.default_backend, "api:anthropic");
    }

    #[test]
    fn binding_lookup_includes_explicit_openai_proof_target() {
        let binding = resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        assert_eq!(binding.backend_id, API_OPENAI_BACKEND);
        assert_eq!(binding.provider_name, OPENAI_PROVIDER_NAME);
    }

    #[test]
    fn binding_lookup_includes_explicit_claude_code_proof_target() {
        let binding =
            resolve_gateway_backend_binding(CLI_CLAUDE_CODE_BACKEND).expect("claude binding");
        assert_eq!(binding.backend_id, CLI_CLAUDE_CODE_BACKEND);
        assert_eq!(binding.provider_name, CLAUDE_PROVIDER_NAME);
    }

    #[test]
    fn binding_lookup_includes_realized_codex_targets() {
        for backend_id in [CLI_CODEX_HOST_BACKEND, CLI_CODEX_WORLD_BACKEND] {
            let binding = resolve_gateway_backend_binding(backend_id)
                .unwrap_or_else(|| panic!("missing binding for {backend_id}"));
            assert_eq!(binding.backend_id, backend_id);
            assert_eq!(binding.auth_bundle_backend_id, DEFAULT_BACKEND);
            assert_eq!(binding.provider_name, DEFAULT_PROVIDER_NAME);
        }
    }

    #[test]
    fn binding_lookup_includes_realized_claude_code_targets() {
        for backend_id in [CLI_CLAUDE_CODE_HOST_BACKEND, CLI_CLAUDE_CODE_WORLD_BACKEND] {
            let binding = resolve_gateway_backend_binding(backend_id)
                .unwrap_or_else(|| panic!("missing binding for {backend_id}"));
            assert_eq!(binding.backend_id, backend_id);
            assert_eq!(binding.auth_bundle_backend_id, CLI_CLAUDE_CODE_BACKEND);
            assert_eq!(binding.provider_name, CLAUDE_PROVIDER_NAME);
        }
    }

    #[test]
    fn binding_lookup_returns_none_for_unbound_backend() {
        assert!(resolve_gateway_backend_binding("api:anthropic").is_none());
    }

    #[test]
    fn integrated_config_rendering_is_binding_driven() {
        let codex_binding =
            resolve_gateway_backend_binding(DEFAULT_BACKEND).expect("codex binding");
        let codex_config = render_integrated_config(4317, codex_binding);
        assert!(codex_config.contains("auth_type = \"oauth\""));
        assert!(codex_config.contains("oauth_provider = \"openai-codex\""));
        assert!(!codex_config.contains("api_key = \"$OPENAI_API_KEY\""));

        let openai_binding =
            resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        let openai_config = render_integrated_config(4318, openai_binding);
        assert!(openai_config.contains("auth_type = \"apikey\""));
        assert!(openai_config.contains("api_key = \"$OPENAI_API_KEY\""));
        assert!(!openai_config.contains("oauth_provider ="));

        let claude_binding =
            resolve_gateway_backend_binding(CLI_CLAUDE_CODE_BACKEND).expect("claude binding");
        let claude_config = render_integrated_config(4319, claude_binding);
        assert!(claude_config.contains("provider_type = \"anthropic\""));
        assert!(claude_config.contains("auth_type = \"apikey\""));
        assert!(claude_config.contains("api_key = \"$ANTHROPIC_API_KEY\""));
        assert!(!claude_config.contains("oauth_provider ="));
    }

    #[test]
    fn openai_auth_handoff_uses_api_env_when_available() {
        let binding = resolve_gateway_backend_binding(API_OPENAI_BACKEND).expect("openai binding");
        let auth = resolve_integrated_auth_handoff(
            binding,
            Some(openai_integrated_auth_payload("sk-openai-proof")),
        )
        .expect("openai auth handoff");

        assert_eq!(
            auth.bundle.fields,
            HashMap::from([(
                SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY.to_string(),
                "sk-openai-proof".to_string(),
            )])
        );
    }

    #[test]
    fn claude_auth_handoff_uses_api_env_when_available() {
        let binding =
            resolve_gateway_backend_binding(CLI_CLAUDE_CODE_BACKEND).expect("claude binding");
        let auth = resolve_integrated_auth_handoff(
            binding,
            Some(anthropic_integrated_auth_payload("sk-ant-proof")),
        )
        .expect("claude auth handoff");

        assert_eq!(
            auth.bundle.fields,
            HashMap::from([(
                SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY.to_string(),
                "sk-ant-proof".to_string(),
            )])
        );
    }

    #[test]
    fn realized_claude_auth_handoff_uses_canonical_bundle_backend() {
        let binding = resolve_gateway_backend_binding(CLI_CLAUDE_CODE_HOST_BACKEND)
            .expect("realized claude binding");
        let auth = resolve_integrated_auth_handoff(
            binding,
            Some(anthropic_integrated_auth_payload_for_backend(
                CLI_CLAUDE_CODE_HOST_BACKEND,
                "sk-ant-proof",
            )),
        )
        .expect("realized claude auth handoff");

        assert_eq!(auth.bundle.backend_id, CLI_CLAUDE_CODE_BACKEND);
        assert_eq!(
            auth.bundle.fields,
            HashMap::from([(
                SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY.to_string(),
                "sk-ant-proof".to_string(),
            )])
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn capability_gate_fails_before_runtime_artifacts_are_created() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let world_id = "missing-capability";

        let err = start_runtime(
            temp_dir.path().join("missing-binary"),
            start_context_with_binding(temp_dir.path(), world_id, &MISSING_CAPABILITY_BINDING),
            ordinary_gateway_child_lease(),
        )
        .expect_err("capability gate should fail before spawn");

        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("required capability"),
            "capability gate should explain the missing requirement: {err}"
        );
        assert!(
            !runtime_dir_for_world(world_id, MISSING_CAPABILITY_BINDING.backend_id).exists(),
            "pre-spawn capability gating should not create runtime artifacts"
        );
    }

    #[test]
    fn gateway_health_probe_accepts_server_that_rejects_half_closed_clients() {
        let port = start_strict_health_server();
        assert!(
            gateway_health_ready_blocking(port),
            "fixed readiness probe should accept strict health server",
        );
    }

    #[test]
    fn legacy_gateway_health_probe_fails_against_strict_server() {
        let port = start_strict_health_server();
        assert!(
            !legacy_gateway_health_probe_blocking(port),
            "legacy half-close readiness probe should fail against strict health server",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn managed_runtime_artifacts_use_group_readable_modes() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, _pid_dir, _launch_count_path) = delayed_gateway_binary(&temp_dir, 0);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);

        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let runtime = start_runtime(
            PathBuf::from(std::env::var_os(GATEWAY_BINARY_OVERRIDE_ENV).unwrap()),
            start_context(&project_dir, "world-modes"),
            ordinary_gateway_child_lease(),
        )
        .expect("create runtime");

        assert_mode(&runtime_root, GATEWAY_RUNTIME_DIR_MODE);
        let backend_root = backend_runtime_root_dir(DEFAULT_BACKEND);
        assert_mode(&backend_root, GATEWAY_RUNTIME_DIR_MODE);
        assert_eq!(runtime.runtime_dir.parent(), Some(backend_root.as_path()));
        assert_mode(&runtime.runtime_dir, GATEWAY_RUNTIME_DIR_MODE);
        assert_mode(&runtime.runtime_dir.join("home"), GATEWAY_RUNTIME_DIR_MODE);
        assert_mode(&runtime.config_path, GATEWAY_RUNTIME_FILE_MODE);
        assert_mode(
            &runtime.runtime_dir.join("stdout.log"),
            GATEWAY_RUNTIME_FILE_MODE,
        );
        assert_mode(
            &runtime.runtime_dir.join("stderr.log"),
            GATEWAY_RUNTIME_FILE_MODE,
        );
        assert_mode(&runtime.manifest_path, GATEWAY_RUNTIME_FILE_MODE);
        let bundle = wait_for_auth_bundle_snapshot(&temp_dir, 1);
        assert_eq!(bundle.backend_id, DEFAULT_BACKEND);
        assert_eq!(
            bundle
                .fields
                .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
            Some(&"header.payload.signature".to_string())
        );
        assert_eq!(
            bundle
                .fields
                .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID),
            Some(&"acct_test".to_string())
        );

        stop_runtime(runtime).expect("stop runtime");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_same_world_sync_reuses_one_runtime() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, launch_count_path) = delayed_gateway_binary(&temp_dir, 350);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new(finished_child_exclusion()));
        let ctx = start_context(temp_dir.path(), "same-world");

        let left = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });
        let right = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });

        let left = left.await.unwrap().expect("left sync");
        let right = right.await.unwrap().expect("right sync");
        assert_eq!(left.status, GatewayStatusV1::Available);
        assert_eq!(right.status, GatewayStatusV1::Available);
        assert_eq!(read_launch_count(&launch_count_path), 1);
        wait_for_pid(&pid_dir, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timed_out_cleanup_does_not_kill_next_same_world_runtime() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, launch_count_path) =
            first_launch_hangs_second_ready_binary(&temp_dir);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new(finished_child_exclusion()));
        let ctx = start_context(temp_dir.path(), "cleanup-safe");

        let first = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move {
                manager
                    .sync_with_timeout(ctx, Duration::from_millis(250))
                    .await
            }
        });
        let first_pid = wait_for_pid(&pid_dir, 1);
        let second = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });

        let first_err = first
            .await
            .unwrap()
            .expect_err("first sync should time out");
        assert!(
            matches!(first_err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {first_err:?}"
        );
        let second = second.await.unwrap().expect("second sync should recover");
        assert_eq!(second.status, GatewayStatusV1::Available);
        assert_eq!(read_launch_count(&launch_count_path), 2);
        assert_process_exited(first_pid);
        let second_pid = wait_for_pid(&pid_dir, 2);
        assert_ne!(first_pid, second_pid);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_during_start_returns_transient_failure() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, pid_dir, _) = delayed_gateway_binary(&temp_dir, 1000);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new(finished_child_exclusion()));
        let ctx = start_context(temp_dir.path(), "status-start");

        let sync = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.sync_with_timeout(ctx, Duration::from_secs(3)).await }
        });
        wait_for_pid(&pid_dir, 1);

        let err = manager
            .status("status-start", DEFAULT_BACKEND)
            .await
            .expect_err("status should surface a transient start failure");
        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("starting"),
            "unexpected error text: {err}"
        );

        let response = sync.await.unwrap().expect("sync should finish");
        assert_eq!(response.status, GatewayStatusV1::Available);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_during_restart_returns_transient_failure() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (ready_binary, _, _) = delayed_gateway_binary(&temp_dir, 0);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, &ready_binary);
        let manager = Arc::new(GatewayRuntimeManager::new(finished_child_exclusion()));
        let ctx = start_context(temp_dir.path(), "restart-start");

        manager
            .sync_with_timeout(ctx.clone(), Duration::from_secs(3))
            .await
            .expect("initial sync");

        let (restart_binary, pid_dir, _) = delayed_gateway_binary(&temp_dir, 1000);
        std::env::set_var(GATEWAY_BINARY_OVERRIDE_ENV, restart_binary);
        let restart = tokio::spawn({
            let manager = Arc::clone(&manager);
            let ctx = ctx.clone();
            async move { manager.restart(ctx).await }
        });
        wait_for_pid(&pid_dir, 1);

        let deadline = Instant::now() + Duration::from_secs(1);
        let err = loop {
            match manager.status("restart-start", DEFAULT_BACKEND).await {
                Err(err) => break err,
                Ok(response) => {
                    assert_ne!(response.status, GatewayStatusV1::Unavailable);
                }
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for restart-in-progress status"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        };
        assert!(
            matches!(err, GatewayRuntimeFailure::Transient(_)),
            "unexpected error: {err:?}"
        );
        assert!(
            err.to_string().contains("restarting"),
            "unexpected error text: {err}"
        );

        let response = restart.await.unwrap().expect("restart should finish");
        assert_eq!(response.status, GatewayStatusV1::Available);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn restart_redelivers_a_fresh_auth_bundle() {
        let _env_lock = ENV_LOCK.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let runtime_root = temp_dir.path().join("runtime-root");
        let _runtime_root_guard = EnvGuard::set("SUBSTRATE_GATEWAY_RUNTIME_ROOT", &runtime_root);
        let (binary, _pid_dir, launch_count_path) = delayed_gateway_binary(&temp_dir, 0);
        let _binary_guard = EnvGuard::set(GATEWAY_BINARY_OVERRIDE_ENV, binary);
        let manager = Arc::new(GatewayRuntimeManager::new(finished_child_exclusion()));

        let first = start_context_with_codex_auth(
            temp_dir.path(),
            "bundle-rotate",
            "token-one",
            Some("acct-one"),
        );
        manager
            .sync_with_timeout(first, Duration::from_secs(3))
            .await
            .expect("initial sync");
        let first_bundle = wait_for_auth_bundle_snapshot(&temp_dir, 1);
        assert_eq!(
            first_bundle
                .fields
                .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
            Some(&"token-one".to_string())
        );

        let second = start_context_with_codex_auth(
            temp_dir.path(),
            "bundle-rotate",
            "token-two",
            Some("acct-two"),
        );
        manager
            .restart(second)
            .await
            .expect("restart should redeliver auth");

        let second_bundle = wait_for_auth_bundle_snapshot(&temp_dir, 2);
        assert_eq!(read_launch_count(&launch_count_path), 2);
        assert_eq!(
            second_bundle
                .fields
                .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
            Some(&"token-two".to_string())
        );
        assert_eq!(
            second_bundle
                .fields
                .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID),
            Some(&"acct-two".to_string())
        );
    }
}

fn attribute(kind: u16, value: &[u8]) -> Vec<u8> {
    let length = value.len() + 4;
    let mut result = Vec::with_capacity((length + 3) & !3);
    result.extend_from_slice(&(length as u16).to_ne_bytes());
    result.extend_from_slice(&kind.to_ne_bytes());
    result.extend_from_slice(value);
    result.resize((length + 3) & !3, 0);
    result
}
fn number(kind: u16, value: u32) -> Vec<u8> {
    attribute(kind, &value.to_be_bytes())
}
fn string(kind: u16, value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    attribute(kind, &bytes)
}
fn attributes(bytes: &[u8]) -> Result<Vec<(u16, &[u8])>> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        anyhow::ensure!(bytes.len() - offset >= 4, "short E3 nftables attribute");
        let length = u16::from_ne_bytes(bytes[offset..offset + 2].try_into()?) as usize;
        let kind = u16::from_ne_bytes(bytes[offset + 2..offset + 4].try_into()?);
        let aligned = (length + 3) & !3;
        anyhow::ensure!(
            length >= 4
                && aligned <= bytes.len() - offset
                && bytes[offset + length..offset + aligned]
                    .iter()
                    .all(|byte| *byte == 0),
            "invalid E3 nftables attribute extent"
        );
        result.push((kind, &bytes[offset + 4..offset + length]));
        offset += aligned;
    }
    Ok(result)
}
fn map(bytes: &[u8]) -> Result<BTreeMap<u16, &[u8]>> {
    let values = attributes(bytes)?;
    let mut result = BTreeMap::new();
    for (kind, value) in values {
        anyhow::ensure!(
            result.insert(kind & 0x3fff, value).is_none(),
            "duplicate E3 nftables attribute"
        );
    }
    Ok(result)
}
fn matches_expression(observed: &[u8], expected: &[u8], depth: usize) -> Result<bool> {
    anyhow::ensure!(
        depth < 8,
        "E3 nftables expression nesting exceeds the fixed schema"
    );
    let mut actual = attributes(observed)?;
    let mut wanted = attributes(expected)?;
    // Kernel dumps deliberately omit NLA_F_NESTED in this ABI. The fixed
    // outgoing schema determines nesting; flags never substitute for values.
    actual.sort_by_key(|(kind, _)| kind & 0x3fff);
    wanted.sort_by_key(|(kind, _)| kind & 0x3fff);
    if actual.len() != wanted.len() {
        return Ok(false);
    }
    for ((actual_kind, actual_value), (wanted_kind, wanted_value)) in actual.into_iter().zip(wanted)
    {
        if actual_kind & 0x3fff != wanted_kind & 0x3fff {
            return Ok(false);
        }
        if wanted_kind & 0x8000 != 0 {
            if !matches_expression(actual_value, wanted_value, depth + 1)? {
                return Ok(false);
            }
        } else if actual_value != wanted_value {
            return Ok(false);
        }
    }
    Ok(true)
}
fn message(kind: u16, flags: u16, sequence: u32, family: u8, payload: &[u8]) -> Vec<u8> {
    let length = 20 + payload.len();
    let mut result = Vec::with_capacity(length);
    result.extend_from_slice(&(length as u32).to_ne_bytes());
    result.extend_from_slice(&kind.to_ne_bytes());
    result.extend_from_slice(&flags.to_ne_bytes());
    result.extend_from_slice(&sequence.to_ne_bytes());
    result.extend_from_slice(&0_u32.to_ne_bytes());
    result.extend_from_slice(&[
        family,
        0,
        0,
        if kind == 0x10 || kind == 0x11 { 10 } else { 0 },
    ]);
    result.extend_from_slice(payload);
    result
}
fn exchange(fd: i32, bytes: &[u8], sequences: &[u32], dump: bool) -> Result<Vec<(u16, Vec<u8>)>> {
    let mut address: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
    address.nl_family = libc::AF_NETLINK as u16;
    let sent = unsafe {
        libc::sendto(
            fd,
            bytes.as_ptr().cast(),
            bytes.len(),
            0,
            (&address as *const libc::sockaddr_nl).cast(),
            std::mem::size_of_val(&address) as libc::socklen_t,
        )
    };
    if sent < 0 {
        return Err(std::io::Error::last_os_error()).context("send E3 nftables transaction");
    }
    anyhow::ensure!(sent as usize == bytes.len(), "short E3 nftables send");
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut awaiting: BTreeSet<u32> = sequences.iter().copied().collect();
    let mut result = Vec::new();
    let mut total = 0;
    while !awaiting.is_empty() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        anyhow::ensure!(!remaining.is_zero(), "E3 nftables readback timed out");
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll, 1, remaining.as_millis().min(2000) as i32) };
        if ready < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        anyhow::ensure!(
            ready == 1 && poll.revents == libc::POLLIN,
            "E3 nftables response unavailable"
        );
        let mut buffer = [0_u8; 65_536];
        let mut from: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        let mut from_length = std::mem::size_of_val(&from) as libc::socklen_t;
        let received = unsafe {
            libc::recvfrom(
                fd,
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                libc::MSG_TRUNC,
                (&mut from as *mut libc::sockaddr_nl).cast(),
                &mut from_length,
            )
        };
        anyhow::ensure!(
            received > 0
                && received as usize <= buffer.len()
                && from_length as usize == std::mem::size_of_val(&from)
                && from.nl_family == libc::AF_NETLINK as u16
                && from.nl_pid == 0
                && from.nl_groups == 0,
            "E3 nftables reply is not an intact kernel response"
        );
        total += received as usize;
        anyhow::ensure!(total <= 1_048_576, "E3 nftables readback exceeds its bound");
        let mut offset = 0;
        while offset < received as usize {
            anyhow::ensure!(received as usize - offset >= 16, "short E3 netlink header");
            let header = &buffer[offset..offset + 16];
            let length = u32::from_ne_bytes(header[..4].try_into()?) as usize;
            let kind = u16::from_ne_bytes(header[4..6].try_into()?);
            let flags = u16::from_ne_bytes(header[6..8].try_into()?);
            let sequence = u32::from_ne_bytes(header[8..12].try_into()?);
            anyhow::ensure!(length >= 16 && (length+3)&!3 <= received as usize-offset && awaiting.contains(&sequence) && flags & 0x10 == 0, "E3 netlink sequence, extent or dump changed: kind={kind:#x} flags={flags:#x} sequence={sequence} length={length} remaining={} awaiting={awaiting:?}", received as usize-offset);
            let payload = &buffer[offset + 16..offset + length];
            if kind == 2 || kind == 3 {
                anyhow::ensure!(payload.len() >= 4, "short E3 netlink completion");
                let error = i32::from_ne_bytes(payload[..4].try_into()?);
                if error != 0 {
                    return Err(std::io::Error::from_raw_os_error(
                        error.checked_neg().context("invalid netlink errno")?,
                    ))
                    .context("E3 nftables kernel rejection");
                }
                anyhow::ensure!(
                    (dump && kind == 3) || (!dump && kind == 2),
                    "unexpected E3 netlink completion kind"
                );
                awaiting.remove(&sequence);
            } else {
                anyhow::ensure!(
                    kind >> 8 == 10 && payload.len() >= 4 && payload[1] == 0,
                    "unexpected E3 nftables response type"
                );
                result.push((kind, payload.to_vec()));
            }
            offset += (length + 3) & !3;
        }
    }
    Ok(result)
}
fn expression(name: &str, data: &[u8]) -> Vec<u8> {
    attribute(0x8001, &[string(1, name), attribute(0x8002, data)].concat())
}
fn compare(value: &[u8]) -> Vec<u8> {
    expression(
        "cmp",
        &[
            number(1, 1),
            number(2, 0),
            attribute(0x8003, &attribute(1, value)),
        ]
        .concat(),
    )
}

#[cfg(test)]
mod e3_e_secret_ready_pipe_tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn e3_e_namespace_parent_lifetime_covers_both_setup_ready_decodes() {
        let source = include_str!("gateway_runtime.rs");
        for (start, end, owner, validation) in [
            (
                "pub(crate) fn spawn_descriptor_pinned(",
                "pub(crate) fn validate_child_security(",
                "owned",
                "self.validate_child_security()?;",
            ),
            (
                "pub(crate) fn spawn_readiness_probe(",
                "pub(crate) fn validate_readiness_probe(",
                "probe",
                "self.validate_readiness_probe()",
            ),
        ] {
            let body = source
                .split_once(start)
                .unwrap()
                .1
                .split_once(end)
                .unwrap()
                .0;
            // Bind the socket-level receiver/ownership regressions to both real parents.
            // A close before the read or decode must fail even without privileged startup.
            let mapped = body
                .find("install_and_validate_e3_child_user_namespace_v1(")
                .unwrap();
            let read = body.find("let bytes = Launch::e3_read_bounded(").unwrap();
            let decode = body
                .find(&format!(
                "{owner}.security = Some(ConfigProjectionCodecV1::decode_canonical_json(&bytes)?);"
            ))
                .unwrap();
            let close = body.find("drop(userns_parent);").unwrap();
            let validate = body.rfind(validation).unwrap();
            assert!(
                mapped < read && read < decode && decode < close && close < validate,
                "{start}: namespace parent must span bounded setup-ready read/decode"
            );
            assert_eq!(body.matches("drop(userns_parent);").count(), 1);
            assert!(!body.contains("userns_parent.into_raw_fd"));
            assert!(!body.contains("forget(userns_parent)"));
        }
    }

    #[test]
    fn e3_e_namespace_parent_closes_after_setup_ready_or_failure() {
        use config_projection::ConfigProjectionCodecV1;
        use std::io::Write;
        type Launch = ManagedGatewayLaunchCapabilityV1;
        for outcome in ["success", "decode_failure", "deadline", "unwind"] {
            let (parent, child) =
                crate::e3_child_security::create_e3_child_user_namespace_channel_v1().unwrap();
            let (reader, mut writer) = create_inherited_auth_bundle_pipe().unwrap();
            let mut deadline = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            if outcome != "deadline" {
                assert_eq!(
                    unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut deadline) },
                    0
                );
                deadline.tv_sec += 2;
            }
            writer
                .write_all(if outcome == "decode_failure" {
                    b"{"
                } else {
                    b"{}"
                })
                .unwrap();
            drop(writer);
            let result = std::panic::catch_unwind(move || -> Result<()> {
                let userns_parent = parent;
                let setup_reader = reader;
                if outcome == "unwind" {
                    panic!("synthetic setup unwind");
                }
                let bytes = Launch::e3_read_bounded(setup_reader.as_raw_fd(), &deadline)?;
                drop(setup_reader);
                let _: serde_json::Value = ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
                drop(userns_parent);
                Ok(())
            });
            match outcome {
                "success" => assert!(result.unwrap().is_ok()),
                "unwind" => assert!(result.is_err()),
                _ => assert!(result.unwrap().is_err()),
            }
            let mut byte = 0u8;
            // Peer EOF proves closure without a raw-fd reuse race with parallel tests.
            assert_eq!(
                unsafe {
                    libc::recv(
                        child.as_raw_fd(),
                        (&mut byte as *mut u8).cast(),
                        1,
                        libc::MSG_DONTWAIT,
                    )
                },
                0,
                "{outcome}: namespace parent leaked"
            );
        }
    }

    #[test]
    fn e3_e_deadline_pipe_protocol() {
        type Launch = ManagedGatewayLaunchCapabilityV1;
        let mut deadline = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        assert_eq!(
            unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut deadline) },
            0
        );
        deadline.tv_sec += 2;
        let (reader, writer) = create_inherited_auth_bundle_pipe().unwrap();
        let data = vec![0x5a; 32768];
        std::thread::scope(|scope| {
            let read =
                scope.spawn(|| Launch::e3_read_bounded(reader.as_raw_fd(), &deadline).unwrap());
            Launch::e3_write_once(writer, &data, &deadline).unwrap();
            assert_eq!(read.join().unwrap(), data);
        });
        let (reader, writer) = create_inherited_auth_bundle_pipe().unwrap();
        let expired = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        assert!(Launch::e3_write_once(writer, b"must-not-arrive", &expired).is_err());
        assert!(Launch::e3_read_bounded(reader.as_raw_fd(), &deadline)
            .unwrap()
            .is_empty());
        let (reader, writer) = create_inherited_auth_bundle_pipe().unwrap();
        drop(reader);
        assert!(Launch::e3_write_once(writer, b"receiver-died", &deadline).is_err());
    }

    #[test]
    fn e3_e_pinned_open_rejects_symlink_and_dot_components() {
        type Launch = ManagedGatewayLaunchCapabilityV1;
        let target = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .canonicalize()
            .unwrap();
        let root = tempfile::tempdir_in(target).unwrap();
        let file = root.path().join("file");
        std::fs::write(&file, b"pinned").unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&file, &alias).unwrap();
        assert!(Launch::e3_open_pinned(alias.to_str().unwrap(), false).is_err());
        assert!(
            Launch::e3_open_pinned(&format!("{}/./file", root.path().display()), false).is_err()
        );
        assert!(
            Launch::e3_open_pinned(&format!("{}/../file", root.path().display()), false).is_err()
        );
        let held = Launch::e3_open_pinned(file.to_str().unwrap(), false).unwrap();
        use std::os::unix::fs::MetadataExt;
        let old_inode = held.metadata().unwrap().ino();
        std::fs::remove_file(&file).unwrap();
        std::fs::write(&file, b"substitution").unwrap();
        assert_eq!(held.metadata().unwrap().ino(), old_inode);
        assert_ne!(
            Launch::e3_open_pinned(file.to_str().unwrap(), false)
                .unwrap()
                .metadata()
                .unwrap()
                .ino(),
            old_inode
        );
    }

    #[test]
    fn e3_e_secret_ready_pipe_is_distinct_cloexec_and_closes_at_eof() {
        let (reader, mut writer) = create_e3_gateway_secret_ready_pipe().unwrap();
        assert_ne!(reader.as_raw_fd(), writer.as_raw_fd());
        for fd in [reader.as_raw_fd(), writer.as_raw_fd()] {
            assert_ne!(
                unsafe { libc::fcntl(fd, libc::F_GETFD) } & libc::FD_CLOEXEC,
                0
            );
        }
        writer
            .write_all(b"synthetic-nonsecret-attestation")
            .unwrap();
        drop(writer);
        let mut bytes = Vec::new();
        std::fs::File::from(reader).read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"synthetic-nonsecret-attestation");
    }
}

// Fixed E3 netfilter wire codecs shared by live installation and recovered revocation.
