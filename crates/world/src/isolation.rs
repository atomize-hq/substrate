//! Linux isolation implementation using namespaces, cgroups, and security features.

#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use world_api::WorldSpec;

#[cfg(target_os = "linux")]
pub struct LinuxIsolation {
    spec: WorldSpec,
}

#[cfg(target_os = "linux")]
impl LinuxIsolation {
    pub fn new(spec: &WorldSpec) -> Self {
        Self { spec: spec.clone() }
    }

    pub fn apply(&self, root_dir: &Path, project_dir: &Path, cgroup_path: &Path) -> Result<()> {
        // Ensure user namespace first so subsequent namespace ops have privileges
        self.setup_user_namespace()?;

        self.setup_mount_namespace(root_dir, project_dir)?;
        self.setup_cgroups(cgroup_path)?;
        self.setup_network_namespace()?;
        // User namespace already set; now drop caps, set NNP, apply seccomp
        self.setup_security_without_userns()?;
        Ok(())
    }

    fn setup_mount_namespace(&self, root_dir: &Path, project_dir: &Path) -> Result<()> {
        // Enter a fresh mount namespace first
        {
            use nix::sched::{unshare, CloneFlags};
            unshare(CloneFlags::CLONE_NEWNS).context("Failed to unshare mount namespace")?;
        }

        // CRITICAL: Prevent mount propagation leaks from host
        self.make_mounts_private()?;
        self.setup_bind_mounts(root_dir, project_dir)?;
        self.pivot_root(root_dir)?;
        self.setup_minimal_filesystem()?;
        Ok(())
    }

    fn make_mounts_private(&self) -> Result<()> {
        use nix::mount::{mount, MsFlags};

        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        )
        .context("Failed to make mounts private")?;

