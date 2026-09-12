#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use config_projection::{
    CanonicalCgroupIdentityV1, CanonicalDirectoryV1, DirectoryPhysicalIdentityV1,
    E3ChildSecurityAttestationV1, E3DeniedControlProbeTargetBindingV1, E3DeniedControlProbeV1,
    E3LinuxIdMapExtentV1, E3UserNamespaceRequirementV1, E3WorldFsEnforcementInputV1,
};
use serde::Serialize;

const CAP_SETGID: u32 = 6;
const CAP_SETUID: u32 = 7;
const LINUX_CAPABILITY_VERSION_3: u32 = 0x2008_0522;
const NS_GET_PARENT: libc::c_ulong = 0xb702;
const NS_GET_NSTYPE: libc::c_ulong = 0xb703;
const NS_GET_OWNER_UID: libc::c_ulong = 0xb704;
const USERNS_CREATED: u8 = 0x01;
const USERNS_MAPPED: u8 = 0x02;

type ExpectedControlProbeV1 = (&'static str, i32, (u8, Vec<u8>));

#[repr(C)]
#[derive(Clone, Copy)]
struct CapabilityHeader {
    version: u32,
    pid: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CapabilityData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

#[repr(C)]
struct OpenHowV1 {
    flags: u64,
    mode: u64,
    resolve: u64,
}

struct HeldE3ServiceUserNamespaceV1 {
    namespace_fd: OwnedFd,
    device_id: u64,
    inode: u64,
    trusted_service_uid: u64,
}

struct HeldE3ChildUserNamespaceV1 {
    namespace_fd: OwnedFd,
    pidfd: OwnedFd,
    pid: u32,
    pid_start_time_ticks: u64,
    device_id: u64,
    inode: u64,
    cgroup: CanonicalCgroupIdentityV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RecoveredNonE3ChildIdentityV1 {
    pid: u32,
    pid_start_time_ticks: u64,
    cgroup: CanonicalCgroupIdentityV1,
}

impl Ord for RecoveredNonE3ChildIdentityV1 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.pid,
            self.pid_start_time_ticks,
            self.cgroup.cgroup_v2_mount_device_id,
            self.cgroup.cgroup_v2_mount_inode,
            self.cgroup.cgroup_directory_inode,
            self.cgroup.cgroup_relative_path.as_bytes(),
        )
            .cmp(&(
                other.pid,
                other.pid_start_time_ticks,
                other.cgroup.cgroup_v2_mount_device_id,
                other.cgroup.cgroup_v2_mount_inode,
                other.cgroup.cgroup_directory_inode,
                other.cgroup.cgroup_relative_path.as_bytes(),
            ))
    }
}

impl PartialOrd for RecoveredNonE3ChildIdentityV1 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

enum ExclusionModeV1 {
    Recovering {
        recovered_non_e3: BTreeSet<RecoveredNonE3ChildIdentityV1>,
    },
    LegacyShared {
        live_non_e3_children: u64,
    },
    E3Exclusive {
        world_id: String,
        world_generation: u64,
        live_e3_leases: u64,
    },
    Poisoned,
}

struct E3PrivilegedChildExclusionStateV1 {
    mode: ExclusionModeV1,
    child_user_namespaces: BTreeMap<(u32, u64), HeldE3ChildUserNamespaceV1>,
}

pub(crate) struct E3PrivilegedChildExclusionV1 {
    service_user_namespace: HeldE3ServiceUserNamespaceV1,
    state: Mutex<E3PrivilegedChildExclusionStateV1>,
}

pub(crate) struct HeldNonE3PrivilegedChildLeaseV1 {
    exclusion: Arc<E3PrivilegedChildExclusionV1>,
    recovered_identity: Option<RecoveredNonE3ChildIdentityV1>,
    released: bool,
}

pub(crate) struct HeldE3PrivilegedChildExclusionLeaseV1 {
    exclusion: Arc<E3PrivilegedChildExclusionV1>,
    world_id: String,
    world_generation: u64,
    released: bool,
}

impl std::fmt::Debug for HeldNonE3PrivilegedChildLeaseV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HeldNonE3PrivilegedChildLeaseV1")
            .field("recovered", &self.recovered_identity.is_some())
            .field("released", &self.released)
            .finish_non_exhaustive()
    }
}

impl E3PrivilegedChildExclusionV1 {
    pub(crate) fn new_recovering() -> Result<Arc<Self>> {
        let service_user_namespace = hold_service_user_namespace()?;
        Ok(Arc::new(Self {
            service_user_namespace,
            state: Mutex::new(E3PrivilegedChildExclusionStateV1 {
                mode: ExclusionModeV1::Recovering {
                    recovered_non_e3: BTreeSet::new(),
                },
                child_user_namespaces: BTreeMap::new(),
            }),
        }))
    }

    pub(crate) fn register_recovered_non_e3_child(
        self: &Arc<Self>,
        pid: u32,
        pid_start_time_ticks: u64,
        cgroup: CanonicalCgroupIdentityV1,
    ) -> Result<HeldNonE3PrivilegedChildLeaseV1> {
        if pid == 0 || pid_start_time_ticks == 0 {
            bail!("invalid recovered non-E3 process identity");
        }
        let identity = RecoveredNonE3ChildIdentityV1 {
            pid,
            pid_start_time_ticks,
            cgroup,
        };
        let mut state = self.lock_state()?;
        let ExclusionModeV1::Recovering { recovered_non_e3 } = &mut state.mode else {
            bail!("recovered child registration is permitted only during recovery");
        };
        if !recovered_non_e3.insert(identity.clone()) {
            bail!("duplicate recovered non-E3 process identity");
        }
        drop(state);
        Ok(HeldNonE3PrivilegedChildLeaseV1 {
            exclusion: Arc::clone(self),
            recovered_identity: Some(identity),
            released: false,
        })
    }

    pub(crate) fn finish_recovery(&self) -> Result<()> {
        let mut state = self.lock_state()?;
        let ExclusionModeV1::Recovering { recovered_non_e3 } = &mut state.mode else {
            bail!("E3-D recovery is not active");
        };
        verify_no_unclassified_non_e3_descendants_or_mounts(recovered_non_e3)?;
        let live_non_e3_children = u64::try_from(recovered_non_e3.len())
            .context("recovered non-E3 child count does not fit u64")?;
        state.mode = ExclusionModeV1::LegacyShared {
            live_non_e3_children,
        };
        Ok(())
    }

    pub(crate) fn acquire_non_e3_child(
        self: &Arc<Self>,
    ) -> Result<HeldNonE3PrivilegedChildLeaseV1> {
        let mut state = self.lock_state()?;
        let ExclusionModeV1::LegacyShared {
            live_non_e3_children,
            ..
        } = &mut state.mode
        else {
            bail!("UnsupportedSecurityPosture: non-E3 child admission is closed");
        };
        *live_non_e3_children = live_non_e3_children
            .checked_add(1)
            .context("non-E3 child lease count overflow")?;
        drop(state);
        Ok(HeldNonE3PrivilegedChildLeaseV1 {
            exclusion: Arc::clone(self),
            recovered_identity: None,
            released: false,
        })
    }

    pub(crate) fn acquire_e3_exclusive(
        self: &Arc<Self>,
        world_id: &str,
        world_generation: u64,
    ) -> Result<HeldE3PrivilegedChildExclusionLeaseV1> {
        if world_id.is_empty() || world_generation == 0 {
            bail!("invalid E3 world binding");
        }
        let mut state = self.lock_state()?;
        match &mut state.mode {
            ExclusionModeV1::LegacyShared {
                live_non_e3_children,
            } if *live_non_e3_children == 0 => {
                verify_no_unclassified_non_e3_descendants_or_mounts(&BTreeSet::new())?;
                state.mode = ExclusionModeV1::E3Exclusive {
                    world_id: world_id.to_string(),
                    world_generation,
                    live_e3_leases: 1,
                };
            }
            ExclusionModeV1::E3Exclusive {
                world_id: active_world,
                world_generation: active_generation,
                live_e3_leases,
            } if active_world == world_id && *active_generation == world_generation => {
                *live_e3_leases = live_e3_leases
                    .checked_add(1)
                    .context("E3 child lease count overflow")?;
            }
            _ => bail!("UnsupportedSecurityPosture: E3-exclusive admission is closed"),
        }
        drop(state);
        Ok(HeldE3PrivilegedChildExclusionLeaseV1 {
            exclusion: Arc::clone(self),
            world_id: world_id.to_string(),
            world_generation,
            released: false,
        })
    }

    fn release_non_e3_child(
        &self,
        recovered_identity: Option<&RecoveredNonE3ChildIdentityV1>,
    ) -> Result<()> {
        let mut state = self.lock_state()?;
        match &mut state.mode {
            ExclusionModeV1::Recovering { recovered_non_e3 } => {
                let identity = recovered_identity
                    .context("ordinary non-E3 lease cannot be released during recovery")?;
                if !recovered_non_e3.remove(identity) {
                    bail!("unknown recovered non-E3 process identity");
                }
            }
            ExclusionModeV1::LegacyShared {
                live_non_e3_children,
            } => {
                *live_non_e3_children = live_non_e3_children
                    .checked_sub(1)
                    .context("non-E3 child lease count underflow")?;
            }
            _ => bail!("non-E3 child release outside recovery/legacy-shared mode"),
        }
        Ok(())
    }

    pub(crate) fn release_e3_exclusive(&self, world_id: &str, world_generation: u64) -> Result<()> {
        let mut state = self.lock_state()?;
        let ExclusionModeV1::E3Exclusive {
            world_id: active_world,
            world_generation: active_generation,
            live_e3_leases,
        } = &mut state.mode
        else {
            bail!("E3 child release outside exclusive mode");
        };
        if active_world != world_id || *active_generation != world_generation {
            bail!("E3 child release has the wrong world binding");
        }
        *live_e3_leases = live_e3_leases
            .checked_sub(1)
            .context("E3 child lease count underflow")?;
        if *live_e3_leases == 0 {
            for namespace in state.child_user_namespaces.values() {
                verify_e3_child_process_and_cgroup_quiescent(namespace)?;
            }
            state.child_user_namespaces.clear();
            state.mode = ExclusionModeV1::LegacyShared {
                live_non_e3_children: 0,
            };
        }
        Ok(())
    }

    pub(crate) fn poison_recovering(&self) -> Result<()> {
        let mut state = self.lock_state()?;
        if !matches!(state.mode, ExclusionModeV1::Recovering { .. }) {
            bail!("recovery poisoning is valid only while recovering");
        }
        state.mode = ExclusionModeV1::Poisoned;
        Ok(())
    }

    fn retain_child_user_namespace(&self, namespace: HeldE3ChildUserNamespaceV1) -> Result<()> {
        let mut state = self.lock_state()?;
        if !matches!(state.mode, ExclusionModeV1::E3Exclusive { .. }) {
            bail!("child user namespace cannot be retained outside an E3 epoch");
        }
        let identity = (namespace.pid, namespace.pid_start_time_ticks);
        if state
            .child_user_namespaces
            .insert(identity, namespace)
            .is_some()
        {
            bail!("duplicate E3 child process identity");
        }
        Ok(())
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, E3PrivilegedChildExclusionStateV1>> {
        self.state
            .lock()
            .map_err(|_| anyhow::anyhow!("E3-D child-exclusion state is poisoned"))
    }
}

fn verify_no_unclassified_non_e3_descendants_or_mounts(
    recovered_non_e3: &BTreeSet<RecoveredNonE3ChildIdentityV1>,
) -> Result<()> {
    #[cfg(test)]
    {
        let _ = recovered_non_e3;
        Ok(())
    }

    #[cfg(not(test))]
    {
        let service_pid = std::process::id();
        let mut parents = BTreeMap::new();
        for entry in std::fs::read_dir("/proc").context("scan process table for E3-D exclusion")? {
            let entry = entry.context("read process-table entry for E3-D exclusion")?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Ok(pid) = name.parse::<u32>() else {
                continue;
            };
            let status = match std::fs::read_to_string(entry.path().join("status")) {
                Ok(status) => status,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error).context("read process parent identity"),
            };
            let parent = status
                .lines()
                .find_map(|line| line.strip_prefix("PPid:\t"))
                .and_then(|value| value.trim().parse::<u32>().ok())
                .context("parse process parent identity")?;
            parents.insert(pid, parent);
        }
        for pid in parents.keys().copied() {
            let mut cursor = pid;
            let mut visited = BTreeSet::new();
            while let Some(parent) = parents.get(&cursor).copied() {
                if !visited.insert(cursor) || parent == 0 {
                    break;
                }
                if parent == service_pid {
                    let recovered = recovered_non_e3
                        .iter()
                        .find(|identity| identity.pid == pid)
                        .context(format!(
                            "UnsupportedSecurityPosture: unclassified live non-E3 descendant {pid}"
                        ))?;
                    if read_process_start_time(pid)? != recovered.pid_start_time_ticks
                        || read_process_cgroup_relative_path(pid)?
                            != recovered.cgroup.cgroup_relative_path
                    {
                        bail!(
                            "UnsupportedSecurityPosture: recovered non-E3 descendant identity drifted"
                        );
                    }
                    break;
                }
                cursor = parent;
            }
        }

        verify_service_owned_cgroup_forest(recovered_non_e3)?;

        let mountinfo = std::fs::read_to_string("/proc/self/mountinfo")
            .context("read service mount table for E3-D exclusion")?;
        if mountinfo.lines().any(|line| {
            line.contains(" - fuse-overlayfs ")
                && (line.contains("/run/substrate")
                    || line.contains("substrate-world")
                    || line.contains("/var/lib/substrate"))
        }) {
            bail!("UnsupportedSecurityPosture: unclassified persistent non-E3 FUSE mount");
        }
        Ok(())
    }
}

