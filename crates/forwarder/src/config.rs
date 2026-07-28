use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::Command;
use transport_api_types::{
    normalize_windows_install_bootstrap_path, normalize_windows_pipe_path,
    InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1, PlatformInstanceIdentityV1,
    PlatformPrincipalV1, PlatformTransportIdentityV1, WindowsForwarderScopeV1,
};

#[cfg(test)]
use std::cell::RefCell;

const DEFAULT_UDS_PATH: &str = "/run/substrate.sock";
const DEFAULT_TCP_PORT: u16 = 61337;
const CONFIG_FILE_NAME: &str = "forwarder.toml";
const TARGET_ENV: &str = "SUBSTRATE_FORWARDER_TARGET";

#[cfg(test)]
thread_local! {
    static TEST_CURRENT_WINDOWS_HOST_OBSERVATION: RefCell<Option<Result<(String, String, String), String>>> =
        RefCell::new(None);
    static TEST_FILE_SETTINGS_OVERRIDE: RefCell<Option<Option<String>>> = RefCell::new(None);
}

#[cfg(test)]
pub(crate) fn set_test_current_windows_host_observation(
    observation: Option<Result<(String, String, String), String>>,
) {
    TEST_CURRENT_WINDOWS_HOST_OBSERVATION.with(|cell| {
        *cell.borrow_mut() = observation;
    });
}

#[cfg(test)]
pub(crate) fn set_test_file_settings_override(override_value: Option<Option<&str>>) {
    TEST_FILE_SETTINGS_OVERRIDE.with(|cell| {
        *cell.borrow_mut() = override_value.map(|value| value.map(str::to_string));
    });
}

