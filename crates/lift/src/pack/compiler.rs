//! Seam-1 pack compiler spine.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use globset::{Glob, GlobSet, GlobSetBuilder};
use jsonschema::error::ValidationErrorKind;
use jsonschema::{Retrieve, Uri, ValidationError, Validator};
use serde::Serialize;
use serde_json::{Map, Number, Value};

use crate::kernel::{sha256_bytes, sha256_canonical_json, Fingerprint, JsonPointer};
use crate::pack::builtin;
use crate::pack::compiled::{
    BoundaryCountingMode, CompiledAnalysisDefaults, CompiledBoundary, CompiledBoundaryTaxonomy,
    CompiledComponent, CompiledComponentMap, CompiledPackHeader, CompiledPathClasses,
    CompiledProfile, CompiledProfileApps, CompiledProfileIncludes, CompiledProfileScore,
    CompiledProfileTopology, ComponentCountingMode, ResolvedProfileTopology,
};
use crate::pack::diagnostics::{PackDiagnostic, PackLocation};
use crate::pack::error::{PackError, PackResult};
use crate::pack::names::{AppName, LanguageId, PackName};
use crate::pack::raw::{
    PackKind, RawBoundaryEntry, RawBoundaryTaxonomy, RawBoundaryTaxonomyCountingMode,
    RawComponentEntry, RawComponentMap, RawComponentMapCountingMode, RawIncludeSection, RawProfile,
    RawProfileAnalysis, RawProfileApps, RawProfileScore, RawProfileTopology,
};
use crate::pack::refs::PackRef;
use crate::pack::schema::{
    PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID, PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_JSON,
    PACK_COMMON_V1_SCHEMA_ID, PACK_COMMON_V1_SCHEMA_JSON, PACK_COMPONENT_MAP_V1_SCHEMA_ID,
    PACK_COMPONENT_MAP_V1_SCHEMA_JSON, PACK_PROFILE_V1_SCHEMA_ID,
};
use crate::pack::source::{PackFormat, PackOrigin, PackSource};
use crate::pack::{BoundaryId, ComponentId};

const DEFAULT_APP: &str = "score";
const DEFAULT_LANGUAGES: &[&str] = &[
    "javascript",
    "json",
    "python",
    "rust",
    "toml",
    "typescript",
    "yaml",
];
const DEFAULT_FOLLOW_SYMLINKS: bool = false;
const DEFAULT_MAX_SCOPE_DEPTH: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TopologySchemaKind {
    BoundaryTaxonomy,
    ComponentMap,
}

impl TopologySchemaKind {
    fn invalid_root_code(self) -> &'static str {
        match self {
            Self::BoundaryTaxonomy => "pack.boundary_taxonomy.invalid_root",
            Self::ComponentMap => "pack.component_map.invalid_root",
        }
    }

    fn validator(self) -> &'static Validator {
        match self {
            Self::BoundaryTaxonomy => boundary_taxonomy_schema_validator(),
            Self::ComponentMap => component_map_schema_validator(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct EmbeddedSchemaRetriever;

impl Retrieve for EmbeddedSchemaRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        match uri.as_str() {
            PACK_COMMON_V1_SCHEMA_ID | "common.v1.json" => {
                serde_json::from_str(PACK_COMMON_V1_SCHEMA_JSON).map_err(|error| {
                    format!("failed to parse embedded common schema: {error}").into()
                })
            }
            other => Err(format!("schema not found: {other}").into()),
        }
    }
}

/// Profile-only compiler entrypoint for Phase A.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackCompiler;

