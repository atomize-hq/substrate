#![cfg(target_os = "linux")]

use std::fs::{File, OpenOptions};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
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
    let store = registry.recover().expect("initialize store");
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

fn native_projection(
    store: &ConfigProjectionStoreV1,
    series_id: &str,
    fence_id: &str,
) -> NativeAgentConfigProjectionV1 {
    let bytes = b"model = \"codex\"\n";
    let encoded = BASE64.encode(bytes);
    let bytes_sha256 = digest(bytes);
    let directories = [
        ".",
        "system-empty",
        "home",
        "codex-home",
        "state",
        "state/sqlite",
        "state/log",
        "tmp",
        "tmp/output-last-message",
    ]
    .into_iter()
    .map(|relative_path| serde_json::json!({"relative_path": relative_path, "mode": 0o700}))
    .collect::<Vec<_>>();
    let mut native: NativeAgentConfigProjectionV1 = serde_json::from_value(serde_json::json!({
        "projection_hash": "",
        "renderer": {"renderer_id": "substrate.codex.config-renderer", "renderer_schema_version": 1, "codex_version": "0.125.0"},
        "root": {
            "root_id": id("cnr_"),
            "authority_relative_path": format!("authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}"),
            "guest_absolute_path": format!("/run/substrate/member-config/{series_id}/{fence_id}"),
            "owner_uid": 1000,
            "owner_gid": 1000,
            "directory_mode": 0o700
        },
        "files": [{"role": "CodexConfigToml", "relative_path": "codex-home/config.toml", "mode": 0o600, "bytes_base64": encoded, "byte_length": bytes.len(), "sha256": bytes_sha256}],
        "directories": directories,
        "environment": {"inherited_names": [], "set": [], "remove": []},
        "invocation": {"wrapper_argv": [], "initial_codex_argv": [], "resume_codex_argv_prefix": [], "prompt_delivery": "stdin-lf-eof", "output_last_message_directory": "tmp/output-last-message", "output_last_message_name_domain": "turn", "forbidden_arguments": []},
        "cwd": store.accepted_home,
        "ambient_closure": {
            "loader_source": {"codex_version": "0.125.0", "upstream_tag": "rust-v0.125.0", "config_loader_source_sha256": "11".repeat(32), "layer_io_source_sha256": "11".repeat(32), "loader_model_source_sha256": "11".repeat(32), "exec_source_sha256": "11".repeat(32), "cloud_requirements_source_sha256": "11".repeat(32), "auth_storage_source_sha256": "11".repeat(32), "config_types_source_sha256": "11".repeat(32), "validator_schema_version": 1},
            "allowed_enabled_layers": [], "inputs": [], "forbidden_cli_overrides": [], "validated_loader_input_fingerprint": "11".repeat(32)
        }
    }))
    .expect("native fixture");
    let mut value = serde_json::to_value(&native).expect("native value");
    value
        .as_object_mut()
        .expect("native object")
        .remove("projection_hash");
    native.projection_hash = ConfigProjectionCodecV1::domain_sha256(
        "",
        &serde_json::json!({"domain": "substrate.e3.native-config-projection.v1", "projection": value}),
    )
    .expect("native hash");
    native
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

#[test]
fn e3c_native_source_is_byte_exact_and_recovers_atomic_directory_rename() {
    let (temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let native = native_projection(&store, &series_id, &fence_id);
    let source = registry
        .publish_native_source(
            &series_id,
            &fence_id,
            &native,
            Timestamp("2026-09-11T12:00:00.000000Z".to_string()),
        )
        .expect("publish native source");
    let series = temp
        .path()
        .join("authority-v1/agent-config-projection-v1/native-sources")
        .join(&series_id);
    let final_path = series.join(&fence_id);
    assert_eq!(
        std::fs::read(final_path.join("codex-home/config.toml")).expect("read native bytes"),
        b"model = \"codex\"\n"
    );
    assert!(final_path.join("system-empty").is_dir());
    assert_eq!(
        source.source_root.physical_path,
        final_path.display().to_string()
    );

    let temp_path = series.join(format!(".e3-native-source-tmp.{}", Uuid::now_v7()));
    std::fs::rename(&final_path, &temp_path).expect("interrupt native directory rename");
    registry
        .recover()
        .expect("recover native directory publication");
    assert!(final_path.join("source-manifest.json").is_file());
    assert!(!temp_path.exists());
}

#[test]
fn e3c_native_source_exact_retry_reuses_the_existing_physical_publication() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let native = native_projection(&store, &series_id, &fence_id);
    let created_at = Timestamp("2026-09-11T12:00:00.000000Z".to_string());

    let first = registry
        .publish_native_source(&series_id, &fence_id, &native, created_at.clone())
        .expect("first native publication");
    let retry = registry
        .publish_native_source(&series_id, &fence_id, &native, created_at)
        .expect("byte-equal retry must reuse the final publication");

    assert_eq!(retry, first);
}

#[test]
fn e3c_native_source_conflicting_retry_does_not_replace_the_final_publication() {
    let (_temp, registry, store) = registry_fixture();
    let series_id = id("cps_");
    let fence_id = id("cpf_");
    let native = native_projection(&store, &series_id, &fence_id);
    registry
        .publish_native_source(
            &series_id,
            &fence_id,
            &native,
            Timestamp("2026-09-11T12:00:00.000000Z".to_string()),
        )
        .expect("first native publication");

    assert_eq!(
        registry.publish_native_source(
            &series_id,
            &fence_id,
            &native,
            Timestamp("2026-09-11T12:00:01.000000Z".to_string()),
        ),
        Err(ConfigProjectionFailureV1::Conflict)
    );
}
