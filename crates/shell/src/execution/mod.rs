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
// Deliberately non-poisoning: panic unwinding drops the guard, restores the exact
// authority-environment snapshot, and permits a later test to acquire the boundary.
static WORLD_ENV_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn world_env_guard() -> ReentrantMutexGuard<'static, ()> {
    WORLD_ENV_LOCK
        .get_or_init(|| ReentrantMutex::new(()))
        .lock()
}

#[cfg(test)]
pub(crate) struct AuthorityEnvTestGuard {
    previous_home: Option<OsString>,
    previous_world_socket: Option<OsString>,
    _lock: ReentrantMutexGuard<'static, ()>,
}

#[cfg(test)]
impl AuthorityEnvTestGuard {
    pub(crate) fn preserve() -> Self {
        Self::apply(None, None)
    }

    pub(crate) fn set_home(value: impl AsRef<OsStr>) -> Self {
        Self::apply(Some(Some(value.as_ref())), None)
    }

    pub(crate) fn remove_home() -> Self {
        Self::apply(Some(None), None)
    }

    pub(crate) fn set_world_socket(value: impl AsRef<OsStr>) -> Self {
        Self::apply(None, Some(Some(value.as_ref())))
    }

    pub(crate) fn remove_world_socket() -> Self {
        Self::apply(None, Some(None))
    }

    pub(crate) fn set_both(home: impl AsRef<OsStr>, world_socket: impl AsRef<OsStr>) -> Self {
        Self::apply(Some(Some(home.as_ref())), Some(Some(world_socket.as_ref())))
    }

    pub(crate) fn remove_both() -> Self {
        Self::apply(Some(None), Some(None))
    }

    pub(crate) fn install_home(&self, value: impl AsRef<OsStr>) {
        std::env::set_var("SUBSTRATE_HOME", value);
    }

    fn apply(home: Option<Option<&OsStr>>, world_socket: Option<Option<&OsStr>>) -> Self {
        let lock = world_env_guard();
        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let previous_world_socket = std::env::var_os("SUBSTRATE_WORLD_SOCKET");
        Self::apply_value("SUBSTRATE_HOME", home);
        Self::apply_value("SUBSTRATE_WORLD_SOCKET", world_socket);
        Self {
            previous_home,
            previous_world_socket,
            _lock: lock,
        }
    }

    fn apply_value(key: &str, value: Option<Option<&OsStr>>) {
        match value {
            None => {}
            Some(Some(value)) => std::env::set_var(key, value),
            Some(None) => std::env::remove_var(key),
        }
    }

    fn restore_value(key: &str, value: Option<&OsStr>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

#[cfg(test)]
impl Drop for AuthorityEnvTestGuard {
    fn drop(&mut self) {
        Self::restore_value("SUBSTRATE_HOME", self.previous_home.as_deref());
        Self::restore_value(
            "SUBSTRATE_WORLD_SOCKET",
            self.previous_world_socket.as_deref(),
        );
    }
}

#[cfg(test)]
pub(crate) struct WorldSocketTestGuard {
    previous: Option<OsString>,
    _authority: AuthorityEnvTestGuard,
}

#[cfg(test)]
pub(crate) struct AuthorityEnvTestTempDir {
    temp: Option<tempfile::TempDir>,
    authority: Option<AuthorityEnvTestGuard>,
}

#[cfg(test)]
impl AuthorityEnvTestTempDir {
    pub(crate) fn new(temp: tempfile::TempDir) -> Self {
        Self {
            temp: Some(temp),
            authority: Some(AuthorityEnvTestGuard::preserve()),
        }
    }

    pub(crate) fn install_as_home(&self) {
        self.authority
            .as_ref()
            .expect("authority tempdir guard")
            .install_home(self.path());
    }
}

#[cfg(test)]
impl std::ops::Deref for AuthorityEnvTestTempDir {
    type Target = tempfile::TempDir;

    fn deref(&self) -> &Self::Target {
        self.temp.as_ref().expect("authority tempdir")
    }
}

#[cfg(test)]
impl Drop for AuthorityEnvTestTempDir {
    fn drop(&mut self) {
        drop(self.temp.take());
        drop(self.authority.take());
    }
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
        let authority = match value {
            Some(value) => AuthorityEnvTestGuard::set_world_socket(value),
            None => AuthorityEnvTestGuard::remove_world_socket(),
        };
        let previous = authority.previous_world_socket.clone();
        Self {
            previous,
            _authority: authority,
        }
    }
}

#[cfg(test)]
mod world_socket_test_guard_tests {
    use super::{AuthorityEnvTestGuard, AuthorityEnvTestTempDir, WorldSocketTestGuard};
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

