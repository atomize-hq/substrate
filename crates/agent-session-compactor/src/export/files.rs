use std::fs::{self, File};
use std::io::{BufWriter, Write};

use camino::Utf8Path;
use time::OffsetDateTime;

use crate::export::{BundleManifest, ExportBundleRequest, ExportError};

const SCHEMA_VERSION: &str = "v0.1";

pub fn export_bundle(request: &ExportBundleRequest) -> Result<BundleManifest, ExportError> {
    fs::create_dir_all(request.output_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: request.output_dir.to_owned(),
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

    write_json_file(&request.output_dir.join("manifest.json"), &manifest)?;
    write_jsonl_file(&request.output_dir.join("rows.archival.jsonl"), request.archival_rows)?;
    write_jsonl_file(&request.output_dir.join("rows.compact.jsonl"), request.compact_rows)?;
    write_jsonl_file(
        &request.output_dir.join("dedupe-audit.jsonl"),
        request.dedupe_groups,
    )?;
    write_summary(&request.output_dir.join("summary.md"), &manifest)?;

    Ok(manifest)
}

fn write_json_file<T: serde::Serialize>(
    path: &Utf8Path,
    value: &T,
) -> Result<(), ExportError> {
    let file = File::create(path).map_err(|source| ExportError::WriteFile {
        path: path.to_owned(),
        source,
    })?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value).map_err(|source| ExportError::Serialize {
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
        })
}
