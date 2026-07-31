// Mac backend typed pre-R3 contract validation example

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("mac_backend_smoke example runs only on macOS; skipping.");
}

#[cfg(target_os = "macos")]
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};
#[cfg(target_os = "macos")]
use world_mac_lima::MacLimaBackend;

#[cfg(target_os = "macos")]
fn parse_args() -> Result<(
    InstallBootstrapContextCarrierV1,
    PlatformBootstrapMappingV1,
    PathBuf,
)> {
    let mut encoded_host_carrier = None;
    let mut encoded_mapping = None;
    let mut project_dir = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--install-bootstrap-context-v1" => {
                let value = args
                    .next()
                    .context("missing value for --install-bootstrap-context-v1")?;
                if encoded_host_carrier.replace(value).is_some() {
                    anyhow::bail!("duplicate --install-bootstrap-context-v1");
                }
            }
            "--platform-bootstrap-mapping-v1" => {
                let value = args
                    .next()
                    .context("missing value for --platform-bootstrap-mapping-v1")?;
                if encoded_mapping.replace(value).is_some() {
                    anyhow::bail!("duplicate --platform-bootstrap-mapping-v1");
                }
            }
            "--project-dir" => {
                let value = args.next().context("missing value for --project-dir")?;
                if project_dir.replace(value).is_some() {
                    anyhow::bail!("duplicate --project-dir");
                }
            }
            _ => anyhow::bail!(
                "usage: mac_backend_smoke --install-bootstrap-context-v1 <carrier> --platform-bootstrap-mapping-v1 <mapping> --project-dir <absolute-path>"
            ),
        }
    }

    let encoded_host_carrier =
        encoded_host_carrier.context("missing --install-bootstrap-context-v1")?;
    let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host_carrier)
        .map_err(|err| anyhow::anyhow!("invalid --install-bootstrap-context-v1: {err}"))?;

    let encoded_mapping = encoded_mapping.context("missing --platform-bootstrap-mapping-v1")?;
    let mapping = PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
        .map_err(|err| anyhow::anyhow!("invalid --platform-bootstrap-mapping-v1: {err}"))?;

    let project_dir = PathBuf::from(project_dir.context("missing --project-dir")?);
    if !project_dir.is_absolute() {
        anyhow::bail!("--project-dir must be an absolute path");
    }

    Ok((host_carrier, mapping, project_dir))
}

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    let (host_carrier, mapping, project_dir) = parse_args()?;
    let project_label = project_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("<root>");

    let _backend = MacLimaBackend::new_with_mapping(host_carrier, mapping)?;
    println!("✓ Typed pre-R3 MacLimaBackend contract validation passed (project={project_label})");

    Ok(())
}
