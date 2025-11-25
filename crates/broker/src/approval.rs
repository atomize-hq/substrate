use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct ApprovalCache {
    entries: HashMap<String, ApprovalEntry>,
}

#[derive(Debug, Clone)]
struct ApprovalEntry {
    status: ApprovalStatus,
    expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalStatus {
    Approved,
    Denied,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum ApprovalScope {
    Once,
    Session,
    Always,
}

#[derive(Debug, Clone)]
pub struct ApprovalContext {
    pub command: String,
    pub cwd: String,
    pub diff_preview: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ApprovalCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn check(&self, cmd: &str) -> ApprovalStatus {
        if let Some(entry) = self.entries.get(cmd) {
            // Check if approval has expired
            if let Some(expires) = entry.expires_at {
                if SystemTime::now() > expires {
                    return ApprovalStatus::Unknown;
                }
            }
            entry.status.clone()
        } else {
            // Check for pattern matches
            for (pattern, entry) in &self.entries {
                if pattern.contains('*') && matches_pattern(cmd, pattern) {
                    if let Some(expires) = entry.expires_at {
                        if SystemTime::now() > expires {
                            continue;
                        }
                    }
                    return entry.status.clone();
                }
            }
            ApprovalStatus::Unknown
        }
    }

    #[allow(private_interfaces)]
    pub fn add(&mut self, cmd: String, status: ApprovalStatus, scope: ApprovalScope) {
        let expires_at = match scope {
            ApprovalScope::Once => Some(SystemTime::now() + Duration::from_secs(0)), // Immediate expiry
            ApprovalScope::Session => Some(SystemTime::now() + Duration::from_secs(3600)), // 1 hour
            ApprovalScope::Always => None, // Never expires
        };

        self.entries
            .insert(cmd, ApprovalEntry { status, expires_at });
    }

    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.entries.retain(|_, entry| {
            if let Some(expires) = entry.expires_at {
                expires > now
            } else {
                true
            }
        });
    }
}

impl Default for ApprovalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalContext {
    pub fn new(command: &str, cwd: &str) -> Self {
        let risk_level = assess_risk_level(command);
        Self {
            command: command.to_string(),
            cwd: cwd.to_string(),
            diff_preview: None,
            risk_level,
        }
    }

    pub fn with_diff_preview(mut self, preview: String) -> Self {
        self.diff_preview = Some(preview);
        self
    }
}

pub fn request_interactive_approval(
    cmd: &str,
    context: &ApprovalContext,
    cache: &Arc<RwLock<ApprovalCache>>,
) -> Result<bool> {
    println!("\n{}", "─".repeat(60).dimmed());
    println!("{}", "Command Approval Request".yellow().bold());
    println!("{}", "─".repeat(60).dimmed());

    println!("{}: {}", "Command".bright_blue(), cmd.white());
    println!("{}: {}", "Directory".bright_blue(), context.cwd.white());

    // Show risk level with color coding
    let risk_display = match context.risk_level {
        RiskLevel::Low => "Low".green(),
        RiskLevel::Medium => "Medium".yellow(),
        RiskLevel::High => "High".bright_red(),
        RiskLevel::Critical => "CRITICAL".red().bold(),
    };
    println!("{}: {}", "Risk Level".bright_blue(), risk_display);

    // Show diff preview if available
    if let Some(preview) = &context.diff_preview {
        println!("\n{}", "Expected Impact:".bright_blue());
        for line in preview.lines().take(10) {
            println!("  {}", line.dimmed());
        }
        if preview.lines().count() > 10 {
            println!("  {}", "... (truncated)".dimmed().italic());
        }
    }

    println!("{}", "─".repeat(60).dimmed());

    let options = vec![
        "Allow once",
        "Allow for this session",
        "Allow always for this command",
        "Deny",
    ];

    let theme = ColorfulTheme::default();
    let selection = Select::with_theme(&theme)
        .with_prompt("Choose an action")
        .items(&options)
        .default(0)
        .interact()?;

    let mut cache = cache
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire approval cache lock: {}", e))?;

    match selection {
        0 => {
            // Allow once
            cache.add(
                cmd.to_string(),
                ApprovalStatus::Approved,
                ApprovalScope::Once,
            );
            println!("{}", "✓ Command approved for this execution only".green());
            Ok(true)
        }
        1 => {
            // Allow for session
            cache.add(
                cmd.to_string(),
                ApprovalStatus::Approved,
                ApprovalScope::Session,
            );
            println!("{}", "✓ Command approved for this session".green());
            Ok(true)
        }
        2 => {
            // Allow always
            cache.add(
                cmd.to_string(),
                ApprovalStatus::Approved,
                ApprovalScope::Always,
            );
            println!("{}", "✓ Command approved permanently".green());

            // Optionally save to policy file
            if let Ok(response) = dialoguer::Confirm::new()
                .with_prompt("Save this approval to policy file?")
                .default(false)
                .interact()
            {
                if response {
                    // Add the command to the policy's allowed list
                    if let Err(e) = add_command_to_policy(cmd) {
                        println!(
                            "{}",
                            format!("Warning: Failed to update policy file: {}", e).yellow()
                        );
                    } else {
                        println!("{}", "✓ Command added to policy file".green());
                    }
                }
            }

            Ok(true)
        }
        _ => {
            // Deny
            cache.add(
                cmd.to_string(),
                ApprovalStatus::Denied,
                ApprovalScope::Session,
            );
            println!("{}", "✗ Command denied".red());
            Ok(false)
        }
    }
}

/// Add a command pattern to the policy file's allowed list
fn add_command_to_policy(cmd: &str) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;

