#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub const SUBSTRATE_LLM_GATEWAY_MODE: &str = "SUBSTRATE_LLM_GATEWAY_MODE";
pub const SUBSTRATE_LLM_GATEWAY_CONFIG_PATH: &str = "SUBSTRATE_LLM_GATEWAY_CONFIG_PATH";
pub const SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH: &str = "SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH";
pub const SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE: &str =
    "SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE";

pub const GATEWAY_MODE_IN_WORLD: &str = "in_world";
pub const GATEWAY_MODE_HOST_ONLY: &str = "host_only";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayMode {
    InWorld,
    HostOnly,
}

impl GatewayMode {
    pub fn as_env_value(self) -> &'static str {
        match self {
            Self::InWorld => GATEWAY_MODE_IN_WORLD,
            Self::HostOnly => GATEWAY_MODE_HOST_ONLY,
        }
    }

    pub fn from_env_or_default() -> Result<Self> {
        match env::var(SUBSTRATE_LLM_GATEWAY_MODE) {
            Ok(value) => Self::parse(value.trim()),
            Err(env::VarError::NotPresent) => Ok(Self::HostOnly),
            Err(err) => Err(anyhow!(
                "Failed to read {}: {}",
                SUBSTRATE_LLM_GATEWAY_MODE,
                err
            )),
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            GATEWAY_MODE_IN_WORLD => Ok(Self::InWorld),
            GATEWAY_MODE_HOST_ONLY => Ok(Self::HostOnly),
            other => Err(anyhow!(
                "Invalid {} value '{}'; expected '{}' or '{}'",
                SUBSTRATE_LLM_GATEWAY_MODE,
                other,
                GATEWAY_MODE_IN_WORLD,
                GATEWAY_MODE_HOST_ONLY
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStoreStrategy {
    Persistent(PathBuf),
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayLaunchContract {
    pub config_path: PathBuf,
    pub mode: GatewayMode,
    pub token_store: TokenStoreStrategy,
}

impl GatewayLaunchContract {
    pub fn integrated(config_path: PathBuf, token_store: TokenStoreStrategy) -> Self {
        Self {
            config_path,
            mode: GatewayMode::InWorld,
            token_store,
        }
    }

    pub fn standalone_local(config_path: PathBuf) -> Self {
        Self {
            config_path,
            mode: GatewayMode::HostOnly,
            token_store: TokenStoreStrategy::Persistent(
                default_token_store_path()
                    .unwrap_or_else(|_| PathBuf::from(".substrate-gateway/oauth_tokens.json")),
            ),
        }
    }

    pub fn apply_to_command(&self, command: &mut Command) {
        command.env(
            SUBSTRATE_LLM_GATEWAY_CONFIG_PATH,
            self.config_path.as_os_str(),
        );
        command.env(SUBSTRATE_LLM_GATEWAY_MODE, self.mode.as_env_value());

        match &self.token_store {
            TokenStoreStrategy::Persistent(path) => {
                command.env(SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH, path.as_os_str());
                command.env(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE, "0");
            }
            TokenStoreStrategy::Disabled => {
                command.env(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE, "1");
                command.env_remove(SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH);
            }
        }
    }

    pub fn resolve(
        cli_config_path: Option<PathBuf>,
        default_config_path: impl FnOnce() -> Result<PathBuf>,
        default_token_store_path: impl FnOnce() -> Result<PathBuf>,
    ) -> Result<Self> {
        let mode = GatewayMode::from_env_or_default()?;
        let config_path = resolve_config_path(cli_config_path, mode, default_config_path)?;
        let token_store = resolve_token_store(mode, default_token_store_path)?;
        Ok(Self {
            config_path,
            mode,
            token_store,
        })
    }
}

fn resolve_config_path(
    cli_config_path: Option<PathBuf>,
    mode: GatewayMode,
    default_config_path: impl FnOnce() -> Result<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = cli_config_path {
        return Ok(path);
    }

    if let Some(path) = read_path_env(SUBSTRATE_LLM_GATEWAY_CONFIG_PATH)? {
        return Ok(path);
    }

    match mode {
        GatewayMode::HostOnly => default_config_path(),
        GatewayMode::InWorld => Err(anyhow!(
            "Integrated gateway launch requires an explicit config path via --config or {}",
            SUBSTRATE_LLM_GATEWAY_CONFIG_PATH
        )),
    }
}

fn resolve_token_store(
    mode: GatewayMode,
    default_token_store_path: impl FnOnce() -> Result<PathBuf>,
) -> Result<TokenStoreStrategy> {
    let disable_persistence =
        read_bool_env(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE)?.unwrap_or(false);
    let token_store_path = read_path_env(SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH)?;

    if disable_persistence && token_store_path.is_some() {
        return Err(anyhow!(
            "{} cannot be combined with {}",
            SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE,
            SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH
        ));
    }

    if disable_persistence {
        return Ok(TokenStoreStrategy::Disabled);
    }

    if let Some(path) = token_store_path {
        return Ok(TokenStoreStrategy::Persistent(path));
    }

    match mode {
        GatewayMode::HostOnly => default_token_store_path().map(TokenStoreStrategy::Persistent),
        GatewayMode::InWorld => Err(anyhow!(
            "Integrated gateway launch requires {} or {}=1",
            SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH,
            SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE
        )),
    }
}

fn read_path_env(key: &str) -> Result<Option<PathBuf>> {
    match env::var_os(key) {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.as_os_str().is_empty() {
                Ok(None)
            } else {
                Ok(Some(path))
            }
        }
        None => Ok(None),
    }
}

fn read_bool_env(key: &str) -> Result<Option<bool>> {
    match env::var(key) {
        Ok(raw) => {
            let value = raw.trim().to_ascii_lowercase();
            let parsed = match value.as_str() {
                "" => None,
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => {
                    return Err(anyhow!(
                        "Invalid {} value '{}'; expected true/false",
                        key,
                        raw
                    ));
                }
            };
            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(err).with_context(|| format!("Failed to read {}", key)),
    }
}

fn default_token_store_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let config_dir = home.join(".substrate-gateway");
    std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    Ok(config_dir.join("oauth_tokens.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    #[test]
    fn integrated_mode_requires_explicit_config_path() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let _mode = EnvGuard::set(SUBSTRATE_LLM_GATEWAY_MODE, GATEWAY_MODE_IN_WORLD);
        let _config = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_CONFIG_PATH);
        let _token_path = EnvGuard::set(
            SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH,
            "/tmp/oauth_tokens.json",
        );
        let _disable = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE);

        let err = GatewayLaunchContract::resolve(
            None,
            || Ok(PathBuf::from("default.toml")),
            || Ok(PathBuf::from("default_tokens.json")),
        )
        .unwrap_err();

        assert!(err.to_string().contains("requires an explicit config path"));
    }

    #[test]
    fn integrated_mode_requires_explicit_token_store_or_disable() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("gateway.toml");

        let _mode = EnvGuard::set(SUBSTRATE_LLM_GATEWAY_MODE, GATEWAY_MODE_IN_WORLD);
        let _config = EnvGuard::set(
            SUBSTRATE_LLM_GATEWAY_CONFIG_PATH,
            config_path.to_str().unwrap(),
        );
        let _token_path = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH);
        let _disable = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE);

        let err = GatewayLaunchContract::resolve(
            None,
            || Ok(PathBuf::from("unused")),
            || Ok(PathBuf::from("unused")),
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("requires SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH"));
    }

    #[test]
    fn host_only_mode_keeps_local_defaults() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let _mode = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_MODE);
        let _config = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_CONFIG_PATH);
        let _token_path = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH);
        let _disable = EnvGuard::clear(SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE);

        let launch = GatewayLaunchContract::resolve(
            None,
            || Ok(PathBuf::from("/tmp/default-config.toml")),
            || Ok(PathBuf::from("/tmp/default-oauth-tokens.json")),
        )
        .unwrap();

        assert_eq!(launch.mode, GatewayMode::HostOnly);
        assert_eq!(
            launch.config_path,
            PathBuf::from("/tmp/default-config.toml")
        );
        assert_eq!(
            launch.token_store,
            TokenStoreStrategy::Persistent(PathBuf::from("/tmp/default-oauth-tokens.json"))
        );
    }

    #[test]
    fn launch_contract_applies_integrated_env_handoff() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("gateway.toml");
        let token_store_path = temp_dir.path().join("oauth_tokens.json");

        let launch = GatewayLaunchContract::integrated(
            config_path.clone(),
            TokenStoreStrategy::Persistent(token_store_path.clone()),
        );
        let mut command = Command::new("env");
        launch.apply_to_command(&mut command);

        let envs: Vec<(String, String)> = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.unwrap().to_string_lossy().into_owned(),
                )
            })
            .collect();

        assert!(envs.iter().any(|(key, value)| {
            key == SUBSTRATE_LLM_GATEWAY_MODE && value == GATEWAY_MODE_IN_WORLD
        }));
        assert!(envs.iter().any(|(key, value)| {
            key == SUBSTRATE_LLM_GATEWAY_CONFIG_PATH
                && value == config_path.to_string_lossy().as_ref()
        }));
        assert!(envs.iter().any(|(key, value)| {
            key == SUBSTRATE_LLM_GATEWAY_TOKEN_STORE_PATH
                && value == token_store_path.to_string_lossy().as_ref()
        }));
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }

        fn clear(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
        }
    }
}
