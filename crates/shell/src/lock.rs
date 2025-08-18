use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub timestamp: i64,
    pub version: String,
}

pub struct ProcessLock {
    _file: File,
}

impl ProcessLock {
    pub fn acquire(lock_path: &Path, timeout: Duration) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create lock directory: {parent:?}"))?;
        }

        // Open or create lock file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(lock_path)
            .with_context(|| format!("Failed to open lock file: {lock_path:?}"))?;

        // Try to acquire lock with timeout
        let start = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => {
                    // Write debug info to lock file
                    let lock_info = LockInfo {
                        pid: std::process::id(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    };
                    
                    if let Ok(json) = serde_json::to_string(&lock_info) {
                        let _ = file.set_len(0); // Truncate the file
                        let _ = file.write_all(json.as_bytes());
                        let _ = file.sync_all();
                    }
                    
                    return Ok(Self { _file: file });
                }
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