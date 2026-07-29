use super::{PlatformWorldContext, WorldTransport};
use crate::execution::policy_snapshot::bootstrap_world_spec as build_bootstrap_world_spec;
use crate::execution::settings;
#[cfg(test)]
use crate::execution::world_env_guard;
use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use substrate_broker::world_fs_mode;
use transport_api_client::{AgentClient, Transport};
use transport_api_types::{
    normalize_windows_pipe_path, InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1,
    PlatformPrincipalV1, PlatformTransportIdentityV1, WindowsForwarderScopeV1,
};
use world_api::{SharedWorldOwnerSpec, WorldBackend, WorldSpec};
use world_windows_wsl::WindowsWslBackend;

fn context() -> Result<Arc<PlatformWorldContext>> {
    if let Some(ctx) = super::get_context() {
        return Ok(ctx);
    }

    let ctx = detect()?;
    super::store_context_globally(ctx);
    super::get_context().ok_or_else(|| anyhow::anyhow!("windows platform context unavailable"))
}

pub fn ensure_world_ready_with_state(no_world: bool) -> Result<Option<String>> {
    ensure_world_ready_impl(no_world, None, get_backend)
}

fn ensure_world_ready_impl<F>(
    no_world: bool,
    shared_world: Option<&SharedWorldOwnerSpec>,
    backend_provider: F,
) -> Result<Option<String>>
where
    F: FnOnce() -> Result<Arc<dyn WorldBackend>>,
{
    if no_world {
        return Ok(None);
    }

    #[cfg(test)]
    let _env_guard = world_env_guard();

    super::with_supported_shared_world_request(shared_world, "windows world bootstrap", || {
        let backend = backend_provider()?;
        let spec = bootstrap_world_spec();
        match backend.ensure_session(&spec) {
            Ok(handle) => {
                std::env::set_var("SUBSTRATE_WORLD", "enabled");
                std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                Ok(Some(handle.id))
            }
            Err(_err) => Ok(None),
        }
    })
}

pub fn get_backend() -> Result<Arc<dyn WorldBackend>> {
    let _ = context()?;
    anyhow::bail!(
        "Windows platform world backend lifecycle remains gated on the R3 provisioning/lifecycle contract"
    )
}

pub fn bootstrap_world_spec() -> WorldSpec {
    build_bootstrap_world_spec(settings::world_root_from_env().path, world_fs_mode())
}

