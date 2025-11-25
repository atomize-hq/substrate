use super::*;
use anyhow::anyhow;
use serial_test::serial;
use std::env;

// Linux world initialization paths
#[cfg(target_os = "linux")]
mod linux_world_tests {
    use super::*;
    use std::env;

    fn clear_env() {
        env::remove_var("SUBSTRATE_WORLD");
        env::remove_var("SUBSTRATE_WORLD_ID");
        env::remove_var("SUBSTRATE_TEST_LOCAL_WORLD_ID");
    }

    #[test]
    #[serial]
    fn agent_probe_enables_world() {
        clear_env();
        let outcome = init_linux_world_with_probe(false, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Agent);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }

    #[test]
    #[serial]
    fn fallback_uses_local_backend_stub() {
        clear_env();
        env::set_var("SUBSTRATE_TEST_LOCAL_WORLD_ID", "wld_test_stub");
        let outcome = init_linux_world_with_probe(false, || Err(anyhow!("no agent")));
        assert_eq!(outcome, LinuxWorldInit::LocalBackend);
        assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_test_stub");
    }

    #[test]
    #[serial]
    fn disabled_skips_initialization() {
        clear_env();
        let outcome = init_linux_world_with_probe(true, || Ok(()));
        assert_eq!(outcome, LinuxWorldInit::Disabled);
        assert!(env::var("SUBSTRATE_WORLD").is_err());
        assert!(env::var("SUBSTRATE_WORLD_ID").is_err());
    }
}
