//! Linux native world backend implementation.
//!
//! This crate provides the LinuxLocal backend that implements world isolation
//! using Linux namespaces, cgroups v2, nftables, and other native security features.

use anyhow::{Context, Result};
use world_api::{
    ExecRequest, ExecResult, FsDiff, SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldBackend,
    WorldHandle, WorldSpec,
};

pub mod cgroups;
pub mod copydiff;
pub mod diff;
pub mod dns;
pub mod exec;
pub mod guard;
pub mod isolation;
pub mod landlock;
pub mod mountinfo;
pub mod netfilter;
pub mod netns;
pub mod network;
pub mod overlayfs;
pub mod session;
pub mod stream;

pub use session::SessionWorld;

/// Internal, non-wire evidence for adopting an already-bound generic world as the exact
/// shared-session owner. The correlation identifier is operation-scoped and is never persisted.
#[derive(Debug, Clone)]
pub struct ExactBoundWorldOwnershipAdoptionV1 {
    pub orchestration_session_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub participant_id: String,
    pub policy_ref_id: String,
    pub policy_revision: String,
    pub policy_snapshot_hash: String,
    pub target_spec: WorldSpec,
    pub adoption_correlation_id: String,
}

impl ExactBoundWorldOwnershipAdoptionV1 {
    fn validate(&self) -> Result<()> {
        for (field, value) in [
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("world_id", self.world_id.as_str()),
            ("participant_id", self.participant_id.as_str()),
            ("policy_ref_id", self.policy_ref_id.as_str()),
            ("policy_revision", self.policy_revision.as_str()),
            ("policy_snapshot_hash", self.policy_snapshot_hash.as_str()),
            (
                "adoption_correlation_id",
                self.adoption_correlation_id.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                anyhow::bail!("exact bound-world adoption {field} is empty");
            }
        }
        let mut world_id_components = std::path::Path::new(&self.world_id).components();
        let safe_world_id = matches!(
            (world_id_components.next(), world_id_components.next()),
            (Some(std::path::Component::Normal(_)), None)
        ) && self.world_id.starts_with("wld_");
        if !safe_world_id {
            anyhow::bail!("exact bound-world adoption world_id is not a safe world identifier");
        }
        if self.policy_snapshot_hash.len() != 64
            || !self
                .policy_snapshot_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            anyhow::bail!(
                "exact bound-world adoption policy_snapshot_hash is not lowercase sha256"
            );
        }
        if self.policy_revision != self.policy_snapshot_hash {
            anyhow::bail!(
                "exact bound-world adoption policy revision does not match canonical snapshot"
            );
        }
        let owner =
            self.target_spec.reuse_mode.shared_owner().ok_or_else(|| {
                anyhow::anyhow!("exact bound-world adoption target is not shared")
            })?;
        if owner.orchestration_session_id != self.orchestration_session_id
            || !matches!(owner.action, SharedWorldOwnerAction::AttachOrCreate)
        {
            anyhow::bail!("exact bound-world adoption target owner does not match evidence");
        }
        Ok(())
    }
}

/// Linux native backend using namespaces, cgroups, and nftables.
#[derive(Default)]
pub struct LinuxLocalBackend {
    session_cache: std::sync::RwLock<std::collections::HashMap<String, SessionWorld>>,
    shared_owner_mutex: std::sync::Mutex<()>,
}