pub fn detect() -> Result<PlatformWorldContext> {
    let encoded_host_context =
        env::var(crate::execution::install_bootstrap::INSTALL_BOOTSTRAP_CONTEXT_ENV)
            .context("checked Windows install bootstrap projection is missing")?;
    let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host_context)
        .context("checked Windows install bootstrap projection is invalid")?;
    crate::execution::install_bootstrap::bind_windows_install_bootstrap_context(&host_carrier)
        .context("checked Windows install bootstrap projection principal is invalid")?;
    crate::execution::install_bootstrap::validate_install_bootstrap_projections(
        &host_carrier,
        &encoded_host_context,
        |key| env::var_os(key),
    )
    .context("checked Windows install bootstrap projection is missing or conflicting")?;

    let (current_principal, local_app_data) =
        crate::execution::install_bootstrap::current_windows_principal_and_known_folder()
            .context("failed to observe current Windows principal and Known Folder")?;
    let PlatformPrincipalV1::Windows { sid, .. } = &current_principal else {
        anyhow::bail!("Windows platform world requires a Windows install bootstrap carrier");
    };

    let default_pipe =
        normalize_windows_pipe_path(world_windows_wsl::transport::DEFAULT_AGENT_PIPE)
            .map_err(anyhow::Error::from)
            .context("default Windows platform world pipe path is invalid")?;
    if let Some(conflicting_pipe) = env::var_os("SUBSTRATE_FORWARDER_PIPE") {
        let conflicting_pipe = conflicting_pipe
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("SUBSTRATE_FORWARDER_PIPE is not valid UTF-8"))?;
        let conflicting_pipe = normalize_windows_pipe_path(conflicting_pipe)
            .map_err(anyhow::Error::from)
            .context("SUBSTRATE_FORWARDER_PIPE is invalid")?;
        if conflicting_pipe != default_pipe {
            anyhow::bail!(
                "Windows platform world pipe projection conflicts with the authenticated mapping"
            );
        }
    }

    let observed_guest = {
        let list_names = |args: &[&str], context: &str| -> Result<String> {
            let output = std::process::Command::new("wsl.exe")
                .args(args)
                .output()
                .with_context(|| {
                    format!(
                        "failed to observe WSL state via `wsl.exe {}`",
                        args.join(" ")
                    )
                })?;
            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "{context}\nstdout:\n{}\nstderr:\n{}",
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim(),
                ));
            }
            String::from_utf8(output.stdout).context("WSL observation produced non-UTF-8 output")
        };

        let declared_distro_name = world_windows_wsl::transport::DEFAULT_DISTRO;
        let registered = list_names(&["-l", "-q"], "unable to enumerate registered WSL distros")?;
        let registered_matches = registered
            .lines()
            .map(|line| {
                line.trim_matches(|ch| ch == '\r' || ch == '\u{feff}')
                    .trim()
            })
            .filter(|line| !line.is_empty())
            .filter(|line| line.eq_ignore_ascii_case(declared_distro_name))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if registered_matches.is_empty() {
            anyhow::bail!("declared WSL distro `{declared_distro_name}` is not registered");
        }
        if registered_matches.len() != 1 {
            anyhow::bail!(
                "declared WSL distro `{declared_distro_name}` matched multiple registered distros"
            );
        }
        let exact_distro_name = registered_matches[0].clone();

        let running = list_names(&["-l", "-v"], "unable to enumerate running WSL distros")?;
        let ordered_names = registered_matches
            .iter()
            .cloned()
            .chain([exact_distro_name.clone()])
            .collect::<Vec<_>>();
        let running_matches = running
            .lines()
            .map(|line| line.trim_matches('\r'))
            .filter_map(|line| {
                let mut trimmed = line.trim_start();
                if trimmed.is_empty()
                    || trimmed.starts_with("NAME")
                    || trimmed.starts_with("Windows Subsystem for Linux")
                {
                    return None;
                }
                if let Some(stripped) = trimmed.strip_prefix('*') {
                    trimmed = stripped.trim_start();
                }
                let matched_name = ordered_names
                    .iter()
                    .filter(|name| trimmed.starts_with(name.as_str()))
                    .max_by_key(|name| name.len())?;
                let remainder = &trimmed[matched_name.len()..];
                let state = remainder
                    .split_once(char::is_whitespace)
                    .map(|_| remainder.trim_start().split_whitespace().next())
                    .flatten()?;
                state
                    .eq_ignore_ascii_case("Running")
                    .then(|| matched_name.to_string())
            })
            .filter(|name| name.eq_ignore_ascii_case(declared_distro_name))
            .collect::<Vec<_>>();
        if running_matches.is_empty() {
            anyhow::bail!("declared WSL distro `{declared_distro_name}` is not running");
        }
        if running_matches.len() != 1 || running_matches[0] != exact_distro_name {
            anyhow::bail!(
                "declared WSL distro `{declared_distro_name}` resolved to an ambiguous running spelling"
            );
        }

        let output = std::process::Command::new("wsl.exe")
            .args([
                "-d",
                &exact_distro_name,
                "--",
                "bash",
                "-lc",
                "set -euo pipefail\nmachine_id=\"$(tr -d '\\n' </etc/machine-id)\"\naccount=\"$(id -un)\"\nuid=\"$(id -u)\"\npasswd_by_name=\"$(getent passwd \"$account\")\"\npasswd_by_uid=\"$(getent passwd \"$uid\")\"\nprintf 'machine_id=%s\\n' \"$machine_id\"\nprintf 'account=%s\\n' \"$account\"\nprintf 'uid=%s\\n' \"$uid\"\nprintf 'passwd_by_name=%s\\n' \"$passwd_by_name\"\nprintf 'passwd_by_uid=%s\\n' \"$passwd_by_uid\"\n",
            ])
            .output()
            .context("failed to observe WSL guest identity")?;
        if !output.status.success() {
            anyhow::bail!(
                "unable to observe WSL guest identity\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            );
        }

        let stdout = String::from_utf8(output.stdout)
            .context("WSL guest identity output is not valid UTF-8")?;
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
            machine_id.ok_or_else(|| anyhow::anyhow!("WSL guest machine ID is missing"))?;
        if machine_id.len() != 32
            || !machine_id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            anyhow::bail!("WSL guest machine ID is malformed");
        }
        let account = account.ok_or_else(|| anyhow::anyhow!("WSL guest account is missing"))?;
        if account.is_empty()
            || account
                .chars()
                .any(|ch| matches!(ch, '\0' | '\n' | '\r' | ':' | '/'))
        {
            anyhow::bail!("WSL guest account is malformed");
        }
        let uid = uid
            .ok_or_else(|| anyhow::anyhow!("WSL guest UID is missing"))?
            .parse::<u32>()
            .context("WSL guest UID is malformed")?;
        let passwd_by_name =
            passwd_by_name.ok_or_else(|| anyhow::anyhow!("WSL passwd entry is missing"))?;
        let passwd_by_uid =
            passwd_by_uid.ok_or_else(|| anyhow::anyhow!("WSL passwd UID entry is missing"))?;
        if passwd_by_name != passwd_by_uid {
            anyhow::bail!("WSL account-database lookup by name and UID did not round-trip");
        }
        let passwd_fields = passwd_by_name.split(':').collect::<Vec<_>>();
        if passwd_fields.len() < 7 {
            anyhow::bail!("WSL passwd entry is malformed");
        }
        if passwd_fields[0] != account {
            anyhow::bail!("WSL passwd entry account does not match the active guest account");
        }
        if passwd_fields[2]
            .parse::<u32>()
            .context("WSL passwd UID is malformed")?
            != uid
        {
            anyhow::bail!("WSL passwd entry UID does not match the active guest UID");
        }
        let home = transport_api_types::normalize_unix_install_bootstrap_path(passwd_fields[5])
            .context("WSL passwd home directory is invalid")?;
        if home.starts_with("/mnt/") {
            anyhow::bail!("WSL passwd home directory may not resolve to a host-mounted path");
        }

        Ok::<_, anyhow::Error>((exact_distro_name, machine_id, account, uid, home))
    }?;

    let (exact_distro_name, guest_machine_id, guest_account, guest_uid, guest_home) =
        observed_guest;
    let scope =
        WindowsForwarderScopeV1::derive(sid, &exact_distro_name, &guest_machine_id, &default_pipe)
            .map_err(anyhow::Error::from)
            .context("failed to derive the canonical Windows forwarder scope")?;
    let control_root = local_app_data
        .join("Substrate")
        .join("forwarder")
        .join(&scope.0);
    let mapping = PlatformBootstrapMappingV1::new_wsl(
        &host_carrier,
        &exact_distro_name,
        &guest_machine_id,
        control_root
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Windows control root is not valid UTF-8"))?,
        &format!("{guest_home}/.substrate"),
        &guest_account,
        guest_uid,
        &default_pipe,
        "/run/substrate.sock",
    )
    .map_err(anyhow::Error::from)
    .context("failed to construct the canonical Windows platform bootstrap mapping")?;

    let project_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let backend = Arc::new(WindowsWslBackend::new_with_mapping(
        host_carrier.clone(),
        mapping.clone(),
        project_path,
    )?);
    let backend_trait: Arc<dyn WorldBackend> = backend;
    let pipe_path = match &mapping.realized_transport {
        PlatformTransportIdentityV1::Wsl { pipe_path, .. } => PathBuf::from(pipe_path),
        _ => anyhow::bail!("Windows platform world requires a WSL transport mapping"),
    };
    let transport = WorldTransport::NamedPipe(pipe_path.clone());
    let socket_path = pipe_path;

    let ensure_ready = Box::new(move || {
        anyhow::bail!(
            "typed Windows WSL provisioning remains gated on the R3 provisioning/lifecycle contract"
        )
    });
    let ensure_persistent_session_ready_async = Box::new(move || {
        Box::pin(async move {
            anyhow::bail!(
                "typed Windows WSL provisioning remains gated on the R3 provisioning/lifecycle contract"
            )
        }) as super::PersistentSessionReadyFuture
    });

    Ok(PlatformWorldContext {
        backend: backend_trait,
        transport,
        socket_path,
        #[cfg(not(test))]
        bootstrap_mapping: Some(mapping),
        ensure_ready,
        ensure_persistent_session_ready_async,
    })
}