impl PackCompiler {
    /// Creates a new pack compiler.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Compiles one standalone profile from builtin, file, or inline sources.
    pub(crate) fn compile_profile(&self, source: PackSource) -> PackResult<CompiledProfile> {
        let loaded = self.load_source(source)?;
        let LoadedSource {
            origin,
            format,
            bytes,
            file_base_dir,
        } = loaded;
        if format != PackFormat::Toml {
            return Err(PackError::UnsupportedFormat {
                origin: origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&bytes);
        let json_value = parse_profile_toml(&origin, &bytes)?;
        let mut diagnostics = validate_profile_document(&origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: origin.display(),
                schema_id: PACK_PROFILE_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw_profile: RawProfile =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: origin.display(),
                schema_id: PACK_PROFILE_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.profile.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        let normalized = normalize_profile(&raw_profile);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.profile.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        compile_normalized_profile(
            origin,
            file_base_dir,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    /// Compiles one standalone boundary taxonomy from builtin, file, or inline JSON sources.
    pub(crate) fn compile_boundary_taxonomy(
        &self,
        source: PackSource,
    ) -> PackResult<CompiledBoundaryTaxonomy> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "boundary_taxonomy")?;
        let mut diagnostics = validate_boundary_taxonomy_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawBoundaryTaxonomy =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.boundary_taxonomy.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        let normalized = normalize_boundary_taxonomy(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.boundary_taxonomy.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        compile_normalized_boundary_taxonomy(
            loaded.origin,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    /// Compiles one standalone component map from builtin, file, or inline JSON sources.
    pub(crate) fn compile_component_map(
        &self,
        source: PackSource,
    ) -> PackResult<CompiledComponentMap> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "component_map")?;
        let mut diagnostics = validate_component_map_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_COMPONENT_MAP_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawComponentMap =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_COMPONENT_MAP_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.component_map.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        let normalized = normalize_component_map(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.component_map.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        compile_normalized_component_map(
            loaded.origin,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    /// Resolves the profile topology refs into compiled topology artifacts.
    pub(crate) fn resolve_profile_topology(
        &self,
        profile: &CompiledProfile,
    ) -> PackResult<ResolvedProfileTopology> {
        let boundary_taxonomy = profile
            .topology
            .boundary_taxonomy
            .as_ref()
            .map(|reference| resolve_boundary_taxonomy(self, profile, reference))
            .transpose()?;
        let component_map = profile
            .topology
            .component_map
            .as_ref()
            .map(|reference| resolve_component_map(self, profile, reference))
            .transpose()?;

        let fingerprint_input = [
            boundary_taxonomy
                .as_ref()
                .map(|compiled| compiled.header.semantic_fingerprint.as_str().to_owned()),
            component_map
                .as_ref()
                .map(|compiled| compiled.header.semantic_fingerprint.as_str().to_owned()),
        ];
        let semantic_fingerprint =
            sha256_canonical_json(&fingerprint_input).map_err(|error| PackError::ParseFailure {
                origin: profile.header.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.profile_topology.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: profile.header.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        Ok(ResolvedProfileTopology {
            boundary_taxonomy,
            component_map,
            semantic_fingerprint,
        })
    }

    fn load_source(&self, source: PackSource) -> PackResult<LoadedSource> {
        match source {
            PackSource::Builtin {
                logical_name,
                format,
                bytes,
            } => Ok(LoadedSource {
                origin: PackOrigin::Builtin {
                    logical_name: logical_name.to_owned(),
                },
                format,
                bytes: bytes.to_vec(),
                file_base_dir: None,
            }),
            PackSource::File { path, format_hint } => {
                let bytes = fs::read(&path).map_err(|error| PackError::Io {
                    origin: path.display().to_string(),
                    reason: error.to_string(),
                })?;
                let resolved_path = absolutize_path(&path)?;
                let display_path = path.display().to_string();
                let format = match format_hint.or_else(|| infer_file_format(&path)) {
                    Some(format) => format,
                    None => {
                        return Err(PackError::UnsupportedFormat {
                            origin: display_path,
                        });
                    }
                };
                Ok(LoadedSource {
                    origin: PackOrigin::File { display_path },
                    format,
                    bytes,
                    file_base_dir: resolved_path.parent().map(Path::to_path_buf),
                })
            }
            PackSource::Inline {
                logical_name,
                format,
                bytes,
            } => Ok(LoadedSource {
                origin: PackOrigin::Inline { logical_name },
                format,
                bytes,
                file_base_dir: None,
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoadedSource {
    origin: PackOrigin,
    format: PackFormat,
    bytes: Vec<u8>,
    file_base_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedProfileDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    apps: NormalizedProfileApps,
    analysis: NormalizedProfileAnalysis,
    topology: NormalizedProfileTopology,
    score: NormalizedProfileScore,
    rules: NormalizedIncludeSection,
    queries: NormalizedIncludeSection,
    recipes: NormalizedIncludeSection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedProfileApps {
    enabled: Vec<String>,
    default: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedProfileAnalysis {
    languages: Vec<String>,
    follow_symlinks: bool,
    max_scope_depth: u8,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
struct NormalizedProfileTopology {
    #[serde(skip_serializing_if = "Option::is_none")]
    boundary_taxonomy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_map: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
struct NormalizedProfileScore {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
struct NormalizedIncludeSection {
    packs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedBoundaryTaxonomyDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    counting: NormalizedBoundaryTaxonomyCounting,
    boundaries: Vec<NormalizedBoundaryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedBoundaryTaxonomyCounting {
    mode: RawBoundaryTaxonomyCountingMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedBoundaryEntry {
    id: String,
    label: String,
    include: Vec<String>,
    exclude: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedComponentMapDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    counting: NormalizedComponentMapCounting,
    components: Vec<NormalizedComponentEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedComponentMapCounting {
    mode: RawComponentMapCountingMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedComponentEntry {
    id: String,
    label: String,
    include: Vec<String>,
    exclude: Vec<String>,
    tags: Vec<String>,
}

type ArrayItemValidator = fn(&PackOrigin, &str, &str, &mut Vec<PackDiagnostic>);

fn infer_file_format(path: &Path) -> Option<PackFormat> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("toml") => Some(PackFormat::Toml),
        Some("json") => Some(PackFormat::Json),
        _ => None,
    }
}

fn parse_profile_toml(origin: &PackOrigin, bytes: &[u8]) -> PackResult<Value> {
    let text = std::str::from_utf8(bytes).map_err(|error| PackError::ParseFailure {
        origin: origin.display(),
        diagnostics: vec![PackDiagnostic::error(
            "pack.profile.invalid_utf8",
            error.to_string(),
            Some(PackLocation {
                origin: origin.clone(),
                path: None,
            }),
        )],
    })?;
    let toml_value: toml::Value =
        toml::from_str(text).map_err(|error| PackError::ParseFailure {
            origin: origin.display(),
            diagnostics: vec![PackDiagnostic::error(
                "pack.profile.invalid_toml",
                error.to_string(),
                Some(PackLocation {
                    origin: origin.clone(),
                    path: None,
                }),
            )],
        })?;
    toml_to_json(origin, JsonPointer::root(), &toml_value)
}

fn parse_json_document(origin: &PackOrigin, bytes: &[u8], pack_code: &str) -> PackResult<Value> {
    let text = std::str::from_utf8(bytes).map_err(|error| PackError::ParseFailure {
        origin: origin.display(),
        diagnostics: vec![PackDiagnostic::error(
            &format!("pack.{pack_code}.invalid_utf8"),
            error.to_string(),
            Some(PackLocation {
                origin: origin.clone(),
                path: None,
            }),
        )],
    })?;

    serde_json::from_str(text).map_err(|error| PackError::ParseFailure {
        origin: origin.display(),
        diagnostics: vec![PackDiagnostic::error(
            &format!("pack.{pack_code}.invalid_json"),
            error.to_string(),
            Some(PackLocation {
                origin: origin.clone(),
                path: None,
            }),
        )],
    })
}

fn toml_to_json(
    origin: &PackOrigin,
    pointer: JsonPointer,
    value: &toml::Value,
) -> PackResult<Value> {
    match value {
        toml::Value::String(string) => Ok(Value::String(string.clone())),
        toml::Value::Integer(number) => Ok(Value::Number(Number::from(*number))),
        toml::Value::Boolean(boolean) => Ok(Value::Bool(*boolean)),
        toml::Value::Float(number) => {
            Number::from_f64(*number)
                .map(Value::Number)
                .ok_or_else(|| PackError::ParseFailure {
                    origin: origin.display(),
                    diagnostics: vec![PackDiagnostic::error(
                        "pack.profile.unsupported_toml_value",
                        "non-finite TOML floats are unsupported",
                        Some(PackLocation {
                            origin: origin.clone(),
                            path: Some(pointer),
                        }),
                    )],
                })
        }
        toml::Value::Datetime(_) => Err(PackError::ParseFailure {
            origin: origin.display(),
            diagnostics: vec![PackDiagnostic::error(
                "pack.profile.unsupported_toml_value",
                "TOML datetime values are unsupported in Phase A profiles",
                Some(PackLocation {
                    origin: origin.clone(),
                    path: Some(pointer),
                }),
            )],
        }),
        toml::Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                out.push(toml_to_json(
                    origin,
                    pointer.push_token(&index.to_string()),
                    item,
                )?);
            }
            Ok(Value::Array(out))
        }
        toml::Value::Table(table) => {
            let mut out = Map::new();
            for (key, item) in table {
                out.insert(
                    key.clone(),
                    toml_to_json(origin, pointer.push_token(key), item)?,
                );
            }
            Ok(Value::Object(out))
        }
    }
}

fn boundary_taxonomy_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| compile_topology_schema_validator(PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_JSON))
}

fn component_map_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| compile_topology_schema_validator(PACK_COMPONENT_MAP_V1_SCHEMA_JSON))
}

fn compile_topology_schema_validator(schema_json: &str) -> Validator {
    let root_schema: Value =
        serde_json::from_str(schema_json).expect("embedded topology schema JSON should parse");
    jsonschema::draft202012::options()
        .with_retriever(EmbeddedSchemaRetriever)
        .build(&root_schema)
        .expect("embedded topology schema should compile")
}

fn validate_topology_document(
    origin: &PackOrigin,
    value: &Value,
    schema_kind: TopologySchemaKind,
) -> Vec<PackDiagnostic> {
    let mut diagnostics = Vec::new();
    for error in schema_kind.validator().iter_errors(value) {
        diagnostics.extend(topology_schema_error_to_diagnostics(
            origin,
            schema_kind,
            &error,
        ));
    }
    diagnostics
}

fn topology_schema_error_to_diagnostics(
    origin: &PackOrigin,
    schema_kind: TopologySchemaKind,
    error: &ValidationError<'_>,
) -> Vec<PackDiagnostic> {
    match error.kind() {
        ValidationErrorKind::AdditionalProperties { unexpected } => {
            let mut fields = sorted_unique_strings(unexpected.clone());
            fields
                .drain(..)
                .map(|field| {
                    let path = append_pointer_segment(error.instance_path().as_str(), &field);
                    PackDiagnostic::error(
                        "pack.schema.unknown_field",
                        format!("unexpected field `{field}`"),
                        Some(location(origin, &path)),
                    )
                })
                .collect()
        }
        ValidationErrorKind::Required { property } => {
            let Some(property) = property.as_str() else {
                return vec![PackDiagnostic::error(
                    "pack.schema.missing_required_field",
                    error.to_string(),
                    pointer_subject(origin, error.instance_path().as_str()),
                )];
            };
            let pointer = append_pointer_segment(error.instance_path().as_str(), property);
            vec![PackDiagnostic::error(
                missing_field_code(schema_kind, error.instance_path().as_str(), property),
                format!("required field `{property}` is missing"),
                Some(location(origin, &pointer)),
            )]
        }
        _ => vec![PackDiagnostic::error(
            topology_schema_error_code(schema_kind, error),
            error.to_string(),
            topology_schema_error_subject(origin, schema_kind, error),
        )],
    }
}

fn topology_schema_error_code(
    schema_kind: TopologySchemaKind,
    error: &ValidationError<'_>,
) -> &'static str {
    let path = error.instance_path().as_str();
    match error.kind() {
        ValidationErrorKind::Type { .. } if path.is_empty() => schema_kind.invalid_root_code(),
        ValidationErrorKind::Type { .. } => "pack.schema.invalid_type",
        ValidationErrorKind::Constant { .. } if path == "/kind" => "pack.schema.invalid_kind",
        ValidationErrorKind::Constant { .. } if path == "/version" => "pack.schema.invalid_version",
        ValidationErrorKind::Enum { .. } if path.ends_with("/counting/mode") => {
            "pack.schema.invalid_counting_mode"
        }
        ValidationErrorKind::Pattern { .. } if path == "/id" => "pack.schema.invalid_pack_name",
        ValidationErrorKind::MinLength { .. } => min_length_code(path),
        _ => "pack.schema.invalid_value",
    }
}

fn topology_schema_error_subject(
    origin: &PackOrigin,
    _schema_kind: TopologySchemaKind,
    error: &ValidationError<'_>,
) -> Option<PackLocation> {
    let path = error.instance_path().as_str();
    if path.is_empty() && matches!(error.kind(), ValidationErrorKind::Type { .. }) {
        Some(PackLocation {
            origin: origin.clone(),
            path: None,
        })
    } else {
        pointer_subject(origin, path)
    }
}

fn pointer_subject(origin: &PackOrigin, path: &str) -> Option<PackLocation> {
    if path.is_empty() {
        None
    } else {
        Some(location(origin, path))
    }
}

fn append_pointer_segment(base: &str, segment: &str) -> String {
    if base.is_empty() {
        format!("/{}", escape_json_pointer_segment(segment))
    } else {
        format!("{base}/{}", escape_json_pointer_segment(segment))
    }
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn missing_field_code(
    schema_kind: TopologySchemaKind,
    parent_path: &str,
    property: &str,
) -> &'static str {
    match (schema_kind, parent_path, property) {
        (_, "", "kind") => "pack.schema.missing_kind",
        (_, "", "version") => "pack.schema.missing_version",
        (_, "", "id") => "pack.schema.missing_id",
        (_, "", "name") => "pack.schema.missing_name",
        (TopologySchemaKind::BoundaryTaxonomy, "", "counting") => "pack.schema.missing_counting",
        (TopologySchemaKind::BoundaryTaxonomy, "", "boundaries") => {
            "pack.schema.missing_boundaries"
        }
        (TopologySchemaKind::ComponentMap, "", "counting") => "pack.schema.missing_counting",
        (TopologySchemaKind::ComponentMap, "", "components") => "pack.schema.missing_components",
        (_, "/counting", "mode") => "pack.schema.missing_counting_mode",
        (TopologySchemaKind::BoundaryTaxonomy, path, "id")
            if is_topology_entry_path(path, "boundaries") =>
        {
            "pack.schema.missing_boundary_id"
        }
        (TopologySchemaKind::BoundaryTaxonomy, path, "label")
            if is_topology_entry_path(path, "boundaries") =>
        {
            "pack.schema.missing_boundary_label"
        }
        (TopologySchemaKind::BoundaryTaxonomy, path, "include")
            if is_topology_entry_path(path, "boundaries") =>
        {
            "pack.schema.missing_boundary_include"
        }
        (TopologySchemaKind::ComponentMap, path, "id")
            if is_topology_entry_path(path, "components") =>
        {
            "pack.schema.missing_component_id"
        }
        (TopologySchemaKind::ComponentMap, path, "label")
            if is_topology_entry_path(path, "components") =>
        {
            "pack.schema.missing_component_label"
        }
        (TopologySchemaKind::ComponentMap, path, "include")
            if is_topology_entry_path(path, "components") =>
        {
            "pack.schema.missing_component_include"
        }
        _ => "pack.schema.missing_required_field",
    }
}

fn is_topology_entry_path(path: &str, collection: &str) -> bool {
    path.strip_prefix(&format!("/{collection}/"))
        .map(|suffix| suffix.parse::<usize>().is_ok())
        .unwrap_or(false)
}

fn min_length_code(path: &str) -> &'static str {
    if path == "/name" || path.ends_with("/id") || path.ends_with("/label") {
        "pack.schema.invalid_name"
    } else if path.contains("/include/") || path.contains("/exclude/") {
        "pack.schema.invalid_glob"
    } else {
        "pack.schema.invalid_value"
    }
}

fn validate_profile_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    let mut diagnostics = Vec::new();
    let Some(root) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.profile.invalid_root",
            "profile document must be an object",
            Some(PackLocation {
                origin: origin.clone(),
                path: None,
            }),
        ));
        return diagnostics;
    };

    require_string(
        root.get("kind"),
        origin,
        "/kind",
        "pack.schema.missing_kind",
        &mut diagnostics,
    );
    require_u64(
        root.get("version"),
        origin,
        "/version",
        "pack.schema.missing_version",
        &mut diagnostics,
    );
    require_string(
        root.get("id"),
        origin,
        "/id",
        "pack.schema.missing_id",
        &mut diagnostics,
    );
    require_string(
        root.get("name"),
        origin,
        "/name",
        "pack.schema.missing_name",
        &mut diagnostics,
    );

    if let Some(kind) = root.get("kind").and_then(Value::as_str) {
        if kind != "profile" {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_kind",
                format!("expected profile kind, found {kind}"),
                Some(location(origin, "/kind")),
            ));
        }
    }
    if let Some(version) = root.get("version").and_then(Value::as_u64) {
        if version != 1 {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_version",
                format!("expected profile version 1, found {version}"),
                Some(location(origin, "/version")),
            ));
        }
    }
    if let Some(id) = root.get("id").and_then(Value::as_str) {
        if PackName::parse(id).is_err() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_pack_name",
                format!("invalid profile id: {id}"),
                Some(location(origin, "/id")),
            ));
        }
    }
    if let Some(name) = root.get("name").and_then(Value::as_str) {
        if name.is_empty() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_name",
                "name must be a non-empty string",
                Some(location(origin, "/name")),
            ));
        }
    }

    validate_optional_string(
        root.get("description"),
        origin,
        "/description",
        &mut diagnostics,
    );
    validate_apps(root.get("apps"), origin, &mut diagnostics);
    validate_analysis(root.get("analysis"), origin, &mut diagnostics);
    validate_optional_ref_object(
        root.get("topology"),
        origin,
        "boundary_taxonomy",
        "component_map",
        "/topology",
        &mut diagnostics,
    );
    validate_single_optional_ref(
        root.get("score"),
        origin,
        "model",
        "/score",
        &mut diagnostics,
    );
    validate_include_section(root.get("rules"), origin, "/rules", &mut diagnostics);
    validate_include_section(root.get("queries"), origin, "/queries", &mut diagnostics);
    validate_include_section(root.get("recipes"), origin, "/recipes", &mut diagnostics);

    diagnostics
}

