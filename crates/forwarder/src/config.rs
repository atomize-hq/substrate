use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

const DEFAULT_UDS_PATH: &str = "/run/substrate.sock";
const DEFAULT_TCP_PORT: u16 = 61337;
const CONFIG_FILE_NAME: &str = "forwarder.toml";
const TARGET_ENV: &str = "SUBSTRATE_FORWARDER_TARGET";

#[derive(Clone, Debug)]
pub struct ForwarderConfig {
    pub distro: String,
    pub pipe_path: String,
    pub host_tcp_bridge: Option<SocketAddr>,
    target: BridgeTarget,
}

impl ForwarderConfig {
    pub fn load(
        distro: String,
        pipe_path: String,
        host_tcp_bridge: Option<SocketAddr>,
        config_path: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let file_settings = load_file_settings(config_path)?;
        let env_override = std::env::var(TARGET_ENV)
            .ok()
            .map(|value| parse_env_override(&value))
            .transpose()?;
        let target = resolve_target(
            file_settings.as_ref().and_then(|s| s.target.as_ref()),
            env_override,
        )?;

        Ok(Self {
            distro,
            pipe_path,
            host_tcp_bridge,
            target,
        })
    }

    pub fn target(&self) -> &BridgeTarget {
        &self.target
    }

    pub fn target_mode(&self) -> &'static str {
        self.target.mode()
    }
}

#[derive(Clone, Debug)]
pub enum BridgeTarget {
    Uds { path: String },
    Tcp { addr: SocketAddr },
}

impl BridgeTarget {
    pub fn mode(&self) -> &'static str {
        match self {
            Self::Uds { .. } => "uds",
            Self::Tcp { .. } => "tcp",
        }
    }
}

impl std::fmt::Display for BridgeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "{}", path),
            Self::Tcp { addr } => write!(f, "{}", addr),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileSettings {
    target: Option<FileTarget>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
struct FileTarget {
    mode: Option<TargetModeSetting>,
    tcp_port: Option<u16>,
    uds_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TargetModeSetting {
    Tcp,
    Uds,
}

impl std::str::FromStr for TargetModeSetting {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "tcp" => Ok(TargetModeSetting::Tcp),
            "uds" | "unix" | "unix_socket" => Ok(TargetModeSetting::Uds),
            other => Err(anyhow!("unsupported target mode: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
struct EnvOverride {
    mode: TargetModeSetting,
    value: Option<String>,
}

fn parse_env_override(raw: &str) -> anyhow::Result<EnvOverride> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{TARGET_ENV} is empty");
    }

    let mut parts = trimmed.splitn(2, ':');
    let mode_part = parts.next().unwrap();
    let mode: TargetModeSetting = mode_part.parse()?;
    let value = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(EnvOverride { mode, value })
}

fn resolve_target(
    file_target: Option<&FileTarget>,
    env_override: Option<EnvOverride>,
) -> anyhow::Result<BridgeTarget> {
    let file_target = file_target.cloned().unwrap_or_default();
    let mode = env_override
        .as_ref()
        .map(|o| o.mode)
        .or(file_target.mode)
        .unwrap_or(TargetModeSetting::Uds);

    match mode {
        TargetModeSetting::Uds => {
            let path = env_override
                .and_then(|o| o.value)
                .or(file_target.uds_path)
                .unwrap_or_else(|| DEFAULT_UDS_PATH.to_string());
            if path.is_empty() {
                anyhow::bail!("unix target path must not be empty");
            }
            Ok(BridgeTarget::Uds { path })
        }
        TargetModeSetting::Tcp => {
            let port = match env_override.and_then(|o| o.value) {
                Some(value) => value
                    .parse::<u16>()
                    .map_err(|err| anyhow!("invalid tcp port {value}: {err}"))?,
                None => file_target.tcp_port.unwrap_or(DEFAULT_TCP_PORT),
            };
            let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
            Ok(BridgeTarget::Tcp { addr })
        }
    }
}

fn load_file_settings(config_path: Option<PathBuf>) -> anyhow::Result<Option<FileSettings>> {
    let path = match config_path {
        Some(path) => path,
        None => match default_config_path() {
            Some(path) => path,
            None => return Ok(None),
        },
    };

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed reading forwarder config {}", path.display()))?;
    let settings: FileSettings = toml::from_str(&content)
        .with_context(|| format!("failed parsing forwarder config {}", path.display()))?;
    Ok(Some(settings))
}

fn default_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut path = PathBuf::from(base);
    path.push("Substrate");
    path.push(CONFIG_FILE_NAME);
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn temp_path(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("substrate-forwarder-test-{}", name));
        let _ = fs::create_dir_all(&dir);
        dir.push(CONFIG_FILE_NAME);
        dir
    }

    fn reset_env() {
        std::env::remove_var(TARGET_ENV);
    }

    #[test]
    fn default_target_is_uds_without_config() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let config = ForwarderConfig::load(
            "distro".to_string(),
            "\\\\.\\pipe\\substrate".to_string(),
            None,
            Some(PathBuf::from("does/not/exist.toml")),
        )
        .unwrap();
        assert_eq!(config.target_mode(), "uds");
        assert_eq!(config.target().to_string(), DEFAULT_UDS_PATH);
    }

    #[test]
    fn file_config_selects_tcp_mode_and_port() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let path = temp_path("tcp");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let toml = "[target]\nmode = \"tcp\"\ntcp_port = 60001\n";
        fs::write(&path, toml).unwrap();

        let config = ForwarderConfig::load(
            "distro".to_string(),
            "\\\\.\\pipe\\substrate".to_string(),
            None,
            Some(path.clone()),
        )
        .unwrap();

        assert_eq!(config.target_mode(), "tcp");
        assert_eq!(config.target().to_string(), "127.0.0.1:60001");
    }

    #[test]
    fn env_override_wins_over_file() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let path = temp_path("env");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let toml = "[target]\nmode = \"uds\"\nuds_path = \"/run/alternate.sock\"\n";
        fs::write(&path, toml).unwrap();

        std::env::set_var(TARGET_ENV, "tcp:60100");

        let config = ForwarderConfig::load(
            "distro".to_string(),
            "\\\\.\\pipe\\substrate".to_string(),
            None,
            Some(path.clone()),
        )
        .unwrap();

        assert_eq!(config.target_mode(), "tcp");
        assert_eq!(config.target().to_string(), "127.0.0.1:60100");

        std::env::remove_var(TARGET_ENV);
    }
}
