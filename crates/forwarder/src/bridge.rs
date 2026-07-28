use crate::config::{BridgeTarget, ForwarderConfig};
use crate::wsl;
use anyhow::Context;
use std::io;
use std::net::SocketAddr;
use std::os::windows::io::AsRawHandle;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::net::TcpStream;
use transport_api_types::InstallBootstrapContextCarrierV1;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::FlushFileBuffers;

const INSTALL_BOOTSTRAP_CONTEXT_ENV: &str = "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1";
const INSTALL_BOOTSTRAP_COMMITMENT_ENV: &str = "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT";
const LEGACY_FORWARDER_CONFIG_ERROR: &str =
    "internal authenticated forwarder mode requires a stored install bootstrap carrier";

#[derive(Clone, Debug)]
pub(crate) struct VerifiedForwarderProjection {
    pub(crate) distro: String,
    pub(crate) pipe_path: String,
    pub(crate) target: BridgeTarget,
    pub(crate) host_context_commitment: Option<String>,
}

pub(crate) fn verify_forwarder_projection(
    config: &ForwarderConfig,
    expected_target: Option<&BridgeTarget>,
) -> anyhow::Result<VerifiedForwarderProjection> {
    let target = config.target().clone();

    match config.validate_mapping() {
        Ok(()) => {}
        Err(err)
            if err
                .chain()
                .any(|cause| cause.to_string().contains(LEGACY_FORWARDER_CONFIG_ERROR)) =>
        {
            if let Some(expected_target) = expected_target {
                if target.mode() != expected_target.mode()
                    || target.to_string() != expected_target.to_string()
                {
                    anyhow::bail!("forwarder target does not match the configured session target");
                }
            }
            return Ok(VerifiedForwarderProjection {
                distro: config.distro.clone(),
                pipe_path: config.pipe_path.clone(),
                target,
                host_context_commitment: None,
            });
        }
        Err(err) => {
            return Err(err)
                .context("forwarder mapping must validate before listener bind or WSL spawn");
        }
    }

    let encoded_carrier = std::env::var(INSTALL_BOOTSTRAP_CONTEXT_ENV).context(
        "internal authenticated forwarder mode requires a projected install bootstrap carrier",
    )?;
    let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_carrier).context(
        "internal authenticated forwarder install bootstrap carrier projection is invalid",
    )?;
    if host_carrier
        .encode()
        .context("internal authenticated forwarder install bootstrap carrier is not canonical")?
        != encoded_carrier
    {
        anyhow::bail!(
            "internal authenticated forwarder install bootstrap carrier projection is not canonical"
        );
    }

    let projected_commitment = std::env::var(INSTALL_BOOTSTRAP_COMMITMENT_ENV)
        .context("internal authenticated forwarder mode requires a projected host commitment")?;
    if projected_commitment != host_carrier.host_context_commitment {
        anyhow::bail!(
            "internal authenticated forwarder host commitment projection does not match the authenticated carrier"
        );
    }

    if let Some(expected_target) = expected_target {
        if target.mode() != expected_target.mode()
            || target.to_string() != expected_target.to_string()
        {
            anyhow::bail!(
                "internal authenticated forwarder target does not match the verified mapping"
            );
        }
    }

    Ok(VerifiedForwarderProjection {
        distro: config.distro.clone(),
        pipe_path: config.pipe_path.clone(),
        target,
        host_context_commitment: Some(projected_commitment),
    })
}

pub async fn run_pipe_session(
    mut pipe: NamedPipeServer,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    let target = config.target().clone();
    tracing::info!(
        session = session_id,
        kind = "pipe",
        target_mode = target.mode(),
        target = %target,
        "client connected"
    );

    let verified = verify_forwarder_projection(&config, Some(&target))?;
    let mut wsl_bundle = spawn_bridge(&verified, session_id, "pipe").await?;
    {
        let mut bridge_stream = wsl_bundle.stream_mut();
        bridge_copy(session_id, "pipe", &target, &mut pipe, &mut bridge_stream).await;

        match pipe.flush().await {
            Ok(()) => tracing::trace!(session = session_id, "pipe async flush complete"),
            Err(err) => tracing::debug!(session = session_id, "pipe async flush error: {err}"),
        }
        match flush_pipe_buffers(&pipe) {
            Ok(()) => tracing::trace!(session = session_id, "FlushFileBuffers complete"),
            Err(err) => tracing::debug!(session = session_id, "FlushFileBuffers error: {err}"),
        }
        match pipe.disconnect() {
            Ok(()) => tracing::trace!(session = session_id, "pipe disconnect complete"),
            Err(err) => tracing::debug!(session = session_id, "pipe disconnect error: {err}"),
        }

        if let Err(err) = bridge_stream.shutdown().await {
            tracing::debug!(
                session = session_id,
                target_mode = target.mode(),
                target = %target,
                "WSL stream shutdown error: {err}"
            );
        }
    }

    finalize_bridge(wsl_bundle, session_id, &target).await
}

