use anyhow::Result;
use std::sync::Arc;
use world_api::WorldBackend;

#[cfg(target_os = "linux")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    let backend = world::LinuxLocalBackend::new();
    Ok(Arc::new(backend))
}

#[cfg(target_os = "macos")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    let backend = world_mac_lima::MacLimaBackend::new()?;
    Ok(Arc::new(backend))
}

#[cfg(target_os = "windows")]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    let backend = world_windows_wsl::WindowsWslBackend::new()?;
    Ok(Arc::new(backend))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    Err(anyhow::anyhow!(
        "World backend not implemented for this platform"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(not(target_os = "windows"), ignore)]
    #[test]
    fn factory_returns_backend() {
        assert!(factory().is_ok());
    }

    #[cfg_attr(
        any(target_os = "linux", target_os = "macos", target_os = "windows"),
        ignore
    )]
    #[test]
    fn factory_errs_on_unsupported() {
        assert!(factory().is_err());
    }
}
