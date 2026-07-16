//! Session world implementation for Linux.

use crate::overlayfs::OverlayFs;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
#[cfg(feature = "test-support")]
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use world_api::{
    ExecResult, FsDiff, SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerSpec,
    WorldFsMode, WorldSpec,
};

const SESSION_METADATA_FILE_NAME: &str = "session.json";
const EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME: &str =
    ".session.json.exact-bound-world-adoption-v1.tmp";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactBoundWorldAdoptionPublicationPoint {
    AfterTrustedDirectoryBinding,
    BeforeTempPersistence,
    AfterTempWriteBeforeFileSync,
    AfterTempFileSync,
    BeforeRename,
    AfterRenameBeforeDirectorySync,
    AfterDirectorySyncBeforeResponse,
    BeforeExactFinalFileSync,
}

#[cfg(test)]
type ExactBoundWorldAdoptionPublicationFault = ExactBoundWorldAdoptionPublicationPoint;

#[cfg(feature = "test-support")]
fn shared_root_dir_override() -> &'static Mutex<Option<PathBuf>> {
    static OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[cfg(feature = "test-support")]
#[doc(hidden)]
pub struct SharedRootDirOverrideGuard {
    previous: Option<PathBuf>,
}

#[cfg(feature = "test-support")]
impl Drop for SharedRootDirOverrideGuard {
    fn drop(&mut self) {
        let mut slot = shared_root_dir_override()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *slot = self.previous.take();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
enum SessionWorldOwnerMode {
    #[default]
    Generic,
    SharedOrchestration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SessionWorldMetadata {
    world_id: String,
    project_dir: PathBuf,
    isolate_network: bool,
    always_isolate: bool,
    allowed_domains: Vec<String>,
    cgroup_path: PathBuf,
    started_at_unix_millis: u64,
    #[serde(default)]
    owner_mode: SessionWorldOwnerMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    orchestration_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    world_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    binding_state: Option<SharedWorldBindingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_snapshot_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exact_world_spec_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exact_adoption_policy_ref_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    world_fs_mode: Option<WorldFsMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_restart_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SessionWorldOwnership {
    Generic,
    Shared(SharedWorldBindingSnapshot),
}

/// A reusable Linux world with proper isolation.
pub struct SessionWorld {
    pub id: String,
    pub root_dir: PathBuf,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub net_namespace: Option<String>,
    pub spec: WorldSpec,
    pub started_at: SystemTime,
    pub network_filter: Option<crate::netfilter::NetFilter>,
    pub fs_by_span: HashMap<String, FsDiff>,
    shared_binding: Option<SharedWorldBindingSnapshot>,
    policy_snapshot_hash: Option<String>,
    exact_adoption_policy_ref_id: Option<String>,
    last_restart_reason: Option<String>,
    /// Persistent overlay mount for this session (writable or read-only).
    overlay: Option<OverlayFs>,
    overlay_mode: Option<WorldFsMode>,
}

struct OverlayExecutionContext<'a> {
    command_to_run: &'a str,
    cwd: &'a Path,
    env: &'a HashMap<String, String>,
    merged_dir: &'a Path,
    desired_cwd: &'a Path,
    require_cgroup_attach: bool,
    span_id: Option<&'a str>,
}

impl SessionWorld {
    fn default_shared_root_dir_for(
        uid: u32,
        xdg_runtime_dir: Option<&Path>,
        has_run_user_dir: bool,
    ) -> PathBuf {
        if let Some(xdg_runtime_dir) = xdg_runtime_dir.filter(|path| !path.as_os_str().is_empty()) {
            return xdg_runtime_dir.join("substrate").join("worlds");
        }

        if has_run_user_dir {
            return PathBuf::from(format!("/run/user/{uid}"))
                .join("substrate")
                .join("worlds");
        }

        PathBuf::from(format!("/tmp/substrate-worlds-{uid}"))
    }

    pub(crate) fn shared_root_dir() -> PathBuf {
        #[cfg(feature = "test-support")]
        if let Some(path) = shared_root_dir_override()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            return path;
        }

        let uid = current_uid();
        let xdg_runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
        let run_user_dir = PathBuf::from(format!("/run/user/{uid}"));
        Self::default_shared_root_dir_for(uid, xdg_runtime_dir.as_deref(), run_user_dir.is_dir())
    }

    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn override_shared_root_dir_for_tests(root_dir: PathBuf) -> SharedRootDirOverrideGuard {
        let mut slot = shared_root_dir_override()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = slot.replace(root_dir);
        SharedRootDirOverrideGuard { previous }
    }

    /// Ensure a session world is started and return it.
    pub fn ensure_started(spec: WorldSpec) -> Result<Self> {
        Self::ensure_started_in_root(spec, Self::shared_root_dir())
    }

    pub(crate) fn ensure_started_in_root(spec: WorldSpec, root_dir: PathBuf) -> Result<Self> {
        Self::ensure_started_with_binding_at_root(spec, None, None, root_dir, None)
    }

    pub(crate) fn ensure_started_for_shared_owner_at_root(
        root_dir: PathBuf,
        spec: WorldSpec,
        orchestration_session_id: String,
        world_generation: u64,
        last_restart_reason: Option<String>,
    ) -> Result<Self> {
        let shared_binding = SharedWorldBindingSnapshot {
            orchestration_session_id,
            world_id: String::new(),
            world_generation,
            binding_state: SharedWorldBindingState::Active,
        };
        Self::ensure_started_with_binding_at_root(
            spec,
            Some(shared_binding),
            last_restart_reason,
            root_dir,
            None,
        )
    }

    pub(crate) fn ensure_started_for_shared_owner_at_root_with_world_id(
        root_dir: PathBuf,
        spec: WorldSpec,
        orchestration_session_id: String,
        world_generation: u64,
        last_restart_reason: Option<String>,
        world_id: String,
    ) -> Result<Self> {
        let shared_binding = SharedWorldBindingSnapshot {
            orchestration_session_id,
            world_id: String::new(),
            world_generation,
            binding_state: SharedWorldBindingState::Active,
        };
        Self::ensure_started_with_binding_at_root(
            spec,
            Some(shared_binding),
            last_restart_reason,
            root_dir,
            Some(world_id),
        )
    }

    fn ensure_started_with_binding_at_root(
        spec: WorldSpec,
        world_binding: Option<SharedWorldBindingSnapshot>,
        last_restart_reason: Option<String>,
        root_dir: PathBuf,
        world_id: Option<String>,
    ) -> Result<Self> {
        let world_id = world_id.unwrap_or_else(|| format!("wld_{}", uuid::Uuid::now_v7()));
        let shared_binding = match world_binding {
            Some(mut binding) => {
                binding.world_id = world_id.clone();
                Some(binding)
            }
            None => None,
        };
        let mut world = Self {
            id: world_id.clone(),
            root_dir,
            project_dir: spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate").join(&world_id),
            net_namespace: None,
            spec,
            started_at: SystemTime::now(),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason,
            overlay: None,
            overlay_mode: None,
        };

        world.setup()?;
        if world.should_persist_metadata() {
            world.persist_metadata()?;
        }
        Ok(world)
    }

    /// Determine whether this world can be reused for the requested spec.
    pub(crate) fn compatible_with(&self, spec: &WorldSpec) -> bool {
        self.project_dir == spec.project_dir
            && self.spec.isolate_network == spec.isolate_network
            && self.spec.always_isolate == spec.always_isolate
            && self.spec.allowed_domains == spec.allowed_domains
    }

    pub(crate) fn is_generic_reusable_with(&self, spec: &WorldSpec) -> bool {
        self.shared_binding.is_none() && self.compatible_with(spec)
    }

    pub(crate) fn is_shared_owner_reusable_with(
        &self,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
    ) -> bool {
        self.compatible_with(spec)
            && self.shared_binding.as_ref().is_some_and(|binding| {
                binding.binding_state == SharedWorldBindingState::Active
                    && binding.orchestration_session_id == owner_spec.orchestration_session_id
            })
    }

    pub(crate) fn shared_binding(&self) -> Option<SharedWorldBindingSnapshot> {
        self.shared_binding.clone()
    }

    pub(crate) fn set_shared_binding_state(
        &mut self,
        binding_state: SharedWorldBindingState,
        last_restart_reason: Option<String>,
    ) -> Result<()> {
        let binding = self
            .shared_binding
            .as_mut()
            .ok_or_else(|| anyhow!("shared binding missing for world {}", self.id))?;

        let current = binding.binding_state.clone();
        let transition_allowed = matches!(
            (&current, &binding_state),
            (
                SharedWorldBindingState::Active,
                SharedWorldBindingState::Replacing
            ) | (
                SharedWorldBindingState::Replacing,
                SharedWorldBindingState::Active
            ) | (
                SharedWorldBindingState::Replacing,
                SharedWorldBindingState::Replaced
            )
        );
        if !transition_allowed {
            anyhow::bail!(
                "invalid shared binding state transition for world {}: {:?} -> {:?}",
                self.id,
                current,
                binding_state
            );
        }

        binding.binding_state = binding_state;
        self.last_restart_reason = last_restart_reason;
        self.persist_metadata()
    }

    pub(crate) fn recover_generic_compatible_from_root(
        root_dir: &Path,
        spec: &WorldSpec,
    ) -> Result<Option<Self>> {
        if !spec.reuse_session || !root_dir.is_dir() {
            return Ok(None);
        }

        Self::recover_first_from_root(root_dir, |metadata_path| {
            Self::recover_generic_from_metadata_path(metadata_path, root_dir, spec)
        })
    }

    pub(crate) fn recover_exact_bound_world_from_root(
        root_dir: &Path,
        spec: &WorldSpec,
        world_id: &str,
    ) -> Result<Option<Self>> {
        if !root_dir.is_dir() {
            return Ok(None);
        }
        let metadata_path = Self::metadata_path(root_dir, world_id);
        if !metadata_path.is_file() {
            return Ok(None);
        }
        let metadata = Self::read_canonical_metadata(&metadata_path)?;
        if metadata.world_id != world_id {
            anyhow::bail!("exact bound-world metadata identity mismatch");
        }
        if !Self::metadata_exactly_matches_physical_spec(&metadata, spec) {
            anyhow::bail!("exact bound-world metadata does not match requested physical spec");
        }
        if !Self::metadata_is_usable(&metadata, &metadata_path) {
            anyhow::bail!("exact bound-world metadata is not usable");
        }
        let shared_binding = match Self::ownership_from_metadata(&metadata)? {
            SessionWorldOwnership::Generic => None,
            SessionWorldOwnership::Shared(binding) => Some(binding),
        };
        let world = Self::from_metadata(root_dir, spec, metadata, shared_binding);
        world
            .ensure_cgroup_attach_target()
            .with_context(|| format!("failed to prepare exact bound world {}", world.id))?;
        Ok(Some(world))
    }

    #[cfg(test)]
    pub(crate) fn recover_compatible_from_root(
        root_dir: &Path,
        spec: &WorldSpec,
    ) -> Result<Option<Self>> {
        Self::recover_generic_compatible_from_root(root_dir, spec)
    }

    pub(crate) fn recover_shared_active_from_root(
        root_dir: &Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
    ) -> Result<Option<Self>> {
        if !root_dir.is_dir() {
            return Ok(None);
        }

        let mut active_worlds = Vec::new();
        let mut replacing_worlds = Vec::new();
        let Some(entries) = read_session_root_dir(root_dir)? else {
            return Ok(None);
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::warn!(error = %err, root = %root_dir.display(), "failed to inspect session metadata entry");
                    continue;
                }
            };
            let metadata_path = entry.path().join(SESSION_METADATA_FILE_NAME);
            if !metadata_path.is_file() {
                continue;
            }

            match Self::recover_shared_from_metadata_path(
                &metadata_path,
                root_dir,
                spec,
                owner_spec,
            ) {
                Ok(Some(world)) => match world
                    .shared_binding
                    .as_ref()
                    .map(|binding| binding.binding_state.clone())
                {
                    Some(SharedWorldBindingState::Active) => active_worlds.push(world),
                    Some(SharedWorldBindingState::Replacing) => replacing_worlds.push(world),
                    Some(
                        SharedWorldBindingState::Replaced | SharedWorldBindingState::Abandoned,
                    ) => {}
                    None => tracing::warn!(
                        metadata = %metadata_path.display(),
                        "shared recovery candidate missing binding proof after parsing"
                    ),
                },
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        metadata = %metadata_path.display(),
                        "ignoring invalid shared session metadata without deleting it"
                    );
                }
            }
        }

        if active_worlds.len() > 1 {
            anyhow::bail!(
                "multiple active shared worlds found for orchestration session {}",
                owner_spec.orchestration_session_id
            );
        }

        if let Some(active_world) = active_worlds.pop() {
            let active_generation = active_world
                .shared_binding
                .as_ref()
                .map(|binding| binding.world_generation)
                .ok_or_else(|| anyhow!("active shared world missing binding proof"))?;
            if replacing_worlds.iter().any(|world| {
                world
                    .shared_binding
                    .as_ref()
                    .is_some_and(|binding| binding.world_generation >= active_generation)
            }) {
                anyhow::bail!(
                    "ambiguous shared world recovery state for orchestration session {}",
                    owner_spec.orchestration_session_id
                );
            }
            return Ok(Some(active_world));
        }

        match replacing_worlds.len() {
            0 => Ok(None),
            1 => {
                let mut world = replacing_worlds
                    .pop()
                    .ok_or_else(|| anyhow!("replacing shared world missing from recovery set"))?;
                world.set_shared_binding_state(SharedWorldBindingState::Active, None)?;
                Ok(Some(world))
            }
            _ => anyhow::bail!(
                "ambiguous replacing shared worlds found for orchestration session {}",
                owner_spec.orchestration_session_id
            ),
        }
    }

    fn recover_first_from_root<F>(root_dir: &Path, mut recover: F) -> Result<Option<Self>>
    where
        F: FnMut(&Path) -> Result<Option<Self>>,
    {
        let Some(entries) = read_session_root_dir(root_dir)? else {
            return Ok(None);
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::warn!(error = %err, root = %root_dir.display(), "failed to inspect session metadata entry");
                    continue;
                }
            };
            let metadata_path = entry.path().join(SESSION_METADATA_FILE_NAME);
            if !metadata_path.is_file() {
                continue;
            }

            match recover(&metadata_path) {
                Ok(Some(world)) => return Ok(Some(world)),
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        metadata = %metadata_path.display(),
                        "ignoring invalid persisted session metadata"
                    );
                    let _ = fs::remove_file(&metadata_path);
                }
            }
        }

        Ok(None)
    }

    fn recover_generic_from_metadata_path(
        metadata_path: &Path,
        root_dir: &Path,
        spec: &WorldSpec,
    ) -> Result<Option<Self>> {
        let metadata = Self::read_metadata(metadata_path)?;
        if !Self::metadata_matches_spec(&metadata, spec) {
            return Ok(None);
        }
        if !Self::metadata_is_usable(&metadata, metadata_path) {
            return Ok(None);
        }
        match Self::ownership_from_metadata(&metadata)? {
            SessionWorldOwnership::Generic => {
                let world = Self::from_metadata(root_dir, spec, metadata, None);
                world.ensure_cgroup_attach_target().with_context(|| {
                    format!("failed to prepare cgroup attach target for {}", world.id)
                })?;
                Ok(Some(world))
            }
            SessionWorldOwnership::Shared(_) => Ok(None),
        }
    }

    fn recover_shared_from_metadata_path(
        metadata_path: &Path,
        root_dir: &Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
    ) -> Result<Option<Self>> {
        let metadata = Self::read_metadata(metadata_path)?;
        if !Self::metadata_matches_spec(&metadata, spec) {
            return Ok(None);
        }
        if !Self::metadata_is_usable(&metadata, metadata_path) {
            return Ok(None);
        }
        match Self::ownership_from_metadata(&metadata)? {
            SessionWorldOwnership::Generic => Ok(None),
            SessionWorldOwnership::Shared(binding)
                if binding.orchestration_session_id == owner_spec.orchestration_session_id =>
            {
                let world = Self::from_metadata(root_dir, spec, metadata, Some(binding));
                world.ensure_cgroup_attach_target().with_context(|| {
                    format!("failed to prepare cgroup attach target for {}", world.id)
                })?;
                Ok(Some(world))
            }
            SessionWorldOwnership::Shared(_) => Ok(None),
        }
    }

    fn metadata_matches_spec(metadata: &SessionWorldMetadata, spec: &WorldSpec) -> bool {
        metadata.project_dir == spec.project_dir
            && metadata.isolate_network == spec.isolate_network
            && metadata.always_isolate == spec.always_isolate
            && metadata.allowed_domains == spec.allowed_domains
    }

    fn metadata_exactly_matches_physical_spec(
        metadata: &SessionWorldMetadata,
        spec: &WorldSpec,
    ) -> bool {
        let Ok(commitment) = Self::exact_world_spec_commitment(spec) else {
            return false;
        };
        Self::metadata_matches_spec(metadata, spec)
            && metadata.world_fs_mode == Some(spec.fs_mode)
            && metadata.exact_world_spec_commitment.as_deref() == Some(commitment.as_str())
    }

    fn exact_world_spec_commitment(spec: &WorldSpec) -> Result<String> {
        let canonical_project = fs::canonicalize(&spec.project_dir).with_context(|| {
            format!(
                "failed to resolve exact world project {}",
                spec.project_dir.display()
            )
        })?;
        let project_metadata = fs::metadata(&canonical_project).with_context(|| {
            format!(
                "failed to inspect exact world project {}",
                canonical_project.display()
            )
        })?;
        if !project_metadata.is_dir() {
            anyhow::bail!("exact world project identity is not a directory");
        }
        if !spec.project_dir.is_absolute() || spec.project_dir != canonical_project {
            anyhow::bail!("exact world project identity must be a canonical absolute directory");
        }
        #[cfg(unix)]
        let (project_device, project_inode) = {
            use std::os::unix::fs::MetadataExt;
            (project_metadata.dev(), project_metadata.ino())
        };
        #[cfg(not(unix))]
        let (project_device, project_inode) = (0_u64, 0_u64);

        let canonical = serde_json::json!({
            "reuse_session": spec.reuse_session,
            "isolate_network": spec.isolate_network,
            "limits": spec.limits,
            "enable_preload": spec.enable_preload,
            "allowed_domains": spec.allowed_domains,
            "project": {
                "canonical_path": canonical_project,
                "device": project_device,
                "inode": project_inode,
            },
            "always_isolate": spec.always_isolate,
            "fs_mode": spec.fs_mode,
            "backend_policy": spec.backend_policy,
        });
        let bytes = serde_json::to_vec(&canonical)
            .context("failed to canonicalize exact world physical spec")?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn current_spec_exactly_matches_target(&self, target: &WorldSpec) -> bool {
        self.spec.reuse_session == target.reuse_session
            && self.spec.isolate_network == target.isolate_network
            && self.spec.limits.cpu == target.limits.cpu
            && self.spec.limits.memory == target.limits.memory
            && self.spec.enable_preload == target.enable_preload
            && self.spec.allowed_domains == target.allowed_domains
            && self.spec.project_dir == target.project_dir
            && self.spec.always_isolate == target.always_isolate
            && self.spec.fs_mode == target.fs_mode
            && self.spec.backend_policy == target.backend_policy
    }

    fn metadata_is_usable(metadata: &SessionWorldMetadata, metadata_path: &Path) -> bool {
        metadata_path.parent().is_some_and(Path::is_dir)
            && metadata.project_dir.is_dir()
            && metadata.cgroup_path.is_dir()
    }

    fn ownership_from_metadata(metadata: &SessionWorldMetadata) -> Result<SessionWorldOwnership> {
        let owner_fields_present = metadata.orchestration_session_id.is_some()
            || metadata.world_generation.is_some()
            || metadata.binding_state.is_some();
        match metadata.owner_mode {
            SessionWorldOwnerMode::Generic => {
                if owner_fields_present {
                    Err(anyhow!(
                        "generic session metadata {} includes shared owner fields",
                        metadata.world_id
                    ))
                } else {
                    Ok(SessionWorldOwnership::Generic)
                }
            }
            SessionWorldOwnerMode::SharedOrchestration => {
                let orchestration_session_id = metadata
                    .orchestration_session_id
                    .clone()
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| {
                        anyhow!(
                            "shared session metadata {} missing orchestration_session_id",
                            metadata.world_id
                        )
                    })?;
                let world_generation = metadata.world_generation.ok_or_else(|| {
                    anyhow!(
                        "shared session metadata {} missing world_generation",
                        metadata.world_id
                    )
                })?;
                let binding_state = metadata.binding_state.clone().ok_or_else(|| {
                    anyhow!(
                        "shared session metadata {} missing binding_state",
                        metadata.world_id
                    )
                })?;
                Ok(SessionWorldOwnership::Shared(SharedWorldBindingSnapshot {
                    orchestration_session_id,
                    world_id: metadata.world_id.clone(),
                    world_generation,
                    binding_state,
                }))
            }
        }
    }

    fn from_metadata(
        root_dir: &Path,
        spec: &WorldSpec,
        metadata: SessionWorldMetadata,
        shared_binding: Option<SharedWorldBindingSnapshot>,
    ) -> Self {
        Self {
            id: metadata.world_id,
            root_dir: root_dir.to_path_buf(),
            project_dir: metadata.project_dir,
            cgroup_path: metadata.cgroup_path,
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(metadata.started_at_unix_millis),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding,
            policy_snapshot_hash: metadata.policy_snapshot_hash,
            exact_adoption_policy_ref_id: metadata.exact_adoption_policy_ref_id,
            last_restart_reason: metadata.last_restart_reason,
            overlay: None,
            overlay_mode: None,
        }
    }

    fn metadata_dir(root_dir: &Path, world_id: &str) -> PathBuf {
        root_dir.join(world_id)
    }

    fn metadata_path(root_dir: &Path, world_id: &str) -> PathBuf {
        Self::metadata_dir(root_dir, world_id).join(SESSION_METADATA_FILE_NAME)
    }

    fn to_metadata(&self) -> Result<SessionWorldMetadata> {
        let started_at_unix_millis = self
            .started_at
            .duration_since(UNIX_EPOCH)
            .context("session start time predates unix epoch")?
            .as_millis()
            .try_into()
            .context("session start time exceeds u64 millis")?;
        let (owner_mode, orchestration_session_id, world_generation, binding_state) =
            match self.shared_binding.as_ref() {
                Some(binding) => (
                    SessionWorldOwnerMode::SharedOrchestration,
                    Some(binding.orchestration_session_id.clone()),
                    Some(binding.world_generation),
                    Some(binding.binding_state.clone()),
                ),
                None => (SessionWorldOwnerMode::Generic, None, None, None),
            };
        let exact_world_spec_commitment = Self::exact_world_spec_commitment(&self.spec).ok();
        if self.exact_adoption_policy_ref_id.is_some() && exact_world_spec_commitment.is_none() {
            anyhow::bail!("exact adopted world physical spec can no longer be proven");
        }
        Ok(SessionWorldMetadata {
            world_id: self.id.clone(),
            project_dir: self.project_dir.clone(),
            isolate_network: self.spec.isolate_network,
            always_isolate: self.spec.always_isolate,
            allowed_domains: self.spec.allowed_domains.clone(),
            cgroup_path: self.cgroup_path.clone(),
            started_at_unix_millis,
            owner_mode,
            orchestration_session_id,
            world_generation,
            binding_state,
            policy_snapshot_hash: self.policy_snapshot_hash.clone(),
            exact_world_spec_commitment,
            exact_adoption_policy_ref_id: self.exact_adoption_policy_ref_id.clone(),
            world_fs_mode: Some(self.spec.fs_mode),
            last_restart_reason: self.last_restart_reason.clone(),
        })
    }

    pub(crate) fn persist_metadata(&self) -> Result<()> {
        let metadata_dir = Self::metadata_dir(&self.root_dir, &self.id);
        fs::create_dir_all(&metadata_dir)
            .with_context(|| format!("failed to create {}", metadata_dir.display()))?;
        let metadata_path = Self::metadata_path(&self.root_dir, &self.id);
        let temp_path = metadata_dir.join(format!(
            ".{}.{}.tmp",
            SESSION_METADATA_FILE_NAME,
            uuid::Uuid::now_v7()
        ));
        let metadata = self.to_metadata()?;
        let mut temp_file = match fs::File::options()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(file) => file,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to create {}", temp_path.display()));
            }
        };

        if let Err(err) = serde_json::to_writer_pretty(&mut temp_file, &metadata)
            .with_context(|| format!("failed to serialize {}", metadata_path.display()))
        {
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }

        if let Err(err) = temp_file
            .sync_all()
            .with_context(|| format!("failed to flush {}", temp_path.display()))
        {
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }

        if let Err(err) = fs::rename(&temp_path, &metadata_path)
            .with_context(|| format!("failed to persist {}", metadata_path.display()))
        {
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }

        #[cfg(unix)]
        if let Some(parent) = metadata_path.parent() {
            match fs::File::open(parent).and_then(|dir| dir.sync_all()) {
                Ok(()) => {}
                Err(err) => tracing::warn!(
                    error = %err,
                    path = %parent.display(),
                    "failed to sync metadata directory after atomic persist"
                ),
            }
        }

        Ok(())
    }

    pub(crate) fn adopt_exact_bound_world_ownership(
        &mut self,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
    ) -> Result<()> {
        self.adopt_exact_bound_world_ownership_impl(adoption, None)
    }

    #[cfg(test)]
    fn adopt_exact_bound_world_ownership_with_fault(
        &mut self,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
        fault: Option<ExactBoundWorldAdoptionPublicationFault>,
    ) -> Result<()> {
        self.adopt_exact_bound_world_ownership_impl(adoption, fault)
    }

    fn adopt_exact_bound_world_ownership_impl(
        &mut self,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
        fault: Option<ExactBoundWorldAdoptionPublicationPoint>,
    ) -> Result<()> {
        self.adopt_exact_bound_world_ownership_unified(adoption, |point| fault == Some(point))
    }

    fn adopt_exact_bound_world_ownership_unified<F>(
        &mut self,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
        mut should_fail: F,
    ) -> Result<()>
    where
        F: FnMut(ExactBoundWorldAdoptionPublicationPoint) -> bool,
    {
        if self.id != adoption.world_id {
            anyhow::bail!("exact bound-world adoption target identity mismatch");
        }
        if !self.current_spec_exactly_matches_target(&adoption.target_spec) {
            anyhow::bail!("exact bound-world adoption current realization spec conflict");
        }
        let metadata_dir = Self::metadata_dir(&self.root_dir, &self.id);
        let metadata_path = Self::metadata_path(&self.root_dir, &self.id);
        if !Self::is_real_directory(&self.root_dir)
            || !Self::is_real_directory(&metadata_dir)
            || !Self::is_real_file(&metadata_path)
        {
            anyhow::bail!("exact bound-world adoption metadata is missing");
        }
        let durable_root = Self::lock_shared_root_for_ownership(&self.root_dir)?;
        let durable_world = Self::open_bound_world_directory(&durable_root, &self.id)?;
        Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
        Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::AfterTrustedDirectoryBinding) {
            anyhow::bail!("injected adoption publication crash after directory binding");
        }
        Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
        Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
        let durable_root_path = Self::descriptor_directory_path(&durable_root)?;
        let durable_metadata_dir = Self::descriptor_directory_path(&durable_world)?;
        let durable_metadata_path = durable_metadata_dir.join(SESSION_METADATA_FILE_NAME);
        let durable_temp_path =
            durable_metadata_dir.join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME);
        Self::reject_unrecognized_metadata_temps(&durable_metadata_dir)?;
        Self::validate_no_other_shared_owner(&durable_root_path, adoption)?;

        let current = Self::read_canonical_metadata(&durable_metadata_path)?;
        let desired = Self::exact_bound_world_adoption_metadata(&current, adoption)?;

        if durable_temp_path.exists() {
            if !Self::is_real_file(&durable_temp_path) {
                anyhow::bail!("invalid exact bound-world adoption temp evidence");
            }
            let temp_bytes = Self::read_metadata_bytes_at(
                &durable_world,
                EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
                &durable_temp_path,
            )?;
            match serde_json::from_slice::<SessionWorldMetadata>(&temp_bytes) {
                Ok(temp_metadata) => {
                    if serde_json::to_vec_pretty(&temp_metadata)
                        .context("failed to canonicalize exact adoption temp metadata")?
                        != temp_bytes
                    {
                        anyhow::bail!("noncanonical exact bound-world adoption temp evidence");
                    }
                    if temp_metadata != desired {
                        anyhow::bail!("conflicting exact bound-world adoption temp evidence");
                    }
                    Self::open_real_metadata_file_at(
                        &durable_world,
                        EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
                        &durable_temp_path,
                        true,
                    )?
                    .sync_all()
                    .with_context(|| format!("failed to flush {}", durable_temp_path.display()))?;

                    let final_metadata = Self::read_canonical_metadata(&durable_metadata_path)?;
                    if final_metadata == desired {
                        if should_fail(
                            ExactBoundWorldAdoptionPublicationPoint::BeforeExactFinalFileSync,
                        ) {
                            anyhow::bail!(
                                "injected adoption publication crash before exact final fsync"
                            );
                        }
                        Self::sync_metadata_file_at(
                            &durable_world,
                            SESSION_METADATA_FILE_NAME,
                            &durable_metadata_path,
                        )?;
                        Self::unlink_metadata_at(
                            &durable_world,
                            EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
                            &durable_temp_path,
                        )?;
                        Self::sync_directory_handle(&durable_world, &durable_metadata_dir)?;
                        Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
                        Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
                        self.apply_exact_adopted_metadata(adoption, desired)?;
                        return Ok(());
                    }
                    let expected_from_final =
                        Self::exact_bound_world_adoption_metadata(&final_metadata, adoption)?;
                    if expected_from_final != temp_metadata {
                        anyhow::bail!("conflicting exact bound-world adoption temp/final evidence");
                    }
                    Self::rename_metadata_at(
                        &durable_world,
                        EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
                        SESSION_METADATA_FILE_NAME,
                        &durable_metadata_path,
                    )?;
                    Self::sync_metadata_file_at(
                        &durable_world,
                        SESSION_METADATA_FILE_NAME,
                        &durable_metadata_path,
                    )?;
                    Self::sync_directory_handle(&durable_world, &durable_metadata_dir)?;
                    Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
                    Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
                    self.apply_exact_adopted_metadata(adoption, desired)?;
                    return Ok(());
                }
                Err(error) if error.is_eof() => {
                    Self::unlink_metadata_at(
                        &durable_world,
                        EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
                        &durable_temp_path,
                    )?;
                    Self::sync_directory_handle(&durable_world, &durable_metadata_dir)?;
                }
                Err(_) => {
                    anyhow::bail!("corrupt exact bound-world adoption temp evidence");
                }
            }
        }

        let current = Self::read_canonical_metadata(&durable_metadata_path)?;
        let desired = Self::exact_bound_world_adoption_metadata(&current, adoption)?;
        if current == desired {
            if should_fail(ExactBoundWorldAdoptionPublicationPoint::BeforeExactFinalFileSync) {
                anyhow::bail!("injected adoption publication crash before exact final fsync");
            }
            Self::sync_metadata_file_at(
                &durable_world,
                SESSION_METADATA_FILE_NAME,
                &durable_metadata_path,
            )?;
            Self::sync_directory_handle(&durable_world, &durable_metadata_dir)?;
            Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
            Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
            self.apply_exact_adopted_metadata(adoption, desired)?;
            return Ok(());
        }

        if should_fail(ExactBoundWorldAdoptionPublicationPoint::BeforeTempPersistence) {
            anyhow::bail!("injected adoption publication crash before temp persistence");
        }
        let mut temp_file = Self::create_metadata_file_at(
            &durable_world,
            EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
            &durable_temp_path,
        )?;
        serde_json::to_writer_pretty(&mut temp_file, &desired)
            .with_context(|| format!("failed to serialize {}", durable_temp_path.display()))?;
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::AfterTempWriteBeforeFileSync) {
            anyhow::bail!("injected adoption publication crash after temp write");
        }
        temp_file
            .sync_all()
            .with_context(|| format!("failed to flush {}", durable_temp_path.display()))?;
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::AfterTempFileSync) {
            anyhow::bail!("injected adoption publication crash after temp fsync");
        }
        drop(temp_file);
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::BeforeRename) {
            anyhow::bail!("injected adoption publication crash before rename");
        }
        Self::rename_metadata_at(
            &durable_world,
            EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME,
            SESSION_METADATA_FILE_NAME,
            &durable_metadata_path,
        )?;
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::AfterRenameBeforeDirectorySync) {
            anyhow::bail!("injected adoption publication crash after rename");
        }
        Self::sync_metadata_file_at(
            &durable_world,
            SESSION_METADATA_FILE_NAME,
            &durable_metadata_path,
        )?;
        Self::sync_directory_handle(&durable_world, &durable_metadata_dir)?;
        if should_fail(ExactBoundWorldAdoptionPublicationPoint::AfterDirectorySyncBeforeResponse) {
            anyhow::bail!("injected adoption publication crash after directory fsync");
        }
        Self::validate_bound_directory_identity(&self.root_dir, &durable_root)?;
        Self::validate_bound_directory_identity(&metadata_dir, &durable_world)?;
        self.apply_exact_adopted_metadata(adoption, desired)
    }

    fn exact_bound_world_adoption_metadata(
        current: &SessionWorldMetadata,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
    ) -> Result<SessionWorldMetadata> {
        if current.world_id != adoption.world_id
            || !Self::metadata_exactly_matches_physical_spec(current, &adoption.target_spec)
        {
            anyhow::bail!("exact bound-world adoption physical metadata conflict");
        }

        match Self::ownership_from_metadata(current)? {
            SessionWorldOwnership::Generic => {
                if current.exact_adoption_policy_ref_id.is_some() {
                    anyhow::bail!("generic exact bound-world metadata includes adoption policy");
                }
                if current
                    .policy_snapshot_hash
                    .as_ref()
                    .is_some_and(|hash| hash != &adoption.policy_snapshot_hash)
                {
                    anyhow::bail!("exact bound-world adoption policy conflict");
                }
                let mut desired = current.clone();
                desired.owner_mode = SessionWorldOwnerMode::SharedOrchestration;
                desired.orchestration_session_id = Some(adoption.orchestration_session_id.clone());
                desired.world_generation = Some(adoption.world_generation);
                desired.binding_state = Some(SharedWorldBindingState::Active);
                desired.policy_snapshot_hash = Some(adoption.policy_snapshot_hash.clone());
                desired.exact_adoption_policy_ref_id = Some(adoption.policy_ref_id.clone());
                desired.last_restart_reason = None;
                Ok(desired)
            }
            SessionWorldOwnership::Shared(binding) => {
                if binding.orchestration_session_id != adoption.orchestration_session_id
                    || binding.world_id != adoption.world_id
                    || binding.world_generation != adoption.world_generation
                    || binding.binding_state != SharedWorldBindingState::Active
                    || current.policy_snapshot_hash.as_deref()
                        != Some(adoption.policy_snapshot_hash.as_str())
                    || current.exact_adoption_policy_ref_id.as_deref()
                        != Some(adoption.policy_ref_id.as_str())
                {
                    anyhow::bail!("exact bound-world adoption ownership conflict");
                }
                Ok(current.clone())
            }
        }
    }

    fn apply_exact_adopted_metadata(
        &mut self,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
        metadata: SessionWorldMetadata,
    ) -> Result<()> {
        if !Self::metadata_exactly_matches_physical_spec(&metadata, &adoption.target_spec) {
            anyhow::bail!("exact bound-world physical spec changed during ownership publication");
        }
        let binding = match Self::ownership_from_metadata(&metadata)? {
            SessionWorldOwnership::Shared(binding) => binding,
            SessionWorldOwnership::Generic => {
                anyhow::bail!("exact bound-world adoption did not publish shared ownership")
            }
        };
        self.shared_binding = Some(binding);
        self.policy_snapshot_hash = metadata.policy_snapshot_hash;
        self.exact_adoption_policy_ref_id = metadata.exact_adoption_policy_ref_id;
        self.last_restart_reason = metadata.last_restart_reason;
        self.spec = adoption.target_spec.clone();
        Ok(())
    }

    fn read_canonical_metadata(path: &Path) -> Result<SessionWorldMetadata> {
        let mut file = Self::open_real_metadata_file(path, false)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let metadata: SessionWorldMetadata = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let canonical = serde_json::to_vec_pretty(&metadata)
            .with_context(|| format!("failed to canonicalize {}", path.display()))?;
        if bytes != canonical {
            anyhow::bail!("noncanonical session metadata {}", path.display());
        }
        Ok(metadata)
    }

    fn reject_unrecognized_metadata_temps(metadata_dir: &Path) -> Result<()> {
        for entry in fs::read_dir(metadata_dir)
            .with_context(|| format!("failed to inspect {}", metadata_dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to inspect {}", metadata_dir.display()))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&format!(".{SESSION_METADATA_FILE_NAME}."))
                && name.ends_with(".tmp")
                && name != EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME
            {
                anyhow::bail!("ambiguous session metadata publication evidence");
            }
        }
        Ok(())
    }

    fn validate_no_other_shared_owner(
        root_dir: &Path,
        adoption: &crate::ExactBoundWorldOwnershipAdoptionV1,
    ) -> Result<()> {
        let entries = fs::read_dir(root_dir)
            .with_context(|| format!("failed to inspect session root {}", root_dir.display()))?;
        for entry in entries {
            let entry = entry.with_context(|| {
                format!("failed to inspect session root {}", root_dir.display())
            })?;
            let path = entry.path();
            if path
                .file_name()
                .is_some_and(|name| name == adoption.world_id.as_str())
            {
                continue;
            }
            if !path.exists() {
                continue;
            }
            if !Self::is_real_directory(&path) {
                anyhow::bail!("ambiguous non-directory session-root evidence");
            }
            Self::reject_unrecognized_metadata_temps(&path)?;
            let metadata_path = path.join(SESSION_METADATA_FILE_NAME);
            if metadata_path.exists() {
                let metadata = Self::read_canonical_metadata(&metadata_path)?;
                if let SessionWorldOwnership::Shared(binding) =
                    Self::ownership_from_metadata(&metadata)?
                {
                    if binding.orchestration_session_id == adoption.orchestration_session_id {
                        anyhow::bail!(
                            "orchestration session already has a different shared world owner"
                        );
                    }
                }
            }
            let adoption_temp = path.join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME);
            if adoption_temp.exists() {
                let temp = Self::read_canonical_metadata(&adoption_temp).map_err(|_| {
                    anyhow!("ambiguous exact bound-world adoption evidence in session root")
                })?;
                if let SessionWorldOwnership::Shared(binding) =
                    Self::ownership_from_metadata(&temp)?
                {
                    if binding.orchestration_session_id == adoption.orchestration_session_id {
                        anyhow::bail!(
                            "orchestration session has a pending different shared world owner"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn open_bound_world_directory(root: &fs::File, world_id: &str) -> Result<fs::File> {
        use nix::fcntl::{openat, OFlag};
        use nix::sys::stat::Mode;
        use std::os::fd::{AsRawFd, FromRawFd};

        let fd = openat(
            root.as_raw_fd(),
            world_id,
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::empty(),
        )
        .with_context(|| format!("failed to bind exact world directory {world_id}"))?;
        // SAFETY: `openat` returned a fresh owned descriptor and ownership transfers to `File`.
        Ok(unsafe { fs::File::from_raw_fd(fd) })
    }

    #[cfg(not(target_os = "linux"))]
    fn open_bound_world_directory(_root: &fs::File, _world_id: &str) -> Result<fs::File> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor-relative operations")
    }

    #[cfg(target_os = "linux")]
    fn descriptor_directory_path(directory: &fs::File) -> Result<PathBuf> {
        use std::os::fd::AsRawFd;

        let path = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
        if !fs::metadata(&path)
            .with_context(|| format!("failed to inspect bound directory {}", path.display()))?
            .is_dir()
        {
            anyhow::bail!("bound metadata descriptor is not a directory");
        }
        Ok(path)
    }

    #[cfg(not(target_os = "linux"))]
    fn descriptor_directory_path(_directory: &fs::File) -> Result<PathBuf> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor paths")
    }

    #[cfg(unix)]
    fn validate_bound_directory_identity(path: &Path, directory: &fs::File) -> Result<()> {
        use std::os::unix::fs::MetadataExt;

        let path_metadata = fs::symlink_metadata(path)
            .with_context(|| format!("failed to revalidate bound directory {}", path.display()))?;
        let descriptor_metadata = directory
            .metadata()
            .with_context(|| format!("failed to inspect bound directory {}", path.display()))?;
        if path_metadata.file_type().is_symlink()
            || !path_metadata.is_dir()
            || !descriptor_metadata.is_dir()
            || path_metadata.dev() != descriptor_metadata.dev()
            || path_metadata.ino() != descriptor_metadata.ino()
        {
            anyhow::bail!("exact bound-world directory identity changed");
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn validate_bound_directory_identity(_path: &Path, _directory: &fs::File) -> Result<()> {
        anyhow::bail!("exact bound-world adoption requires directory identity support")
    }

    #[cfg(target_os = "linux")]
    fn open_real_metadata_file_at(
        directory: &fs::File,
        name: &str,
        display_path: &Path,
        writable: bool,
    ) -> Result<fs::File> {
        use nix::fcntl::{openat, OFlag};
        use nix::sys::stat::{fstat, Mode, SFlag};
        use std::os::fd::{AsRawFd, FromRawFd};

        let mut flags = OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC;
        if writable {
            flags = OFlag::O_RDWR | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC;
        }
        let fd = openat(directory.as_raw_fd(), name, flags, Mode::empty()).with_context(|| {
            format!("failed to open session metadata {}", display_path.display())
        })?;
        let stat = fstat(fd).with_context(|| {
            format!(
                "failed to inspect session metadata {}",
                display_path.display()
            )
        })?;
        if SFlag::from_bits_truncate(stat.st_mode) & SFlag::S_IFMT != SFlag::S_IFREG {
            let _ = nix::unistd::close(fd);
            anyhow::bail!(
                "session metadata is not a regular file {}",
                display_path.display()
            );
        }
        // SAFETY: `openat` returned a fresh owned descriptor and ownership transfers to `File`.
        Ok(unsafe { fs::File::from_raw_fd(fd) })
    }

    #[cfg(not(target_os = "linux"))]
    fn open_real_metadata_file_at(
        _directory: &fs::File,
        _name: &str,
        _display_path: &Path,
        _writable: bool,
    ) -> Result<fs::File> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor-relative metadata")
    }

    fn read_metadata_bytes_at(
        directory: &fs::File,
        name: &str,
        display_path: &Path,
    ) -> Result<Vec<u8>> {
        let mut file = Self::open_real_metadata_file_at(directory, name, display_path, false)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .with_context(|| format!("failed to read {}", display_path.display()))?;
        Ok(bytes)
    }

    #[cfg(target_os = "linux")]
    fn create_metadata_file_at(
        directory: &fs::File,
        name: &str,
        display_path: &Path,
    ) -> Result<fs::File> {
        use nix::fcntl::{openat, OFlag};
        use nix::sys::stat::Mode;
        use std::os::fd::{AsRawFd, FromRawFd};

        let fd = openat(
            directory.as_raw_fd(),
            name,
            OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::from_bits_truncate(0o666),
        )
        .with_context(|| format!("failed to create {}", display_path.display()))?;
        // SAFETY: `openat` returned a fresh owned descriptor and ownership transfers to `File`.
        Ok(unsafe { fs::File::from_raw_fd(fd) })
    }

    #[cfg(not(target_os = "linux"))]
    fn create_metadata_file_at(
        _directory: &fs::File,
        _name: &str,
        _display_path: &Path,
    ) -> Result<fs::File> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor-relative metadata")
    }

    #[cfg(target_os = "linux")]
    fn rename_metadata_at(
        directory: &fs::File,
        old_name: &str,
        new_name: &str,
        display_path: &Path,
    ) -> Result<()> {
        use std::os::fd::AsRawFd;

        nix::fcntl::renameat(
            Some(directory.as_raw_fd()),
            old_name,
            Some(directory.as_raw_fd()),
            new_name,
        )
        .with_context(|| {
            format!(
                "failed to publish exact adoption {}",
                display_path.display()
            )
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn rename_metadata_at(
        _directory: &fs::File,
        _old_name: &str,
        _new_name: &str,
        _display_path: &Path,
    ) -> Result<()> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor-relative metadata")
    }

    #[cfg(target_os = "linux")]
    fn unlink_metadata_at(directory: &fs::File, name: &str, display_path: &Path) -> Result<()> {
        use nix::unistd::{unlinkat, UnlinkatFlags};
        use std::os::fd::AsRawFd;

        unlinkat(
            Some(directory.as_raw_fd()),
            name,
            UnlinkatFlags::NoRemoveDir,
        )
        .with_context(|| format!("failed to remove {}", display_path.display()))
    }

    #[cfg(not(target_os = "linux"))]
    fn unlink_metadata_at(_directory: &fs::File, _name: &str, _display_path: &Path) -> Result<()> {
        anyhow::bail!("exact bound-world adoption requires Linux descriptor-relative metadata")
    }

    fn sync_metadata_file_at(directory: &fs::File, name: &str, display_path: &Path) -> Result<()> {
        Self::open_real_metadata_file_at(directory, name, display_path, true)?
            .sync_all()
            .with_context(|| format!("failed to sync metadata file {}", display_path.display()))
    }

    fn sync_directory_handle(directory: &fs::File, display_path: &Path) -> Result<()> {
        directory.sync_all().with_context(|| {
            format!(
                "failed to sync metadata directory {}",
                display_path.display()
            )
        })
    }

    fn open_real_metadata_file(path: &Path, writable: bool) -> Result<fs::File> {
        let mut options = fs::OpenOptions::new();
        options.read(true).write(writable);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let file = options
            .open(path)
            .with_context(|| format!("failed to open session metadata {}", path.display()))?;
        if !file
            .metadata()
            .with_context(|| format!("failed to inspect session metadata {}", path.display()))?
            .is_file()
        {
            anyhow::bail!("session metadata is not a regular file {}", path.display());
        }
        Ok(file)
    }

    fn is_real_directory(path: &Path) -> bool {
        fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_dir() && !metadata.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn is_real_file(path: &Path) -> bool {
        fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_file() && !metadata.file_type().is_symlink())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    pub(crate) fn lock_shared_root_for_ownership(metadata_dir: &Path) -> Result<fs::File> {
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::OpenOptionsExt;

        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(metadata_dir)
            .with_context(|| {
                format!(
                    "failed to open metadata directory {}",
                    metadata_dir.display()
                )
            })?;
        // SAFETY: `directory` owns a valid open directory descriptor for the duration of the
        // operation, and `flock` neither aliases nor dereferences Rust memory.
        let result = unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX) };
        if result != 0 {
            return Err(std::io::Error::last_os_error()).with_context(|| {
                format!(
                    "failed to lock metadata directory {}",
                    metadata_dir.display()
                )
            });
        }
        Ok(directory)
    }

    #[cfg(not(unix))]
    pub(crate) fn lock_shared_root_for_ownership(_metadata_dir: &Path) -> Result<fs::File> {
        anyhow::bail!("exact bound-world adoption requires durable ownership locking")
    }

    fn should_persist_metadata(&self) -> bool {
        self.spec.reuse_session || self.shared_binding.is_some()
    }

    fn read_metadata(path: &Path) -> Result<SessionWorldMetadata> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse {}", path.display()))
    }

    /// Set up the world isolation.
    fn setup(&mut self) -> Result<()> {
        tracing::info!("world.setup: creating directories");
        self.create_directories()
            .context("create_directories failed")?;

        #[cfg(target_os = "linux")]
        {
            // Lightweight Linux setup for PTY: avoid unsharing/pivoting the current process.
            tracing::info!("world.setup: linux isolation");
            self.setup_linux_isolation()
                .context("setup_linux_isolation failed")?;

            // Set up network filtering if enabled (scoped to netns when available)
            if self.spec.isolate_network {
                tracing::info!("world.setup: installing nftables rules");
                self.setup_network_filter().context(
                    "requested network isolation could not be enforced during world setup",
                )?;
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  Linux isolation not available on this platform");
        }

        Ok(())
    }

    /// Set up network filtering with nftables.
    #[allow(dead_code)]
    fn setup_network_filter(&mut self) -> Result<()> {
        let mut filter =
            crate::netfilter::NetFilter::new(&self.id, self.spec.allowed_domains.clone())?;
        #[cfg(target_os = "linux")]
        filter.set_cgroup_path(&self.cgroup_path);
        filter.resolve_domains()?;
        filter.install_rules()?;
        self.network_filter = Some(filter);
        Ok(())
    }

    pub fn refresh_network_filter(&mut self) -> Result<()> {
        if !self.spec.isolate_network {
            return Ok(());
        }

        if self.network_filter.is_none() {
            self.setup_network_filter()?;
        }

        let filter = self
            .network_filter
            .as_mut()
            .ok_or_else(|| anyhow!("network filter missing for isolated session"))?;
        filter.refresh_rules()
    }

    pub fn cgroup_path(&self) -> PathBuf {
        self.cgroup_path.clone()
    }

    fn fallback_cgroup_path(&self) -> PathBuf {
        let uid = current_uid();
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/cgroup").join(&self.id);
            }
        }
        let run = PathBuf::from(format!("/run/user/{uid}/substrate/cgroup/{}", self.id));
        if run.parent().unwrap_or(Path::new("/run")).exists() {
            return run;
        }
        PathBuf::from(format!("/tmp/substrate-{uid}-cgroup/{}", self.id))
    }

    fn create_directories(&mut self) -> Result<()> {
        if let Err(e) = std::fs::create_dir_all(&self.root_dir) {
            tracing::error!(
                error = %e,
                path = %self.root_dir.display(),
                "[world] failed to create world root directory"
            );
            return Err(e).context("Failed to create world root directory");
        }
        if let Err(e) = std::fs::create_dir_all(&self.cgroup_path) {
            let fallback_allowed = current_uid() != 0
                && matches!(
                    e.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem
                );
            if fallback_allowed {
                let fallback = self.fallback_cgroup_path();
                tracing::warn!(
                    error = %e,
                    path = %self.cgroup_path.display(),
                    fallback = %fallback.display(),
                    "[world] failed to create cgroup directory; using unprivileged fallback path"
                );
                std::fs::create_dir_all(&fallback)
                    .context("Failed to create fallback cgroup directory")?;
                self.cgroup_path = fallback;
                self.ensure_cgroup_attach_target()?;
                return Ok(());
            }
            tracing::error!(
                error = %e,
                path = %self.cgroup_path.display(),
                "[world] failed to create cgroup directory"
            );
            return Err(e).context("Failed to create cgroup directory");
        }
        self.ensure_cgroup_attach_target()?;
        Ok(())
    }

    fn ensure_cgroup_attach_target(&self) -> Result<()> {
        if self.cgroup_path.starts_with(Path::new("/sys/fs/cgroup")) {
            return Ok(());
        }

        let cgroup_procs = self.cgroup_path.join("cgroup.procs");
        if cgroup_procs.exists() {
            return Ok(());
        }

        // Unprivileged fallback directories are ordinary host paths rather than kernel cgroups,
        // so materialize a writable surrogate that the launchers can use for placement proofing.
        fs::File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&cgroup_procs)
            .with_context(|| format!("failed to create {}", cgroup_procs.display()))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn setup_linux_isolation(&self) -> Result<()> {
        // Lightweight no-op for PTY phase: avoid unshare/pivot_root in the agent path.
        // Non-PTY overlayfs isolation remains handled by overlayfs::execute_with_overlay().
        Ok(())
    }

    /// Execute a command in this world.
    pub fn execute(
        &mut self,
        cmd: &str,
        cwd: &Path,
        env: HashMap<String, String>,
        _pty: bool,
        span_id: Option<String>,
    ) -> Result<ExecResult> {
        let output;
        let process_capture =
            crate::exec::ProcessCaptureSpec::from_env(&self.id, &env, span_id.as_deref());
        let process_telemetry;
        let scopes_used;
        let mut diff_opt: Option<FsDiff> = None;
        let mut fs_strategy_meta: Option<crate::overlayfs::WorldFsStrategyMeta> = None;

        let mut command_to_run = cmd.to_string();
        if crate::guard::should_guard_anchor(&env) {
            command_to_run =
                crate::guard::wrap_with_anchor_guard(&command_to_run, &self.project_dir);
        }
        command_to_run = crate::guard::wrap_with_world_env_contract(&command_to_run, &env);

        let force_direct_exec = env
            .get("SUBSTRATE_WORLD_EXEC_FORCE_DIRECT")
            .is_some_and(|value| is_truthy(value));
        let require_cgroup_attach = self.spec.isolate_network;

        if require_cgroup_attach && force_direct_exec {
            return Err(anyhow!(
                "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT is unsupported when isolate_network=true because cgroup attach is not guaranteed"
            ));
        }

        if require_cgroup_attach {
            self.refresh_network_filter()?;
        }

        // When fs_mode is enforced or heuristics request isolation, run against a persistent overlay
        // so state is consistent across commands within this session.
        if require_cgroup_attach
            || (!force_direct_exec
                && (self.spec.fs_mode == WorldFsMode::ReadOnly
                    || self.spec.fs_mode != WorldFsMode::Writable
                    || self.should_isolate_command(cmd)))
        {
            let merged_dir = self.ensure_overlay_mounted()?;
            fs_strategy_meta = crate::overlayfs::world_fs_strategy_meta(&self.id);
            let desired_cwd = if cwd.starts_with(&self.project_dir) {
                cwd.to_path_buf()
            } else {
                self.project_dir.clone()
            };
            let exec_ctx = OverlayExecutionContext {
                command_to_run: &command_to_run,
                cwd,
                env: &env,
                merged_dir: &merged_dir,
                desired_cwd: &desired_cwd,
                require_cgroup_attach,
                span_id: span_id.as_deref(),
            };
            let captured = self.execute_with_overlay_helpers(&exec_ctx, &process_capture)?;
            output = captured.output;
            process_telemetry = captured.process_telemetry;

            if self.spec.fs_mode == WorldFsMode::ReadOnly {
                diff_opt = Some(FsDiff::default());
            } else if let Some(ref overlay) = self.overlay {
                let diff = overlay.compute_diff()?;
                diff_opt = Some(diff.clone());
                if let Some(id) = span_id.as_ref() {
                    self.fs_by_span.insert(id.clone(), diff);
                }
            }
        } else {
            let captured = crate::exec::execute_shell_command_with_capture(
                &command_to_run,
                cwd,
                &env,
                false,
                crate::exec::CommandCapture::new(span_id.as_deref(), Some(&process_capture)),
            )
            .context("Failed to execute command")?;
            output = captured.output;
            process_telemetry = captured.process_telemetry;
        }

        // Track network scopes if filter is active
        if let Some(ref mut filter) = self.network_filter {
            scopes_used = crate::netfilter::monitor_network_scopes(filter)?;
        } else {
            scopes_used = vec![];
        }

        Ok(ExecResult {
            exit: exit_code_from_status(output.status),
            stdout: output.stdout,
            stderr: output.stderr,
            scopes_used,
            fs_diff: diff_opt,
            world_fs_strategy_primary: fs_strategy_meta.as_ref().map(|m| m.primary),
            world_fs_strategy_final: fs_strategy_meta.as_ref().map(|m| m.final_strategy),
            world_fs_strategy_fallback_reason: fs_strategy_meta.as_ref().map(|m| m.fallback_reason),
            process_telemetry,
        })
    }

    fn execute_with_overlay_helpers(
        &self,
        exec_ctx: &OverlayExecutionContext<'_>,
        process_capture: &crate::exec::ProcessCaptureSpec,
    ) -> Result<crate::exec::CapturedCommandOutput> {
        let project_attach_policy = if exec_ctx.require_cgroup_attach {
            crate::exec::CgroupAttachPolicy::required(
                "project_bind_mount",
                self.cgroup_path.as_path(),
            )
        } else {
            crate::exec::CgroupAttachPolicy::optional("project_bind_mount")
        };

        match crate::exec::execute_shell_command_with_project_bind_mount_capture(
            exec_ctx.command_to_run,
            crate::exec::ProjectBindMount {
                merged_dir: exec_ctx.merged_dir,
                project_dir: &self.project_dir,
                desired_cwd: exec_ctx.desired_cwd,
                fs_mode: self.spec.fs_mode,
            },
            exec_ctx.env,
            false,
            project_attach_policy,
            crate::exec::CommandCapture::new(exec_ctx.span_id, Some(process_capture)),
        ) {
            Ok(output)
                if !exec_ctx.require_cgroup_attach
                    || !crate::exec::is_cgroup_attach_wrapper_failure(&output.output.stderr) =>
            {
                Ok(output)
            }
            Ok(output) => self.execute_world_deps_fallback(
                exec_ctx,
                anyhow!(
                    "project bind mount helper refused isolated execution before command start: {}",
                    String::from_utf8_lossy(&output.output.stderr).trim()
                ),
                process_capture,
            ),
            Err(err) => self.execute_world_deps_fallback(exec_ctx, err, process_capture),
        }
    }

    fn execute_world_deps_fallback(
        &self,
        exec_ctx: &OverlayExecutionContext<'_>,
        primary_err: anyhow::Error,
        process_capture: &crate::exec::ProcessCaptureSpec,
    ) -> Result<crate::exec::CapturedCommandOutput> {
        if self.spec.fs_mode == WorldFsMode::ReadOnly {
            return Err(primary_err).context(
                "failed to enforce read-only world via mount-namespace bind; refusing to run with possible absolute-path escape",
            );
        }

        let mut rel = if exec_ctx.cwd.starts_with(&self.project_dir) {
            exec_ctx
                .cwd
                .strip_prefix(&self.project_dir)
                .unwrap_or_else(|_| Path::new("."))
                .to_path_buf()
        } else {
            PathBuf::from(".")
        };
        if rel.as_os_str().is_empty() {
            rel = PathBuf::from(".");
        }
        let target_dir = exec_ctx.merged_dir.join(&rel);
        let fallback_world_deps_root =
            crate::exec::stable_world_deps_fallback_root(&self.project_dir);
        let fallback_attach_policy = if exec_ctx.require_cgroup_attach {
            crate::exec::CgroupAttachPolicy::required(
                "world_deps_fallback",
                self.cgroup_path.as_path(),
            )
        } else {
            crate::exec::CgroupAttachPolicy::optional("world_deps_fallback")
        };

        match crate::exec::execute_shell_command_with_world_deps_bind_mount_capture(
            exec_ctx.command_to_run,
            &target_dir,
            exec_ctx.env,
            false,
            &fallback_world_deps_root,
            fallback_attach_policy,
            crate::exec::CommandCapture::new(exec_ctx.span_id, Some(process_capture)),
        ) {
            Ok(output)
                if !exec_ctx.require_cgroup_attach
                    || !crate::exec::is_cgroup_attach_wrapper_failure(&output.output.stderr) =>
            {
                Ok(output)
            }
            Ok(output) => Err(primary_err).context(format!(
                "world-deps fallback helper refused isolated execution before command start: {}",
                String::from_utf8_lossy(&output.output.stderr).trim()
            )),
            Err(world_deps_err) => {
                if exec_ctx.require_cgroup_attach {
                    Err(primary_err).context(format!(
                        "world-deps fallback helper also failed: {world_deps_err:#}"
                    ))
                } else {
                    crate::exec::execute_shell_command_with_capture(
                        exec_ctx.command_to_run,
                        &target_dir,
                        exec_ctx.env,
                        false,
                        crate::exec::CommandCapture::new(exec_ctx.span_id, Some(process_capture)),
                    )
                        .with_context(|| {
                            format!(
                                "Failed to execute command in overlay after mount-namespace bind failed: {primary_err:#}; world-deps fallback also failed: {world_deps_err:#}"
                            )
                        })
                }
            }
        }
    }

    /// Compute filesystem diff for a span.
    pub fn compute_fs_diff(&self, span_id: &str) -> Result<FsDiff> {
        if let Some(diff) = self.fs_by_span.get(span_id) {
            return Ok(diff.clone());
        }
        Ok(FsDiff::default())
    }

    /// Compute the current session's pending diff (cumulative overlay state).
    pub fn compute_pending_diff(&self) -> Result<FsDiff> {
        match self.overlay.as_ref() {
            Some(overlay) => overlay.compute_diff(),
            None => Ok(FsDiff::default()),
        }
    }

    /// Clear the current session's pending diff state by discarding the overlay upper/work layers.
    pub fn clear_pending_diff(&mut self) -> Result<()> {
        if let Some(mut overlay) = self.overlay.take() {
            overlay.cleanup().context("overlay cleanup failed")?;
        }
        self.overlay_mode = None;
        Ok(())
    }

    /// Discard the overlay upper entry for a set of workspace-relative paths.
    ///
    /// Missing paths are ignored. Returns the number of filesystem entries removed from the
    /// backing upper/work layer.
    pub fn discard_pending_paths(&mut self, paths: &[PathBuf]) -> Result<u32> {
        let Some(ref mut overlay) = self.overlay else {
            return Ok(0);
        };
        overlay.discard_paths(paths)
    }

    /// Ensure the overlay is mounted and return the merged root for reuse across entry points.
    pub(crate) fn ensure_overlay_root(&mut self) -> Result<PathBuf> {
        self.ensure_overlay_mounted()
    }

    /// Check if a command should be isolated with overlayfs.
    fn should_isolate_command(&self, cmd: &str) -> bool {
        // Force isolation if always_isolate is set
        if self.spec.always_isolate {
            return true;
        }

        // Commands that should run in isolated overlayfs
        let isolated_patterns = [
            "pip install",
            "npm install",
            "cargo install",
            "go get",
            "gem install",
            "apt install",
            "yum install",
            "brew install",
        ];

        isolated_patterns
            .iter()
            .any(|pattern| cmd.contains(pattern))
    }

    /// Ensure a persistent overlay mount is available for this session and return the merged root.
    fn ensure_overlay_mounted(&mut self) -> Result<PathBuf> {
        if self.overlay.is_none() {
            self.overlay = Some(OverlayFs::new(&self.id)?);
        }

        let desired_mode = self.spec.fs_mode;
        let overlay = self
            .overlay
            .as_mut()
            .expect("overlay should be initialized above");

        if !overlay.is_mounted() {
            if desired_mode == WorldFsMode::ReadOnly {
                overlay.mount_read_only(&self.project_dir)?;
            } else {
                overlay.mount(&self.project_dir)?;
            }
            self.overlay_mode = Some(desired_mode);
            return Ok(overlay.merged_dir_path().to_path_buf());
        }

        if self.overlay_mode != Some(desired_mode) {
            if desired_mode == WorldFsMode::ReadOnly {
                // fuse-overlayfs does not reliably honor MS_RDONLY remount semantics, so rebuild the mount.
                if overlay.is_using_fuse() {
                    overlay.unmount().context("Failed to unmount overlay")?;
                    overlay
                        .mount_read_only(&self.project_dir)
                        .context("Failed to mount read-only overlay")?;
                } else {
                    #[cfg(target_os = "linux")]
                    overlay
                        .remount_read_only()
                        .context("Failed to remount overlay read-only")?;
                    #[cfg(not(target_os = "linux"))]
                    anyhow::bail!("read-only overlay remount is only supported on Linux");
                }
            } else {
                // Switching from a read-only lower-only mount back to writable requires a full remount.
                if self.overlay_mode == Some(WorldFsMode::ReadOnly) {
                    overlay.unmount().context("Failed to unmount overlay")?;
                    overlay
                        .mount(&self.project_dir)
                        .context("Failed to mount writable overlay")?;
                } else {
                    #[cfg(target_os = "linux")]
                    overlay
                        .remount_writable()
                        .context("Failed to remount overlay writable")?;
                }
            }
            self.overlay_mode = Some(desired_mode);
        } else if self.overlay_mode.is_none() {
            self.overlay_mode = Some(desired_mode);
        }

        Ok(overlay.merged_dir_path().to_path_buf())
    }

    /// Apply policy to this world.
    pub fn apply_policy(&self, _spec: &WorldSpec) -> Result<()> {
        // TODO: Implement policy application
        Ok(())
    }
}

fn read_session_root_dir(root_dir: &Path) -> Result<Option<fs::ReadDir>> {
    match fs::read_dir(root_dir) {
        Ok(entries) => Ok(Some(entries)),
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
            ) =>
        {
            tracing::warn!(
                error = %err,
                root = %root_dir.display(),
                "session root is unavailable for recovery lookup; treating as no reusable session"
            );
            Ok(None)
        }
        Err(err) => {
            Err(err).with_context(|| format!("failed to read session root {}", root_dir.display()))
        }
    }
}

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::geteuid() as u32 }
}

