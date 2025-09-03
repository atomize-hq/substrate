use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
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
        let canonical_start = start_dir.canonicalize()
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
                self.cache.insert(canonical_start.clone(), Some(profile_file.clone()));
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
                        self.cache.insert(canonical_start.clone(), Some(policy_file.clone()));
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
                if current == dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")) ||
                   current == Path::new("/") {
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

/// Load all profiles from a directory and merge them
pub fn load_profile_directory(dir: &Path) -> Result<Vec<crate::Policy>> {
    let mut policies = Vec::new();
    
    if !dir.exists() || !dir.is_dir() {
        return Ok(policies);
    }
    
    // Read all .yaml and .yml files in the directory
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") ||
           path.extension().and_then(|s| s.to_str()) == Some("yml") {
            
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
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_detection() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path().join("project");
        let sub_dir = project_dir.join("src").join("lib");
        std::fs::create_dir_all(&sub_dir).unwrap();
        
        // Create a profile file in project root
        let profile_path = project_dir.join(PROFILE_FILENAME);
        std::fs::write(&profile_path, "test").unwrap();
        
        let mut detector = ProfileDetector::new();
        
        // Should find profile from subdirectory
        let result = detector.find_profile(&sub_dir).unwrap();
        assert!(result.is_some());
        // Compare canonical paths to handle symlinks
        let found = result.unwrap().canonicalize().unwrap();
        let expected = profile_path.canonicalize().unwrap();
        assert_eq!(found, expected);
        
        // Should use cache on second call
        let result2 = detector.find_profile(&sub_dir).unwrap();
        assert!(result2.is_some());
    }

    #[test]
    fn test_no_profile() {
        let temp = tempdir().unwrap();
        let mut detector = ProfileDetector::new();
        
        let result = detector.find_profile(temp.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_profile_directory() {
        let temp = tempdir().unwrap();
        let profile_dir = temp.path().join(PROFILE_DIR_FILENAME);
        std::fs::create_dir(&profile_dir).unwrap();
        
        let policy_file = profile_dir.join("default.yaml");
        std::fs::write(&policy_file, "id: test\nname: Test").unwrap();
        
        let mut detector = ProfileDetector::new();
        let result = detector.find_profile(temp.path()).unwrap();
        assert!(result.is_some());
        // Compare canonical paths to handle symlinks
        let found = result.unwrap().canonicalize().unwrap();
        let expected = policy_file.canonicalize().unwrap();
        assert_eq!(found, expected);
    }

    #[test]
    fn test_sample_profile_creation() {
        let temp = tempdir().unwrap();
        let profile_path = temp.path().join(".substrate-profile");
        
        create_sample_profile(&profile_path).unwrap();
        assert!(profile_path.exists());
        
        // Verify it can be parsed
        let content = std::fs::read_to_string(&profile_path).unwrap();
        let policy: crate::Policy = serde_yaml::from_str(&content).unwrap();
        assert_eq!(policy.id, "project-policy");
    }
}