fn validate_boundary_taxonomy_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_topology_document(origin, value, TopologySchemaKind::BoundaryTaxonomy)
}

fn validate_component_map_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_topology_document(origin, value, TopologySchemaKind::ComponentMap)
}

fn require_string(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    code: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    match value {
        Some(Value::String(_)) => {}
        Some(_) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "expected string value",
            Some(location(origin, path)),
        )),
        None => diagnostics.push(PackDiagnostic::error(
            code,
            "required string field is missing",
            Some(location(origin, path)),
        )),
    }
}

fn require_u64(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    code: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    match value {
        Some(Value::Number(number)) if number.as_u64().is_some() => {}
        Some(_) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "expected integer value",
            Some(location(origin, path)),
        )),
        None => diagnostics.push(PackDiagnostic::error(
            code,
            "required integer field is missing",
            Some(location(origin, path)),
        )),
    }
}

fn require_object(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    code: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    match value {
        Some(Value::Object(_)) => {}
        Some(_) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "expected object value",
            Some(location(origin, path)),
        )),
        None => diagnostics.push(PackDiagnostic::error(
            code,
            "required object field is missing",
            Some(location(origin, path)),
        )),
    }
}

fn require_array(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    code: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    match value {
        Some(Value::Array(_)) => {}
        Some(_) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "expected array value",
            Some(location(origin, path)),
        )),
        None => diagnostics.push(PackDiagnostic::error(
            code,
            "required array field is missing",
            Some(location(origin, path)),
        )),
    }
}

