//! Helpers for reporting isolation cleanup diagnostics.

use std::collections::BTreeSet;

/// Summary of leftover isolation artifacts detected from command output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupReport {
    netns: Vec<String>,
    nft_tables: Vec<(String, String)>,
}

impl CleanupReport {
    /// Build a cleanup report from `ip netns list` and `nft list tables` output.
    pub fn from_outputs(netns_output: &str, nft_output: &str) -> Self {
        Self {
            netns: parse_netns_output(netns_output),
            nft_tables: parse_nft_output(nft_output),
        }
    }

    /// Returns true when either namespaces or nft tables are still present.
    pub fn has_findings(&self) -> bool {
        !self.netns.is_empty() || !self.nft_tables.is_empty()
    }

    /// Leftover namespaces detected in the diagnostic output.
    pub fn netns_leftovers(&self) -> &[String] {
        &self.netns
    }

    /// Leftover nft tables detected in the diagnostic output.
    pub fn nft_table_leftovers(&self) -> &[(String, String)] {
        &self.nft_tables
    }

    /// Human-readable lines describing each leftover artifact.
    pub fn describe(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for ns in &self.netns {
            lines.push(format!("leftover netns detected: {}", ns));
        }
        for (family, table) in &self.nft_tables {
            lines.push(format!(
                "leftover nft table detected: family={} table={}",
                family, table
            ));
        }
        lines
    }
}

fn parse_netns_output(output: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(first) = trimmed.split_whitespace().next() {
            if first.starts_with("substrate-") {
                names.insert(first.to_string());
            }
        }
    }
    names.into_iter().collect()
}

fn parse_nft_output(output: &str) -> Vec<(String, String)> {
    let mut tables = BTreeSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("table ") {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let keyword = parts.next();
        let family = parts.next();
        let name = parts.next();
        if keyword == Some("table") {
            if let (Some(family), Some(name)) = (family, name) {
                if name.starts_with("substrate_") {
                    tables.insert((family.to_string(), name.to_string()));
                }
            }
        }
    }
    tables.into_iter().collect()
}