impl LinuxLocalBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a compatible cached session if one already exists without creating a new world.
    pub fn find_compatible_session(&self, spec: &WorldSpec) -> Result<Option<WorldHandle>> {
        let root_dir = SessionWorld::shared_root_dir();
        self.find_compatible_session_with_root_lock_from_root(&root_dir, spec)
    }

    fn find_compatible_session_with_root_lock_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
    ) -> Result<Option<WorldHandle>> {
        if spec.reuse_mode.shared_owner().is_some() {
            let _guard = self
                .shared_owner_mutex
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire shared owner mutex: {}", e))?;
            let _durable_owner_lock = SessionWorld::lock_shared_root_for_ownership(root_dir)?;
            return self.find_compatible_session_from_root(root_dir, spec, false);
        }
        self.find_compatible_session_from_root(root_dir, spec, false)
    }

    fn world_handle(world: &SessionWorld) -> WorldHandle {
        WorldHandle {
            id: world.id.clone(),
            shared_binding: world.shared_binding(),
        }
    }

    fn find_compatible_session_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        update_fs_mode: bool,
    ) -> Result<Option<WorldHandle>> {
        if let Some(owner_spec) = spec.reuse_mode.shared_owner() {
            return self.find_shared_owner_session_from_root(
                root_dir,
                spec,
                owner_spec,
                update_fs_mode,
            );
        }

        if !spec.reuse_session {
            return Ok(None);
        }

        self.find_generic_session_from_root(root_dir, spec, update_fs_mode)
    }

    fn find_generic_session_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        update_fs_mode: bool,
    ) -> Result<Option<WorldHandle>> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;

        if let Some(world) = cache
            .values_mut()
            .find(|world| world.is_generic_reusable_with(spec))
        {
            if update_fs_mode {
                world.spec.fs_mode = spec.fs_mode;
            }
            return Ok(Some(Self::world_handle(world)));
        }

        let Some(mut world) = SessionWorld::recover_generic_compatible_from_root(root_dir, spec)?
        else {
            return Ok(None);
        };
        if update_fs_mode {
            world.spec.fs_mode = spec.fs_mode;
        }
        let handle = Self::world_handle(&world);
        cache.insert(world.id.clone(), world);
        Ok(Some(handle))
    }

    fn find_shared_owner_session_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        update_fs_mode: bool,
    ) -> Result<Option<WorldHandle>> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;

        let matching_ids = cache
            .iter()
            .filter_map(|(world_id, world)| {
                world
                    .is_shared_owner_reusable_with(spec, owner_spec)
                    .then_some(world_id.clone())
            })
            .collect::<Vec<_>>();
        match matching_ids.as_slice() {
            [] => {}
            [world_id] => {
                let world = cache
                    .get_mut(world_id)
                    .context("shared world missing from cache during reuse")?;
                if update_fs_mode {
                    world.spec.fs_mode = spec.fs_mode;
                }
                return Ok(Some(Self::world_handle(world)));
            }
            _ => {
                anyhow::bail!(
                    "multiple active shared worlds found for orchestration session {}",
                    owner_spec.orchestration_session_id
                );
            }
        }

        let Some(mut world) =
            SessionWorld::recover_shared_active_from_root(root_dir, spec, owner_spec)?
        else {
            return Ok(None);
        };
        if update_fs_mode {
            world.spec.fs_mode = spec.fs_mode;
        }
        let handle = Self::world_handle(&world);
        cache.insert(world.id.clone(), world);
        Ok(Some(handle))
    }

    fn create_generic_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.create_generic_session_in_root(&SessionWorld::shared_root_dir(), spec)
    }

    fn create_generic_session_in_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
    ) -> Result<WorldHandle> {
        let world = SessionWorld::ensure_started_in_root(spec.clone(), root_dir.to_path_buf())
            .context("Failed to create session world")?;
        let handle = Self::world_handle(&world);
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        cache.insert(world.id.clone(), world);
        Ok(handle)
    }

    fn create_shared_owner_session_in_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        world_generation: u64,
        last_restart_reason: Option<String>,
    ) -> Result<WorldHandle> {
        let world = SessionWorld::ensure_started_for_shared_owner_at_root(
            root_dir.to_path_buf(),
            spec.clone(),
            owner_spec.orchestration_session_id.clone(),
            world_generation,
            last_restart_reason,
        )
        .context("Failed to create shared session world")?;
        let handle = Self::world_handle(&world);
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        cache.insert(world.id.clone(), world);
        Ok(handle)
    }

    fn create_shared_owner_session_in_root_with_world_id(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        world_generation: u64,
        last_restart_reason: Option<String>,
        world_id: String,
    ) -> Result<WorldHandle> {
        let world = SessionWorld::ensure_started_for_shared_owner_at_root_with_world_id(
            root_dir.to_path_buf(),
            spec.clone(),
            owner_spec.orchestration_session_id.clone(),
            world_generation,
            last_restart_reason,
            world_id,
        )
        .context("Failed to create shared session world")?;
        let handle = Self::world_handle(&world);
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        cache.insert(world.id.clone(), world);
        Ok(handle)
    }

    fn replace_shared_owner_session_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        expected_generation: u64,
        reason: String,
    ) -> Result<WorldHandle> {
        let replacement_reason = reason.clone();
        self.replace_shared_owner_session_from_root_with_creator(
            root_dir,
            spec,
            owner_spec,
            expected_generation,
            reason,
            |replacement_world_id| {
                self.create_shared_owner_session_in_root_with_world_id(
                    root_dir,
                    spec,
                    owner_spec,
                    expected_generation + 1,
                    Some(replacement_reason),
                    replacement_world_id,
                )
            },
        )
    }

    fn replace_shared_owner_session_from_root_with_creator<F>(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        expected_generation: u64,
        reason: String,
        create_replacement: F,
    ) -> Result<WorldHandle>
    where
        F: FnOnce(String) -> Result<WorldHandle>,
    {
        let handle = self
            .find_shared_owner_session_from_root(root_dir, spec, owner_spec, true)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no active shared world found for orchestration session {}",
                    owner_spec.orchestration_session_id
                )
            })?;
        let current_generation = handle
            .shared_binding
            .as_ref()
            .map(|binding| binding.world_generation)
            .ok_or_else(|| anyhow::anyhow!("active shared world missing binding proof"))?;
        if current_generation != expected_generation {
            anyhow::bail!(
                "shared world generation conflict for {}: expected {}, found {}",
                owner_spec.orchestration_session_id,
                expected_generation,
                current_generation
            );
        }

        let replacement_world_id = format!("wld_{}", uuid::Uuid::now_v7());

        {
            let mut cache = self.session_cache.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire session cache write lock: {}", e)
            })?;
            let world = cache
                .get_mut(&handle.id)
                .context("replaced shared world missing from cache")?;
            world.set_shared_binding_state(
                world_api::SharedWorldBindingState::Replacing,
                Some(reason.clone()),
            )?;
        }

        let replacement_handle = match create_replacement(replacement_world_id.clone()) {
            Ok(handle) => handle,
            Err(create_err) => {
                let rollback_err = {
                    let mut cache = self.session_cache.write().map_err(|e| {
                        anyhow::anyhow!("Failed to acquire session cache write lock: {}", e)
                    })?;
                    let world = cache
                        .get_mut(&handle.id)
                        .context("rollback shared world missing from cache")?;
                    world.set_shared_binding_state(world_api::SharedWorldBindingState::Active, None)
                };

                let cleanup_err = match rollback_err {
                    Ok(()) => {
                        self.cleanup_partial_shared_world_root(root_dir, &replacement_world_id)
                    }
                    Err(_) => Ok(()),
                };

                let mut message = format!("failed to create replacement world: {create_err:#}");
                if let Err(err) = rollback_err {
                    message.push_str(&format!("; rollback failed: {err:#}"));
                }
                if let Err(err) = cleanup_err {
                    message.push_str(&format!("; cleanup failed: {err:#}"));
                }
                return Err(anyhow::anyhow!(message));
            }
        };

        let finalize_result = {
            let mut cache = self.session_cache.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire session cache write lock: {}", e)
            })?;
            let world = cache
                .get_mut(&handle.id)
                .context("finalized shared world missing from cache")?;
            world.set_shared_binding_state(
                world_api::SharedWorldBindingState::Replaced,
                Some(reason),
            )
        };
        if let Err(err) = finalize_result {
            tracing::warn!(
                error = %err,
                world_id = %handle.id,
                "shared world replacement committed but old world finalize failed"
            );
        }

        Ok(replacement_handle)
    }

    fn cleanup_partial_shared_world_root(
        &self,
        root_dir: &std::path::Path,
        world_id: &str,
    ) -> Result<()> {
        let partial_root = root_dir.join(world_id);
        match std::fs::remove_dir_all(&partial_root) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => {
                Err(err).with_context(|| format!("failed to remove {}", partial_root.display()))
            }
        }
    }

    fn ensure_shared_owner_session_from_root(
        &self,
        root_dir: &std::path::Path,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
    ) -> Result<WorldHandle> {
        let _guard = self
            .shared_owner_mutex
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire shared owner mutex: {}", e))?;
        let _durable_owner_lock = SessionWorld::lock_shared_root_for_ownership(root_dir)?;

        match &owner_spec.action {
            SharedWorldOwnerAction::AttachOrCreate => {
                if let Some(handle) =
                    self.find_shared_owner_session_from_root(root_dir, spec, owner_spec, true)?
                {
                    return Ok(handle);
                }
                self.create_shared_owner_session_in_root(root_dir, spec, owner_spec, 0, None)
            }
            SharedWorldOwnerAction::ReplaceExpectedGeneration {
                expected_generation,
                reason,
            } => self.replace_shared_owner_session_from_root(
                root_dir,
                spec,
                owner_spec,
                *expected_generation,
                reason.clone(),
            ),
        }
    }

    /// Adopt the exact already-created HSA-bound world as the shared-session owner.
    ///
    /// Unlike `ensure_session`, this operation never creates or replaces a world.
    pub fn adopt_exact_bound_world_ownership(
        &self,
        adoption: &ExactBoundWorldOwnershipAdoptionV1,
    ) -> Result<WorldHandle> {
        self.check_platform()?;
        self.adopt_exact_bound_world_ownership_from_root(&SessionWorld::shared_root_dir(), adoption)
    }

    fn adopt_exact_bound_world_ownership_from_root(
        &self,
        root_dir: &std::path::Path,
        adoption: &ExactBoundWorldOwnershipAdoptionV1,
    ) -> Result<WorldHandle> {
        adoption.validate()?;
        let _guard = self
            .shared_owner_mutex
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire shared owner mutex: {}", e))?;
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;

        if !cache.contains_key(&adoption.world_id) {
            let recovered = SessionWorld::recover_exact_bound_world_from_root(
                root_dir,
                &adoption.target_spec,
                &adoption.world_id,
            )?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "exact bound-world adoption target {} is missing",
                    adoption.world_id
                )
            })?;
            cache.insert(adoption.world_id.clone(), recovered);
        }

        let world = cache
            .get_mut(&adoption.world_id)
            .context("exact bound-world adoption target disappeared from cache")?;
        world.adopt_exact_bound_world_ownership(adoption)?;
        Ok(Self::world_handle(world))
    }

    /// Ensure the overlay for a world is mounted and return its merged root.
    pub fn ensure_overlay_root(&self, world: &WorldHandle) -> Result<std::path::PathBuf> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        let session_world = cache
            .get_mut(&world.id)
            .context("World not found in cache")?;

        session_world.ensure_overlay_root()
    }

    pub fn refresh_network_filter(&self, world: &WorldHandle) -> Result<()> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        let session_world = cache
            .get_mut(&world.id)
            .context("World not found in cache")?;
        session_world.refresh_network_filter()
    }

    pub fn cgroup_path(&self, world: &WorldHandle) -> Result<std::path::PathBuf> {
        let cache = self
            .session_cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache read lock: {}", e))?;
        let session_world = cache.get(&world.id).context("World not found in cache")?;
        Ok(session_world.cgroup_path())
    }

    /// Retrieve the current session's pending diff and session start time.
    pub fn pending_diff(&self, world: &WorldHandle) -> Result<(std::time::SystemTime, FsDiff)> {
        let cache = self
            .session_cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache read lock: {}", e))?;
        let session_world = cache.get(&world.id).context("World not found in cache")?;
        let diff = session_world.compute_pending_diff()?;
        Ok((session_world.started_at, diff))
    }

    /// Clear the current session's pending diff state (discard overlay upper/work layers).
    pub fn clear_pending_diff(&self, world: &WorldHandle) -> Result<()> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        let session_world = cache
            .get_mut(&world.id)
            .context("World not found in cache")?;
        session_world.clear_pending_diff()
    }

    /// Discard the overlay upper entries for specific workspace-relative paths.
    pub fn discard_pending_paths(
        &self,
        world: &WorldHandle,
        paths: &[std::path::PathBuf],
    ) -> Result<u32> {
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        let session_world = cache
            .get_mut(&world.id)
            .context("World not found in cache")?;
        session_world.discard_pending_paths(paths)
    }

    #[cfg(not(target_os = "linux"))]
    fn check_platform(&self) -> Result<()> {
        anyhow::bail!("LinuxLocal backend is only supported on Linux")
    }

    #[cfg(target_os = "linux")]
    fn check_platform(&self) -> Result<()> {
        Ok(())
    }
}