pub fn build_agent_client() -> Result<AgentClient> {
    let ctx = context()?;
    let pipe_path = authenticated_pipe_path(&ctx)?;
    AgentClient::new(Transport::NamedPipe { path: pipe_path })
}

fn authenticated_pipe_path(ctx: &PlatformWorldContext) -> Result<PathBuf> {
    #[cfg(not(test))]
    {
        let mapping = ctx
            .bootstrap_mapping
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Windows platform world mapping is unavailable"))?;
        let PlatformTransportIdentityV1::Wsl { pipe_path, .. } = &mapping.realized_transport else {
            anyhow::bail!("Windows platform world mapping does not carry a WSL pipe transport");
        };
        return Ok(PathBuf::from(pipe_path));
    }

    #[cfg(test)]
    {
        match &ctx.transport {
            WorldTransport::NamedPipe(path) => Ok(path.clone()),
            _ => anyhow::bail!("Windows platform world mapping is unavailable in unit-test builds"),
        }
    }
}

/// Convert a host Windows path to the corresponding WSL path string using the active backend.
pub fn to_wsl_path_string(path: &std::path::Path) -> Result<String> {
    // If relative, resolve against current_dir then convert
    let path = if path.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    } else {
        path.to_path_buf()
    };
    let raw = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path is not valid UTF-8: {}", path.display()))?;
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with("//") {
        // UNC path: //server/share/dir -> /mnt/unc/server/share/dir
        let rest = normalized.trim_start_matches('/');
        Ok(format!("/mnt/unc/{}", rest))
    } else if let Some((drive, rest)) = normalized.split_once(':') {
        // Drive letter path: C:/foo -> /mnt/c/foo
        let rest = rest.trim_start_matches('/');
        Ok(format!("/mnt/{}/{}", drive.to_lowercase(), rest))
    } else {
        // Already a Unix-style path
        Ok(normalized)
    }
}

