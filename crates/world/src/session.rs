//! Session world implementation for Linux.

use crate::overlayfs::OverlayFs;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use world_api::{ExecResult, FsDiff, WorldFsMode, WorldSpec};

/// A reusable Linux world with proper isolation.
pub struct SessionWorld {
    pub id: String,
    pub root_dir: PathBuf,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub net_namespace: Option<String>,
    pub spec: WorldSpec,
    pub network_filter: Option<crate::netfilter::NetFilter>,
    pub fs_by_span: HashMap<String, FsDiff>,
    /// Persistent overlay mount for this session (writable or read-only).
    overlay: Option<OverlayFs>,
    overlay_mode: Option<WorldFsMode>,
}

impl SessionWorld {
    /// Ensure a session world is started and return it.
    pub fn ensure_started(spec: WorldSpec) -> Result<Self> {
        // Check if session world already exists
        if spec.reuse_session {
            if let Some(existing) = Self::find_existing()? {
                return Ok(existing);
            }
        }

        // Create new session world
        let world_id = format!("wld_{}", uuid::Uuid::now_v7());
        let mut world = Self {
            id: world_id.clone(),
            root_dir: PathBuf::from("/tmp/substrate-worlds"),
            project_dir: spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate").join(&world_id),
            net_namespace: None,
            spec,
            network_filter: None,
            fs_by_span: HashMap::new(),
            overlay: None,
            overlay_mode: None,
        };

        world.setup()?;
        Ok(world)
    }

    /// Determine whether this world can be reused for the requested spec.
    pub(crate) fn compatible_with(&self, spec: &WorldSpec) -> bool {
        self.project_dir == spec.project_dir
            && self.spec.isolate_network == spec.isolate_network
            && self.spec.always_isolate == spec.always_isolate
            && self.spec.allowed_domains == spec.allowed_domains
    }

