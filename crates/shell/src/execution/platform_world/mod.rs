//! Platform-specific world detection and context (macOS + Linux compatibility shell-facing API).

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub fn detect() -> Result<PlatformWorldContext> {
    windows::detect()
}

#[cfg(not(target_os = "windows"))]
use crate::execution::policy_snapshot::bootstrap_world_spec;
#[cfg(not(target_os = "windows"))]
use crate::execution::settings;
#[cfg(not(target_os = "windows"))]
use anyhow::Context;
use anyhow::Result;
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
#[cfg(not(target_os = "windows"))]
use tokio::net::{TcpStream, UnixStream};
#[cfg(not(target_os = "windows"))]
use tokio_tungstenite as tungs;
#[cfg(not(test))]
use transport_api_types::PlatformBootstrapMappingV1;
use transport_api_types::SharedWorldOwnerSpec;
use world_api::WorldBackend;

#[derive(Clone, Debug)]
pub enum WorldTransport {
    Unix(PathBuf),
    Tcp {
        host: String,
        port: u16,
    },
    Vsock {
        port: u16,
    },
    #[cfg(target_os = "windows")]
    NamedPipe(PathBuf),
}

#[cfg(target_os = "windows")]
const _: Option<WorldTransport> = Some(WorldTransport::Vsock { port: 0 });

impl fmt::Display for WorldTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldTransport::Unix(p) => write!(f, "unix:{}", p.display()),
            WorldTransport::Tcp { host, port } => write!(f, "tcp:{}:{}", host, port),
            WorldTransport::Vsock { port } => write!(f, "vsock:{}", port),
            #[cfg(target_os = "windows")]
            WorldTransport::NamedPipe(p) => write!(f, "pipe:{}", p.display()),
        }
    }
}

pub struct PlatformWorldContext {
    pub backend: Arc<dyn WorldBackend>,
    pub transport: WorldTransport,
    #[allow(dead_code)]
    pub socket_path: PathBuf,
    #[cfg(not(test))]
    pub bootstrap_mapping: Option<PlatformBootstrapMappingV1>,
    pub ensure_ready: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    #[allow(dead_code)]
    pub ensure_persistent_session_ready_async: Box<PersistentSessionReadyFn>,
}

pub type PersistentSessionReadyFuture =
    Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>;
pub type PersistentSessionReadyFn = dyn Fn() -> PersistentSessionReadyFuture + Send + Sync;

#[cfg(not(target_os = "windows"))]
pub(crate) trait WorldTransportStreamIo:
    tokio::io::AsyncRead + tokio::io::AsyncWrite
{
}

#[cfg(not(target_os = "windows"))]
impl<T> WorldTransportStreamIo for T where T: tokio::io::AsyncRead + tokio::io::AsyncWrite + ?Sized {}

#[cfg(not(target_os = "windows"))]
pub(crate) type WorldTransportWsIo = Box<dyn WorldTransportStreamIo + Unpin + Send>;

static GLOBAL_CTX: OnceLock<Arc<PlatformWorldContext>> = OnceLock::new();