pub async fn run_tcp_session(
    mut stream: TcpStream,
    peer: SocketAddr,
    config: Arc<ForwarderConfig>,
    session_id: u64,
) -> anyhow::Result<()> {
    let target = config.target().clone();
    let label = format!("tcp:{peer}");
    tracing::info!(
        session = session_id,
        kind = "tcp",
        peer = %peer,
        target_mode = target.mode(),
        target = %target,
        "client connected"
    );

    let verified = verify_forwarder_projection(&config, Some(&target))?;
    let mut wsl_bundle = spawn_bridge(&verified, session_id, &label).await?;
    {
        let mut bridge_stream = wsl_bundle.stream_mut();
        bridge_copy(session_id, &label, &target, &mut stream, &mut bridge_stream).await;

        if let Err(err) = stream.shutdown().await {
            tracing::debug!(session = session_id, "tcp client shutdown error: {err}");
        }
        if let Err(err) = bridge_stream.shutdown().await {
            tracing::debug!(
                session = session_id,
                target_mode = target.mode(),
                target = %target,
                "WSL stream shutdown error: {err}"
            );
        }
    }

    finalize_bridge(wsl_bundle, session_id, &target).await
}

async fn spawn_bridge(
    verified: &VerifiedForwarderProjection,
    session_id: u64,
    label: &str,
) -> anyhow::Result<wsl::WslStreamBundle> {
    wsl::spawn(
        &verified.distro,
        &verified.target,
        verified.host_context_commitment.as_deref(),
        session_id,
        label,
    )
    .await
    .with_context(|| {
        format!(
            "session {session_id}: failed to spawn WSL bridge for {label} via {}",
            verified.target
        )
    })
}

async fn finalize_bridge(
    bundle: wsl::WslStreamBundle,
    session_id: u64,
    target: &BridgeTarget,
) -> anyhow::Result<()> {
    let status = bundle
        .wait()
        .await
        .with_context(|| format!("session {session_id}: failed to wait for WSL process"))?;
    if !status.success() {
        tracing::warn!(
            session = session_id,
            target_mode = target.mode(),
            target = %target,
            exit = ?status.code(),
            "WSL bridge exited with failure"
        );
    }

    Ok(())
}

async fn bridge_copy<C, W>(
    session_id: u64,
    label: &str,
    target: &BridgeTarget,
    client: &mut C,
    bridge: &mut W,
) where
    C: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    W: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match tokio::io::copy_bidirectional(client, bridge).await {
        Ok((c2w, w2c)) => {
            tracing::debug!(
                session = session_id,
                kind = label,
                target_mode = target.mode(),
                target = %target,
                client_to_wsl = c2w,
                wsl_to_client = w2c,
                "stream closed"
            );
        }
        Err(err) => {
            tracing::warn!(
                session = session_id,
                kind = label,
                target_mode = target.mode(),
                target = %target,
                error = %err,
                "bidirectional copy failed"
            );
        }
    }
}

fn flush_pipe_buffers(pipe: &NamedPipeServer) -> io::Result<()> {
    let handle = pipe.as_raw_handle();
    unsafe {
        FlushFileBuffers(HANDLE(handle as *mut _))
            .map_err(|err| io::Error::from_raw_os_error(err.code().0))?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) static TEST_PROJECTION_ENV_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) struct TestProjectionEnvReset;