#[cfg(not(test))]
fn read_process_cgroup_relative_path(pid: u32) -> Result<String> {
    let membership = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .context("read E3-D process cgroup membership")?;
    let relative = membership
        .strip_prefix("0::/")
        .and_then(|value| value.strip_suffix('\n'))
        .context("process has noncanonical cgroup v2 membership")?;
    if relative.is_empty()
        || relative
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        bail!("process cgroup path is not canonical");
    }
    Ok(relative.to_string())
}

#[cfg(not(test))]
fn verify_service_owned_cgroup_forest(
    recovered_non_e3: &BTreeSet<RecoveredNonE3ChildIdentityV1>,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;

    fn scan(
        directory: &Path,
        relative: &str,
        mount_device_id: u64,
        mount_inode: u64,
        recovered_non_e3: &BTreeSet<RecoveredNonE3ChildIdentityV1>,
    ) -> Result<()> {
        let procs = std::fs::read_to_string(directory.join("cgroup.procs"))
            .with_context(|| format!("read service-owned {relative}/cgroup.procs"))?;
        for line in procs.lines() {
            let pid = line
                .parse::<u32>()
                .context("parse service-owned cgroup process identity")?;
            let identity = recovered_non_e3
                .iter()
                .find(|identity| identity.pid == pid)
                .context(format!(
                    "UnsupportedSecurityPosture: unclassified process {pid} in service-owned cgroup {relative}"
                ))?;
            let metadata = std::fs::metadata(directory)
                .with_context(|| format!("stat service-owned cgroup {relative}"))?;
            if identity.pid_start_time_ticks != read_process_start_time(pid)?
                || identity.cgroup.cgroup_v2_mount_device_id != mount_device_id
                || identity.cgroup.cgroup_v2_mount_inode != mount_inode
                || identity.cgroup.cgroup_directory_inode != metadata.ino()
                || identity.cgroup.cgroup_relative_path != relative
                || metadata.dev() != mount_device_id
            {
                bail!("UnsupportedSecurityPosture: recovered non-E3 cgroup identity drifted");
            }
        }
        for entry in std::fs::read_dir(directory)
            .with_context(|| format!("enumerate service-owned cgroup {relative}"))?
        {
            let entry = entry.context("read service-owned cgroup entry")?;
            if entry
                .file_type()
                .context("inspect service-owned cgroup entry")?
                .is_dir()
            {
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("non-UTF-8 service-owned cgroup name"))?;
                if name.is_empty() || name == "." || name == ".." || name.contains('/') {
                    bail!("noncanonical service-owned cgroup name");
                }
                scan(
                    &entry.path(),
                    &format!("{relative}/{name}"),
                    mount_device_id,
                    mount_inode,
                    recovered_non_e3,
                )?;
            }
        }
        Ok(())
    }

    let root = Path::new("/sys/fs/cgroup/substrate");
    let root_metadata = match std::fs::metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("stat service-owned cgroup root"),
    };
    if !root_metadata.is_dir() {
        bail!("service-owned cgroup root is not a directory");
    }
    let mount = std::fs::metadata("/sys/fs/cgroup").context("stat cgroup v2 mount")?;
    scan(
        root,
        "substrate",
        mount.dev(),
        mount.ino(),
        recovered_non_e3,
    )
}

impl Drop for HeldNonE3PrivilegedChildLeaseV1 {
    fn drop(&mut self) {
        if !self.released {
            if self
                .exclusion
                .release_non_e3_child(self.recovered_identity.as_ref())
                .is_err()
            {
                std::process::abort();
            }
            self.released = true;
        }
    }
}

impl Drop for HeldE3PrivilegedChildExclusionLeaseV1 {
    fn drop(&mut self) {
        if !self.released {
            if self
                .exclusion
                .release_e3_exclusive(&self.world_id, self.world_generation)
                .is_err()
            {
                std::process::abort();
            }
            self.released = true;
        }
    }
}

pub(crate) fn bind_e3_service_user_namespace_v1(
    exclusion: &E3PrivilegedChildExclusionV1,
    target_uid: u64,
    target_gid: u64,
) -> Result<E3UserNamespaceRequirementV1> {
    if target_uid == 0 || target_gid == 0 {
        bail!("E3-D child identity must be nonroot");
    }
    Ok(E3UserNamespaceRequirementV1 {
        trusted_service_uid: exclusion.service_user_namespace.trusted_service_uid,
        parent_namespace_device_id: exclusion.service_user_namespace.device_id,
        parent_namespace_inode: exclusion.service_user_namespace.inode,
        uid_map: E3LinuxIdMapExtentV1 {
            inside_id: target_uid,
            outside_id: target_uid,
            length: 1,
        },
        gid_map: E3LinuxIdMapExtentV1 {
            inside_id: target_gid,
            outside_id: target_gid,
            length: 1,
        },
    })
}

pub(crate) fn build_authenticated_world_fs_enforcement_input_v1(
    exclusion: &E3PrivilegedChildExclusionV1,
    input: &E3WorldFsEnforcementInputV1,
) -> Result<E3WorldFsEnforcementInputV1> {
    let requirement =
        bind_e3_service_user_namespace_v1(exclusion, input.target_uid, input.target_gid)?;
    if input.schema_version != 1
        || input.support_policy_version != 1
        || input.user_namespace_requirement != requirement
        || input.policy_snapshot_byte_length == 0
        || input.policy_snapshot_hash.len() != 64
        || input.enforcement_input_hash.len() != 64
    {
        bail!("invalid authenticated E3-D filesystem enforcement input");
    }
    Ok(input.clone())
}

pub(crate) fn create_e3_child_user_namespace_channel_v1() -> Result<(OwnedFd, OwnedFd)> {
    let mut sockets = [-1; 2];
    let rc = unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            sockets.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("create E3-D userns channel");
    }
    Ok(unsafe {
        (
            OwnedFd::from_raw_fd(sockets[0]),
            OwnedFd::from_raw_fd(sockets[1]),
        )
    })
}

pub(crate) fn install_and_validate_e3_child_user_namespace_v1(
    exclusion: &E3PrivilegedChildExclusionV1,
    parent_setup_socket: &OwnedFd,
    child_pid: u32,
    child_pid_start_time_ticks: u64,
    requirement: &E3UserNamespaceRequirementV1,
    child_cgroup: &CanonicalCgroupIdentityV1,
) -> Result<()> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                install_and_validate_e3_child_user_namespace_on_sync_thread_v1(
                    exclusion,
                    parent_setup_socket,
                    child_pid,
                    child_pid_start_time_ticks,
                    requirement,
                    child_cgroup,
                )
            })
            .join()
            .map_err(|_| anyhow::anyhow!("dedicated E3-D namespace setup thread panicked"))?
    })
}

fn install_and_validate_e3_child_user_namespace_on_sync_thread_v1(
    exclusion: &E3PrivilegedChildExclusionV1,
    parent_setup_socket: &OwnedFd,
    child_pid: u32,
    child_pid_start_time_ticks: u64,
    requirement: &E3UserNamespaceRequirementV1,
    child_cgroup: &CanonicalCgroupIdentityV1,
) -> Result<()> {
    if requirement.trusted_service_uid != exclusion.service_user_namespace.trusted_service_uid
        || requirement.parent_namespace_device_id != exclusion.service_user_namespace.device_id
        || requirement.parent_namespace_inode != exclusion.service_user_namespace.inode
        || requirement.uid_map.inside_id == 0
        || requirement.uid_map.inside_id != requirement.uid_map.outside_id
        || requirement.uid_map.length != 1
        || requirement.gid_map.inside_id == 0
        || requirement.gid_map.inside_id != requirement.gid_map.outside_id
        || requirement.gid_map.length != 1
    {
        bail!("wrong E3-D user namespace requirement");
    }
    receive_exact_setup_byte(parent_setup_socket.as_raw_fd(), USERNS_CREATED)?;
    if read_process_start_time(child_pid)? != child_pid_start_time_ticks {
        bail!("substituted E3-D child process identity");
    }
    validate_child_cgroup_membership(child_pid, child_cgroup)?;
    let pidfd = pidfd_open(child_pid)?;
    let namespace_fd = open_child_user_namespace(child_pid, child_pid_start_time_ticks)?;
    let (device_id, inode) = descriptor_identity(namespace_fd.as_raw_fd())?;
    validate_child_namespace_descriptor(
        &namespace_fd,
        &exclusion.service_user_namespace,
        requirement.trusted_service_uid,
    )?;
    install_maps_with_bounded_effective_capabilities(child_pid, requirement)?;
    validate_map_file(child_pid, "uid_map", &requirement.uid_map)?;
    validate_map_file(child_pid, "gid_map", &requirement.gid_map)?;
    validate_child_namespace_descriptor(
        &namespace_fd,
        &exclusion.service_user_namespace,
        requirement.trusted_service_uid,
    )?;
    exclusion.retain_child_user_namespace(HeldE3ChildUserNamespaceV1 {
        namespace_fd,
        pidfd,
        pid: child_pid,
        pid_start_time_ticks: child_pid_start_time_ticks,
        device_id,
        inode,
        cgroup: child_cgroup.clone(),
    })?;
    send_exact_setup_byte(parent_setup_socket.as_raw_fd(), USERNS_MAPPED)
}

