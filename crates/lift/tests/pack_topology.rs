#![allow(unused_crate_dependencies)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd as _;
use gix as _;
use jsonschema as _;
use predicates as _;
use serde as _;
use serde_jcs as _;
use serde_json as _;
use sha2 as _;
use substrate_lift as _;
use thiserror as _;
use toml as _;
use walkdir as _;

mod kernel {
    pub(crate) use substrate_lift::kernel::{
        sha256_bytes, sha256_canonical_json, DiagnosticCode, Fingerprint, JsonPointer, QueryId,
        RecipeId, RuleId, Severity,
    };
}
#[path = "../src/pack/mod.rs"]
mod pack;

fn running_under_wine() -> bool {
    std::env::var_os("WINELOADER").is_some()
}

fn normalize_display_path(path: impl AsRef<str>) -> String {
    let mut normalized = path.as_ref().replace('\\', "/");

    if cfg!(windows) && running_under_wine() {
        if let Some(stripped) = normalized.strip_prefix("Z:") {
            normalized = stripped.to_owned();
        }
    }

    normalized
}

#[test]
fn builtin_profile_resolves_builtin_topology_refs() {
    let compiler = pack::PackCompiler::new();
    let profile = compiler
        .compile_profile(pack::builtin::profile_source("generic/default").expect("builtin"))
        .expect("builtin profile");

    let resolved = compiler
        .resolve_profile_topology(&profile)
        .expect("builtin topology should resolve");

    let boundary = resolved
        .boundary_taxonomy
        .expect("builtin boundary taxonomy should be present");
    let component = resolved
        .component_map
        .expect("builtin component map should be present");

    assert_eq!(boundary.header.id.as_str(), "generic/boundaries");
    assert_eq!(component.header.id.as_str(), "generic/components");
    assert_eq!(
        boundary.header.origin.display(),
        "builtin:generic/boundaries"
    );
    assert_eq!(
        component.header.origin.display(),
        "builtin:generic/components"
    );
}

#[test]
fn file_backed_profile_resolves_relative_topology_refs() {
    let compiler = pack::PackCompiler::new();
    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: crate_root().join("fixtures/pack/valid/profile_file_backed.toml"),
            format_hint: None,
        })
        .expect("file-backed profile");

    let resolved = compiler
        .resolve_profile_topology(&profile)
        .expect("file topology should resolve");

    let boundary = resolved.boundary_taxonomy.expect("boundary taxonomy");
    let component = resolved.component_map.expect("component map");

    assert_eq!(boundary.header.id.as_str(), "acme/boundaries");
    assert_eq!(component.header.id.as_str(), "acme/components");
    assert_eq!(
        normalize_display_path(boundary.header.origin.display()),
        normalize_display_path(boundary_fixture_display_path())
    );
    assert_eq!(
        normalize_display_path(component.header.origin.display()),
        normalize_display_path(component_fixture_display_path())
    );
}

#[test]
fn file_backed_profile_topology_resolution_is_stable_across_cwd_changes() {
    let compiler = pack::PackCompiler::new();
    let cwd_guard = CwdGuard::acquire();
    cwd_guard.set_current_dir(crate_root());

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: PathBuf::from("fixtures/pack/valid/profile_file_backed.toml"),
            format_hint: None,
        })
        .expect("file-backed profile from relative path");

    cwd_guard.set_current_dir(std::env::temp_dir());

    let resolved = compiler
        .resolve_profile_topology(&profile)
        .expect("topology resolution should ignore later cwd changes");

    let boundary = resolved.boundary_taxonomy.expect("boundary taxonomy");
    let component = resolved.component_map.expect("component map");
    assert_eq!(boundary.header.id.as_str(), "acme/boundaries");
    assert_eq!(component.header.id.as_str(), "acme/components");
}

#[test]
fn topology_resolution_supports_zero_one_or_two_slots() {
    let compiler = pack::PackCompiler::new();

    let zero = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "zero-topology".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "acme/zero"
name = "Zero topology"
"#
            .to_vec(),
        })
        .expect("zero topology profile");
    let zero_resolved = compiler
        .resolve_profile_topology(&zero)
        .expect("zero topology should resolve");
    assert!(zero_resolved.boundary_taxonomy.is_none());
    assert!(zero_resolved.component_map.is_none());

    let one = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "one-topology".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "acme/one"
name = "One topology"

