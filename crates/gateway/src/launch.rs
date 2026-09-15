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

/// One process-local adoption of the four E3 launch descriptors.
#[cfg(target_os = "linux")]
pub(crate) struct E3GatewayLaunchContractV1 {
    launch_reader: Option<std::fs::File>,
    listener: Option<std::os::fd::OwnedFd>,
    ready_writer: Option<std::fs::File>,
    pub(crate) auth_reader: Option<std::fs::File>,
    pub(crate) input: Option<config_projection::ManagedGatewayLaunchInputV1>,
}

#[cfg(target_os = "linux")]
impl E3GatewayLaunchContractV1 {
    pub(crate) fn from_environment() -> Result<Option<Self>> {
        use nix::libc;
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        let pointers = [
            "SUBSTRATE_E3_GATEWAY_LAUNCH_FD",
            "SUBSTRATE_E3_GATEWAY_LISTENER_FD",
            "SUBSTRATE_E3_GATEWAY_SECRET_READY_FD",
            "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
        ];
        if !pointers[..3].iter().any(|name| env::var_os(name).is_some()) {
            return Ok(None);
        }
        let values = pointers.map(|name| {
            let value = env::var_os(name);
            env::remove_var(name);
            value
        });
        let mut descriptors = Vec::with_capacity(4);
        let mut seen = std::collections::BTreeSet::new();
        for (index, value) in values.into_iter().enumerate() {
            let value = value.context("E3 gateway launch descriptor is absent")?;
            let value = value
                .to_str()
                .context("E3 gateway launch descriptor is malformed")?;
            let fd: i32 = value
                .parse()
                .context("E3 gateway launch descriptor is malformed")?;
            anyhow::ensure!(
                fd >= 3 && fd.to_string() == value && seen.insert(fd),
                "E3 gateway launch descriptor is aliased or malformed"
            );
            let mut stat: libc::stat = unsafe { std::mem::zeroed() };
            anyhow::ensure!(
                unsafe { libc::fstat(fd, &mut stat) } == 0,
                "E3 gateway launch descriptor is unavailable"
            );
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            anyhow::ensure!(
                flags >= 0,
                "E3 gateway launch descriptor flags are unavailable"
            );
            // SAFETY: fstat/F_GETFL verified this unique inherited descriptor is open.
            let owned = unsafe { OwnedFd::from_raw_fd(fd) };
            if index == 1 {
                anyhow::ensure!(
                    stat.st_mode & libc::S_IFMT == libc::S_IFSOCK,
                    "E3 gateway listener is not a socket"
                );
            } else {
                anyhow::ensure!(
                    stat.st_mode & libc::S_IFMT == libc::S_IFIFO
                        && stat.st_uid == unsafe { libc::geteuid() }
                        && flags & libc::O_ACCMODE
                            == if index == 2 {
                                libc::O_WRONLY
                            } else {
                                libc::O_RDONLY
                            },
                    "E3 gateway pipe identity is invalid"
                );
            }
            anyhow::ensure!(
                unsafe { libc::fcntl(owned.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } == 0,
                "E3 gateway descriptor could not be closed to child inheritance"
            );
            descriptors.push(owned);
        }
        // Runtime-created descriptors are CLOEXEC; no unrelated inherited FD is admitted.
        for entry in std::fs::read_dir("/proc/self/fd")? {
            let entry = entry?;
            let Some(fd) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<i32>().ok())
            else {
                continue;
            };
            if fd > 2 && !seen.contains(&fd) {
                let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
                anyhow::ensure!(
                    flags == -1 || flags & libc::FD_CLOEXEC != 0,
                    "E3 gateway has an extra inherited descriptor"
                );
            }
        }
        let mut descriptors = descriptors.into_iter();
        Ok(Some(Self {
            launch_reader: Some(std::fs::File::from(
                descriptors.next().context("E3 launch reader absent")?,
            )),
            listener: descriptors.next(),
            ready_writer: Some(std::fs::File::from(
                descriptors.next().context("E3 ready writer absent")?,
            )),
            auth_reader: Some(std::fs::File::from(
                descriptors.next().context("E3 auth reader absent")?,
            )),
            input: None,
        }))
    }

