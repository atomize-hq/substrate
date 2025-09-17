//! Network namespace garbage collection for orphaned substrate worlds.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcReport {
    pub removed: Vec<String>,
    pub kept: Vec<Kept>,
    pub errors: Vec<GcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kept {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcError {
    pub name: String,
    pub message: String,
}

const NETNS_PREFIX: &str = "substrate-";
const WORLD_ID_PREFIX: &str = "wld_";

fn extract_world_id(ns_name: &str) -> Option<String> {
    if ns_name.starts_with(NETNS_PREFIX) {
        let without_prefix = &ns_name[NETNS_PREFIX.len()..];
        if without_prefix.starts_with(WORLD_ID_PREFIX) {
            return Some(without_prefix.to_string());
        }
    }
    None
}

pub fn list_netns() -> Result<Vec<String>> {
    let output = Command::new("ip")
        .args(&["netns", "list"])
        .output()
        .context("Failed to list network namespaces")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list netns: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let namespaces: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let ns_name = parts[0];
                if ns_name.starts_with(NETNS_PREFIX) && extract_world_id(ns_name).is_some() {
                    return Some(ns_name.to_string());
                }
            }
            None
        })
        .collect();

    Ok(namespaces)
}

pub fn netns_pids(ns: &str) -> Result<Vec<i32>> {
    let output = Command::new("ip")
        .args(&["netns", "pids", ns])
        .output()
        .context(format!("Failed to get PIDs for netns {}", ns))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get netns pids for {}: {}", ns, stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<i32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<i32>().ok())
        .collect();

    Ok(pids)
}

pub fn cgroup_procs(world_id: &str) -> Result<Vec<i32>> {
    let cgroup_path = format!("/sys/fs/cgroup/substrate/{}/cgroup.procs", world_id);
    let path = Path::new(&cgroup_path);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)
        .context(format!("Failed to read cgroup.procs for {}", world_id))?;

    let pids: Vec<i32> = contents
        .lines()
        .filter_map(|line| line.trim().parse::<i32>().ok())
        .collect();

    Ok(pids)
}

pub fn delete_nft_table(ns: &str, world_id: &str) -> Result<()> {
    let table_name = format!("substrate_{}", world_id);

    let output = Command::new("timeout")
        .args(&[
            "2",
            "ip",
            "netns",
            "exec",
            ns,
            "nft",
            "delete",
            "table",
            "inet",
            &table_name,
        ])
        .output()
        .context(format!("Failed to delete nft table for {}", ns))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such file or directory") || stderr.contains("does not exist") {
            debug!(
                "nft table {} not found in netns {} (already gone)",
                table_name, ns
            );
        } else {
            debug!("Failed to delete nft table in {}: {}", ns, stderr);
        }
    } else {
        debug!("Deleted nft table {} in netns {}", table_name, ns);
    }

    Ok(())
}

pub fn delete_netns(ns: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(&["netns", "delete", ns])
        .output()
        .context(format!("Failed to delete netns {}", ns))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to delete netns {}: {}", ns, stderr);
    }

    info!("Deleted netns {}", ns);
    Ok(())
}

pub fn try_rmdir_cgroup(world_id: &str) -> Result<()> {
    let cgroup_path = format!("/sys/fs/cgroup/substrate/{}", world_id);
    let path = Path::new(&cgroup_path);

    if !path.exists() {
        debug!("Cgroup {} does not exist, skipping", cgroup_path);
        return Ok(());
    }

    let procs = cgroup_procs(world_id)?;
    if !procs.is_empty() {
        debug!("Cgroup {} has active processes, skipping", cgroup_path);
        return Ok(());
    }

    match std::fs::remove_dir_all(path) {
        Ok(_) => {
            debug!("Removed cgroup directory {}", cgroup_path);
            Ok(())
        }
        Err(e) => {
            debug!("Failed to remove cgroup {}: {}", cgroup_path, e);
            Ok(())
        }
    }
}

fn get_netns_mtime(ns: &str) -> Result<SystemTime> {
    let netns_path = format!("/var/run/netns/{}", ns);
    let metadata = std::fs::metadata(&netns_path)
        .context(format!("Failed to get metadata for {}", netns_path))?;

    metadata
        .modified()
        .context(format!("Failed to get mtime for {}", netns_path))
}