/// Convert current working directory to a WSL path string.
pub fn current_dir_wsl() -> Result<String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    to_wsl_path_string(&cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use world_api::{ExecRequest, ExecResult, FsDiff, WorldHandle};

    #[derive(Clone)]
    struct StubBackend {
        handle: WorldHandle,
        ensure_calls: Arc<AtomicUsize>,
    }

    impl StubBackend {
        fn new(id: &str, ensure_calls: Arc<AtomicUsize>) -> Self {
            Self {
                handle: WorldHandle {
                    id: id.to_string(),
                    shared_binding: None,
                },
                ensure_calls,
            }
        }
    }

    impl WorldBackend for StubBackend {
        fn ensure_session(&self, _spec: &WorldSpec) -> Result<WorldHandle> {
            self.ensure_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.handle.clone())
        }

        fn exec(&self, _world: &WorldHandle, _req: ExecRequest) -> Result<ExecResult> {
            Err(anyhow!("exec not implemented in stub"))
        }

        fn fs_diff(&self, _world: &WorldHandle, _span_id: &str) -> Result<FsDiff> {
            Ok(FsDiff::default())
        }

        fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    #[serial_test::serial]
    fn ensure_world_ready_sets_env_on_success() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");

        let calls = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(StubBackend::new("wld_test", calls.clone()));

        let result = ensure_world_ready_impl(false, None, || Ok(backend.clone())).unwrap();
        assert_eq!(result.as_deref(), Some("wld_test"));
        assert_eq!(std::env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(std::env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_test");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");
    }

    #[test]
    #[serial_test::serial]
    fn ensure_world_ready_ignores_disabled_env_when_forced() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::set_var("SUBSTRATE_WORLD", "disabled");
        std::env::remove_var("SUBSTRATE_WORLD_ID");

        let calls = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(StubBackend::new("wld_forced", calls.clone()));

        let result = ensure_world_ready_impl(false, None, || Ok(backend.clone())).unwrap();
        assert_eq!(result.as_deref(), Some("wld_forced"));
        assert_eq!(std::env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(std::env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_forced");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");
    }

    #[test]
    #[serial_test::serial]
    fn ensure_world_ready_respects_no_world_flag() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");

        let calls = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(StubBackend::new("wld_test", calls.clone()));

        let result = ensure_world_ready_impl(true, None, || Ok(backend.clone())).unwrap();
        assert!(result.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");
    }

    #[test]
    fn authenticated_pipe_path_uses_explicit_named_pipe_transport_in_tests() {
        let calls = Arc::new(AtomicUsize::new(0));
        let backend: Arc<dyn WorldBackend> = Arc::new(StubBackend::new("wld_test", calls));
        let ctx = PlatformWorldContext {
            backend,
            transport: WorldTransport::NamedPipe(PathBuf::from(r"\\.\pipe\substrate-agent")),
            socket_path: PathBuf::from(r"\\.\pipe\substrate-agent"),
            ensure_ready: Box::new(|| Ok(())),
            ensure_persistent_session_ready_async: Box::new(|| {
                Box::pin(async { Ok(()) }) as super::PersistentSessionReadyFuture
            }),
        };

        assert_eq!(
            authenticated_pipe_path(&ctx).unwrap(),
            PathBuf::from(r"\\.\pipe\substrate-agent")
        );
    }
}
