#![cfg(target_os = "linux")]

use std::fs::{File, OpenOptions};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

use base64::Engine as _;
use std::path::PathBuf;
use std::sync::Arc;

use config_projection::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

struct TestParent {
    authority: PathBuf,
    lock: PathBuf,
}

impl ConfigProjectionHsaAuthorityV1 for TestParent {
    fn with_locked_parent(
        &self,
        operation: &mut dyn for<'fd> FnMut(
            BorrowedFd<'fd>,
        ) -> Result<(), ConfigProjectionFailureV1>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.lock)
            .map_err(|_| ConfigProjectionFailureV1::Conflict)?;
        if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock), libc::LOCK_EX) } != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        let authority = File::open(&self.authority)
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let result = operation(authority.as_fd());
        let unlock = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock), libc::LOCK_UN) };
        if unlock != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        result
    }
}

fn id(prefix: &str) -> String {
    format!("{prefix}{}", Uuid::now_v7())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn registry_fixture() -> (
    tempfile::TempDir,
    ConfigProjectionRegistryV1,
    ConfigProjectionStoreV1,
) {
    let temp = tempfile::tempdir().expect("temp root");
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700))
        .expect("secure temp root");
    let authority = temp.path().join("authority-v1");
    std::fs::create_dir(&authority).expect("authority root");
    std::fs::set_permissions(&authority, std::fs::Permissions::from_mode(0o700))
        .expect("secure authority root");
    let lock = temp.path().join("parent.lock");
    File::create(&lock).expect("parent lock");
    std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o600))
        .expect("secure parent lock");
    let registry = ConfigProjectionRegistryV1::open(Arc::new(TestParent { authority, lock }))
        .expect("open registry");
    let store = registry
        .recover(None)
        .map(|readback| readback.store)
        .expect("initialize store");
    (temp, registry, store)
}

fn effective_source(store: &ConfigProjectionStoreV1) -> EffectiveSubstrateConfigSourceV1 {
    let keys = [
        "llm.enabled",
        "llm.gateway.enabled",
        "llm.gateway.mode",
        "llm.routing.default_backend",
        "agents.enabled",
        "agents.defaults.execution.scope",
        "agents.defaults.cli.mode",
        "world.enabled",
    ];
    let mut source = EffectiveSubstrateConfigSourceV1 {
        schema_version: 1,
        authority_store_id: store.authority_store_id.clone(),
        accepted_home: store.accepted_home.clone(),
        workspace_root: store.accepted_home.clone(),
        values: E3EffectiveConfigInputV1 {
            llm_enabled: true,
            agents_enabled: true,
            world_enabled: true,
            default_execution_scope: "world".to_string(),
            default_cli_mode: "persistent".to_string(),
            managed_gateway_enabled: true,
            managed_gateway_mode: "in_world".to_string(),
            default_backend_id: "cli:codex-world".to_string(),
        },
        ordered_explain_origins: keys
            .into_iter()
            .map(|key| E3ConfigExplainOriginV1 {
                key: key.to_string(),
                source_kind: E3ConfigExplainOriginKindV1::Default,
                source_location: None,
            })
            .collect(),
        source_revision: String::new(),
        source_hash: String::new(),
    };
    let mut revision = serde_json::to_value(&source).expect("serialize revision");
    let object = revision.as_object_mut().expect("revision object");
    object.remove("source_revision");
    object.remove("source_hash");
    source.source_revision = format!(
        "ecsr1_{}",
        digest(
            &ConfigProjectionCodecV1::encode_canonical_json(&revision).expect("canonical revision")
        )
    );
    let mut value = serde_json::to_value(&source).expect("effective source hash value");
    value.as_object_mut().unwrap().remove("source_hash");
    source.source_hash = ConfigProjectionCodecV1::domain_sha256(
        "",
        &serde_json::json!({"domain": "substrate.e3.effective-substrate-config-source.v1", "source": value}),
    )
    .expect("effective source hash");
    source
}

