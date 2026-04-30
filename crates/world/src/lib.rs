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
        let world =
            SessionWorld::ensure_started(spec.clone()).context("Failed to create session world")?;
        let handle = Self::world_handle(&world);
        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        cache.insert(world.id.clone(), world);
        Ok(handle)
    }

    fn create_shared_owner_session(
        &self,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        world_generation: u64,
        last_restart_reason: Option<String>,
    ) -> Result<WorldHandle> {
        let world = SessionWorld::ensure_started_for_shared_owner(
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

    fn replace_shared_owner_session(
        &self,
        spec: &WorldSpec,
        owner_spec: &SharedWorldOwnerSpec,
        expected_generation: u64,
        reason: String,
    ) -> Result<WorldHandle> {
        let handle = self
            .find_shared_owner_session_from_root(
                &SessionWorld::shared_root_dir(),
                spec,
                owner_spec,
                true,
            )?
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

        {
            let mut cache = self.session_cache.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire session cache write lock: {}", e)
            })?;
            let world = cache
                .get_mut(&handle.id)
                .context("replaced shared world missing from cache")?;
            world.mark_shared_binding_replaced(reason.clone())?;
        }

        self.create_shared_owner_session(spec, owner_spec, expected_generation + 1, Some(reason))
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
            Some(owner_spec) => match &owner_spec.action {
                SharedWorldOwnerAction::AttachOrCreate => {
                    if let Some(handle) = self.find_compatible_session_from_root(
                        &SessionWorld::shared_root_dir(),
                        spec,
                        true,
                    )? {
                        return Ok(handle);
                    }
                    self.create_shared_owner_session(spec, owner_spec, 0, None)
                }
                SharedWorldOwnerAction::ReplaceExpectedGeneration {
                    expected_generation,
                    reason,
                } => self.replace_shared_owner_session(
                    spec,
                    owner_spec,
                    *expected_generation,
                    reason.clone(),
                ),
            },
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
    use tempfile::tempdir;

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

    #[cfg(target_os = "linux")]
    #[test]
    fn cache_miss_with_valid_metadata_repopulates_backend_cache() {
        let temp = tempdir().unwrap();
        let root_dir = temp.path().join("world-root");
        let project_dir = temp.path().join("project");
        let cgroup_path = temp.path().join("cgroup").join("wld_recovered");
        std::fs::create_dir_all(&root_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&cgroup_path).unwrap();
        let metadata_dir = root_dir.join("wld_recovered");
        std::fs::create_dir_all(&metadata_dir).unwrap();

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
            metadata_dir.join("session.json"),
            format!(
                r#"{{
  "world_id": "wld_recovered",
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

    #[cfg(target_os = "linux")]
    #[test]
    fn poisoned_cache_returns_error_in_fs_diff() {
        let backend = LinuxLocalBackend::new();
        poison_cache(&backend.session_cache);
        let handle = WorldHandle {
            id: "missing".to_string(),
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
