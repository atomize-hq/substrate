//! Structural gateway custody for service-wrapper tests. No execution admission is minted.
use super::*;
use std::sync::Weak;

pub(crate) struct RouteFixture {
    pub(crate) lease: Arc<OrdinaryGatewayRuntimeLeaseV1>,
    instance: Arc<OrdinaryGatewayInstanceV1>,
    manager: Arc<GatewayRuntimeManager>,
    process: Arc<Mutex<ManagedGatewayProcess>>,
    manifest: PathBuf,
    exclusion: Weak<HeldNonE3PrivilegedChildLeaseV1>,
}
impl RouteFixture {
    pub(crate) fn new(exclusion: &Arc<E3PrivilegedChildExclusionV1>) -> Self {
        let manager = Arc::new(GatewayRuntimeManager::new(Arc::clone(exclusion)));
        let child = Command::new("/bin/sleep").arg("30").spawn().unwrap();
        let pid = child.id();
        let process = Arc::new(Mutex::new(ManagedGatewayProcess::Child(child)));
        let runtime_id = format!("service_route_disposal_{}", uuid::Uuid::now_v7());
        let backend_id = "cli:codex".to_string();
        let runtime_dir = runtime_dir_for_world(&runtime_id, &backend_id);
        fs::create_dir_all(&runtime_dir).unwrap();
        let config_path = runtime_dir.join("nonsecret-config.toml");
        fs::write(&config_path, "component = true\n").unwrap();
        let manifest = manifest_path_for_world(&runtime_id, &backend_id);
        let lease = Arc::new(exclusion.acquire_non_e3_child().unwrap());
        let exclusion = Arc::downgrade(&lease);
        let runtime = ManagedGatewayRuntime {
            world_id: runtime_id.clone(),
            backend_id: backend_id.clone(),
            port: 32123,
            runtime_dir,
            config_path,
            manifest_path: manifest.clone(),
            pid_start_time_ticks: read_pid_start_time_ticks(pid).unwrap(),
            process: Arc::clone(&process),
            state: Arc::new(RwLock::new(GatewayRuntimeState::Ready)),
            privileged_child_exclusion_lease: lease,
        };
        runtime.persist_manifest().unwrap();
        manager.insert_runtime(runtime);
        let instance = bind_ordinary_gateway_fixture_owner_for_test(&manager, &runtime_id);
        let lease = Arc::new(OrdinaryGatewayRuntimeLeaseV1 {
            runtime_id: runtime_id.clone(),
            backend_id: backend_id.clone(),
            manager: Arc::clone(&manager),
            owner: Arc::new(OrdinaryGatewayLeaseOwnerV1 {
                runtime_id,
                backend_id,
                closing: std::sync::atomic::AtomicBool::new(false),
                instance: Mutex::new(Some(Arc::clone(&instance))),
            }),
        });
        Self {
            lease,
            instance,
            manager,
            process,
            manifest,
            exclusion,
        }
    }

    pub(crate) async fn assert_disposed(&self) {
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                if self.instance.resources.lock().unwrap().is_none()
                    && self.instance.settlement.lock().unwrap().is_some()
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("gateway disposal must precede held native ACK release");
        assert!(self
            .lease
            .owner
            .closing
            .load(std::sync::atomic::Ordering::Acquire));
        assert!(self
            .instance
            .closing
            .load(std::sync::atomic::Ordering::Acquire));
        assert!(!self
            .manager
            .runtimes
            .lock()
            .unwrap()
            .contains_key(&self.lease.runtime_id));
        assert!(!self.manifest.exists());
        assert!(
            self.exclusion.upgrade().is_none(),
            "actual gateway resource lease released with metadata clones retained"
        );
        let lifecycle = self
            .manager
            .ordinary_lifecycle_for_owner(&self.lease.runtime_id)
            .unwrap();
        assert!(lifecycle.ordinary.lock().unwrap().is_none());
        let observed = self
            .instance
            .settlement
            .lock()
            .unwrap()
            .clone()
            .unwrap()
            .unwrap();
        use std::os::unix::process::ExitStatusExt as _;
        assert_eq!(observed.1.signal(), Some(libc::SIGKILL));
        let mut process = self.process.lock().unwrap();
        let ManagedGatewayProcess::Child(child) = &mut *process else {
            panic!("actual Child replaced")
        };
        assert_eq!(child.id(), observed.0);
        assert_eq!(child.try_wait().unwrap(), Some(observed.1));
    }
}
impl Drop for RouteFixture {
    fn drop(&mut self) {
        let mut process = self.process.lock().unwrap_or_else(|e| e.into_inner());
        if let ManagedGatewayProcess::Child(child) = &mut *process {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}