fn validate_shared_owner_request_support(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
) -> Result<()> {
    let Some(request) = request else {
        return Ok(());
    };

    #[cfg(target_os = "linux")]
    {
        let _ = operation;
        let _ = request;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        if std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
            anyhow::bail!(
                "{} rejects explicit shared-owner world reuse when SUBSTRATE_WORLD_SOCKET overrides the Lima-backed transport (orchestration_session_id={})",
                operation,
                request.orchestration_session_id
            );
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        anyhow::bail!(
            "{} rejects explicit shared-owner world reuse on this platform (orchestration_session_id={})",
            operation,
            request.orchestration_session_id
        );
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn reject_non_linux_shared_owner_request(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
) -> Result<()> {
    validate_shared_owner_request_support(request, operation)
}

pub(crate) fn with_supported_shared_world_request<T, F>(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
    on_supported: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    validate_shared_owner_request_support(request, operation)?;
    on_supported()
}

pub fn store_context_globally(ctx: PlatformWorldContext) {
    let _ = GLOBAL_CTX.set(Arc::new(ctx));
}

pub fn get_context() -> Option<Arc<PlatformWorldContext>> {
    GLOBAL_CTX.get().cloned()
}

#[allow(dead_code)]
pub async fn ensure_persistent_session_ready_async(ctx: &PlatformWorldContext) -> Result<()> {
    (ctx.ensure_persistent_session_ready_async.as_ref())().await
}

#[cfg(not(target_os = "windows"))]
pub(crate) async fn connect_transport_stream_ws(
    transport: &WorldTransport,
) -> Result<tungs::WebSocketStream<WorldTransportWsIo>> {
    match transport {
        WorldTransport::Unix(path) => {
            let stream = UnixStream::connect(path)
                .await
                .with_context(|| format!("connect world-service UDS ({})", path.display()))?;
            let url = url::Url::parse("ws://localhost/v1/stream").expect("static ws URL");
            let io: WorldTransportWsIo = Box::new(stream);
            let (ws, _resp) = tungs::client_async(url, io)
                .await
                .context("ws handshake /v1/stream")?;
            Ok(ws)
        }
        WorldTransport::Tcp { host, port } => {
            let ws_url = format!("ws://{host}:{port}/v1/stream");
            let url = url::Url::parse(&ws_url).context("invalid ws URL")?;
            let stream = TcpStream::connect((host.as_str(), *port))
                .await
                .with_context(|| format!("connect world-service TCP ({host}:{port})"))?;
            let io: WorldTransportWsIo = Box::new(stream);
            let (ws, _resp) = tungs::client_async(url, io)
                .await
                .context("ws handshake /v1/stream")?;
            Ok(ws)
        }
        WorldTransport::Vsock { port } => {
            let host = "127.0.0.1";
            let ws_url = format!("ws://{host}:{port}/v1/stream");
            let url = url::Url::parse(&ws_url).context("invalid ws URL")?;
            let stream = TcpStream::connect((host, *port)).await.with_context(|| {
                format!("connect world-service VSock proxy TCP ({host}:{port})")
            })?;
            let io: WorldTransportWsIo = Box::new(stream);
            let (ws, _resp) = tungs::client_async(url, io)
                .await
                .context("ws handshake /v1/stream")?;
            Ok(ws)
        }
        #[cfg(target_os = "windows")]
        WorldTransport::NamedPipe(path) => anyhow::bail!(
            "named-pipe stream websocket transport not supported here ({})",
            path.display()
        ),
    }
}

#[cfg(target_os = "macos")]
pub fn detect() -> Result<PlatformWorldContext> {
    use std::process::Command;
    use transport_api_types::{normalize_unix_install_bootstrap_path, PlatformBootstrapMappingV1};
    use world_mac_lima::transport::{
        managed_host_socket_path_for_mapping, CANONICAL_GUEST_SOCKET_PATH,
    };
    use world_mac_lima::MacLimaBackend;

    let install_context =
        crate::execution::install_bootstrap::checked_install_bootstrap_context_from_projections()
            .context("macOS platform world requires checked install bootstrap projections")?;
    if std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
        anyhow::bail!(
            "macOS platform world requires the authenticated Lima mapping; SUBSTRATE_WORLD_SOCKET is non-authoritative"
        );
    }

    let vm_name = std::env::var("SUBSTRATE_LIMA_VM_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "macOS platform world requires an explicitly declared Lima instance via SUBSTRATE_LIMA_VM_NAME"
            )
        })?;
    let (_, host_account_home) =
        crate::execution::install_bootstrap::current_unix_principal_and_home()
            .context("macOS platform world requires the current install principal account home")?;
    let host_control_root = host_account_home.join(".lima");
    let host_socket = PathBuf::from(&install_context.context.selected_host_prefix)
        .join("sock")
        .join("agent.sock");

    let observed_guest = {
        let previous_home = std::env::var_os("HOME");
        let previous_lima_home = std::env::var_os("LIMA_HOME");
        std::env::set_var("HOME", &host_account_home);
        std::env::set_var("LIMA_HOME", &host_control_root);
        let result = (|| -> Result<(String, String, u32, String)> {
            let limactl_path =
                if let Some(test_override) = std::env::var_os("SUBSTRATE_TEST_LIMACTL_PATH") {
                    PathBuf::from(test_override)
                } else {
                    which::which("limactl").or_else(|_| {
                        for candidate in [
                            "/opt/homebrew/bin/limactl",
                            "/usr/local/bin/limactl",
                            "/opt/homebrew/sbin/limactl",
                            "/usr/local/sbin/limactl",
                        ] {
                            let candidate = std::path::Path::new(candidate);
                            if candidate.is_file() {
                                return Ok(candidate.to_path_buf());
                            }
                        }

                        Err(anyhow::anyhow!(
                            "limactl not found. Install Lima with: brew install lima"
                        ))
                    })?
                };
            let output = Command::new(&limactl_path)
                .args([
                    "shell",
                    "--workdir=/",
                    &vm_name,
                    "bash",
                    "-lc",
                    "set -euo pipefail\nmachine_id=\"$(tr -d '\\n' </etc/machine-id)\"\naccount=\"$(id -un)\"\nuid=\"$(id -u)\"\npasswd_by_name=\"$(getent passwd \"$account\")\"\npasswd_by_uid=\"$(getent passwd \"$uid\")\"\nprintf 'machine_id=%s\\n' \"$machine_id\"\nprintf 'account=%s\\n' \"$account\"\nprintf 'uid=%s\\n' \"$uid\"\nprintf 'passwd_by_name=%s\\n' \"$passwd_by_name\"\nprintf 'passwd_by_uid=%s\\n' \"$passwd_by_uid\"\n",
                ])
                .output()
                .context("failed to observe Lima guest identity")?;
            if !output.status.success() {
                anyhow::bail!(
                    "unable to observe Lima guest identity\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim(),
                );
            }

            let stdout = String::from_utf8(output.stdout)
                .context("Lima guest identity output is not valid UTF-8")?;
            let mut machine_id = None;
            let mut account = None;
            let mut uid = None;
            let mut passwd_by_name = None;
            let mut passwd_by_uid = None;
            for line in stdout.lines() {
                let trimmed = line.trim_matches('\r');
                let Some((key, value)) = trimmed.split_once('=') else {
                    continue;
                };
                match key {
                    "machine_id" => machine_id = Some(value.to_string()),
                    "account" => account = Some(value.to_string()),
                    "uid" => uid = Some(value.to_string()),
                    "passwd_by_name" => passwd_by_name = Some(value.to_string()),
                    "passwd_by_uid" => passwd_by_uid = Some(value.to_string()),
                    _ => {}
                }
            }

            let machine_id =
                machine_id.ok_or_else(|| anyhow::anyhow!("Lima guest machine ID is missing"))?;
            if machine_id.len() != 32
                || !machine_id
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                anyhow::bail!("Lima guest machine ID is malformed");
            }

            let account =
                account.ok_or_else(|| anyhow::anyhow!("Lima guest account is missing"))?;
            if account.is_empty()
                || account
                    .chars()
                    .any(|ch| matches!(ch, '\0' | '\n' | '\r' | ':' | '/'))
            {
                anyhow::bail!("Lima guest account is malformed");
            }

            let uid = uid
                .ok_or_else(|| anyhow::anyhow!("Lima guest UID is missing"))?
                .parse::<u32>()
                .context("Lima guest UID is malformed")?;
            let passwd_by_name =
                passwd_by_name.ok_or_else(|| anyhow::anyhow!("Lima passwd entry is missing"))?;
            let passwd_by_uid =
                passwd_by_uid.ok_or_else(|| anyhow::anyhow!("Lima passwd UID entry is missing"))?;
            if passwd_by_name != passwd_by_uid {
                anyhow::bail!("Lima account-database lookup by name and UID did not round-trip");
            }

            let passwd_fields = passwd_by_name.split(':').collect::<Vec<_>>();
            if passwd_fields.len() < 7 {
                anyhow::bail!("Lima passwd entry is malformed");
            }
            if passwd_fields[0] != account {
                anyhow::bail!("Lima passwd entry account does not match the active guest account");
            }
            if passwd_fields[2]
                .parse::<u32>()
                .context("Lima passwd UID is malformed")?
                != uid
            {
                anyhow::bail!("Lima passwd entry UID does not match the active guest UID");
            }
            let guest_home = normalize_unix_install_bootstrap_path(passwd_fields[5])
                .context("Lima passwd home directory is invalid")?;
            if guest_home.starts_with("/mnt/") {
                anyhow::bail!("Lima passwd home directory may not resolve to a host-mounted path");
            }

            Ok((machine_id, account, uid, guest_home))
        })();
        match previous_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match previous_lima_home {
            Some(value) => std::env::set_var("LIMA_HOME", value),
            None => std::env::remove_var("LIMA_HOME"),
        }
        result
    }?;

    let (guest_machine_id, guest_account, guest_uid, guest_home) = observed_guest;
    let mapping = PlatformBootstrapMappingV1::new_lima(
        &install_context,
        &vm_name,
        &guest_machine_id,
        host_control_root
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("macOS Lima control root is not valid UTF-8"))?,
        &format!("{guest_home}/.substrate"),
        &guest_account,
        guest_uid,
        host_socket
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("macOS host socket projection is not valid UTF-8"))?,
        CANONICAL_GUEST_SOCKET_PATH,
    )
    .map_err(anyhow::Error::from)
    .context("failed to construct canonical Lima platform bootstrap mapping")?;

    let socket_path = managed_host_socket_path_for_mapping(&install_context, &mapping)
        .context("failed to derive typed Lima host socket projection")?;
    let transport = WorldTransport::Unix(socket_path.clone());

    let backend = Arc::new(MacLimaBackend::new_with_mapping(
        install_context,
        mapping.clone(),
    )?);
    let ensure_ready_backend = backend.clone();
    let ensure_persistent_session_ready_async_backend = backend.clone();
    let ensure_ready = Box::new(move || {
        use world_api::WorldBackend as _;
        let spec = bootstrap_world_spec(
            settings::world_root_from_env().path,
            substrate_broker::world_fs_mode(),
        );
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });
    let ensure_persistent_session_ready_async = Box::new(move || {
        let backend = ensure_persistent_session_ready_async_backend.clone();
        Box::pin(async move { backend.ensure_persistent_session_ready_async().await })
            as PersistentSessionReadyFuture
    });

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path,
        #[cfg(not(test))]
        bootstrap_mapping: Some(mapping),
        ensure_ready,
        ensure_persistent_session_ready_async,
    })
}