fn validate_optional_string(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    if let Some(value) = value {
        if !value.is_string() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "expected string value",
                Some(location(origin, path)),
            ));
        }
    }
}

fn validate_non_empty_string(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    if let Some(value) = value {
        match value.as_str() {
            Some(text) if !text.is_empty() => {}
            Some(_) => diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_name",
                "value must be a non-empty string",
                Some(location(origin, path)),
            )),
            None => diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "expected string value",
                Some(location(origin, path)),
            )),
        }
    }
}

fn validate_apps(
    value: Option<&Value>,
    origin: &PackOrigin,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "apps must be an object",
            Some(location(origin, "/apps")),
        ));
        return;
    };
    if let Some(enabled) = object.get("enabled") {
        validate_string_array(
            enabled,
            origin,
            "/apps/enabled",
            diagnostics,
            Some(validate_app_name),
        );
    }
    if let Some(default) = object.get("default") {
        if let Some(default) = default.as_str() {
            validate_app_name(origin, "/apps/default", default, diagnostics);
        } else {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "apps.default must be a string",
                Some(location(origin, "/apps/default")),
            ));
        }
    }
}

fn validate_analysis(
    value: Option<&Value>,
    origin: &PackOrigin,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "analysis must be an object",
            Some(location(origin, "/analysis")),
        ));
        return;
    };
    if let Some(languages) = object.get("languages") {
        validate_string_array(
            languages,
            origin,
            "/analysis/languages",
            diagnostics,
            Some(validate_language_id),
        );
    }
    if let Some(follow_symlinks) = object.get("follow_symlinks") {
        if !follow_symlinks.is_boolean() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "analysis.follow_symlinks must be a boolean",
                Some(location(origin, "/analysis/follow_symlinks")),
            ));
        }
    }
    if let Some(max_scope_depth) = object.get("max_scope_depth") {
        if max_scope_depth.as_u64().is_none() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "analysis.max_scope_depth must be an integer",
                Some(location(origin, "/analysis/max_scope_depth")),
            ));
        }
    }
}

fn validate_optional_ref_object(
    value: Option<&Value>,
    origin: &PackOrigin,
    first: &str,
    second: &str,
    base_path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "section must be an object",
            Some(location(origin, base_path)),
        ));
        return;
    };
    for field in [first, second] {
        if let Some(item) = object.get(field) {
            if !item.is_string() {
                diagnostics.push(PackDiagnostic::error(
                    "pack.schema.invalid_type",
                    format!("{field} must be a string reference"),
                    Some(location(origin, &format!("{base_path}/{field}"))),
                ));
            }
        }
    }
}

fn validate_single_optional_ref(
    value: Option<&Value>,
    origin: &PackOrigin,
    field: &str,
    base_path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "section must be an object",
            Some(location(origin, base_path)),
        ));
        return;
    };
    if let Some(item) = object.get(field) {
        if !item.is_string() {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                format!("{field} must be a string reference"),
                Some(location(origin, &format!("{base_path}/{field}"))),
            ));
        }
    }
}

fn validate_include_section(
    value: Option<&Value>,
    origin: &PackOrigin,
    base_path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "section must be an object",
            Some(location(origin, base_path)),
        ));
        return;
    };
    if let Some(packs) = object.get("packs") {
        validate_string_array(
            packs,
            origin,
            &format!("{base_path}/packs"),
            diagnostics,
            Some(validate_pack_ref_string),
        );
    }
}

