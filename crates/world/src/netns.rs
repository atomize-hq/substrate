//! Lightweight helpers for named Linux network namespaces via iproute2.
//!
//! Best-effort only; callers should print `[replay] warn: ...` messages on
//! privilege or availability issues and degrade gracefully.

use anyhow::{Context, Result};
use std::process::Command;

pub struct NetNs {
    name: String,
    active: bool,
}

impl NetNs {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if `ip` command exists.
    pub fn ip_available() -> bool {
        Command::new("sh")
            .arg("-lc")
            .arg("command -v ip >/dev/null 2>&1")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Create the named netns.
    pub fn add(&mut self) -> Result<bool> {
        let status = Command::new("ip")
            .args(["netns", "add", &self.name])
            .status();
        match status {
            Ok(s) if s.success() => {
                self.active = true;
                Ok(true)
            }
            Ok(s) => Err(anyhow::anyhow!("ip netns add exited with status {}", s)),
            Err(e) => Err(e).context("failed to run ip netns add"),
        }
    }

    /// Bring loopback up in the namespace.
    pub fn lo_up(&self) -> Result<bool> {
        if !self.active {
            return Err(anyhow::anyhow!("netns not active"));
        }
        let status = Command::new("ip")
            .args(["-n", &self.name, "link", "set", "lo", "up"])
            .status();
        match status {
            Ok(s) if s.success() => Ok(true),
            Ok(s) => Err(anyhow::anyhow!(
                "ip -n {} link set lo up exited with status {}",
                self.name,
                s
            )),
            Err(e) => Err(e).context("failed to run ip -n <ns> link set lo up"),
        }
    }

    /// Delete the namespace (best-effort).
    pub fn delete(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let _ = Command::new("ip")
            .args(["netns", "delete", &self.name])
            .status();
        self.active = false;
        Ok(())
    }
}

impl Drop for NetNs {
    fn drop(&mut self) {
        let _ = self.delete();
    }
}
