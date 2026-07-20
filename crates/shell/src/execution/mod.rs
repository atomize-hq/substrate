#[doc(hidden)]
pub mod agent_dev_support;
pub mod agent_events;
pub(crate) mod agent_inventory;
pub(crate) mod agent_runtime;
mod agents_cmd;
mod auto_sync;
mod cli;
mod config_cmd;
pub(crate) mod config_model;
mod env_scripts;
mod home_bootstrap;
#[cfg(any(target_os = "linux", test))]
pub(crate) mod host_inbox_materialization;
pub(crate) mod install_bootstrap;
mod invocation;
pub mod lock;
mod manager;
pub mod manager_init;
pub(crate) mod orchestrator_world_dispatch;
mod platform;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod platform_world;
mod policy_cmd;
pub(crate) mod policy_model;
pub(crate) mod policy_snapshot;
pub(crate) mod prompt_fulfillment;
mod pty;
pub mod repl_persistent_session;
mod routing;
mod settings;
pub(crate) use settings::resolve_world_root;
pub(crate) use settings::WorldRootSettings;
pub mod shim_deploy;
#[cfg(target_os = "linux")]
pub(crate) mod socket_activation; // Made public for integration tests
mod value_parse;
mod workspace;
mod workspace_cmd;

pub(crate) use agents_cmd::handle_agent_command;
pub(crate) use agents_cmd::handle_agents_command;
pub(crate) use auto_sync::run_auto_sync_if_enabled;
pub use cli::*;
pub(crate) use config_cmd::handle_config_command;
pub(crate) use env_scripts::{
    env_sh_path, export_runtime_config_env, write_env_sh, write_env_sh_at,
};
pub use invocation::{needs_shell, ShellConfig, ShellMode};
pub(crate) use policy_cmd::handle_policy_command;
#[cfg(unix)]
pub(crate) use pty::get_terminal_size;
pub(crate) use pty::MinimalTerminalGuard;
pub use routing::*;
pub(crate) use workspace::find_workspace_root;
pub(crate) use workspace_cmd::handle_workspace_command;

pub(crate) use manager::{
    configure_child_shell_env, configure_manager_init, current_platform, log_manager_init_event,
    manager_manifest_base_path, write_manager_env_script,
};
pub(crate) use platform::{
    handle_health_command, handle_host_command, handle_world_command, update_world_env,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use platform_world as pw;

#[cfg(test)]
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
#[cfg(test)]
use std::ffi::{OsStr, OsString};
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
static WORLD_ENV_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn world_env_guard() -> ReentrantMutexGuard<'static, ()> {
    WORLD_ENV_LOCK
        .get_or_init(|| ReentrantMutex::new(()))
        .lock()
}

#[cfg(test)]
pub(crate) struct WorldSocketTestGuard {
    previous: Option<OsString>,
    _lock: ReentrantMutexGuard<'static, ()>,
}

#[cfg(test)]
impl WorldSocketTestGuard {
    pub(crate) fn set(value: impl AsRef<OsStr>) -> Self {
        Self::set_optional(Some(value.as_ref()))
    }

    pub(crate) fn remove() -> Self {
        Self::set_optional(None)
    }

    pub(crate) fn set_optional(value: Option<&OsStr>) -> Self {
        let lock = world_env_guard();
        let previous = std::env::var_os("SUBSTRATE_WORLD_SOCKET");
        match value {
            Some(value) => std::env::set_var("SUBSTRATE_WORLD_SOCKET", value),
            None => std::env::remove_var("SUBSTRATE_WORLD_SOCKET"),
        }
        Self {
            previous,
            _lock: lock,
        }
    }
}

#[cfg(test)]
impl Drop for WorldSocketTestGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(previous) => std::env::set_var("SUBSTRATE_WORLD_SOCKET", previous),
            None => std::env::remove_var("SUBSTRATE_WORLD_SOCKET"),
        }
    }
}

#[cfg(test)]
mod world_socket_test_guard_tests {
    use super::WorldSocketTestGuard;
    use std::ffi::{OsStr, OsString};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn world_socket_test_guard_restores_prior_absence() {
        let outer = WorldSocketTestGuard::remove();
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-inner.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[cfg(unix)]
    #[test]
    fn world_socket_test_guard_restores_exact_non_unicode_prior_value() {
        use std::os::unix::ffi::OsStringExt;

        let prior = OsString::from_vec(b"/tmp/f0-prior-\xff.sock".to_vec());
        let outer = WorldSocketTestGuard::set(prior.as_os_str());
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), Some(prior));
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_restores_after_panic_and_allows_reacquisition() {
        let outer = WorldSocketTestGuard::remove();
        let panic_result = std::panic::catch_unwind(|| {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-panic.sock"));
            panic!("intentional world-socket guard panic");
        });
        assert!(panic_result.is_err());
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);

        {
            let _later = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-reacquired.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-reacquired.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_nested_acquisition_restores_in_stack_order() {
        let outer = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-outer.sock"));
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-inner.sock"))
            );
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::from("/tmp/f0-outer.sock"))
        );
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_blocks_competitor_until_cleanup_and_restoration() {
        let cleanup_complete = Arc::new(AtomicBool::new(false));
        let (owner_ready_tx, owner_ready_rx) = mpsc::channel();
        let (release_owner_tx, release_owner_rx) = mpsc::channel();
        let owner_cleanup = Arc::clone(&cleanup_complete);
        let owner = std::thread::spawn(move || {
            let guard = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-owner.sock"));
            owner_ready_tx
                .send(guard.previous.clone())
                .expect("signal owner ready");
            release_owner_rx.recv().expect("release owner");
            owner_cleanup.store(true, Ordering::SeqCst);
            drop(guard);
        });
        let original = owner_ready_rx.recv().expect("owner prior state");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let competitor_cleanup = Arc::clone(&cleanup_complete);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("signal competitor");
            let guard = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-competitor.sock"));
            assert!(
                competitor_cleanup.load(Ordering::SeqCst),
                "competitor acquired before owner cleanup"
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-competitor.sock"))
            );
            competitor_acquired_tx
                .send(guard.previous.clone())
                .expect("signal competitor acquisition");
            drop(guard);
        });

        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while owner held the guard"
        );
        release_owner_tx.send(()).expect("release owner");
        assert_eq!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("competitor eventually acquired"),
            original
        );
        owner.join().expect("owner thread");
        competitor.join().expect("competitor thread");

        let probe = WorldSocketTestGuard::remove();
        assert_eq!(probe.previous, original);
        drop(probe);
    }
}
