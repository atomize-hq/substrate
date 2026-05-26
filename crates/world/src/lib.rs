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
        self.find_compatible_session_from_root(&SessionWorld::shared_root_dir(), spec, false)
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
}