#[cfg(target_os = "linux")]
pub fn detect() -> Result<PlatformWorldContext> {
    // Preserve Linux behavior: local world backend
    use world::LinuxLocalBackend;
    use world_api::WorldBackend as _;

    let backend = Arc::new(LinuxLocalBackend::new());
    let ensure_ready_backend = backend.clone();
    let ensure_persistent_session_ready_async_backend = backend.clone();
    let ensure_ready = Box::new(move || {
        let spec = bootstrap_world_spec(
            settings::world_root_from_env().path,
            substrate_broker::world_fs_mode(),
        );
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });
    let ensure_persistent_session_ready_async = Box::new(move || {
        let backend = ensure_persistent_session_ready_async_backend.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                use world_api::WorldBackend as _;
                let spec = bootstrap_world_spec(
                    settings::world_root_from_env().path,
                    substrate_broker::world_fs_mode(),
                );
                backend.ensure_session(&spec).map(|_| ())
            })
            .await
            .context("persistent-session readiness join failure")?
        }) as PersistentSessionReadyFuture
    });

    // Native Linux agent socket path
    let sock = PathBuf::from("/run/substrate.sock");
    let transport = WorldTransport::Unix(sock.clone());

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path: sock,
        #[cfg(not(test))]
        bootstrap_mapping: None,
        ensure_ready,
        ensure_persistent_session_ready_async,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn vsock_variant_displays_port() {
        let transport = WorldTransport::Vsock { port: 17788 };
        assert_eq!(transport.to_string(), "vsock:17788");
    }

    #[test]
    fn unix_and_tcp_transports_format_endpoints() {
        let unix = WorldTransport::Unix(PathBuf::from("/tmp/substrate.sock"));
        assert_eq!(unix.to_string(), "unix:/tmp/substrate.sock");

        let tcp = WorldTransport::Tcp {
            host: "127.0.0.1".into(),
            port: 9001,
        };
        assert_eq!(tcp.to_string(), "tcp:127.0.0.1:9001");
    }

    #[test]
    fn windows_world_doctor_source_persists_context_before_client_selection() {
        let source = include_str!("../platform/windows.rs");
        let start = source
            .find("pub(crate) fn world_doctor_main(")
            .expect("world_doctor_main");
        let end = source[start..]
            .find("let ok =")
            .map(|offset| start + offset)
            .expect("world_doctor_main summary boundary");
        let section = &source[start..end];

        let detect_pos = section
            .find("crate::execution::pw::detect().and_then(|detected| {")
            .expect("detect path");
        let store_pos = section
            .find("crate::execution::pw::store_context_globally(detected);")
            .expect("context store");
        let client_pos = section
            .find("crate::execution::pw::windows::build_agent_client()?")
            .expect("typed client build");

        assert!(
            detect_pos < store_pos && store_pos < client_pos,
            "Windows world doctor must persist the detected context before building the typed client so host diagnostics and world doctor share one mapping"
        );
    }

    #[test]
    fn macos_world_doctor_human_output_formats_optional_guest_socket_safely() {
        let source = include_str!("../platform/macos.rs");
        let start = source
            .find("Observed transport target: selected-prefix/sock/agent.sock -> {}")
            .expect("transport target formatting");
        let section = &source[start..source.len().min(start + 300)];

        assert!(
            section.contains(".transport_guest_socket")
                && section.contains(".as_deref()")
                && section.contains(".unwrap_or(\"unavailable\")"),
            "human macOS doctor output must render the optional guest socket without requiring Display on Option<String>"
        );
    }
}
