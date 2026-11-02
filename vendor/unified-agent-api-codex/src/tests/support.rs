use std::fs as std_fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn env_mutex() -> &'static tokio::sync::Mutex<()> {
    static ENV_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    ENV_MUTEX.get_or_init(|| tokio::sync::Mutex::new(()))
}

pub(super) fn env_guard() -> tokio::sync::MutexGuard<'static, ()> {
    env_mutex().blocking_lock()
}

pub(super) async fn env_guard_async() -> tokio::sync::MutexGuard<'static, ()> {
    env_mutex().lock().await
}

fn write_executable(dir: &Path, name: &str, script: &str) -> PathBuf {
    let path = dir.join(name);
    std_fs::write(&path, script).unwrap();
    let mut perms = std_fs::metadata(&path).unwrap().permissions();
    #[cfg(unix)]
    {
        perms.set_mode(0o755);
    }
    std_fs::set_permissions(&path, perms).unwrap();
    path
}

pub(super) fn write_fake_codex(dir: &Path, script: &str) -> PathBuf {
    write_executable(dir, "codex", script)
}

pub(super) fn write_fake_bundled_codex(dir: &Path, platform: &str, script: &str) -> PathBuf {
    write_executable(dir, super::bundled_binary_filename(platform), script)
}
