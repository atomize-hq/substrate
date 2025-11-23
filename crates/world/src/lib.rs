//! Linux native world backend implementation.
//!
//! This crate provides the LinuxLocal backend that implements world isolation
//! using Linux namespaces, cgroups v2, nftables, and other native security features.

use anyhow::{Context, Result};
use world_api::{ExecRequest, ExecResult, FsDiff, WorldBackend, WorldHandle, WorldSpec};

pub mod cgroups;
pub mod copydiff;
pub mod diff;
pub mod dns;
pub mod exec;
pub mod isolation;
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

        if spec.reuse_session {
            // Try to find existing session
            let cache = self
                .session_cache
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire session cache read lock: {}", e))?;
            if let Some(world) = cache.values().next() {
                return Ok(WorldHandle {
                    id: world.id.clone(),
                });
            }
        }

        // Create new session world
        let world =
            SessionWorld::ensure_started(spec.clone()).context("Failed to create session world")?;

        let handle = WorldHandle {
            id: world.id.clone(),
        };

        let mut cache = self
            .session_cache
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire session cache write lock: {}", e))?;
        cache.insert(world.id.clone(), world);

        Ok(handle)
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