fn validate_string_array(
    value: &Value,
    origin: &PackOrigin,
    path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
    validate_item: Option<ArrayItemValidator>,
) {
    let Some(items) = value.as_array() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "expected string array",
            Some(location(origin, path)),
        ));
        return;
    };
    for (index, item) in items.iter().enumerate() {
        if let Some(item) = item.as_str() {
            if let Some(validate_item) = validate_item {
                validate_item(origin, &format!("{path}/{index}"), item, diagnostics);
            }
        } else {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "array entries must be strings",
                Some(location(origin, &format!("{path}/{index}"))),
            ));
        }
    }
}

fn location(origin: &PackOrigin, path: &str) -> PackLocation {
    PackLocation {
        origin: origin.clone(),
        path: Some(JsonPointer::parse(path).expect("pointer should be valid")),
    }
}

fn validate_app_name(
    origin: &PackOrigin,
    path: &str,
    value: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    if AppName::parse(value).is_err() {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_app_name",
            format!("invalid app name: {value}"),
            Some(location(origin, path)),
        ));
    }
}

fn validate_language_id(
    origin: &PackOrigin,
    path: &str,
    value: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    if LanguageId::parse(value).is_err() {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_language_id",
            format!("invalid language id: {value}"),
            Some(location(origin, path)),
        ));
    }
}

fn validate_pack_ref_string(
    origin: &PackOrigin,
    path: &str,
    value: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    if PackRef::parse(value).is_err() {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_pack_ref",
            format!("invalid pack reference: {value}"),
            Some(location(origin, path)),
        ));
    }
}

fn validate_counting_mode(
    value: Option<&Value>,
    origin: &PackOrigin,
    path: &str,
    expected_mode: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(object) = value.as_object() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "counting must be an object",
            Some(location(origin, path)),
        ));
        return;
    };

    match object.get("mode") {
        Some(Value::String(mode)) if mode == expected_mode => {}
        Some(Value::String(mode)) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_counting_mode",
            format!("expected counting.mode {expected_mode}, found {mode}"),
            Some(location(origin, &format!("{path}/mode"))),
        )),
        Some(_) => diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "counting.mode must be a string",
            Some(location(origin, &format!("{path}/mode"))),
        )),
        None => diagnostics.push(PackDiagnostic::error(
            "pack.schema.missing_counting_mode",
            "counting.mode is required",
            Some(location(origin, &format!("{path}/mode"))),
        )),
    }
}

fn validate_boundary_entries(
    value: Option<&Value>,
    origin: &PackOrigin,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(items) = value.as_array() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "boundaries must be an array",
            Some(location(origin, "/boundaries")),
        ));
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let base = format!("/boundaries/{index}");
        let Some(object) = item.as_object() else {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "boundary entries must be objects",
                Some(location(origin, &base)),
            ));
            continue;
        };

        require_string(
            object.get("id"),
            origin,
            &format!("{base}/id"),
            "pack.schema.missing_boundary_id",
            diagnostics,
        );
        require_string(
            object.get("label"),
            origin,
            &format!("{base}/label"),
            "pack.schema.missing_boundary_label",
            diagnostics,
        );
        require_array(
            object.get("include"),
            origin,
            &format!("{base}/include"),
            "pack.schema.missing_boundary_include",
            diagnostics,
        );

        validate_non_empty_string(object.get("id"), origin, &format!("{base}/id"), diagnostics);
        validate_non_empty_string(
            object.get("label"),
            origin,
            &format!("{base}/label"),
            diagnostics,
        );
        if let Some(include) = object.get("include") {
            validate_string_array(
                include,
                origin,
                &format!("{base}/include"),
                diagnostics,
                None,
            );
            validate_non_empty_string_array(
                include,
                origin,
                &format!("{base}/include"),
                diagnostics,
            );
        }
        if let Some(exclude) = object.get("exclude") {
            validate_string_array(
                exclude,
                origin,
                &format!("{base}/exclude"),
                diagnostics,
                None,
            );
            validate_non_empty_string_array(
                exclude,
                origin,
                &format!("{base}/exclude"),
                diagnostics,
            );
        }
    }
}

fn validate_component_entries(
    value: Option<&Value>,
    origin: &PackOrigin,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    let Some(items) = value.as_array() else {
        diagnostics.push(PackDiagnostic::error(
            "pack.schema.invalid_type",
            "components must be an array",
            Some(location(origin, "/components")),
        ));
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let base = format!("/components/{index}");
        let Some(object) = item.as_object() else {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_type",
                "component entries must be objects",
                Some(location(origin, &base)),
            ));
            continue;
        };

        require_string(
            object.get("id"),
            origin,
            &format!("{base}/id"),
            "pack.schema.missing_component_id",
            diagnostics,
        );
        require_string(
            object.get("label"),
            origin,
            &format!("{base}/label"),
            "pack.schema.missing_component_label",
            diagnostics,
        );
        require_array(
            object.get("include"),
            origin,
            &format!("{base}/include"),
            "pack.schema.missing_component_include",
            diagnostics,
        );

        validate_non_empty_string(object.get("id"), origin, &format!("{base}/id"), diagnostics);
        validate_non_empty_string(
            object.get("label"),
            origin,
            &format!("{base}/label"),
            diagnostics,
        );
        if let Some(include) = object.get("include") {
            validate_string_array(
                include,
                origin,
                &format!("{base}/include"),
                diagnostics,
                None,
            );
            validate_non_empty_string_array(
                include,
                origin,
                &format!("{base}/include"),
                diagnostics,
            );
        }
        if let Some(exclude) = object.get("exclude") {
            validate_string_array(
                exclude,
                origin,
                &format!("{base}/exclude"),
                diagnostics,
                None,
            );
            validate_non_empty_string_array(
                exclude,
                origin,
                &format!("{base}/exclude"),
                diagnostics,
            );
        }
        if let Some(tags) = object.get("tags") {
            validate_string_array(tags, origin, &format!("{base}/tags"), diagnostics, None);
        }
    }
}

fn validate_non_empty_string_array(
    value: &Value,
    origin: &PackOrigin,
    path: &str,
    diagnostics: &mut Vec<PackDiagnostic>,
) {
    let Some(items) = value.as_array() else {
        return;
    };

    for (index, item) in items.iter().enumerate() {
        if matches!(item.as_str(), Some(text) if text.is_empty()) {
            diagnostics.push(PackDiagnostic::error(
                "pack.schema.invalid_glob",
                "glob entries must be non-empty strings",
                Some(location(origin, &format!("{path}/{index}"))),
            ));
        }
    }
}

fn normalize_profile(raw: &RawProfile) -> NormalizedProfileDocument {
    let apps = normalize_apps(raw.apps.as_ref());
    let analysis = normalize_analysis(raw.analysis.as_ref());
    let topology = normalize_topology(raw.topology.as_ref());
    let score = normalize_score(raw.score.as_ref());

    NormalizedProfileDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        apps,
        analysis,
        topology,
        score,
        rules: normalize_include_section(raw.rules.as_ref()),
        queries: normalize_include_section(raw.queries.as_ref()),
        recipes: normalize_include_section(raw.recipes.as_ref()),
    }
}