fn inventory_source(store: &ConfigProjectionStoreV1) -> AgentInventorySourceMaterialV1 {
    let raw_hash = digest(b"version: 3\n");
    let mut source = AgentInventorySourceMaterialV1 {
        inventory_scope: "global".to_string(),
        accepted_root: store.accepted_home.clone(),
        relative_path: "agents/codex.yaml".to_string(),
        file_device_id: 1,
        file_inode: 2,
        byte_length: 11,
        raw_bytes_sha256: raw_hash.clone(),
        source_revision: format!("aisr1_{raw_hash}"),
        source_hash: String::new(),
    };
    let mut value = serde_json::to_value(&source).expect("inventory source hash value");
    value.as_object_mut().unwrap().remove("source_hash");
    source.source_hash = ConfigProjectionCodecV1::domain_sha256(
        "",
        &serde_json::json!({"domain": "substrate.e3.agent-inventory-source.v1", "source": value}),
    )
    .expect("inventory source hash");
    source
}

#[test]
fn e3c_authoring_requires_the_two_fixed_authenticated_artifact_sources() {
    let (_temp, registry, store) = registry_fixture();
    let effective = effective_source(&store);
    let inventory = inventory_source(&store);
    assert_eq!(
        registry.import_runtime_artifacts(
            &effective,
            &inventory,
            Timestamp("2026-09-11T12:00:00.000000Z".to_string()),
        ),
        Err(ConfigProjectionFailureV1::MissingPreparation)
    );
}

