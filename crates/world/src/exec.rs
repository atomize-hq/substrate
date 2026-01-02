//! Helpers for executing commands with incremental streaming callbacks.

use crate::stream::{emit_chunk, StreamKind};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use tracing::warn;
use world_api::WorldFsMode;

/// Execute `cmd` via `sh -lc` in the provided directory/environment, pushing
/// stdout/stderr chunks through the global stream sink while accumulating the
/// full output for the caller.
pub fn execute_shell_command(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    login_shell: bool,
) -> Result<Output> {
    let mut command = Command::new("sh");
    if login_shell {
        command.arg("-lc");
    } else {
        command.arg("-c");
    }
    command.arg(cmd);
    command.current_dir(cwd);
    command.envs(env);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to spawn command: {cmd}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

    let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
    let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

    let status = child
        .wait()
        .context("Failed to wait for child process completion")?;

    let stdout_buf = join_reader(stdout_handle, "stdout");
    let stderr_buf = join_reader(stderr_handle, "stderr");

    Ok(Output {
        status,
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}

pub struct ProjectBindMount<'a> {
    pub merged_dir: &'a Path,
    pub project_dir: &'a Path,
    pub desired_cwd: &'a Path,
    pub fs_mode: WorldFsMode,
}

pub const PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT: &str = r#"set -eu
set -f

mount --make-rprivate / 2>/dev/null || mount --make-private / 2>/dev/null || true

if [ "${SUBSTRATE_WORLD_FS_ISOLATION:-workspace}" = "full" ]; then
  new_root="$(mktemp -d /tmp/substrate-full-isolation.XXXXXX)"

  # Ensure new_root is a mountpoint (required by pivot_root).
  mount --bind "$new_root" "$new_root"
  mkdir -p "$new_root/old_root"

  bind_ro() {
    src="$1"
    dst="$2"
    if [ -e "$src" ]; then
      mkdir -p "$dst"
      mount --rbind "$src" "$dst"
      mount -o remount,bind,ro "$dst"
    fi
  }

  # Minimal system mounts.
  bind_ro /usr "$new_root/usr"
  bind_ro /bin "$new_root/bin"
  bind_ro /lib "$new_root/lib"
  bind_ro /lib64 "$new_root/lib64"
  bind_ro /etc "$new_root/etc"

  # /dev: bind-mounted read-only.
  mkdir -p "$new_root/dev"
  mount --rbind /dev "$new_root/dev"
  mount -o remount,bind,ro "$new_root/dev"

  # /var/lib/substrate/world-deps: bind-mounted read-write.
  mkdir -p /var/lib/substrate/world-deps
  mkdir -p "$new_root/var/lib/substrate/world-deps"
  mount --rbind /var/lib/substrate/world-deps "$new_root/var/lib/substrate/world-deps"

  # Fresh /proc and writable /tmp.
  #
  # Note: /tmp is a tmpfs in full-isolation mode. This must be mounted before binding the project
  # into its host-absolute path. Otherwise, when the host project lives under /tmp, the tmpfs
  # mount would cover that project bind mount and `cd $SUBSTRATE_MOUNT_CWD` would fail.
  mkdir -p "$new_root/proc"
  mount -t proc proc "$new_root/proc"

  mkdir -p "$new_root/tmp"
  mount -t tmpfs tmpfs "$new_root/tmp"
  chmod 1777 "$new_root/tmp" || true

  # Project mount points: stable (/project) and host-absolute ($SUBSTRATE_MOUNT_PROJECT_DIR).
  mkdir -p "$new_root/project"
  mount --bind "$SUBSTRATE_MOUNT_MERGED_DIR" "$new_root/project"

  mkdir -p "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"
  mount --bind "$SUBSTRATE_MOUNT_MERGED_DIR" "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"

  # Ensure allowlisted writable prefixes exist before we remount the project read-only.
  if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" != "read_only" ] && [ -n "${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}" ]; then
    oldIFS=$IFS
    IFS='
'
    for rel in $SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST; do
      [ -z "$rel" ] && continue
      case "$rel" in
        /*) continue ;;
      esac
      case "/$rel/" in
        */../*) continue ;;
      esac
      [ "$rel" = "." ] && continue
      mkdir -p "$new_root/project/$rel"
      mkdir -p "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
    done
    IFS=$oldIFS
  fi

  # Project is read-only by default; remount allowlisted prefixes writable.
  mount -o remount,bind,ro "$new_root/project"
  mount -o remount,bind,ro "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"

  if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" != "read_only" ] && [ -n "${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}" ]; then
    oldIFS=$IFS
    IFS='
'
    for rel in $SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST; do
      [ -z "$rel" ] && continue
      case "$rel" in
        /*) continue ;;
      esac
      case "/$rel/" in
        */../*) continue ;;
      esac
      if [ "$rel" = "." ]; then
        mount -o remount,bind,rw "$new_root/project"
        mount -o remount,bind,rw "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR"
        continue
      fi
      mount --bind "$new_root/project/$rel" "$new_root/project/$rel"
      mount -o remount,bind,rw "$new_root/project/$rel"
      mount --bind "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel" "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
      mount -o remount,bind,rw "$new_root$SUBSTRATE_MOUNT_PROJECT_DIR/$rel"
    done
    IFS=$oldIFS
  fi

  # Optional: bind-mount the host world-agent binary into the isolated rootfs so it can apply Landlock
  # restrictions before executing the command.
  if [ -n "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ] && [ -e "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ]; then
    touch "$new_root/substrate-landlock-helper" 2>/dev/null || true
    mount --bind "$SUBSTRATE_LANDLOCK_HELPER_SRC" "$new_root/substrate-landlock-helper" 2>/dev/null || true
    mount -o remount,bind,ro "$new_root/substrate-landlock-helper" 2>/dev/null || true
    export SUBSTRATE_LANDLOCK_HELPER_PATH="/substrate-landlock-helper"
  fi

  pivot_root "$new_root" "$new_root/old_root"
  cd /
  umount -l /old_root 2>/dev/null || true
  rmdir /old_root 2>/dev/null || true

  mkdir -p "${HOME:-/tmp/substrate-home}" 2>/dev/null || true

else
  mount --bind "$SUBSTRATE_MOUNT_MERGED_DIR" "$SUBSTRATE_MOUNT_PROJECT_DIR"
  if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" = "read_only" ]; then
    mount -o remount,bind,ro "$SUBSTRATE_MOUNT_PROJECT_DIR"
  fi
  if [ -n "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ] && [ -x "${SUBSTRATE_LANDLOCK_HELPER_SRC:-}" ]; then
    export SUBSTRATE_LANDLOCK_HELPER_PATH="${SUBSTRATE_LANDLOCK_HELPER_SRC}"
  fi
  mkdir -p "${HOME:-/tmp/substrate-home}" 2>/dev/null || true
  mkdir -p "${XDG_CACHE_HOME:-/tmp/substrate-xdg/cache}" 2>/dev/null || true
  mkdir -p "${XDG_CONFIG_HOME:-/tmp/substrate-xdg/config}" 2>/dev/null || true
  mkdir -p "${XDG_DATA_HOME:-/tmp/substrate-xdg/data}" 2>/dev/null || true
fi

cd "$SUBSTRATE_MOUNT_CWD"
if [ -n "${SUBSTRATE_LANDLOCK_HELPER_PATH:-}" ] && [ -x "${SUBSTRATE_LANDLOCK_HELPER_PATH}" ]; then
  exec "$SUBSTRATE_LANDLOCK_HELPER_PATH" "__substrate_world_landlock_exec"
fi
if [ "${SUBSTRATE_INNER_LOGIN_SHELL:-0}" = "1" ]; then
  exec sh -lc "$SUBSTRATE_INNER_CMD"
else
  exec sh -c "$SUBSTRATE_INNER_CMD"
fi
"#;

pub fn execute_shell_command_with_project_bind_mount(
    cmd: &str,
    mount: ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
) -> Result<Output> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cmd;
        let _ = mount;
        let _ = env;
        let _ = login_shell;
        Err(anyhow!(
            "project bind mount enforcement is only supported on Linux"
        ))
    }

    #[cfg(target_os = "linux")]
    {
        use nix::unistd::Uid;
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;

        // Outer script: establish a private mount namespace, then either:
        // - isolation=workspace: bind the overlay merged root onto the host project path to prevent
        //   absolute-path escapes back into the host project, or
        // - isolation=full: build a minimal rootfs, bind-mount only the allowed paths, then pivot_root
        //   so host paths are no longer nameable.
        //
        // We avoid setting the child's cwd via Command::current_dir() because holding an inode
        // reference into the host project dir would bypass the bind mount (absolute-path escape).
        let script = PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT;

        let mut env_map = env.clone();
        env_map.insert(
            "SUBSTRATE_MOUNT_MERGED_DIR".to_string(),
            mount.merged_dir.display().to_string(),
        );
        env_map.insert(
            "SUBSTRATE_MOUNT_PROJECT_DIR".to_string(),
            mount.project_dir.display().to_string(),
        );
        env_map.insert(
            "SUBSTRATE_MOUNT_CWD".to_string(),
            mount.desired_cwd.display().to_string(),
        );
        env_map.insert(
            "SUBSTRATE_MOUNT_FS_MODE".to_string(),
            mount.fs_mode.as_str().to_string(),
        );
        env_map.insert("SUBSTRATE_INNER_CMD".to_string(), cmd.to_string());
        env_map.insert(
            "SUBSTRATE_INNER_LOGIN_SHELL".to_string(),
            if login_shell {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );

        let isolation = env_map
            .get("SUBSTRATE_WORLD_FS_ISOLATION")
            .map(|raw| raw.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "workspace".to_string());
        let isolation_full = isolation == "full";
        let isolation_enabled = matches!(isolation.as_str(), "full" | "workspace" | "project");
        if isolation_enabled {
            env_map
                .entry("HOME".to_string())
                .or_insert_with(|| "/tmp/substrate-home".to_string());
            env_map
                .entry("XDG_CACHE_HOME".to_string())
                .or_insert_with(|| "/tmp/substrate-xdg/cache".to_string());
            env_map
                .entry("XDG_CONFIG_HOME".to_string())
                .or_insert_with(|| "/tmp/substrate-xdg/config".to_string());
            env_map
                .entry("XDG_DATA_HOME".to_string())
                .or_insert_with(|| "/tmp/substrate-xdg/data".to_string());
        }

        let mut command = Command::new("unshare");
        command.arg("--mount");
        command.arg("--fork");
        if !Uid::effective().is_root() {
            // Best-effort: try to acquire mount privileges via user namespaces when unprivileged.
            // If user namespaces are disabled on the host, the command will fail and the caller
            // should fall back to the non-caged path.
            command.arg("--user");
            command.arg("--map-root-user");
        }
        command.arg("--");
        command.arg("sh");
        command.arg("-c");
        command.arg(script);
        command.current_dir("/");
        command.envs(env_map);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                if isolation_full {
                    let message = format!(
                        "substrate: error: world_fs.isolation=full requested but failed to spawn unshare wrapper: {err}\n"
                    );
                    return Ok(Output {
                        status: std::process::ExitStatus::from_raw(126 << 8),
                        stdout: Vec::new(),
                        stderr: message.into_bytes(),
                    });
                }
                return Err(err).with_context(|| format!("Failed to spawn command: {cmd}"));
            }
        };

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout pipe"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stderr pipe"))?;

        let stdout_handle = spawn_reader(stdout, StreamKind::Stdout);
        let stderr_handle = spawn_reader(stderr, StreamKind::Stderr);

        let status = child
            .wait()
            .context("Failed to wait for child process completion")?;

        let stdout_buf = join_reader(stdout_handle, "stdout");
        let mut stderr_buf = join_reader(stderr_handle, "stderr");

        if isolation_full && !status.success() {
            if let Ok(stderr_str) = std::str::from_utf8(&stderr_buf) {
                if stderr_str.starts_with("unshare:") {
                    let mut wrapped = Vec::new();
                    wrapped.extend_from_slice(b"substrate: error: world_fs.isolation=full requested but failed to enter a mount namespace (unshare).\n");
                    wrapped.extend_from_slice(b"substrate: hint: run with CAP_SYS_ADMIN (root) or enable unprivileged user namespaces (kernel.unprivileged_userns_clone=1).\n");
                    wrapped.extend_from_slice(stderr_buf.as_slice());
                    stderr_buf = wrapped;
                }
            }
        }

        Ok(Output {
            status,
            stdout: stdout_buf,
            stderr: stderr_buf,
        })
    }
}

fn spawn_reader<R>(mut reader: R, kind: StreamKind) -> thread::JoinHandle<Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let slice = &chunk[..n];
                    buf.extend_from_slice(slice);
                    emit_chunk(kind, slice);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(anyhow!("pipe read failed: {e}"));
                }
            }
        }
        Ok(buf)
    })
}

fn join_reader(handle: thread::JoinHandle<Result<Vec<u8>>>, label: &str) -> Vec<u8> {
    match handle.join() {
        Ok(Ok(buf)) => buf,
        Ok(Err(e)) => {
            warn!(stream = label, error = %e, "stream reader error");
            Vec::new()
        }
        Err(e) => {
            warn!(stream = label, error = ?e, "stream reader panicked");
            Vec::new()
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn read_only_bind_mount_blocks_absolute_project_writes() {
        let merged = tempdir().expect("merged tempdir");
        let project = tempdir().expect("project tempdir");

        let mount = ProjectBindMount {
            merged_dir: merged.path(),
            project_dir: project.path(),
            desired_cwd: project.path(),
            fs_mode: WorldFsMode::ReadOnly,
        };

        let env: HashMap<String, String> = HashMap::new();
        let cmd = r#"touch "$SUBSTRATE_MOUNT_PROJECT_DIR/abs_escape.txt""#;

        let output = match execute_shell_command_with_project_bind_mount(cmd, mount, &env, true) {
            Ok(output) => output,
            Err(err) => {
                let message = err.to_string();
                if message.contains("Operation not permitted")
                    || message.contains("EPERM")
                    || message.contains("unshare")
                {
                    println!("Skipping bind-mount caging test: {}", message);
                    return;
                }
                panic!("unexpected error running bind-mount wrapper: {:#}", err);
            }
        };

        assert!(
            !output.status.success(),
            "expected read-only bind mount to reject writes via absolute project path"
        );
        assert!(
            !project.path().join("abs_escape.txt").exists(),
            "file should not appear in host project dir"
        );
    }
}
