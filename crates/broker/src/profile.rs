#[cfg(any(test, feature = "policy-watcher"))]
use anyhow::Context;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

const PROFILE_FILENAME: &str = ".substrate-profile";
const PROFILE_DIR_FILENAME: &str = ".substrate-profile.d";
const MAX_SEARCH_DEPTH: usize = 10; // Prevent infinite traversal

pub struct ProfileDetector {
    cache: HashMap<PathBuf, Option<PathBuf>>,
}

impl ProfileDetector {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn find_profile(&mut self, start_dir: &Path) -> Result<Option<PathBuf>> {
        let canonical_start = start_dir
            .canonicalize()
            .unwrap_or_else(|_| start_dir.to_path_buf());

        // Check cache first
        if let Some(cached) = self.cache.get(&canonical_start) {
            debug!("Using cached profile result for {:?}", canonical_start);
            return Ok(cached.clone());
        }

        // Search up the directory tree
        let mut current = canonical_start.clone();
        let mut depth = 0;

        loop {
            // Check for .substrate-profile file
            let profile_file = current.join(PROFILE_FILENAME);
            if profile_file.exists() && profile_file.is_file() {
                info!("Found profile at {:?}", profile_file);
                self.cache
                    .insert(canonical_start.clone(), Some(profile_file.clone()));
                return Ok(Some(profile_file));
            }

            // Check for .substrate-profile.d directory
            let profile_dir = current.join(PROFILE_DIR_FILENAME);
            if profile_dir.exists() && profile_dir.is_dir() {
                // Look for default.yaml or similar
                for entry in ["default.yaml", "default.yml", "policy.yaml", "policy.yml"] {
                    let policy_file = profile_dir.join(entry);
                    if policy_file.exists() && policy_file.is_file() {
                        info!("Found profile at {:?}", policy_file);
                        self.cache
                            .insert(canonical_start.clone(), Some(policy_file.clone()));
                        return Ok(Some(policy_file));
                    }
                }
            }

            // Move up one directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
                depth += 1;

                if depth > MAX_SEARCH_DEPTH {
                    debug!("Reached max search depth, no profile found");
                    break;
                }

                // Stop at home directory or root
                if current == dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                    || current == Path::new("/")
                {
                    debug!("Reached home or root directory, no profile found");
                    break;
                }
            } else {
                break;
            }
        }

        // Cache the negative result
        self.cache.insert(canonical_start, None);
        Ok(None)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for ProfileDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Load all profiles from a directory and merge them
#[cfg(any(test, feature = "policy-watcher"))]
#[allow(dead_code)]
pub fn load_profile_directory(dir: &Path) -> Result<Vec<crate::Policy>> {
    let mut policies = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return Ok(policies);
    }

    // Read all .yaml and .yml files in the directory
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            debug!("Loading profile from {:?}", path);
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read profile from {:?}", path))?;

            let policy: crate::Policy = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse profile from {:?}", path))?;

            policies.push(policy);
        }
    }

    // Sort by filename for consistent ordering
    policies.sort_by_key(|p| p.id.clone());

    Ok(policies)
}

/// Create a sample profile file
#[cfg(any(test, feature = "policy-watcher"))]
#[allow(dead_code)]
pub fn create_sample_profile(path: &Path) -> Result<()> {
    let sample = r#"# Substrate Security Profile
# Place this file in your project root as .substrate-profile

id: project-policy
name: My Project Security Policy

# Filesystem permissions
fs_read:
  - "*"           # Allow reading all files (default)

fs_write:
  - "./dist/*"    # Only allow writing to dist directory
  - "./build/*"   # And build directory
  - "/tmp/*"      # And temp files

# Network permissions
net_allowed:
  - "github.com"
  - "*.githubusercontent.com"
  - "registry.npmjs.org"
  - "pypi.org"
  - "crates.io"

# Command allowlist (empty = all allowed)
cmd_allowed: []

# Command denylist (always checked first)
cmd_denied:
  - "rm -rf /"
  - "rm -rf /*"
  - "curl * | bash"
  - "wget * | bash"
  - "sudo rm -rf"

# Commands that require isolated execution
cmd_isolated:
  - "npm install"
  - "npm ci"
  - "pip install"
  - "cargo install"
  - "make install"

# Behavior settings
require_approval: false          # Ask before running commands
allow_shell_operators: true      # Allow pipes, redirects, etc.

# Resource limits (optional)
limits:
  max_memory_mb: 4096
  max_cpu_percent: 80
  max_runtime_ms: 600000        # 10 minutes
  max_egress_bytes: 1073741824  # 1GB

# Metadata
metadata:
  created: "2024-01-01"
  author: "security-team"
  version: "1.0.0"
"#;

    std::fs::write(path, sample)
        .with_context(|| format!("Failed to write sample profile to {:?}", path))?;

    info!("Created sample profile at {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests;
