use super::*;

#[test]
fn resolve_bundled_binary_defaults_to_runtime_platform() {
    let temp = tempfile::tempdir().unwrap();
    let platform = default_bundled_platform_label();
    let version = "1.2.3";
    let version_dir = temp.path().join(&platform).join(version);
    std_fs::create_dir_all(&version_dir).unwrap();
    let binary = write_fake_bundled_codex(&version_dir, &platform, "#!/usr/bin/env bash\necho ok");

    let resolved = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: temp.path(),
        version,
        platform: None,
    })
    .unwrap();

    assert_eq!(resolved.platform, platform);
    assert_eq!(resolved.version, version);
    assert_eq!(resolved.binary_path, std_fs::canonicalize(&binary).unwrap());
}

#[test]
fn resolve_bundled_binary_honors_platform_override() {
    let temp = tempfile::tempdir().unwrap();
    let platform = "windows-x64";
    let version = "5.6.7";
    let version_dir = temp.path().join(platform).join(version);
    std_fs::create_dir_all(&version_dir).unwrap();
    let binary = write_fake_bundled_codex(&version_dir, platform, "#!/usr/bin/env bash\necho win");

    let resolved = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: temp.path(),
        version,
        platform: Some(platform),
    })
    .unwrap();

    assert_eq!(resolved.platform, platform);
    assert_eq!(resolved.version, version);
    assert_eq!(resolved.binary_path, std_fs::canonicalize(&binary).unwrap());
    assert_eq!(
        resolved
            .binary_path
            .file_name()
            .and_then(|name| name.to_str()),
        Some("codex.exe")
    );
}

#[test]
fn resolve_bundled_binary_errors_when_binary_missing() {
    let temp = tempfile::tempdir().unwrap();
    let platform = default_bundled_platform_label();
    let version = "0.0.1";
    let version_dir = temp.path().join(&platform).join(version);
    std_fs::create_dir_all(&version_dir).unwrap();

    let err = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: temp.path(),
        version,
        platform: None,
    })
    .unwrap_err();

    match err {
        BundledBinaryError::BinaryUnreadable { binary, .. }
        | BundledBinaryError::BinaryNotFile { binary }
        | BundledBinaryError::BinaryNotExecutable { binary } => {
            assert_eq!(binary, version_dir.join(bundled_binary_filename(&platform)));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn resolve_bundled_binary_rejects_empty_version() {
    let temp = tempfile::tempdir().unwrap();
    let err = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: temp.path(),
        version: "  ",
        platform: None,
    })
    .unwrap_err();
    assert!(matches!(err, BundledBinaryError::EmptyVersion));
}