#[derive(Clone, Debug)]
pub struct ForwarderConfig {
    pub distro: String,
    pub pipe_path: String,
    pub host_tcp_bridge: Option<SocketAddr>,
    target: BridgeTarget,
    internal_host_carrier: Option<InstallBootstrapContextCarrierV1>,
    internal_platform_mapping: Option<PlatformBootstrapMappingV1>,
    internal_config_path: Option<String>,
    internal_log_dir: Option<String>,
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
            internal_host_carrier: None,
            internal_platform_mapping: None,
            internal_config_path: None,
            internal_log_dir: None,
        })
    }

    pub fn load_internal(
        distro: String,
        pipe_path: String,
        host_tcp_bridge: Option<SocketAddr>,
        config_path: PathBuf,
        log_dir: PathBuf,
        encoded_host_carrier: &str,
        encoded_mapping: &str,
    ) -> anyhow::Result<Self> {
        let normalize_cli_path = |path: &PathBuf, label: &str| -> anyhow::Result<String> {
            let path_str = path.to_str().ok_or_else(|| {
                anyhow!("internal authenticated forwarder {label} must be valid UTF-8")
            })?;
            normalize_windows_install_bootstrap_path(path_str).map_err(|_| {
                anyhow!("internal authenticated forwarder {label} must be an absolute normalized Windows path")
            })
        };
        let parse_internal_file_target =
            |file_target: Option<&FileTarget>| -> anyhow::Result<Option<BridgeTarget>> {
                let Some(file_target) = file_target else {
                    return Ok(None);
                };
                if file_target.mode.is_none()
                    && file_target.tcp_port.is_none()
                    && file_target.uds_path.is_none()
                {
                    return Ok(None);
                }
                let Some(mode) = file_target.mode else {
                    anyhow::bail!(
                        "internal authenticated forwarder mode requires explicit target.mode in the config file"
                    );
                };
                match mode {
                    TargetModeSetting::Uds => Ok(Some(BridgeTarget::Uds {
                        path: file_target
                            .uds_path
                            .clone()
                            .unwrap_or_else(|| DEFAULT_UDS_PATH.to_string()),
                    })),
                    TargetModeSetting::Tcp => Ok(Some(BridgeTarget::Tcp {
                        addr: SocketAddr::from((
                            Ipv4Addr::LOCALHOST,
                            file_target.tcp_port.unwrap_or(DEFAULT_TCP_PORT),
                        )),
                    })),
                }
            };
        let parse_env_target =
            |env_override: Option<EnvOverride>| -> anyhow::Result<Option<BridgeTarget>> {
                let Some(env_override) = env_override else {
                    return Ok(None);
                };
                match env_override.mode {
                    TargetModeSetting::Uds => {
                        let Some(path) = env_override.value else {
                            anyhow::bail!("internal authenticated forwarder target projection must include a UDS path");
                        };
                        Ok(Some(BridgeTarget::Uds { path }))
                    }
                    TargetModeSetting::Tcp => {
                        let Some(port) = env_override.value else {
                            anyhow::bail!("internal authenticated forwarder target projection must include a TCP port");
                        };
                        let port = port.parse::<u16>().map_err(|err| {
                            anyhow!(
                                "invalid internal authenticated forwarder TCP port {port}: {err}"
                            )
                        })?;
                        Ok(Some(BridgeTarget::Tcp {
                            addr: SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
                        }))
                    }
                }
            };
        let target_matches = |candidate: &BridgeTarget, expected: &BridgeTarget| -> bool {
            match (candidate, expected) {
                (BridgeTarget::Uds { path: left }, BridgeTarget::Uds { path: right }) => {
                    left == right
                }
                (BridgeTarget::Tcp { addr: left }, BridgeTarget::Tcp { addr: right }) => {
                    left == right
                }
                _ => false,
            }
        };

        let host_carrier = InstallBootstrapContextCarrierV1::decode(encoded_host_carrier).context(
            "internal authenticated forwarder mode requires a valid install bootstrap carrier",
        )?;
        if host_carrier.encode().context(
            "internal authenticated forwarder install bootstrap carrier is not canonical",
        )? != encoded_host_carrier
        {
            anyhow::bail!(
                "internal authenticated forwarder install bootstrap carrier is not canonical"
            );
        }
        let platform_mapping = PlatformBootstrapMappingV1::decode(encoded_mapping, &host_carrier)
            .context(
            "internal authenticated forwarder mode requires a valid platform bootstrap mapping",
        )?;
        if platform_mapping.encode(&host_carrier).context(
            "internal authenticated forwarder platform bootstrap mapping is not canonical",
        )? != encoded_mapping
        {
            anyhow::bail!(
                "internal authenticated forwarder platform bootstrap mapping is not canonical"
            );
        }

        let normalized_config_path = normalize_cli_path(&config_path, "config path")?;
        let normalized_log_dir = normalize_cli_path(&log_dir, "log directory")?;

        let mapping_guest_socket = match (
            &platform_mapping.platform_instance,
            &platform_mapping.realized_transport,
        ) {
            (
                PlatformInstanceIdentityV1::Wsl { .. },
                PlatformTransportIdentityV1::Wsl { guest_socket, .. },
            ) => guest_socket.clone(),
            _ => {
                anyhow::bail!(
                        "internal authenticated forwarder mode requires a WSL platform bootstrap mapping"
                    );
            }
        };
        let product_target = BridgeTarget::Uds {
            path: mapping_guest_socket,
        };
        let effective_target = if let Some(addr) = host_tcp_bridge {
            if !addr.ip().is_loopback() {
                anyhow::bail!(
                    "internal authenticated forwarder diagnostic TCP bridge must use a loopback address"
                );
            }
            BridgeTarget::Tcp { addr }
        } else {
            product_target.clone()
        };

        let config = Self {
            distro,
            pipe_path,
            host_tcp_bridge,
            target: effective_target,
            internal_host_carrier: Some(host_carrier),
            internal_platform_mapping: Some(platform_mapping),
            internal_config_path: Some(normalized_config_path),
            internal_log_dir: Some(normalized_log_dir),
        };
        config.validate_mapping()?;

        let file_settings = load_file_settings(Some(PathBuf::from(
            config
                .internal_config_path
                .as_deref()
                .context("internal authenticated forwarder mode requires a stored config path")?,
        )))?;
        let env_override = std::env::var(TARGET_ENV)
            .ok()
            .map(|value| parse_env_override(&value))
            .transpose()?;
        let file_target =
            parse_internal_file_target(file_settings.as_ref().and_then(|s| s.target.as_ref()))?;
        let env_target = parse_env_target(env_override)?;

        if let Some(candidate) = file_target.as_ref() {
            if !target_matches(candidate, &product_target) {
                anyhow::bail!(
                    "internal authenticated forwarder config target does not match the verified mapping"
                );
            }
        }
        if let Some(candidate) = env_target.as_ref() {
            if !target_matches(candidate, &product_target) {
                anyhow::bail!(
                    "internal authenticated forwarder target projection does not match the verified mapping"
                );
            }
        }

        Ok(config)
    }

    pub fn target(&self) -> &BridgeTarget {
        &self.target
    }

    pub fn target_mode(&self) -> &'static str {
        self.target.mode()
    }

    pub fn validate_mapping(&self) -> anyhow::Result<()> {
        let host_carrier = self.internal_host_carrier.as_ref().context(
            "internal authenticated forwarder mode requires a stored install bootstrap carrier",
        )?;
        let platform_mapping = self.internal_platform_mapping.as_ref().context(
            "internal authenticated forwarder mode requires a stored platform bootstrap mapping",
        )?;
        let config_path = self
            .internal_config_path
            .as_deref()
            .context("internal authenticated forwarder mode requires a stored config path")?;
        let log_dir = self
            .internal_log_dir
            .as_deref()
            .context("internal authenticated forwarder mode requires a stored log directory")?;

        let encoded_host_carrier = host_carrier
            .encode()
            .context("internal authenticated forwarder install bootstrap carrier is invalid")?;
        let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host_carrier)
            .context(
                "internal authenticated forwarder install bootstrap carrier is not canonical",
            )?;
        let encoded_mapping = platform_mapping
            .encode(&host_carrier)
            .context("internal authenticated forwarder platform bootstrap mapping is invalid")?;
        let platform_mapping = PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
            .context(
                "internal authenticated forwarder platform bootstrap mapping is not canonical",
            )?;

        let PlatformPrincipalV1::Windows {
            account: expected_account,
            sid: expected_sid,
        } = &host_carrier.context.intended_host_principal
        else {
            anyhow::bail!(
                "internal authenticated forwarder mode requires a Windows install bootstrap carrier"
            );
        };

        let (expected_distro, expected_guest_machine_id, expected_pipe_path, expected_guest_socket) =
            match (
                &platform_mapping.platform_instance,
                &platform_mapping.realized_transport,
            ) {
                (
                    PlatformInstanceIdentityV1::Wsl {
                        distro_name,
                        guest_machine_id,
                    },
                    PlatformTransportIdentityV1::Wsl {
                        pipe_path,
                        guest_socket,
                    },
                ) => (
                    distro_name.as_str(),
                    guest_machine_id.as_str(),
                    pipe_path.as_str(),
                    guest_socket.as_str(),
                ),
                _ => {
                    anyhow::bail!(
                    "internal authenticated forwarder mode requires a WSL platform bootstrap mapping"
                );
                }
            };

        if normalize_windows_pipe_path(&self.pipe_path)
            .context("internal authenticated forwarder pipe path is invalid")?
            != self.pipe_path
        {
            anyhow::bail!("internal authenticated forwarder pipe path is not canonical");
        }
        if self.distro != expected_distro {
            anyhow::bail!(
                "internal authenticated forwarder distro does not match the verified mapping"
            );
        }
        if self.pipe_path != expected_pipe_path {
            anyhow::bail!(
                "internal authenticated forwarder pipe path does not match the verified mapping"
            );
        }
        if expected_guest_socket != DEFAULT_UDS_PATH {
            anyhow::bail!(
                "internal authenticated forwarder guest socket does not match the verified mapping"
            );
        }

        #[cfg(test)]
        let current_windows_host = if let Some(observation) =
            TEST_CURRENT_WINDOWS_HOST_OBSERVATION.with(|cell| cell.borrow().clone())
        {
            let (account, sid, local_app_data) = observation.map_err(anyhow::Error::msg)?;
            (
                account,
                sid,
                normalize_windows_install_bootstrap_path(&local_app_data)
                    .context("current token LocalApplicationData is invalid")?,
            )
        } else {
            let output = Command::new("pwsh")
                .args([
                    "-NoProfile",
                    "-NoLogo",
                    "-Command",
                    "$ErrorActionPreference = 'Stop'\nif (-not ('SubstrateKnownFolderNative' -as [type])) {\nAdd-Type -TypeDefinition @'\nusing System;\nusing System.Runtime.InteropServices;\npublic static class SubstrateKnownFolderNative {\n    [DllImport(\"shell32.dll\")]\n    public static extern int SHGetKnownFolderPath(ref Guid rfid, uint dwFlags, IntPtr hToken, out IntPtr ppszPath);\n}\n'@\n}\n$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()\nif ($null -eq $identity -or $null -eq $identity.User) { throw 'current Windows principal is unavailable' }\n$folderId = [Guid]'F1B32785-6FBA-4FCF-9D55-7B8E7F157091'\n$raw = [IntPtr]::Zero\n$hr = [SubstrateKnownFolderNative]::SHGetKnownFolderPath([ref]$folderId, 0, $identity.Token, [ref]$raw)\nif ($hr -lt 0 -or $raw -eq [IntPtr]::Zero) { throw 'current token LocalApplicationData is unavailable' }\ntry {\n    $path = [Runtime.InteropServices.Marshal]::PtrToStringUni($raw)\n    if ([string]::IsNullOrEmpty($path)) { throw 'current token LocalApplicationData is malformed' }\n    Write-Output ('account=' + $identity.Name)\n    Write-Output ('sid=' + $identity.User.Value)\n    Write-Output ('local_app_data=' + $path)\n} finally {\n    if ($raw -ne [IntPtr]::Zero) {\n        [Runtime.InteropServices.Marshal]::FreeCoTaskMem($raw)\n    }\n}\n",
                ])
                .output()
                .context("failed to observe the current Windows principal")?;
            if !output.status.success() {
                anyhow::bail!("unable to observe the current Windows principal");
            }
            let stdout = String::from_utf8(output.stdout)
                .context("current Windows principal observation output is not valid UTF-8")?;
            let mut account = None;
            let mut sid = None;
            let mut local_app_data = None;
            for line in stdout.lines() {
                let trimmed = line.trim_matches('\r');
                let Some((key, value)) = trimmed.split_once('=') else {
                    continue;
                };
                match key {
                    "account" => account = Some(value.to_string()),
                    "sid" => sid = Some(value.to_string()),
                    "local_app_data" => local_app_data = Some(value.to_string()),
                    _ => {}
                }
            }
            (
                account.ok_or_else(|| anyhow!("current Windows account is missing"))?,
                sid.ok_or_else(|| anyhow!("current Windows SID is missing"))?,
                normalize_windows_install_bootstrap_path(
                    &local_app_data
                        .ok_or_else(|| anyhow!("current token LocalApplicationData is missing"))?,
                )
                .context("current token LocalApplicationData is invalid")?,
            )
        };

        #[cfg(not(test))]
        let current_windows_host = {
            unsafe extern "system" {
                fn GetSystemDirectoryW(lpbuffer: *mut u16, usize: u32) -> u32;
            }

            let mut buffer = vec![0_u16; 32768];
            // SAFETY: buffer points to writable UTF-16 storage for the system directory query.
            let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
            if length == 0 || length as usize >= buffer.len() {
                anyhow::bail!("trusted Windows system directory is unavailable");
            }
            let system_directory = String::from_utf16(&buffer[..length as usize])
                .context("trusted Windows system directory is malformed")?;
            let powershell_path = PathBuf::from(system_directory)
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe");

            let output = Command::new(&powershell_path)
                .env_remove("PATH")
                .args([
                    "-NoProfile",
                    "-NoLogo",
                    "-Command",
                    "$ErrorActionPreference = 'Stop'\nif (-not ('SubstrateKnownFolderNative' -as [type])) {\nAdd-Type -TypeDefinition @'\nusing System;\nusing System.Runtime.InteropServices;\npublic static class SubstrateKnownFolderNative {\n    [DllImport(\"shell32.dll\")]\n    public static extern int SHGetKnownFolderPath(ref Guid rfid, uint dwFlags, IntPtr hToken, out IntPtr ppszPath);\n}\n'@\n}\n$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()\nif ($null -eq $identity -or $null -eq $identity.User) { throw 'current Windows principal is unavailable' }\n$folderId = [Guid]'F1B32785-6FBA-4FCF-9D55-7B8E7F157091'\n$raw = [IntPtr]::Zero\n$hr = [SubstrateKnownFolderNative]::SHGetKnownFolderPath([ref]$folderId, 0, $identity.Token, [ref]$raw)\nif ($hr -lt 0 -or $raw -eq [IntPtr]::Zero) { throw 'current token LocalApplicationData is unavailable' }\ntry {\n    $path = [Runtime.InteropServices.Marshal]::PtrToStringUni($raw)\n    if ([string]::IsNullOrEmpty($path)) { throw 'current token LocalApplicationData is malformed' }\n    Write-Output ('account=' + $identity.Name)\n    Write-Output ('sid=' + $identity.User.Value)\n    Write-Output ('local_app_data=' + $path)\n} finally {\n    if ($raw -ne [IntPtr]::Zero) {\n        [Runtime.InteropServices.Marshal]::FreeCoTaskMem($raw)\n    }\n}\n",
                ])
                .output()
                .context("failed to observe the current Windows principal")?;
            if !output.status.success() {
                anyhow::bail!("unable to observe the current Windows principal");
            }
            let stdout = String::from_utf8(output.stdout)
                .context("current Windows principal observation output is not valid UTF-8")?;
            let mut account = None;
            let mut sid = None;
            let mut local_app_data = None;
            for line in stdout.lines() {
                let trimmed = line.trim_matches('\r');
                let Some((key, value)) = trimmed.split_once('=') else {
                    continue;
                };
                match key {
                    "account" => account = Some(value.to_string()),
                    "sid" => sid = Some(value.to_string()),
                    "local_app_data" => local_app_data = Some(value.to_string()),
                    _ => {}
                }
            }
            (
                account.ok_or_else(|| anyhow!("current Windows account is missing"))?,
                sid.ok_or_else(|| anyhow!("current Windows SID is missing"))?,
                normalize_windows_install_bootstrap_path(
                    &local_app_data
                        .ok_or_else(|| anyhow!("current token LocalApplicationData is missing"))?,
                )
                .context("current token LocalApplicationData is invalid")?,
            )
        };

        let (current_account, current_sid, local_app_data) = current_windows_host;
        if current_account != *expected_account || current_sid != *expected_sid {
            anyhow::bail!(
                "internal authenticated forwarder install bootstrap carrier does not match the current Windows principal"
            );
        }

        let expected_scope = WindowsForwarderScopeV1::derive(
            expected_sid,
            expected_distro,
            expected_guest_machine_id,
            expected_pipe_path,
        )
        .context("internal authenticated forwarder scope is invalid")?;
        let expected_control_root = normalize_windows_install_bootstrap_path(&format!(
            r"{}\Substrate\forwarder\{}",
            local_app_data, expected_scope.0
        ))
        .context("internal authenticated forwarder control root is invalid")?;
        if platform_mapping.host_platform_control_root != expected_control_root {
            anyhow::bail!(
                "internal authenticated forwarder control root does not match the current Windows scope"
            );
        }

        let expected_config_path = normalize_windows_install_bootstrap_path(&format!(
            r"{}\forwarder\{}",
            host_carrier.context.selected_host_prefix, CONFIG_FILE_NAME
        ))
        .context("internal authenticated forwarder config path is invalid")?;
        if config_path != expected_config_path {
            anyhow::bail!(
                "internal authenticated forwarder config path does not match the verified host prefix"
            );
        }
        let expected_log_dir = normalize_windows_install_bootstrap_path(&format!(
            r"{}\forwarder\logs",
            host_carrier.context.selected_host_prefix
        ))
        .context("internal authenticated forwarder log directory is invalid")?;
        if log_dir != expected_log_dir {
            anyhow::bail!(
                "internal authenticated forwarder log directory does not match the verified host prefix"
            );
        }

        match (&self.target, self.host_tcp_bridge) {
            (BridgeTarget::Uds { path }, None) if path == expected_guest_socket => Ok(()),
            (BridgeTarget::Tcp { addr }, Some(expected_addr)) if *addr == expected_addr => Ok(()),
            (BridgeTarget::Uds { .. }, Some(_)) => Err(anyhow!(
                "internal authenticated forwarder diagnostic TCP mode was not preserved"
            )),
            _ => Err(anyhow!(
                "internal authenticated forwarder target does not match the verified mapping"
            )),
        }
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
    let mode_part = parts
        .next()
        .ok_or_else(|| anyhow!("{TARGET_ENV} must include mode (tcp:PORT or uds:PATH)"))?;
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
        .unwrap_or(TargetModeSetting::Tcp);

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
    #[cfg(test)]
    if let Some(override_value) = TEST_FILE_SETTINGS_OVERRIDE.with(|cell| cell.borrow().clone()) {
        return match override_value {
            Some(content) => {
                let settings: FileSettings = toml::from_str(&content)
                    .context("failed parsing forwarder config from test override")?;
                Ok(Some(settings))
            }
            None => Ok(None),
        };
    }

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
    use transport_api_types::{
        InstallBootstrapContextV1, PlatformBootstrapMappingV1, WindowsForwarderScopeV1,
    };

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

    fn set_current_windows_host_observation(
        observation: Option<Result<(String, String, String), String>>,
    ) {
        set_test_current_windows_host_observation(observation);
    }

    fn set_file_settings_override(override_value: Option<Option<&str>>) {
        set_test_file_settings_override(override_value);
    }

    fn sample_internal_state() -> (String, String, PathBuf, PathBuf, String, String) {
        let host_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_windows(
                r"C:\Users\Alice\AppData\Local\Substrate",
                r"ACME\Alice",
                "S-1-5-21-1000",
            )
            .unwrap(),
        )
        .unwrap();
        let scope = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .unwrap();
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &host_carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            &format!(
                r"C:\Users\Alice\AppData\Local\Substrate\forwarder\{}",
                scope.0
            ),
            "/home/substrate/.substrate",
            "substrate",
            1000,
            r"\\.\pipe\substrate-agent",
            "/run/substrate.sock",
        )
        .unwrap();
        (
            host_carrier.encode().unwrap(),
            mapping.encode(&host_carrier).unwrap(),
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml"),
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs"),
            "Substrate-WSL".to_string(),
            r"\\.\pipe\substrate-agent".to_string(),
        )
    }

    #[test]
    fn default_target_is_tcp_without_config() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let pipe = r"\\.\pipe\substrate";
        let config = ForwarderConfig::load(
            "distro".to_string(),
            pipe.to_string(),
            None,
            Some(PathBuf::from("does/not/exist.toml")),
        )
        .unwrap();
        assert_eq!(config.target_mode(), "tcp");
        assert_eq!(
            config.target().to_string(),
            format!("127.0.0.1:{}", DEFAULT_TCP_PORT)
        );
    }

    #[test]
    fn file_config_selects_uds_mode() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let path = temp_path("uds");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let toml = "[target]\nmode = \"uds\"\nuds_path = \"/run/custom.sock\"\n";
        fs::write(&path, toml).unwrap();

        let pipe = r"\\.\pipe\substrate";
        let config = ForwarderConfig::load(
            "distro".to_string(),
            pipe.to_string(),
            None,
            Some(path.clone()),
        )
        .unwrap();

        assert_eq!(config.target_mode(), "uds");
        assert_eq!(config.target().to_string(), "/run/custom.sock");
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

        let pipe = r"\\.\pipe\substrate";
        let config = ForwarderConfig::load(
            "distro".to_string(),
            pipe.to_string(),
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

        let pipe = r"\\.\pipe\substrate";
        let config = ForwarderConfig::load(
            "distro".to_string(),
            pipe.to_string(),
            None,
            Some(path.clone()),
        )
        .unwrap();

        assert_eq!(config.target_mode(), "tcp");
        assert_eq!(config.target().to_string(), "127.0.0.1:60100");

        std::env::remove_var(TARGET_ENV);
    }

    #[test]
    fn env_override_empty_is_error() {
        let _guard = ENV_GUARD.lock().unwrap();
        std::env::set_var(TARGET_ENV, "   ");

        let pipe = r"\\.\pipe\substrate";
        let result = ForwarderConfig::load(
            "distro".to_string(),
            pipe.to_string(),
            None,
            Some(PathBuf::from("does/not/exist.toml")),
        );

        assert!(
            result.is_err(),
            "expected empty {TARGET_ENV} to return an error"
        );

        std::env::remove_var(TARGET_ENV);
    }

    #[test]
    fn internal_load_accepts_matching_mapping_and_paths() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();
        std::env::set_var(TARGET_ENV, "uds:/run/substrate.sock");

        let config = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap();

        assert_eq!(config.target_mode(), "uds");
        assert_eq!(config.target().to_string(), "/run/substrate.sock");

        reset_env();
        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_accepts_explicit_diagnostic_tcp_bridge() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();
        std::env::set_var(TARGET_ENV, "uds:/run/substrate.sock");

        let config = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 5000))),
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap();

        assert_eq!(config.target_mode(), "tcp");
        assert_eq!(config.target().to_string(), "127.0.0.1:5000");

        reset_env();
        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_non_loopback_diagnostic_tcp_bridge() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            Some(SocketAddr::from(([0, 0, 0, 0], 5000))),
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("must use a loopback address"),
            "unexpected error: {err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_conflicting_target_projection() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();
        std::env::set_var(TARGET_ENV, "tcp:61337");

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("target projection does not match the verified mapping"),
            "unexpected error: {err}"
        );

        reset_env();
        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_target_config_without_explicit_mode() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(Some("[target]\nuds_path = \"/run/substrate.sock\"\n")));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("requires explicit target.mode in the config file"),
            "unexpected error: {err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_conflicting_file_target() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(Some(
            "[target]\nmode = \"uds\"\nuds_path = \"/run/other.sock\"\n",
        )));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("config target does not match the verified mapping"),
            "unexpected error: {err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_wrong_current_windows_principal() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Bob".to_string(),
            "S-1-5-21-1001".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("does not match the current Windows principal"),
            "unexpected error: {err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_conflicting_known_folder_scope() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"D:\Other".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();

        let err = ForwarderConfig::load_internal(
            distro,
            pipe_path,
            None,
            config_path,
            log_dir,
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("control root does not match the current Windows scope"),
            "unexpected error: {err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_wrong_pipe_or_config_or_log_path() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, _pipe_path) = sample_internal_state();

        let pipe_err = ForwarderConfig::load_internal(
            distro.clone(),
            r"\\.\pipe\Substrate-Agent".to_string(),
            None,
            config_path.clone(),
            log_dir.clone(),
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            pipe_err.to_string().contains("pipe path is not canonical"),
            "unexpected error: {pipe_err}"
        );

        let config_err = ForwarderConfig::load_internal(
            distro.clone(),
            r"\\.\pipe\substrate-agent".to_string(),
            None,
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\other\forwarder.toml"),
            log_dir.clone(),
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            config_err
                .to_string()
                .contains("config path does not match the verified host prefix"),
            "unexpected error: {config_err}"
        );

        let log_err = ForwarderConfig::load_internal(
            distro,
            r"\\.\pipe\substrate-agent".to_string(),
            None,
            config_path,
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\other\logs"),
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            log_err
                .to_string()
                .contains("log directory does not match the verified host prefix"),
            "unexpected error: {log_err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }

    #[test]
    fn internal_load_rejects_wrong_distro_or_guest_socket_or_carrier() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        set_file_settings_override(Some(None));
        set_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, _distro, pipe_path) = sample_internal_state();

        let distro_err = ForwarderConfig::load_internal(
            "substrate-wsl".to_string(),
            pipe_path.clone(),
            None,
            config_path.clone(),
            log_dir.clone(),
            &carrier,
            &mapping,
        )
        .unwrap_err();
        assert!(
            distro_err
                .to_string()
                .contains("distro does not match the verified mapping"),
            "unexpected error: {distro_err}"
        );

        let guest_socket_err = {
            let host_carrier = InstallBootstrapContextCarrierV1::decode(&carrier).unwrap();
            let decoded =
                transport_api_types::PlatformBootstrapMappingV1::decode(&mapping, &host_carrier)
                    .unwrap();
            let tampered_mapping = PlatformBootstrapMappingV1::new_wsl(
                &host_carrier,
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                &decoded.host_platform_control_root,
                "/home/substrate/.substrate",
                "substrate",
                1000,
                r"\\.\pipe\substrate-agent",
                "/run/other.sock",
            )
            .unwrap();
            ForwarderConfig::load_internal(
                "Substrate-WSL".to_string(),
                pipe_path.clone(),
                None,
                config_path.clone(),
                log_dir.clone(),
                &carrier,
                &tampered_mapping.encode(&host_carrier).unwrap(),
            )
            .unwrap_err()
        };
        assert!(
            guest_socket_err
                .to_string()
                .contains("guest socket does not match the verified mapping"),
            "unexpected error: {guest_socket_err}"
        );

        let commitment_err = {
            let other_host_carrier = InstallBootstrapContextCarrierV1::from_context(
                InstallBootstrapContextV1::new_windows(
                    r"C:\Users\Alice\AppData\Local\OtherSubstrate",
                    r"ACME\Alice",
                    "S-1-5-21-1000",
                )
                .unwrap(),
            )
            .unwrap();
            let foreign_mapping = PlatformBootstrapMappingV1::new_wsl(
                &other_host_carrier,
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"C:\Users\Alice\AppData\Local\Substrate\forwarder\3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a",
                "/home/substrate/.substrate",
                "substrate",
                1000,
                r"\\.\pipe\substrate-agent",
                "/run/substrate.sock",
            )
            .unwrap();
            ForwarderConfig::load_internal(
                "Substrate-WSL".to_string(),
                pipe_path.clone(),
                None,
                config_path.clone(),
                log_dir.clone(),
                &carrier,
                &foreign_mapping.encode(&other_host_carrier).unwrap(),
            )
            .unwrap_err()
        };
        assert!(
            commitment_err
                .to_string()
                .contains("requires a valid platform bootstrap mapping"),
            "unexpected error: {commitment_err}"
        );

        let carrier_err = ForwarderConfig::load_internal(
            "Substrate-WSL".to_string(),
            pipe_path,
            None,
            config_path,
            log_dir,
            "not-valid",
            &mapping,
        )
        .unwrap_err();
        assert!(
            carrier_err
                .to_string()
                .contains("requires a valid install bootstrap carrier"),
            "unexpected error: {carrier_err}"
        );

        set_current_windows_host_observation(None);
        set_file_settings_override(None);
    }
}
