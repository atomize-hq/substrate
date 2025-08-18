use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct ProcessLock {
    _file: File,
}

impl ProcessLock {
    pub fn acquire(lock_path: &Path, timeout: Duration) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create lock directory: {:?}", parent))?;
        }

        // Open or create lock file
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(lock_path)
            .with_context(|| format!("Failed to open lock file: {:?}", lock_path))?;

        // Try to acquire lock with timeout
        let start = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        return Err(anyhow::anyhow!(
                            "Timeout waiting for lock after {:?}",
                            timeout
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to acquire lock: {}", e));
                }
            }
        }
    }
}

impl Drop for ProcessLock {
    fn drop(&mut self) {
        // Lock is automatically released when file is closed
    }
}