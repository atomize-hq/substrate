use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};

use camino::{Utf8Path, Utf8PathBuf};
use time::OffsetDateTime;

use crate::dedupe::{DedupeGroup, RowRef};
use crate::export::{
    BundleFileV0_2, BundleManifest, DedupeGroupV0_2, ExportBundleRequest, ExportError,
    ExportRowV0_2, RowRefV0_2,
};
use crate::normalize::CompactionRow;

const SCHEMA_VERSION: &str = "v0.2";
const STAGING_DIR_LABEL: &str = "staging";
const BACKUP_DIR_LABEL: &str = "backup";
const INJECT_FAILURE_ENV: &str = "AGENT_SESSION_COMPACTOR_EXPORT_FAIL_AT";

pub fn export_bundle(request: &ExportBundleRequest) -> Result<BundleManifest, ExportError> {
    let paths = BundlePaths::new(request.output_dir)?;
    fs::create_dir_all(&paths.parent_dir).map_err(|source| ExportError::CreateOutputDirectory {
        path: paths.parent_dir.clone(),
        source,
    })?;
    fs::create_dir_all(&paths.staging_dir).map_err(|source| {
        ExportError::CreateOutputDirectory {
            path: paths.staging_dir.clone(),
            source,
        }
    })?;

    let file_registry = FileRegistry::build(
        request.archival_rows,
        request.compact_rows,
        request.dedupe_groups,
    )?;
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
        files: file_registry.files.clone(),
    };
    let archival_rows = export_rows(request.archival_rows, &file_registry)?;
    let compact_rows = export_rows(request.compact_rows, &file_registry)?;
    let dedupe_groups = export_dedupe_groups(request.dedupe_groups, &file_registry)?;

    write_jsonl_file(
        &paths.staging_dir.join("rows.archival.jsonl"),
        &archival_rows,
    )?;
    maybe_inject_failure("after_rows_archival")?;
    write_jsonl_file(&paths.staging_dir.join("rows.compact.jsonl"), &compact_rows)?;
    maybe_inject_failure("after_rows_compact")?;
    write_jsonl_file(
        &paths.staging_dir.join("dedupe-audit.jsonl"),
        &dedupe_groups,
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
        let bundle_name =
            output_dir
                .file_name()
                .ok_or_else(|| ExportError::InvalidOutputDirectory {
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

fn unique_sibling_dir(
    parent_dir: &Utf8Path,
    bundle_name: &str,
    label: &str,
) -> camino::Utf8PathBuf {
    let timestamp_nanos = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let pid = std::process::id();
    for attempt in 0..1024 {
        let candidate = parent_dir.join(format!(
            ".{bundle_name}.{label}-{timestamp_nanos}-{pid}-{attempt}"
        ));
        if !candidate.exists() {
            return candidate;
        }
    }

    parent_dir.join(format!(
        ".{bundle_name}.{label}-{timestamp_nanos}-{pid}-overflow"
    ))
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

fn write_json_file<T: serde::Serialize>(path: &Utf8Path, value: &T) -> Result<(), ExportError> {
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

fn write_jsonl_file<T: serde::Serialize>(path: &Utf8Path, rows: &[T]) -> Result<(), ExportError> {
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

fn export_rows(
    rows: &[CompactionRow],
    file_registry: &FileRegistry,
) -> Result<Vec<ExportRowV0_2>, ExportError> {
    rows.iter()
        .map(|row| {
            Ok(ExportRowV0_2 {
                source_file_id: file_registry.source_file_id(&row.source_file)?,
                source_kind: row.source_kind,
                turn_id_ref: file_registry.turn_id_ref(&row.source_file, row.turn_id.as_deref())?,
                event_index: row.event_index,
                row_ordinal: row.row_ordinal,
                timestamp: row.timestamp,
                kind: row.kind,
                user_message_role: row.user_message_role,
                dedupe_identity: row.dedupe_identity.clone(),
                text: row.text.clone(),
                text_hash_hex: row.text_hash_hex.clone(),
            })
        })
        .collect()
}

fn export_dedupe_groups(
    groups: &[DedupeGroup],
    file_registry: &FileRegistry,
) -> Result<Vec<DedupeGroupV0_2>, ExportError> {
    groups
        .iter()
        .map(|group| {
            Ok(DedupeGroupV0_2 {
                kind: group.kind,
                canonical_text_hash_hex: group.canonical_text_hash_hex.clone(),
                representative: export_row_ref(&group.representative, file_registry)?,
                duplicates: group
                    .duplicates
                    .iter()
                    .map(|row_ref| export_row_ref(row_ref, file_registry))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect()
}

fn export_row_ref(
    row_ref: &RowRef,
    file_registry: &FileRegistry,
) -> Result<RowRefV0_2, ExportError> {
    Ok(RowRefV0_2 {
        source_file_id: file_registry.source_file_id(&row_ref.source_file)?,
        event_index: row_ref.event_index,
        row_ordinal: row_ref.row_ordinal,
    })
}

#[derive(Debug, Clone)]
struct FileRegistry {
    files: Vec<BundleFileV0_2>,
    ids_by_path: BTreeMap<Utf8PathBuf, u32>,
    turn_ids_by_path: BTreeMap<Utf8PathBuf, BTreeMap<String, u16>>,
}

impl FileRegistry {
    fn build(
        archival_rows: &[CompactionRow],
        compact_rows: &[CompactionRow],
        dedupe_groups: &[DedupeGroup],
    ) -> Result<Self, ExportError> {
        let mut sessions_by_path: BTreeMap<Utf8PathBuf, Option<String>> = BTreeMap::new();
        let mut turns_by_path: BTreeMap<Utf8PathBuf, BTreeSet<String>> = BTreeMap::new();
        for row in archival_rows.iter().chain(compact_rows.iter()) {
            match sessions_by_path.entry(row.source_file.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(row.session_id.clone());
                }
                std::collections::btree_map::Entry::Occupied(entry)
                    if entry.get() != &row.session_id =>
                {
                    return Err(ExportError::ConflictingSourceFileSessionIds {
                        path: row.source_file.clone(),
                        left: entry.get().clone(),
                        right: row.session_id.clone(),
                    });
                }
                std::collections::btree_map::Entry::Occupied(_) => {}
            }
            if let Some(turn_id) = row.turn_id.as_ref() {
                turns_by_path
                    .entry(row.source_file.clone())
                    .or_default()
                    .insert(turn_id.clone());
            }
        }

        let mut paths = sessions_by_path.keys().cloned().collect::<BTreeSet<_>>();
        for group in dedupe_groups {
            paths.insert(group.representative.source_file.clone());
            for duplicate in &group.duplicates {
                paths.insert(duplicate.source_file.clone());
            }
        }

        let mut files = Vec::with_capacity(paths.len());
        let mut ids_by_path = BTreeMap::new();
        let mut turn_ids_by_path = BTreeMap::new();
        for (index, path) in paths.into_iter().enumerate() {
            let id = u32::try_from(index).map_err(|_| ExportError::SourceFileIdOverflow)?;
            let turns = turns_by_path
                .remove(&path)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let mut turn_ids = BTreeMap::new();
            for (turn_index, turn_id) in turns.iter().enumerate() {
                let turn_id_ref = u16::try_from(turn_index)
                    .map_err(|_| ExportError::SourceFileTurnIdOverflow { path: path.clone() })?;
                turn_ids.insert(turn_id.clone(), turn_id_ref);
            }
            files.push(BundleFileV0_2 {
                id,
                path: path.clone(),
                session_id: sessions_by_path.get(&path).cloned().unwrap_or(None),
                turns,
            });
            ids_by_path.insert(path, id);
            turn_ids_by_path.insert(files.last().expect("file").path.clone(), turn_ids);
        }

        Ok(Self {
            files,
            ids_by_path,
            turn_ids_by_path,
        })
    }

    fn source_file_id(&self, path: &Utf8Path) -> Result<u32, ExportError> {
        self.ids_by_path
            .get(path)
            .copied()
            .ok_or_else(|| ExportError::UnregisteredSourceFile {
                path: path.to_owned(),
            })
    }

    fn turn_id_ref(
        &self,
        path: &Utf8Path,
        turn_id: Option<&str>,
    ) -> Result<Option<u16>, ExportError> {
        let Some(turn_id) = turn_id else {
            return Ok(None);
        };
        self.turn_ids_by_path
            .get(path)
            .and_then(|turn_ids| turn_ids.get(turn_id).copied())
            .map(Some)
            .ok_or_else(|| ExportError::UnregisteredTurnId {
                path: path.to_owned(),
                turn_id: turn_id.to_string(),
            })
    }
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