#[cfg(test)]
pub(crate) fn lock_test_projection_env_guard() -> std::sync::MutexGuard<'static, ()> {
    TEST_PROJECTION_ENV_GUARD
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
pub(crate) fn set_test_projection_env(
    carrier: Option<&str>,
    commitment: Option<&str>,
) -> TestProjectionEnvReset {
    match carrier {
        Some(value) => std::env::set_var(INSTALL_BOOTSTRAP_CONTEXT_ENV, value),
        None => std::env::remove_var(INSTALL_BOOTSTRAP_CONTEXT_ENV),
    }
    match commitment {
        Some(value) => std::env::set_var(INSTALL_BOOTSTRAP_COMMITMENT_ENV, value),
        None => std::env::remove_var(INSTALL_BOOTSTRAP_COMMITMENT_ENV),
    }
    TestProjectionEnvReset
}

#[cfg(test)]
impl Drop for TestProjectionEnvReset {
    fn drop(&mut self) {
        std::env::remove_var(INSTALL_BOOTSTRAP_CONTEXT_ENV);
        std::env::remove_var(INSTALL_BOOTSTRAP_COMMITMENT_ENV);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        set_test_current_windows_host_observation, set_test_file_settings_override, BridgeTarget,
        ForwarderConfig,
    };
    use std::path::PathBuf;
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        WindowsForwarderScopeV1,
    };

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

    fn load_internal_config() -> ForwarderConfig {
        set_test_file_settings_override(Some(None));
        set_test_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));
        let (carrier, mapping, config_path, log_dir, distro, pipe_path) = sample_internal_state();
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
        config
    }

    #[test]
    fn verified_projection_uses_exact_authenticated_values() {
        let _guard = lock_test_projection_env_guard();
        let config = load_internal_config();
        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        );

        let verified = verify_forwarder_projection(
            &config,
            Some(&BridgeTarget::Uds {
                path: "/run/substrate.sock".to_string(),
            }),
        )
        .unwrap();

        assert_eq!(verified.distro, "Substrate-WSL");
        assert_eq!(verified.pipe_path, r"\\.\pipe\substrate-agent");
        assert_eq!(verified.target.to_string(), "/run/substrate.sock");
        assert_eq!(
            verified.host_context_commitment.as_deref(),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7")
        );

        set_test_current_windows_host_observation(None);
        set_test_file_settings_override(None);
    }

    #[test]
    fn verified_projection_rejects_missing_or_conflicting_commitment_projection() {
        let _guard = lock_test_projection_env_guard();
        let config = load_internal_config();

        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            None,
        );
        let missing_err = verify_forwarder_projection(&config, None).unwrap_err();
        assert!(
            missing_err
                .to_string()
                .contains("requires a projected host commitment"),
            "unexpected error: {missing_err}"
        );

        let _projection = set_test_projection_env(
            std::env::var(INSTALL_BOOTSTRAP_CONTEXT_ENV).ok().as_deref(),
            Some("0"),
        );
        let conflicting_err = verify_forwarder_projection(&config, None).unwrap_err();
        assert!(
            conflicting_err
                .to_string()
                .contains("host commitment projection does not match"),
            "unexpected error: {conflicting_err}"
        );

        set_test_current_windows_host_observation(None);
        set_test_file_settings_override(None);
    }

    #[test]
    fn verified_projection_rejects_target_drift_before_spawn() {
        let _guard = lock_test_projection_env_guard();
        let config = load_internal_config();
        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        );

        let err = verify_forwarder_projection(
            &config,
            Some(&BridgeTarget::Tcp {
                addr: "127.0.0.1:61337".parse().unwrap(),
            }),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("target does not match the verified mapping"),
            "unexpected error: {err}"
        );

        set_test_current_windows_host_observation(None);
        set_test_file_settings_override(None);
    }

    #[test]
    fn verified_projection_preserves_legacy_forwarder_startup() {
        let _guard = lock_test_projection_env_guard();
        set_test_file_settings_override(Some(None));

        let config = ForwarderConfig::load(
            "Substrate-WSL".to_string(),
            r"\\localhost\pipe\Substrate-Agent".to_string(),
            None,
            None,
        )
        .unwrap();

        let verified = verify_forwarder_projection(&config, Some(config.target())).unwrap();
        assert_eq!(verified.distro, "Substrate-WSL");
        assert_eq!(verified.pipe_path, r"\\localhost\pipe\Substrate-Agent");
        assert!(verified.host_context_commitment.is_none());

        set_test_file_settings_override(None);
    }
}
