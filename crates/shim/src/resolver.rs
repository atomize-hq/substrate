//! Binary resolution with caching and clean PATH handling
//!
//! This module implements the core path resolution logic that finds the real
//! binary to execute, with caching to reduce filesystem calls.

use crate::context::CACHE_BUST_VAR;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Resolution cache to avoid repeated stat() calls
static RESOLUTION_CACHE: Lazy<RwLock<HashMap<String, Option<PathBuf>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Resolve the real binary for a command, with caching
pub fn resolve_real_binary(command: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
    // Check cache first (unless cache busting is enabled)
    if env::var(CACHE_BUST_VAR).is_err() {
        let cache_key = build_cache_key(command, search_paths);

        if let Ok(cache) = RESOLUTION_CACHE.read() {
            if let Some(cached_result) = cache.get(&cache_key) {
                return cached_result.clone();
            }
        }
    }

    // Perform resolution
    let result = resolve_binary_uncached(command, search_paths);

    // Cache the result (unless cache busting is enabled)
    if env::var(CACHE_BUST_VAR).is_err() {
        let cache_key = build_cache_key(command, search_paths);

        if let Ok(mut cache) = RESOLUTION_CACHE.write() {
            cache.insert(cache_key, result.clone());
        }
    }

    result
}

/// Build cache key for command and search paths
fn build_cache_key(command: &str, search_paths: &[PathBuf]) -> String {
    // Cache key based only on command name and search paths
    // PATH resolution doesn't depend on CWD, so including it would reduce cache hit rate

    // Normalize paths by trimming trailing slashes
    let normalized_paths: Vec<String> = search_paths
        .iter()
        .map(|p| p.display().to_string().trim_end_matches('/').to_string())
        .collect();

    format!("{}:{}", command, normalized_paths.join(":"))
}

/// Resolve binary without caching
fn resolve_binary_uncached(command: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
    for dir in search_paths {
        let candidate = dir.join(command);

        // On Windows, try with common executable extensions
        #[cfg(windows)]
        {
            let extensions =
                env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());

            for ext in extensions.split(';') {
                if !ext.is_empty() {
                    let mut path_with_ext = candidate.clone();
                    path_with_ext.set_extension(&ext[1..]); // Remove leading dot
                    if is_executable(&path_with_ext) {
                        return Some(path_with_ext);
                    }
                }
            }
        }

        // Unix or Windows without extension
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

/// Check if a path is executable
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0)
        } else {
            false
        }
    }

    #[cfg(windows)]
    {
        std::fs::metadata(path)
            .map(|m| m.is_file())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_real_binary_finds_existing() {
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir(&bin_dir).unwrap();

        let test_binary = bin_dir.join("test_cmd");
        fs::write(&test_binary, "#!/bin/bash\necho test").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&test_binary).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&test_binary, perms).unwrap();
        }

        let search_paths = vec![bin_dir];
        let result = resolve_real_binary("test_cmd", &search_paths);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), test_binary);
    }

    #[test]
    fn test_resolve_real_binary_returns_none_for_missing() {
        let temp = TempDir::new().unwrap();
        let search_paths = vec![temp.path().to_path_buf()];
        let result = resolve_real_binary("nonexistent_cmd", &search_paths);

        assert!(result.is_none());
    }

    #[test]
    fn test_cache_key_normalization() {
        let paths = vec![PathBuf::from("/usr/bin/"), PathBuf::from("/bin")];

        let key1 = build_cache_key("git", &paths);
        let normalized_paths = vec![PathBuf::from("/usr/bin"), PathBuf::from("/bin")];
        let key2 = build_cache_key("git", &normalized_paths);

        // Keys should be the same due to normalization
        assert_eq!(key1, key2);

        // Verify cache key doesn't include CWD
        assert!(!key1.contains("tmp"));
        assert!(!key1.contains("home"));
    }

    #[test]
    fn test_executable_bit_check() {
        let temp = TempDir::new().unwrap();
        let non_executable = temp.path().join("not_exec");
        fs::write(&non_executable, "content").unwrap();

        // Should not be considered executable
        assert!(!is_executable(&non_executable));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = temp.path().join("exec");
            fs::write(&executable, "#!/bin/bash\necho test").unwrap();
            let mut perms = fs::metadata(&executable).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&executable, perms).unwrap();

            assert!(is_executable(&executable));
        }
    }
}
