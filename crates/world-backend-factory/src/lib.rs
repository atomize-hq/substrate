use anyhow::Result;
use std::sync::Arc;
use world_api::WorldBackend;

/// Return a platform-appropriate world backend instance.
/// - macOS: world-mac-lima::MacLimaBackend
/// - Linux: world::LinuxLocalBackend
/// - Windows/other: not yet implemented (Phase 5)
pub fn factory() -> Result<Arc<dyn WorldBackend>> {
    #[cfg(target_os = "linux")]
    {
        let be = world::LinuxLocalBackend::new();
        Ok(Arc::new(be))
    }

    #[cfg(target_os = "macos")]
    {
        let be = world_mac_lima::MacLimaBackend::new()?;
        Ok(Arc::new(be))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(anyhow::anyhow!(
            "World backend not implemented for this platform"
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_compiles_and_runs() {
        // On supported platforms, factory should return Ok
        // On others, it should return Err (Phase 5 TODO)
        let _ = factory();
    }
}
