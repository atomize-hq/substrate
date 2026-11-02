use std::{
    env,
    fs::{self, File, OpenOptions},
    io::ErrorKind,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

pub(crate) struct TestEnvLockGuard {
    path: PathBuf,
    _file: File,
}

pub(crate) fn test_env_lock() -> TestEnvLockGuard {
    let path = env::temp_dir().join(format!("agent-api-test-env-{}.lock", std::process::id()));

    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return TestEnvLockGuard { path, _file: file };
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(err) => panic!("test env lock should be acquired: {err}"),
        }
    }
}

impl Drop for TestEnvLockGuard {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_file(&self.path) {
            assert!(
                err.kind() == ErrorKind::NotFound,
                "test env lock should be released: {err}"
            );
        }
    }
}

pub(crate) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(crate) fn set(path: &Path) -> Self {
        let previous = env::current_dir().unwrap_or_else(|_| env::temp_dir());
        env::set_current_dir(path).expect("current dir should be set");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        if env::set_current_dir(&self.previous).is_err() {
            env::set_current_dir(env::temp_dir()).expect("fallback current dir should be restored");
        }
    }
}