[topology]
boundary_taxonomy = "builtin:generic/boundaries"
"#
            .to_vec(),
        })
        .expect("one topology profile");
    let one_resolved = compiler
        .resolve_profile_topology(&one)
        .expect("one topology should resolve");
    assert!(one_resolved.boundary_taxonomy.is_some());
    assert!(one_resolved.component_map.is_none());

    let two = compiler
        .compile_profile(pack::builtin::profile_source("generic/default").expect("builtin"))
        .expect("builtin profile");
    let two_resolved = compiler
        .resolve_profile_topology(&two)
        .expect("two topology slots should resolve");
    assert!(two_resolved.boundary_taxonomy.is_some());
    assert!(two_resolved.component_map.is_some());
}

#[test]
fn missing_topology_ref_fails_with_unknown_pack_reference() {
    let compiler = pack::PackCompiler::new();
    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path: write_temp_profile(
                "missing-topology.toml",
                r#"kind = "profile"
version = 1
id = "acme/missing"
name = "Missing topology"

[topology]
boundary_taxonomy = "file:topology/does-not-exist.json"
"#,
            ),
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_topology(&profile)
        .expect_err("missing topology ref should fail");

    match error {
        pack::PackError::UnknownPackReference {
            referring_pack,
            reference,
        } => {
            assert_eq!(referring_pack, "acme/missing");
            assert_eq!(reference, "file:topology/does-not-exist.json");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn topology_resolution_surfaces_filesystem_errors_from_try_exists() {
    let compiler = pack::PackCompiler::new();
    let dir = unique_temp_dir("topology-io-error");
    let path = dir.join("io-error.toml");
    let blocking_segment = dir.join("topology");
    let expected_ref_path = dir.join("topology/component-map.json");

    fs::write(
        &path,
        r#"kind = "profile"
version = 1
id = "acme/io-error"
name = "Topology IO error"

[topology]
component_map = "file:topology/component-map.json"
"#,
    )
    .expect("write temp profile");
    fs::write(&blocking_segment, "not a directory").expect("write blocking segment");

    let profile = compiler
        .compile_profile(pack::PackSource::File {
            path,
            format_hint: None,
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_topology(&profile)
        .expect_err("topology resolution should surface path I/O failures");

    match error {
        pack::PackError::Io { origin, reason } => {
            assert_eq!(origin, expected_ref_path.display().to_string());
            assert!(!reason.is_empty(), "io reason should not be empty");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn wrong_kind_topology_ref_fails_with_ref_kind_mismatch() {
    let compiler = pack::PackCompiler::new();
    let profile = compiler
        .compile_profile(pack::PackSource::Inline {
            logical_name: "wrong-kind".to_owned(),
            format: pack::PackFormat::Toml,
            bytes: br#"kind = "profile"
version = 1
id = "acme/wrong-kind"
name = "Wrong kind"

[topology]
boundary_taxonomy = "builtin:generic/components"
"#
            .to_vec(),
        })
        .expect("profile should compile");

    let error = compiler
        .resolve_profile_topology(&profile)
        .expect_err("wrong kind should fail");

    match error {
        pack::PackError::RefKindMismatch {
            reference,
            expected,
            actual,
        } => {
            assert_eq!(reference, "builtin:generic/components");
            assert_eq!(expected, pack::PackKind::BoundaryTaxonomy);
            assert_eq!(actual, pack::PackKind::ComponentMap);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

fn write_temp_profile(name: &str, contents: &str) -> PathBuf {
    let dir = unique_temp_dir(name);
    let topology_dir = dir.join("topology");
    fs::create_dir_all(&topology_dir).expect("create topology dir");
    let path = dir.join(name);
    fs::write(&path, contents).expect("write temp profile");
    path
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("substrate-lift-{nanos}-{name}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn cwd_test_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct CwdGuard {
    _lock: MutexGuard<'static, ()>,
    previous: PathBuf,
}

impl CwdGuard {
    fn acquire() -> Self {
        let lock = cwd_test_mutex().lock().expect("cwd mutex poisoned");
        let previous = std::env::current_dir().expect("capture current dir");
        Self {
            _lock: lock,
            previous,
        }
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) {
        std::env::set_current_dir(path).expect("set current dir");
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous).expect("restore current dir");
    }
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn boundary_fixture_display_path() -> String {
    crate_root()
        .join("fixtures/pack/valid/profile_file_backed.toml")
        .parent()
        .expect("fixture parent should exist")
        .join("topology/boundary-taxonomy.json")
        .display()
        .to_string()
}

fn component_fixture_display_path() -> String {
    crate_root()
        .join("fixtures/pack/valid/profile_file_backed.toml")
        .parent()
        .expect("fixture parent should exist")
        .join("topology/component-map.json")
        .display()
        .to_string()
}
