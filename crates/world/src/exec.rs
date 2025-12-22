//! Helpers for executing commands with incremental streaming callbacks.

use crate::stream::{emit_chunk, StreamKind};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use tracing::warn;
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
pub struct ProjectBindMount<'a> {
    pub merged_dir: &'a Path,
    pub project_dir: &'a Path,
    pub desired_cwd: &'a Path,
    pub fs_mode: WorldFsMode,
}

#[cfg(target_os = "linux")]
pub fn execute_shell_command_with_project_bind_mount(
    cmd: &str,
    mount: ProjectBindMount<'_>,
    env: &HashMap<String, String>,
    login_shell: bool,
) -> Result<Output> {
    use nix::unistd::Uid;

    // Outer script: establish a private mount namespace, bind the overlay root onto the
    // project path, optionally remount read-only, then cd into the desired cwd and exec the
    // requested command via sh.
    //
    // We avoid setting the child's cwd via Command::current_dir() because holding an inode
    // reference into the host project dir would bypass the bind mount (absolute-path escape).
    let script = r#"set -eu
mount --make-rprivate /
mount --bind "$SUBSTRATE_MOUNT_MERGED_DIR" "$SUBSTRATE_MOUNT_PROJECT_DIR"
if [ "${SUBSTRATE_MOUNT_FS_MODE:-writable}" = "read_only" ]; then
  mount -o remount,bind,ro "$SUBSTRATE_MOUNT_PROJECT_DIR"
fi
cd "$SUBSTRATE_MOUNT_CWD"
if [ "${SUBSTRATE_INNER_LOGIN_SHELL:-0}" = "1" ]; then
  exec sh -lc "$SUBSTRATE_INNER_CMD"
else
  exec sh -c "$SUBSTRATE_INNER_CMD"
fi
"#;

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
