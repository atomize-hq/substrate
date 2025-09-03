//! DNS resolution and pinning for network isolation.

use anyhow::Result;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct DnsResolver {
    allowed_domains: Vec<String>,
    resolved_ips: RwLock<HashMap<String, CachedResolution>>,
}

struct CachedResolution {
    ips: Vec<IpAddr>,
    expires_at: Instant,
}

impl DnsResolver {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self {
            allowed_domains,
            resolved_ips: RwLock::new(HashMap::new()),
        }
    }

    pub fn spawn_resolver(allowed_domains: Vec<String>) -> Result<()> {
        let resolver = std::sync::Arc::new(Self::new(allowed_domains));

        // Spawn background task to refresh IPs
        std::thread::spawn(move || {
            loop {
                if let Err(e) = resolver.refresh_all() {
                    eprintln!("DNS resolver error: {}", e);
                }
                std::thread::sleep(Duration::from_secs(60)); // Refresh every minute
            }
        });

        Ok(())
    }

    fn refresh_all(&self) -> Result<()> {
        let mut cache = self.resolved_ips.write().unwrap();

        for domain in &self.allowed_domains {
            match dns_lookup::lookup_host(domain) {
                Ok(ips) => {
                    cache.insert(
                        domain.clone(),
                        CachedResolution {
                            ips,
                            expires_at: Instant::now() + Duration::from_secs(300), // 5 min TTL
                        },
                    );
                }
                Err(e) => {
                    eprintln!("Failed to resolve {}: {}", domain, e);
                }
            }
        }

        // Update nftables set atomically
        self.update_nftables_set(&cache)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn update_nftables_set(&self, cache: &HashMap<String, CachedResolution>) -> Result<()> {
        use std::process::Command;

        let mut all_ips = Vec::new();
        for resolution in cache.values() {
            all_ips.extend(&resolution.ips);
        }

        // Ensure table exists, then atomic set update
        let cmds = format!(
            "nft list table inet substrate >/dev/null 2>&1 || nft add table inet substrate\n\
             nft add set inet substrate allowed_ips {{ type ipv4_addr; flags interval; }} 2>/dev/null || true\n\
             nft flush set inet substrate allowed_ips\n\
             nft add element inet substrate allowed_ips {{ {} }}",
            all_ips.iter().map(|ip| ip.to_string()).collect::<Vec<_>>().join(", ")
        );

        Command::new("sh").arg("-c").arg(&cmds).output()?;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn update_nftables_set(&self, _cache: &HashMap<String, CachedResolution>) -> Result<()> {
        // Stub for non-Linux platforms
        Ok(())
    }

    pub fn setup_dns_stub(&self, root_dir: &std::path::Path) -> Result<()> {
        // Create resolv.conf pointing to our stub
        let resolv_conf = root_dir.join("etc/resolv.conf");
        if let Some(parent) = resolv_conf.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            resolv_conf,
            "nameserver 127.0.0.53\noptions edns0 trust-ad\n",
        )?;

        // Start dnsmasq stub on 127.0.0.53
        self.start_stub_resolver()?;
        Ok(())
    }

    fn start_stub_resolver(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;

            // Start dnsmasq in the background
            Command::new("dnsmasq")
                .args([
                    "--no-resolv",
                    "--server=1.1.1.1",
                    "--listen-address=127.0.0.53",
                    "--bind-interfaces",
                    "--cache-size=1000",
                    "--pid-file=/run/substrate/dnsmasq.pid",
                ])
                .spawn()?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  DNS stub resolver not supported on this platform");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_resolver_creation() {
        let domains = vec!["example.com".to_string()];
        let resolver = DnsResolver::new(domains);
        assert_eq!(resolver.allowed_domains.len(), 1);
    }
}
