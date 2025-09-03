use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use crossbeam::channel::{unbounded, Receiver};
use tracing::{info, warn, error, debug};

pub struct PolicyWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    rx: Receiver<Result<Event, notify::Error>>,
    watched_paths: Vec<PathBuf>,
}

impl PolicyWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = unbounded();
        
        let watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = tx.send(res) {
                error!("Failed to send file watch event: {}", e);
            }
        })?;
        
        Ok(Self {
            watcher: Some(watcher),
            rx,
            watched_paths: Vec::new(),
        })
    }
    
    pub fn watch_path(&mut self, path: &Path) -> Result<()> {
        if let Some(watcher) = &mut self.watcher {
            watcher.watch(path, RecursiveMode::NonRecursive)
                .with_context(|| format!("Failed to watch path: {:?}", path))?;
            
            self.watched_paths.push(path.to_path_buf());
            info!("Watching for policy changes at: {:?}", path);
        }
        Ok(())
    }
    
    pub fn unwatch_path(&mut self, path: &Path) -> Result<()> {
        if let Some(watcher) = &mut self.watcher {
            watcher.unwatch(path)
                .with_context(|| format!("Failed to unwatch path: {:?}", path))?;
            
            self.watched_paths.retain(|p| p != path);
            info!("Stopped watching: {:?}", path);
        }
        Ok(())
    }
    
    pub fn check_for_changes(&self) -> Option<PathBuf> {
        // Non-blocking check for file changes
        if let Ok(event_result) = self.rx.try_recv() {
            match event_result {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        if let Some(path) = event.paths.first() {
                            debug!("Detected change in: {:?}", path);
                            return Some(path.clone());
                        }
                    }
                },
                Err(e) => {
                    warn!("File watch error: {}", e);
                }
            }
        }
        None
    }
    
    pub fn stop(&mut self) {
        self.watcher = None;
        self.watched_paths.clear();
    }
}

/// Spawn a background task to watch for policy changes and reload
pub async fn spawn_policy_watcher(
    policy_path: PathBuf,
    reload_callback: Arc<dyn Fn(&Path) -> Result<()> + Send + Sync>,
) -> Result<()> {
    tokio::spawn(async move {
        let mut watcher = match PolicyWatcher::new() {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create policy watcher: {}", e);
                return;
            }
        };
        
        if let Err(e) = watcher.watch_path(&policy_path) {
            error!("Failed to watch policy file: {}", e);
            return;
        }
        
        loop {
            if let Some(changed_path) = watcher.check_for_changes() {
                info!("Policy file changed, reloading: {:?}", changed_path);
                
                // Small delay to ensure file write is complete
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                if let Err(e) = reload_callback(&changed_path) {
                    error!("Failed to reload policy: {}", e);
                }
            }
            
            // Check every second
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    
    Ok(())
}

/// Watch multiple policy directories for changes
pub struct MultiPolicyWatcher {
    watchers: Vec<PolicyWatcher>,
}

impl MultiPolicyWatcher {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }
    
    pub fn add_directory(&mut self, dir: &Path) -> Result<()> {
        let mut watcher = PolicyWatcher::new()?;
        
        // Watch for .yaml and .yml files
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()).map(|s| s == "yaml" || s == "yml").unwrap_or(false) {
                watcher.watch_path(&path)?;
            }
        }
        
        // Also watch the directory itself for new files
        watcher.watch_path(dir)?;
        
        self.watchers.push(watcher);
        Ok(())
    }
    
    pub fn check_all(&self) -> Vec<PathBuf> {
        let mut changes = Vec::new();
        
        for watcher in &self.watchers {
            if let Some(path) = watcher.check_for_changes() {
                changes.push(path);
            }
        }
        
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_policy_watcher() {
        let temp = tempdir().unwrap();
        let policy_file = temp.path().join("policy.yaml");
        fs::write(&policy_file, "id: test\nname: Test").unwrap();
        
        let mut watcher = PolicyWatcher::new().unwrap();
        watcher.watch_path(&policy_file).unwrap();
        
        // Modify the file
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&policy_file, "id: test\nname: Modified").unwrap();
        
        // Give the watcher time to detect the change
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check for changes
        let changed = watcher.check_for_changes();
        assert!(changed.is_some());
    }

    #[test]
    fn test_multi_watcher() {
        let temp = tempdir().unwrap();
        let dir = temp.path().join("policies");
        fs::create_dir(&dir).unwrap();
        
        // Create some policy files
        fs::write(dir.join("policy1.yaml"), "id: p1").unwrap();
        fs::write(dir.join("policy2.yml"), "id: p2").unwrap();
        fs::write(dir.join("readme.txt"), "not a policy").unwrap();
        
        let mut multi_watcher = MultiPolicyWatcher::new();
        multi_watcher.add_directory(&dir).unwrap();
        
        // Should have watchers set up
        assert!(!multi_watcher.watchers.is_empty());
    }
}