pub(crate) fn validate_child_security_attestation_v1(
    exclusion: &E3PrivilegedChildExclusionV1,
    input: &E3WorldFsEnforcementInputV1,
    attestation: &E3ChildSecurityAttestationV1,
) -> Result<()> {
    let requirement = &input.user_namespace_requirement;
    let namespace = &attestation.user_namespace;
    let expected_attestation_hash = canonical_hash_omitting_v1(
        "substrate.e3.child-security-attestation.v1",
        "attestation",
        attestation,
        "attestation_hash",
    )?;
    let expected_probes = input
        .denied_control_probe_targets
        .iter()
        .map(|target| {
            let (operation, result_errno, _) = expected_control_probe_v1(target)?;
            Ok(E3DeniedControlProbeV1 {
                target: target.clone(),
                operation: operation.to_string(),
                result_errno,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let expected_probe_hash = probe_child_control_path_denials_v1(&expected_probes)?;
    let observed_landlock_abi = world::landlock::detect_support().abi.unwrap_or(0);
    if attestation.schema_version != 1
        || attestation.child_role != input.child_role
        || attestation.projection_identity_hash != input.projection_identity_hash
        || attestation.real_uid != input.target_uid
        || attestation.effective_uid != input.target_uid
        || attestation.saved_uid != input.target_uid
        || attestation.filesystem_uid != input.target_uid
        || attestation.real_gid != input.target_gid
        || attestation.effective_gid != input.target_gid
        || attestation.saved_gid != input.target_gid
        || attestation.filesystem_gid != input.target_gid
        || attestation.supplementary_group_count != 0
        || [
            &attestation.cap_inheritable,
            &attestation.cap_permitted,
            &attestation.cap_effective,
            &attestation.cap_bounding,
            &attestation.cap_ambient,
        ]
        .into_iter()
        .any(|mask| mask.as_str() != "0000000000000000")
        || attestation.cap_last_cap > 63
        || !attestation.no_new_privs
        || attestation.dumpable != 0
        || !role_has_exact_tracer_v1(input.child_role, attestation.tracer_pid)
        || attestation.seccomp_mode != 2
        || attestation.landlock_abi == 0
        || attestation.landlock_abi != observed_landlock_abi
        || !is_lower_hex_sha256_v1(&attestation.e2_enforcement_plan_hash)
        || !is_lower_hex_sha256_v1(&attestation.derived_support_ruleset_hash)
        || !is_lower_hex_sha256_v1(&attestation.role_narrowing_ruleset_hash)
        || !is_lower_hex_sha256_v1(&attestation.effective_landlock_hash)
        || attestation.kernel_boot_id != input.kernel_boot_id
        || attestation.policy_snapshot_ref != input.policy_snapshot_ref
        || attestation.policy_snapshot_hash != input.policy_snapshot_hash
        || attestation.policy_snapshot_revision != input.policy_snapshot_revision
        || attestation.enforcement_input_hash != input.enforcement_input_hash
        || attestation.denied_control_probe_hash != expected_probe_hash
        || attestation.attestation_hash != expected_attestation_hash
        || namespace.owner_uid != requirement.trusted_service_uid
        || namespace.parent_namespace_device_id != requirement.parent_namespace_device_id
        || namespace.parent_namespace_inode != requirement.parent_namespace_inode
        || namespace.uid_map != requirement.uid_map
        || namespace.gid_map != requirement.gid_map
        || (namespace.namespace_device_id == requirement.parent_namespace_device_id
            && namespace.namespace_inode == requirement.parent_namespace_inode)
    {
        bail!("invalid E3-D child security attestation");
    }
    let state = exclusion.lock_state()?;
    let held = state
        .child_user_namespaces
        .get(&(attestation.pid, attestation.pid_start_time_ticks))
        .context("E3-D child attestation lacks a retained user namespace")?;
    let mut pidfd_poll = libc::pollfd {
        fd: held.pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    if held.cgroup != input.expected_process_cgroup
        || held.device_id != namespace.namespace_device_id
        || held.inode != namespace.namespace_inode
        || descriptor_identity(held.namespace_fd.as_raw_fd())?
            != (namespace.namespace_device_id, namespace.namespace_inode)
        || read_process_start_time(held.pid)? != held.pid_start_time_ticks
        || unsafe { libc::fcntl(held.pidfd.as_raw_fd(), libc::F_GETFD) } < 0
        || unsafe { libc::poll(&mut pidfd_poll, 1, 0) } != 0
    {
        bail!("E3-D retained child namespace/process identity drifted");
    }
    validate_child_namespace_descriptor(
        &held.namespace_fd,
        &exclusion.service_user_namespace,
        requirement.trusted_service_uid,
    )?;
    let process_fd = open_child_process_directory(held.pid, held.pid_start_time_ticks)?;
    validate_live_child_security_readback_v1(
        process_fd.as_raw_fd(),
        input,
        attestation,
        &held.cgroup,
    )?;
    validate_denied_control_probe_target_identities_v1(
        process_fd.as_raw_fd(),
        input,
        attestation.pid,
    )?;
    let live_namespace = open_child_user_namespace(held.pid, held.pid_start_time_ticks)?;
    if descriptor_identity(live_namespace.as_raw_fd())?
        != (namespace.namespace_device_id, namespace.namespace_inode)
    {
        bail!("E3-D child is no longer a member of its retained user namespace");
    }
    pidfd_poll.revents = 0;
    if unsafe { libc::poll(&mut pidfd_poll, 1, 0) } != 0 {
        bail!("E3-D child terminated during security readback");
    }
    Ok(())
}

fn role_has_exact_tracer_v1(
    role: config_projection::E3IsolatedChildRoleV1,
    tracer_pid: u32,
) -> bool {
    match role {
        config_projection::E3IsolatedChildRoleV1::Codex => tracer_pid == std::process::id(),
        config_projection::E3IsolatedChildRoleV1::ManagedGateway
        | config_projection::E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe => tracer_pid == 0,
    }
}

pub(crate) fn probe_child_control_path_denials_v1(
    probes: &[E3DeniedControlProbeV1],
) -> Result<String> {
    let mut prior_key: Option<(u8, Vec<u8>)> = None;
    let mut observed_hashes = BTreeSet::new();
    for probe in probes {
        let expected_target_hash = canonical_hash_omitting_v1(
            "substrate.e3.denied-control-probe-target.v1",
            "target",
            &probe.target,
            "target_hash",
        )?;
        let (expected_operation, expected_errno, order_key) =
            expected_control_probe_v1(&probe.target)?;
        if probe.target.target_hash != expected_target_hash
            || !observed_hashes.insert(probe.target.target_hash.clone())
            || prior_key.as_ref().is_some_and(|prior| prior >= &order_key)
            || probe.operation != expected_operation
            || probe.result_errno != expected_errno
        {
            bail!("E3-D control-path denial evidence is malformed");
        }
        prior_key = Some(order_key);
    }
    config_projection::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.denied-control-probes.v1",
        &serde_json::json!({ "probes": probes }),
    )
    .map_err(Into::into)
}

fn expected_control_probe_v1(
    target: &config_projection::E3DeniedControlProbeTargetV1,
) -> Result<ExpectedControlProbeV1> {
    let result = match &target.binding {
        E3DeniedControlProbeTargetBindingV1::AcceptedHomeRegistry { directory } => (
            "open_read_directory",
            libc::EACCES,
            (0, directory.physical_path.as_bytes().to_vec()),
        ),
        E3DeniedControlProbeTargetBindingV1::SiblingNativeRealization { directory } => (
            "open_read_directory",
            libc::EACCES,
            (1, directory.physical_path.as_bytes().to_vec()),
        ),
        E3DeniedControlProbeTargetBindingV1::CgroupControl {
            cgroup,
            control_file,
        } => {
            if control_file != "cgroup.procs" {
                bail!("E3-D cgroup control probe has the wrong operation target");
            }
            let mut key = cgroup.cgroup_relative_path.as_bytes().to_vec();
            key.push(0);
            key.extend_from_slice(control_file.as_bytes());
            ("open_write_cgroup_procs", libc::EACCES, (2, key))
        }
        E3DeniedControlProbeTargetBindingV1::NftablesControl {
            network_namespace_inode,
        } => {
            if *network_namespace_inode == 0 {
                bail!("E3-D nftables control probe lacks a namespace identity");
            }
            (
                "send_noop_nft_batch",
                libc::EPERM,
                (3, network_namespace_inode.to_be_bytes().to_vec()),
            )
        }
        E3DeniedControlProbeTargetBindingV1::WorldServiceState { directory } => (
            "open_read_directory",
            libc::EACCES,
            (4, directory.physical_path.as_bytes().to_vec()),
        ),
        E3DeniedControlProbeTargetBindingV1::OtherRolePrivateRoot { directory } => (
            "open_read_directory",
            libc::EACCES,
            (5, directory.physical_path.as_bytes().to_vec()),
        ),
        E3DeniedControlProbeTargetBindingV1::OtherRoleProcessState {
            pid,
            pid_start_time_ticks,
            procfs_mount_device_id,
            procfs_mount_inode,
        } => {
            if *pid == 0
                || *pid_start_time_ticks == 0
                || *procfs_mount_device_id == 0
                || *procfs_mount_inode == 0
            {
                bail!("E3-D other-role process probe lacks an exact identity");
            }
            let mut key = pid.to_be_bytes().to_vec();
            key.extend_from_slice(&pid_start_time_ticks.to_be_bytes());
            ("open_read_proc_status", libc::EACCES, (6, key))
        }
    };
    Ok(result)
}

fn canonical_hash_omitting_v1<T: Serialize>(
    domain: &str,
    key: &str,
    value: &T,
    omitted: &str,
) -> Result<String> {
    let mut value = serde_json::to_value(value).context("encode E3-D hash input")?;
    value
        .as_object_mut()
        .context("E3-D hash input must be an object")?
        .remove(omitted)
        .context("E3-D hash input lacks its omitted field")?;
    let mut preimage = serde_json::Map::new();
    preimage.insert(
        "domain".to_string(),
        serde_json::Value::String(domain.to_string()),
    );
    preimage.insert(key.to_string(), value);
    config_projection::ConfigProjectionCodecV1::domain_sha256(
        "",
        &serde_json::Value::Object(preimage),
    )
    .map_err(Into::into)
}

fn is_lower_hex_sha256_v1(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hold_service_user_namespace() -> Result<HeldE3ServiceUserNamespaceV1> {
    let namespace_fd: OwnedFd = File::open("/proc/self/ns/user")
        .context("open service user namespace")?
        .into();
    let namespace_type = unsafe { libc::ioctl(namespace_fd.as_raw_fd(), NS_GET_NSTYPE as _) };
    if namespace_type != libc::CLONE_NEWUSER {
        bail!("service user namespace descriptor has the wrong namespace type");
    }
    let (device_id, inode) = descriptor_identity(namespace_fd.as_raw_fd())?;
    Ok(HeldE3ServiceUserNamespaceV1 {
        namespace_fd,
        device_id,
        inode,
        trusted_service_uid: unsafe { libc::geteuid() } as u64,
    })
}

fn open_child_user_namespace(pid: u32, expected_start_time_ticks: u64) -> Result<OwnedFd> {
    let process_fd = open_child_process_directory(pid, expected_start_time_ticks)?;
    let namespace = std::ffi::CString::new("ns/user").unwrap();
    let namespace_fd = unsafe {
        libc::openat(
            process_fd.as_raw_fd(),
            namespace.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC,
        )
    };
    if namespace_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open child user namespace");
    }
    let namespace_fd = unsafe { OwnedFd::from_raw_fd(namespace_fd) };
    if read_process_start_time_at(process_fd.as_raw_fd())? != expected_start_time_ticks {
        bail!("E3-D child process changed while opening its namespace");
    }
    Ok(namespace_fd)
}

fn open_child_process_directory(pid: u32, expected_start_time_ticks: u64) -> Result<OwnedFd> {
    let proc_root = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open("/proc")
        .context("open trusted procfs root for child namespace")?;
    let mut procfs: libc::statfs = unsafe { zeroed() };
    if unsafe { libc::fstatfs(proc_root.as_raw_fd(), &mut procfs) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("stat trusted procfs root for child namespace");
    }
    if procfs.f_type as i128 != libc::PROC_SUPER_MAGIC as i128 {
        bail!("trusted child process root is not procfs");
    }
    let component = std::ffi::CString::new(pid.to_string()).unwrap();
    let how = OpenHowV1 {
        flags: (libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
        mode: 0,
        resolve: 0x01 | 0x02 | 0x04 | 0x08,
    };
    let process_fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            proc_root.as_raw_fd(),
            component.as_ptr(),
            &how,
            std::mem::size_of::<OpenHowV1>(),
        )
    } as RawFd;
    if process_fd < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open numeric child proc directory without links");
    }
    let process_fd = unsafe { OwnedFd::from_raw_fd(process_fd) };
    if read_process_start_time_at(process_fd.as_raw_fd())? != expected_start_time_ticks {
        bail!("substituted E3-D child process directory");
    }
    Ok(process_fd)
}

fn validate_denied_control_probe_target_identities_v1(
    process_fd: RawFd,
    input: &E3WorldFsEnforcementInputV1,
    child_pid: u32,
) -> Result<()> {
    for target in &input.denied_control_probe_targets {
        match &target.binding {
            E3DeniedControlProbeTargetBindingV1::AcceptedHomeRegistry { directory }
            | E3DeniedControlProbeTargetBindingV1::SiblingNativeRealization { directory }
            | E3DeniedControlProbeTargetBindingV1::WorldServiceState { directory }
            | E3DeniedControlProbeTargetBindingV1::OtherRolePrivateRoot { directory } => {
                let _held = open_and_validate_canonical_directory_v1(directory)?;
            }
            E3DeniedControlProbeTargetBindingV1::CgroupControl {
                cgroup,
                control_file,
            } => {
                if control_file != "cgroup.procs" || cgroup != &input.expected_process_cgroup {
                    bail!("E3-D cgroup denial target is not the held child cgroup");
                }
                validate_child_cgroup_membership(child_pid, cgroup)?;
            }
            E3DeniedControlProbeTargetBindingV1::NftablesControl {
                network_namespace_inode,
            } => {
                let namespace = open_namespace_at_v1(process_fd, "ns/net")?;
                if unsafe { libc::ioctl(namespace.as_raw_fd(), NS_GET_NSTYPE as _) }
                    != libc::CLONE_NEWNET
                    || descriptor_identity(namespace.as_raw_fd())?.1 != *network_namespace_inode
                {
                    bail!("E3-D nftables denial target namespace identity drifted");
                }
            }
            E3DeniedControlProbeTargetBindingV1::OtherRoleProcessState {
                pid,
                pid_start_time_ticks,
                procfs_mount_device_id,
                procfs_mount_inode,
            } => {
                if *pid == child_pid {
                    bail!("E3-D other-role process target substituted the current child");
                }
                let proc_root = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                    .open("/proc")
                    .context("open procfs root for E3-D other-role target")?;
                let mut procfs: libc::statfs = unsafe { zeroed() };
                if unsafe { libc::fstatfs(proc_root.as_raw_fd(), &mut procfs) } != 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("stat procfs root for E3-D other-role target");
                }
                if procfs.f_type as i128 != libc::PROC_SUPER_MAGIC as i128
                    || descriptor_identity(proc_root.as_raw_fd())?
                        != (*procfs_mount_device_id, *procfs_mount_inode)
                {
                    bail!("E3-D other-role target procfs identity drifted");
                }
                let other = open_child_process_directory(*pid, *pid_start_time_ticks)?;
                if read_process_start_time_at(other.as_raw_fd())? != *pid_start_time_ticks {
                    bail!("E3-D other-role process target identity drifted");
                }
            }
        }
    }
    Ok(())
}

fn open_and_validate_canonical_directory_v1(directory: &CanonicalDirectoryV1) -> Result<OwnedFd> {
    let path = std::path::Path::new(&directory.physical_path);
    if !path.is_absolute()
        || directory.physical_path == "/"
        || directory.physical_path.ends_with('/')
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        bail!("E3-D denied-control directory path is not canonical");
    }
    let root = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open("/")
        .context("open root for E3-D denied-control identity")?;
    let relative = std::ffi::CString::new(directory.physical_path.trim_start_matches('/'))?;
    let how = OpenHowV1 {
        flags: (libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW) as u64,
        mode: 0,
        resolve: 0x02 | 0x04 | 0x08,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root.as_raw_fd(),
            relative.as_ptr(),
            &how,
            std::mem::size_of::<OpenHowV1>(),
        )
    } as RawFd;
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open exact E3-D denied-control directory without links");
    }
    let held = unsafe { OwnedFd::from_raw_fd(fd) };
    let DirectoryPhysicalIdentityV1::Linux { device_id, inode } = directory.physical_identity;
    if descriptor_identity(held.as_raw_fd())? != (device_id, inode) {
        bail!("E3-D denied-control directory identity drifted");
    }
    Ok(held)
}

