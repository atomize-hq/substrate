#![cfg(target_os = "linux")]

#[path = "agent_config_projection_v1.rs"]
mod historical;

use base64::Engine as _;
use config_projection::*;

fn inputs(
    store: &ConfigProjectionStoreV1,
    series: &str,
    fence: &str,
) -> historical::NativeInputsFixtureV1 {
    let (mut identity, effective, gateway, root, project) =
        historical::native_inputs(store, series, fence);
    let pins = Codex0156ReleaseBundleV1::pinned();
    let main = &mut identity.runtime_artifacts.codex;
    main.role = ConfigProjectionArtifactRoleV1::Codex0156;
    main.configured_absolute_path = CODEX_0156_MAIN_PATH.to_string();
    main.sha256 = pins.main.extracted_executable_sha256.clone();
    main.byte_length = pins.main.extracted_byte_length;
    main.provenance = pins.main.provenance();
    main.runtime_support.support_policy_version = 2;
    main.runtime_support.code_mode_host = Some(Codex0156CodeModeHostV1 {
        file: RuntimeSupportFileV1 {
            absolute_path: CODEX_0156_HOST_PATH.into(),
            device_id: 11,
            inode: 22,
            mode: 0o755,
            byte_length: pins.code_mode_host.extracted_byte_length,
            sha256: pins.code_mode_host.extracted_executable_sha256.clone(),
        },
        owner_uid: 0,
        elf_execution_model: E3ElfExecutionModelV1::StaticExec,
        provenance: pins.code_mode_host.provenance(),
    });
    (identity, effective, gateway, root, project)
}

#[test]
fn codex0156_real_native_publication_has_stock_defaults_and_disjoint_source_identity() {
    let (_temp, registry, store) = historical::registry_fixture();
    let series = format!("cps_{}", uuid::Uuid::now_v7());
    let fence = format!("cpf_{}", uuid::Uuid::now_v7());
    let (identity, effective, gateway, root, project) = inputs(&store, &series, &fence);
    let plan = Codex0156ProjectionV1::render(
        &identity,
        &effective,
        &gateway,
        root.clone(),
        &fence,
        project.clone(),
    )
    .unwrap();
    let (native, source) = registry
        .publish_codex0156_native_source(
            &series,
            &fence,
            &plan,
            Timestamp("2026-10-25T12:00:00.000000Z".into()),
        )
        .unwrap();
    assert_eq!(native.renderer.codex_version, "0.156.1");
    assert_eq!(native.renderer.renderer_schema_version, 2);
    assert_eq!(
        native.ambient_closure.loader_source.codex_version,
        "0.156.1"
    );
    let sources = native
        .ambient_closure
        .loader_source
        .codex_0156_sources
        .as_ref()
        .unwrap();
    assert_eq!(sources.models_cache_filename, "models_cache.json");
    assert_eq!(
        sources.cloud_cache_filename,
        "cloud-config-bundle-cache.json"
    );
    assert_eq!(
        sources.upstream_commit,
        "b412ff32c417f855c2b2d1581b77058eed87c84b"
    );
    assert!(Codex0125ProjectionV1::validate_loader_inputs(&native.ambient_closure).is_err());
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&native.files[0].bytes_base64)
        .unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("model = \"codex\""));
    assert!(text.contains("cli_auth_credentials_store = \"ephemeral\""));
    assert!(!text.contains("code_mode_host"));
    assert!(!text.contains("disable_in_process_fallback"));
    assert_eq!(
        std::fs::read(
            std::path::Path::new(&source.source_root.physical_path).join("codex-home/config.toml")
        )
        .unwrap(),
        bytes
    );
    let (_, retry) = registry
        .publish_codex0156_native_source(
            &series,
            &fence,
            &plan,
            Timestamp("2026-10-25T12:00:00.000000Z".into()),
        )
        .unwrap();
    assert_eq!(source, retry);
    Codex0156ProjectionV1::validate_loader_inputs(&native.ambient_closure).unwrap();
    for mutation in 0..4 {
        let mut closure = native.ambient_closure.clone();
        match mutation {
            0 => closure.loader_source.config_loader_source_sha256 = "00".repeat(32),
            1 => {
                closure
                    .loader_source
                    .codex_0156_sources
                    .as_mut()
                    .unwrap()
                    .builtin_catalogue_sha256 = "00".repeat(32)
            }
            2 => closure.inputs[15].relative_path = "cloud-requirements-cache.json".into(),
            _ => {
                closure.inputs[16].disposition = CodexLoaderInputDispositionV1::Projected;
            }
        }
        assert!(Codex0156ProjectionV1::validate_loader_inputs(&closure).is_err());
    }
    let cache = std::path::Path::new(&source.source_root.physical_path)
        .join("codex-home/models_cache.json");
    std::fs::write(&cache, b"changed cache").unwrap();
    assert!(registry
        .publish_codex0156_native_source(
            &series,
            &fence,
            &plan,
            Timestamp("2026-10-25T12:00:00.000000Z".into())
        )
        .is_err());
    std::fs::remove_file(cache).unwrap();
    let mut mixed = identity.clone();
    mixed.runtime_artifacts.codex.role = ConfigProjectionArtifactRoleV1::Codex0125;
    assert!(Codex0156ProjectionV1::render(
        &mixed,
        &effective,
        &gateway,
        root.clone(),
        &fence,
        project.clone()
    )
    .is_err());
    let mut missing = identity.clone();
    missing
        .runtime_artifacts
        .codex
        .runtime_support
        .code_mode_host = None;
    assert!(Codex0156ProjectionV1::render(
        &missing,
        &effective,
        &gateway,
        root.clone(),
        &fence,
        project.clone()
    )
    .is_err());
    let mut wrong = identity;
    wrong
        .runtime_artifacts
        .codex
        .runtime_support
        .code_mode_host
        .as_mut()
        .unwrap()
        .provenance = Codex0156ReleaseBundleV1::pinned().main.provenance();
    assert!(
        Codex0156ProjectionV1::render(&wrong, &effective, &gateway, root, &fence, project).is_err()
    );
}

#[test]
#[ignore = "designated pinned catalogue source fixture; requires SUBSTRATE_CODEX_0156_CATALOGUE"]
fn codex0156_stock_catalogue_profile_is_a_fixture_only() {
    use sha2::{Digest, Sha256};
    let path = std::env::var("SUBSTRATE_CODEX_0156_CATALOGUE")
        .expect("named stock-profile fixture requires official catalogue");
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "1892a933a420e79a30779ef5f1ce5b7dbc145464aa6fb1432106a9e9859981d3"
    );
    let catalogue: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let models = catalogue["models"].as_array().unwrap();
    let profile = models
        .iter()
        .find(|model| model["slug"] == "gpt-6-astra")
        .expect("exact stock fixture profile");
    assert_eq!(profile["tool_mode"], "code_mode_only");
    assert!(profile["default_service_tier"].is_null());
    assert!(profile["supported_reasoning_levels"]
        .as_array()
        .unwrap()
        .iter()
        .any(|level| level["effort"] == "high"));
    assert!(
        !models.iter().any(|model| model["slug"] == "codex"),
        "production alias is not qualified by this fixture"
    );
    let mut changed = catalogue.clone();
    changed["models"][0]["tool_mode"] = serde_json::json!("direct");
    assert_ne!(
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&changed).unwrap())
        ),
        "1892a933a420e79a30779ef5f1ce5b7dbc145464aa6fb1432106a9e9859981d3"
    );
}