// Synthetic nonsecret E2/artifact inputs exercise source publication, not manager authentication.
// All native source identities and bytes are supplied by the production renderer/registry.
fn native_projection(
    store: &ConfigProjectionStoreV1,
    series_id: &str,
    fence_id: &str,
) -> Codex0125ProjectionPlanV1 {
    let digest = "11".repeat(32);
    let commitment = serde_json::json!({
        "authority_store_id": "hsa_store",
        "commitment_id": "dpc_01890f3e-7b8c-7a11-8c55-0242ac120002",
        "exact_linkage_hash": digest,
    });
    let policy = serde_json::json!({
        "ref_id": format!("ao_{}", "22".repeat(16)),
        "object_kind": "policy",
        "schema_version": 1,
        "commitment": {"kind": "CanonicalSha256", "value": {"digest_hex": digest}},
    });
    let cap = serde_json::json!({
        "e2_activation_id": "e2a_test",
        "e2_launch_kind": "fresh_spawn",
        "commitment_ref": commitment,
        "commitment_subject": {"RetainedWorkerLaunch": {
            "retained_participant_id": "participant",
            "bootstrap_run_id": "bootstrap"
        }},
        "immutable_worker_cap_ref": commitment,
        "immutable_worker_cap_created_revision": 1,
        "immutable_worker_cap_application_revision": 1,
        "policy_snapshot_ref": policy,
        "policy_snapshot_hash": digest,
        "policy_snapshot_revision": "1",
        "request_id": "request",
        "idempotency_key": "idempotency",
        "caller_participant_id": "caller",
        "caller_backend_id": "cli:codex-world",
        "target_backend_id": "cli:codex-world",
        "target_world": {"world_id": "world", "world_generation": 1},
        "registry_publication_revision": 1
    });
    let support = serde_json::json!({
        "schema_version": 1,
        "support_policy_version": 1,
        "elf_execution_model": "StaticExec",
        "elf_interpreter": null,
        "dynamic_loader_cache": null,
        "ordered_elf_dependencies": [],
        "ordered_present_common_files": [],
        "system_config_mount_target": {
            "absolute_path": "/etc/codex", "device_id": 1, "inode": 2,
            "mode": 0o755, "owner_uid": 0, "owner_gid": 0,
            "ordered_entry_names": []
        },
        "manifest_hash": digest,
    });
    let artifact = |role: &str| {
        serde_json::json!({
            "role": role,
            "configured_absolute_path": "/artifact",
            "device_id": 1,
            "inode": 2,
            "file_type": "regular",
            "mode": 0o755,
            "owner_uid": 0,
            "byte_length": 1,
            "sha256": digest,
            "authority_ref": {
                "authority_store_id": store.authority_store_id, "manifest_id": "ram_test",
                "manifest_revision": 1, "manifest_entry_id": "rae_test",
                "manifest_hash": digest, "entry_hash": digest
            },
            "provenance": {"OfficialCodexRelease": {
                "version": "0.125.0", "target_triple": "x86_64-unknown-linux-musl",
                "archive_name": "codex.tar.gz", "archive_url": "https://example.invalid/codex",
                "archive_sha256": digest, "archive_entry_path": "codex",
                "extracted_executable_sha256": digest
            }},
            "runtime_support": support,
        })
    };
    let canonical_directory = serde_json::to_value(&store.accepted_home).unwrap();
    let identity: ConfigProjectionIdentityV1 = serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "authority_store_id": store.authority_store_id,
        "series_id": series_id,
        "accepted_home": canonical_directory,
        "workspace_root": canonical_directory,
        "orchestration_session_id": "session",
        "retained_participant_id": "participant",
        "bootstrap_run_id": "bootstrap",
        "backend_id": "cli:codex-world",
        "runtime_family": "codex",
        "world_id": "world",
        "world_generation": 1,
        "immutable_launch_cap": cap,
        "runtime_artifacts": {
            "codex": artifact("Codex0125"),
            "world_entry_wrapper": artifact("WorldEntryWrapper"),
            "managed_gateway": artifact("ManagedGateway")
        },
        "identity_hash": "identity"
    }))
    .unwrap();
    let intent = serde_json::json!({
        "authority_store_id": store.authority_store_id,
        "activation_intent_id": "gai_01890f3e-7b8c-7a11-8c55-0242ac120003",
        "intent_hash": digest
    });
    let root = NativeProjectionRootV1 {
        root_id: format!("cnr_{}", fence_id.strip_prefix("cpf_").unwrap()),
        authority_relative_path: format!(
            "authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}"
        ),
        guest_absolute_path: format!("/run/substrate/member-config/{series_id}/{fence_id}"),
        owner_uid: u64::from(unsafe { libc::geteuid() }),
        owner_gid: u64::from(unsafe { libc::getegid() }),
        directory_mode: 0o700,
    };
    let environment = EffectiveEnvironmentV1 {
            inherited_names: Vec::new(),
            set: [
                ("CODEX_HOME", format!("{}/codex-home", root.guest_absolute_path)),
                (
                    "CODEX_SQLITE_HOME",
                    format!("{}/state/sqlite", root.guest_absolute_path),
                ),
                ("HOME", format!("{}/home", root.guest_absolute_path)),
                ("LANG", "C.UTF-8".to_string()),
                ("LC_ALL", "C.UTF-8".to_string()),
                ("PATH", "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()),
                ("RUST_LOG", "error".to_string()),
                ("TMPDIR", format!("{}/tmp", root.guest_absolute_path)),
            ]
            .into_iter()
            .map(|(name, value)| NamedValueV1 { name: name.to_string(), value })
            .collect(),
            remove: [
                "ANTHROPIC_API_KEY", "CODEX_API_KEY", "CODEX_BINARY", "CODEX_OSS_BASE_URL",
                "CODEX_OSS_PORT", "OPENAI_ACCESS_TOKEN", "OPENAI_API_KEY", "OPENAI_BASE_URL",
                "OPENAI_ORGANIZATION", "OPENAI_PROJECT", "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD",
                "SUBSTRATE_E3_NATIVE_REALIZATION_FD", "SUBSTRATE_E3_NATIVE_SOURCE_FD",
                "SUBSTRATE_E3_SYSTEM_EMPTY_FD", "SUBSTRATE_E3_WORLD_FS_INPUT_FD",
                "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME", "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
                "SUBSTRATE_WORLD_ENTRY_BINARY", "SUBSTRATE_WORLD_ENTRY_BINARY_FD",
                "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH",
                "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD", "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH",
                "SUBSTRATE_WORLD_ENTRY_ROLE", "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD",
                "SUBSTRATE_WORLD_ENTRY_WORKING_DIR", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD",
            ].into_iter().map(str::to_string).collect(),
        };
    let effective: EffectiveAgentConfigProjectionV1 = serde_json::from_value(serde_json::json!({
        "projection_hash": digest,
        "logical_projection_hash": digest,
        "accepted_policy": cap,
        "capabilities": [],
        "model": "codex",
        "provider": {
            "provider_id": "substrate-managed-gateway", "wire_api": "responses",
            "requires_openai_auth": false, "supports_websockets": false,
            "gateway_intent_ref": intent
        },
        "mcp_servers": [], "features": [], "environment": environment,
        "workspace_overlay": "Disabled"
    }))
    .unwrap();
    let gateway: ManagedGatewayProjectionV1 =
            serde_json::from_value(serde_json::json!({
                "projection_hash": digest,
                "activation_intent_ref": intent,
                "expected_gateway_ref": {"authority_store_id": store.authority_store_id, "gateway_instance_id": "cgi_01890f3e-7b8c-7a11-8c55-0242ac120004", "gateway_identity_hash": digest},
                "codex_base_url": "http://127.0.0.1:43123/v1",
                "access_boundary_ref": {"authority_store_id": store.authority_store_id, "access_boundary_id": "gab_01890f3e-7b8c-7a11-8c55-0242ac120005", "revision": 1, "boundary_hash": digest},
                "activation_ack_ref": null,
                "posture": "Dormant"
            }))
            .unwrap();
    // The test-owned workspace has no .codex entry: all project inputs are observed absent.
    assert!(!PathBuf::from(&store.accepted_home.physical_path)
        .join(".codex")
        .exists());
    let inputs = [".codex/config.toml", ".codex/rules", ".codex/skills"]
        .into_iter()
        .map(|relative_path| CodexLoaderInputAttestationV1 {
            layer: "Project".into(),
            locator: format!("{}/{}", store.accepted_home.physical_path, relative_path),
            disposition: CodexLoaderInputDispositionV1::DisabledByTrust,
            directory: store.accepted_home.clone(),
            relative_path: relative_path.into(),
            device_id: None,
            inode: None,
            byte_length: None,
            sha256: None,
        })
        .collect();
    Codex0125ProjectionV1::render(&identity, &effective, &gateway, root, fence_id, inputs).unwrap()
}