fn normalize_apps(raw: Option<&RawProfileApps>) -> NormalizedProfileApps {
    let mut enabled = sorted_unique_strings(
        raw.and_then(|apps| apps.enabled.clone())
            .unwrap_or_else(|| vec![DEFAULT_APP.to_owned()]),
    );
    let default = raw
        .and_then(|apps| apps.default.clone())
        .unwrap_or_else(|| DEFAULT_APP.to_owned());
    if !enabled.iter().any(|app| app == &default) {
        enabled.push(default.clone());
        enabled.sort();
    }
    NormalizedProfileApps { enabled, default }
}

fn normalize_analysis(raw: Option<&RawProfileAnalysis>) -> NormalizedProfileAnalysis {
    NormalizedProfileAnalysis {
        languages: sorted_unique_strings(
            raw.and_then(|analysis| analysis.languages.clone())
                .unwrap_or_else(|| {
                    DEFAULT_LANGUAGES
                        .iter()
                        .map(|language| (*language).to_owned())
                        .collect()
                }),
        ),
        follow_symlinks: raw
            .and_then(|analysis| analysis.follow_symlinks)
            .unwrap_or(DEFAULT_FOLLOW_SYMLINKS),
        max_scope_depth: raw
            .and_then(|analysis| analysis.max_scope_depth)
            .unwrap_or(DEFAULT_MAX_SCOPE_DEPTH),
    }
}

fn normalize_topology(raw: Option<&RawProfileTopology>) -> NormalizedProfileTopology {
    NormalizedProfileTopology {
        boundary_taxonomy: raw.and_then(|topology| topology.boundary_taxonomy.clone()),
        component_map: raw.and_then(|topology| topology.component_map.clone()),
    }
}

fn normalize_score(raw: Option<&RawProfileScore>) -> NormalizedProfileScore {
    NormalizedProfileScore {
        model: raw.and_then(|score| score.model.clone()),
    }
}

fn normalize_include_section(raw: Option<&RawIncludeSection>) -> NormalizedIncludeSection {
    NormalizedIncludeSection {
        packs: sorted_unique_strings(raw.map(|section| section.packs.clone()).unwrap_or_default()),
    }
}

fn normalize_boundary_taxonomy(raw: &RawBoundaryTaxonomy) -> NormalizedBoundaryTaxonomyDocument {
    let mut boundaries = raw
        .boundaries
        .iter()
        .map(normalize_boundary_entry)
        .collect::<Vec<_>>();
    boundaries.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.include.cmp(&right.include))
            .then_with(|| left.exclude.cmp(&right.exclude))
    });

    NormalizedBoundaryTaxonomyDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        counting: NormalizedBoundaryTaxonomyCounting {
            mode: raw.counting.mode,
        },
        boundaries,
    }
}

fn normalize_boundary_entry(raw: &RawBoundaryEntry) -> NormalizedBoundaryEntry {
    NormalizedBoundaryEntry {
        id: raw.id.clone(),
        label: raw.label.clone(),
        include: sorted_unique_strings(raw.include.clone()),
        exclude: sorted_unique_strings(raw.exclude.clone()),
    }
}

fn normalize_component_map(raw: &RawComponentMap) -> NormalizedComponentMapDocument {
    let mut components = raw
        .components
        .iter()
        .map(normalize_component_entry)
        .collect::<Vec<_>>();
    components.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.include.cmp(&right.include))
            .then_with(|| left.exclude.cmp(&right.exclude))
            .then_with(|| left.tags.cmp(&right.tags))
    });

    NormalizedComponentMapDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        counting: NormalizedComponentMapCounting {
            mode: raw.counting.mode,
        },
        components,
    }
}

fn normalize_component_entry(raw: &RawComponentEntry) -> NormalizedComponentEntry {
    NormalizedComponentEntry {
        id: raw.id.clone(),
        label: raw.label.clone(),
        include: sorted_unique_strings(raw.include.clone()),
        exclude: sorted_unique_strings(raw.exclude.clone()),
        tags: sorted_unique_strings(raw.tags.clone()),
    }
}

fn sorted_unique_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn compile_normalized_profile(
    origin: PackOrigin,
    source_file_base_dir: Option<PathBuf>,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedProfileDocument,
) -> PackResult<CompiledProfile> {
    let id = PackName::parse(&normalized.id)?;
    let enabled_apps = normalized
        .apps
        .enabled
        .iter()
        .map(|app| AppName::parse(app))
        .collect::<PackResult<BTreeSet<_>>>()?;
    let default_app = AppName::parse(&normalized.apps.default)?;
    let languages = normalized
        .analysis
        .languages
        .iter()
        .map(|language| LanguageId::parse(language))
        .collect::<PackResult<BTreeSet<_>>>()?;

    let topology = CompiledProfileTopology {
        boundary_taxonomy: compile_optional_ref(
            &origin,
            normalized.topology.boundary_taxonomy.as_deref(),
            "/topology/boundary_taxonomy",
        )?,
        component_map: compile_optional_ref(
            &origin,
            normalized.topology.component_map.as_deref(),
            "/topology/component_map",
        )?,
        classes: CompiledPathClasses,
    };
    let score = CompiledProfileScore {
        model: compile_optional_ref(&origin, normalized.score.model.as_deref(), "/score/model")?,
    };
    let includes = CompiledProfileIncludes {
        rule_packs: compile_ref_set(&origin, &normalized.rules.packs, "/rules/packs")?,
        query_packs: compile_ref_set(&origin, &normalized.queries.packs, "/queries/packs")?,
        recipe_packs: compile_ref_set(&origin, &normalized.recipes.packs, "/recipes/packs")?,
    };

    Ok(CompiledProfile {
        header: CompiledPackHeader {
            kind: PackKind::Profile,
            id,
            version: normalized.version,
            name: normalized.name,
            description: normalized.description,
            schema_id: PACK_PROFILE_V1_SCHEMA_ID,
            origin,
            source_fingerprint,
            semantic_fingerprint,
        },
        source_file_base_dir,
        apps: CompiledProfileApps {
            enabled: enabled_apps,
            default: default_app,
        },
        analysis: CompiledAnalysisDefaults {
            languages,
            follow_symlinks: normalized.analysis.follow_symlinks,
            max_scope_depth: normalized.analysis.max_scope_depth,
        },
        topology,
        score,
        includes,
        diagnostics: Vec::new(),
    })
}

fn absolutize_path(path: &Path) -> PackResult<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let cwd = std::env::current_dir().map_err(|error| PackError::Io {
        origin: path.display().to_string(),
        reason: error.to_string(),
    })?;
    Ok(cwd.join(path))
}