impl WorldBackend for LinuxLocalBackend {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.check_platform()?;

        match spec.reuse_mode.shared_owner() {
            Some(owner_spec) => self.ensure_shared_owner_session_from_root(
                &SessionWorld::shared_root_dir(),
                spec,
                owner_spec,
            ),
            None => {
                if spec.reuse_session {
                    if let Some(handle) = self.find_compatible_session_from_root(
                        &SessionWorld::shared_root_dir(),
                        spec,
                        true,
                    )? {
                        return Ok(handle);
                    }
                }

                self.create_generic_session(spec)
            }
        }
    }

    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        self.check_platform()?;

        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        let session_world = cache
            .get_mut(&world.id)
            .context("World not found in cache")?;

        session_world.execute(&req.cmd, &req.cwd, req.env, req.pty, req.span_id)
    }

    fn fs_diff(&self, world: &WorldHandle, span_id: &str) -> Result<FsDiff> {
        self.check_platform()?;

        let cache = self
            .session_cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache read lock: {}", e))?;
        let session_world = cache.get(&world.id).context("World not found in cache")?;

        session_world.compute_fs_diff(span_id)
    }

    fn apply_policy(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()> {
        self.check_platform()?;

        let cache = self
            .session_cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache read lock: {}", e))?;
        let session_world = cache.get(&world.id).context("World not found in cache")?;

        session_world.apply_policy(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::collections::HashMap;
    #[cfg(target_os = "linux")]
    use std::sync::RwLock;
    #[cfg(target_os = "linux")]
    use std::sync::{mpsc, Arc};
    #[cfg(target_os = "linux")]
    use std::time::Duration;
    use tempfile::tempdir;

    #[cfg(target_os = "linux")]
    fn shared_owner_spec(action: SharedWorldOwnerAction) -> SharedWorldOwnerSpec {
        SharedWorldOwnerSpec {
            orchestration_session_id: "orch_123".into(),
            action,
        }
    }

    #[cfg(target_os = "linux")]
    fn shared_world_spec(
        project_dir: &std::path::Path,
        action: SharedWorldOwnerAction,
    ) -> WorldSpec {
        WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(shared_owner_spec(action)),
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.to_path_buf(),
            always_isolate: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        }
    }

    #[test]
    fn test_backend_creation() {
        let backend = LinuxLocalBackend::new();
        assert!(backend.session_cache.read().unwrap().is_empty());
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_platform_check_fails_on_non_linux() {
        let backend = LinuxLocalBackend::new();
        assert!(backend.check_platform().is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_platform_check_succeeds_on_linux() {
        let backend = LinuxLocalBackend::new();
        assert!(backend.check_platform().is_ok());
    }

    #[test]
    fn cache_miss_with_valid_metadata_repopulates_backend_cache() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_recovered");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();
        std::fs::create_dir_all(root_dir.join("wld_recovered")).unwrap();

        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: project_dir.clone(),
            always_isolate: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        std::fs::write(
            root_dir.join("wld_recovered").join("session.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "world_id": "wld_recovered",
                "project_dir": project_dir,
                "isolate_network": false,
                "always_isolate": false,
                "allowed_domains": ["example.com"],
                "cgroup_path": cgroup_path,
                "started_at_unix_millis": 5000,
            }))
            .unwrap(),
        )
        .unwrap();

        let backend = LinuxLocalBackend::new();
        let handle = backend
            .find_compatible_session_from_root(&root_dir, &spec, false)
            .unwrap()
            .expect("expected recovered session handle");
        assert_eq!(handle.id, "wld_recovered");
        assert!(backend
            .session_cache
            .read()
            .unwrap()
            .contains_key(&handle.id));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn replace_success_commits_new_active_and_finalizes_old_world() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let owner_spec = shared_owner_spec(SharedWorldOwnerAction::AttachOrCreate);
        let spec = shared_world_spec(&project_dir, owner_spec.action.clone());
        let original = SessionWorld::ensure_started_for_shared_owner_at_root_with_world_id(
            root_dir.clone(),
            spec.clone(),
            owner_spec.orchestration_session_id.clone(),
            0,
            None,
            "wld_original".into(),
        )
        .unwrap();
        original.persist_metadata().unwrap();

        let backend = LinuxLocalBackend::new();
        let handle = backend
            .replace_shared_owner_session_from_root(
                &root_dir,
                &spec,
                &owner_spec,
                0,
                "restart".into(),
            )
            .unwrap();

        let binding = handle.shared_binding.expect("replacement shared binding");
        assert_eq!(
            binding.binding_state,
            world_api::SharedWorldBindingState::Active
        );
        assert_eq!(binding.world_generation, 1);
        assert_ne!(binding.world_id, "wld_original");

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec)
                .unwrap()
                .expect("active world should recover");
        assert_eq!(
            recovered.shared_binding().unwrap().world_generation,
            1,
            "recovery should prefer the committed replacement"
        );

        let previous_metadata = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root_dir.join("wld_original").join("session.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(previous_metadata["binding_state"], "replaced");
        assert_eq!(previous_metadata["world_generation"], 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn replace_failure_rolls_back_old_world_and_cleans_partial_root() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let owner_spec = shared_owner_spec(SharedWorldOwnerAction::AttachOrCreate);
        let spec = shared_world_spec(&project_dir, owner_spec.action.clone());
        let original = SessionWorld::ensure_started_for_shared_owner_at_root_with_world_id(
            root_dir.clone(),
            spec.clone(),
            owner_spec.orchestration_session_id.clone(),
            0,
            None,
            "wld_original".into(),
        )
        .unwrap();
        original.persist_metadata().unwrap();

        let backend = LinuxLocalBackend::new();
        let err = backend
            .replace_shared_owner_session_from_root_with_creator(
                &root_dir,
                &spec,
                &owner_spec,
                0,
                "restart".into(),
                |replacement_world_id| {
                    std::fs::create_dir_all(root_dir.join(&replacement_world_id)).unwrap();
                    anyhow::bail!("boom")
                },
            )
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("failed to create replacement world: boom"),
            "unexpected error: {err:#}"
        );

        let recovered =
            SessionWorld::recover_shared_active_from_root(&root_dir, &spec, &owner_spec)
                .unwrap()
                .expect("original world should still recover");
        let binding = recovered.shared_binding().unwrap();
        assert_eq!(binding.world_id, "wld_original");
        assert_eq!(binding.world_generation, 0);
        assert_eq!(
            binding.binding_state,
            world_api::SharedWorldBindingState::Active
        );

        let previous_metadata = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(root_dir.join("wld_original").join("session.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(previous_metadata["binding_state"], "active");
        assert!(previous_metadata["last_restart_reason"].is_null());

        let replacement_roots = std::fs::read_dir(&root_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != "wld_original")
            .collect::<Vec<_>>();
        assert!(
            replacement_roots.is_empty(),
            "partial replacement roots should be cleaned up: {replacement_roots:?}"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn shared_owner_branch_waits_on_backend_mutex() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();

        let spec = shared_world_spec(&project_dir, SharedWorldOwnerAction::AttachOrCreate);
        let backend = Arc::new(LinuxLocalBackend::new());
        let guard = backend.shared_owner_mutex.lock().unwrap();
        let (tx, rx) = mpsc::channel();
        let owner_spec = spec.reuse_mode.shared_owner().unwrap().clone();

        let backend_for_thread = Arc::clone(&backend);
        let root_for_thread = root_dir.clone();
        let spec_for_thread = spec.clone();
        let owner_spec_for_thread = owner_spec.clone();
        let worker = std::thread::spawn(move || {
            let result = backend_for_thread.ensure_shared_owner_session_from_root(
                &root_for_thread,
                &spec_for_thread,
                &owner_spec_for_thread,
            );
            tx.send(result).unwrap();
        });

        assert!(
            rx.recv_timeout(Duration::from_millis(100)).is_err(),
            "shared-owner request should block while the backend mutex is held"
        );
        drop(guard);

        let handle = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("worker should complete once the mutex is released")
            .unwrap();
        let second = backend
            .ensure_shared_owner_session_from_root(&root_dir, &spec, &owner_spec)
            .unwrap();
        assert_eq!(handle.id, second.id);

        worker.join().unwrap();
    }

    #[cfg(target_os = "linux")]
    fn poison_cache(cache: &RwLock<HashMap<String, SessionWorld>>) {
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let _guard = cache.write().unwrap();
                    panic!("poison cache lock");
                })
                .join()
                .ok();
        });
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn poisoned_cache_returns_error_in_fs_diff() {
        let backend = LinuxLocalBackend::new();
        poison_cache(&backend.session_cache);
        let handle = WorldHandle {
            id: "missing".to_string(),
            shared_binding: None,
        };

        let result = std::panic::catch_unwind(|| backend.fs_diff(&handle, "span"));
        assert!(result.is_ok(), "fs_diff panicked on poisoned cache");

        let err = result
            .unwrap()
            .expect_err("expected error from poisoned cache");
        assert!(
            err.to_string()
                .contains("Failed to acquire session cache read lock")
                || err.to_string().contains("poison"),
            "unexpected error: {err}"
        );

        backend.session_cache.clear_poison();
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn backend_adopts_only_the_exact_bound_generic_world_without_creating_another() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        let generic_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::GenericCompatible,
            project_dir: project_dir.clone(),
            isolate_network: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let backend = LinuxLocalBackend::new();
        let generic = backend
            .create_generic_session_in_root(&root_dir, &generic_spec)
            .expect("generic exact HSA-bound world");
        let root_entries_before = std::fs::read_dir(&root_dir).unwrap().count();
        let target_spec = WorldSpec {
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_exact".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            ..generic_spec
        };
        let adoption = ExactBoundWorldOwnershipAdoptionV1 {
            orchestration_session_id: "orch_exact".into(),
            world_id: generic.id.clone(),
            world_generation: 4,
            participant_id: "rwp_exact".into(),
            policy_ref_id: "ao_policy_exact".into(),
            policy_revision: "a".repeat(64),
            policy_snapshot_hash: "a".repeat(64),
            target_spec,
            adoption_correlation_id: "transport_claim_123".into(),
        };

        let first = backend
            .adopt_exact_bound_world_ownership_from_root(&root_dir, &adoption)
            .expect("first adoption");
        let second = backend
            .adopt_exact_bound_world_ownership_from_root(&root_dir, &adoption)
            .expect("exact retry");

        assert_eq!(first.id, second.id);
        assert_eq!(first.shared_binding, second.shared_binding);
        assert_eq!(first.id, generic.id);
        assert_eq!(first.shared_binding.unwrap().world_generation, 4);
        assert_eq!(
            std::fs::read_dir(&root_dir).unwrap().count(),
            root_entries_before
        );
        assert_eq!(backend.session_cache.read().unwrap().len(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn concurrent_exact_bound_world_adoptions_join_one_owner() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        let generic_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::GenericCompatible,
            project_dir: project_dir.clone(),
            isolate_network: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let backend = Arc::new(LinuxLocalBackend::new());
        let generic = backend
            .create_generic_session_in_root(&root_dir, &generic_spec)
            .expect("generic world");
        let adoption = Arc::new(ExactBoundWorldOwnershipAdoptionV1 {
            orchestration_session_id: "orch_exact".into(),
            world_id: generic.id,
            world_generation: 9,
            participant_id: "rwp_exact".into(),
            policy_ref_id: "ao_policy_exact".into(),
            policy_revision: "a".repeat(64),
            policy_snapshot_hash: "a".repeat(64),
            target_spec: WorldSpec {
                reuse_mode: world_api::WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_exact".into(),
                    action: SharedWorldOwnerAction::AttachOrCreate,
                }),
                ..generic_spec
            },
            adoption_correlation_id: "transport_claim_123".into(),
        });

        let mut workers = Vec::new();
        for _ in 0..2 {
            let backend = Arc::clone(&backend);
            let root_dir = root_dir.clone();
            let adoption = Arc::clone(&adoption);
            workers.push(std::thread::spawn(move || {
                backend.adopt_exact_bound_world_ownership_from_root(&root_dir, &adoption)
            }));
        }
        let handles = workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(handles[0].id, handles[1].id);
        assert_eq!(handles[0].shared_binding, handles[1].shared_binding);
        assert_eq!(std::fs::read_dir(&root_dir).unwrap().count(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn separate_backend_instances_join_exact_adoption_and_conflicts_cannot_steal() {
        fn generic_fixture(
            root_dir: &std::path::Path,
            project_dir: &std::path::Path,
        ) -> (WorldSpec, WorldHandle) {
            std::fs::create_dir_all(root_dir).unwrap();
            std::fs::create_dir_all(project_dir).unwrap();
            let spec = WorldSpec {
                reuse_session: true,
                reuse_mode: world_api::WorldReuseMode::GenericCompatible,
                project_dir: project_dir.to_path_buf(),
                isolate_network: false,
                fs_mode: world_api::WorldFsMode::Writable,
                ..WorldSpec::default()
            };
            let handle = LinuxLocalBackend::new()
                .create_generic_session_in_root(root_dir, &spec)
                .expect("generic exact world");
            (spec, handle)
        }

        fn adoption_for(
            generic_spec: &WorldSpec,
            world_id: &str,
            session_id: &str,
            policy_marker: char,
        ) -> ExactBoundWorldOwnershipAdoptionV1 {
            ExactBoundWorldOwnershipAdoptionV1 {
                orchestration_session_id: session_id.into(),
                world_id: world_id.into(),
                world_generation: 3,
                participant_id: "rwp_exact".into(),
                policy_ref_id: format!("ao_policy_{session_id}"),
                policy_revision: policy_marker.to_string().repeat(64),
                policy_snapshot_hash: policy_marker.to_string().repeat(64),
                target_spec: WorldSpec {
                    reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                        SharedWorldOwnerSpec {
                            orchestration_session_id: session_id.into(),
                            action: SharedWorldOwnerAction::AttachOrCreate,
                        },
                    ),
                    ..generic_spec.clone()
                },
                adoption_correlation_id: format!("rtc_{session_id}"),
            }
        }

        let exact_temp = tempdir().unwrap();
        let exact_root = exact_temp.path().join("world-root");
        let exact_project = exact_temp.path().join("project");
        let (generic_spec, generic) = generic_fixture(&exact_root, &exact_project);
        let exact_adoption = Arc::new(adoption_for(&generic_spec, &generic.id, "orch_exact", 'a'));
        let exact_barrier = Arc::new(std::sync::Barrier::new(3));
        let mut exact_workers = Vec::new();
        for _ in 0..2 {
            let backend = LinuxLocalBackend::new();
            let root = exact_root.clone();
            let adoption = Arc::clone(&exact_adoption);
            let barrier = Arc::clone(&exact_barrier);
            exact_workers.push(std::thread::spawn(move || {
                barrier.wait();
                backend.adopt_exact_bound_world_ownership_from_root(&root, &adoption)
            }));
        }
        exact_barrier.wait();
        let exact_handles = exact_workers
            .into_iter()
            .map(|worker| worker.join().unwrap().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(exact_handles[0].id, exact_handles[1].id);
        assert_eq!(
            exact_handles[0].shared_binding,
            exact_handles[1].shared_binding
        );

        let conflict_temp = tempdir().unwrap();
        let conflict_root = conflict_temp.path().join("world-root");
        let conflict_project = conflict_temp.path().join("project");
        let (generic_spec, generic) = generic_fixture(&conflict_root, &conflict_project);
        let first_adoption = adoption_for(&generic_spec, &generic.id, "orch_first", 'a');
        let second_adoption = adoption_for(&generic_spec, &generic.id, "orch_second", 'b');
        let conflict_barrier = Arc::new(std::sync::Barrier::new(3));
        let mut conflict_workers = Vec::new();
        for adoption in [first_adoption, second_adoption] {
            let backend = LinuxLocalBackend::new();
            let root = conflict_root.clone();
            let barrier = Arc::clone(&conflict_barrier);
            conflict_workers.push(std::thread::spawn(move || {
                barrier.wait();
                backend.adopt_exact_bound_world_ownership_from_root(&root, &adoption)
            }));
        }
        conflict_barrier.wait();
        let results = conflict_workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);

        let winner = results
            .iter()
            .find_map(|result| result.as_ref().ok())
            .unwrap();
        let metadata = serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(conflict_root.join(&generic.id).join("session.json")).unwrap(),
        )
        .unwrap();
        let winner_binding = winner.shared_binding.as_ref().unwrap();
        assert_eq!(
            metadata["orchestration_session_id"],
            winner_binding.orchestration_session_id
        );
        assert_eq!(metadata["world_id"], generic.id);
        assert_eq!(std::fs::read_dir(&conflict_root).unwrap().count(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn separate_world_ids_cannot_both_own_one_orchestration_session() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        let generic_spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::GenericCompatible,
            project_dir,
            isolate_network: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let creator = LinuxLocalBackend::new();
        let first = creator
            .create_generic_session_in_root(&root_dir, &generic_spec)
            .unwrap();
        let second = creator
            .create_generic_session_in_root(&root_dir, &generic_spec)
            .unwrap();
        assert_ne!(first.id, second.id);

        let adoption = |world_id: String, correlation: &str| ExactBoundWorldOwnershipAdoptionV1 {
            orchestration_session_id: "orch_one_owner".into(),
            world_id,
            world_generation: 5,
            participant_id: "rwp_exact".into(),
            policy_ref_id: "ao_policy_exact".into(),
            policy_revision: "a".repeat(64),
            policy_snapshot_hash: "a".repeat(64),
            target_spec: WorldSpec {
                reuse_mode: world_api::WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_one_owner".into(),
                    action: SharedWorldOwnerAction::AttachOrCreate,
                }),
                ..generic_spec.clone()
            },
            adoption_correlation_id: correlation.into(),
        };
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let mut workers = Vec::new();
        for adoption in [
            adoption(first.id, "rtc_first"),
            adoption(second.id, "rtc_second"),
        ] {
            let backend = LinuxLocalBackend::new();
            let root = root_dir.clone();
            let barrier = Arc::clone(&barrier);
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                backend.adopt_exact_bound_world_ownership_from_root(&root, &adoption)
            }));
        }
        barrier.wait();
        let results = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn compatibility_shared_owner_creation_waits_on_durable_root_lock() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        let spec = WorldSpec {
            reuse_session: true,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_root_lock".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            project_dir,
            isolate_network: false,
            fs_mode: world_api::WorldFsMode::Writable,
            ..WorldSpec::default()
        };
        let owner = spec.reuse_mode.shared_owner().unwrap().clone();
        let lookup_spec = spec.clone();
        let durable_lock = SessionWorld::lock_shared_root_for_ownership(&root_dir).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let worker_root = root_dir.clone();
        let worker = std::thread::spawn(move || {
            let result = LinuxLocalBackend::new().ensure_shared_owner_session_from_root(
                &worker_root,
                &spec,
                &owner,
            );
            tx.send(result).unwrap();
        });

        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_millis(100)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        drop(durable_lock);
        let handle = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("shared creation resumes after durable root lock")
            .expect("shared creation succeeds");
        worker.join().unwrap();
        assert_eq!(
            handle.shared_binding.unwrap().orchestration_session_id,
            "orch_root_lock"
        );

        let durable_lock = SessionWorld::lock_shared_root_for_ownership(&root_dir).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let lookup_root = root_dir.clone();
        let lookup = std::thread::spawn(move || {
            tx.send(
                LinuxLocalBackend::new()
                    .find_compatible_session_with_root_lock_from_root(&lookup_root, &lookup_spec),
            )
            .unwrap();
        });
        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_millis(100)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        drop(durable_lock);
        assert!(rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("shared lookup resumes after durable root lock")
            .expect("shared lookup succeeds")
            .is_some());
        lookup.join().unwrap();
    }
}