#[test]
fn e3c_native_source_is_byte_exact_and_recovers_atomic_directory_rename() {
    let (temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let plan = native_projection(&store, &series_id, &fence_id);
    let series = temp
        .path()
        .join("authority-v1/agent-config-projection-v1/native-sources")
        .join(&series_id);
    let final_path = series.join(&fence_id);
    assert!(!final_path.exists());
    let created_at = Timestamp("2026-09-11T12:00:00.000000Z".into());
    let (native, source) = registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at.clone())
        .unwrap();
    let bytes = std::fs::read(final_path.join("codex-home/config.toml")).unwrap();
    assert_eq!(
        bytes,
        base64::engine::general_purpose::STANDARD
            .decode(&native.files[0].bytes_base64)
            .unwrap()
    );
    assert_eq!(digest(&bytes), native.files[0].sha256);
    assert_eq!(bytes.len() as u64, native.files[0].byte_length);
    let config = File::open(final_path.join("codex-home/config.toml")).unwrap();
    let metadata = config.metadata().unwrap();
    let observed = native
        .ambient_closure
        .inputs
        .iter()
        .find(|input| input.layer == "User" && input.relative_path == "config.toml")
        .unwrap();
    assert_eq!(observed.device_id, Some(metadata.dev()));
    assert_eq!(observed.inode, Some(metadata.ino()));
    assert_eq!(observed.sha256.as_ref(), Some(&digest(&bytes)));
    assert_eq!(observed.byte_length, Some(metadata.len()));
    assert_eq!(metadata.mode() & 0o7777, 0o600);
    assert_eq!(metadata.nlink(), 1);
    assert_eq!(metadata.uid() as u64, native.root.owner_uid);
    assert_eq!(metadata.gid() as u64, native.root.owner_gid);
    assert!(final_path.join("system-empty").is_dir());
    assert_eq!(
        source.source_root.physical_path,
        final_path.display().to_string()
    );
    assert_eq!(source.native_projection_hash, native.projection_hash);
    assert!(native
        .ambient_closure
        .inputs
        .iter()
        .filter(|input| input.layer != "Project")
        .all(|input| input
            .directory
            .physical_path
            .starts_with(&source.source_root.physical_path)));

    let temporary = series.join(format!(".e3-native-source-tmp.{}", Uuid::now_v7()));
    std::fs::rename(&final_path, &temporary).unwrap();
    registry
        .recover(None)
        .map(|readback| readback.store)
        .expect("recover complete atomic directory publication");
    assert!(final_path.join("source-manifest.json").is_file());
    assert!(!temporary.exists());
    assert_eq!(
        registry
            .publish_native_source(&series_id, &fence_id, &plan, created_at)
            .unwrap(),
        (native, source)
    );
}