pub async fn sweep(ttl: Option<Duration>) -> Result<GcReport> {
    let mut report = GcReport {
        removed: Vec::new(),
        kept: Vec::new(),
        errors: Vec::new(),
    };

    info!("Starting netns GC sweep");

    let namespaces = match list_netns() {
        Ok(ns) => ns,
        Err(e) => {
            warn!("Failed to list network namespaces: {}", e);
            return Ok(report);
        }
    };

    debug!("Found {} substrate namespaces to check", namespaces.len());

    for ns in namespaces {
        let world_id = match extract_world_id(&ns) {
            Some(id) => id,
            None => {
                debug!("Skipping {}: couldn't extract world_id", ns);
                continue;
            }
        };

        if let Some(ttl_duration) = ttl {
            match get_netns_mtime(&ns) {
                Ok(mtime) => {
                    if let Ok(age) = SystemTime::now().duration_since(mtime) {
                        if age < ttl_duration {
                            debug!("Namespace {} is too recent (age: {:?}), skipping", ns, age);
                            report.kept.push(Kept {
                                name: ns.clone(),
                                reason: format!("too recent (age: {}s)", age.as_secs()),
                            });
                            continue;
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to check mtime for {}: {}", ns, e);
                }
            }
        }

        match netns_pids(&ns) {
            Ok(pids) if !pids.is_empty() => {
                debug!("Namespace {} has active PIDs: {:?}", ns, pids);
                report.kept.push(Kept {
                    name: ns.clone(),
                    reason: format!("active pids: {:?}", pids),
                });
                continue;
            }
            Err(e) => {
                warn!("Failed to check PIDs for {}: {}", ns, e);
                report.errors.push(GcError {
                    name: ns.clone(),
                    message: format!("pid check failed: {}", e),
                });
                continue;
            }
            _ => {}
        }

        match cgroup_procs(&world_id) {
            Ok(procs) if !procs.is_empty() => {
                debug!(
                    "World {} has active cgroup processes: {:?}",
                    world_id, procs
                );
                report.kept.push(Kept {
                    name: ns.clone(),
                    reason: format!("active cgroup procs: {:?}", procs),
                });
                continue;
            }
            Err(e) => {
                debug!("Failed to check cgroup procs for {}: {}", world_id, e);
            }
            _ => {}
        }

        if let Err(e) = delete_nft_table(&ns, &world_id) {
            debug!("Failed to delete nft table for {}: {}", ns, e);
        }

        match delete_netns(&ns) {
            Ok(_) => {
                report.removed.push(ns.clone());

                if let Err(e) = try_rmdir_cgroup(&world_id) {
                    debug!("Failed to remove cgroup for {}: {}", world_id, e);
                }
            }
            Err(e) => {
                warn!("Failed to delete netns {}: {}", ns, e);
                report.errors.push(GcError {
                    name: ns.clone(),
                    message: format!("delete failed: {}", e),
                });
            }
        }
    }

    info!(
        "GC sweep complete: removed={}, kept={}, errors={}",
        report.removed.len(),
        report.kept.len(),
        report.errors.len()
    );

    for removed in &report.removed {
        debug!("Removed: {}", removed);
    }
    for kept in &report.kept {
        debug!("Kept: {} ({})", kept.name, kept.reason);
    }
    for error in &report.errors {
        debug!("Error: {} - {}", error.name, error.message);
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_world_id() {
        assert_eq!(
            extract_world_id("substrate-wld_01994abc123"),
            Some("wld_01994abc123".to_string())
        );

        assert_eq!(extract_world_id("substrate-other"), None);

        assert_eq!(extract_world_id("not-substrate"), None);

        assert_eq!(extract_world_id("substrate-wld_"), Some("wld_".to_string()));
    }

    #[test]
    fn test_gc_report_serialization() {
        let report = GcReport {
            removed: vec!["substrate-wld_test1".to_string()],
            kept: vec![Kept {
                name: "substrate-wld_test2".to_string(),
                reason: "active pids".to_string(),
            }],
            errors: vec![GcError {
                name: "substrate-wld_test3".to_string(),
                message: "permission denied".to_string(),
            }],
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: GcReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.removed.len(), 1);
        assert_eq!(deserialized.kept.len(), 1);
        assert_eq!(deserialized.errors.len(), 1);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod integration_tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::process::Command;
    use std::sync::Mutex;

    // Global mutex to ensure tests run serially
    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn is_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    #[tokio::test]
    async fn test_gc_sweep_removes_empty_netns() {
        let _guard = TEST_MUTEX.lock().unwrap();

        if !is_root() {
            eprintln!("Skipping root-only test");
            return;
        }

        let test_ns = "substrate-wld_GCTEST123";

        // Clean up any existing test namespace
        let _ = Command::new("ip")
            .args(&["netns", "delete", test_ns])
            .output();

        let output = Command::new("ip")
            .args(&["netns", "add", test_ns])
            .output()
            .expect("Failed to create test netns");

        if !output.status.success() {
            panic!("Failed to create test netns: {:?}", output);
        }

        let _ = Command::new("ip")
            .args(&["-n", test_ns, "link", "set", "lo", "up"])
            .output();

        let report = sweep(None).await.expect("Sweep failed");

        // Clean up after test
        let _ = Command::new("ip")
            .args(&["netns", "delete", test_ns])
            .output();

        assert!(
            report.removed.contains(&test_ns.to_string()),
            "Test netns {} should be in removed list. Report: {:?}",
            test_ns,
            report
        );
    }

    #[tokio::test]
    async fn test_gc_sweep_keeps_netns_with_process() {
        let _guard = TEST_MUTEX.lock().unwrap();

        if !is_root() {
            eprintln!("Skipping root-only test");
            return;
        }

        let test_ns = "substrate-wld_GCTEST456";

        // Clean up any existing test namespace
        let _ = Command::new("ip")
            .args(&["netns", "delete", test_ns])
            .output();

        let output = Command::new("ip")
            .args(&["netns", "add", test_ns])
            .output()
            .expect("Failed to create test netns");

        if !output.status.success() {
            panic!("Failed to create test netns: {:?}", output);
        }

        let mut child = Command::new("ip")
            .args(&["netns", "exec", test_ns, "sleep", "60"])
            .spawn()
            .expect("Failed to start process in netns");

        // Give the process a moment to start
        std::thread::sleep(std::time::Duration::from_millis(100));

        let report = sweep(None).await.expect("Sweep failed");

        let kept_names: Vec<String> = report.kept.iter().map(|k| k.name.clone()).collect();

        // Clean up: kill the process and delete the namespace
        child.kill().ok();
        child.wait().ok();

        // Wait a moment for process to fully exit
        std::thread::sleep(std::time::Duration::from_millis(100));

        let _ = Command::new("ip")
            .args(&["netns", "delete", test_ns])
            .output();

        assert!(
            kept_names.contains(&test_ns.to_string()),
            "Test netns with process should be kept. Report: {:?}",
            report
        );
    }
}
