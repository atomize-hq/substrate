//! Session world implementation for Linux.

use crate::overlayfs::OverlayFs;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use world_api::{ExecResult, FsDiff, WorldFsMode, WorldSpec};

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
            started_at: SystemTime::now(),
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
                return Ok(());
            }
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
            output = self.execute_with_overlay_helpers(
                &command_to_run,
                cwd,
                &env,
                &merged_dir,
                &desired_cwd,
                require_cgroup_attach,
            )?;

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
            output = crate::exec::execute_shell_command(&command_to_run, cwd, &env, false)
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

    fn execute_with_overlay_helpers(
        &self,
        command_to_run: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        merged_dir: &Path,
        desired_cwd: &Path,
        require_cgroup_attach: bool,
    ) -> Result<std::process::Output> {
        let project_attach_policy = if require_cgroup_attach {
            crate::exec::CgroupAttachPolicy::required(
                "project_bind_mount",
                self.cgroup_path.as_path(),
            )
        } else {
            crate::exec::CgroupAttachPolicy::optional("project_bind_mount")
        };

        match crate::exec::execute_shell_command_with_project_bind_mount(
            command_to_run,
            crate::exec::ProjectBindMount {
                merged_dir,
                project_dir: &self.project_dir,
                desired_cwd,
                fs_mode: self.spec.fs_mode,
            },
            env,
            false,
            project_attach_policy,
        ) {
            Ok(output)
                if !require_cgroup_attach
                    || !crate::exec::is_cgroup_attach_wrapper_failure(&output.stderr) =>
            {
                Ok(output)
            }
            Ok(output) => self.execute_world_deps_fallback(
                command_to_run,
                cwd,
                env,
                merged_dir,
                anyhow!(
                    "project bind mount helper refused isolated execution before command start: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
                require_cgroup_attach,
            ),
            Err(err) => self.execute_world_deps_fallback(
                command_to_run,
                cwd,
                env,
                merged_dir,
                err,
                require_cgroup_attach,
            ),
        }
    }

    fn execute_world_deps_fallback(
        &self,
        command_to_run: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        merged_dir: &Path,
        primary_err: anyhow::Error,
        require_cgroup_attach: bool,
    ) -> Result<std::process::Output> {
        if self.spec.fs_mode == WorldFsMode::ReadOnly {
            return Err(primary_err).context(
                "failed to enforce read-only world via mount-namespace bind; refusing to run with possible absolute-path escape",
            );
        }

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
        let fallback_world_deps_root =
            crate::exec::stable_world_deps_fallback_root(&self.project_dir);
        let fallback_attach_policy = if require_cgroup_attach {
            crate::exec::CgroupAttachPolicy::required(
                "world_deps_fallback",
                self.cgroup_path.as_path(),
            )
        } else {
            crate::exec::CgroupAttachPolicy::optional("world_deps_fallback")
        };

        match crate::exec::execute_shell_command_with_world_deps_bind_mount(
            command_to_run,
            &target_dir,
            env,
            false,
            &fallback_world_deps_root,
            fallback_attach_policy,
        ) {
            Ok(output)
                if !require_cgroup_attach
                    || !crate::exec::is_cgroup_attach_wrapper_failure(&output.stderr) =>
            {
                Ok(output)
            }
            Ok(output) => Err(primary_err).context(format!(
                "world-deps fallback helper refused isolated execution before command start: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
            Err(world_deps_err) => {
                if require_cgroup_attach {
                    Err(primary_err).context(format!(
                        "world-deps fallback helper also failed: {world_deps_err:#}"
                    ))
                } else {
                    crate::exec::execute_shell_command(command_to_run, &target_dir, env, false)
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
    #[cfg(target_os = "linux")]
    use std::sync::Mutex;
    #[cfg(target_os = "linux")]
    use tempfile::{tempdir, TempDir};

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
            overlay: None,
            overlay_mode: None,
        }
    }

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
            started_at: std::time::SystemTime::UNIX_EPOCH,
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
    fn isolated_helper_flow_does_not_fall_back_to_plain_exec_when_attach_fails() {
        let temp = tempdir().unwrap();
        let world = test_world(&temp, true);
        let merged_dir = temp.path().join("merged");
        std::fs::create_dir_all(&merged_dir).expect("merged dir");

        let err = match world.execute_with_overlay_helpers(
            "printf should-not-run",
            &world.project_dir,
            &HashMap::new(),
            &merged_dir,
            &world.project_dir,
            true,
        ) {
            Ok(output) => {
                if output.status.success() {
                    panic!(
                        "isolated helper flow should not succeed when cgroup attach cannot start"
                    );
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
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
            ),
            "unexpected error: {message}"
        );
        assert!(
            !message.contains("should-not-run"),
            "isolated attach failure should stop before plain command execution: {message}"
        );
    }
}
