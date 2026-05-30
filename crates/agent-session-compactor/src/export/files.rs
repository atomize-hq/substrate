use std::fs::{self, File};
use std::io::{BufWriter, Write};

use camino::Utf8Path;
use time::OffsetDateTime;

use crate::export::{BundleManifest, ExportBundleRequest, ExportError};

const SCHEMA_VERSION: &str = "v0.1";
const STAGING_DIR_LABEL: &str = "staging";
const BACKUP_DIR_LABEL: &str = "backup";
const INJECT_FAILURE_ENV: &str = "AGENT_SESSION_COMPACTOR_EXPORT_FAIL_AT";

pub fn export_bundle(request: &ExportBundleRequest) -> Result<BundleManifest, ExportError> {
    let paths = BundlePaths::new(request.output_dir)?;
    fs::create_dir_all(&paths.parent_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: paths.parent_dir.clone(),
        source,
    })?;
    fs::create_dir_all(&paths.staging_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: paths.staging_dir.clone(),
        source,
    })?;

    let manifest = BundleManifest {
        schema_version: SCHEMA_VERSION.to_string(),
        generated_at: request.generated_at,
        codex_home: request.codex_home.to_owned(),
        output_dir: request.output_dir.to_owned(),
        discovered_file_count: request.source_files.len(),
        archival_row_count: request.archival_rows.len(),
        compact_row_count: request.compact_rows.len(),
        dedupe_group_count: request.dedupe_groups.len(),
        session_ids: request.session_ids.clone(),
        source_files: request.source_files.to_vec(),
    };

    write_jsonl_file(
        &paths.staging_dir.join("rows.archival.jsonl"),
        request.archival_rows,
    )?;
    maybe_inject_failure("after_rows_archival")?;
    write_jsonl_file(
        &paths.staging_dir.join("rows.compact.jsonl"),
        request.compact_rows,
    )?;
    maybe_inject_failure("after_rows_compact")?;
    write_jsonl_file(
        &paths.staging_dir.join("dedupe-audit.jsonl"),
        request.dedupe_groups,
    )?;
    maybe_inject_failure("after_dedupe_audit")?;
    write_summary(&paths.staging_dir.join("summary.md"), &manifest)?;
    maybe_inject_failure("after_summary")?;
    maybe_inject_failure("before_manifest")?;
    write_json_file(&paths.staging_dir.join("manifest.json"), &manifest)?;
    maybe_inject_failure("before_publish")?;
    publish_bundle(&paths)?;

    Ok(manifest)
}

struct BundlePaths {
    parent_dir: camino::Utf8PathBuf,
    output_dir: camino::Utf8PathBuf,
    staging_dir: camino::Utf8PathBuf,
    backup_dir: camino::Utf8PathBuf,
}

impl BundlePaths {
    fn new(output_dir: &Utf8Path) -> Result<Self, ExportError> {
        let bundle_name = output_dir.file_name().ok_or_else(|| ExportError::InvalidOutputDirectory {
            path: output_dir.to_owned(),
        })?;
        let parent_dir = output_parent_dir(output_dir);
        Ok(Self {
            parent_dir: parent_dir.clone(),
            output_dir: output_dir.to_owned(),
            staging_dir: unique_sibling_dir(&parent_dir, bundle_name, STAGING_DIR_LABEL),
            backup_dir: unique_sibling_dir(&parent_dir, bundle_name, BACKUP_DIR_LABEL),
        })
    }
}

fn output_parent_dir(output_dir: &Utf8Path) -> camino::Utf8PathBuf {
    output_dir
        .parent()
        .filter(|path| !path.as_str().is_empty())
        .unwrap_or_else(|| Utf8Path::new("."))
        .to_owned()
}

fn unique_sibling_dir(parent_dir: &Utf8Path, bundle_name: &str, label: &str) -> camino::Utf8PathBuf {
    let timestamp_nanos = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let pid = std::process::id();
    for attempt in 0..1024 {
        let candidate =
            parent_dir.join(format!(".{bundle_name}.{label}-{timestamp_nanos}-{pid}-{attempt}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    parent_dir.join(format!(".{bundle_name}.{label}-{timestamp_nanos}-{pid}-overflow"))
}

fn publish_bundle(paths: &BundlePaths) -> Result<(), ExportError> {
    let mut previous_bundle = None;
    if paths.output_dir.exists() {
        fs::rename(&paths.output_dir, &paths.backup_dir).map_err(|source| {
            ExportError::PublishOutputDirectory {
                path: paths.output_dir.clone(),
                source,
            }
        })?;
        previous_bundle = Some(paths.backup_dir.clone());
    }

    match fs::rename(&paths.staging_dir, &paths.output_dir) {
        Ok(()) => {
            if let Some(previous_bundle) = previous_bundle.as_ref() {
                let _ = fs::remove_dir_all(previous_bundle);
            }
            Ok(())
        }
        Err(source) => {
            if let Some(previous_bundle) = previous_bundle.as_ref() {
                let _ = fs::rename(previous_bundle, &paths.output_dir);
            }
            Err(ExportError::PublishOutputDirectory {
                path: paths.output_dir.clone(),
                source,
            })
        }
    }
}

fn maybe_inject_failure(point: &str) -> Result<(), ExportError> {
    match std::env::var(INJECT_FAILURE_ENV) {
        Ok(requested) if requested == point => Err(ExportError::InjectedFailure {
            point: point.to_string(),
        }),
        _ => Ok(()),
    }
}

fn write_json_file<T: serde::Serialize>(
    path: &Utf8Path,
    value: &T,
) -> Result<(), ExportError> {
    let file = File::create(path).map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value).map_err(|source| ExportError::Serialize {
        path: path.to_owned(),
        source,
    })?;
    writer.flush().map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })
}

fn write_jsonl_file<T: serde::Serialize>(
    path: &Utf8Path,
    rows: &[T],
) -> Result<(), ExportError> {
    let file = File::create(path).map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })?;
    let mut writer = BufWriter::new(file);
    for row in rows {
        serde_json::to_writer(&mut writer, row).map_err(|source| ExportError::Serialize {
            path: path.to_owned(),
            source,
        })?;
        writer
            .write_all(b"\n")
            .map_err(|source| ExportError::WriteFile {
                path: path.to_owned(),
                source,
            })?;
    }
    writer.flush().map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })
}

fn write_summary(path: &Utf8Path, manifest: &BundleManifest) -> Result<(), ExportError> {
    let mut file = File::create(path).map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })?;
    let generated_at = manifest
        .generated_at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::UNIX_EPOCH.to_string());
    let summary = format!(
        "# Agent Session Compactor Summary\n\n\
Schema version: `{}`\n\
Generated at: `{}`\n\
Codex home: `{}`\n\
Source files: `{}`\n\
Session ids: `{}`\n\
Archival rows: `{}`\n\
Compact rows: `{}`\n\
Dedupe groups: `{}`\n",
        manifest.schema_version,
        generated_at,
        manifest.codex_home,
        manifest.discovered_file_count,
        if manifest.session_ids.is_empty() {
            "(none)".to_string()
        } else {
            manifest.session_ids.join(", ")
        },
        manifest.archival_row_count,
        manifest.compact_row_count,
        manifest.dedupe_group_count
    );
    file.write_all(summary.as_bytes())
        .map_err(|source| ExportError::WriteFile {
            path: path.to_owned(),
            source,
        })?;
    file.flush().map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })
}