fn compile_normalized_boundary_taxonomy(
    origin: PackOrigin,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedBoundaryTaxonomyDocument,
) -> PackResult<CompiledBoundaryTaxonomy> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let mut boundaries = BTreeMap::new();
    let mut local_ids = BTreeSet::new();

    for boundary in normalized.boundaries {
        if !local_ids.insert(boundary.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::BoundaryTaxonomy,
                pack_id: pack_id.clone(),
                entry_kind: "boundary",
                entry_id: boundary.id,
            });
        }

        let boundary_id = compile_boundary_id(&pack_id, &boundary.id);
        let compiled = CompiledBoundary {
            local_id: boundary.id,
            id: boundary_id.clone(),
            label: boundary.label,
            include_patterns: boundary.include.clone(),
            exclude_patterns: boundary.exclude.clone(),
            include_matcher: compile_glob_set(&boundary.include)?,
            exclude_matcher: compile_glob_set(&boundary.exclude)?,
        };
        boundaries.insert(boundary_id, compiled);
    }

    Ok(CompiledBoundaryTaxonomy {
        header: CompiledPackHeader {
            kind: PackKind::BoundaryTaxonomy,
            id,
            version: normalized.version,
            name: normalized.name,
            description: normalized.description,
            schema_id: PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID,
            origin,
            source_fingerprint,
            semantic_fingerprint,
        },
        counting_mode: match normalized.counting.mode {
            RawBoundaryTaxonomyCountingMode::DistinctMinusOne => {
                BoundaryCountingMode::DistinctMinusOne
            }
        },
        boundaries,
        diagnostics: Vec::new(),
    })
}

fn compile_normalized_component_map(
    origin: PackOrigin,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedComponentMapDocument,
) -> PackResult<CompiledComponentMap> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let mut components = BTreeMap::new();
    let mut local_ids = BTreeSet::new();

    for component in normalized.components {
        if !local_ids.insert(component.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::ComponentMap,
                pack_id: pack_id.clone(),
                entry_kind: "component",
                entry_id: component.id,
            });
        }

        let component_id = compile_component_id(&pack_id, &component.id);
        let compiled = CompiledComponent {
            local_id: component.id,
            id: component_id.clone(),
            label: component.label,
            include_patterns: component.include.clone(),
            exclude_patterns: component.exclude.clone(),
            tags: component.tags.into_iter().collect(),
            include_matcher: compile_glob_set(&component.include)?,
            exclude_matcher: compile_glob_set(&component.exclude)?,
        };
        components.insert(component_id, compiled);
    }

    Ok(CompiledComponentMap {
        header: CompiledPackHeader {
            kind: PackKind::ComponentMap,
            id,
            version: normalized.version,
            name: normalized.name,
            description: normalized.description,
            schema_id: PACK_COMPONENT_MAP_V1_SCHEMA_ID,
            origin,
            source_fingerprint,
            semantic_fingerprint,
        },
        counting_mode: match normalized.counting.mode {
            RawComponentMapCountingMode::Distinct => ComponentCountingMode::Distinct,
        },
        components,
        diagnostics: Vec::new(),
    })
}

fn compile_glob_set(patterns: &[String]) -> PackResult<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|error| PackError::GlobCompile {
            pattern: pattern.clone(),
            reason: error.to_string(),
        })?;
        builder.add(glob);
    }

    builder.build().map_err(|error| PackError::GlobCompile {
        pattern: patterns.join(","),
        reason: error.to_string(),
    })
}

fn compile_boundary_id(pack_id: &str, local_id: &str) -> BoundaryId {
    BoundaryId::from_identity(&format!(
        "pack\0boundary_taxonomy\0{pack_id}\0boundary\0{local_id}"
    ))
}

fn compile_component_id(pack_id: &str, local_id: &str) -> ComponentId {
    ComponentId::from_identity(&format!(
        "pack\0component_map\0{pack_id}\0component\0{local_id}"
    ))
}

fn resolve_boundary_taxonomy(
    compiler: &PackCompiler,
    profile: &CompiledProfile,
    reference: &PackRef,
) -> PackResult<CompiledBoundaryTaxonomy> {
    match reference {
        PackRef::Builtin(name) => {
            if let Some(source) = builtin::boundary_taxonomy_source(name.as_str()) {
                compiler.compile_boundary_taxonomy(source)
            } else if builtin::component_map_source(name.as_str()).is_some() {
                Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::BoundaryTaxonomy,
                    actual: PackKind::ComponentMap,
                })
            } else {
                Err(unknown_pack_reference(profile, reference))
            }
        }
        PackRef::File(path) => {
            let source = resolve_file_source(profile, path.as_str())?;
            match compiler.compile_boundary_taxonomy(source) {
                Ok(compiled) => Ok(compiled),
                Err(error) if is_invalid_kind_schema_violation(&error) => {
                    let alternate_source = resolve_file_source(profile, path.as_str())?;
                    match compiler.compile_component_map(alternate_source) {
                        Ok(_) => Err(PackError::RefKindMismatch {
                            reference: reference.as_str(),
                            expected: PackKind::BoundaryTaxonomy,
                            actual: PackKind::ComponentMap,
                        }),
                        Err(_) => Err(error),
                    }
                }
                Err(error) => Err(error),
            }
        }
    }
}

fn resolve_component_map(
    compiler: &PackCompiler,
    profile: &CompiledProfile,
    reference: &PackRef,
) -> PackResult<CompiledComponentMap> {
    match reference {
        PackRef::Builtin(name) => {
            if let Some(source) = builtin::component_map_source(name.as_str()) {
                compiler.compile_component_map(source)
            } else if builtin::boundary_taxonomy_source(name.as_str()).is_some() {
                Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::ComponentMap,
                    actual: PackKind::BoundaryTaxonomy,
                })
            } else {
                Err(unknown_pack_reference(profile, reference))
            }
        }
        PackRef::File(path) => {
            let source = resolve_file_source(profile, path.as_str())?;
            match compiler.compile_component_map(source) {
                Ok(compiled) => Ok(compiled),
                Err(error) if is_invalid_kind_schema_violation(&error) => {
                    let alternate_source = resolve_file_source(profile, path.as_str())?;
                    match compiler.compile_boundary_taxonomy(alternate_source) {
                        Ok(_) => Err(PackError::RefKindMismatch {
                            reference: reference.as_str(),
                            expected: PackKind::ComponentMap,
                            actual: PackKind::BoundaryTaxonomy,
                        }),
                        Err(_) => Err(error),
                    }
                }
                Err(error) => Err(error),
            }
        }
    }
}

/// Phase B file refs resolve lexically from the compiled profile's captured
/// parent directory, never from a display string or the ambient cwd.
fn resolve_file_source(profile: &CompiledProfile, relative_path: &str) -> PackResult<PackSource> {
    let Some(base) = &profile.source_file_base_dir else {
        return Err(unknown_file_reference(profile, relative_path));
    };

    let path = base.join(relative_path);
    match path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(unknown_file_reference(profile, relative_path)),
        Err(error) => {
            return Err(PackError::Io {
                origin: path.display().to_string(),
                reason: error.to_string(),
            });
        }
    }

    Ok(PackSource::File {
        path,
        format_hint: Some(PackFormat::Json),
    })
}