    // Look for policy file in standard locations
    let mut policy_paths = vec![
        Path::new(".substrate/policy.yaml").to_path_buf(),
        Path::new(".substrate-policy.yaml").to_path_buf(),
    ];

    if let Some(home) = dirs::home_dir() {
        policy_paths.push(home.join(".substrate/policy.yaml"));
    }

    for path in &policy_paths {
        if path.exists() {
            // Read existing policy
            let content = fs::read_to_string(path)?;
            let mut policy: crate::Policy = serde_yaml::from_str(&content)?;

            // Add command to allowed list if not already present
            let cmd_pattern = simplify_command_pattern(cmd);
            if !policy.cmd_allowed.contains(&cmd_pattern) {
                policy.cmd_allowed.push(cmd_pattern);

                // Write updated policy back
                let updated_content = serde_yaml::to_string(&policy)?;
                fs::write(path, updated_content)?;
            }

            return Ok(());
        }
    }

    // If no existing policy file, create a new one
    let policy_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".substrate");

    fs::create_dir_all(&policy_dir)?;
    let policy_path = policy_dir.join("policy.yaml");

    let mut policy = crate::Policy::default();
    policy.cmd_allowed.push(simplify_command_pattern(cmd));

    let content = serde_yaml::to_string(&policy)?;
    fs::write(policy_path, content)?;

    Ok(())
}

/// Simplify a command to a reusable pattern
fn simplify_command_pattern(cmd: &str) -> String {
    // Extract the base command without arguments for common cases
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return cmd.to_string();
    }

    // For common package managers, create specific patterns
    match parts[0] {
        "npm" | "yarn" | "pnpm" if parts.len() > 1 => {
            format!("{} {}*", parts[0], parts[1])
        }
        "pip" | "pip3" | "cargo" | "go" if parts.len() > 1 => {
            format!("{} {}*", parts[0], parts[1])
        }
        "git" if parts.len() > 1 => {
            format!("git {}*", parts[1])
        }
        _ => {
            // For other commands, just use the base command with wildcard
            format!("{}*", parts[0])
        }
    }
}

fn assess_risk_level(cmd: &str) -> RiskLevel {
    let cmd_lower = cmd.to_lowercase();

    // Critical risk patterns
    if cmd_lower.contains("rm -rf")
        || cmd_lower.contains("format")
        || cmd_lower.contains("dd if=")
        || cmd_lower.contains(":(){ :|:& };:")
    {
        // Fork bomb
        return RiskLevel::Critical;
    }

    // High risk patterns
    if cmd_lower.contains("sudo")
        || cmd_lower.contains("chmod 777")
        || cmd_lower.contains("| bash")
        || cmd_lower.contains("| sh")
        || cmd_lower.contains("eval")
        || cmd_lower.contains("exec")
    {
        return RiskLevel::High;
    }

    // Medium risk patterns
    if cmd_lower.contains("npm install")
        || cmd_lower.contains("pip install")
        || cmd_lower.contains("cargo install")
        || cmd_lower.contains("curl")
        || cmd_lower.contains("wget")
        || cmd_lower.contains("git clone")
    {
        return RiskLevel::Medium;
    }

    // Default to low risk
    RiskLevel::Low
}

fn matches_pattern(cmd: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            return glob_pattern.matches(cmd);
        }
    }
    cmd.contains(pattern)
}

#[cfg(test)]
mod tests;