    #[test]
    fn authority_env_test_guard_restores_home_and_socket_as_one_snapshot() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-home"),
                OsStr::new("/tmp/f0a-world.sock"),
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0a-world.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_preserve_restores_direct_mutations() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::preserve();
            std::env::set_var("SUBSTRATE_HOME", "/tmp/f0a-direct-home");
            std::env::set_var("SUBSTRATE_WORLD_SOCKET", "/tmp/f0a-direct.sock");
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_preserves_unicode_and_empty_values() {
        let outer =
            AuthorityEnvTestGuard::set_both(OsStr::new("/tmp/authority-雪"), OsStr::new(""));
        {
            let _inner = AuthorityEnvTestGuard::remove_both();
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/authority-雪"))
        );
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::new())
        );
        drop(outer);
    }

    #[cfg(unix)]
    #[test]
    fn authority_env_test_guard_restores_exact_non_unicode_pair() {
        use std::os::unix::ffi::OsStringExt;

        let prior_home = OsString::from_vec(b"/tmp/f0a-home-\xff".to_vec());
        let prior_socket = OsString::from_vec(b"/tmp/f0-socket-\xfe".to_vec());
        let outer = AuthorityEnvTestGuard::set_both(&prior_home, &prior_socket);
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-inner-home"),
                OsStr::new("/tmp/f0-inner.sock"),
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), Some(prior_home));
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(prior_socket)
        );
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_nested_home_override_preserves_outer_pair() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _middle = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-outer-home"),
                OsStr::new("/tmp/f0-outer.sock"),
            );
            {
                let _inner = AuthorityEnvTestGuard::set_home(OsStr::new("/tmp/f0a-inner-home"));
                assert_eq!(
                    std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                    Some(OsString::from("/tmp/f0-outer.sock"))
                );
            }
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-outer-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-outer.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_restores_pair_after_panic_without_poisoning() {
        let outer = AuthorityEnvTestGuard::set_both(
            OsStr::new("/tmp/f0a-prior-home"),
            OsStr::new("/tmp/f0-prior.sock"),
        );
        let panic_result = std::panic::catch_unwind(|| {
            let _inner = AuthorityEnvTestGuard::remove_both();
            panic!("intentional authority-environment guard panic");
        });
        assert!(panic_result.is_err());
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/f0a-prior-home"))
        );
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::from("/tmp/f0-prior.sock"))
        );

        {
            let _later = AuthorityEnvTestGuard::remove_home();
            assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/f0a-prior-home"))
        );
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_panic_releases_boundary_for_cross_thread_reacquisition() {
        let (prior_tx, prior_rx) = mpsc::channel();
        let owner = std::thread::spawn(move || {
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-panicking-owner-home"),
                OsStr::new("/tmp/f0-panicking-owner.sock"),
            );
            prior_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("send panicking owner prior pair");
            panic!("intentional cross-thread authority-environment panic");
        });
        let prior = prior_rx.recv().expect("receive panicking owner prior pair");
        assert!(owner.join().is_err());

        let later = std::thread::spawn(move || {
            let _guard = AuthorityEnvTestGuard::preserve();
            assert_eq!(std::env::var_os("SUBSTRATE_HOME"), prior.0);
            assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), prior.1);
        });
        later
            .join()
            .expect("cross-thread reacquisition after owner panic");
    }

    #[cfg(unix)]
    #[test]
    fn authority_env_test_guard_bounds_subprocess_inheritance_through_wait_and_output() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-child-home"),
                OsStr::new("/tmp/f0-child.sock"),
            );
            let output = std::process::Command::new("sh")
                .args([
                    "-c",
                    "printf '%s\\n%s\\n' \"$SUBSTRATE_HOME\" \"$SUBSTRATE_WORLD_SOCKET\"",
                ])
                .output()
                .expect("run authority environment child");
            assert!(output.status.success());
            assert_eq!(output.stdout, b"/tmp/f0a-child-home\n/tmp/f0-child.sock\n");
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_tempdir_cleanup_precedes_unlock_during_panic() {
        let fixture = AuthorityEnvTestTempDir::new(tempfile::tempdir().expect("authority tempdir"));
        let fixture_path = fixture.path().to_path_buf();
        fixture.install_as_home();
        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let path_for_competitor = fixture_path.clone();
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("competitor started");
            let _guard = AuthorityEnvTestGuard::preserve();
            assert!(
                !path_for_competitor.exists(),
                "competitor acquired before authority tempdir cleanup"
            );
            competitor_acquired_tx
                .send(())
                .expect("competitor acquired");
        });
        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while authority tempdir boundary was live"
        );

        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _fixture = fixture;
            panic!("intentional authority tempdir panic");
        }));
        assert!(panic_result.is_err());
        competitor_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("competitor acquired after authority tempdir cleanup");
        competitor.join().expect("authority tempdir competitor");
        assert!(!fixture_path.exists());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn authority_env_test_guard_holds_boundary_through_aborted_server_cleanup() {
        struct ServerTerminationFlag(Arc<AtomicBool>);

        impl Drop for ServerTerminationFlag {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let server_terminated = Arc::new(AtomicBool::new(false));
        let fixture_cleaned = Arc::new(AtomicBool::new(false));
        let boundary = AuthorityEnvTestGuard::set_both(
            OsStr::new("/tmp/f0a-async-owner-home"),
            OsStr::new("/tmp/f0-async-owner.sock"),
        );
        let socket_home = tempfile::tempdir().expect("async socket tempdir");
        let socket_path = socket_home.path().join("world.sock");
        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind async socket");
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let termination_for_server = Arc::clone(&server_terminated);
        let server = tokio::spawn(async move {
            let _termination = ServerTerminationFlag(termination_for_server);
            let _listener = listener;
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
        });
        started_rx.await.expect("async server started");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let termination_for_competitor = Arc::clone(&server_terminated);
        let cleanup_for_competitor = Arc::clone(&fixture_cleaned);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("competitor started");
            let _guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-async-competitor-home"),
                OsStr::new("/tmp/f0-async-competitor.sock"),
            );
            assert!(termination_for_competitor.load(Ordering::SeqCst));
            assert!(cleanup_for_competitor.load(Ordering::SeqCst));
            competitor_acquired_tx
                .send(())
                .expect("competitor acquisition signal");
        });
        competitor_started_rx.recv().expect("competitor started");

        server.abort();
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired after abort but before awaited server termination"
        );
        match server.await {
            Err(error) if error.is_cancelled() => {}
            Ok(()) => panic!("async server completed normally instead of confirming cancellation"),
            Err(error) => panic!("async server join failed unexpectedly: {error}"),
        }
        assert!(server_terminated.load(Ordering::SeqCst));
        assert!(
            std::os::unix::net::UnixStream::connect(&socket_path).is_err(),
            "terminated server still accepted socket connections"
        );
        drop(socket_home);
        assert!(!socket_path.exists());
        fixture_cleaned.store(true, Ordering::SeqCst);
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired before authority restoration and unlock"
        );
        drop(boundary);
        competitor_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("competitor acquired after async cleanup and unlock");
        competitor.join().expect("async cleanup competitor");
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

    #[test]
    fn authority_env_test_guard_blocks_competitor_until_pair_cleanup_and_restoration() {
        let cleanup_complete = Arc::new(AtomicBool::new(false));
        let (owner_ready_tx, owner_ready_rx) = mpsc::channel();
        let (release_owner_tx, release_owner_rx) = mpsc::channel();
        let owner_cleanup = Arc::clone(&cleanup_complete);
        let owner = std::thread::spawn(move || {
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-owner-home"),
                OsStr::new("/tmp/f0-owner.sock"),
            );
            owner_ready_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("signal authority owner ready");
            release_owner_rx.recv().expect("release authority owner");
            owner_cleanup.store(true, Ordering::SeqCst);
            drop(guard);
        });
        let original = owner_ready_rx.recv().expect("owner prior authority pair");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let competitor_cleanup = Arc::clone(&cleanup_complete);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("signal competitor");
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-competitor-home"),
                OsStr::new("/tmp/f0-competitor.sock"),
            );
            assert!(
                competitor_cleanup.load(Ordering::SeqCst),
                "competitor acquired before authority owner cleanup"
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-competitor-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-competitor.sock"))
            );
            competitor_acquired_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("signal authority competitor acquisition");
            drop(guard);
        });

        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while owner held the authority boundary"
        );
        release_owner_tx.send(()).expect("release authority owner");
        assert_eq!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("authority competitor eventually acquired"),
            original
        );
        owner.join().expect("authority owner thread");
        competitor.join().expect("authority competitor thread");

        let probe = AuthorityEnvTestGuard::preserve();
        assert_eq!(
            (
                probe.previous_home.clone(),
                probe.previous_world_socket.clone()
            ),
            original
        );
        drop(probe);
    }
}