    pub(crate) fn consume_launch_input(&mut self, configured_path: &std::path::Path) -> Result<()> {
        use config_projection::{ConfigProjectionCodecV1 as Codec, ManagedGatewayLaunchInputV1};
        use nix::libc;
        use sha2::{Digest, Sha256};
        use std::io::Read;
        use std::os::fd::{AsFd, AsRawFd, FromRawFd};
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let mut bytes = Vec::new();
        self.launch_reader
            .take()
            .context("E3 launch input was already consumed")?
            .take(65_537)
            .read_to_end(&mut bytes)?;
        anyhow::ensure!(bytes.len() <= 65_536, "E3 launch input is too large");
        let input: ManagedGatewayLaunchInputV1 = Codec::decode_canonical_json(&bytes)?;
        let mut preimage = serde_json::to_value(&input)?;
        preimage
            .as_object_mut()
            .context("E3 launch input is malformed")?
            .remove("launch_input_hash");
        anyhow::ensure!(
            input.launch_input_hash
                == Codec::domain_sha256(
                    "substrate.e3.managed-gateway-launch-input.v1",
                    &serde_json::json!({"launch_input":preimage})
                )?,
            "E3 launch input hash mismatch"
        );
        let expected_root = input.gateway_config.root.physical_path.clone();
        let prefix = format!(
            "/run/substrate/e3-gateway/{}/",
            input.dormant_projection_ref.series_id
        );
        let fence = expected_root
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_prefix("cpf_"))
            .context("E3 gateway root has the wrong series or fence")?;
        let fence_uuid = uuid::Uuid::parse_str(fence).context("E3 gateway fence is malformed")?;
        anyhow::ensure!(
            fence_uuid.get_version_num() == 7 && fence_uuid.to_string() == fence,
            "E3 gateway fence is malformed"
        );
        anyhow::ensure!(
            input.schema_version == 1
                && input.backend_id == "cli:codex-world"
                && input.gateway_config.root.physical_path == expected_root
                && input.gateway_config.relative_path == "config.toml"
                && configured_path == std::path::Path::new(&expected_root).join("config.toml")
                && input.http_surface.inherited_listener_only
                && input.http_surface.auxiliary_listener_count == 0
                && input.http_surface.readiness_method == "GET"
                && input.http_surface.readiness_path == "/health"
                && input.http_surface.member_method == "POST"
                && input.http_surface.member_path == "/v1/responses"
                && input.gateway_ref.authority_store_id == input.authority_store_id
                && input.dormant_projection_ref.authority_store_id == input.authority_store_id
                && input.activation_intent_ref.authority_store_id == input.authority_store_id
                && input.access_boundary_ref.authority_store_id == input.authority_store_id
                && input.secret_handoff_prepared_ref.authority_store_id == input.authority_store_id,
            "E3 launch input bindings are invalid"
        );
        input
            .gateway_ref
            .validate()
            .map_err(|_| anyhow!("E3 gateway reference is invalid"))?;
        input
            .dormant_projection_ref
            .validate()
            .map_err(|_| anyhow!("E3 dormant reference is invalid"))?;
        input
            .activation_intent_ref
            .validate()
            .map_err(|_| anyhow!("E3 intent reference is invalid"))?;
        let allowed = [
            "HOME",
            "LANG",
            "LC_ALL",
            "PATH",
            "RUST_LOG",
            SUBSTRATE_LLM_GATEWAY_MODE,
            SUBSTRATE_LLM_GATEWAY_CONFIG_PATH,
            SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE,
        ];
        anyhow::ensure!(
            env::vars_os()
                .all(|(name, _)| name.to_str().is_some_and(|name| allowed.contains(&name))),
            "E3 gateway has an ambient environment value"
        );
        for (name,expected) in [("HOME",expected_root.as_str()),("LANG","C.UTF-8"),("LC_ALL","C.UTF-8"),
            ("PATH","/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"),
            ("RUST_LOG","error"),(SUBSTRATE_LLM_GATEWAY_MODE,"in_world"),
            (SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE,"1")] {
            anyhow::ensure!(env::var(name).ok().as_deref()==Some(expected),"E3 gateway environment binding mismatch");
        }
        anyhow::ensure!(
            env::var_os(SUBSTRATE_LLM_GATEWAY_CONFIG_PATH).as_deref()
                == Some(configured_path.as_os_str()),
            "E3 gateway config environment binding mismatch"
        );
        let root = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&expected_root)?;
        let metadata = root.metadata()?;
        let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
            input.gateway_config.root.physical_identity;
        anyhow::ensure!(
            metadata.dev() == device_id
                && metadata.ino() == inode
                && metadata.uid() == unsafe { libc::geteuid() }
                && metadata.gid() == unsafe { libc::getegid() }
                && metadata.mode() & 0o7777 == 0o700,
            "E3 gateway config root changed"
        );
        input
            .gateway_config
            .root
            .revalidate_linux_from_fd(root.as_fd())?;
        let config_fd = unsafe {
            libc::openat(
                root.as_raw_fd(),
                c"config.toml".as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            )
        };
        anyhow::ensure!(
            config_fd >= 0,
            "E3 gateway config descriptor is unavailable"
        );
        // SAFETY: openat returned an independently owned descriptor beneath the held root.
        let mut config = unsafe { std::fs::File::from_raw_fd(config_fd) };
        let metadata = config.metadata()?;
        anyhow::ensure!(
            metadata.is_file()
                && metadata.nlink() == 1
                && metadata.mode() & 0o7777 == 0o600
                && metadata.uid() == unsafe { libc::geteuid() }
                && metadata.gid() == unsafe { libc::getegid() }
                && metadata.len() == input.gateway_config.byte_length
                && metadata.len() <= 65_536,
            "E3 gateway config identity changed"
        );
        let mut config_bytes = Vec::new();
        config.read_to_end(&mut config_bytes)?;
        anyhow::ensure!(
            format!("{:x}", Sha256::digest(&config_bytes)) == input.gateway_config.sha256,
            "E3 gateway config bytes changed"
        );
        input
            .gateway_config
            .root
            .revalidate_linux_from_fd(root.as_fd())?;
        self.input = Some(input);
        Ok(())
    }

    pub(crate) fn adopt_listener(&mut self) -> Result<tokio::net::TcpListener> {
        use nix::libc;
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::MetadataExt;
        let input = self
            .input
            .as_ref()
            .context("E3 launch input has not been consumed")?;
        let listener = self
            .listener
            .take()
            .context("E3 listener was already adopted")?;
        let fd = listener.as_raw_fd();
        for (option, expected) in [
            (libc::SO_DOMAIN, libc::AF_INET),
            (libc::SO_TYPE, libc::SOCK_STREAM),
            (libc::SO_ACCEPTCONN, 1),
        ] {
            let mut observed: i32 = 0;
            let mut length = std::mem::size_of_val(&observed) as libc::socklen_t;
            anyhow::ensure!(
                unsafe {
                    libc::getsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        option,
                        (&mut observed as *mut i32).cast(),
                        &mut length,
                    )
                } == 0
                    && length as usize == std::mem::size_of_val(&observed)
                    && observed == expected,
                "E3 listener socket state mismatch"
            );
        }
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        anyhow::ensure!(
            unsafe { libc::fstat(fd, &mut stat) } == 0
                && stat.st_ino == input.listener_identity.socket_inode
                && std::fs::metadata("/proc/self/ns/net")?.ino()
                    == input.listener_identity.network_namespace_inode
                && input.listener_identity.transport == "tcp"
                && input.listener_identity.address == "127.0.0.1"
                && input.listener_identity.listen_backlog == 16
                && input
                    .listener_identity
                    .deny_boundary_effective_before_listen
                && input.listener_identity.responses_base_path == "/v1",
            "E3 listener identity mismatch"
        );
        let listener = std::net::TcpListener::from(listener);
        let expected = std::net::SocketAddr::from(([127, 0, 0, 1], input.listener_identity.port));
        anyhow::ensure!(
            listener.local_addr()? == expected && input.listener_identity.port != 0,
            "E3 listener address mismatch"
        );
        listener.set_nonblocking(true)?;
        Ok(tokio::net::TcpListener::from_std(listener)?)
    }

    pub(crate) fn lock_and_attest_secret_ready(&mut self) -> Result<String> {
        use config_projection::{
            ConfigProjectionCodecV1 as Codec, E3GatewaySecretReadyAttestationV1,
        };
        use nix::libc;
        use std::io::Write;
        use std::os::unix::fs::MetadataExt;
        let input = self
            .input
            .as_ref()
            .context("E3 launch input has not been consumed")?;
        anyhow::ensure!(self.listener.is_none(), "E3 listener has not been adopted");
        let limits = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        anyhow::ensure!(
            unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limits) } == 0
                && unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) } == 0,
            "E3 gateway secret memory hardening failed"
        );
        let mut observed = libc::rlimit {
            rlim_cur: 1,
            rlim_max: 1,
        };
        anyhow::ensure!(
            unsafe { libc::getrlimit(libc::RLIMIT_CORE, &mut observed) } == 0
                && observed.rlim_cur == 0
                && observed.rlim_max == 0
                && unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) } == 0,
            "E3 gateway secret memory readback failed"
        );
        let status = std::fs::read_to_string("/proc/self/status")?;
        anyhow::ensure!(
            status
                .lines()
                .find_map(|line| line.strip_prefix("TracerPid:"))
                .is_some_and(|s| s.trim() == "0"),
            "E3 gateway is traced"
        );
        let stat = std::fs::read_to_string("/proc/self/stat")?;
        let (_, fields) = stat
            .rsplit_once(") ")
            .context("E3 gateway process identity is malformed")?;
        let start: u64 = fields
            .split_whitespace()
            .nth(19)
            .context("E3 gateway process start is absent")?
            .parse()?;
        let namespace = std::fs::metadata("/proc/self/ns/user")?;
        let mut attestation = E3GatewaySecretReadyAttestationV1 {
            schema_version: 1,
            gateway_instance_id: input.gateway_ref.gateway_instance_id.clone(),
            launch_input_hash: input.launch_input_hash.clone(),
            gateway_pid: std::process::id(),
            gateway_pid_start_time_ticks: start,
            user_namespace_device_id: namespace.dev(),
            user_namespace_inode: namespace.ino(),
            dumpable: 0,
            rlimit_core_soft: 0,
            rlimit_core_hard: 0,
            tracer_pid: 0,
            attestation_hash: String::new(),
        };
        let mut value = serde_json::to_value(&attestation)?;
        value
            .as_object_mut()
            .context("E3 secret-ready attestation is malformed")?
            .remove("attestation_hash");
        attestation.attestation_hash = Codec::domain_sha256(
            "substrate.e3.gateway-secret-ready-attestation.v1",
            &serde_json::json!({"attestation":value}),
        )?;
        let bytes = Codec::encode_canonical_json(&attestation)?;
        anyhow::ensure!(
            bytes.len() <= 4096,
            "E3 secret-ready attestation exceeds its bound"
        );
        self.ready_writer
            .take()
            .context("E3 secret-ready attestation was already sent")?
            .write_all(&bytes)?;
        Ok(attestation.attestation_hash)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod e3_descriptor_tests {
    use super::E3GatewayLaunchContractV1;
    use nix::libc;
    use std::io::Write;
    use std::os::fd::{AsRawFd, FromRawFd};

    fn pipe() -> (std::fs::File, std::fs::File) {
        let mut fds = [-1; 2];
        assert_eq!(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
        unsafe {
            (
                std::fs::File::from_raw_fd(fds[0]),
                std::fs::File::from_raw_fd(fds[1]),
            )
        }
    }

    #[test]
    fn e3_descriptor_environment_rejects_invalid_aliases_before_auth_read() {
        let case = std::env::var("E3_DESCRIPTOR_TEST_CASE").ok();
        let Some(case) = case else {
            for case in [
                "absent",
                "missing",
                "nondecimal",
                "closed",
                "alias",
                "noncanonical",
            ] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "launch::e3_descriptor_tests::e3_descriptor_environment_rejects_invalid_aliases_before_auth_read", "--test-threads=1"])
                    .env_clear().env("E3_DESCRIPTOR_TEST_CASE",case).output().unwrap();
                assert!(
                    output.status.success(),
                    "case {case}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            }
            return;
        };
        if case == "absent" {
            std::env::set_var("SUBSTRATE_LLM_AUTH_BUNDLE_FD", "compatibility-pointer");
            assert!(E3GatewayLaunchContractV1::from_environment()
                .unwrap()
                .is_none());
            assert_eq!(
                std::env::var("SUBSTRATE_LLM_AUTH_BUNDLE_FD").unwrap(),
                "compatibility-pointer"
            );
            return;
        }
        if matches!(case.as_str(), "missing" | "nondecimal" | "closed") {
            std::env::set_var(
                "SUBSTRATE_E3_GATEWAY_LAUNCH_FD",
                match case.as_str() {
                    "missing" => "0",
                    "nondecimal" => "03",
                    _ => "2000000000",
                },
            );
            assert!(E3GatewayLaunchContractV1::from_environment().is_err());
            assert!(std::env::var_os("SUBSTRATE_E3_GATEWAY_LAUNCH_FD").is_none());
            return;
        }
        use std::os::fd::IntoRawFd;
        let (launch_read, mut launch_write) = pipe();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let (_ready_read, ready_write) = pipe();
        let (auth_read, mut auth_write) = pipe();
        let canary = b"synthetic-e3-descriptor-unread-canary";
        auth_write.write_all(canary).unwrap();
        launch_write.write_all(b"{}\n").unwrap();
        drop(launch_write);
        let launch_fd = launch_read.into_raw_fd();
        let listener_fd = listener.into_raw_fd();
        let ready_fd = ready_write.into_raw_fd();
        let auth_fd = auth_read.into_raw_fd();
        for (name, fd) in [
            ("SUBSTRATE_E3_GATEWAY_LAUNCH_FD", launch_fd),
            ("SUBSTRATE_E3_GATEWAY_LISTENER_FD", listener_fd),
            ("SUBSTRATE_E3_GATEWAY_SECRET_READY_FD", ready_fd),
            (
                "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
                if case == "alias" { launch_fd } else { auth_fd },
            ),
        ] {
            std::env::set_var(name, fd.to_string());
        }
        if case == "alias" {
            assert!(E3GatewayLaunchContractV1::from_environment().is_err());
            assert_eq!(unsafe { libc::fcntl(launch_fd, libc::F_GETFD) }, -1);
        } else {
            let mut contract = E3GatewayLaunchContractV1::from_environment()
                .unwrap()
                .unwrap();
            assert!(contract
                .consume_launch_input(std::path::Path::new("/invalid"))
                .is_err());
            let mut unread: libc::c_int = 0;
            assert_eq!(
                unsafe {
                    libc::ioctl(
                        contract.auth_reader.as_ref().unwrap().as_raw_fd(),
                        libc::FIONREAD,
                        &mut unread,
                    )
                },
                0
            );
            assert_eq!(unread as usize, canary.len());
            drop(contract);
            assert_eq!(unsafe { libc::fcntl(auth_fd, libc::F_GETFD) }, -1);
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod e3_readiness_tests {
    use super::E3GatewayLaunchContractV1;
    use config_projection::*;
    use nix::libc;
    use std::io::{Read, Write};
    use std::os::fd::{AsFd, FromRawFd};
    use std::os::unix::fs::MetadataExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn pipe() -> (std::fs::File, std::fs::File) {
        let mut fds = [-1; 2];
        assert_eq!(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
        unsafe {
            (
                std::fs::File::from_raw_fd(fds[0]),
                std::fs::File::from_raw_fd(fds[1]),
            )
        }
    }

    fn input(listener: &std::net::TcpListener) -> ManagedGatewayLaunchInputV1 {
        let uuid = "019f3946-ec00-7000-8000-000000000001";
        let store = format!("cpa_{uuid}");
        let root = std::fs::File::open("/proc/self").unwrap();
        let directory = CanonicalDirectoryV1::capture_linux_from_fd(root.as_fd()).unwrap();
        let mut value:ManagedGatewayLaunchInputV1=serde_json::from_value(serde_json::json!({
            "schema_version":1,"authority_store_id":store,"launch_input_id":format!("gli_{uuid}"),
            "activation_intent_ref":{"authority_store_id":store,"activation_intent_id":format!("gai_{uuid}"),"intent_hash":"11".repeat(32)},
            "dormant_projection_ref":{"authority_store_id":store,"series_id":format!("cps_{uuid}"),"record_id":format!("cpr_{uuid}"),"revision":1,"record_hash":"22".repeat(32)},
            "gateway_ref":{"authority_store_id":store,"gateway_instance_id":format!("cgi_{uuid}"),"gateway_identity_hash":"33".repeat(32)},
            "config_projection_identity_hash":"44".repeat(32),"orchestration_session_id":"e3-test-session","retained_participant_id":"e3-test-member",
            "backend_id":"cli:codex-world","world_id":"e3-test-world","world_generation":1,
            "listener_identity":{"transport":"tcp","network_namespace_inode":std::fs::metadata("/proc/self/ns/net").unwrap().ino(),
                "address":"127.0.0.1","port":listener.local_addr().unwrap().port(),"socket_inode":std::fs::metadata(format!("/proc/self/fd/{}",std::os::fd::AsRawFd::as_raw_fd(listener))).unwrap().ino(),
                "listen_backlog":16,"deny_boundary_effective_before_listen":true,"responses_base_path":"/v1"},
            "gateway_config":{"root":directory,"relative_path":"config.toml","mode":384,"byte_length":1,"sha256":"55".repeat(32)},
            "http_surface":{"inherited_listener_only":true,"readiness_method":"GET","readiness_path":"/health","member_method":"POST","member_path":"/v1/responses","auxiliary_listener_count":0},
            "access_boundary_ref":{"authority_store_id":store,"access_boundary_id":"test-boundary","revision":1,"boundary_hash":"66".repeat(32)},
            "secret_handoff_prepared_ref":{"authority_store_id":store,"handoff_id":"test-handoff","orchestration_session_id":"e3-test-session","retained_participant_id":"e3-test-member","runtime_family":"codex",
                "world_id":"e3-test-world","world_generation":1,"receiving_gateway_identity_hash":"33".repeat(32),"handoff_state_revision":1,"handoff_hash":"77".repeat(32)},
            "readiness_nonce":uuid,"launch_input_hash":""
        })).unwrap();
        let mut preimage = serde_json::to_value(&value).unwrap();
        preimage
            .as_object_mut()
            .unwrap()
            .remove("launch_input_hash");
        value.launch_input_hash = ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.managed-gateway-launch-input.v1",
            &serde_json::json!({"launch_input":preimage}),
        )
        .unwrap();
        value
    }

    #[test]
    fn e3_readiness_exact_wire_and_member_header_gate() {
        if std::env::var_os("E3_READINESS_TEST_CHILD").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "launch::e3_readiness_tests::e3_readiness_exact_wire_and_member_header_gate",
                    "--test-threads=1",
                ])
                .env("E3_READINESS_TEST_CHILD", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }
        // Hardening changes are confined to this short-lived test process.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let input = input(&listener);
        let expected = input.clone();
        let port = listener.local_addr().unwrap().port();
        let (mut ready_read, ready_write) = pipe();
        let (auth_read, mut auth_write) = pipe();
        let canary = "synthetic-e3-one-time-readiness-canary";
        let (attest_send, attest_recv) = std::sync::mpsc::channel();
        let writer = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            ready_read.read_to_end(&mut bytes).unwrap();
            let attestation: E3GatewaySecretReadyAttestationV1 =
                ConfigProjectionCodecV1::decode_canonical_json(&bytes).unwrap();
            assert_eq!(attestation.dumpable, 0);
            assert_eq!(attestation.rlimit_core_soft, 0);
            assert_eq!(attestation.rlimit_core_hard, 0);
            let bundle = substrate_common::GatewayAuthBundleV1 {
                schema_version: 1,
                backend_id: "cli:codex".into(),
                fields: std::collections::HashMap::from([(
                    substrate_common::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.into(),
                    canary.into(),
                )]),
            };
            auth_write
                .write_all(&serde_json::to_vec(&bundle).unwrap())
                .unwrap();
            drop(auth_write);
            attest_send.send(attestation.attestation_hash).unwrap();
        });
        let launch = E3GatewayLaunchContractV1 {
            launch_reader: None,
            listener: Some(listener.into()),
            ready_writer: Some(ready_write),
            auth_reader: Some(auth_read),
            input: Some(input),
        };
        let config: crate::cli::AppConfig = toml::from_str(&format!(
            r#"
[server]
host = "127.0.0.1"
port = {port}
log_level = "info"
[router]
default = "codex"
[[providers]]
name = "openai-codex"
provider_type = "openai"
auth_type = "oauth"
oauth_provider = "openai-codex"
models = ["codex-mini-latest"]
enabled = true
[[models]]
name = "codex"
[[models.mappings]]
priority = 1
provider = "openai-codex"
actual_model = "codex-mini-latest"
"#
        ))
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            let serving=tokio::spawn(crate::server::serve_e3_inherited_listener(config,launch));
            let mut probe=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
            let request=format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Substrate-E3-Readiness-Nonce: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",expected.readiness_nonce);
            probe.write_all(request.as_bytes()).await.unwrap();
            let mut response=Vec::new();
            tokio::time::timeout(std::time::Duration::from_secs(5),probe.read_to_end(&mut response)).await.unwrap().unwrap();
            let split=response.windows(4).position(|w|w==b"\r\n\r\n").unwrap()+4;
            let body=&response[split..];
            assert_eq!(&response[..split],format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).as_bytes());
            let ready:serde_json::Value=ConfigProjectionCodecV1::decode_canonical_json(body).unwrap();
            assert_eq!(ready["launch_input_hash"],expected.launch_input_hash);
            assert_eq!(ready["secret_ready_attestation_hash"],attest_recv.recv().unwrap());
            assert_eq!(ready["secret_handoff_consumed"],true);
            assert!(!response.windows(canary.len()).any(|w|w==canary.as_bytes()));
            let client=reqwest::Client::new();
            for path in ["/v1/messages","/v1/chat/completions","/api/oauth/tokens","/auth/callback","/unknown"] {
                assert_eq!(client.get(format!("http://127.0.0.1:{port}{path}")).send().await.unwrap().status(),404);
            }
            assert_eq!(client.head(format!("http://127.0.0.1:{port}/health")).send().await.unwrap().status(),405);
            assert_eq!(client.get(format!("http://127.0.0.1:{port}/health")).header("X-Substrate-E3-Readiness-Nonce",&expected.readiness_nonce).send().await.unwrap().status(),403);
            let mut denied=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
            denied.write_all(format!("POST /v1/responses HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
            let mut reply=[0;512];
            let length=tokio::time::timeout(std::time::Duration::from_secs(2),denied.read(&mut reply)).await.unwrap().unwrap();
            assert!(reply[..length].starts_with(b"HTTP/1.1 403"));
            serving.abort();
            assert!(serving.await.unwrap_err().is_cancelled());
        });
        writer.join().unwrap();
    }
}
