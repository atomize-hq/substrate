use anyhow::Result;
#[cfg(any(test, not(target_os = "linux")))]
use anyhow::{anyhow, Context};
use std::path::Path;
#[cfg(any(test, not(target_os = "linux")))]
use std::path::PathBuf;
use std::sync::Arc;
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};
#[cfg(any(test, not(target_os = "linux")))]
use transport_api_types::{PlatformInstanceIdentityV1, PlatformTransportIdentityV1};
use world_api::WorldBackend;

#[cfg(any(test, not(target_os = "linux")))]
#[cfg_attr(any(test, not(target_os = "linux")), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FactoryPlatform {
    Macos,
    Windows,
    Unsupported,
}

#[cfg(any(test, target_os = "macos", windows))]
fn fail_closed_platform_factory(platform: FactoryPlatform) -> Result<Arc<dyn WorldBackend>> {
    let platform_name = match platform {
        FactoryPlatform::Macos => "macOS",
        FactoryPlatform::Windows => "Windows",
        FactoryPlatform::Unsupported => "unsupported",
    };
    Err(anyhow!(
        "{platform_name} world backends require an explicit authenticated platform bootstrap mapping via factory_with_platform_bootstrap"
    ))
}

#[cfg(any(test, not(target_os = "linux")))]
#[cfg_attr(test, allow(dead_code))]
#[derive(Debug, Clone)]
struct CanonicalizedFactoryInput {
    host_carrier: Option<InstallBootstrapContextCarrierV1>,
    platform_bootstrap_mapping: Option<PlatformBootstrapMappingV1>,
    project_path: Option<PathBuf>,
}

#[cfg(any(test, not(target_os = "linux")))]
#[cfg_attr(test, allow(dead_code))]
fn canonicalize_factory_input(
    platform: FactoryPlatform,
    host_carrier: Option<&InstallBootstrapContextCarrierV1>,
    platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    project_path: Option<&Path>,
) -> Result<CanonicalizedFactoryInput> {
    match platform {
        FactoryPlatform::Macos => {
            let host_carrier = host_carrier.ok_or_else(|| {
                anyhow!(
                    "macOS world replay requires an explicit authenticated install bootstrap carrier"
                )
            })?;
            let encoded_host = host_carrier
                .encode()
                .context("macOS world replay host carrier is invalid")?;
            let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host)
                .context("macOS world replay host carrier is not canonical")?;
            let platform_bootstrap_mapping = platform_bootstrap_mapping.ok_or_else(|| {
                anyhow!("macOS world replay requires an explicit platform bootstrap mapping")
            })?;
            let encoded_mapping = platform_bootstrap_mapping
                .encode(&host_carrier)
                .context("macOS world replay platform bootstrap mapping is invalid")?;
            let platform_bootstrap_mapping =
                PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
                    .context("macOS world replay platform bootstrap mapping is not canonical")?;
            match (
                &platform_bootstrap_mapping.platform_instance,
                &platform_bootstrap_mapping.realized_transport,
            ) {
                (
                    PlatformInstanceIdentityV1::Lima { .. },
                    PlatformTransportIdentityV1::Lima { .. },
                ) => {}
                _ => {
                    return Err(anyhow!(
                        "macOS world replay requires a Lima platform bootstrap mapping"
                    ));
                }
            }
            Ok(CanonicalizedFactoryInput {
                host_carrier: Some(host_carrier),
                platform_bootstrap_mapping: Some(platform_bootstrap_mapping),
                project_path: None,
            })
        }
        FactoryPlatform::Windows => {
            let host_carrier = host_carrier.ok_or_else(|| {
                anyhow!(
                    "Windows world replay requires an explicit authenticated install bootstrap carrier"
                )
            })?;
            let encoded_host = host_carrier
                .encode()
                .context("Windows world replay host carrier is invalid")?;
            let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host)
                .context("Windows world replay host carrier is not canonical")?;
            let platform_bootstrap_mapping = platform_bootstrap_mapping.ok_or_else(|| {
                anyhow!("Windows world replay requires an explicit platform bootstrap mapping")
            })?;
            let encoded_mapping = platform_bootstrap_mapping
                .encode(&host_carrier)
                .context("Windows world replay platform bootstrap mapping is invalid")?;
            let platform_bootstrap_mapping =
                PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
                    .context("Windows world replay platform bootstrap mapping is not canonical")?;
            match (
                &platform_bootstrap_mapping.platform_instance,
                &platform_bootstrap_mapping.realized_transport,
            ) {
                (
                    PlatformInstanceIdentityV1::Wsl { .. },
                    PlatformTransportIdentityV1::Wsl { .. },
                ) => {}
                _ => {
                    return Err(anyhow!(
                        "Windows world replay requires a WSL platform bootstrap mapping"
                    ));
                }
            }

            let project_path = project_path
                .ok_or_else(|| anyhow!("Windows world replay requires an explicit project path"))?
                .to_path_buf();
            if project_path.as_os_str().is_empty() {
                return Err(anyhow!(
                    "Windows world replay requires an explicit project path"
                ));
            }
            if project_path.to_str().is_none() {
                return Err(anyhow!(
                    "Windows world replay project path must be valid UTF-8"
                ));
            }

            Ok(CanonicalizedFactoryInput {
                host_carrier: Some(host_carrier),
                platform_bootstrap_mapping: Some(platform_bootstrap_mapping),
                project_path: Some(project_path),
            })
        }
        FactoryPlatform::Unsupported => {
            Err(anyhow!("World backend not implemented for this platform"))
        }
    }
}

