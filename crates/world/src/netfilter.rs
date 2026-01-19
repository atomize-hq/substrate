//! Network filtering and scope tracking using nftables.
//!
//! This module provides network isolation and tracking capabilities using
//! Linux nftables for policy enforcement and scope monitoring.

#[cfg(target_os = "linux")]
use anyhow::anyhow;
#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
use std::collections::HashSet;
#[cfg(target_os = "linux")]
use std::io::Read;
use std::net::IpAddr;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
fn monitor_timeout() -> Duration {
    std::env::var("WORLD_NETFILTER_MONITOR_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(1_000))
}

#[cfg(target_os = "linux")]
fn output_with_timeout(mut cmd: Command, timeout: Duration) -> Result<std::process::Output> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn command")?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_end(&mut stderr);
                }
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(anyhow!("command timed out after {}ms", timeout.as_millis()));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(anyhow!(err).context("Failed to poll command completion")),
        }
    }
}

/// Network scope tracking for command execution.
#[derive(Debug, Clone)]
pub struct NetworkScope {
    /// Domain name or IP address accessed.
    pub target: String,
    /// Port number if applicable.
    pub port: Option<u16>,
    /// Protocol (tcp/udp).
    pub protocol: String,
    /// Number of bytes transferred.
    pub bytes: usize,
}

/// Network filter manager using nftables.
pub struct NetFilter {
    #[allow(dead_code)]
    world_id: String,
    #[allow(dead_code)]
    table_name: String,
    #[allow(dead_code)]
    chain_name: String,
    allowed_domains: Vec<String>,
    allowed_ips: HashSet<IpAddr>,
    scopes_used: Vec<NetworkScope>,
    is_active: bool,
    /// Optional network namespace name to scope nft commands
    ns_name: Option<String>,
    #[cfg(target_os = "linux")]
    cgroup_match: Option<Vec<CgroupMatch>>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct CgroupMatch {
    level: u32,
    value: String,
}

impl NetFilter {
    /// Create a new network filter for the given world.
    pub fn new(world_id: &str, allowed_domains: Vec<String>) -> Result<Self> {
        let table_name = format!("substrate_{}", world_id);
        let chain_name = format!("filter_{}", world_id);

        Ok(Self {
            world_id: world_id.to_string(),
            table_name,
            chain_name,
            allowed_domains,
            allowed_ips: HashSet::new(),
            scopes_used: Vec::new(),
            is_active: false,
            ns_name: None,
            #[cfg(target_os = "linux")]
            cgroup_match: None,
        })
    }

    /// Scope nft calls to the given named netns.
    pub fn set_namespace(&mut self, ns: Option<String>) {
        self.ns_name = ns;
        #[cfg(target_os = "linux")]
        {
            if self.ns_name.is_some() {
                self.cgroup_match = None;
            }
        }
    }

    /// Scope nft calls to a socket cgroup path (host namespace fallback).
    #[cfg(target_os = "linux")]
    pub fn set_cgroup_path(&mut self, path: &Path) {
        self.ns_name = None;
        let base = Path::new("/sys/fs/cgroup");
        let rel = path.strip_prefix(base).unwrap_or(path);
        let mut entries = Vec::new();
        for (idx, component) in rel
            .iter()
            .filter_map(|c| c.to_str())
            .filter(|c| !c.is_empty())
            .enumerate()
        {
            entries.push(CgroupMatch {
                level: (idx as u32) + 1,
                value: component.to_string(),
            });
        }
        if entries.is_empty() {
            self.cgroup_match = None;
        } else {
            self.cgroup_match = Some(entries);
        }
    }

    /// Resolve allowed domains to IP addresses.
    pub fn resolve_domains(&mut self) -> Result<()> {
        for domain in &self.allowed_domains {
            // Use dns-lookup to resolve domain to IPs
            if let Ok(ips) = dns_lookup::lookup_host(domain) {
                for ip in ips {
                    self.allowed_ips.insert(ip);
                }
            }
        }
        Ok(())
    }