fn open_namespace_at_v1(process_fd: RawFd, name: &str) -> Result<OwnedFd> {
    let name = std::ffi::CString::new(name)?;
    let fd = unsafe { libc::openat(process_fd, name.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open E3-D child namespace through retained proc descriptor");
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn validate_live_child_security_readback_v1(
    process_fd: RawFd,
    input: &E3WorldFsEnforcementInputV1,
    attestation: &E3ChildSecurityAttestationV1,
    cgroup: &CanonicalCgroupIdentityV1,
) -> Result<()> {
    let status = read_child_process_file_at(process_fd, "status")?;
    let pid = status_number_v1(&status, "Pid:")? as u32;
    let namespace_pids = status_line_v1(&status, "NSpid:")?
        .split_ascii_whitespace()
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parse child NSpid")?;
    let uid = status_quad_v1(&status, "Uid:")?;
    let gid = status_quad_v1(&status, "Gid:")?;
    let groups = status_line_v1(&status, "Groups:")?
        .split_ascii_whitespace()
        .count() as u32;
    let zero = "0000000000000000";
    let cap_inheritable = status_hex_v1(&status, "CapInh:")?;
    let cap_permitted = status_hex_v1(&status, "CapPrm:")?;
    let cap_effective = status_hex_v1(&status, "CapEff:")?;
    let cap_bounding = status_hex_v1(&status, "CapBnd:")?;
    let cap_ambient = status_hex_v1(&status, "CapAmb:")?;
    let no_new_privs = status_number_v1(&status, "NoNewPrivs:")?;
    let seccomp_mode = status_number_v1(&status, "Seccomp:")? as u32;
    let tracer_pid = status_number_v1(&status, "TracerPid:")? as u32;
    let expected_uid = (
        input.target_uid,
        input.target_uid,
        input.target_uid,
        input.target_uid,
    );
    let expected_gid = (
        input.target_gid,
        input.target_gid,
        input.target_gid,
        input.target_gid,
    );
    if pid != attestation.pid
        || namespace_pids.first() != Some(&attestation.pid)
        || namespace_pids.last() != Some(&attestation.pid)
        || read_process_start_time_at(process_fd)? != attestation.pid_start_time_ticks
        || uid != expected_uid
        || uid
            != (
                attestation.real_uid,
                attestation.effective_uid,
                attestation.saved_uid,
                attestation.filesystem_uid,
            )
        || gid != expected_gid
        || gid
            != (
                attestation.real_gid,
                attestation.effective_gid,
                attestation.saved_gid,
                attestation.filesystem_gid,
            )
        || groups != 0
        || groups != attestation.supplementary_group_count
        || cap_inheritable != zero
        || cap_inheritable != attestation.cap_inheritable
        || cap_permitted != zero
        || cap_permitted != attestation.cap_permitted
        || cap_effective != zero
        || cap_effective != attestation.cap_effective
        || cap_bounding != zero
        || cap_bounding != attestation.cap_bounding
        || cap_ambient != zero
        || cap_ambient != attestation.cap_ambient
        || no_new_privs != 1
        || !attestation.no_new_privs
        || seccomp_mode != 2
        || seccomp_mode != attestation.seccomp_mode
        || tracer_pid != attestation.tracer_pid
        || !role_has_exact_tracer_v1(input.child_role, tracer_pid)
    {
        bail!("live E3-D child status differs from its authenticated security posture");
    }

    validate_map_file_at(
        process_fd,
        "uid_map",
        &input.user_namespace_requirement.uid_map,
    )?;
    validate_map_file_at(
        process_fd,
        "gid_map",
        &input.user_namespace_requirement.gid_map,
    )?;
    let membership = read_child_process_file_at(process_fd, "cgroup")?;
    if membership != format!("0::/{}\n", cgroup.cgroup_relative_path) {
        bail!("live E3-D child cgroup differs from its retained identity");
    }
    validate_child_cgroup_membership(attestation.pid, cgroup)?;
    if read_child_process_file_at(process_fd, "cgroup")? != membership
        || read_process_start_time_at(process_fd)? != attestation.pid_start_time_ticks
    {
        bail!("E3-D child process identity changed during security readback");
    }
    let cap_last_cap = std::fs::read_to_string("/proc/sys/kernel/cap_last_cap")
        .context("read cap_last_cap during child security validation")?
        .trim()
        .parse::<u32>()
        .context("parse cap_last_cap during child security validation")?;
    let kernel_boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .context("read kernel boot ID during child security validation")?;
    if cap_last_cap > 63
        || cap_last_cap != attestation.cap_last_cap
        || kernel_boot_id.trim() != input.kernel_boot_id
        || kernel_boot_id.trim() != attestation.kernel_boot_id
    {
        bail!("live E3-D kernel security identity differs from its attestation");
    }
    Ok(())
}

fn read_child_process_file_at(process_fd: RawFd, name: &str) -> Result<String> {
    let name = std::ffi::CString::new(name)?;
    let fd = unsafe {
        libc::openat(
            process_fd,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open child {name:?} through retained proc descriptor"));
    }
    let mut value = String::new();
    unsafe { File::from_raw_fd(fd) }
        .take(131_073)
        .read_to_string(&mut value)
        .context("read child proc file through retained descriptor")?;
    if value.len() > 131_072 {
        bail!("child proc readback exceeds its fixed bound");
    }
    Ok(value)
}

fn validate_map_file_at(
    process_fd: RawFd,
    name: &str,
    extent: &E3LinuxIdMapExtentV1,
) -> Result<()> {
    let bytes = read_child_process_file_at(process_fd, name)?;
    let values = bytes
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("parse live child {name}"))?;
    if values != [extent.inside_id, extent.outside_id, extent.length] {
        bail!("live child {name} differs from the authenticated extent");
    }
    Ok(())
}

fn status_line_v1<'a>(status: &'a str, key: &str) -> Result<&'a str> {
    status
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .with_context(|| format!("missing {key} in live child proc status"))
}

fn status_quad_v1(status: &str, key: &str) -> Result<(u64, u64, u64, u64)> {
    let values = status_line_v1(status, key)?
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if let [a, b, c, d] = values.as_slice() {
        Ok((*a, *b, *c, *d))
    } else {
        bail!("invalid {key} live child proc status shape")
    }
}

fn status_hex_v1(status: &str, key: &str) -> Result<String> {
    let value = status_line_v1(status, key)?.trim();
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid {key} live child proc status value");
    }
    Ok(value.to_ascii_lowercase())
}

fn status_number_v1(status: &str, key: &str) -> Result<u64> {
    status_line_v1(status, key)?
        .trim()
        .parse()
        .with_context(|| format!("parse {key} from live child proc status"))
}

fn validate_child_namespace_descriptor(
    child: &OwnedFd,
    parent: &HeldE3ServiceUserNamespaceV1,
    trusted_service_uid: u64,
) -> Result<()> {
    if unsafe { libc::ioctl(child.as_raw_fd(), NS_GET_NSTYPE as _) } != libc::CLONE_NEWUSER {
        bail!("child namespace descriptor is not a user namespace");
    }
    let mut owner: libc::uid_t = u32::MAX;
    if unsafe { libc::ioctl(child.as_raw_fd(), NS_GET_OWNER_UID as _, &mut owner) } != 0 {
        return Err(std::io::Error::last_os_error()).context("read child user namespace owner");
    }
    if u64::from(owner) != trusted_service_uid {
        bail!("child user namespace has a substituted owner");
    }
    let parent_fd = unsafe { libc::ioctl(child.as_raw_fd(), NS_GET_PARENT as _) };
    if parent_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open child user namespace parent");
    }
    let parent_fd = unsafe { OwnedFd::from_raw_fd(parent_fd) };
    if descriptor_identity(parent_fd.as_raw_fd())?
        != descriptor_identity(parent.namespace_fd.as_raw_fd())?
    {
        bail!("child user namespace has a substituted parent");
    }
    Ok(())
}

