use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
use super::OverlayFs;
#[cfg(target_os = "linux")]
use crate::overlayfs::utils::is_path_mounted;

pub(crate) fn prepare_overlay_dirs(
    upper_dir: &Path,
    work_dir: &Path,
    merged_dir: &Path,
) -> Result<()> {
    std::fs::create_dir_all(upper_dir).context("Failed to create upper directory")?;
    std::fs::create_dir_all(work_dir).context("Failed to create work directory")?;
    std::fs::create_dir_all(merged_dir).context("Failed to create merged directory")?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) fn mount_linux(overlay: &mut OverlayFs, lower_dir: &Path) -> Result<()> {
    use nix::mount::{mount, umount2, MntFlags, MsFlags};
    use std::thread::sleep;
    use std::time::Duration;

    let bind_lower = overlay.overlay_dir.join("lower");
    std::fs::create_dir_all(&bind_lower)?;
    let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
    mount(
        Some(lower_dir),
        &bind_lower,
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>,
    )
    .with_context(|| {
        format!(
            "Failed to bind-mount lower {} -> {}",
            lower_dir.display(),
            bind_lower.display()
        )
    })?;
    overlay.bind_lower_dir = Some(bind_lower.clone());

    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        bind_lower.display(),
        overlay.upper_dir.display(),
        overlay.work_dir.display()
    );

    match mount(
        Some("overlay"),
        &overlay.merged_dir,
        Some("overlay"),
        MsFlags::empty(),
        Some(options.as_bytes()),
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            let fuse_bin = which::which("fuse-overlayfs").map_err(|which_err| {
                anyhow::anyhow!(
                    "Failed to mount overlayfs: {e}. Also missing fuse-overlayfs binary: {which_err}"
                )
            })?;
            let fuse_opts = format!(
                "lowerdir={},upperdir={},workdir={}",
                bind_lower.display(),
                overlay.upper_dir.display(),
                overlay.work_dir.display()
            );
            let mut child = Command::new(&fuse_bin)
                .arg("-o")
                .arg(&fuse_opts)
                .arg(&overlay.merged_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Failed to spawn fuse-overlayfs")?;
            let mut ready = false;
            for _ in 0..30 {
                if let Ok(Some(fs_type)) = is_path_mounted(&overlay.merged_dir) {
                    if fs_type.contains("fuse") || fs_type.contains("fuse-overlayfs") {
                        ready = true;
                        break;
                    }
                }
                sleep(Duration::from_millis(33));
            }
            if !ready {
                let _ = child.kill();
                Err(anyhow::anyhow!(
                    "fuse-overlayfs did not mount {} within timeout",
                    overlay.merged_dir.display()
                ))
            } else {
                overlay.using_fuse = true;
                overlay.fuse_child = Some(child);
                Ok(())
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn mount_linux_read_only(overlay: &mut OverlayFs, lower_dir: &Path) -> Result<()> {
    use nix::mount::{mount, umount2, MntFlags, MsFlags};

    let bind_lower = overlay.overlay_dir.join("lower");
    std::fs::create_dir_all(&bind_lower)?;
    let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
    mount(
        Some(lower_dir),
        &bind_lower,
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>,
    )
    .with_context(|| {
        format!(
            "Failed to bind-mount lower {} -> {}",
            lower_dir.display(),
            bind_lower.display()
        )
    })?;

    let options = format!("lowerdir={}", bind_lower.display());
    mount(
        Some("overlay"),
        &overlay.merged_dir,
        Some("overlay"),
        MsFlags::MS_RDONLY,
        Some(options.as_bytes()),
    )
    .with_context(|| {
        format!(
            "Failed to mount read-only overlayfs on {}",
            overlay.merged_dir.display()
        )
    })?;
    overlay.bind_lower_dir = Some(bind_lower);
    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) fn mount_fuse_only(overlay: &mut OverlayFs, lower_dir: &Path) -> Result<()> {
    use nix::mount::{mount, umount2, MntFlags, MsFlags};
    use std::thread::sleep;
    use std::time::Duration;

    let bind_lower = overlay.overlay_dir.join("lower");
    std::fs::create_dir_all(&bind_lower)?;
    let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
    mount(
        Some(lower_dir),
        &bind_lower,
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>,
    )
    .with_context(|| {
        format!(
            "Failed to bind-mount lower {} -> {}",
            lower_dir.display(),
            bind_lower.display()
        )
    })?;

    let fuse_bin =
        which::which("fuse-overlayfs").context("fuse-overlayfs binary not found in PATH")?;
    let fuse_opts = format!(
        "lowerdir={},upperdir={},workdir={}",
        bind_lower.display(),
        overlay.upper_dir.display(),
        overlay.work_dir.display()
    );
    let mut child = Command::new(&fuse_bin)
        .arg("-o")
        .arg(&fuse_opts)
        .arg(&overlay.merged_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn fuse-overlayfs")?;

    let mut ready = false;
    for _ in 0..30 {
        if let Ok(Some(fs_type)) = is_path_mounted(&overlay.merged_dir) {
            if fs_type.contains("fuse") || fs_type.contains("fuse-overlayfs") {
                ready = true;
                break;
            }
        }
        sleep(Duration::from_millis(33));
    }
    if !ready {
        let _ = child.kill();
        anyhow::bail!(
            "fuse-overlayfs did not mount {} within timeout",
            overlay.merged_dir.display()
        );
    }

    overlay.using_fuse = true;
    overlay.fuse_child = Some(child);
    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) fn unmount_linux(overlay: &mut OverlayFs) -> Result<()> {
    use nix::mount::{umount2, MntFlags};

    if overlay.using_fuse {
        let _status = Command::new("fusermount3")
            .arg("-u")
            .arg(&overlay.merged_dir)
            .status();
        let _ = umount2(&overlay.merged_dir, MntFlags::MNT_DETACH);
        if let Some(mut ch) = overlay.fuse_child.take() {
            let _ = ch.kill();
        }
    } else {
        umount2(&overlay.merged_dir, MntFlags::MNT_DETACH)
            .context("Failed to unmount overlayfs")?;
    }

    if let Some(ref bind_lower) = overlay.bind_lower_dir {
        let _ = umount2(bind_lower, MntFlags::MNT_DETACH);
    }

    Ok(())
}

pub(crate) fn choose_base_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let uid = nix::unistd::Uid::current();
        if uid.is_root() {
            return Ok(PathBuf::from("/var/lib/substrate/overlay"));
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return Ok(PathBuf::from(xdg).join("substrate/overlay"));
            }
        }
        let run_user = PathBuf::from(format!("/run/user/{}/substrate/overlay", uid.as_raw()));
        if run_user.parent().unwrap_or(Path::new("/run")).exists() {
            return Ok(run_user);
        }
        Ok(PathBuf::from(format!(
            "/tmp/substrate-{}-overlay",
            uid.as_raw()
        )))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(PathBuf::from("/tmp/substrate-overlay"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::sync::Mutex;
    #[cfg(target_os = "linux")]
    use tempfile::tempdir;

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
    #[test]
    fn choose_base_dir_prefers_xdg_runtime_dir() {
        let temp = tempdir().unwrap();
        let xdg = temp.path().join("xdg-run");
        std::fs::create_dir_all(&xdg).unwrap();

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[("XDG_RUNTIME_DIR", xdg.to_str()), ("HOME", None)]);

        let base = choose_base_dir().expect("base dir");
        assert!(
            base.starts_with(&xdg),
            "base dir should live under XDG runtime dir"
        );
        assert!(
            base.ends_with("substrate/overlay"),
            "base dir should include substrate overlay suffix"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn choose_base_dir_defaults_to_tmp_on_non_linux() {
        let base = choose_base_dir().expect("base dir");
        assert_eq!(base, PathBuf::from("/tmp/substrate-overlay"));
    }
}
