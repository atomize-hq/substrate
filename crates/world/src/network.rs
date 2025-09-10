#![cfg(test)]
//! Network isolation using nftables and network namespaces.

use anyhow::Result;

/// Network isolation manager for worlds.
pub struct NetworkIsolation {
    allowed_domains: Vec<String>,
    namespace_name: Option<String>,
}

impl NetworkIsolation {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self {
            allowed_domains,
            namespace_name: None,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn setup_network_namespace(&mut self) -> Result<()> {
        use std::process::Command;

        let ns_name = format!("substrate-{}", uuid::Uuid::now_v7());

        // Create network namespace
        Command::new("ip")
            .args(["netns", "add", &ns_name])
            .status()?;

        self.namespace_name = Some(ns_name.clone());

        // Setup nftables rules
        self.setup_nftables(&ns_name)?;

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn setup_network_namespace(&mut self) -> Result<()> {
        eprintln!("⚠️  Network namespaces not supported on this platform");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    fn setup_nftables(&self, ns_name: &str) -> Result<()> {
        use std::process::Command;

        let rules = format!(
            r#"
            nft add table inet substrate
            nft add set inet substrate allowed_ips {{ type ipv4_addr; flags interval; }}
            nft add chain inet substrate output {{ type filter hook output priority 0; }}
            nft add rule inet substrate output ip daddr @allowed_ips tcp dport 443 accept
            nft add rule inet substrate output ip daddr 127.0.0.0/8 accept
            nft add rule inet substrate output ip6 daddr ::/0 drop
            nft add rule inet substrate output drop
            "#
        );

        // Execute rules in the network namespace
        Command::new("ip")
            .args(["netns", "exec", ns_name, "sh", "-c", &rules])
            .status()?;

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)]
    fn setup_nftables(&self, _ns_name: &str) -> Result<()> {
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if let Some(ref ns_name) = self.namespace_name {
                std::process::Command::new("ip")
                    .args(["netns", "del", ns_name])
                    .status()
                    .ok(); // Best effort cleanup
            }
        }
        Ok(())
    }
}

impl Drop for NetworkIsolation {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_isolation_creation() {
        let domains = vec!["example.com".to_string()];
        let isolation = NetworkIsolation::new(domains);
        assert_eq!(isolation.allowed_domains.len(), 1);
        assert!(isolation.namespace_name.is_none());
    }
}