fn descriptor_identity(fd: RawFd) -> Result<(u64, u64)> {
    let mut stat: libc::stat = unsafe { zeroed() };
    if unsafe { libc::fstat(fd, &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat namespace descriptor");
    }
    Ok((stat.st_dev, stat.st_ino))
}

fn verify_e3_child_process_and_cgroup_quiescent(child: &HeldE3ChildUserNamespaceV1) -> Result<()> {
    let mut poll = libc::pollfd {
        fd: child.pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let polled = unsafe { libc::poll(&mut poll, 1, 0) };
    if polled < 0 {
        return Err(std::io::Error::last_os_error()).context("poll terminal E3 child pidfd");
    }
    if polled != 1 || poll.revents & libc::POLLIN == 0 {
        bail!("E3 child process is not terminal at exclusion release");
    }
    let first = read_empty_e3_cgroup_tree(&child.cgroup)?;
    let second = read_empty_e3_cgroup_tree(&child.cgroup)?;
    if first != second {
        bail!("E3 child cgroup changed across the twice-empty release proof");
    }
    Ok(())
}

fn read_empty_e3_cgroup_tree(
    identity: &CanonicalCgroupIdentityV1,
) -> Result<Vec<(String, String, String)>> {
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;

    fn read_file_at(directory: RawFd, name: &str) -> Result<String> {
        let name = std::ffi::CString::new(name).unwrap();
        let fd = unsafe {
            libc::openat(
                directory,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error()).context("open E3 cgroup proof file");
        }
        let mut bytes = String::new();
        unsafe { File::from_raw_fd(fd) }
            .read_to_string(&mut bytes)
            .context("read E3 cgroup proof file")?;
        Ok(bytes)
    }

    fn scan(
        directory: &File,
        relative: &str,
        expected_device: u64,
        observations: &mut Vec<(String, String, String)>,
    ) -> Result<()> {
        let metadata = directory
            .metadata()
            .with_context(|| format!("stat E3 cgroup {relative}"))?;
        if !metadata.is_dir() || metadata.dev() != expected_device {
            bail!("E3 cgroup tree crossed a filesystem boundary");
        }
        let events = read_file_at(directory.as_raw_fd(), "cgroup.events")?;
        let procs = read_file_at(directory.as_raw_fd(), "cgroup.procs")?;
        let populated = events
            .lines()
            .find_map(|line| line.strip_prefix("populated "))
            .context("E3 cgroup.events lacks populated state")?;
        if populated != "0" || !procs.trim().is_empty() {
            bail!("E3 child cgroup is not empty at exclusion release");
        }
        observations.push((relative.to_string(), events, procs));

        let duplicate = unsafe { libc::fcntl(directory.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
        if duplicate < 0 {
            return Err(std::io::Error::last_os_error()).context("duplicate E3 cgroup directory");
        }
        let stream = unsafe { libc::fdopendir(duplicate) };
        if stream.is_null() {
            unsafe { libc::close(duplicate) };
            return Err(std::io::Error::last_os_error()).context("enumerate E3 cgroup directory");
        }
        let mut children = Vec::new();
        loop {
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                break;
            }
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if name == b"." || name == b".." {
                continue;
            }
            let name = std::str::from_utf8(name).context("non-UTF-8 E3 cgroup component")?;
            let name_c = std::ffi::CString::new(name).unwrap();
            let mut stat: libc::stat = unsafe { zeroed() };
            if unsafe {
                libc::fstatat(
                    directory.as_raw_fd(),
                    name_c.as_ptr(),
                    &mut stat,
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                unsafe { libc::closedir(stream) };
                return Err(std::io::Error::last_os_error()).context("inspect E3 cgroup entry");
            }
            if stat.st_mode & libc::S_IFMT == libc::S_IFDIR {
                children.push(name.to_string());
            }
        }
        if unsafe { libc::closedir(stream) } != 0 {
            return Err(std::io::Error::last_os_error()).context("close E3 cgroup enumeration");
        }
        children.sort();
        for child in children {
            let child_c = std::ffi::CString::new(child.as_str()).unwrap();
            let fd = unsafe {
                libc::openat(
                    directory.as_raw_fd(),
                    child_c.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error()).context("open child E3 cgroup");
            }
            let child_directory = unsafe { File::from_raw_fd(fd) };
            scan(
                &child_directory,
                &format!("{relative}/{child}"),
                expected_device,
                observations,
            )?;
        }
        Ok(())
    }

    if identity.cgroup_relative_path.is_empty()
        || identity.cgroup_relative_path.starts_with('/')
        || identity
            .cgroup_relative_path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        bail!("E3 child cgroup identity is not canonical");
    }
    let mount = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open("/sys/fs/cgroup")
        .context("open cgroup v2 mount for E3 release")?;
    let mount_metadata = mount.metadata().context("stat cgroup v2 mount")?;
    let mut mount_fs: libc::statfs = unsafe { zeroed() };
    if unsafe { libc::fstatfs(mount.as_raw_fd(), &mut mount_fs) } != 0 {
        return Err(std::io::Error::last_os_error()).context("statfs cgroup v2 mount");
    }
    if mount_fs.f_type as i128 != 0x6367_7270i128
        || mount_metadata.dev() != identity.cgroup_v2_mount_device_id
        || mount_metadata.ino() != identity.cgroup_v2_mount_inode
    {
        bail!("E3 child cgroup mount identity drifted");
    }
    let mut directory = mount;
    for component in Path::new(&identity.cgroup_relative_path).components() {
        let std::path::Component::Normal(component) = component else {
            bail!("E3 child cgroup component is not canonical");
        };
        let component = component
            .to_str()
            .context("non-UTF-8 E3 child cgroup component")?;
        let component = std::ffi::CString::new(component).unwrap();
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error()).context("open exact E3 child cgroup");
        }
        directory = unsafe { File::from_raw_fd(fd) };
    }
    let metadata = directory.metadata().context("stat exact E3 child cgroup")?;
    if metadata.dev() != identity.cgroup_v2_mount_device_id
        || metadata.ino() != identity.cgroup_directory_inode
    {
        bail!("E3 child cgroup directory identity drifted");
    }
    let mut observations = Vec::new();
    scan(
        &directory,
        &identity.cgroup_relative_path,
        identity.cgroup_v2_mount_device_id,
        &mut observations,
    )?;
    Ok(observations)
}

fn pidfd_open(pid: u32) -> Result<OwnedFd> {
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("pidfd_open E3-D child");
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn read_process_start_time(pid: u32) -> Result<u64> {
    let mut stat = String::new();
    File::open(format!("/proc/{pid}/stat"))
        .and_then(|mut file| file.read_to_string(&mut stat))
        .context("read child process stat")?;
    parse_process_start_time(&stat)
}

fn validate_child_cgroup_membership(pid: u32, identity: &CanonicalCgroupIdentityV1) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let membership = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .context("read E3 child cgroup membership")?;
    if membership != format!("0::/{}\n", identity.cgroup_relative_path) {
        bail!("E3 child is not in its registered cgroup");
    }
    let mount = std::fs::metadata("/sys/fs/cgroup").context("stat cgroup v2 mount")?;
    let directory = std::fs::metadata(format!("/sys/fs/cgroup/{}", identity.cgroup_relative_path))
        .context("stat registered E3 child cgroup")?;
    if mount.dev() != identity.cgroup_v2_mount_device_id
        || mount.ino() != identity.cgroup_v2_mount_inode
        || directory.dev() != identity.cgroup_v2_mount_device_id
        || directory.ino() != identity.cgroup_directory_inode
    {
        bail!("registered E3 child cgroup identity drifted");
    }
    Ok(())
}

fn read_process_start_time_at(process_fd: RawFd) -> Result<u64> {
    let name = std::ffi::CString::new("stat").unwrap();
    let stat_fd = unsafe {
        libc::openat(
            process_fd,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if stat_fd < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open child process stat through numeric proc descriptor");
    }
    let mut stat = String::new();
    unsafe { File::from_raw_fd(stat_fd) }
        .read_to_string(&mut stat)
        .context("read child process stat through numeric proc descriptor")?;
    parse_process_start_time(&stat)
}

fn parse_process_start_time(stat: &str) -> Result<u64> {
    let after_comm = stat
        .rfind(") ")
        .and_then(|index| stat.get(index + 2..))
        .context("parse child process stat comm")?;
    after_comm
        .split_ascii_whitespace()
        .nth(19)
        .context("parse child process start time")?
        .parse()
        .context("parse numeric child process start time")
}

fn receive_exact_setup_byte(fd: RawFd, expected: u8) -> Result<()> {
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
        bail!("malformed E3-D namespace setup message");
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
        bail!("duplicate, trailing, or closed E3-D namespace setup message");
    }
    Ok(())
}

fn send_exact_setup_byte(fd: RawFd, byte: u8) -> Result<()> {
    let sent = unsafe { libc::send(fd, (&byte as *const u8).cast(), 1, libc::MSG_NOSIGNAL) };
    if sent != 1 {
        return Err(std::io::Error::last_os_error()).context("send E3-D namespace release");
    }
    Ok(())
}

fn validate_map_file(pid: u32, name: &str, extent: &E3LinuxIdMapExtentV1) -> Result<()> {
    let mut bytes = String::new();
    File::open(format!("/proc/{pid}/{name}"))
        .and_then(|mut file| file.read_to_string(&mut bytes))
        .with_context(|| format!("read child {name}"))?;
    let values = bytes
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("parse child {name}"))?;
    if values != [extent.inside_id, extent.outside_id, extent.length] {
        bail!("child {name} differs from the authenticated extent");
    }
    Ok(())
}

fn install_maps_with_bounded_effective_capabilities(
    pid: u32,
    requirement: &E3UserNamespaceRequirementV1,
) -> Result<()> {
    let original = read_capabilities()?;
    let transition_mask = capability_mask(CAP_SETUID) | capability_mask(CAP_SETGID);
    let original_effective = capability_effective_mask(&original);
    let original_permitted = capability_permitted_mask(&original);
    if original_effective & transition_mask != 0
        || original_permitted & transition_mask != transition_mask
    {
        bail!("E3-D map capabilities are not parked in the permitted set");
    }
    let mut raised = original;
    set_effective_mask(&mut raised, original_effective | transition_mask);
    write_capabilities(&raised).context("raise bounded E3-D map capabilities")?;
    let result = (|| {
        write_map_file(pid, "uid_map", &requirement.uid_map)?;
        write_map_file(pid, "gid_map", &requirement.gid_map)
    })();
    if write_capabilities(&original).is_err() || read_capabilities().ok() != Some(original) {
        std::process::abort();
    }
    result
}

fn write_map_file(pid: u32, name: &str, extent: &E3LinuxIdMapExtentV1) -> Result<()> {
    let value = format!(
        "{} {} {}\n",
        extent.inside_id, extent.outside_id, extent.length
    );
    OpenOptions::new()
        .write(true)
        .open(format!("/proc/{pid}/{name}"))
        .and_then(|mut file| file.write_all(value.as_bytes()))
        .with_context(|| format!("install child {name}"))
}

fn capability_mask(capability: u32) -> u64 {
    1u64 << capability
}

fn capability_effective_mask(data: &[CapabilityData; 2]) -> u64 {
    u64::from(data[0].effective) | (u64::from(data[1].effective) << 32)
}

fn capability_permitted_mask(data: &[CapabilityData; 2]) -> u64 {
    u64::from(data[0].permitted) | (u64::from(data[1].permitted) << 32)
}

