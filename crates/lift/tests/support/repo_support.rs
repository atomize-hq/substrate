#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use jsonschema::{Retrieve, Uri, Validator};
use serde_json::{json, Value};

use crate::repo;

pub(crate) struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let base = Path::new("/tmp");
        let path = if base.exists() {
            base.to_path_buf()
        } else {
            std::env::temp_dir()
        }
        .join(format!(
            "substrate-lift-{prefix}-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp dir should be creatable");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

pub(crate) fn fixture_path(relative: &str) -> PathBuf {
    crate_root().join(relative)
}

pub(crate) fn load_json(relative: &str) -> Value {
    let path = fixture_path(relative);
    let contents = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });

    serde_json::from_str(&contents).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

pub(crate) fn load_text(relative: &str) -> String {
    let path = fixture_path(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

pub(crate) fn copy_fixture_tree(relative: &str, prefix: &str) -> TempDir {
    let src = fixture_path(relative);
    let temp = TempDir::new(prefix);
    copy_dir_all(&src, temp.path());
    temp
}

pub(crate) fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("destination should be creatable");
    for entry in fs::read_dir(src).expect("source should be readable") {
        let entry = entry.expect("dir entry should be readable");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type().expect("file type should be readable");
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap_or_else(|error| {
                panic!(
                    "failed to copy fixture {} -> {}: {error}",
                    src_path.display(),
                    dst_path.display()
                );
            });
        }
    }
}

pub(crate) fn write_file(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir should be creatable");
    }
    fs::write(path, contents).unwrap_or_else(|error| {
        panic!("failed to write {}: {error}", path.display());
    });
}

pub(crate) fn default_snapshot_options() -> repo::SnapshotOptions {
    repo::SnapshotOptions::default()
}

pub(crate) fn snapshot_request(
    root: &Path,
    options: repo::SnapshotOptions,
) -> repo::SnapshotRequest {
    repo::SnapshotRequest {
        root: repo::root::RepoRoot::from_dir(root).expect("repo root should parse"),
        source: repo::SnapshotSource::Worktree,
        options,
    }
}

pub(crate) fn materialize(root: &Path, options: repo::SnapshotOptions) -> repo::RepoSnapshot {
    repo::materialize_snapshot(&snapshot_request(root, options)).expect("snapshot should build")
}

pub(crate) fn inventory_paths(snapshot: &repo::RepoSnapshot) -> Vec<String> {
    snapshot
        .inventory
        .iter()
        .map(|entry| entry.path.as_str().to_owned())
        .collect()
}

pub(crate) fn snapshot_validator() -> Validator {
    let root_schema: Value = serde_json::from_str(repo::schema::SNAPSHOT_MANIFEST_V1_SCHEMA_JSON)
        .expect("embedded repo schema should parse");
    let kernel_schema: Value =
        serde_json::from_str(crate::kernel::PRIMITIVES_V1_SCHEMA_JSON).expect("kernel schema");
    let retriever = InMemoryRetriever {
        schemas: HashMap::from([
            (
                crate::kernel::PRIMITIVES_V1_SCHEMA_ID.to_owned(),
                kernel_schema.clone(),
            ),
            ("../kernel/primitives.v1.json".to_owned(), kernel_schema),
        ]),
    };

    jsonschema::draft202012::options()
        .with_retriever(retriever)
        .build(&root_schema)
        .expect("repo schema should compile")
}

pub(crate) fn manifest_from_snapshot(
    case: &str,
    options: &repo::SnapshotOptions,
    snapshot: &repo::RepoSnapshot,
) -> Value {
    json!({
        "version": 1,
        "case": case,
        "source_kind": "worktree",
        "options": {
            "symlink_policy": match options.symlink_policy {
                repo::SymlinkPolicy::Skip => "skip",
            },
            "exclude_globs": options.exclude_globs,
            "non_utf8_path_policy": match options.non_utf8_path_policy {
                repo::NonUtf8PathPolicy::Error => "error",
                repo::NonUtf8PathPolicy::Skip => "skip",
            },
            "max_file_bytes": options.max_file_bytes,
            "large_file_policy": match options.large_file_policy {
                repo::LargeFilePolicy::Error => "error",
                repo::LargeFilePolicy::Skip => "skip",
            },
        },
        "files": snapshot.inventory.iter().map(|entry| {
            json!({
                "path": entry.path.as_str(),
                "file_id": entry.file_id.as_str(),
                "blob_fingerprint": entry.blob_fingerprint.as_str(),
                "size_bytes": entry.size_bytes,
            })
        }).collect::<Vec<_>>(),
        "snapshot_fingerprint": snapshot.fingerprint.as_str(),
        "stats": {
            "file_count": snapshot.stats.file_count,
            "total_bytes": snapshot.stats.total_bytes,
            "skipped_by_ignore": snapshot.stats.skipped_by_ignore,
            "skipped_symlinks": snapshot.stats.skipped_symlinks,
            "skipped_non_utf8_paths": snapshot.stats.skipped_non_utf8_paths,
            "skipped_large_files": snapshot.stats.skipped_large_files,
            "skipped_unsupported_file_kinds": snapshot.stats.skipped_unsupported_file_kinds,
        }
    })
}

#[derive(Clone, Debug)]
struct InMemoryRetriever {
    schemas: HashMap<String, Value>,
}

impl Retrieve for InMemoryRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.schemas
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("schema not found: {uri}").into())
    }
}