#[cfg(not(unix))]
fn current_uid() -> u32 {
    0
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

fn exit_code_from_status(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }

    -1
}

impl Drop for SessionWorld {
    fn drop(&mut self) {
        if let Some(ref mut overlay) = self.overlay {
            let _ = overlay.cleanup();
        }
        self.overlay_mode = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(target_os = "linux")]
    use std::sync::Mutex;
    use tempfile::tempdir;
    #[cfg(target_os = "linux")]
    use tempfile::TempDir;
    use world_api::SharedWorldOwnerAction;

    #[cfg(target_os = "linux")]
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(target_os = "linux")]
    struct EnvGuard {
        previous: Vec<(String, Option<std::ffi::OsString>)>,
    }

    #[cfg(target_os = "linux")]
    impl EnvGuard {
        fn set(vars: &[(&str, Option<&str>)]) -> Self {
            let previous = vars
                .iter()
                .map(|(key, _)| (key.to_string(), std::env::var_os(key)))
                .collect::<Vec<_>>();
            for (key, value) in vars {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
            Self { previous }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
                match value {
                    Some(v) => std::env::set_var(&key, v),
                    None => std::env::remove_var(&key),
                }
            }
        }
    }

    #[cfg(unix)]
    fn running_as_root() -> bool {
        // SAFETY: geteuid reads the effective uid of the current process and does not require
        // any additional invariants from Rust.
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(target_os = "linux")]
    fn test_world(temp: &TempDir, isolate_network: bool) -> SessionWorld {
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&project_dir).expect("project dir");
        SessionWorld {
            id: "wld_test".into(),
            root_dir: temp.path().join("world-root"),
            project_dir,
            cgroup_path: temp.path().join("cgroup").join("wld_test"),
            net_namespace: None,
            spec: WorldSpec {
                isolate_network,
                project_dir: temp.path().join("project"),
                fs_mode: WorldFsMode::Writable,
                ..WorldSpec::default()
            },
            started_at: std::time::SystemTime::UNIX_EPOCH,
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: None,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn create_directories_materializes_surrogate_cgroup_procs_for_fallback_paths() {
        let temp = tempdir().unwrap();
        let fallback_cgroup_path = temp.path().join("fallback-cgroup").join("wld_test");
        let mut world = test_world(&temp, false);
        world.cgroup_path = fallback_cgroup_path.clone();

        world.create_directories().unwrap();

        assert!(fallback_cgroup_path.is_dir());
        assert!(
            fallback_cgroup_path.join("cgroup.procs").is_file(),
            "expected fallback cgroup paths to materialize a writable cgroup.procs surrogate"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn recovered_fallback_world_materializes_surrogate_cgroup_procs() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("fallback-cgroup").join("wld_recovered");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: false,
            project_dir: project_dir.clone(),
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let world = SessionWorld {
            id: "wld_recovered".into(),
            root_dir: root_dir.clone(),
            project_dir: project_dir.clone(),
            cgroup_path: cgroup_path.clone(),
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: None,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();
        assert!(
            !cgroup_path.join("cgroup.procs").exists(),
            "test precondition should reflect pre-patch recovered fallback worlds"
        );

        let recovered = SessionWorld::recover_compatible_from_root(&root_dir, &spec)
            .unwrap()
            .expect("metadata should recover");
        assert_eq!(recovered.cgroup_path, cgroup_path);
        assert!(
            recovered.cgroup_path.join("cgroup.procs").is_file(),
            "expected recovered fallback worlds to self-heal their cgroup attach target"
        );
    }

    #[test]
    fn test_session_world_creation() {
        let spec = WorldSpec::default();

        // This test should work on all platforms, just with different behavior
        match SessionWorld::ensure_started(spec) {
            Ok(world) => {
                assert!(world.id.starts_with("wld_"));
                assert_eq!(world.root_dir, SessionWorld::shared_root_dir());
                assert!(world.cgroup_path.ends_with(&world.id));
            }
            Err(e) => {
                // On non-Linux platforms, setup may fail, which is expected
                println!("Expected failure on non-Linux: {}", e);
            }
        }
    }

    #[test]
    fn session_compatibility_respects_core_spec_fields() {
        let base_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::GenericCompatible,
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: PathBuf::from("/tmp/project-a"),
            always_isolate: false,
            fs_mode: world_api::WorldFsMode::Writable,
            backend_policy: None,
        };
        let world = SessionWorld {
            id: "wld_test".into(),
            root_dir: SessionWorld::shared_root_dir(),
            project_dir: base_spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate/wld_test"),
            net_namespace: None,
            spec: base_spec.clone(),
            started_at: std::time::SystemTime::UNIX_EPOCH,
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: None,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        assert!(world.compatible_with(&base_spec));

        let mut changed = base_spec.clone();
        changed.project_dir = PathBuf::from("/tmp/other");
        assert!(!world.compatible_with(&changed));

        let mut changed = base_spec.clone();
        changed.isolate_network = false;
        assert!(!world.compatible_with(&changed));

        let mut changed = base_spec.clone();
        changed.always_isolate = true;
        assert!(!world.compatible_with(&changed));

        let mut changed = base_spec;
        changed.allowed_domains = vec!["other.com".into()];
        assert!(!world.compatible_with(&changed));

        let mut changed = world.spec.clone();
        changed.fs_mode = world_api::WorldFsMode::ReadOnly;
        assert!(
            world.compatible_with(&changed),
            "fs_mode differences should not force a new world; overlay remount handles mode changes"
        );
    }

    #[test]
    fn default_shared_root_dir_prefers_xdg_runtime_dir() {
        let root = SessionWorld::default_shared_root_dir_for(
            1000,
            Some(Path::new("/tmp/runtime-dir")),
            true,
        );

        assert_eq!(root, PathBuf::from("/tmp/runtime-dir/substrate/worlds"));
    }

    #[test]
    fn default_shared_root_dir_uses_run_user_when_xdg_runtime_dir_is_missing() {
        let root = SessionWorld::default_shared_root_dir_for(1000, None, true);

        assert_eq!(root, PathBuf::from("/run/user/1000/substrate/worlds"));
    }

    #[test]
    fn default_shared_root_dir_falls_back_to_user_scoped_tmp_root() {
        let root = SessionWorld::default_shared_root_dir_for(1000, None, false);

        assert_eq!(root, PathBuf::from("/tmp/substrate-worlds-1000"));
    }

    #[test]
    fn persisted_metadata_round_trips_for_recovery() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_roundtrip");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let world = SessionWorld {
            id: "wld_roundtrip".into(),
            root_dir: root_dir.clone(),
            project_dir: project_dir.clone(),
            cgroup_path: cgroup_path.clone(),
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: None,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();

        let recovered = SessionWorld::recover_compatible_from_root(&root_dir, &spec)
            .unwrap()
            .expect("metadata should recover");
        assert_eq!(recovered.id, world.id);
        assert_eq!(recovered.project_dir, world.project_dir);
        assert_eq!(recovered.cgroup_path, world.cgroup_path);
        assert_eq!(recovered.started_at, world.started_at);
        assert!(recovered.compatible_with(&spec));
        assert_eq!(recovered.shared_binding(), None);
    }

    #[test]
    #[cfg(unix)]
    fn recover_compatible_from_root_returns_none_when_root_unreadable() {
        if running_as_root() {
            eprintln!("skipping unreadable-root recovery test when running as root");
            return;
        }

        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let original_permissions = std::fs::metadata(&root_dir).unwrap().permissions();
        std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o000)).unwrap();
        let recovered = SessionWorld::recover_compatible_from_root(&root_dir, &spec).unwrap();
        std::fs::set_permissions(&root_dir, original_permissions).unwrap();

        assert!(recovered.is_none());
    }

    #[test]
    fn shared_metadata_round_trips_for_recovery() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_shared");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let world = SessionWorld {
            id: "wld_shared".into(),
            root_dir: root_dir.clone(),
            project_dir: project_dir.clone(),
            cgroup_path: cgroup_path.clone(),
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id.clone(),
                world_id: "wld_shared".into(),
                world_generation: 0,
                binding_state: SharedWorldBindingState::Active,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec)
                .unwrap()
                .expect("shared metadata should recover");
        assert_eq!(
            recovered.shared_binding(),
            Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_123".into(),
                world_id: "wld_shared".into(),
                world_generation: 0,
                binding_state: SharedWorldBindingState::Active,
            })
        );
        assert_eq!(recovered.last_restart_reason, None);
    }

    #[test]
    #[cfg(unix)]
    fn recover_shared_active_from_root_returns_none_when_root_unreadable() {
        if running_as_root() {
            eprintln!("skipping unreadable-root shared recovery test when running as root");
            return;
        }

        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let original_permissions = std::fs::metadata(&root_dir).unwrap().permissions();
        std::fs::set_permissions(&root_dir, std::fs::Permissions::from_mode(0o000)).unwrap();
        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec).unwrap();
        std::fs::set_permissions(&root_dir, original_permissions).unwrap();

        assert!(recovered.is_none());
    }

    #[test]
    fn shared_binding_state_transitions_persist_and_reject_invalid_edges() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_shared");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let mut world = SessionWorld {
            id: "wld_shared".into(),
            root_dir: root_dir.clone(),
            project_dir,
            cgroup_path,
            net_namespace: None,
            spec,
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id,
                world_id: "wld_shared".into(),
                world_generation: 7,
                binding_state: SharedWorldBindingState::Active,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();
        world
            .set_shared_binding_state(SharedWorldBindingState::Replacing, Some("restart".into()))
            .unwrap();
        assert_eq!(
            world.shared_binding().unwrap().binding_state,
            SharedWorldBindingState::Replacing
        );
        assert_eq!(world.last_restart_reason.as_deref(), Some("restart"));

        world
            .set_shared_binding_state(SharedWorldBindingState::Active, None)
            .unwrap();
        assert_eq!(
            world.shared_binding().unwrap().binding_state,
            SharedWorldBindingState::Active
        );
        assert_eq!(world.last_restart_reason, None);

        world
            .set_shared_binding_state(SharedWorldBindingState::Replacing, Some("restart".into()))
            .unwrap();
        world
            .set_shared_binding_state(SharedWorldBindingState::Replaced, Some("restart".into()))
            .unwrap();
        assert_eq!(
            world.shared_binding().unwrap().binding_state,
            SharedWorldBindingState::Replaced
        );
        assert_eq!(world.last_restart_reason.as_deref(), Some("restart"));

        let err = world
            .set_shared_binding_state(SharedWorldBindingState::Active, None)
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("invalid shared binding state transition"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn lone_replacing_world_recovers_back_to_active() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_shared");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let world = SessionWorld {
            id: "wld_shared".into(),
            root_dir: root_dir.clone(),
            project_dir,
            cgroup_path,
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id.clone(),
                world_id: "wld_shared".into(),
                world_generation: 3,
                binding_state: SharedWorldBindingState::Replacing,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: Some("restart".into()),
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec)
                .unwrap()
                .expect("replacing metadata should recover");
        let binding = recovered.shared_binding().expect("shared binding");
        assert_eq!(binding.binding_state, SharedWorldBindingState::Active);
        assert_eq!(binding.world_generation, 3);
        assert_eq!(recovered.last_restart_reason, None);
    }

    #[test]
    fn shared_recovery_prefers_newer_active_over_older_replacing() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let old_cgroup_path = temp.path().join("cgroup").join("wld_old");
        let new_cgroup_path = temp.path().join("cgroup").join("wld_new");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&old_cgroup_path).unwrap();
        std::fs::create_dir_all(&new_cgroup_path).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let old_world = SessionWorld {
            id: "wld_old".into(),
            root_dir: root_dir.clone(),
            project_dir: project_dir.clone(),
            cgroup_path: old_cgroup_path,
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(1_000),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id.clone(),
                world_id: "wld_old".into(),
                world_generation: 4,
                binding_state: SharedWorldBindingState::Replacing,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: Some("restart".into()),
            overlay: None,
            overlay_mode: None,
        };
        old_world.persist_metadata().unwrap();

        let new_world = SessionWorld {
            id: "wld_new".into(),
            root_dir: root_dir.clone(),
            project_dir,
            cgroup_path: new_cgroup_path,
            net_namespace: None,
            spec: spec.clone(),
            started_at: UNIX_EPOCH + Duration::from_millis(2_000),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id.clone(),
                world_id: "wld_new".into(),
                world_generation: 5,
                binding_state: SharedWorldBindingState::Active,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: Some("restart".into()),
            overlay: None,
            overlay_mode: None,
        };
        new_world.persist_metadata().unwrap();

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec)
                .unwrap()
                .expect("newer active metadata should recover");
        let binding = recovered.shared_binding().expect("shared binding");
        assert_eq!(binding.world_id, "wld_new");
        assert_eq!(binding.world_generation, 5);
        assert_eq!(binding.binding_state, SharedWorldBindingState::Active);
    }

    #[test]
    fn stale_or_invalid_metadata_is_ignored() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let invalid_dir = root_dir.join("wld_invalid");
        std::fs::create_dir_all(&invalid_dir).unwrap();
        std::fs::write(invalid_dir.join(SESSION_METADATA_FILE_NAME), b"{not-json").unwrap();

        let stale_dir = root_dir.join("wld_stale");
        std::fs::create_dir_all(&stale_dir).unwrap();
        let stale = SessionWorldMetadata {
            world_id: "wld_stale".into(),
            project_dir: project_dir.clone(),
            isolate_network: false,
            always_isolate: false,
            allowed_domains: vec!["example.com".into()],
            cgroup_path: temp.path().join("missing-cgroup"),
            started_at_unix_millis: 42,
            owner_mode: SessionWorldOwnerMode::Generic,
            orchestration_session_id: None,
            world_generation: None,
            binding_state: None,
            policy_snapshot_hash: None,
            exact_world_spec_commitment: None,
            exact_adoption_policy_ref_id: None,
            world_fs_mode: None,
            last_restart_reason: None,
        };
        std::fs::write(
            stale_dir.join(SESSION_METADATA_FILE_NAME),
            serde_json::to_vec(&stale).unwrap(),
        )
        .unwrap();

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let recovered = SessionWorld::recover_compatible_from_root(&root_dir, &spec).unwrap();
        assert!(recovered.is_none(), "stale metadata should be ignored");
    }

    #[test]
    fn shared_recovery_rejects_ownerless_legacy_metadata() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_legacy");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();
        let metadata_dir = root_dir.join("wld_legacy");
        std::fs::create_dir_all(&metadata_dir).unwrap();

        std::fs::write(
            metadata_dir.join(SESSION_METADATA_FILE_NAME),
            format!(
                r#"{{
  "world_id": "wld_legacy",
  "project_dir": "{}",
  "isolate_network": false,
  "always_isolate": false,
  "allowed_domains": ["example.com"],
  "cgroup_path": "{}",
  "started_at_unix_millis": 5000
}}"#,
                project_dir.display(),
                cgroup_path.display()
            ),
        )
        .unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec).unwrap();
        assert!(
            recovered.is_none(),
            "legacy ownerless metadata must not be reused for shared-owner mode"
        );
    }

    #[test]
    fn shared_recovery_rejects_cross_owned_or_inactive_metadata() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_shared");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let metadata_dir = root_dir.join("wld_shared");
        std::fs::create_dir_all(&metadata_dir).unwrap();
        let metadata = SessionWorldMetadata {
            world_id: "wld_shared".into(),
            project_dir: project_dir.clone(),
            isolate_network: false,
            always_isolate: false,
            allowed_domains: vec!["example.com".into()],
            cgroup_path: cgroup_path.clone(),
            started_at_unix_millis: 42,
            owner_mode: SessionWorldOwnerMode::SharedOrchestration,
            orchestration_session_id: Some("orch_other".into()),
            world_generation: Some(0),
            binding_state: Some(SharedWorldBindingState::Replaced),
            policy_snapshot_hash: None,
            exact_world_spec_commitment: None,
            exact_adoption_policy_ref_id: None,
            world_fs_mode: Some(WorldFsMode::Writable),
            last_restart_reason: Some("restart".into()),
        };
        std::fs::write(
            metadata_dir.join(SESSION_METADATA_FILE_NAME),
            serde_json::to_vec(&metadata).unwrap(),
        )
        .unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec).unwrap();
        assert!(
            recovered.is_none(),
            "cross-owned or inactive metadata must not be reused"
        );
    }

    #[test]
    fn shared_recovery_ignores_partial_owner_metadata() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_partial");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let metadata_dir = root_dir.join("wld_partial");
        std::fs::create_dir_all(&metadata_dir).unwrap();
        let partial = SessionWorldMetadata {
            world_id: "wld_partial".into(),
            project_dir: project_dir.clone(),
            isolate_network: false,
            always_isolate: false,
            allowed_domains: vec!["example.com".into()],
            cgroup_path,
            started_at_unix_millis: 42,
            owner_mode: SessionWorldOwnerMode::SharedOrchestration,
            orchestration_session_id: Some("orch_123".into()),
            world_generation: Some(0),
            binding_state: None,
            policy_snapshot_hash: None,
            exact_world_spec_commitment: None,
            exact_adoption_policy_ref_id: None,
            world_fs_mode: Some(WorldFsMode::Writable),
            last_restart_reason: None,
        };
        std::fs::write(
            metadata_dir.join(SESSION_METADATA_FILE_NAME),
            serde_json::to_vec(&partial).unwrap(),
        )
        .unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec).unwrap();
        assert!(
            recovered.is_none(),
            "partial owner metadata must be treated as non-reusable"
        );
        assert!(
            metadata_dir.join(SESSION_METADATA_FILE_NAME).is_file(),
            "shared recovery must retain malformed owner metadata on disk"
        );
    }

    #[test]
    #[cfg(unix)]
    fn atomic_persist_failure_preserves_prior_metadata_bytes() {
        if current_uid() == 0 {
            return;
        }

        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_shared");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();

        let owner_spec = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(owner_spec.clone()),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir,
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let mut world = SessionWorld {
            id: "wld_shared".into(),
            root_dir: root_dir.clone(),
            project_dir: temp.path().join("project"),
            cgroup_path,
            net_namespace: None,
            spec,
            started_at: UNIX_EPOCH + Duration::from_millis(1_234),
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: owner_spec.orchestration_session_id,
                world_id: "wld_shared".into(),
                world_generation: 0,
                binding_state: SharedWorldBindingState::Active,
            }),
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        world.persist_metadata().unwrap();
        let metadata_path = root_dir.join("wld_shared").join(SESSION_METADATA_FILE_NAME);
        let original = std::fs::read(&metadata_path).unwrap();

        let metadata_dir = root_dir.join("wld_shared");
        let original_permissions = std::fs::metadata(&metadata_dir).unwrap().permissions();
        let mut read_only_permissions = original_permissions.clone();
        read_only_permissions.set_mode(0o555);
        std::fs::set_permissions(&metadata_dir, read_only_permissions).unwrap();

        let err = world
            .set_shared_binding_state(SharedWorldBindingState::Replacing, Some("restart".into()))
            .unwrap_err();
        assert!(
            err.to_string().contains("failed to create")
                || err.to_string().contains("Permission denied"),
            "unexpected error: {err:#}"
        );

        std::fs::set_permissions(&metadata_dir, original_permissions).unwrap();
        assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn setup_fails_when_requested_isolation_cannot_install_netfilter() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let cgroup_path = temp.path().join("cgroup").join("wld_test");

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", None)]);

        let mut world = SessionWorld {
            id: "wld_test".into(),
            root_dir,
            project_dir: temp.path().join("project"),
            cgroup_path,
            net_namespace: None,
            spec: WorldSpec {
                isolate_network: true,
                ..WorldSpec::default()
            },
            started_at: std::time::SystemTime::UNIX_EPOCH,
            network_filter: None,
            fs_by_span: HashMap::new(),
            shared_binding: None,
            policy_snapshot_hash: None,
            exact_adoption_policy_ref_id: None,
            last_restart_reason: None,
            overlay: None,
            overlay_mode: None,
        };

        let err = world.setup().unwrap_err();
        let message = format!("{err:#}");
        assert!(
            message.contains("requested network isolation could not be enforced"),
            "unexpected error: {message}"
        );
        assert!(world.network_filter.is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn execute_rejects_forced_direct_exec_when_isolation_is_requested() {
        let temp = tempdir().unwrap();
        let mut world = test_world(&temp, true);
        let project_dir = world.project_dir.clone();
        let mut env = HashMap::new();
        env.insert("SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".into(), "1".into());

        let err = world
            .execute("printf should-not-run", &project_dir, env, false, None)
            .unwrap_err();
        let message = format!("{err:#}");
        assert!(
            message.contains(
                "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT is unsupported when isolate_network=true"
            ),
            "unexpected error: {message}"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn execute_allows_forced_direct_exec_without_isolation() {
        let temp = tempdir().unwrap();
        let mut world = test_world(&temp, false);
        let project_dir = world.project_dir.clone();
        let mut env = HashMap::new();
        env.insert("SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".into(), "1".into());

        let result = world
            .execute("printf direct-ok", &project_dir, env, false, None)
            .expect("non-isolated direct exec should remain available");

        assert_eq!(result.exit, 0);
        assert_eq!(String::from_utf8_lossy(&result.stdout), "direct-ok");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn direct_exec_reports_sigint_with_shell_convention_exit_code() {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("sleep 10")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("sleep process should spawn");

        std::thread::sleep(std::time::Duration::from_millis(200));
        let rc = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };
        assert_eq!(rc, 0, "failed to send SIGINT to child");

        let status = child.wait().expect("child wait should succeed");
        assert_eq!(exit_code_from_status(status), 130);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn isolated_helper_flow_does_not_fall_back_to_plain_exec_when_attach_fails() {
        let temp = tempdir().unwrap();
        let world = test_world(&temp, true);
        let merged_dir = temp.path().join("merged");
        std::fs::create_dir_all(&merged_dir).expect("merged dir");
        let env = HashMap::new();
        let exec_ctx = OverlayExecutionContext {
            command_to_run: "printf should-not-run",
            cwd: &world.project_dir,
            env: &env,
            merged_dir: &merged_dir,
            desired_cwd: &world.project_dir,
            require_cgroup_attach: true,
            span_id: None,
        };
        let process_capture =
            crate::exec::ProcessCaptureSpec::from_env(&world.id, &env, exec_ctx.span_id);

        let err = match world.execute_with_overlay_helpers(&exec_ctx, &process_capture) {
            Ok(output) => {
                if output.output.status.success() {
                    panic!(
                        "isolated helper flow should not succeed when cgroup attach cannot start"
                    );
                }
                let stderr = String::from_utf8_lossy(&output.output.stderr);
                if stderr.contains("Operation not permitted")
                    || stderr.contains("EPERM")
                    || stderr.contains("unshare")
                {
                    println!("Skipping isolated attach helper test: {stderr}");
                    return;
                }
                panic!("expected isolated helper flow to return an error, got stderr={stderr}");
            }
            Err(err) => err,
        };

        let message = format!("{err:#}");
        assert!(
            message.contains(
                "project bind mount helper refused isolated execution before command start"
            ) || message.contains(
                "world-deps fallback helper refused isolated execution before command start"
            ) || message.contains("world-deps fallback helper also failed"),
            "unexpected error: {message}"
        );
        assert!(
            !message.contains("should-not-run"),
            "isolated attach failure should stop before plain command execution: {message}"
        );
    }

    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_fixture(
        temp: &TempDir,
    ) -> (SessionWorld, crate::ExactBoundWorldOwnershipAdoptionV1) {
        let mut world = test_world(temp, false);
        world.id = "wld_hsa_bound".into();
        world.cgroup_path = temp.path().join("cgroup").join(&world.id);
        world.spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::GenericCompatible,
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: world.project_dir.clone(),
            always_isolate: false,
            fs_mode: WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        std::fs::create_dir_all(&world.root_dir).expect("world root");
        std::fs::create_dir_all(&world.cgroup_path).expect("cgroup");
        world.persist_metadata().expect("generic metadata");

        let target_spec = WorldSpec {
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_exact".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            ..world.spec.clone()
        };
        let adoption = crate::ExactBoundWorldOwnershipAdoptionV1 {
            orchestration_session_id: "orch_exact".into(),
            world_id: world.id.clone(),
            world_generation: 7,
            participant_id: "rwp_exact".into(),
            policy_ref_id: "ao_policy_exact".into(),
            policy_revision: "a".repeat(64),
            policy_snapshot_hash: "a".repeat(64),
            target_spec,
            adoption_correlation_id: "adopt_corr_no_prompt_marker".into(),
        };
        (world, adoption)
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_preserves_identity_and_exact_retry_is_byte_stable() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_path = world
            .root_dir
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);
        let world_dirs_before = std::fs::read_dir(&world.root_dir).unwrap().count();

        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("first exact adoption");
        let first_bytes = std::fs::read(&metadata_path).unwrap();
        let first_binding = world.shared_binding().expect("adopted binding");

        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("exact retry joins");

        assert_eq!(world.id, "wld_hsa_bound");
        assert_eq!(first_binding.world_id, "wld_hsa_bound");
        assert_eq!(first_binding.world_generation, 7);
        assert_eq!(first_binding.orchestration_session_id, "orch_exact");
        assert_eq!(std::fs::read(&metadata_path).unwrap(), first_bytes);
        assert_eq!(
            std::fs::read_dir(&world.root_dir).unwrap().count(),
            world_dirs_before
        );
        assert!(!String::from_utf8_lossy(&first_bytes).contains("adopt_corr_no_prompt_marker"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_conflicts_fail_without_mutation() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("first exact adoption");
        let metadata_path = world
            .root_dir
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);
        let adopted_bytes = std::fs::read(&metadata_path).unwrap();

        let mut conflicts = Vec::new();
        let mut changed = adoption.clone();
        changed.orchestration_session_id = "orch_foreign".into();
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed.world_id = "wld_other".into();
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed.world_generation += 1;
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed.policy_snapshot_hash = "b".repeat(64);
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed.policy_ref_id = "ao_policy_other".into();
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed.target_spec.project_dir = temp.path().join("foreign-project");
        conflicts.push(changed);
        let mut changed = adoption.clone();
        changed
            .target_spec
            .allowed_domains
            .push("foreign.example".into());
        conflicts.push(changed);

        for conflict in conflicts {
            assert!(
                world.adopt_exact_bound_world_ownership(&conflict).is_err(),
                "conflicting adoption must fail closed"
            );
            assert_eq!(std::fs::read(&metadata_path).unwrap(), adopted_bytes);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_reconciles_every_publication_crash_boundary() {
        for fault in [
            ExactBoundWorldAdoptionPublicationFault::BeforeTempPersistence,
            ExactBoundWorldAdoptionPublicationFault::AfterTempWriteBeforeFileSync,
            ExactBoundWorldAdoptionPublicationFault::AfterTempFileSync,
            ExactBoundWorldAdoptionPublicationFault::BeforeRename,
            ExactBoundWorldAdoptionPublicationFault::AfterRenameBeforeDirectorySync,
            ExactBoundWorldAdoptionPublicationFault::AfterDirectorySyncBeforeResponse,
        ] {
            let temp = tempdir().unwrap();
            let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
            let crash = world
                .adopt_exact_bound_world_ownership_with_fault(&adoption, Some(fault))
                .expect_err("injected crash boundary");
            assert!(crash
                .to_string()
                .contains("injected adoption publication crash"));

            let mut reopened = SessionWorld::recover_exact_bound_world_from_root(
                &world.root_dir,
                &adoption.target_spec,
                &adoption.world_id,
            )
            .expect("reopen evidence")
            .expect("exact world remains recoverable");
            reopened
                .adopt_exact_bound_world_ownership(&adoption)
                .expect("exact retry reconciles durable state");
            let binding = reopened.shared_binding().expect("adopted binding");
            assert_eq!(binding.world_id, adoption.world_id);
            assert_eq!(binding.world_generation, adoption.world_generation);
            assert_eq!(
                binding.orchestration_session_id,
                adoption.orchestration_session_id
            );
        }

        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_path = world
            .root_dir
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);
        let original_generic = std::fs::read(&metadata_path).unwrap();
        world
            .adopt_exact_bound_world_ownership_with_fault(
                &adoption,
                Some(ExactBoundWorldAdoptionPublicationFault::AfterRenameBeforeDirectorySync),
            )
            .expect_err("post-rename crash");
        std::fs::write(&metadata_path, &original_generic)
            .expect("simulate allowed reopen shape where rename was not retained");
        let mut reopened = SessionWorld::recover_exact_bound_world_from_root(
            &world.root_dir,
            &adoption.target_spec,
            &adoption.world_id,
        )
        .unwrap()
        .expect("original generic final remains exact");
        reopened
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("exact retry restarts publication from original generic final");
        assert_eq!(
            reopened.shared_binding().unwrap().world_generation,
            adoption.world_generation
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_adopted_join_requires_final_file_resync_before_success() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("first adoption");
        let metadata_dir = world.root_dir.join(&world.id);
        let metadata_path = metadata_dir.join(SESSION_METADATA_FILE_NAME);
        let adopted_bytes = std::fs::read(&metadata_path).unwrap();

        let no_temp_error = world
            .adopt_exact_bound_world_ownership_with_fault(
                &adoption,
                Some(ExactBoundWorldAdoptionPublicationFault::BeforeExactFinalFileSync),
            )
            .expect_err("no-temp exact join must not succeed before final fsync");
        assert!(no_temp_error
            .to_string()
            .contains("before exact final fsync"));

        std::fs::write(
            metadata_dir.join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME),
            &adopted_bytes,
        )
        .unwrap();
        let temp_error = world
            .adopt_exact_bound_world_ownership_with_fault(
                &adoption,
                Some(ExactBoundWorldAdoptionPublicationFault::BeforeExactFinalFileSync),
            )
            .expect_err("temp/final exact join must not succeed before final fsync");
        assert!(temp_error.to_string().contains("before exact final fsync"));
        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("exact retry re-fsyncs final and reconciles temp");
        assert_eq!(std::fs::read(&metadata_path).unwrap(), adopted_bytes);
        assert!(!metadata_dir
            .join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME)
            .exists());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_missing_corrupt_and_substituted_truth() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_path = world
            .root_dir
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);

        std::fs::remove_file(&metadata_path).unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert!(
            !metadata_path.exists(),
            "missing truth must not be recreated"
        );

        world.persist_metadata().unwrap();
        std::fs::write(&metadata_path, b"{corrupt").unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), b"{corrupt");

        world.persist_metadata().unwrap();
        let mut substituted = SessionWorld::read_metadata(&metadata_path).unwrap();
        substituted.world_id = "wld_substituted".into();
        let substituted_bytes = serde_json::to_vec_pretty(&substituted).unwrap();
        std::fs::write(&metadata_path, &substituted_bytes).unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), substituted_bytes);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_symlinked_metadata_without_touching_target() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_dir = world.root_dir.join(&world.id);
        let metadata_path = metadata_dir.join(SESSION_METADATA_FILE_NAME);
        let original = std::fs::read(&metadata_path).unwrap();
        let external = temp.path().join("external-session.json");
        std::fs::write(&external, &original).unwrap();
        std::fs::remove_file(&metadata_path).unwrap();
        symlink(&external, &metadata_path).unwrap();

        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&external).unwrap(), original);
        assert!(std::fs::symlink_metadata(&metadata_path)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_directory_replacement_after_descriptor_binding() {
        for replacement_point in [
            ExactBoundWorldAdoptionPublicationPoint::AfterTrustedDirectoryBinding,
            ExactBoundWorldAdoptionPublicationPoint::AfterRenameBeforeDirectorySync,
        ] {
            let temp = tempdir().unwrap();
            let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
            let metadata_dir = world.root_dir.join(&world.id);
            let metadata_path = metadata_dir.join(SESSION_METADATA_FILE_NAME);
            let original = std::fs::read(&metadata_path).unwrap();
            let displaced = world.root_dir.join(format!("{}-displaced", world.id));
            let mut replaced = false;

            let error = world
                .adopt_exact_bound_world_ownership_unified(&adoption, |point| {
                    if point == replacement_point && !replaced {
                        std::fs::rename(&metadata_dir, &displaced).unwrap();
                        std::fs::create_dir(&metadata_dir).unwrap();
                        std::fs::write(&metadata_path, &original).unwrap();
                        replaced = true;
                    }
                    false
                })
                .expect_err("directory replacement must prevent successful adoption");

            assert!(error
                .to_string()
                .contains("exact bound-world directory identity changed"));
            assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
            assert!(replaced);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_root_replacement_after_descriptor_binding() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let original_root = world.root_dir.clone();
        let original_metadata = std::fs::read(
            original_root
                .join(&world.id)
                .join(SESSION_METADATA_FILE_NAME),
        )
        .unwrap();
        let displaced_root = temp.path().join("displaced-world-root");
        let replacement_metadata = original_root
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);
        let mut replaced = false;

        let error = world
            .adopt_exact_bound_world_ownership_unified(&adoption, |point| {
                if point == ExactBoundWorldAdoptionPublicationPoint::AfterTrustedDirectoryBinding
                    && !replaced
                {
                    std::fs::rename(&original_root, &displaced_root).unwrap();
                    std::fs::create_dir_all(replacement_metadata.parent().unwrap()).unwrap();
                    std::fs::write(&replacement_metadata, &original_metadata).unwrap();
                    replaced = true;
                }
                false
            })
            .expect_err("root replacement must prevent successful adoption");

        assert!(error
            .to_string()
            .contains("exact bound-world directory identity changed"));
        assert_eq!(
            std::fs::read(&replacement_metadata).unwrap(),
            original_metadata
        );
        assert!(replaced);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_cleans_only_incomplete_temp_and_retains_conflicts() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_dir = world.root_dir.join(&world.id);
        let metadata_path = metadata_dir.join(SESSION_METADATA_FILE_NAME);
        let temp_path = metadata_dir.join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME);
        std::fs::write(&temp_path, b"{\"world_id\":\"wld_incomplete").unwrap();

        world
            .adopt_exact_bound_world_ownership(&adoption)
            .expect("operation-bound incomplete temp is cleaned before exact publication");
        assert!(!temp_path.exists());
        let adopted_bytes = std::fs::read(&metadata_path).unwrap();

        let current = SessionWorld::read_metadata(&metadata_path).unwrap();
        let mut conflicting_adoption = adoption.clone();
        conflicting_adoption.world_generation += 1;
        let conflicting_temp =
            SessionWorld::exact_bound_world_adoption_metadata(&current, &conflicting_adoption)
                .expect_err("already-adopted ownership must reject conflicting projection");
        assert!(conflicting_temp.to_string().contains("ownership conflict"));

        let mut forged_temp = current;
        forged_temp.world_generation = Some(adoption.world_generation + 1);
        let forged_bytes = serde_json::to_vec_pretty(&forged_temp).unwrap();
        std::fs::write(&temp_path, &forged_bytes).unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), adopted_bytes);
        assert_eq!(std::fs::read(&temp_path).unwrap(), forged_bytes);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_unknown_metadata_and_ambiguous_temps() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_dir = world.root_dir.join(&world.id);
        let metadata_path = metadata_dir.join(SESSION_METADATA_FILE_NAME);
        let original = std::fs::read(&metadata_path).unwrap();
        let mut unknown_final = serde_json::from_slice::<serde_json::Value>(&original).unwrap();
        unknown_final["prompt_marker"] = serde_json::json!("PROMPT_MUST_NOT_PERSIST");
        let unknown_final_bytes = serde_json::to_vec_pretty(&unknown_final).unwrap();
        std::fs::write(&metadata_path, &unknown_final_bytes).unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), unknown_final_bytes);

        std::fs::write(&metadata_path, &original).unwrap();
        let ambiguous_temp = metadata_dir.join(".session.json.unrecognized.tmp");
        std::fs::write(&ambiguous_temp, b"partial request marker").unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
        assert!(ambiguous_temp.is_file());

        std::fs::remove_file(&ambiguous_temp).unwrap();
        let current = SessionWorld::read_metadata(&metadata_path).unwrap();
        let desired = SessionWorld::exact_bound_world_adoption_metadata(&current, &adoption)
            .expect("desired metadata");
        let mut unknown_temp = serde_json::to_value(&desired).unwrap();
        unknown_temp["request_marker"] = serde_json::json!("REQUEST_MUST_NOT_PERSIST");
        let unknown_temp_bytes = serde_json::to_vec_pretty(&unknown_temp).unwrap();
        let exact_temp = metadata_dir.join(EXACT_BOUND_WORLD_ADOPTION_TEMP_FILE_NAME);
        std::fs::write(&exact_temp, &unknown_temp_bytes).unwrap();
        assert!(world.adopt_exact_bound_world_ownership(&adoption).is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
        assert_eq!(std::fs::read(&exact_temp).unwrap(), unknown_temp_bytes);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_rejects_nonmatching_current_realization_spec() {
        for mismatch in ["limits", "preload", "backend_policy"] {
            let temp = tempdir().unwrap();
            let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
            let metadata_path = world
                .root_dir
                .join(&world.id)
                .join(SESSION_METADATA_FILE_NAME);
            let original = std::fs::read(&metadata_path).unwrap();
            match mismatch {
                "limits" => world.spec.limits.memory = Some("3Gi".into()),
                "preload" => world.spec.enable_preload = true,
                "backend_policy" => {
                    world.spec.backend_policy = Some(world_api::BackendPolicyInputV1 {
                        schema_version: 1,
                        policy_snapshot: world_api::BackendPolicySnapshotV3 {
                            schema_version: 3,
                            net_allowed: Vec::new(),
                            world_fs: world_api::BackendPolicySnapshotWorldFsV3 {
                                host_visible: true,
                                fail_closed: world_api::BackendPolicySnapshotWorldFsFailClosedV3 {
                                    routing: false,
                                },
                                deny_enforcement: None,
                                caged_required: false,
                                discover: None,
                                read: None,
                                write: world_api::BackendPolicySnapshotWorldFsWriteV3 {
                                    enabled: true,
                                    allow_list: vec![".".into()],
                                    deny_list: Vec::new(),
                                },
                            },
                        },
                        world_network: world_api::BackendWorldNetworkRoutingV1 {
                            isolate_network: false,
                            allowed_domains: Vec::new(),
                        },
                    });
                }
                _ => unreachable!(),
            }
            assert!(
                world.adopt_exact_bound_world_ownership(&adoption).is_err(),
                "{mismatch} mismatch must fail closed"
            );
            assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_reopen_rejects_nonmatching_durable_spec_and_missing_commitment() {
        for mismatch in ["limits", "preload", "backend_policy", "project_identity"] {
            let temp = tempdir().unwrap();
            let (world, adoption) = exact_bound_world_adoption_fixture(&temp);
            let metadata_path = world
                .root_dir
                .join(&world.id)
                .join(SESSION_METADATA_FILE_NAME);
            let original = std::fs::read(&metadata_path).unwrap();
            let mut conflicting_spec = adoption.target_spec.clone();
            match mismatch {
                "limits" => conflicting_spec.limits.memory = Some("3Gi".into()),
                "preload" => conflicting_spec.enable_preload = true,
                "backend_policy" => {
                    conflicting_spec.backend_policy = Some(world_api::BackendPolicyInputV1 {
                        schema_version: 1,
                        policy_snapshot: world_api::BackendPolicySnapshotV3 {
                            schema_version: 3,
                            net_allowed: Vec::new(),
                            world_fs: world_api::BackendPolicySnapshotWorldFsV3 {
                                host_visible: true,
                                fail_closed: world_api::BackendPolicySnapshotWorldFsFailClosedV3 {
                                    routing: false,
                                },
                                deny_enforcement: None,
                                caged_required: false,
                                discover: None,
                                read: None,
                                write: world_api::BackendPolicySnapshotWorldFsWriteV3 {
                                    enabled: true,
                                    allow_list: vec![".".into()],
                                    deny_list: Vec::new(),
                                },
                            },
                        },
                        world_network: world_api::BackendWorldNetworkRoutingV1 {
                            isolate_network: false,
                            allowed_domains: Vec::new(),
                        },
                    });
                }
                "project_identity" => {
                    let displaced = temp.path().join("displaced-project");
                    std::fs::rename(&conflicting_spec.project_dir, &displaced).unwrap();
                    std::fs::create_dir_all(&conflicting_spec.project_dir).unwrap();
                }
                _ => unreachable!(),
            }
            assert!(SessionWorld::recover_exact_bound_world_from_root(
                &world.root_dir,
                &conflicting_spec,
                &world.id,
            )
            .is_err());
            assert_eq!(std::fs::read(&metadata_path).unwrap(), original);
        }

        let temp = tempdir().unwrap();
        let (world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let metadata_path = world
            .root_dir
            .join(&world.id)
            .join(SESSION_METADATA_FILE_NAME);
        let mut metadata = SessionWorld::read_metadata(&metadata_path).unwrap();
        metadata.exact_world_spec_commitment = None;
        let unsupported = serde_json::to_vec_pretty(&metadata).unwrap();
        std::fs::write(&metadata_path, &unsupported).unwrap();
        assert!(SessionWorld::recover_exact_bound_world_from_root(
            &world.root_dir,
            &adoption.target_spec,
            &world.id,
        )
        .is_err());
        assert_eq!(std::fs::read(&metadata_path).unwrap(), unsupported);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_spec_rejects_relative_and_symlinked_project_aliases() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let (_world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let mut relative = adoption.target_spec.clone();
        relative.project_dir = PathBuf::from(".");
        assert!(SessionWorld::exact_world_spec_commitment(&relative).is_err());

        let alias = temp.path().join("project-alias");
        symlink(&adoption.target_spec.project_dir, &alias).unwrap();
        let mut symlinked = adoption.target_spec;
        symlinked.project_dir = alias;
        assert!(SessionWorld::exact_world_spec_commitment(&symlinked).is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn exact_bound_world_adoption_revalidates_project_inode_before_success() {
        let temp = tempdir().unwrap();
        let (mut world, adoption) = exact_bound_world_adoption_fixture(&temp);
        let project_dir = adoption.target_spec.project_dir.clone();
        let displaced = temp.path().join("displaced-project-after-publication");
        let mut replaced = false;

        let error = world
            .adopt_exact_bound_world_ownership_unified(&adoption, |point| {
                if point == ExactBoundWorldAdoptionPublicationPoint::AfterRenameBeforeDirectorySync
                    && !replaced
                {
                    std::fs::rename(&project_dir, &displaced).unwrap();
                    std::fs::create_dir(&project_dir).unwrap();
                    replaced = true;
                }
                false
            })
            .expect_err("project identity replacement must prevent successful adoption");

        assert!(error
            .to_string()
            .contains("physical spec changed during ownership publication"));
        assert!(replaced);
        assert!(world.shared_binding().is_none());
    }
}