fn unknown_pack_reference(profile: &CompiledProfile, reference: &PackRef) -> PackError {
    PackError::UnknownPackReference {
        referring_pack: profile.header.id.as_str().to_owned(),
        reference: reference.as_str(),
    }
}

fn unknown_file_reference(profile: &CompiledProfile, relative_path: &str) -> PackError {
    PackError::UnknownPackReference {
        referring_pack: profile.header.id.as_str().to_owned(),
        reference: format!("file:{relative_path}"),
    }
}

fn is_invalid_kind_schema_violation(error: &PackError) -> bool {
    matches!(
        error,
        PackError::SchemaViolation { diagnostics, .. }
            if diagnostics.iter().any(|diagnostic| diagnostic.code.as_str() == "pack.schema.invalid_kind")
    )
}

fn compile_optional_ref(
    origin: &PackOrigin,
    reference: Option<&str>,
    pointer: &str,
) -> PackResult<Option<PackRef>> {
    reference
        .map(|reference| compile_ref(origin, reference, pointer))
        .transpose()
}

fn compile_ref_set(
    origin: &PackOrigin,
    references: &[String],
    pointer: &str,
) -> PackResult<BTreeSet<PackRef>> {
    let mut out = BTreeSet::new();
    for (index, reference) in references.iter().enumerate() {
        out.insert(compile_ref(
            origin,
            reference,
            &format!("{pointer}/{index}"),
        )?);
    }
    Ok(out)
}

fn compile_ref(origin: &PackOrigin, reference: &str, pointer: &str) -> PackResult<PackRef> {
    let parsed = PackRef::parse(reference).map_err(|_| PackError::SchemaViolation {
        origin: origin.display(),
        schema_id: PACK_PROFILE_V1_SCHEMA_ID,
        diagnostics: vec![PackDiagnostic::error(
            "pack.refs.invalid_reference",
            format!("invalid pack reference: {reference}"),
            Some(location(origin, pointer)),
        )],
    })?;

    if !origin.is_file_backed() && matches!(parsed, PackRef::File(_)) {
        return Err(PackError::SchemaViolation {
            origin: origin.display(),
            schema_id: PACK_PROFILE_V1_SCHEMA_ID,
            diagnostics: vec![PackDiagnostic::error(
                "pack.refs.file_requires_file_origin",
                "builtin and inline profile sources may only use builtin references in Phase A",
                Some(location(origin, pointer)),
            )],
        });
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::pack::builtin;
    use crate::pack::compiler::PackCompiler;
    use crate::pack::error::PackError;
    use crate::pack::source::{PackFormat, PackSource};

    #[test]
    fn compiles_builtin_file_and_inline_profiles() {
        let compiler = PackCompiler::new();
        let builtin = compiler
            .compile_profile(builtin::profile_source("generic/default").expect("builtin source"))
            .expect("builtin profile");
        let inline = compiler
            .compile_profile(PackSource::Inline {
                logical_name: "inline".to_owned(),
                format: PackFormat::Toml,
                bytes: br#"
kind = "profile"
version = 1
id = "generic/default"
name = "Generic default profile"
description = "Default deterministic profile for generic repositories"

[topology]
boundary_taxonomy = "builtin:generic/boundaries"
component_map = "builtin:generic/components"

[analysis]
max_scope_depth = 2
follow_symlinks = false
languages = ["typescript", "javascript", "python", "rust", "yaml", "toml", "json"]

[apps]
default = "score"
enabled = ["score"]
"#
                .to_vec(),
            })
            .expect("inline profile");
        let path = unique_temp_file("phase-a-profile.toml");
        std::fs::write(&path, builtin_source_bytes()).expect("write temp profile");
        let file = compiler
            .compile_profile(PackSource::File {
                path: path.clone(),
                format_hint: None,
            })
            .expect("file profile");
        std::fs::remove_file(path).expect("remove temp profile");

        assert_eq!(
            builtin.header.semantic_fingerprint,
            inline.header.semantic_fingerprint
        );
        assert_eq!(
            builtin.header.semantic_fingerprint,
            file.header.semantic_fingerprint
        );
        assert_ne!(
            builtin.header.source_fingerprint,
            inline.header.source_fingerprint
        );
    }

    #[test]
    fn rejects_file_refs_from_builtin_and_inline_sources() {
        let compiler = PackCompiler::new();
        let error = compiler
            .compile_profile(PackSource::Inline {
                logical_name: "inline".to_owned(),
                format: PackFormat::Toml,
                bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = "Invalid profile"

[rules]
packs = ["file:rules/security.v1.json"]
"#
                .to_vec(),
            })
            .expect_err("inline file refs should fail");

        assert!(matches!(error, PackError::SchemaViolation { .. }));
    }

    #[test]
    fn rejects_unsupported_format_and_missing_files() {
        let compiler = PackCompiler::new();
        let unsupported = compiler
            .compile_profile(PackSource::Inline {
                logical_name: "inline".to_owned(),
                format: PackFormat::Json,
                bytes: br#"{}"#.to_vec(),
            })
            .expect_err("json profiles should fail");
        assert!(matches!(unsupported, PackError::UnsupportedFormat { .. }));

        let missing = compiler
            .compile_profile(PackSource::File {
                path: PathBuf::from("definitely-missing-profile.toml"),
                format_hint: None,
            })
            .expect_err("missing profile should fail");
        assert!(matches!(missing, PackError::Io { .. }));
    }

    #[test]
    fn semantic_fingerprint_ignores_toml_key_order() {
        let compiler = PackCompiler::new();
        let first = compiler
            .compile_profile(PackSource::Inline {
                logical_name: "first".to_owned(),
                format: PackFormat::Toml,
                bytes: br#"kind = "profile"
version = 1
id = "generic/default"
name = "Generic default profile"

[apps]
enabled = ["score"]
default = "score"

[analysis]
languages = ["json", "toml"]
follow_symlinks = false
max_scope_depth = 2
"#
                .to_vec(),
            })
            .expect("first profile");
        let second = compiler
            .compile_profile(PackSource::Inline {
                logical_name: "second".to_owned(),
                format: PackFormat::Toml,
                bytes: br#"name = "Generic default profile"
id = "generic/default"
version = 1
kind = "profile"

[analysis]
max_scope_depth = 2
follow_symlinks = false
languages = ["toml", "json"]

[apps]
default = "score"
enabled = ["score"]
"#
                .to_vec(),
            })
            .expect("second profile");

        assert_eq!(
            first.header.semantic_fingerprint,
            second.header.semantic_fingerprint
        );
    }

    fn builtin_source_bytes() -> &'static [u8] {
        match builtin::profile_source("generic/default").expect("builtin") {
            PackSource::Builtin { bytes, .. } => bytes,
            _ => unreachable!("builtin registry should return builtin source"),
        }
    }

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{nanos}-{name}"))
    }
}
