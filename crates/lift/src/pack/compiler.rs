//! Seam-1 pack compiler spine.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Number, Value};

use crate::kernel::{sha256_bytes, sha256_canonical_json, Fingerprint, JsonPointer};
use crate::pack::compiled::{
    CompiledAnalysisDefaults, CompiledPackHeader, CompiledPathClasses, CompiledProfile,
    CompiledProfileApps, CompiledProfileIncludes, CompiledProfileScore, CompiledProfileTopology,
};
use crate::pack::diagnostics::{PackDiagnostic, PackLocation};
use crate::pack::error::{PackError, PackResult};
use crate::pack::names::{AppName, LanguageId, PackName};
use crate::pack::raw::{
    PackKind, RawIncludeSection, RawProfile, RawProfileAnalysis, RawProfileApps, RawProfileScore,
    RawProfileTopology,
};
use crate::pack::refs::PackRef;
use crate::pack::schema::PACK_PROFILE_V1_SCHEMA_ID;
use crate::pack::source::{PackFormat, PackOrigin, PackSource};

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
        if loaded.format != PackFormat::Toml {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_profile_toml(&loaded.origin, &loaded.bytes)?;
        let mut diagnostics = validate_profile_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_PROFILE_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw_profile: RawProfile =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_PROFILE_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.profile.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        let normalized = normalize_profile(&raw_profile);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.profile.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;

        compile_normalized_profile(
            loaded.origin,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
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
            }),
            PackSource::File { path, format_hint } => {
                let bytes = fs::read(&path).map_err(|error| PackError::Io {
                    origin: path.display().to_string(),
                    reason: error.to_string(),
                })?;
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
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoadedSource {
    origin: PackOrigin,
    format: PackFormat,
    bytes: Vec<u8>,
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

fn sorted_unique_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn compile_normalized_profile(
    origin: PackOrigin,
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