#[cfg(target_os = "linux")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    let backend = world::LinuxLocalBackend::new();
    Ok(Arc::new(backend))
}

#[cfg(target_os = "macos")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    fail_closed_platform_factory(FactoryPlatform::Macos)
}

#[cfg(target_os = "windows")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    fail_closed_platform_factory(FactoryPlatform::Windows)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    Err(anyhow::anyhow!(
        "World backend not implemented for this platform"
    ))
}

#[cfg(target_os = "linux")]
pub fn factory_with_platform_bootstrap(
    _host_carrier: Option<&InstallBootstrapContextCarrierV1>,
    _platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    _project_path: Option<&Path>,
) -> Result<Arc<dyn WorldBackend>> {
    factory()
}

#[cfg(target_os = "macos")]
pub fn factory_with_platform_bootstrap(
    host_carrier: Option<&InstallBootstrapContextCarrierV1>,
    platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    project_path: Option<&Path>,
) -> Result<Arc<dyn WorldBackend>> {
    let input = canonicalize_factory_input(
        FactoryPlatform::Macos,
        host_carrier,
        platform_bootstrap_mapping,
        project_path,
    )?;
    let backend = world_mac_lima::MacLimaBackend::new_with_mapping(
        input.host_carrier.expect("typed macOS host carrier"),
        input
            .platform_bootstrap_mapping
            .expect("typed macOS platform bootstrap mapping"),
    )?;
    Ok(Arc::new(backend))
}