fn read_capabilities() -> Result<[CapabilityData; 2]> {
    let mut header = CapabilityHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    let mut data = [CapabilityData::default(); 2];
    if unsafe { libc::syscall(libc::SYS_capget, &mut header, data.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("capget E3-D service thread");
    }
    Ok(data)
}

fn write_capabilities(data: &[CapabilityData; 2]) -> Result<()> {
    let mut header = CapabilityHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0,
    };
    if unsafe { libc::syscall(libc::SYS_capset, &mut header, data.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("capset E3-D service thread");
    }
    Ok(())
}

fn set_effective_mask(data: &mut [CapabilityData; 2], mask: u64) {
    data[0].effective = mask as u32;
    data[1].effective = (mask >> 32) as u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine as _;
    use config_projection::{
        ConfigProjectionArtifactRoleV1, ConfigProjectionCodecV1, DescriptorPinnedArtifactV1,
        E3DeniedControlProbeTargetV1, E3IsolatedChildRoleV1, E3PolicyAuthoritySourceV1,
        E3RuntimeSupportManifestV1, GatewayListenerIdentityV1, InWorldGatewayRefV1,
        LinuxArtifactSourceV1, RuntimeArtifactAuthorityRefV1, RuntimeArtifactProvenanceV1,
        RuntimeSupportDirectoryV1, RuntimeSupportFileV1,
    };
    use serde::Serialize;
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use transport_api_types::{
        AuthorityObjectKindV1, AuthorityObjectRefV1, DispatchPolicyCommitmentRefCarrierV1,
        OpaqueAuthorityCommitmentV1, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
        PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    };

    struct CapabilityRestore([CapabilityData; 2]);

    impl Drop for CapabilityRestore {
        fn drop(&mut self) {
            if write_capabilities(&self.0).is_err() {
                std::process::abort();
            }
        }
    }

    fn id(prefix: &str) -> String {
        format!("{prefix}{}", uuid::Uuid::now_v7())
    }

    fn hash_omitting<T: Serialize>(domain: &str, key: &str, value: &T, omitted: &str) -> String {
        let mut inner = serde_json::to_value(value).unwrap();
        inner.as_object_mut().unwrap().remove(omitted);
        let mut preimage = serde_json::Map::new();
        preimage.insert("domain".to_string(), Value::String(domain.to_string()));
        preimage.insert(key.to_string(), inner);
        ConfigProjectionCodecV1::domain_sha256("", &Value::Object(preimage)).unwrap()
    }

    fn pipe() -> (OwnedFd, OwnedFd) {
        let mut descriptors = [-1; 2];
        assert_eq!(
            unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
            0
        );
        unsafe {
            (
                OwnedFd::from_raw_fd(descriptors[0]),
                OwnedFd::from_raw_fd(descriptors[1]),
            )
        }
    }

    fn clear_close_on_exec(fd: RawFd) {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) },
            0
        );
    }

    fn write_owned(fd: OwnedFd, bytes: &[u8]) {
        let mut file = File::from(fd);
        file.write_all(bytes).unwrap();
    }

    fn read_owned(fd: OwnedFd) -> Vec<u8> {
        let mut file = File::from(fd);
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes
    }

    fn copy_regular(source: &str, destination: &std::path::Path) {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::copy(source, destination).unwrap();
        std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    fn runtime_support_file(root: &std::path::Path, absolute_path: &str) -> RuntimeSupportFileV1 {
        use std::os::unix::fs::MetadataExt;

        let path = root.join(absolute_path.trim_start_matches('/'));
        let metadata = std::fs::metadata(&path).unwrap();
        RuntimeSupportFileV1 {
            absolute_path: absolute_path.to_string(),
            device_id: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode() & 0o7777,
            byte_length: metadata.len(),
            sha256: format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
        }
    }

    #[test]
    fn tracer_binding_is_exact_for_each_e3_child_role() {
        let service_pid = std::process::id();
        assert!(role_has_exact_tracer_v1(
            E3IsolatedChildRoleV1::Codex,
            service_pid
        ));
        assert!(!role_has_exact_tracer_v1(E3IsolatedChildRoleV1::Codex, 0));
        assert!(!role_has_exact_tracer_v1(
            E3IsolatedChildRoleV1::Codex,
            service_pid.saturating_add(1)
        ));
        for role in [
            E3IsolatedChildRoleV1::ManagedGateway,
            E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe,
        ] {
            assert!(role_has_exact_tracer_v1(role, 0));
            assert!(!role_has_exact_tracer_v1(role, service_pid));
        }
    }

    #[test]
    fn process_wide_exclusion_is_recovering_legacy_or_one_exact_e3_epoch() {
        let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        assert!(exclusion.acquire_non_e3_child().is_err());
        assert!(exclusion.acquire_e3_exclusive("world-a", 1).is_err());

        let recovered = exclusion
            .register_recovered_non_e3_child(
                41,
                101,
                CanonicalCgroupIdentityV1 {
                    cgroup_v2_mount_device_id: 7,
                    cgroup_v2_mount_inode: 11,
                    cgroup_directory_inode: 13,
                    cgroup_relative_path: "substrate/world-a/gateway".to_string(),
                },
            )
            .unwrap();
        exclusion.finish_recovery().unwrap();
        assert!(exclusion.acquire_e3_exclusive("world-a", 1).is_err());
        drop(recovered);

        let ordinary = exclusion.acquire_non_e3_child().unwrap();
        assert!(exclusion.acquire_e3_exclusive("world-a", 1).is_err());
        drop(ordinary);

        let e3 = exclusion.acquire_e3_exclusive("world-a", 1).unwrap();
        assert!(exclusion.acquire_non_e3_child().is_err());
        let sibling = exclusion.acquire_e3_exclusive("world-a", 1).unwrap();
        assert!(exclusion.acquire_e3_exclusive("world-b", 1).is_err());
        assert!(exclusion.acquire_e3_exclusive("world-a", 2).is_err());
        drop(sibling);
        drop(e3);

        assert!(exclusion.acquire_non_e3_child().is_ok());
    }

    #[test]
    fn poisoned_recovery_never_reopens_child_admission() {
        let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        exclusion.poison_recovering().unwrap();
        assert!(exclusion.finish_recovery().is_err());
        assert!(exclusion.acquire_non_e3_child().is_err());
        assert!(exclusion.acquire_e3_exclusive("world-a", 1).is_err());
    }

    #[test]
    fn setup_messages_reject_wrong_duplicate_trailing_and_eof_packets() {
        for malformed in [vec![0x00], vec![USERNS_CREATED, 0xff], Vec::new()] {
            let (parent, child) = create_e3_child_user_namespace_channel_v1().unwrap();
            if malformed.is_empty() {
                drop(child);
            } else {
                assert_eq!(
                    unsafe {
                        libc::send(
                            child.as_raw_fd(),
                            malformed.as_ptr().cast(),
                            malformed.len(),
                            libc::MSG_NOSIGNAL,
                        )
                    },
                    malformed.len() as isize
                );
            }
            assert!(receive_exact_setup_byte(parent.as_raw_fd(), USERNS_CREATED).is_err());
        }
    }

    #[test]
    #[ignore = "requires the explicit privileged E3-D acceptance environment"]
    fn privileged_namespace_handshake_maps_exact_nonroot_identity_and_restores_cap_window() {
        assert_eq!(
            std::env::var_os("SUBSTRATE_E3D_PRIVILEGED_TEST").as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "privileged E3-D acceptance must be explicitly enabled"
        );
        assert_eq!(
            unsafe { libc::geteuid() },
            0,
            "privileged E3-D test must run as root"
        );
        let target_uid = std::env::var("SUBSTRATE_E3D_TARGET_UID")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let target_gid = std::env::var("SUBSTRATE_E3D_TARGET_GID")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_ne!(target_uid, 0);
        assert_ne!(target_gid, 0);

        let original = read_capabilities().unwrap();
        let _restore = CapabilityRestore(original);
        let transition = capability_mask(CAP_SETUID) | capability_mask(CAP_SETGID);
        assert_eq!(
            capability_permitted_mask(&original) & transition,
            transition
        );
        let mut parked = original;
        set_effective_mask(
            &mut parked,
            capability_effective_mask(&original) & !transition,
        );
        write_capabilities(&parked).unwrap();

        let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        exclusion.finish_recovery().unwrap();
        let e3 = exclusion
            .acquire_e3_exclusive("world-privileged-test", 1)
            .unwrap();
        let cgroup_relative = format!("substrate-e3d-test-{}", std::process::id());
        let cgroup_path = format!("/sys/fs/cgroup/{cgroup_relative}");
        std::fs::create_dir(&cgroup_path).unwrap();
        let mount_metadata = std::fs::metadata("/sys/fs/cgroup").unwrap();
        let cgroup_metadata = std::fs::metadata(&cgroup_path).unwrap();
        let child_cgroup = CanonicalCgroupIdentityV1 {
            cgroup_v2_mount_device_id: std::os::unix::fs::MetadataExt::dev(&mount_metadata),
            cgroup_v2_mount_inode: std::os::unix::fs::MetadataExt::ino(&mount_metadata),
            cgroup_directory_inode: std::os::unix::fs::MetadataExt::ino(&cgroup_metadata),
            cgroup_relative_path: cgroup_relative,
        };
        let requirement =
            bind_e3_service_user_namespace_v1(&exclusion, target_uid, target_gid).unwrap();
        let (parent_setup, child_setup) = create_e3_child_user_namespace_channel_v1().unwrap();
        let mut ready = [-1; 2];
        let mut release = [-1; 2];
        assert_eq!(
            unsafe { libc::pipe2(ready.as_mut_ptr(), libc::O_CLOEXEC) },
            0
        );
        assert_eq!(
            unsafe { libc::pipe2(release.as_mut_ptr(), libc::O_CLOEXEC) },
            0
        );
        let child_pid = unsafe { libc::fork() };
        assert!(child_pid >= 0);
        if child_pid == 0 {
            unsafe {
                libc::close(parent_setup.as_raw_fd());
                libc::close(ready[0]);
                libc::close(release[1]);
            }
            if unsafe { libc::unshare(libc::CLONE_NEWUSER) } != 0
                || send_exact_setup_byte(child_setup.as_raw_fd(), USERNS_CREATED).is_err()
                || receive_exact_setup_byte(child_setup.as_raw_fd(), USERNS_MAPPED).is_err()
                || validate_map_file(std::process::id(), "uid_map", &requirement.uid_map).is_err()
                || validate_map_file(std::process::id(), "gid_map", &requirement.gid_map).is_err()
            {
                unsafe { libc::_exit(120) };
            }
            let marker = [0x7a_u8];
            if unsafe { libc::write(ready[1], marker.as_ptr().cast(), 1) } != 1 {
                unsafe { libc::_exit(121) };
            }
            unsafe { libc::close(ready[1]) };
            let mut gate = 0u8;
            if unsafe { libc::read(release[0], (&mut gate as *mut u8).cast(), 1) } != 1
                || gate != 0x01
            {
                unsafe { libc::_exit(122) };
            }
            unsafe { libc::_exit(0) };
        }
        drop(child_setup);
        std::fs::write(
            format!("{cgroup_path}/cgroup.procs"),
            format!("{child_pid}\n"),
        )
        .unwrap();
        unsafe {
            libc::close(ready[1]);
            libc::close(release[0]);
        }
        let child_pid = child_pid as u32;
        let child_start = read_process_start_time(child_pid).unwrap();
        install_and_validate_e3_child_user_namespace_v1(
            &exclusion,
            &parent_setup,
            child_pid,
            child_start,
            &requirement,
            &child_cgroup,
        )
        .unwrap();
        assert_eq!(read_capabilities().unwrap(), parked);
        let mut marker = 0u8;
        assert_eq!(
            unsafe { libc::read(ready[0], (&mut marker as *mut u8).cast(), 1) },
            1
        );
        assert_eq!(marker, 0x7a);
        {
            let state = exclusion.lock_state().unwrap();
            let held = state
                .child_user_namespaces
                .get(&(child_pid, child_start))
                .expect("retained child namespace and pidfd");
            assert_ne!(
                (held.device_id, held.inode),
                (
                    requirement.parent_namespace_device_id,
                    requirement.parent_namespace_inode
                )
            );
            assert!(unsafe { libc::fcntl(held.pidfd.as_raw_fd(), libc::F_GETFD) } >= 0);
        }
        let gate = [0x01_u8];
        assert_eq!(
            unsafe { libc::write(release[1], gate.as_ptr().cast(), 1) },
            1
        );
        unsafe {
            libc::close(release[1]);
            libc::close(ready[0]);
        }
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(child_pid as i32, &mut status, 0) },
            child_pid as i32
        );
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);

        let (failure_parent, failure_child) = create_e3_child_user_namespace_channel_v1().unwrap();
        let failure_pid = unsafe { libc::fork() };
        assert!(failure_pid >= 0);
        if failure_pid == 0 {
            unsafe { libc::close(failure_parent.as_raw_fd()) };
            if unsafe { libc::unshare(libc::CLONE_NEWUSER) } != 0
                || send_exact_setup_byte(failure_child.as_raw_fd(), USERNS_CREATED).is_err()
            {
                unsafe { libc::_exit(123) };
            }
            let _ = receive_exact_setup_byte(failure_child.as_raw_fd(), USERNS_MAPPED);
            unsafe { libc::_exit(0) };
        }
        drop(failure_child);
        std::fs::write(
            format!("{cgroup_path}/cgroup.procs"),
            format!("{failure_pid}\n"),
        )
        .unwrap();
        let failure_pid = failure_pid as u32;
        let failure_start = read_process_start_time(failure_pid).unwrap();
        let mut invalid_requirement = requirement.clone();
        invalid_requirement.uid_map.inside_id = u64::MAX;
        invalid_requirement.uid_map.outside_id = u64::MAX;
        assert!(install_and_validate_e3_child_user_namespace_v1(
            &exclusion,
            &failure_parent,
            failure_pid,
            failure_start,
            &invalid_requirement,
            &child_cgroup,
        )
        .is_err());
        assert_eq!(read_capabilities().unwrap(), parked);
        assert!(!exclusion
            .lock_state()
            .unwrap()
            .child_user_namespaces
            .contains_key(&(failure_pid, failure_start)));
        drop(failure_parent);
        let mut failure_status = 0;
        assert_eq!(
            unsafe { libc::waitpid(failure_pid as i32, &mut failure_status, 0) },
            failure_pid as i32
        );
        assert!(libc::WIFEXITED(failure_status));
        drop(e3);
        std::fs::remove_dir(&cgroup_path).unwrap();
        assert!(exclusion.acquire_non_e3_child().is_ok());
    }

    #[test]
    #[ignore = "requires the explicit privileged E3-D wrapper acceptance environment"]
    fn privileged_static_readiness_wrapper_uses_actual_service_handshake_and_release_barriers() {
        assert_eq!(
            std::env::var_os("SUBSTRATE_E3D_PRIVILEGED_TEST").as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "privileged E3-D acceptance must be explicitly enabled"
        );
        let static_wrapper = std::env::var_os("SUBSTRATE_E3D_STATIC_WORLD_ENTRY")
            .expect("explicit static wrapper proof input is required");
        assert_eq!(unsafe { libc::geteuid() }, 0);
        let target_uid = std::env::var("SUBSTRATE_E3D_TARGET_UID")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let target_gid = std::env::var("SUBSTRATE_E3D_TARGET_GID")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_ne!(target_uid, 0);
        assert_ne!(target_gid, 0);

        let root = tempfile::tempdir().unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        for directory in [
            ".old-root",
            "dev",
            "proc",
            "sys/fs/cgroup",
            "etc/codex",
            "etc/ssl/certs",
            "usr/local/lib/substrate/e3",
        ] {
            let path = root.path().join(directory);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        for path in [
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/passwd",
            "/etc/group",
            "/etc/resolv.conf",
            "/etc/ssl/certs/ca-certificates.crt",
        ] {
            copy_regular(path, &root.path().join(path.trim_start_matches('/')));
        }
        let installed_wrapper = root
            .path()
            .join("usr/local/lib/substrate/e3/substrate-world-entry");
        std::fs::copy(&static_wrapper, &installed_wrapper).unwrap();
        std::fs::set_permissions(&installed_wrapper, std::fs::Permissions::from_mode(0o755))
            .unwrap();

        use std::os::unix::fs::MetadataExt;
        let wrapper_file = File::open(&installed_wrapper).unwrap();
        let wrapper_metadata = wrapper_file.metadata().unwrap();
        let wrapper_sha256 = format!(
            "{:x}",
            Sha256::digest(std::fs::read(&installed_wrapper).unwrap())
        );
        let mut runtime_support = E3RuntimeSupportManifestV1 {
            schema_version: 1,
            support_policy_version: 1,
            elf_execution_model: LinuxArtifactSourceV1::validate_e3_static_elf_v1(&wrapper_file)
                .unwrap(),
            elf_interpreter: None,
            dynamic_loader_cache: None,
            ordered_elf_dependencies: Vec::new(),
            ordered_present_common_files: [
                "/etc/hosts",
                "/etc/nsswitch.conf",
                "/etc/passwd",
                "/etc/group",
                "/etc/resolv.conf",
                "/etc/ssl/certs/ca-certificates.crt",
            ]
            .into_iter()
            .map(|path| runtime_support_file(root.path(), path))
            .collect(),
            system_config_mount_target: {
                let metadata = std::fs::metadata(root.path().join("etc/codex")).unwrap();
                RuntimeSupportDirectoryV1 {
                    absolute_path: "/etc/codex".to_string(),
                    device_id: metadata.dev(),
                    inode: metadata.ino(),
                    mode: metadata.mode() & 0o7777,
                    owner_uid: u64::from(metadata.uid()),
                    owner_gid: u64::from(metadata.gid()),
                    ordered_entry_names: Vec::new(),
                }
            },
            manifest_hash: String::new(),
        };
        runtime_support.manifest_hash = hash_omitting(
            "substrate.e3.runtime-support-manifest.v1",
            "manifest",
            &runtime_support,
            "manifest_hash",
        );
        let artifact = DescriptorPinnedArtifactV1 {
            role: ConfigProjectionArtifactRoleV1::WorldEntryWrapper,
            configured_absolute_path: "/usr/local/lib/substrate/e3/substrate-world-entry"
                .to_string(),
            device_id: wrapper_metadata.dev(),
            inode: wrapper_metadata.ino(),
            file_type: "regular".to_string(),
            mode: wrapper_metadata.mode() & 0o7777,
            owner_uid: u64::from(wrapper_metadata.uid()),
            byte_length: wrapper_metadata.len(),
            sha256: wrapper_sha256.clone(),
            authority_ref: RuntimeArtifactAuthorityRefV1 {
                authority_store_id: id("cpa_"),
                manifest_id: id("ram_"),
                manifest_revision: 1,
                manifest_entry_id: id("rae_"),
                manifest_hash: "1".repeat(64),
                entry_hash: "2".repeat(64),
            },
            provenance: RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                component: "substrate-world-entry".to_string(),
                source_commit: "3".repeat(40),
                source_tree: "4".repeat(40),
                cargo_lock_sha256: "5".repeat(64),
                target_triple: "x86_64-unknown-linux-musl".to_string(),
                profile: "release".to_string(),
                executable_sha256: wrapper_sha256,
            },
            runtime_support,
        };

        let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        exclusion.finish_recovery().unwrap();
        let e3 = exclusion
            .acquire_e3_exclusive("world-static-wrapper-test", 1)
            .unwrap();
        let requirement =
            bind_e3_service_user_namespace_v1(&exclusion, target_uid, target_gid).unwrap();

        let cgroup_component = format!("substrate-e3d-wrapper-test-{}", uuid::Uuid::now_v7());
        let cgroup_path = std::path::Path::new("/sys/fs/cgroup").join(&cgroup_component);
        std::fs::create_dir(&cgroup_path).unwrap();
        let mount_metadata = std::fs::metadata("/sys/fs/cgroup").unwrap();
        let cgroup_metadata = std::fs::metadata(&cgroup_path).unwrap();
        let child_cgroup = CanonicalCgroupIdentityV1 {
            cgroup_v2_mount_device_id: mount_metadata.dev(),
            cgroup_v2_mount_inode: mount_metadata.ino(),
            cgroup_directory_inode: cgroup_metadata.ino(),
            cgroup_relative_path: cgroup_component,
        };

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let mut listener_metadata: libc::stat = unsafe { zeroed() };
        assert_eq!(
            unsafe { libc::fstat(listener.as_raw_fd(), &mut listener_metadata) },
            0
        );
        let network_namespace = File::open("/proc/self/ns/net").unwrap();
        let network_namespace_inode = network_namespace.metadata().unwrap().ino();
        let listener_identity = GatewayListenerIdentityV1 {
            transport: "tcp".to_string(),
            network_namespace_inode,
            address: "127.0.0.1".to_string(),
            port: listener_port,
            socket_inode: listener_metadata.st_ino,
            listen_backlog: 16,
            deny_boundary_effective_before_listen: true,
            responses_base_path: "/v1/responses".to_string(),
        };
        let gateway_ref = InWorldGatewayRefV1 {
            authority_store_id: id("cpa_"),
            gateway_instance_id: id("cgi_"),
            gateway_identity_hash: "6".repeat(64),
        };

        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: None,
                caged_required: true,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };
        let snapshot_bytes = ConfigProjectionCodecV1::encode_canonical_json(&snapshot).unwrap();
        let immutable_cap = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: id("hsa_"),
            commitment_id: id("dpc_"),
            exact_linkage_hash: "7".repeat(64),
        };
        let mut cgroup_probe = E3DeniedControlProbeTargetV1 {
            binding: E3DeniedControlProbeTargetBindingV1::CgroupControl {
                cgroup: child_cgroup.clone(),
                control_file: "cgroup.procs".to_string(),
            },
            target_hash: String::new(),
        };
        cgroup_probe.target_hash = hash_omitting(
            "substrate.e3.denied-control-probe-target.v1",
            "target",
            &cgroup_probe,
            "target_hash",
        );
        let mut nft_probe = E3DeniedControlProbeTargetV1 {
            binding: E3DeniedControlProbeTargetBindingV1::NftablesControl {
                network_namespace_inode,
            },
            target_hash: String::new(),
        };
        nft_probe.target_hash = hash_omitting(
            "substrate.e3.denied-control-probe-target.v1",
            "target",
            &nft_probe,
            "target_hash",
        );
        let mut enforcement = E3WorldFsEnforcementInputV1 {
            schema_version: 1,
            child_role: E3IsolatedChildRoleV1::ManagedGatewayReadinessProbe,
            projection_identity_hash: "8".repeat(64),
            policy_authority: E3PolicyAuthoritySourceV1::InitialLaunch {
                e2_activation_id: id("e2a_"),
                commitment_ref: immutable_cap.clone(),
            },
            immutable_worker_cap_ref: immutable_cap,
            policy_snapshot_bytes_base64: BASE64.encode(&snapshot_bytes),
            policy_snapshot_byte_length: snapshot_bytes.len() as u64,
            policy_snapshot_ref: AuthorityObjectRefV1 {
                ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                object_kind: AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "9".repeat(64),
                },
            },
            policy_snapshot_hash: format!("{:x}", Sha256::digest(&snapshot_bytes)),
            policy_snapshot_revision: "e3d-static-wrapper-test".to_string(),
            expected_process_cgroup: child_cgroup.clone(),
            kernel_boot_id: std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .unwrap()
                .trim()
                .to_string(),
            user_namespace_requirement: requirement.clone(),
            target_uid,
            target_gid,
            immutable_config_source: None,
            private_realization: None,
            codex_launch_plan_hash: None,
            executable_artifact: artifact,
            denied_control_probe_targets: vec![cgroup_probe, nft_probe],
            support_policy_version: 1,
            enforcement_input_hash: String::new(),
        };
        enforcement.enforcement_input_hash = hash_omitting(
            "substrate.e3.world-fs-enforcement-input.v1",
            "input",
            &enforcement,
            "enforcement_input_hash",
        );
        let launch_input_id = id("mgl_");
        let readiness_nonce = uuid::Uuid::now_v7().to_string();
        let mut probe = json!({
            "schema_version": 1,
            "launch_input_ref": {
                "authority_store_id": gateway_ref.authority_store_id,
                "launch_input_id": launch_input_id,
                "launch_input_hash": "a".repeat(64),
            },
            "gateway_ref": gateway_ref,
            "listener_identity": listener_identity,
            "readiness_nonce": readiness_nonce,
            "readiness_probe_cgroup": child_cgroup,
            "target_uid": target_uid,
            "target_gid": target_gid,
            "enforcement_input": enforcement,
            "connect_deadline_ms": 1_000,
            "request_release_deadline_ms": 1_000,
            "response_deadline_ms": 1_000,
            "maximum_response_bytes": 65_536,
            "input_hash": "",
        });
        probe["input_hash"] = Value::String(hash_omitting(
            "substrate.e3.managed-gateway-readiness-probe-input.v1",
            "input",
            &probe,
            "input_hash",
        ));
        let probe_bytes = ConfigProjectionCodecV1::encode_canonical_json(&probe).unwrap();
        let enforcement: E3WorldFsEnforcementInputV1 =
            serde_json::from_value(probe["enforcement_input"].clone()).unwrap();
        let gateway_ref: InWorldGatewayRefV1 =
            serde_json::from_value(probe["gateway_ref"].clone()).unwrap();
        let listener_identity: GatewayListenerIdentityV1 =
            serde_json::from_value(probe["listener_identity"].clone()).unwrap();
        let readiness_nonce = probe["readiness_nonce"].as_str().unwrap().to_string();

        let (probe_read, probe_write) = pipe();
        let (setup_read, setup_write) = pipe();
        let (parent_setup, child_setup) = create_e3_child_user_namespace_channel_v1().unwrap();
        let (start_read, start_write) = pipe();
        let (connected_read, connected_write) = pipe();
        let (request_release_read, request_release_write) = pipe();
        let (result_read, result_write) = pipe();
        for fd in [
            probe_read.as_raw_fd(),
            setup_write.as_raw_fd(),
            child_setup.as_raw_fd(),
            start_read.as_raw_fd(),
            connected_write.as_raw_fd(),
            request_release_read.as_raw_fd(),
            result_write.as_raw_fd(),
            wrapper_file.as_raw_fd(),
        ] {
            clear_close_on_exec(fd);
        }
        let environment = [
            (
                "SUBSTRATE_E3_READINESS_PROBE_INPUT_FD",
                probe_read.as_raw_fd(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                setup_write.as_raw_fd(),
            ),
            ("SUBSTRATE_WORLD_ENTRY_USERNS_FD", child_setup.as_raw_fd()),
            (
                "SUBSTRATE_E3_READINESS_PROBE_START_FD",
                start_read.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_CONNECTED_FD",
                connected_write.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_REQUEST_RELEASE_FD",
                request_release_read.as_raw_fd(),
            ),
            (
                "SUBSTRATE_E3_READINESS_PROBE_RESULT_FD",
                result_write.as_raw_fd(),
            ),
            (
                "SUBSTRATE_WORLD_ENTRY_SELF_ARTIFACT_FD",
                wrapper_file.as_raw_fd(),
            ),
        ]
        .into_iter()
        .map(|(key, fd)| CString::new(format!("{key}={fd}")).unwrap())
        .chain(std::iter::once(
            CString::new("SUBSTRATE_WORLD_ENTRY_ROLE=managed_gateway_readiness_probe").unwrap(),
        ))
        .collect::<Vec<_>>();
        let environment_pointers = environment
            .iter()
            .map(|value| value.as_ptr() as usize)
            .chain(std::iter::once(0usize))
            .collect::<Vec<_>>();
        let argv = [CString::new("substrate-world-entry").unwrap()];
        let argv_pointers = [argv[0].as_ptr() as usize, 0usize];
        let empty = CString::new("").unwrap();
        let mount_root = CString::new("/").unwrap();
        let synthetic_root = CString::new(root.path().as_os_str().as_encoded_bytes()).unwrap();
        let synthetic_old_root =
            CString::new(root.path().join(".old-root").as_os_str().as_encoded_bytes()).unwrap();
        let synthetic_proc =
            CString::new(root.path().join("proc").as_os_str().as_encoded_bytes()).unwrap();
        let proc_source = CString::new("proc").unwrap();
        let proc_type = CString::new("proc").unwrap();
        let cgroup_source = CString::new("/sys/fs/cgroup").unwrap();
        let synthetic_cgroup = CString::new(
            root.path()
                .join("sys/fs/cgroup")
                .as_os_str()
                .as_encoded_bytes(),
        )
        .unwrap();
        let dev_source = CString::new("/dev").unwrap();
        let synthetic_dev =
            CString::new(root.path().join("dev").as_os_str().as_encoded_bytes()).unwrap();
        let new_root = CString::new("/").unwrap();
        let old_root = CString::new("/.old-root").unwrap();
        let wrapper_fd = wrapper_file.as_raw_fd();

        let original = read_capabilities().unwrap();
        let _restore = CapabilityRestore(original);
        let transition =
            capability_mask(CAP_SETGID) | capability_mask(CAP_SETUID) | capability_mask(8);
        assert_eq!(
            capability_permitted_mask(&original) & transition,
            transition
        );
        let mut parked = original;
        set_effective_mask(
            &mut parked,
            capability_effective_mask(&original) & !transition,
        );
        write_capabilities(&parked).unwrap();

        let mut command = Command::new("/bin/false");
        command.env_clear();
        unsafe {
            command.pre_exec(move || {
                let _keep_environment_alive = &environment;
                let _keep_argv_alive = &argv;
                if libc::unshare(libc::CLONE_NEWNS) != 0
                    || libc::mount(
                        std::ptr::null(),
                        mount_root.as_ptr(),
                        std::ptr::null(),
                        libc::MS_REC | libc::MS_PRIVATE,
                        std::ptr::null(),
                    ) != 0
                    || libc::mount(
                        synthetic_root.as_ptr(),
                        synthetic_root.as_ptr(),
                        std::ptr::null(),
                        libc::MS_BIND | libc::MS_REC,
                        std::ptr::null(),
                    ) != 0
                    || libc::mount(
                        proc_source.as_ptr(),
                        synthetic_proc.as_ptr(),
                        proc_type.as_ptr(),
                        0,
                        std::ptr::null(),
                    ) != 0
                    || libc::mount(
                        cgroup_source.as_ptr(),
                        synthetic_cgroup.as_ptr(),
                        std::ptr::null(),
                        libc::MS_BIND | libc::MS_REC,
                        std::ptr::null(),
                    ) != 0
                    || libc::mount(
                        dev_source.as_ptr(),
                        synthetic_dev.as_ptr(),
                        std::ptr::null(),
                        libc::MS_BIND | libc::MS_REC,
                        std::ptr::null(),
                    ) != 0
                    || libc::syscall(
                        libc::SYS_pivot_root,
                        synthetic_root.as_ptr(),
                        synthetic_old_root.as_ptr(),
                    ) != 0
                    || libc::chdir(new_root.as_ptr()) != 0
                    || libc::umount2(old_root.as_ptr(), libc::MNT_DETACH) != 0
                    || libc::rmdir(old_root.as_ptr()) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                let mut header = CapabilityHeader {
                    version: LINUX_CAPABILITY_VERSION_3,
                    pid: 0,
                };
                let mut capabilities = [CapabilityData::default(); 2];
                if libc::syscall(libc::SYS_capget, &mut header, capabilities.as_mut_ptr()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let permitted = capability_permitted_mask(&capabilities);
                if permitted & transition != transition {
                    return Err(std::io::Error::other(
                        "wrapper child lacks parked transition capabilities",
                    ));
                }
                let effective = capability_effective_mask(&capabilities) | transition;
                set_effective_mask(&mut capabilities, effective);
                capabilities[0].inheritable |= transition as u32;
                capabilities[1].inheritable |= (transition >> 32) as u32;
                if libc::syscall(libc::SYS_capset, &mut header, capabilities.as_ptr()) != 0
                    || libc::prctl(libc::PR_SET_SECUREBITS, 0x03, 0, 0, 0) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                for capability in [CAP_SETGID, CAP_SETUID, 8] {
                    if libc::prctl(
                        libc::PR_CAP_AMBIENT,
                        libc::PR_CAP_AMBIENT_RAISE,
                        capability,
                        0,
                        0,
                    ) != 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                libc::syscall(
                    libc::SYS_execveat,
                    wrapper_fd,
                    empty.as_ptr(),
                    argv_pointers.as_ptr().cast::<*const libc::c_char>(),
                    environment_pointers.as_ptr().cast::<*const libc::c_char>(),
                    libc::AT_EMPTY_PATH,
                );
                Err(std::io::Error::last_os_error())
            });
        }
        let mut child = command.spawn().unwrap();
        let child_pid = child.id();
        std::fs::write(cgroup_path.join("cgroup.procs"), format!("{child_pid}\n")).unwrap();
        write_owned(probe_write, &probe_bytes);
        drop(probe_read);
        drop(setup_write);
        drop(child_setup);
        drop(start_read);
        drop(connected_write);
        drop(request_release_read);
        drop(result_write);
        drop(wrapper_file);

        let child_start = read_process_start_time(child_pid).unwrap();
        install_and_validate_e3_child_user_namespace_v1(
            &exclusion,
            &parent_setup,
            child_pid,
            child_start,
            &requirement,
            &enforcement.expected_process_cgroup,
        )
        .unwrap();
        assert_eq!(read_capabilities().unwrap(), parked);
        let setup_bytes = read_owned(setup_read);
        let security: E3ChildSecurityAttestationV1 =
            ConfigProjectionCodecV1::decode_canonical_json(&setup_bytes).unwrap();
        let mut wrong_hash = security.clone();
        wrong_hash.attestation_hash = "0".repeat(64);
        assert!(
            validate_child_security_attestation_v1(&exclusion, &enforcement, &wrong_hash).is_err()
        );
        let mut self_consistent_but_false = security.clone();
        self_consistent_but_false.cap_last_cap = if security.cap_last_cap == 63 {
            62
        } else {
            security.cap_last_cap + 1
        };
        self_consistent_but_false.attestation_hash = hash_omitting(
            "substrate.e3.child-security-attestation.v1",
            "attestation",
            &self_consistent_but_false,
            "attestation_hash",
        );
        assert!(validate_child_security_attestation_v1(
            &exclusion,
            &enforcement,
            &self_consistent_but_false,
        )
        .is_err());
        validate_child_security_attestation_v1(&exclusion, &enforcement, &security).unwrap();
        {
            let state = exclusion.lock_state().unwrap();
            assert!(state
                .child_user_namespaces
                .contains_key(&(child_pid, child_start)));
        }

        write_owned(start_write, &[0x01]);
        let connected_bytes = read_owned(connected_read);
        let connected: Value =
            ConfigProjectionCodecV1::decode_canonical_json(&connected_bytes).unwrap();
        assert_eq!(connected["probe_pid"], child_pid);
        assert_eq!(connected["probe_pid_start_time_ticks"], child_start);
        assert_eq!(connected["input_hash"], probe["input_hash"]);
        assert_eq!(
            connected["connected_hash"],
            hash_omitting(
                "substrate.e3.managed-gateway-readiness-probe-connected.v1",
                "connected",
                &connected,
                "connected_hash",
            )
        );
        let (mut stream, _) = listener.accept().unwrap();
        write_owned(request_release_write, &[0x01]);
        let mut request = Vec::new();
        let mut buffer = [0u8; 512];
        while !request.ends_with(b"\r\n\r\n") {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(
            request,
            format!(
                "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{listener_port}\r\nX-Substrate-E3-Readiness-Nonce: {readiness_nonce}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
            )
            .into_bytes()
        );
        let world_id = id("wld_");
        let orchestration_session_id = id("orch_");
        let retained_participant_id = id("part_");
        let response_body = ConfigProjectionCodecV1::encode_canonical_json(&json!({
            "backend_id": "api:openai",
            "config_projection_identity_hash": enforcement.projection_identity_hash,
            "gateway_ref": gateway_ref,
            "launch_input_hash": probe["launch_input_ref"]["launch_input_hash"],
            "listener_identity": listener_identity,
            "orchestration_session_id": orchestration_session_id,
            "readiness_nonce": readiness_nonce,
            "retained_participant_id": retained_participant_id,
            "schema_version": 1,
            "secret_handoff_consumed": true,
            "secret_handoff_prepared_ref": {
                "authority_store_id": probe["launch_input_ref"]["authority_store_id"],
                "handoff_id": id("hnd_"),
                "orchestration_session_id": orchestration_session_id,
                "retained_participant_id": retained_participant_id,
                "runtime_family": "codex",
                "world_id": world_id,
                "world_generation": 1,
                "receiving_gateway_identity_hash": probe["gateway_ref"]["gateway_identity_hash"],
                "handoff_state_revision": 1,
                "handoff_hash": "b".repeat(64),
            },
            "secret_ready_attestation_hash": "c".repeat(64),
            "world_generation": 1,
            "world_id": world_id,
        }))
        .unwrap();
        let response_header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response_body.len()
        );
        stream.write_all(response_header.as_bytes()).unwrap();
        stream.write_all(&response_body).unwrap();
        stream.shutdown(std::net::Shutdown::Both).unwrap();

        let result_bytes = read_owned(result_read);
        let result: Value = ConfigProjectionCodecV1::decode_canonical_json(&result_bytes).unwrap();
        assert_eq!(
            result["child_security_attestation"],
            serde_json::to_value(&security).unwrap()
        );
        assert_eq!(result["connected_hash"], connected["connected_hash"]);
        assert_eq!(
            result["result_hash"],
            hash_omitting(
                "substrate.e3.managed-gateway-readiness-probe-result.v1",
                "result",
                &result,
                "result_hash",
            )
        );
        let status = child.wait().unwrap();
        assert!(
            status.success(),
            "static readiness wrapper failed: {status}"
        );
        drop(e3);
        assert!(exclusion
            .lock_state()
            .unwrap()
            .child_user_namespaces
            .is_empty());
        std::fs::remove_dir(&cgroup_path).unwrap();
    }
}
