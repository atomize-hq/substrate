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

#[derive(Debug)]
pub struct ProcessLock {
    _file: File,
}

impl ProcessLock {
    /// Acquires an exclusive file lock with timeout.
    ///
    /// Creates a lock file at the specified path and acquires an exclusive lock.
    /// If the lock cannot be acquired within the timeout period, returns an error.
    /// The lock is automatically released when the `ProcessLock` is dropped.
    ///
    /// # Arguments
    ///
    /// * `lock_path` - Path where the lock file should be created
    /// * `timeout` - Maximum time to wait for lock acquisition
    ///
    /// # Returns
    ///
    /// Returns `Ok(ProcessLock)` if the lock was successfully acquired,
    /// or an error if the lock could not be acquired or created.
    ///
    /// # Examples
    ///
    /// ```
    /// use substrate_shell::lock::ProcessLock;
    /// use std::time::Duration;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new().unwrap();
    /// let lock_path = temp_dir.path().join("test.lock");
    ///
    /// // Acquire a lock with 1 second timeout
    /// let lock = ProcessLock::acquire(&lock_path, Duration::from_secs(1)).unwrap();
    /// // Lock is automatically released when `lock` goes out of scope
    /// ```
    ///
    /// # Concurrent Access
    ///
    /// ```
    /// use substrate_shell::lock::ProcessLock;
    /// use std::time::Duration;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new().unwrap();
    /// let lock_path = temp_dir.path().join("concurrent.lock");
    ///
    /// // First process acquires the lock
    /// let _lock1 = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();
    ///
    /// // Second process cannot acquire the lock and times out
    /// let result = ProcessLock::acquire(&lock_path, Duration::from_millis(50));
    /// assert!(result.is_err());
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_lock_acquisition_and_release() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // First lock should succeed
        let lock1 = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();
        assert!(lock_path.exists());

        // Second lock should fail immediately with timeout
        let start = std::time::Instant::now();
        let lock2_result = ProcessLock::acquire(&lock_path, Duration::from_millis(50));
        let elapsed = start.elapsed();

        assert!(lock2_result.is_err());
        // Very lenient timing for CI environments
        assert!(elapsed >= Duration::from_millis(30)); // Allow significant variance
        assert!(elapsed < Duration::from_millis(500)); // Much more lenient for slow CI

        // Drop first lock
        drop(lock1);

        // Should be able to acquire after release
        let lock3 = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();
        drop(lock3);
    }

    #[test]
    fn test_lock_timeout_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("timeout.lock");

        let _lock1 = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();

        let start = std::time::Instant::now();
        let result = ProcessLock::acquire(&lock_path, Duration::from_millis(100));
        let elapsed = start.elapsed();

        assert!(result.is_err());
        // Very lenient timing for CI environments
        assert!(elapsed >= Duration::from_millis(50)); // Allow significant variance
        assert!(elapsed < Duration::from_millis(500)); // Much more lenient for slow CI

        let error_msg = result.unwrap_err().to_string();
        // Windows may return different error messages than Unix
        assert!(
            error_msg.contains("Timeout waiting for lock")
                || error_msg.contains("Failed to acquire lock")
        );
    }

    #[test]
    fn test_lock_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dirs")
            .join("test.lock");

        // Parent directories should be created automatically
        let lock = ProcessLock::acquire(&nested_path, Duration::from_millis(100)).unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().is_dir());

        drop(lock);
    }

    #[test]
    fn test_lock_info_written_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("info.lock");

        let _lock = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();

        // Read the lock file contents
        let contents = std::fs::read_to_string(&lock_path).unwrap();
        let lock_info: LockInfo = serde_json::from_str(&contents).unwrap();

        assert_eq!(lock_info.pid, std::process::id());
        assert_eq!(lock_info.version, env!("CARGO_PKG_VERSION"));
        assert!(lock_info.timestamp > 0);

        // Timestamp should be recent (within last 10 seconds)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(now - lock_info.timestamp < 10);
    }

    #[test]
    fn test_concurrent_lock_attempts() {
        use std::sync::{Arc, Barrier};

        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("concurrent.lock");

        let lock_path_clone = lock_path.clone();

        // Use a barrier to ensure proper synchronization
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);

        // Spawn thread that holds lock for a short time
        let handle = thread::spawn(move || {
            let _lock = ProcessLock::acquire(&lock_path_clone, Duration::from_millis(100)).unwrap();
            // Signal that we have the lock
            barrier_clone.wait();
            // Hold lock for a while
            thread::sleep(Duration::from_millis(300)); // Hold lock long enough
        });

        // Wait for the first thread to acquire the lock
        barrier.wait();

        // Small additional delay to ensure lock is really held
        thread::sleep(Duration::from_millis(50));

        // This should timeout while first thread holds the lock
        let start = std::time::Instant::now();
        let result = ProcessLock::acquire(&lock_path, Duration::from_millis(100));
        let elapsed = start.elapsed();

        assert!(
            result.is_err(),
            "Lock should have been held by other thread"
        );
        // Very lenient timing assertions for CI
        assert!(
            elapsed >= Duration::from_millis(50),
            "Should have waited at least 50ms"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "Should timeout within 500ms"
        );

        // Wait for first thread to finish and release lock
        handle.join().unwrap();

        // Now we should be able to acquire it
        let _lock = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();
    }

    #[test]
    fn test_zero_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("zero_timeout.lock");

        let _lock1 = ProcessLock::acquire(&lock_path, Duration::from_millis(100)).unwrap();

        // Zero timeout should fail immediately
        let start = std::time::Instant::now();
        let result = ProcessLock::acquire(&lock_path, Duration::from_millis(0));
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(elapsed < Duration::from_millis(50)); // Should fail very quickly
    }

    #[test]
    fn test_invalid_path_handling() {
        // Try to create lock on a file that can't be created (depends on platform)
        #[cfg(unix)]
        {
            use std::path::PathBuf;
            let invalid_path = PathBuf::from("/root/definitely_cannot_create.lock");
            let result = ProcessLock::acquire(&invalid_path, Duration::from_millis(100));

            // This should fail, but we don't know exactly how (permission denied, etc.)
            // Just ensure it doesn't panic
            let _ = result;
        }
    }

    #[test]
    fn test_lock_info_serialization() {
        let info = LockInfo {
            pid: 12345,
            timestamp: 1234567890,
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: LockInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.pid, deserialized.pid);
        assert_eq!(info.timestamp, deserialized.timestamp);
        assert_eq!(info.version, deserialized.version);
    }
}