#[cfg(target_os = "windows")]
pub fn factory_with_platform_bootstrap(
    host_carrier: Option<&InstallBootstrapContextCarrierV1>,
    platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    project_path: Option<&Path>,
) -> Result<Arc<dyn WorldBackend>> {
    let input = canonicalize_factory_input(
        FactoryPlatform::Windows,
        host_carrier,
        platform_bootstrap_mapping,
        project_path,
    )?;
    let backend = world_windows_wsl::WindowsWslBackend::new_with_mapping(
        input.host_carrier.expect("typed Windows host carrier"),
        input
            .platform_bootstrap_mapping
            .expect("typed Windows platform bootstrap mapping"),
        input.project_path.expect("typed Windows project path"),
    )?;
    Ok(Arc::new(backend))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn factory_with_platform_bootstrap(
    host_carrier: Option<&InstallBootstrapContextCarrierV1>,
    platform_bootstrap_mapping: Option<&PlatformBootstrapMappingV1>,
    project_path: Option<&Path>,
) -> Result<Arc<dyn WorldBackend>> {
    let _ = canonicalize_factory_input(
        FactoryPlatform::Unsupported,
        host_carrier,
        platform_bootstrap_mapping,
        project_path,
    )?;
    unreachable!("unsupported typed world backend factory always returns an error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        PlatformTransportIdentityV1,
    };

    fn unix_host_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/tmp/substrate", "alice", 1000)
                .expect("host context"),
        )
        .expect("host carrier")
    }

    fn windows_host_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_windows("C:/Substrate", "ACME\\Alice", "S-1-5-21-1000")
                .expect("host context"),
        )
        .expect("host carrier")
    }

    fn lima_mapping(host_carrier: &InstallBootstrapContextCarrierV1) -> PlatformBootstrapMappingV1 {
        PlatformBootstrapMappingV1::new_lima(
            host_carrier,
            "substrate",
            "0123456789abcdef0123456789abcdef",
            "/Users/alice/.lima",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/tmp/substrate/sock/agent.sock",
            "/run/substrate.sock",
        )
        .expect("mapping")
    }

    fn wsl_mapping(host_carrier: &InstallBootstrapContextCarrierV1) -> PlatformBootstrapMappingV1 {
        PlatformBootstrapMappingV1::new_wsl(
            host_carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            "C:/Users/Alice/AppData/Local/Substrate/forwarder/abcdef",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            r"\\.\pipe\substrate-agent",
            "/run/substrate.sock",
        )
        .expect("mapping")
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn factory_returns_backend() {
        assert!(factory().is_ok());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn zero_arg_public_factory_fails_closed_for_macos() {
        let err = match factory() {
            Ok(_) => panic!("macOS zero-arg factory must fail closed"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("factory_with_platform_bootstrap"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn zero_arg_public_factory_fails_closed_for_windows() {
        let err = match factory() {
            Ok(_) => panic!("Windows zero-arg factory must fail closed"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("factory_with_platform_bootstrap"));
    }

    #[test]
    fn zero_arg_factory_fails_closed_for_macos() {
        let err = match fail_closed_platform_factory(FactoryPlatform::Macos) {
            Ok(_) => panic!("macOS zero-arg factory must fail closed"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("factory_with_platform_bootstrap"));
    }

    #[test]
    fn zero_arg_factory_fails_closed_for_windows() {
        let err = match fail_closed_platform_factory(FactoryPlatform::Windows) {
            Ok(_) => panic!("Windows zero-arg factory must fail closed"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("factory_with_platform_bootstrap"));
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    #[test]
    fn factory_errs_on_unsupported() {
        assert!(factory().is_err());
    }

    #[test]
    fn typed_factory_input_requires_explicit_host_carrier_for_macos() {
        let err = canonicalize_factory_input(
            FactoryPlatform::Macos,
            None,
            Some(&wsl_mapping(&windows_host_carrier())),
            None,
        )
        .expect_err("macOS typed factory should require an authenticated host carrier");

        assert!(err
            .to_string()
            .contains("authenticated install bootstrap carrier"));
    }

    #[test]
    fn typed_factory_input_requires_explicit_mapping_for_windows() {
        let err = canonicalize_factory_input(
            FactoryPlatform::Windows,
            Some(&windows_host_carrier()),
            None,
            Some(std::path::Path::new("C:/repo")),
        )
        .expect_err("Windows typed factory should require an explicit mapping");

        assert!(err.to_string().contains("platform bootstrap mapping"));
    }

    #[test]
    fn typed_factory_input_rejects_wrong_platform_for_macos() {
        let windows_host = windows_host_carrier();
        let err = canonicalize_factory_input(
            FactoryPlatform::Macos,
            Some(&windows_host),
            Some(&wsl_mapping(&windows_host)),
            None,
        )
        .expect_err("macOS typed factory should reject WSL mappings");

        assert!(err.to_string().contains("Lima"));
    }

    #[test]
    fn typed_factory_input_rejects_wrong_transport_for_macos() {
        let unix_host = unix_host_carrier();
        let mut mapping = lima_mapping(&unix_host);
        mapping.realized_transport = PlatformTransportIdentityV1::Wsl {
            pipe_path: r"\\.\pipe\substrate-agent".to_string(),
            guest_socket: "/run/substrate.sock".to_string(),
        };

        let err = canonicalize_factory_input(
            FactoryPlatform::Macos,
            Some(&unix_host),
            Some(&mapping),
            None,
        )
        .expect_err("macOS typed factory should reject non-Lima transport");

        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn typed_factory_input_requires_explicit_project_path_for_windows() {
        let windows_host = windows_host_carrier();
        let err = canonicalize_factory_input(
            FactoryPlatform::Windows,
            Some(&windows_host),
            Some(&wsl_mapping(&windows_host)),
            None,
        )
        .expect_err("Windows typed factory should require an explicit project path");

        assert!(err.to_string().contains("project path"));
    }

    #[test]
    fn typed_factory_input_rejects_wrong_commitment_for_macos() {
        let unix_host = unix_host_carrier();
        let other_host = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/tmp/substrate-b", "bob", 1001)
                .expect("other host context"),
        )
        .expect("other host carrier");

        let err = canonicalize_factory_input(
            FactoryPlatform::Macos,
            Some(&unix_host),
            Some(&lima_mapping(&other_host)),
            None,
        )
        .expect_err("macOS typed factory should reject host/mapping commitment mismatches");

        let rendered = format!("{err:#}");
        assert!(rendered.contains("commitment mismatch"));
    }
}