        Ok(())
    }

    fn setup_bind_mounts(&self, root_dir: &Path, project_dir: &Path) -> Result<()> {
        use nix::mount::{mount, MsFlags};

        std::fs::create_dir_all(root_dir)?;

        // Project directory: read-write
        let project_mount = root_dir.join("project");
        std::fs::create_dir_all(&project_mount)?;

        mount(
            Some(project_dir),
            &project_mount,
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )
        .context("Failed to bind mount project directory")?;

        // System directories: read-only (two-step for proper RO)
        for dir in &["/usr", "/bin", "/lib", "/lib64", "/etc"] {
            if Path::new(dir).exists() {
                let target = root_dir.join(dir.trim_start_matches('/'));
                std::fs::create_dir_all(&target)?;

                // Step 1: Bind mount
                mount(
                    Some(*dir),
                    &target,
                    None::<&str>,
                    MsFlags::MS_BIND | MsFlags::MS_REC,
                    None::<&[u8]>,
                )
                .context(format!("Failed to bind mount {}", dir))?;

                // Step 2: Remount as read-only (required for actual RO)
                mount(
                    None::<&str>,
                    &target,
                    None::<&str>,
                    MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY,
                    None::<&[u8]>,
                )
                .context(format!("Failed to remount {} as read-only", dir))?;
            }
        }

        Ok(())
    }

    fn pivot_root(&self, root_dir: &Path) -> Result<()> {
        use nix::unistd::pivot_root;

        let old_root = root_dir.join("old_root");
        std::fs::create_dir_all(&old_root)?;

        pivot_root(root_dir, &old_root).context("Failed to pivot root")?;

        Ok(())
    }

    fn setup_minimal_filesystem(&self) -> Result<()> {
        use nix::mount::MntFlags;
        use nix::mount::{mount, MsFlags, umount2};

        // Mount clean /proc
        std::fs::create_dir_all("/proc")?;
        mount(
            Some("proc"),
            "/proc",
            Some("proc"),
            MsFlags::empty(),
            None::<&[u8]>,
        )
        .context("Failed to mount /proc")?;

        // Create minimal /dev
        self.setup_minimal_dev()?;

        // Unmount old root
        umount2("/old_root", MntFlags::MNT_DETACH).context("Failed to unmount old root")?;
        std::fs::remove_dir_all("/old_root").ok(); // Best effort

        Ok(())
    }

    fn setup_minimal_dev(&self) -> Result<()> {
        use nix::sys::stat::{makedev, mknod, Mode, SFlag};

        std::fs::create_dir_all("/dev")?;

        let devices = [
            ("null", 1, 3),
            ("zero", 1, 5),
            ("urandom", 1, 9),
            ("tty", 5, 0),
        ];

        for (name, major, minor) in devices {
            let path = Path::new("/dev").join(name);
            mknod(
                &path,
                SFlag::S_IFCHR,
                Mode::from_bits_truncate(0o666),
                makedev(major, minor),
            )
            .context(format!("Failed to create device {}", name))?;
        }

        Ok(())
    }

    fn setup_cgroups(&self, cgroup_path: &Path) -> Result<()> {
        // Ensure cgroup v2 is mounted; attempt mount if missing
        let controllers = Path::new("/sys/fs/cgroup/cgroup.controllers");
        if !controllers.exists() {
            // try to mount cgroup2
            #[cfg(target_os = "linux")]
            {
                use nix::mount::{mount, MsFlags};
                let _ = std::fs::create_dir_all("/sys/fs/cgroup");
                if let Err(e) = mount(
                    Some("cgroup2"),
                    "/sys/fs/cgroup",
                    Some("cgroup2"),
                    MsFlags::empty(),
                    None::<&str>,
                ) {
                    eprintln!("⚠️  Failed to mount cgroup v2: {}", e);
                }
            }
        }
        // If still unavailable, degrade gracefully
        if !Path::new("/sys/fs/cgroup/cgroup.controllers").exists() {
            eprintln!("⚠️  cgroups v2 not available, skipping resource limits");
            return Ok(());
        }

        std::fs::create_dir_all(cgroup_path)?;

        // Attach current process to the cgroup
        let pid = std::process::id();
        let procs = cgroup_path.join("cgroup.procs");
        let _ = std::fs::write(&procs, pid.to_string());

        // Apply resource limits
        if let Some(ref memory) = self.spec.limits.memory {
            std::fs::write(cgroup_path.join("memory.max"), memory)
                .context("Failed to set memory limit")?;
        }

        if let Some(ref cpu) = self.spec.limits.cpu {
            let cpus = cpu.parse::<f64>().context("Invalid CPU limit")?;
            let quota = (cpus * 100000.0).round() as u64;
            std::fs::write(cgroup_path.join("cpu.max"), format!("{} 100000", quota))
                .context("Failed to set CPU limit")?;
        }

        Ok(())
    }

    fn setup_network_namespace(&self) -> Result<()> {
        if self.spec.isolate_network {
            #[cfg(target_os = "linux")]
            {
                use nix::sched::{unshare, CloneFlags};
                if let Err(e) = unshare(CloneFlags::CLONE_NEWNET) {
                    eprintln!("⚠️  Failed to unshare network namespace (continuing without netns): {}", e);
                }
            }
        }
        Ok(())
    }

    fn setup_security(&self) -> Result<()> {
        // Backward-compatible wrapper
        self.setup_security_without_userns()
    }

    fn setup_security_without_userns(&self) -> Result<()> {
        self.drop_capabilities()?;
        self.set_no_new_privs()?;
        self.apply_seccomp_baseline()?;
        Ok(())
    }

    fn setup_user_namespace(&self) -> Result<()> {
        use nix::sched::{unshare, CloneFlags};
        use nix::unistd::{getgid, getuid};

        // User namespaces if available (fallback gracefully)
        if let Err(e) = unshare(CloneFlags::CLONE_NEWUSER) {
            eprintln!(
                "⚠️  User namespaces disabled, consider Incus/Docker fallback: {}",
                e
            );
            return Ok(());
        }

        // Must write uid/gid maps before other namespace ops
        let uid = getuid();
        let gid = getgid();

        std::fs::write("/proc/self/setgroups", "deny")?;
        std::fs::write("/proc/self/uid_map", format!("0 {} 1", uid))?;
        std::fs::write("/proc/self/gid_map", format!("0 {} 1", gid))?;

        Ok(())
    }

    fn drop_capabilities(&self) -> Result<()> {
        // TODO: Implement capability dropping
        Ok(())
    }

    fn set_no_new_privs(&self) -> Result<()> {
        // prctl crate names this function `set_no_new_privileges(bool)`
        match prctl::set_no_new_privileges(true) {
            Ok(()) => Ok(()),
            Err(code) => Err(anyhow::anyhow!("Failed to set no_new_privileges: {}", code)),
        }
    }

    fn apply_seccomp_baseline(&self) -> Result<()> {
        // Baseline seccomp using libseccomp: default allow; log risky syscalls
        use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};

        let mut ctx = ScmpFilterContext::new_filter(ScmpAction::Allow)
            .map_err(|e| anyhow::anyhow!("seccomp init failed: {}", e))?;

        // Mark dangerous syscalls to be logged (kernel must support SCMP_ACT_LOG)
        let dangerous = [
            "mount",
            "umount2",
            "pivot_root",
            "keyctl",
            "perf_event_open",
            "bpf",
        ];

        for name in dangerous {
            if let Ok(num) = ScmpSyscall::from_name(name) {
                ctx.add_rule(ScmpAction::Log, num)
                    .map_err(|e| anyhow::anyhow!("seccomp add_rule failed: {}", e))?;
            }
        }

        ctx.load()
            .map_err(|e| anyhow::anyhow!("seccomp load failed: {}", e))?;
        Ok(())
    }
}

// Stub implementation for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub struct LinuxIsolation;

#[cfg(not(target_os = "linux"))]
impl LinuxIsolation {
    pub fn new(_spec: &world_api::WorldSpec) -> Self {
        Self
    }

    pub fn apply(
        &self,
        _root_dir: &std::path::Path,
        _project_dir: &std::path::Path,
        _cgroup_path: &std::path::Path,
    ) -> anyhow::Result<()> {
        anyhow::bail!("Linux isolation not supported on this platform")
    }
}