    /// Install nftables rules for network filtering.
    pub fn install_rules(&mut self) -> Result<()> {
        if self.is_active {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            self.install_rules_linux()?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Network filtering only works on Linux
            eprintln!("⚠️  Network filtering not available on this platform");
        }

        self.is_active = true;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn install_rules_linux(&self) -> Result<()> {
        // IMPORTANT SAFETY PROPERTY:
        // Never leave a partially-installed output-hook chain behind that could disrupt host networking.
        // Any error after table/chain creation must roll back the whole table.

        // Create nftables table
        let create_table = format!("table inet {}", self.table_name);
        self.run_nft(&["add", &create_table])?;

        // Create base chain for filtering.
        //
        // Use `policy accept` and enforce via an explicit final `drop` rule. This makes partial setup
        // fail-open (safe) rather than fail-closed (can brick host networking if later rule-adds fail).
        let create_chain = format!(
            "chain inet {} {} {{ type filter hook output priority 0; policy accept; }}",
            self.table_name, self.chain_name
        );

        let result: Result<()> = (|| {
            self.run_nft(&["add", &create_chain])?;

            // Create IPv4/IPv6 sets for allowed destinations (idempotent add)
            let set_v4 = format!(
                "set inet {} allowed4 {{ type ipv4_addr; flags interval; }}",
                self.table_name
            );
            let _ = self.run_nft(&["add", &set_v4]);
            let set_v6 = format!(
                "set inet {} allowed6 {{ type ipv6_addr; flags interval; }}",
                self.table_name
            );
            let _ = self.run_nft(&["add", &set_v6]);

            // Allow loopback traffic
            let allow_loopback = self.format_rule("oif lo accept");
            self.run_nft(&["add", &allow_loopback])?;

            // Allow established connections
            let allow_established = self.format_rule("ct state established,related accept");
            self.run_nft(&["add", &allow_established])?;

            // Allow DNS queries
            let allow_dns = self.format_rule("udp dport 53 accept");
            self.run_nft(&["add", &allow_dns])?;

            // Populate sets with allowed IPs
            for ip in &self.allowed_ips {
                match ip {
                    IpAddr::V4(v4) => {
                        let add_elem =
                            format!("add element inet {} allowed4 {{ {} }}", self.table_name, v4);
                        let _ = self.run_nft(&["add", &add_elem]);
                    }
                    IpAddr::V6(v6) => {
                        let add_elem =
                            format!("add element inet {} allowed6 {{ {} }}", self.table_name, v6);
                        let _ = self.run_nft(&["add", &add_elem]);
                    }
                }
            }

            // Allow traffic to addresses in sets
            let allow_v4 = self.format_rule("ip daddr @allowed4 accept");
            let allow_v6 = self.format_rule("ip6 daddr @allowed6 accept");
            self.run_nft(&["add", &allow_v4])?;
            self.run_nft(&["add", &allow_v6])?;

            // Rate-limited LOG + drop for everything else.
            // (Safe even with `policy accept` because this is an explicit final drop rule.)
            let log_dropped = self.format_rule(&format!(
                "limit rate 10/second log prefix \"substrate-dropped-{}:\"",
                self.world_id
            ));
            let drop_rule = self.format_rule("counter drop");
            self.run_nft(&["add", &log_dropped])?;
            self.run_nft(&["add", &drop_rule])?;

            Ok(())
        })();

        if let Err(err) = result {
            // Best-effort rollback.
            let delete_table = format!("table inet {}", self.table_name);
            let _ = self.run_nft(&["delete", &delete_table]);
            return Err(err);
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn cgroup_clause(&self) -> Option<String> {
        self.cgroup_match.as_ref().map(|entries| {
            entries
                .iter()
                .map(|entry| format!("socket cgroupv2 level {} \"{}\"", entry.level, entry.value))
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    #[cfg(target_os = "linux")]
    fn format_rule(&self, body: &str) -> String {
        if let Some(clause) = self.cgroup_clause() {
            format!(
                "rule inet {} {} {} {}",
                self.table_name, self.chain_name, clause, body
            )
        } else {
            format!("rule inet {} {} {}", self.table_name, self.chain_name, body)
        }
    }

    #[cfg(target_os = "linux")]
    fn run_nft(&self, args: &[&str]) -> Result<()> {
        fn nft_timeout() -> Duration {
            std::env::var("WORLD_NFT_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.trim().parse::<u64>().ok())
                .map(Duration::from_millis)
                .unwrap_or_else(|| Duration::from_millis(2_000))
        }

        fn output_with_timeout(
            mut cmd: Command,
            timeout: Duration,
        ) -> Result<std::process::Output> {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = cmd.spawn().context("Failed to spawn nft command")?;
            let start = Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let mut stdout = Vec::new();
                        let mut stderr = Vec::new();
                        if let Some(mut out) = child.stdout.take() {
                            let _ = out.read_to_end(&mut stdout);
                        }
                        if let Some(mut err) = child.stderr.take() {
                            let _ = err.read_to_end(&mut stderr);
                        }
                        return Ok(std::process::Output {
                            status,
                            stdout,
                            stderr,
                        });
                    }
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            return Err(anyhow!(
                                "nft command timed out after {}ms",
                                timeout.as_millis()
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(25));
                    }
                    Err(err) => {
                        return Err(anyhow!(err).context("Failed to poll nft command completion"))
                    }
                }
            }
        }

        let timeout = nft_timeout();
        let output = if let Some(ref ns) = self.ns_name {
            let mut cmd = Command::new("ip");
            cmd.args(["netns", "exec", ns, "nft"]).args(args);
            output_with_timeout(cmd, timeout).with_context(|| {
                format!(
                    "Failed to run ip netns exec nft (timeout={}ms)",
                    timeout.as_millis()
                )
            })?
        } else {
            let mut cmd = Command::new("nft");
            cmd.args(args);
            output_with_timeout(cmd, timeout)
                .with_context(|| format!("Failed to run nft (timeout={}ms)", timeout.as_millis()))?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nft command failed: {}", stderr);
        }

        Ok(())
    }

    /// Remove nftables rules.
    pub fn remove_rules(&mut self) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            // Delete the entire table (and all its chains/rules)
            let delete_table = format!("table inet {}", self.table_name);
            let _ = self.run_nft(&["delete", &delete_table]);
        }

        self.is_active = false;
        Ok(())
    }

    /// Track a network scope usage.
    pub fn track_scope(
        &mut self,
        target: String,
        port: Option<u16>,
        protocol: String,
        bytes: usize,
    ) {
        self.scopes_used.push(NetworkScope {
            target,
            port,
            protocol,
            bytes,
        });
    }

    /// Get the list of network scopes used.
    pub fn get_scopes_used(&self) -> Vec<String> {
        self.scopes_used
            .iter()
            .map(|scope| {
                if let Some(port) = scope.port {
                    format!("{}:{}:{}", scope.protocol, scope.target, port)
                } else {
                    format!("{}:{}", scope.protocol, scope.target)
                }
            })
            .collect()
    }

    /// Parse network activity from system logs or packet captures.
    pub fn parse_network_activity(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // Parse kernel log for dropped packets
            self.parse_dropped_packets()?;

            // Parse connection tracking for allowed connections
            self.parse_conntrack()?;
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn parse_dropped_packets(&mut self) -> Result<()> {
        // WSL kernel logs can be very large and `dmesg` access/latency can be unpredictable.
        // Scope monitoring should never block command completion, so skip on WSL.
        if std::env::var_os("WSL_INTEROP").is_some() {
            return Ok(());
        }

        // Read kernel log for dropped packets with our prefix
        let output = {
            let mut cmd = Command::new("dmesg");
            cmd.args(["-t"]);
            output_with_timeout(cmd, monitor_timeout())
        }
        .context("Failed to read kernel log")?;

        let log = String::from_utf8_lossy(&output.stdout);
        let prefix = format!("substrate-dropped-{}:", self.world_id);

        for line in log.lines() {
            if line.contains(&prefix) {
                // Parse the dropped packet info
                // Format: substrate-dropped-<world_id>: IN=... OUT=... DST=<ip> ...
                if let Some(dst_start) = line.find("DST=") {
                    let dst_part = &line[dst_start + 4..];
                    if let Some(space_pos) = dst_part.find(' ') {
                        let dst_ip = &dst_part[..space_pos];

                        // Track this as a blocked scope
                        self.track_scope(format!("blocked:{}", dst_ip), None, "tcp".to_string(), 0);
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn parse_conntrack(&mut self) -> Result<()> {
        // Parse connection tracking table for active connections
        if std::env::var_os("WSL_INTEROP").is_some() {
            return Ok(());
        }

        let output = {
            let mut cmd = Command::new("conntrack");
            cmd.args(["-L", "-n"]);
            output_with_timeout(cmd, monitor_timeout())
        };

        if let Ok(output) = output {
            let conntrack = String::from_utf8_lossy(&output.stdout);

            for line in conntrack.lines() {
                // Parse conntrack entries
                // Format: tcp      6 431999 ESTABLISHED src=... dst=<ip> sport=... dport=<port> ...
                if line.contains("ESTABLISHED") {
                    let parts: Vec<&str> = line.split_whitespace().collect();

                    let mut protocol = "tcp";
                    let mut dst_ip = None;
                    let mut dst_port = None;

                    for (i, part) in parts.iter().enumerate() {
                        if i == 0 {
                            protocol = part;
                        } else if let Some(value) = part.strip_prefix("dst=") {
                            dst_ip = Some(value.to_string());
                        } else if let Some(port_str) = part.strip_prefix("dport=") {
                            dst_port = port_str.parse().ok();
                        }
                    }

                    if let Some(ip) = dst_ip {
                        // Check if this IP is in our allowed list
                        if let Ok(addr) = ip.parse::<IpAddr>() {
                            if self.allowed_ips.contains(&addr) {
                                // Reverse lookup to get domain if possible
                                let target = self.reverse_lookup(&ip).unwrap_or(ip);
                                self.track_scope(target, dst_port, protocol.to_string(), 0);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn reverse_lookup(&self, ip: &str) -> Option<String> {
        // Try to find the domain that resolved to this IP
        for domain in &self.allowed_domains {
            if let Ok(ips) = dns_lookup::lookup_host(domain) {
                if ips.iter().any(|addr| addr.to_string() == ip) {
                    return Some(domain.clone());
                }
            }
        }
        None
    }
}

impl Drop for NetFilter {
    fn drop(&mut self) {
        // Best effort cleanup on drop
        let _ = self.remove_rules();
    }
}

/// Apply network filtering for a world.
pub fn apply_network_filter(world_id: &str, allowed_domains: Vec<String>) -> Result<NetFilter> {
    let mut filter = NetFilter::new(world_id, allowed_domains)?;

    // Resolve domains to IPs
    filter.resolve_domains()?;

    // Install filtering rules
    filter.install_rules()?;

    Ok(filter)
}

/// Monitor network activity and return scopes used.
pub fn monitor_network_scopes(filter: &mut NetFilter) -> Result<Vec<String>> {
    // Best-effort: scope monitoring must never fail the underlying command.
    if let Err(err) = filter.parse_network_activity() {
        tracing::debug!(
            target: "world::netfilter",
            error = %err,
            "network scope monitoring failed; returning empty scopes"
        );
        return Ok(Vec::new());
    }

    // Return the formatted scope list
    Ok(filter.get_scopes_used())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netfilter_creation() {
        let filter = NetFilter::new("test_world", vec!["github.com".to_string()]).unwrap();

        assert_eq!(filter.world_id, "test_world");
        assert_eq!(filter.table_name, "substrate_test_world");
        assert!(!filter.is_active);
    }

    #[test]
    fn test_domain_resolution() {
        let mut filter = NetFilter::new("test_world", vec!["localhost".to_string()]).unwrap();

        filter.resolve_domains().unwrap();

        // localhost should resolve to at least one IP
        assert!(!filter.allowed_ips.is_empty());
    }

    #[test]
    fn test_scope_tracking() {
        let mut filter = NetFilter::new("test_world", vec![]).unwrap();

        filter.track_scope("github.com".to_string(), Some(443), "tcp".to_string(), 1024);
        filter.track_scope("npmjs.org".to_string(), Some(443), "tcp".to_string(), 2048);

        let scopes = filter.get_scopes_used();
        assert_eq!(scopes.len(), 2);
        assert!(scopes.contains(&"tcp:github.com:443".to_string()));
        assert!(scopes.contains(&"tcp:npmjs.org:443".to_string()));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_nftables_rules() {
        // This test requires root privileges
        if !nix::unistd::Uid::current().is_root() {
            println!("Skipping nftables test (requires root)");
            return;
        }

        let mut filter = NetFilter::new("test_nft", vec!["github.com".to_string()]).unwrap();

        // Resolve and install rules
        filter.resolve_domains().unwrap();
        filter.install_rules().unwrap();
        assert!(filter.is_active);

        // Cleanup
        filter.remove_rules().unwrap();
        assert!(!filter.is_active);
    }
}
