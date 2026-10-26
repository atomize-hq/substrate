#![cfg(target_os = "linux")]
#[path = "agent_config_projection_v1.rs"]
mod historical;
use config_projection::*;
fn canonical(value: &impl serde::Serialize) -> Vec<u8> {
    ConfigProjectionCodecV1::encode_canonical_json(value).unwrap()
}
#[test]
fn selected_release_binds_verified_pair_and_stable_profile() {
    let catalog = CodexReleaseCatalogV1::embedded().unwrap();
    let selected = TrustedCodexReleaseV1::selected(CODEX_RELEASE_TARGET_V1).unwrap();
    assert_eq!(
        Some(&selected.record().version),
        catalog.selected.get(CODEX_RELEASE_TARGET_V1)
    );
    let historical = catalog
        .releases
        .iter()
        .find(|record| record.version == "0.156.1")
        .expect("immutable admitted156 record");
    let release = TrustedCodexReleaseV1::resolve(&historical.binding().unwrap()).unwrap();
    assert_eq!(release.record().version, "0.156.1");
    assert_eq!(
        release.binding().manifest_sha256,
        "8e602fdde28cb3a407a22fe81ecf69a59691f129d0a6cd94e91640cb06ea5a49"
    );
    assert_eq!(
        release.main().archive.sha256,
        "aff46539a83aff86e3c62c592bce2c50d95391f9df289afaf03a50c01d14533d"
    );
    assert_eq!(
        release.host().extracted.sha256,
        "45ba654b0e145406f3316be4729b886980ce811aa2e3b6ed4e033dd41ab524f1"
    );
    assert_eq!(release.record().source.files.len(), 19);
    assert_eq!(
        TrustedCodexReleaseV1::authenticate(release.input()).unwrap(),
        release
    );
    let mut external = release.input().clone();
    external.manifest.source.files[0].sha256 = "00".repeat(32);
    external.binding = external.manifest.binding().unwrap();
    assert!(TrustedCodexReleaseV1::authenticate(&external).is_err());
}
#[test]
fn catalog_rejects_duplicate_json_unknown_fields_schema_profile_and_unsafe_data() {
    let good = CodexReleaseCatalogV1::embedded().unwrap();
    let bytes = canonical(&good);
    let duplicate = String::from_utf8(bytes.clone()).unwrap().replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    assert!(CodexReleaseCatalogV1::parse(duplicate.as_bytes()).is_err());
    assert!(CodexReleaseCatalogV1::parse(&[bytes.clone(), b"\n".to_vec()].concat()).is_err());
    for mutation in 0..18 {
        let mut value = serde_json::to_value(&good).unwrap();
        let r = &mut value["releases"][0];
        match mutation {
            0 => value["unknown"] = serde_json::json!(true),
            1 => value["schema_version"] = serde_json::json!(2),
            2 => r["behavior_profile"] = serde_json::json!("unknown"),
            3 => r["target"] = serde_json::json!("aarch64-unknown-linux-musl"),
            4 => value["selected"] = serde_json::json!({}),
            5 => value["selected"][CODEX_RELEASE_TARGET_V1] = serde_json::json!("unknown"),
            6 => r["artifacts"]
                .as_array_mut()
                .unwrap()
                .pop()
                .map(|_| ())
                .unwrap(),
            7 => {
                let a = r["artifacts"][0].clone();
                r["artifacts"].as_array_mut().unwrap().push(a)
            }
            8 => r["artifacts"].as_array_mut().unwrap().swap(0, 1),
            9 => {
                r["artifacts"][1]["archive"]["url"] = serde_json::json!("https://evil.example/host")
            }
            10 => r["artifacts"][1]["archive"]["member"] = serde_json::json!("../host"),
            11 => r["artifacts"][1]["archive"]["byte_length"] = serde_json::json!(0),
            12 => r["artifacts"][1]["extracted"]["sha256"] = serde_json::json!("G".repeat(64)),
            13 => r["source"]["files"][0]["path"] = serde_json::json!("../../secret"),
            14 => r["source"]["files"].as_array_mut().unwrap().swap(0, 1),
            15 => {
                let x = value["releases"][0].clone();
                value["releases"].as_array_mut().unwrap().push(x)
            }
            16 => r["source"]["tag"] = serde_json::json!("rust-vother"),
            _ => r["source"]["builtin_catalogue"]["sha256"] = serde_json::json!("bad"),
        }
        assert!(
            CodexReleaseCatalogV1::parse(&canonical(&value)).is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn unrelated_catalog_addition_and_selection_do_not_change_existing_record_digest() {
    let mut catalog = CodexReleaseCatalogV1::embedded().unwrap();
    let first = catalog.releases[0].clone();
    let digest = first.binding().unwrap();
    let mut later = first.clone();
    let mut serial = 0;
    loop {
        later.version = format!("999999.0.{serial}-fixture");
        if !catalog.releases.iter().any(|r| r.version == later.version) {
            break;
        }
        serial += 1;
    }
    later.source.tag = format!("rust-v{}", later.version);
    for a in &mut later.artifacts {
        a.archive.url = format!(
            "https://github.com/openai/codex/releases/download/{}/{}",
            later.source.tag, a.archive.name
        )
    }
    catalog
        .selected
        .insert(later.target.clone(), later.version.clone());
    catalog.releases.push(later.clone());
    catalog.validate().unwrap();
    assert_eq!(first.binding().unwrap(), digest);
    assert_ne!(later.binding().unwrap(), digest);
    assert!(
        TrustedCodexReleaseV1::resolve(&later.binding().unwrap()).is_err(),
        "parsed external data confers no authority"
    );
}
#[test]
fn generic_native_publication_preserves_config_defaults_and_rejects_crossbinding() {
    let (_temp, registry, store) = historical::registry_fixture();
    let series = format!("cps_{}", uuid::Uuid::now_v7());
    let fence = format!("cpf_{}", uuid::Uuid::now_v7());
    let (mut identity, effective, gateway, root, project) =
        historical::native_inputs(&store, &series, &fence);
    let release = TrustedCodexReleaseV1::selected(CODEX_RELEASE_TARGET_V1).unwrap();
    let main = &mut identity.runtime_artifacts.codex;
    main.role = ConfigProjectionArtifactRoleV1::CodexMain;
    main.configured_absolute_path = CODEX_MAIN_PATH_V1.into();
    main.provenance = release.provenance(CodexReleaseArtifactRoleV1::Main);
    main.sha256 = release.main().extracted.sha256.clone();
    main.byte_length = release.main().extracted.byte_length;
    main.runtime_support.support_policy_version = 3;
    main.runtime_support.elf_execution_model = release.main().extracted.elf.clone();
    main.runtime_support.code_mode_host = Some(Codex0156CodeModeHostV1 {
        file: RuntimeSupportFileV1 {
            absolute_path: CODEX_HOST_PATH_V1.into(),
            device_id: 11,
            inode: 22,
            mode: 0o755,
            byte_length: release.host().extracted.byte_length,
            sha256: release.host().extracted.sha256.clone(),
        },
        owner_uid: 0,
        elf_execution_model: release.host().extracted.elf.clone(),
        provenance: release.provenance(CodexReleaseArtifactRoleV1::CodeModeHost),
    });
    let plan = CodexProjectionV1::render(
        &identity,
        &effective,
        &gateway,
        root.clone(),
        &fence,
        project.clone(),
    )
    .unwrap();
    let (native, source) = registry
        .publish_codex_native_source(
            &series,
            &fence,
            &plan,
            Timestamp("2026-10-25T12:00:00.000000Z".into()),
        )
        .unwrap();
    assert_eq!(
        native.renderer.release_binding.as_ref(),
        Some(release.binding())
    );
    assert_eq!(
        native.ambient_closure.loader_source,
        release.loader_source()
    );
    CodexProjectionV1::validate_loader_inputs(&native.ambient_closure).unwrap();
    assert!(Codex0156ProjectionV1::validate_loader_inputs(&native.ambient_closure).is_err());
    let config = std::fs::read(
        std::path::Path::new(&source.source_root.physical_path).join("codex-home/config.toml"),
    )
    .unwrap();
    let text = std::str::from_utf8(&config).unwrap();
    assert!(!text.contains("code_mode_host"));
    assert!(!text.contains("disable_in_process_fallback"));
    assert!(text.contains("model = \"codex\""));
    let (_, retry) = registry
        .publish_codex_native_source(
            &series,
            &fence,
            &plan,
            Timestamp("2026-10-25T12:00:00.000000Z".into()),
        )
        .unwrap();
    assert_eq!(retry, source);
    for mutation in 0..5 {
        let mut bad = identity.clone();
        let main = &mut bad.runtime_artifacts.codex;
        match mutation {
            0 => main.runtime_support.code_mode_host = None,
            1 => {
                main.runtime_support
                    .code_mode_host
                    .as_mut()
                    .unwrap()
                    .file
                    .sha256 = "00".repeat(32)
            }
            2 => main.runtime_support.support_policy_version = 2,
            3 => {
                main.runtime_support
                    .code_mode_host
                    .as_mut()
                    .unwrap()
                    .file
                    .absolute_path = "/tmp/host".into()
            }
            _ => main.role = ConfigProjectionArtifactRoleV1::Codex0156,
        };
        assert!(CodexProjectionV1::render(
            &bad,
            &effective,
            &gateway,
            root.clone(),
            &fence,
            project.clone()
        )
        .is_err());
    }
    for mutation in 0..4 {
        let mut bad = native.ambient_closure.clone();
        match mutation {
            0 => bad.loader_source.config_loader_source_sha256 = "00".repeat(32),
            1 => {
                bad.loader_source
                    .release_binding
                    .as_mut()
                    .unwrap()
                    .manifest_sha256 = "00".repeat(32)
            }
            2 => bad.inputs[15].relative_path = "cloud-requirements-cache.json".into(),
            _ => bad.inputs[16].disposition = CodexLoaderInputDispositionV1::Projected,
        };
        assert!(CodexProjectionV1::validate_loader_inputs(&bad).is_err());
    }
}

#[test]
fn historical_role_stream_renderer_and_build_input_bytes_are_frozen() {
    use sha2::{Digest, Sha256};
    for (version, schema) in [("0.125.0", 1), ("0.156.1", 2)] {
        let renderer = NativeRendererIdentityV1 {
            release_binding: None,
            renderer_id: "substrate.codex.config-renderer".into(),
            renderer_schema_version: schema,
            codex_version: version.into(),
        };
        assert_eq!(canonical(&renderer),format!("{{\"codex_version\":\"{version}\",\"renderer_id\":\"substrate.codex.config-renderer\",\"renderer_schema_version\":{schema}}}").as_bytes());
    }
    assert_eq!(
        canonical(&ConfigProjectionArtifactRoleV1::Codex0125),
        br#""Codex0125""#
    );
    assert_eq!(
        canonical(&InstallerArtifactSourceStreamV1::Codex0156OfficialArchives),
        br#""Codex0156OfficialArchives""#
    );
    let legacy = InstallerArtifactBuildInputV1::Codex0156OfficialArchives(
        Codex0156ReleaseBundleV1::pinned(),
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(canonical(&legacy))),
        "c410d024ef06a7d31fc6dd82e130c7299cb78609e464827becc826ff3b9c8fda"
    );
}

#[test]
fn archive_asset_and_member_names_are_safe_release_data() {
    let original = CodexReleaseCatalogV1::embedded().unwrap();
    let mut catalog = original.clone();
    let release = &mut catalog.releases[0];
    for (artifact, (archive, member)) in release.artifacts.iter_mut().zip([
        ("renamed-main.tar.gz", "renamed-main-payload"),
        ("renamed-stock-host.tar.gz", "renamed-stock-host-payload"),
    ]) {
        artifact.archive.name = archive.into();
        artifact.archive.member = member.into();
        artifact.archive.url = format!(
            "https://github.com/openai/codex/releases/download/{}/{archive}",
            release.source.tag
        );
    }
    let renamed = release.clone();
    assert!(
        CodexReleaseCatalogV1::parse(&canonical(&catalog)).is_ok(),
        "MUR1 safe reviewed asset/member renaming must not require profile code"
    );
    assert_ne!(
        renamed.binding().unwrap(),
        original.releases[0].binding().unwrap()
    );
    // A parsed renamed fixture cannot authorize new bytes or change runtime paths.
    assert!(TrustedCodexReleaseV1::resolve(&renamed.binding().unwrap()).is_err());
    assert_eq!(
        CODEX_MAIN_PATH_V1,
        "/var/lib/substrate/world-deps/codex-runtime/bin/codex"
    );
    assert_eq!(
        CODEX_HOST_PATH_V1,
        "/var/lib/substrate/world-deps/codex-runtime/bin/codex-code-mode-host"
    );
    for name in [
        "",
        ".",
        "..",
        ".tar.gz",
        "main.zip",
        "/main.tar.gz",
        "../main.tar.gz",
        "nested/main.tar.gz",
        "main\\host.tar.gz",
        "main.tar.gz?query",
        "main.tar.gz#fragment",
        "main'quote.tar.gz",
        "main\n.tar.gz",
        "$(bad).tar.gz",
    ] {
        let mut bad = renamed.clone();
        bad.artifacts[0].archive.name = name.into();
        bad.artifacts[0].archive.url = format!(
            "https://github.com/openai/codex/releases/download/{}/{name}",
            bad.source.tag
        );
        assert!(
            bad.validate_structure().is_err(),
            "unsafe archive name {name:?}"
        );
    }
    for member in [
        "",
        ".",
        "..",
        "/payload",
        "../payload",
        "nested/payload",
        "payload\\host",
        "payload name",
        "payload'quote",
        "payload\n",
        "$(bad)",
    ] {
        let mut bad = renamed.clone();
        bad.artifacts[1].archive.member = member.into();
        assert!(
            bad.validate_structure().is_err(),
            "unsafe sole member {member:?}"
        );
    }
    for mutation in 0..6 {
        let mut bad = renamed.clone();
        match mutation {
            0 => {
                bad.artifacts[1].archive.name = bad.artifacts[0].archive.name.clone();
                bad.artifacts[1].archive.url = bad.artifacts[0].archive.url.clone();
            }
            1 => bad.artifacts[1].archive.member = bad.artifacts[0].archive.member.clone(),
            2 => bad.artifacts[0].archive.url.push_str("?query=1"),
            3 => bad.artifacts[0].archive.url.push_str("#fragment"),
            4 => {
                bad.artifacts[0].archive.url = bad.artifacts[0]
                    .archive
                    .url
                    .replace("github.com/openai/codex", "example.com/openai/codex")
            }
            _ => {
                bad.artifacts[0].archive.name = format!("{}.tar.gz", "a".repeat(256));
                bad.artifacts[0].archive.url = format!(
                    "https://github.com/openai/codex/releases/download/{}/{}",
                    bad.source.tag, bad.artifacts[0].archive.name
                );
            }
        }
        assert!(
            bad.validate_structure().is_err(),
            "archive/member uniqueness or URL mutation {mutation}"
        );
    }
}
