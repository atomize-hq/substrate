use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use substrate_common::paths;

pub struct SupervisorConfig {
    pub shim_dir: PathBuf,
    pub original_path: String,
    pub bash_env_file: Option<PathBuf>,
    pub target_command: Vec<String>,
    pub environment: HashMap<String, String>,
}

impl SupervisorConfig {
    pub fn new(target_command: Vec<String>) -> Result<Self> {
        let home = env::var("HOME").context("HOME environment variable not set")?;

        let shim_dir = paths::shims_dir()?;

        // Build clean original path from current PATH, removing any shim directories
        let original_path = build_default_path(&home)?;

        let bash_env_file = Some(PathBuf::from(&home).join(".substrate_bashenv"));

        Ok(Self {
            shim_dir,
            original_path,
            bash_env_file,
            target_command,
            environment: HashMap::new(),
        })
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    pub fn with_log_file<P: Into<PathBuf>>(self, log_file: P) -> Self {
        self.with_env(
            "SHIM_TRACE_LOG".to_string(),
            log_file.into().display().to_string(),
        )
    }
}

pub fn launch_supervised(config: SupervisorConfig) -> Result<()> {
    if config.target_command.is_empty() {
        return Err(anyhow!("No target command specified"));
    }

    // Prepare environment
    let mut cmd = Command::new(&config.target_command[0]);

    if config.target_command.len() > 1 {
        cmd.args(&config.target_command[1..]);
    }

    // Set up clean environment with session seeding
    let session_id =
        env::var("SHIM_SESSION_ID").unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());
    cmd.env("SHIM_SESSION_ID", &session_id);
    cmd.env("SHIM_BUILD", env!("CARGO_PKG_VERSION"));
    cmd.env("SHIM_ORIGINAL_PATH", &config.original_path);

    // Build shimmed PATH with deduplication
    let shimmed_path = format!("{}:{}", config.shim_dir.display(), config.original_path);
    cmd.env("PATH", dedupe_path(&shimmed_path));

    // Set BASH_ENV for non-interactive shells (Claude Code integration)
    // BASH_ENV will handle all PATH manipulation and shim setup
    if let Some(bash_env) = &config.bash_env_file {
        if bash_env.exists() {
            cmd.env("BASH_ENV", bash_env);
        }
    }

    // Apply additional environment variables
    for (key, value) in &config.environment {
        cmd.env(key, value);
    }

    // Inherit stdio for interactive use
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().context("Failed to spawn target command")?;

    let status = child.wait().context("Failed to wait for target command")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn build_default_path(home: &str) -> Result<String> {
    // Start with parent PATH if available, otherwise use common paths
    if let Ok(parent_path) = env::var("PATH") {
        let shim_dir = format!("{home}/.substrate/shims");
        Ok(strip_shim_dir_from_path(&parent_path, &shim_dir))
    } else {
        // Fallback to common paths for macOS/Linux development environments
        let paths = vec![
            format!("{}/.nvm/versions/node/v22.16.0/bin", home),
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
            "/usr/sbin".to_string(),
            "/sbin".to_string(),
            format!("{}/.bun/bin", home),
            format!("{}/.cargo/bin", home),
        ];
        Ok(paths.join(":"))
    }
}

fn strip_shim_dir_from_path(path: &str, shim_dir: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let shim_dir_normalized = shim_dir.trim_end_matches('/');

    // Helper to validate PATH entries
    fn is_good_dir(p: &str) -> bool {
        let pb = std::path::Path::new(p);
        pb.is_absolute() && pb.is_dir()
    }

    path.split(separator)
        .filter(|s| !s.is_empty())
        .filter(|p| p.trim_end_matches('/') != shim_dir_normalized)
        .filter(|p| is_good_dir(p)) // Validate paths
        .collect::<Vec<_>>()
        .join(&separator.to_string())
}

fn dedupe_path(path: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();

    for component in path.split(separator) {
        if !component.is_empty() {
            let canonical = component.trim_end_matches('/');
            if seen.insert(canonical.to_string()) {
                deduped.push(component);
            }
        }
    }

    deduped.join(&separator.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_shim_dir() {
        let path = "/usr/bin:/home/user/.substrate/shims:/bin";
        let shim_dir = "/home/user/.substrate/shims";
        let result = strip_shim_dir_from_path(path, shim_dir);
        assert_eq!(result, "/usr/bin:/bin");
    }

    #[test]
    fn test_supervisor_config_creation() {
        let config = SupervisorConfig::new(vec!["echo".to_string(), "test".to_string()]);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.target_command, vec!["echo", "test"]);
        assert!(config.shim_dir.ends_with("shims"));
    }
}