    /// Find an existing session world if available.
    fn find_existing() -> Result<Option<Self>> {
        // TODO: Implement session discovery logic
        // For now, always create new
        Ok(None)
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

            // Create a named network namespace for this session world (best-effort)
            let ns_name = format!("substrate-{}", self.id);
            if crate::netns::NetNs::ip_available() {
                // Create named netns and bring loopback up. Best-effort; ignore failures.
                tracing::info!("world.setup: creating netns {}", ns_name);
                let _ = std::process::Command::new("ip")
                    .args(["netns", "add", &ns_name])
                    .status();
                let _ = std::process::Command::new("ip")
                    .args(["-n", &ns_name, "link", "set", "lo", "up"])
                    .status();
                // Record only if it exists afterwards
                if std::path::Path::new(&format!("/var/run/netns/{}", ns_name)).exists() {
                    self.net_namespace = Some(ns_name);
                }
            }

            // Set up network filtering if enabled (scoped to netns when available)
            if self.spec.isolate_network {
                tracing::info!("world.setup: installing nftables rules");
                if let Err(e) = self.setup_network_filter() {
                    tracing::warn!(
                        "[agent] netfilter setup failed; continuing without network scoping: {}",
                        e
                    );
                }
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
        // Build NetFilter scoped to named netns when available
        let mut filter =
            crate::netfilter::NetFilter::new(&self.id, self.spec.allowed_domains.clone())?;
        filter.set_namespace(self.net_namespace.clone());
        filter.resolve_domains()?;
        filter.install_rules()?;
        self.network_filter = Some(filter);
        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        if let Err(e) = std::fs::create_dir_all(&self.root_dir) {
            tracing::error!(
                error = %e,
                path = %self.root_dir.display(),
                "[world] failed to create world root directory"
            );
            return Err(e).context("Failed to create world root directory");
        }
        if let Err(e) = std::fs::create_dir_all(&self.cgroup_path) {
            tracing::error!(
                error = %e,
                path = %self.cgroup_path.display(),
                "[world] failed to create cgroup directory"
            );
            return Err(e).context("Failed to create cgroup directory");
        }
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
        let scopes_used;
        let mut diff_opt: Option<FsDiff> = None;
        let mut fs_strategy_meta: Option<crate::overlayfs::WorldFsStrategyMeta> = None;

        // When fs_mode is enforced or heuristics request isolation, run against a persistent overlay
        // so state is consistent across commands within this session.
        if self.spec.fs_mode == WorldFsMode::ReadOnly
            || self.spec.fs_mode != WorldFsMode::Writable
            || self.should_isolate_command(cmd)
        {
            let merged_dir = self.ensure_overlay_mounted()?;
            fs_strategy_meta = crate::overlayfs::world_fs_strategy_meta(&self.id);
            let desired_cwd = if cwd.starts_with(&self.project_dir) {
                cwd.to_path_buf()
            } else {
                self.project_dir.clone()
            };
            let mut command_to_run = cmd.to_string();
            if crate::guard::should_guard_anchor(&env) {
                command_to_run = crate::guard::wrap_with_anchor_guard(cmd, &self.project_dir);
            }
            output = match crate::exec::execute_shell_command_with_project_bind_mount(
                &command_to_run,
                crate::exec::ProjectBindMount {
                    merged_dir: &merged_dir,
                    project_dir: &self.project_dir,
                    desired_cwd: &desired_cwd,
                    fs_mode: self.spec.fs_mode,
                },
                &env,
                true,
            ) {
                Ok(output) => output,
                Err(err) => {
                    // If mount namespaces are unavailable (e.g., user namespaces disabled),
                    // enforce safety for read-only mode by failing closed (otherwise the
                    // caller could bypass read-only by using absolute paths).
                    if self.spec.fs_mode == WorldFsMode::ReadOnly {
                        return Err(err).context(
                            "failed to enforce read-only world via mount-namespace bind; refusing to run with possible absolute-path escape",
                        );
                    }

                    // Otherwise, fall back to the older behavior (cwd inside the overlay root).
                    // This preserves functionality but may allow absolute-path escapes into the
                    // host project directory.
                    let mut rel = if cwd.starts_with(&self.project_dir) {
                        cwd.strip_prefix(&self.project_dir)
                            .unwrap_or_else(|_| Path::new("."))
                            .to_path_buf()
                    } else {
                        PathBuf::from(".")
                    };
                    if rel.as_os_str().is_empty() {
                        rel = PathBuf::from(".");
                    }
                    let target_dir = merged_dir.join(&rel);
                    crate::exec::execute_shell_command(&command_to_run, &target_dir, &env, true)
                        .with_context(|| format!("Failed to execute command in overlay after mount-namespace bind failed: {err:#}"))?
                }
            };

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
            output = crate::exec::execute_shell_command(cmd, cwd, &env, false)
                .context("Failed to execute command")?;
        }

        // Track network scopes if filter is active
        if let Some(ref mut filter) = self.network_filter {
            scopes_used = crate::netfilter::monitor_network_scopes(filter)?;
        } else {
            scopes_used = vec![];
        }

        Ok(ExecResult {
            exit: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            scopes_used,
            fs_diff: diff_opt,
            world_fs_strategy_primary: fs_strategy_meta.as_ref().map(|m| m.primary),
            world_fs_strategy_final: fs_strategy_meta.as_ref().map(|m| m.final_strategy),
            world_fs_strategy_fallback_reason: fs_strategy_meta.as_ref().map(|m| m.fallback_reason),
        })
    }

    /// Compute filesystem diff for a span.
    pub fn compute_fs_diff(&self, span_id: &str) -> Result<FsDiff> {
        if let Some(diff) = self.fs_by_span.get(span_id) {
            return Ok(diff.clone());
        }
        Ok(FsDiff::default())
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

    #[test]
    fn test_session_world_creation() {
        let spec = WorldSpec::default();

        // This test should work on all platforms, just with different behavior
        match SessionWorld::ensure_started(spec) {
            Ok(world) => {
                assert!(world.id.starts_with("wld_"));
                assert_eq!(world.root_dir, PathBuf::from("/tmp/substrate-worlds"));
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
            isolate_network: true,
            limits: world_api::ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: vec!["example.com".into()],
            project_dir: PathBuf::from("/tmp/project-a"),
            always_isolate: false,
            fs_mode: world_api::WorldFsMode::Writable,
        };
        let world = SessionWorld {
            id: "wld_test".into(),
            root_dir: PathBuf::from("/tmp/substrate-worlds"),
            project_dir: base_spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate/wld_test"),
            net_namespace: None,
            spec: base_spec.clone(),
            network_filter: None,
            fs_by_span: HashMap::new(),
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
}
