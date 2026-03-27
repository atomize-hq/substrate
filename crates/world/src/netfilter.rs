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
use std::net::ToSocketAddrs;
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

pub fn world_netfilter_enable_present() -> bool {
    std::env::var("WORLD_NETFILTER_ENABLE")
        .ok()
        .is_some_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
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
    cgroup_match: Option<CgroupMatch>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct CgroupMatch {
    level: u32,
    value: String,
}

impl NetFilter {
    #[cfg(target_os = "linux")]
    fn netfilter_enabled() -> bool {
        world_netfilter_enable_present()
    }

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
        let Ok(rel) = path.strip_prefix(base) else {
            self.cgroup_match = None;
            return;
        };
        let components = rel
            .iter()
            .filter_map(|c| c.to_str())
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>();
        if components.is_empty() {
            self.cgroup_match = None;
        } else {
            self.cgroup_match = Some(CgroupMatch {
                level: components.len() as u32,
                value: components.join("/"),
            });
        }
    }

    /// Resolve allowed domains to IP addresses.
    pub fn resolve_domains(&mut self) -> Result<()> {
        for domain in &self.allowed_domains {
            #[cfg(target_os = "linux")]
            let resolved = (domain.as_str(), 0)
                .to_socket_addrs()
                .with_context(|| format!("failed to resolve allowed domain `{domain}`"))?
                .map(|addr| addr.ip())
                .collect::<HashSet<_>>();

            #[cfg(not(target_os = "linux"))]
            let resolved = dns_lookup::lookup_host(domain)
                .map(|ips| ips.into_iter().collect::<HashSet<_>>())?;

            if resolved.is_empty() {
                anyhow::bail!("allowed domain `{domain}` resolved to no addresses");
            }

            for ip in resolved {
                self.allowed_ips.insert(ip);
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
            if !Self::netfilter_enabled() {
                anyhow::bail!(
                    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
                );
            }

            self.install_rules_linux()?;
            self.is_active = true;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Network filtering only works on Linux.
            eprintln!("⚠️  Network filtering not available on this platform");
            self.is_active = true;
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    fn allow_rule_bodies(&self) -> Vec<String> {
        vec![
            "oif lo accept".to_string(),
            "ct state established,related accept".to_string(),
            "ip daddr @allowed4 accept".to_string(),
            "ip6 daddr @allowed6 accept".to_string(),
            format!(
                "limit rate 10/second log prefix \"substrate-dropped-{}:\"",
                self.world_id
            ),
            "counter drop".to_string(),
        ]
    }

    #[cfg(target_os = "linux")]
    fn allowed_ip_element_spec(&self, ip: &IpAddr) -> String {
        match ip {
            IpAddr::V4(v4) => format!("element inet {} allowed4 {{ {} }}", self.table_name, v4),
            IpAddr::V6(v6) => format!("element inet {} allowed6 {{ {} }}", self.table_name, v6),
        }
    }

    #[cfg(target_os = "linux")]
    fn install_rules_linux(&self) -> Result<()> {
        // Safety guard: never attach an output-hook chain without scoping.
        //
        // A non-scoped output hook affects the entire host, and when paired with a final `drop`
        // rule can sever host networking for the duration of the session (or longer if cleanup
        // fails). Require either a dedicated netns or cgroupv2 socket matching.
        if self.ns_name.is_none() && self.cgroup_clause().is_none() {
            return Err(anyhow!(
                "refusing to install nftables output-hook without netns/cgroup scoping"
            ));
        }

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
            self.run_nft(&["add", &set_v4])?;
            let set_v6 = format!(
                "set inet {} allowed6 {{ type ipv6_addr; flags interval; }}",
                self.table_name
            );
            self.run_nft(&["add", &set_v6])?;

            // Populate sets with allowed IPs
            for ip in &self.allowed_ips {
                let add_elem = self.allowed_ip_element_spec(ip);
                self.run_nft(&["add", &add_elem])?;
            }

            for body in self.allow_rule_bodies() {
                let rule = self.format_rule(&body);
                self.run_nft(&["add", &rule])?;
            }

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
        self.cgroup_match
            .as_ref()
            .map(|entry| format!("socket cgroupv2 level {} \"{}\"", entry.level, entry.value))
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
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(target_os = "linux")]
    use std::sync::Mutex;
    #[cfg(target_os = "linux")]
    use tempfile::tempdir;

    #[cfg(target_os = "linux")]
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(target_os = "linux")]
    struct EnvGuard {
        previous: Vec<(String, Option<std::ffi::OsString>)>,
    }

    #[cfg(target_os = "linux")]
    impl EnvGuard {
        fn set(vars: &[(&str, Option<&str>)]) -> Self {
            let previous = vars
                .iter()
                .map(|(key, _)| (key.to_string(), std::env::var_os(key)))
                .collect::<Vec<_>>();
            for (key, value) in vars {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
            Self { previous }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
                match value {
                    Some(v) => std::env::set_var(&key, v),
                    None => std::env::remove_var(&key),
                }
            }
        }
    }

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
    fn test_empty_allowlist_skips_resolution() {
        let mut filter = NetFilter::new("test_world", vec![]).unwrap();

        filter.resolve_domains().unwrap();

        assert!(filter.allowed_ips.is_empty());
    }

    #[test]
    fn test_domain_resolution_fails_for_unresolvable_host() {
        let mut filter = NetFilter::new(
            "test_world",
            vec!["definitely-not-a-real-substrate-test-host.invalid".to_string()],
        )
        .unwrap();

        let err = filter.resolve_domains().unwrap_err();
        let message = format!("{err:#}");
        assert!(
            message.contains("failed to resolve allowed domain")
                || message.contains("failed to lookup address information")
                || message.contains("resolved to no addresses"),
            "unexpected error: {message}"
        );
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
    fn test_deny_all_rule_bodies_do_not_allow_dns() {
        let filter = NetFilter::new("test_world", vec![]).unwrap();

        let rules = filter.allow_rule_bodies();
        assert!(
            !rules.iter().any(|rule| rule.contains("dport 53")),
            "deny-all rules must not include a DNS allow rule: {rules:?}"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_install_rules_requires_world_netfilter_enable() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", None)]);
        let mut filter = NetFilter::new("test_world", vec![]).unwrap();

        let err = filter.install_rules().unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("WORLD_NETFILTER_ENABLE"));
        assert!(!filter.is_active);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_install_rules_errors_when_nft_fails() {
        let temp = tempdir().unwrap();
        let fake_nft = temp.path().join("nft");
        std::fs::write(&fake_nft, "#!/bin/sh\nexit 17\n").unwrap();
        let mut perms = std::fs::metadata(&fake_nft).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&fake_nft, perms).unwrap();

        let original_path = std::env::var("PATH").unwrap_or_default();
        let stubbed_path = format!("{}:{original_path}", temp.path().display());

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("WORLD_NETFILTER_ENABLE", Some("1")),
            ("PATH", Some(&stubbed_path)),
        ]);

        let mut filter = NetFilter::new("test_world", vec![]).unwrap();
        filter.set_cgroup_path(Path::new("/sys/fs/cgroup/substrate/test_world"));

        let err = filter.install_rules().unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("nft command failed"));
        assert!(!filter.is_active);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_install_rules_adds_allowed_ip_elements_without_duplicate_add_keyword() {
        let temp = tempdir().unwrap();
        let fake_nft = temp.path().join("nft");
        let log_path = temp.path().join("nft.log");
        std::fs::write(
            &fake_nft,
            format!(
                "#!/bin/sh\nprintf '%s|' \"$@\" >> \"{}\"\nprintf '\\n' >> \"{}\"\nexit 0\n",
                log_path.display(),
                log_path.display()
            ),
        )
        .unwrap();
        let mut perms = std::fs::metadata(&fake_nft).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&fake_nft, perms).unwrap();

        let original_path = std::env::var("PATH").unwrap_or_default();
        let stubbed_path = format!("{}:{original_path}", temp.path().display());

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("WORLD_NETFILTER_ENABLE", Some("1")),
            ("PATH", Some(&stubbed_path)),
        ]);

        let mut filter = NetFilter::new("test_world", vec![]).unwrap();
        filter.allowed_ips.insert("203.0.113.10".parse().unwrap());
        filter.set_cgroup_path(Path::new("/sys/fs/cgroup/substrate/test_world"));

        filter.install_rules().expect("install rules");

        let log = std::fs::read_to_string(&log_path).expect("read nft log");
        assert!(
            log.contains("add|element inet substrate_test_world allowed4 { 203.0.113.10 }|"),
            "expected add element command without duplicated keyword, got log: {log}"
        );
        assert!(
            !log.contains("add|add element"),
            "unexpected duplicated add keyword in nft command log: {log}"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_install_rules_fail_closed_when_allowed_ip_population_fails() {
        let temp = tempdir().unwrap();
        let fake_nft = temp.path().join("nft");
        std::fs::write(
            &fake_nft,
            "#!/bin/sh\ncase \"$2\" in\n  element\\ *)\n    echo 'simulated add element failure' >&2\n    exit 23\n    ;;\nesac\nexit 0\n",
        )
        .unwrap();
        let mut perms = std::fs::metadata(&fake_nft).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&fake_nft, perms).unwrap();

        let original_path = std::env::var("PATH").unwrap_or_default();
        let stubbed_path = format!("{}:{original_path}", temp.path().display());

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("WORLD_NETFILTER_ENABLE", Some("1")),
            ("PATH", Some(&stubbed_path)),
        ]);

        let mut filter = NetFilter::new("test_world", vec![]).unwrap();
        filter.allowed_ips.insert("203.0.113.10".parse().unwrap());
        filter.set_cgroup_path(Path::new("/sys/fs/cgroup/substrate/test_world"));

        let err = filter.install_rules().unwrap_err();
        let message = format!("{err:#}");
        assert!(
            message.contains("simulated add element failure"),
            "unexpected error: {message}"
        );
        assert!(!filter.is_active);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cgroup_clause_uses_full_relative_path_at_deepest_level() {
        let mut filter = NetFilter::new("test_world", vec![]).unwrap();
        filter.set_cgroup_path(Path::new("/sys/fs/cgroup/substrate/test_world"));

        assert_eq!(
            filter.cgroup_clause().as_deref(),
            Some("socket cgroupv2 level 2 \"substrate/test_world\"")
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cgroup_clause_is_disabled_for_non_cgroup_paths() {
        let mut filter = NetFilter::new("test_world", vec![]).unwrap();
        filter.set_cgroup_path(Path::new("/run/user/1000/substrate/cgroup/test_world"));

        assert!(filter.cgroup_clause().is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[ignore = "requires root + iproute2 + nftables; run with `cargo test -p world -- --ignored --nocapture`"]
    fn test_nftables_rules() {
        // This test requires root privileges.
        if !nix::unistd::Uid::current().is_root() {
            println!("Skipping nftables test (requires root)");
            return;
        }

        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(&[("WORLD_NETFILTER_ENABLE", Some("1"))]);

        let mut filter = NetFilter::new("test_nft", vec!["github.com".to_string()]).unwrap();

        // Always run inside an isolated netns so we cannot disrupt host networking.
        let netns_name = format!("substrate_test_nft_{}", std::process::id());
        let add_status = match Command::new("ip")
            .args(["netns", "add", &netns_name])
            .status()
        {
            Ok(status) => status,
            Err(err) => {
                println!("Skipping nftables test (failed to run `ip netns add`: {err})");
                return;
            }
        };
        if !add_status.success() {
            println!("Skipping nftables test (`ip netns add` failed)");
            return;
        }

        struct NetnsGuard {
            name: String,
        }
        impl Drop for NetnsGuard {
            fn drop(&mut self) {
                let _ = Command::new("ip")
                    .args(["netns", "delete", &self.name])
                    .status();
            }
        }
        let _guard = NetnsGuard {
            name: netns_name.clone(),
        };

        filter.set_namespace(Some(netns_name));

        // Resolve and install rules
        filter.resolve_domains().unwrap();
        filter.install_rules().unwrap();
        assert!(filter.is_active);

        // Cleanup
        filter.remove_rules().unwrap();
        assert!(!filter.is_active);
    }
}