#[test]
fn e3c_native_source_exact_retry_reuses_the_existing_physical_publication() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let plan = native_projection(&store, &series_id, &fence_id);
    let created_at = Timestamp("2026-09-11T12:00:00.000000Z".into());
    let first = registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at.clone())
        .unwrap();
    let retry = registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at)
        .unwrap();
    assert_eq!(retry, first);
}

#[test]
fn e3c_native_source_conflicting_retry_does_not_replace_the_final_publication() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let plan = native_projection(&store, &series_id, &fence_id);
    let created_at = Timestamp("2026-09-11T12:00:00.000000Z".into());
    let first = registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at.clone())
        .unwrap();
    assert_eq!(
        registry.publish_native_source(
            &series_id,
            &fence_id,
            &plan,
            Timestamp("2026-09-11T12:00:01.000000Z".into())
        ),
        Err(ConfigProjectionFailureV1::Conflict)
    );
    assert_eq!(
        registry
            .publish_native_source(&series_id, &fence_id, &plan, created_at)
            .unwrap(),
        first
    );
}

#[test]
fn test_native_source_rejects_equal_bytes_on_a_replaced_config_inode() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let plan = native_projection(&store, &series_id, &fence_id);
    let created_at = Timestamp("2026-09-11T12:00:00.000000Z".into());
    let (_, source) = registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at.clone())
        .unwrap();
    let path = PathBuf::from(&source.source_root.physical_path).join("codex-home/config.toml");
    let original = File::open(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_ne!(
        original.metadata().unwrap().ino(),
        std::fs::metadata(&path).unwrap().ino()
    );
    assert!(registry
        .publish_native_source(&series_id, &fence_id, &plan, created_at)
        .is_err());
}

#[test]
fn test_native_source_recovery_does_not_promote_an_incomplete_tree() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let plan = native_projection(&store, &series_id, &fence_id);
    let (_, source) = registry
        .publish_native_source(
            &series_id,
            &fence_id,
            &plan,
            Timestamp("2026-09-11T12:00:00.000000Z".into()),
        )
        .unwrap();
    let final_path = PathBuf::from(source.source_root.physical_path);
    let temporary = final_path
        .parent()
        .unwrap()
        .join(format!(".e3-native-source-tmp.{}", Uuid::now_v7()));
    std::fs::rename(&final_path, &temporary).unwrap();
    std::fs::remove_file(temporary.join("source-manifest.json")).unwrap();
    assert!(registry
        .recover(None)
        .map(|readback| readback.store)
        .is_err());
    assert!(!final_path.exists());
    assert!(temporary.join("codex-home/config.toml").is_file());
    assert!(!temporary.join("source-manifest.json").exists());
}
