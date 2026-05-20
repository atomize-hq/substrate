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

use crate::kernel::{
    sha256_bytes, sha256_canonical_json, Fingerprint, JsonPointer, QueryId, RecipeId, RuleId,
};
use crate::pack::builtin;
use crate::pack::compiled::{
    BoundaryCountingMode, CompiledAnalysisDefaults, CompiledBoundary, CompiledBoundaryTaxonomy,
    CompiledComponent, CompiledComponentMap, CompiledConfidenceModel, CompiledConfidenceRule,
    CompiledMissingInputRule, CompiledPackHeader, CompiledPackSet, CompiledPathClasses,
    CompiledProfile, CompiledProfileApps, CompiledProfileIncludes, CompiledProfileScore,
    CompiledProfileTopology, CompiledQueryCapture, CompiledQueryDef, CompiledQueryPack,
    CompiledRecipeDef, CompiledRecipePack, CompiledRecipeTransform, CompiledRuleDef,
    CompiledRuleEmit, CompiledRulePack, CompiledRuleScope, CompiledScoreModel, CompiledTriggerRule,
    ComponentCountingMode, QueryEngineKind, ReservedPathClass, ResolvedProfileTopology,
};
use crate::pack::diagnostics::{PackDiagnostic, PackLocation};
use crate::pack::error::{PackError, PackResult};
use crate::pack::expr::{compile_expr, compile_query_ref, CompiledQueryRef};
use crate::pack::names::{AppName, LanguageId, PackName};
use crate::pack::raw::{
    PackKind, RawBoundaryEntry, RawBoundaryTaxonomy, RawBoundaryTaxonomyCountingMode,
    RawComponentEntry, RawComponentMap, RawComponentMapCountingMode, RawIncludeSection, RawProfile,
    RawProfileAnalysis, RawProfileApps, RawProfileScore, RawProfileTopology, RawQueryPack,
    RawRecipePack, RawRecipeTransform, RawReservedPathClass, RawRuleEmit, RawRulePack,
    RawScoreModel,
};
use crate::pack::refs::PackRef;
use crate::pack::schema::{
    PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_ID, PACK_BOUNDARY_TAXONOMY_V1_SCHEMA_JSON,
    PACK_COMMON_V1_SCHEMA_ID, PACK_COMMON_V1_SCHEMA_JSON, PACK_COMPONENT_MAP_V1_SCHEMA_ID,
    PACK_COMPONENT_MAP_V1_SCHEMA_JSON, PACK_PROFILE_V1_SCHEMA_ID, PACK_QUERY_PACK_V1_SCHEMA_ID,
    PACK_QUERY_PACK_V1_SCHEMA_JSON, PACK_RECIPE_PACK_V1_SCHEMA_ID, PACK_RECIPE_PACK_V1_SCHEMA_JSON,
    PACK_RULE_PACK_V1_SCHEMA_ID, PACK_RULE_PACK_V1_SCHEMA_JSON, PACK_SCORE_MODEL_V1_SCHEMA_ID,
    PACK_SCORE_MODEL_V1_SCHEMA_JSON,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdvancedSchemaKind {
    ScoreModel,
    QueryPack,
    RulePack,
    RecipePack,
}

impl AdvancedSchemaKind {
    fn invalid_root_code(self) -> &'static str {
        match self {
            Self::ScoreModel => "pack.score_model.invalid_root",
            Self::QueryPack => "pack.query_pack.invalid_root",
            Self::RulePack => "pack.rule_pack.invalid_root",
            Self::RecipePack => "pack.recipe_pack.invalid_root",
        }
    }

    fn validator(self) -> &'static Validator {
        match self {
            Self::ScoreModel => score_model_schema_validator(),
            Self::QueryPack => query_pack_schema_validator(),
            Self::RulePack => rule_pack_schema_validator(),
            Self::RecipePack => recipe_pack_schema_validator(),
        }
    }

    fn schema_id(self) -> &'static str {
        match self {
            Self::ScoreModel => PACK_SCORE_MODEL_V1_SCHEMA_ID,
            Self::QueryPack => PACK_QUERY_PACK_V1_SCHEMA_ID,
            Self::RulePack => PACK_RULE_PACK_V1_SCHEMA_ID,
            Self::RecipePack => PACK_RECIPE_PACK_V1_SCHEMA_ID,
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

    /// Compiles one standalone score model from builtin, file, or inline JSON sources.
    pub(crate) fn compile_score_model(&self, source: PackSource) -> PackResult<CompiledScoreModel> {
        self.compile_score_model_loaded(source)
            .map(|loaded| loaded.pack)
    }

    /// Compiles one standalone query pack from builtin, file, or inline JSON sources.
    pub(crate) fn compile_query_pack(&self, source: PackSource) -> PackResult<CompiledQueryPack> {
        self.compile_query_pack_loaded(source)
            .map(|loaded| loaded.pack)
    }

    /// Compiles one standalone rule pack from builtin, file, or inline JSON sources.
    pub(crate) fn compile_rule_pack(&self, source: PackSource) -> PackResult<CompiledRulePack> {
        self.compile_rule_pack_loaded(source)
            .map(|loaded| loaded.pack)
    }

    /// Compiles one standalone recipe pack from builtin, file, or inline JSON sources.
    pub(crate) fn compile_recipe_pack(&self, source: PackSource) -> PackResult<CompiledRecipePack> {
        self.compile_recipe_pack_loaded(source)
            .map(|loaded| loaded.pack)
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

    /// Resolves all selected advanced packs for a compiled profile into one bundle.
    pub(crate) fn resolve_profile_pack_set(
        &self,
        profile: &CompiledProfile,
    ) -> PackResult<CompiledPackSet> {
        let topology = self.resolve_profile_topology(profile)?;
        let mut memo = BTreeMap::new();
        let mut seen_pack_ids = BTreeMap::new();
        register_bundle_pack_id(
            &profile.header,
            &resolved_pack_origin_key(&profile.header.origin)?,
            &mut seen_pack_ids,
        )?;
        if let Some(boundary_taxonomy) = topology.boundary_taxonomy.as_ref() {
            register_bundle_pack_id(
                &boundary_taxonomy.header,
                &resolved_pack_origin_key(&boundary_taxonomy.header.origin)?,
                &mut seen_pack_ids,
            )?;
        }
        if let Some(component_map) = topology.component_map.as_ref() {
            register_bundle_pack_id(
                &component_map.header,
                &resolved_pack_origin_key(&component_map.header.origin)?,
                &mut seen_pack_ids,
            )?;
        }

        let score_model = profile
            .score
            .model
            .as_ref()
            .map(|reference| {
                self.resolve_bundle_score_model(
                    profile.header.id.as_str(),
                    profile.source_file_base_dir.as_ref(),
                    reference,
                    &mut memo,
                    &mut seen_pack_ids,
                )
            })
            .transpose()?
            .map(|loaded| loaded.pack);

        let mut query_packs = BTreeMap::new();
        let mut rule_packs = BTreeMap::new();
        let mut recipe_packs = BTreeMap::new();

        for reference in &profile.includes.query_packs {
            let loaded = self.resolve_bundle_query_pack(
                profile.header.id.as_str(),
                profile.source_file_base_dir.as_ref(),
                reference,
                &mut memo,
                &mut seen_pack_ids,
            )?;
            query_packs.insert(loaded.pack.header.id.clone(), loaded.pack);
        }

        for reference in &profile.includes.rule_packs {
            let loaded = self.resolve_bundle_rule_pack(
                profile.header.id.as_str(),
                profile.source_file_base_dir.as_ref(),
                reference,
                &mut memo,
                &mut seen_pack_ids,
            )?;
            self.resolve_rule_pack_queries(
                &loaded,
                &mut query_packs,
                &mut memo,
                &mut seen_pack_ids,
            )?;
            rule_packs.insert(loaded.pack.header.id.clone(), loaded.pack);
        }

        for reference in &profile.includes.recipe_packs {
            let loaded = self.resolve_bundle_recipe_pack(
                profile.header.id.as_str(),
                profile.source_file_base_dir.as_ref(),
                reference,
                &mut memo,
                &mut seen_pack_ids,
            )?;
            self.resolve_recipe_pack_queries(
                &loaded,
                &mut query_packs,
                &mut memo,
                &mut seen_pack_ids,
            )?;
            recipe_packs.insert(loaded.pack.header.id.clone(), loaded.pack);
        }

        let semantic_fingerprint = compile_pack_set_fingerprint(
            profile,
            &topology,
            score_model.as_ref(),
            &rule_packs,
            &query_packs,
            &recipe_packs,
        )?;

        Ok(CompiledPackSet {
            profile: profile.clone(),
            boundary_taxonomy: topology.boundary_taxonomy,
            component_map: topology.component_map,
            score_model,
            rule_packs,
            query_packs,
            recipe_packs,
            diagnostics: Vec::new(),
            semantic_fingerprint,
        })
    }

    fn compile_score_model_loaded(
        &self,
        source: PackSource,
    ) -> PackResult<LoadedCompiledScoreModel> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "score_model")?;
        let mut diagnostics = validate_score_model_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_SCORE_MODEL_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawScoreModel =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_SCORE_MODEL_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.score_model.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        let normalized = normalize_score_model(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.score_model.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        compile_normalized_score_model(
            loaded.origin,
            loaded.file_base_dir,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    fn compile_query_pack_loaded(&self, source: PackSource) -> PackResult<LoadedCompiledQueryPack> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "query_pack")?;
        let mut diagnostics = validate_query_pack_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_QUERY_PACK_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawQueryPack =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_QUERY_PACK_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.query_pack.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        let normalized = normalize_query_pack(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.query_pack.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        compile_normalized_query_pack(
            loaded.origin,
            loaded.file_base_dir,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    fn compile_rule_pack_loaded(&self, source: PackSource) -> PackResult<LoadedCompiledRulePack> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "rule_pack")?;
        let mut diagnostics = validate_rule_pack_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_RULE_PACK_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawRulePack =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_RULE_PACK_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.rule_pack.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        let normalized = normalize_rule_pack(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.rule_pack.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        compile_normalized_rule_pack(
            loaded.origin,
            loaded.file_base_dir,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    fn compile_recipe_pack_loaded(
        &self,
        source: PackSource,
    ) -> PackResult<LoadedCompiledRecipePack> {
        let loaded = self.load_source(source)?;
        if loaded.format != PackFormat::Json {
            return Err(PackError::UnsupportedFormat {
                origin: loaded.origin.display(),
            });
        }

        let source_fingerprint = sha256_bytes(&loaded.bytes);
        let json_value = parse_json_document(&loaded.origin, &loaded.bytes, "recipe_pack")?;
        let mut diagnostics = validate_recipe_pack_document(&loaded.origin, &json_value);
        if !diagnostics.is_empty() {
            diagnostics.sort();
            return Err(PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_RECIPE_PACK_V1_SCHEMA_ID,
                diagnostics,
            });
        }

        let raw: RawRecipePack =
            serde_json::from_value(json_value).map_err(|error| PackError::SchemaViolation {
                origin: loaded.origin.display(),
                schema_id: PACK_RECIPE_PACK_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.recipe_pack.deserialize_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        let normalized = normalize_recipe_pack(&raw);
        let semantic_fingerprint =
            sha256_canonical_json(&normalized).map_err(|error| PackError::ParseFailure {
                origin: loaded.origin.display(),
                diagnostics: vec![PackDiagnostic::error(
                    "pack.recipe_pack.canonicalization_failed",
                    error.to_string(),
                    Some(PackLocation {
                        origin: loaded.origin.clone(),
                        path: None,
                    }),
                )],
            })?;
        compile_normalized_recipe_pack(
            loaded.origin,
            loaded.file_base_dir,
            source_fingerprint,
            semantic_fingerprint,
            normalized,
        )
    }

    fn resolve_bundle_score_model(
        &self,
        referring_pack: &str,
        referring_base_dir: Option<&PathBuf>,
        reference: &PackRef,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<LoadedCompiledScoreModel> {
        let (source, key) = resolve_bundle_ref_source(
            self,
            referring_pack,
            referring_base_dir,
            reference,
            PackKind::ScoreModel,
        )?;
        if let Some(existing) = memo.get(&key) {
            return match existing {
                ResolvedBundlePack::ScoreModel(loaded) => Ok(loaded.clone()),
                other => Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::ScoreModel,
                    actual: resolved_bundle_pack_kind(other),
                }),
            };
        }

        let loaded = self.compile_score_model_loaded(source)?;
        register_bundle_pack_id(&loaded.pack.header, &key, seen_pack_ids)?;
        memo.insert(key, ResolvedBundlePack::ScoreModel(loaded.clone()));
        Ok(loaded)
    }

    fn resolve_bundle_query_pack(
        &self,
        referring_pack: &str,
        referring_base_dir: Option<&PathBuf>,
        reference: &PackRef,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<LoadedCompiledQueryPack> {
        let (source, key) = resolve_bundle_ref_source(
            self,
            referring_pack,
            referring_base_dir,
            reference,
            PackKind::QueryPack,
        )?;
        if let Some(existing) = memo.get(&key) {
            return match existing {
                ResolvedBundlePack::QueryPack(loaded) => Ok(loaded.clone()),
                other => Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::QueryPack,
                    actual: resolved_bundle_pack_kind(other),
                }),
            };
        }

        let loaded = self.compile_query_pack_loaded(source)?;
        register_bundle_pack_id(&loaded.pack.header, &key, seen_pack_ids)?;
        memo.insert(key, ResolvedBundlePack::QueryPack(loaded.clone()));
        Ok(loaded)
    }

    fn resolve_bundle_rule_pack(
        &self,
        referring_pack: &str,
        referring_base_dir: Option<&PathBuf>,
        reference: &PackRef,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<LoadedCompiledRulePack> {
        let (source, key) = resolve_bundle_ref_source(
            self,
            referring_pack,
            referring_base_dir,
            reference,
            PackKind::RulePack,
        )?;
        if let Some(existing) = memo.get(&key) {
            return match existing {
                ResolvedBundlePack::RulePack(loaded) => Ok(loaded.clone()),
                other => Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::RulePack,
                    actual: resolved_bundle_pack_kind(other),
                }),
            };
        }

        let loaded = self.compile_rule_pack_loaded(source)?;
        register_bundle_pack_id(&loaded.pack.header, &key, seen_pack_ids)?;
        memo.insert(key, ResolvedBundlePack::RulePack(loaded.clone()));
        Ok(loaded)
    }

    fn resolve_bundle_recipe_pack(
        &self,
        referring_pack: &str,
        referring_base_dir: Option<&PathBuf>,
        reference: &PackRef,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<LoadedCompiledRecipePack> {
        let (source, key) = resolve_bundle_ref_source(
            self,
            referring_pack,
            referring_base_dir,
            reference,
            PackKind::RecipePack,
        )?;
        if let Some(existing) = memo.get(&key) {
            return match existing {
                ResolvedBundlePack::RecipePack(loaded) => Ok(loaded.clone()),
                other => Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected: PackKind::RecipePack,
                    actual: resolved_bundle_pack_kind(other),
                }),
            };
        }

        let loaded = self.compile_recipe_pack_loaded(source)?;
        register_bundle_pack_id(&loaded.pack.header, &key, seen_pack_ids)?;
        memo.insert(key, ResolvedBundlePack::RecipePack(loaded.clone()));
        Ok(loaded)
    }

    fn resolve_rule_pack_queries(
        &self,
        loaded: &LoadedCompiledRulePack,
        query_packs: &mut BTreeMap<PackName, CompiledQueryPack>,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<()> {
        for rule in loaded.pack.rules.values() {
            let query_pack = self.resolve_bundle_query_pack(
                loaded.pack.header.id.as_str(),
                loaded.source_file_base_dir.as_ref(),
                &rule.query.pack,
                memo,
                seen_pack_ids,
            )?;
            ensure_query_ref_exists(&loaded.pack.header.id, &rule.query, &query_pack.pack)?;
            query_packs
                .entry(query_pack.pack.header.id.clone())
                .or_insert(query_pack.pack);
        }
        Ok(())
    }

    fn resolve_recipe_pack_queries(
        &self,
        loaded: &LoadedCompiledRecipePack,
        query_packs: &mut BTreeMap<PackName, CompiledQueryPack>,
        memo: &mut BTreeMap<ResolvedPackSourceKey, ResolvedBundlePack>,
        seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
    ) -> PackResult<()> {
        for recipe in loaded.pack.recipes.values() {
            let query_pack = self.resolve_bundle_query_pack(
                loaded.pack.header.id.as_str(),
                loaded.source_file_base_dir.as_ref(),
                &recipe.query.pack,
                memo,
                seen_pack_ids,
            )?;
            ensure_query_ref_exists(&loaded.pack.header.id, &recipe.query, &query_pack.pack)?;
            query_packs
                .entry(query_pack.pack.header.id.clone())
                .or_insert(query_pack.pack);
        }
        Ok(())
    }

    fn detect_source_pack_kind(&self, source: &PackSource) -> PackResult<Option<PackKind>> {
        let loaded = self.load_source(source.clone())?;
        let value = match loaded.format {
            PackFormat::Json => {
                parse_json_document(&loaded.origin, &loaded.bytes, "pack_kind_probe")?
            }
            PackFormat::Toml => parse_profile_toml(&loaded.origin, &loaded.bytes)?,
        };
        Ok(value
            .as_object()
            .and_then(|object| object.get("kind"))
            .and_then(Value::as_str)
            .and_then(parse_pack_kind_string))
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum ResolvedPackSourceKey {
    Builtin(PackName),
    File(PathBuf),
    Inline(String),
}

#[derive(Clone, Debug)]
struct LoadedCompiledScoreModel {
    pack: CompiledScoreModel,
    source_file_base_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct LoadedCompiledQueryPack {
    pack: CompiledQueryPack,
    source_file_base_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct LoadedCompiledRulePack {
    pack: CompiledRulePack,
    source_file_base_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct LoadedCompiledRecipePack {
    pack: CompiledRecipePack,
    source_file_base_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
enum ResolvedBundlePack {
    ScoreModel(LoadedCompiledScoreModel),
    QueryPack(LoadedCompiledQueryPack),
    RulePack(LoadedCompiledRulePack),
    RecipePack(LoadedCompiledRecipePack),
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedScoreModelDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    vector_version: u32,
    lift_score: Value,
    estimated_slices: Value,
    triggers: Vec<NormalizedScoreTriggerRule>,
    confidence: NormalizedScoreConfidenceModel,
    missing_input_rules: Vec<NormalizedScoreMissingInputRule>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedScoreTriggerRule {
    id: String,
    when: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedScoreConfidenceModel {
    default: String,
    rules: Vec<NormalizedScoreConfidenceRule>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedScoreConfidenceRule {
    id: String,
    when: Value,
    set: String,
    causes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedScoreMissingInputRule {
    field: String,
    when: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedQueryPackDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    language: String,
    engine: String,
    queries: Vec<NormalizedQueryDef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedQueryDef {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    pattern: String,
    captures: Vec<NormalizedQueryCapture>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedQueryCapture {
    name: String,
    required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedRulePackDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    rules: Vec<NormalizedRuleDef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedRuleDef {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    severity: crate::kernel::Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<NormalizedRuleScope>,
    query: Value,
    emit: Vec<RawRuleEmit>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedRuleScope {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    languages: Vec<LanguageId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_classes: Vec<RawReservedPathClass>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedRecipePackDocument {
    kind: PackKind,
    version: u32,
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    recipes: Vec<NormalizedRecipeDef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NormalizedRecipeDef {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    query: Value,
    transforms: Vec<RawRecipeTransform>,
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

fn score_model_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| compile_embedded_schema_validator(PACK_SCORE_MODEL_V1_SCHEMA_JSON))
}

fn query_pack_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| compile_embedded_schema_validator(PACK_QUERY_PACK_V1_SCHEMA_JSON))
}

fn rule_pack_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| compile_embedded_schema_validator(PACK_RULE_PACK_V1_SCHEMA_JSON))
}

fn recipe_pack_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| compile_embedded_schema_validator(PACK_RECIPE_PACK_V1_SCHEMA_JSON))
}

fn compile_topology_schema_validator(schema_json: &str) -> Validator {
    compile_embedded_schema_validator(schema_json)
}

fn compile_embedded_schema_validator(schema_json: &str) -> Validator {
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

fn validate_score_model_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_advanced_document(origin, value, AdvancedSchemaKind::ScoreModel)
}

fn validate_query_pack_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_advanced_document(origin, value, AdvancedSchemaKind::QueryPack)
}

fn validate_rule_pack_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_advanced_document(origin, value, AdvancedSchemaKind::RulePack)
}

fn validate_recipe_pack_document(origin: &PackOrigin, value: &Value) -> Vec<PackDiagnostic> {
    validate_advanced_document(origin, value, AdvancedSchemaKind::RecipePack)
}

fn validate_advanced_document(
    origin: &PackOrigin,
    value: &Value,
    schema_kind: AdvancedSchemaKind,
) -> Vec<PackDiagnostic> {
    let mut diagnostics = Vec::new();
    for error in schema_kind.validator().iter_errors(value) {
        diagnostics.extend(advanced_schema_error_to_diagnostics(
            origin,
            schema_kind,
            &error,
        ));
    }
    diagnostics
}

fn advanced_schema_error_to_diagnostics(
    origin: &PackOrigin,
    schema_kind: AdvancedSchemaKind,
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
                "pack.schema.missing_required_field",
                format!("required field `{property}` is missing"),
                Some(location(origin, &pointer)),
            )]
        }
        ValidationErrorKind::Type { .. } if error.instance_path().as_str().is_empty() => {
            vec![PackDiagnostic::error(
                schema_kind.invalid_root_code(),
                error.to_string(),
                Some(PackLocation {
                    origin: origin.clone(),
                    path: None,
                }),
            )]
        }
        ValidationErrorKind::Constant { .. } if error.instance_path().as_str() == "/kind" => {
            vec![PackDiagnostic::error(
                "pack.schema.invalid_kind",
                error.to_string(),
                Some(location(origin, "/kind")),
            )]
        }
        ValidationErrorKind::Constant { .. } if error.instance_path().as_str() == "/version" => {
            vec![PackDiagnostic::error(
                "pack.schema.invalid_version",
                error.to_string(),
                Some(location(origin, "/version")),
            )]
        }
        ValidationErrorKind::Pattern { .. } if error.instance_path().as_str() == "/id" => {
            vec![PackDiagnostic::error(
                "pack.schema.invalid_pack_name",
                error.to_string(),
                Some(location(origin, "/id")),
            )]
        }
        _ => vec![PackDiagnostic::error(
            "pack.schema.invalid_value",
            error.to_string(),
            pointer_subject(origin, error.instance_path().as_str()),
        )],
    }
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

fn normalize_score_model(raw: &RawScoreModel) -> NormalizedScoreModelDocument {
    NormalizedScoreModelDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        vector_version: raw.vector_version,
        lift_score: raw.lift_score.clone(),
        estimated_slices: raw.estimated_slices.clone(),
        triggers: raw
            .triggers
            .iter()
            .map(|trigger| NormalizedScoreTriggerRule {
                id: trigger.id.clone(),
                when: trigger.when.clone(),
            })
            .collect(),
        confidence: NormalizedScoreConfidenceModel {
            default: raw.confidence.default_level.clone(),
            rules: raw
                .confidence
                .rules
                .iter()
                .map(|rule| NormalizedScoreConfidenceRule {
                    id: rule.id.clone(),
                    when: rule.when.clone(),
                    set: rule.set.clone(),
                    causes: sorted_unique_strings(rule.causes.clone()),
                })
                .collect(),
        },
        missing_input_rules: raw
            .missing_input_rules
            .iter()
            .map(|rule| NormalizedScoreMissingInputRule {
                field: rule.field.clone(),
                when: rule.when.clone(),
            })
            .collect(),
    }
}

fn normalize_query_pack(raw: &RawQueryPack) -> NormalizedQueryPackDocument {
    NormalizedQueryPackDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        language: raw.language.clone(),
        engine: raw.engine.clone(),
        queries: raw
            .queries
            .iter()
            .map(|query| NormalizedQueryDef {
                id: query.id.clone(),
                summary: query.summary.clone(),
                pattern: query.pattern.clone(),
                captures: query
                    .captures
                    .iter()
                    .map(|capture| NormalizedQueryCapture {
                        name: capture.name.clone(),
                        required: capture.required,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn normalize_rule_pack(raw: &RawRulePack) -> NormalizedRulePackDocument {
    NormalizedRulePackDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        rules: raw
            .rules
            .iter()
            .map(|rule| NormalizedRuleDef {
                id: rule.id.clone(),
                summary: rule.summary.clone(),
                severity: rule.severity,
                scope: rule.scope.as_ref().map(|scope| NormalizedRuleScope {
                    languages: scope.languages.clone().unwrap_or_default(),
                    path_classes: scope.path_classes.clone().unwrap_or_default(),
                }),
                query: serde_json::json!({
                    "pack": rule.query.pack.clone(),
                    "id": rule.query.id.clone(),
                }),
                emit: rule.emit.clone(),
            })
            .collect(),
    }
}

fn normalize_recipe_pack(raw: &RawRecipePack) -> NormalizedRecipePackDocument {
    NormalizedRecipePackDocument {
        kind: raw.kind,
        version: raw.version,
        id: raw.id.clone(),
        name: raw.name.clone(),
        description: raw.description.clone(),
        recipes: raw
            .recipes
            .iter()
            .map(|recipe| NormalizedRecipeDef {
                id: recipe.id.clone(),
                summary: recipe.summary.clone(),
                query: serde_json::json!({
                    "pack": recipe.query.pack.clone(),
                    "id": recipe.query.id.clone(),
                }),
                transforms: recipe.transforms.clone(),
            })
            .collect(),
    }
}

fn sorted_unique_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn parse_pack_kind_string(input: &str) -> Option<PackKind> {
    match input {
        "profile" => Some(PackKind::Profile),
        "boundary_taxonomy" => Some(PackKind::BoundaryTaxonomy),
        "component_map" => Some(PackKind::ComponentMap),
        "score_model" => Some(PackKind::ScoreModel),
        "rule_pack" => Some(PackKind::RulePack),
        "query_pack" => Some(PackKind::QueryPack),
        "recipe_pack" => Some(PackKind::RecipePack),
        _ => None,
    }
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

fn compile_normalized_score_model(
    origin: PackOrigin,
    source_file_base_dir: Option<PathBuf>,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedScoreModelDocument,
) -> PackResult<LoadedCompiledScoreModel> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let lift_score = compile_expr(
        &pack_id,
        &JsonPointer::parse("/lift_score").expect("pointer"),
        &normalized.lift_score,
    )?;
    let estimated_slices = compile_expr(
        &pack_id,
        &JsonPointer::parse("/estimated_slices").expect("pointer"),
        &normalized.estimated_slices,
    )?;

    let mut trigger_ids = BTreeSet::new();
    let mut triggers = Vec::with_capacity(normalized.triggers.len());
    for (index, trigger) in normalized.triggers.into_iter().enumerate() {
        if !trigger_ids.insert(trigger.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::ScoreModel,
                pack_id: pack_id.clone(),
                entry_kind: "trigger",
                entry_id: trigger.id,
            });
        }
        triggers.push(CompiledTriggerRule {
            id: trigger.id,
            when: compile_expr(
                &pack_id,
                &JsonPointer::parse(&format!("/triggers/{index}/when")).expect("pointer"),
                &trigger.when,
            )?,
        });
    }

    let mut confidence_rule_ids = BTreeSet::new();
    let mut confidence_rules = Vec::with_capacity(normalized.confidence.rules.len());
    for (index, rule) in normalized.confidence.rules.into_iter().enumerate() {
        if !confidence_rule_ids.insert(rule.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::ScoreModel,
                pack_id: pack_id.clone(),
                entry_kind: "confidence_rule",
                entry_id: rule.id,
            });
        }
        confidence_rules.push(CompiledConfidenceRule {
            id: rule.id,
            when: compile_expr(
                &pack_id,
                &JsonPointer::parse(&format!("/confidence/rules/{index}/when")).expect("pointer"),
                &rule.when,
            )?,
            set: rule.set,
            causes: rule.causes.into_iter().collect(),
        });
    }

    let mut missing_fields = BTreeSet::new();
    let mut missing_input_rules = Vec::with_capacity(normalized.missing_input_rules.len());
    for (index, rule) in normalized.missing_input_rules.into_iter().enumerate() {
        if !missing_fields.insert(rule.field.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::ScoreModel,
                pack_id: pack_id.clone(),
                entry_kind: "missing_input_field",
                entry_id: rule.field,
            });
        }
        let field =
            JsonPointer::parse(&rule.field).map_err(|error| PackError::ExpressionCompile {
                pack_id: pack_id.clone(),
                path: JsonPointer::parse(&format!("/missing_input_rules/{index}/field"))
                    .expect("pointer"),
                reason: error.to_string(),
            })?;
        missing_input_rules.push(CompiledMissingInputRule {
            field,
            when: compile_expr(
                &pack_id,
                &JsonPointer::parse(&format!("/missing_input_rules/{index}/when"))
                    .expect("pointer"),
                &rule.when,
            )?,
        });
    }

    Ok(LoadedCompiledScoreModel {
        pack: CompiledScoreModel {
            header: CompiledPackHeader {
                kind: PackKind::ScoreModel,
                id,
                version: normalized.version,
                name: normalized.name,
                description: normalized.description,
                schema_id: PACK_SCORE_MODEL_V1_SCHEMA_ID,
                origin,
                source_fingerprint,
                semantic_fingerprint,
            },
            vector_version: normalized.vector_version,
            lift_score,
            estimated_slices,
            triggers,
            confidence: CompiledConfidenceModel {
                default_level: normalized.confidence.default,
                rules: confidence_rules,
            },
            missing_input_rules,
            diagnostics: Vec::new(),
        },
        source_file_base_dir,
    })
}

fn compile_normalized_query_pack(
    origin: PackOrigin,
    source_file_base_dir: Option<PathBuf>,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedQueryPackDocument,
) -> PackResult<LoadedCompiledQueryPack> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let language = LanguageId::parse(&normalized.language)?;
    let engine = match normalized.engine.as_str() {
        "tree_sitter" => QueryEngineKind::TreeSitter,
        other => {
            return Err(PackError::SchemaViolation {
                origin: origin.display(),
                schema_id: PACK_QUERY_PACK_V1_SCHEMA_ID,
                diagnostics: vec![PackDiagnostic::error(
                    "pack.query_pack.unsupported_engine",
                    format!("unsupported query engine `{other}`"),
                    Some(location(&origin, "/engine")),
                )],
            });
        }
    };

    let mut local_ids = BTreeSet::new();
    let mut queries = BTreeMap::new();
    for query in normalized.queries {
        if !local_ids.insert(query.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::QueryPack,
                pack_id: pack_id.clone(),
                entry_kind: "query",
                entry_id: query.id,
            });
        }
        let query_id = compile_query_id(&pack_id, &query.id);
        queries.insert(
            query_id.clone(),
            CompiledQueryDef {
                local_id: query.id,
                id: query_id,
                summary: query.summary,
                pattern: query.pattern,
                captures: query
                    .captures
                    .into_iter()
                    .map(|capture| CompiledQueryCapture {
                        name: capture.name,
                        required: capture.required,
                    })
                    .collect(),
            },
        );
    }

    Ok(LoadedCompiledQueryPack {
        pack: CompiledQueryPack {
            header: CompiledPackHeader {
                kind: PackKind::QueryPack,
                id,
                version: normalized.version,
                name: normalized.name,
                description: normalized.description,
                schema_id: PACK_QUERY_PACK_V1_SCHEMA_ID,
                origin,
                source_fingerprint,
                semantic_fingerprint,
            },
            language,
            engine,
            queries,
            diagnostics: Vec::new(),
        },
        source_file_base_dir,
    })
}

fn compile_normalized_rule_pack(
    origin: PackOrigin,
    source_file_base_dir: Option<PathBuf>,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedRulePackDocument,
) -> PackResult<LoadedCompiledRulePack> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let mut local_ids = BTreeSet::new();
    let mut rules = BTreeMap::new();

    for (index, rule) in normalized.rules.into_iter().enumerate() {
        if !local_ids.insert(rule.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::RulePack,
                pack_id: pack_id.clone(),
                entry_kind: "rule",
                entry_id: rule.id,
            });
        }
        let rule_id = compile_rule_id(&pack_id, &rule.id);
        let query = compile_query_ref(
            &pack_id,
            &JsonPointer::parse(&format!("/rules/{index}/query")).expect("pointer"),
            &rule.query,
        )?;
        rules.insert(
            rule_id.clone(),
            CompiledRuleDef {
                local_id: rule.id,
                id: rule_id,
                summary: rule.summary,
                severity: rule.severity,
                scope: rule.scope.map(|scope| CompiledRuleScope {
                    languages: scope.languages.into_iter().collect(),
                    path_classes: scope
                        .path_classes
                        .into_iter()
                        .map(map_reserved_path_class)
                        .collect(),
                }),
                query,
                emit: rule
                    .emit
                    .into_iter()
                    .map(|emit| match emit {
                        RawRuleEmit::Finding { code, message } => {
                            CompiledRuleEmit::Finding { code, message }
                        }
                    })
                    .collect(),
            },
        );
    }

    Ok(LoadedCompiledRulePack {
        pack: CompiledRulePack {
            header: CompiledPackHeader {
                kind: PackKind::RulePack,
                id,
                version: normalized.version,
                name: normalized.name,
                description: normalized.description,
                schema_id: PACK_RULE_PACK_V1_SCHEMA_ID,
                origin,
                source_fingerprint,
                semantic_fingerprint,
            },
            rules,
            diagnostics: Vec::new(),
        },
        source_file_base_dir,
    })
}

fn compile_normalized_recipe_pack(
    origin: PackOrigin,
    source_file_base_dir: Option<PathBuf>,
    source_fingerprint: Fingerprint,
    semantic_fingerprint: Fingerprint,
    normalized: NormalizedRecipePackDocument,
) -> PackResult<LoadedCompiledRecipePack> {
    let id = PackName::parse(&normalized.id)?;
    let pack_id = normalized.id;
    let mut local_ids = BTreeSet::new();
    let mut recipes = BTreeMap::new();

    for (index, recipe) in normalized.recipes.into_iter().enumerate() {
        if !local_ids.insert(recipe.id.clone()) {
            return Err(PackError::DuplicateEntryId {
                pack_kind: PackKind::RecipePack,
                pack_id: pack_id.clone(),
                entry_kind: "recipe",
                entry_id: recipe.id,
            });
        }
        let recipe_id = compile_recipe_id(&pack_id, &recipe.id);
        let query = compile_query_ref(
            &pack_id,
            &JsonPointer::parse(&format!("/recipes/{index}/query")).expect("pointer"),
            &recipe.query,
        )?;
        recipes.insert(
            recipe_id.clone(),
            CompiledRecipeDef {
                local_id: recipe.id,
                id: recipe_id,
                summary: recipe.summary,
                query,
                transforms: recipe
                    .transforms
                    .into_iter()
                    .map(|transform| match transform {
                        RawRecipeTransform::ReplaceCaptureText { capture, text } => {
                            CompiledRecipeTransform::ReplaceCaptureText { capture, text }
                        }
                    })
                    .collect(),
            },
        );
    }

    Ok(LoadedCompiledRecipePack {
        pack: CompiledRecipePack {
            header: CompiledPackHeader {
                kind: PackKind::RecipePack,
                id,
                version: normalized.version,
                name: normalized.name,
                description: normalized.description,
                schema_id: PACK_RECIPE_PACK_V1_SCHEMA_ID,
                origin,
                source_fingerprint,
                semantic_fingerprint,
            },
            recipes,
            diagnostics: Vec::new(),
        },
        source_file_base_dir,
    })
}

fn compile_query_id(pack_id: &str, local_id: &str) -> QueryId {
    QueryId::from_identity(&format!("pack\0query_pack\0{pack_id}\0query\0{local_id}"))
}

fn compile_rule_id(pack_id: &str, local_id: &str) -> RuleId {
    RuleId::from_identity(&format!("pack\0rule_pack\0{pack_id}\0rule\0{local_id}"))
}

fn compile_recipe_id(pack_id: &str, local_id: &str) -> RecipeId {
    RecipeId::from_identity(&format!("pack\0recipe_pack\0{pack_id}\0recipe\0{local_id}"))
}

fn map_reserved_path_class(value: RawReservedPathClass) -> ReservedPathClass {
    match value {
        RawReservedPathClass::Test => ReservedPathClass::Test,
        RawReservedPathClass::Docs => ReservedPathClass::Docs,
        RawReservedPathClass::Ci => ReservedPathClass::Ci,
        RawReservedPathClass::Migration => ReservedPathClass::Migration,
        RawReservedPathClass::Security => ReservedPathClass::Security,
        RawReservedPathClass::PublicApi => ReservedPathClass::PublicApi,
        RawReservedPathClass::Generated => ReservedPathClass::Generated,
        RawReservedPathClass::Vendor => ReservedPathClass::Vendor,
    }
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
    resolve_file_source_from_base(
        profile.header.id.as_str(),
        profile.source_file_base_dir.as_ref(),
        relative_path,
        Some(PackFormat::Json),
    )
}

fn unknown_pack_reference(profile: &CompiledProfile, reference: &PackRef) -> PackError {
    PackError::UnknownPackReference {
        referring_pack: profile.header.id.as_str().to_owned(),
        reference: reference.as_str(),
    }
}

fn unknown_file_reference(profile: &CompiledProfile, relative_path: &str) -> PackError {
    unknown_file_reference_by_pack(profile.header.id.as_str(), relative_path)
}

fn unknown_file_reference_by_pack(referring_pack: &str, relative_path: &str) -> PackError {
    PackError::UnknownPackReference {
        referring_pack: referring_pack.to_owned(),
        reference: format!("file:{relative_path}"),
    }
}

fn resolve_file_source_from_base(
    referring_pack: &str,
    base_dir: Option<&PathBuf>,
    relative_path: &str,
    format_hint: Option<PackFormat>,
) -> PackResult<PackSource> {
    let Some(base) = base_dir else {
        return Err(unknown_file_reference_by_pack(
            referring_pack,
            relative_path,
        ));
    };

    let path = base.join(relative_path);
    match path.try_exists() {
        Ok(true) => {}
        Ok(false) => {
            if let Some(reason) = blocked_file_reference_reason(base, &path) {
                return Err(PackError::Io {
                    origin: path.display().to_string(),
                    reason,
                });
            }
            return Err(unknown_file_reference_by_pack(
                referring_pack,
                relative_path,
            ));
        }
        Err(error) => {
            return Err(PackError::Io {
                origin: path.display().to_string(),
                reason: error.to_string(),
            });
        }
    }

    Ok(PackSource::File { path, format_hint })
}

fn blocked_file_reference_reason(base: &Path, path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let relative_parent = parent.strip_prefix(base).ok()?;
    let mut probe = base.to_path_buf();

    for component in relative_parent.components() {
        probe.push(component.as_os_str());
        match fs::metadata(&probe) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    return Some(format!(
                        "path component is not a directory: {}",
                        probe.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => return Some(error.to_string()),
        }
    }

    None
}

fn resolve_bundle_ref_source(
    compiler: &PackCompiler,
    referring_pack: &str,
    referring_base_dir: Option<&PathBuf>,
    reference: &PackRef,
    expected: PackKind,
) -> PackResult<(PackSource, ResolvedPackSourceKey)> {
    match reference {
        PackRef::Builtin(name) => {
            if let Some(source) = builtin_source_for_kind(expected, name.as_str()) {
                Ok((source, ResolvedPackSourceKey::Builtin(name.clone())))
            } else if let Some(actual) = builtin_pack_kind(name.as_str()) {
                Err(PackError::RefKindMismatch {
                    reference: reference.as_str(),
                    expected,
                    actual,
                })
            } else {
                Err(PackError::UnknownPackReference {
                    referring_pack: referring_pack.to_owned(),
                    reference: reference.as_str(),
                })
            }
        }
        PackRef::File(path) => {
            let source = resolve_file_source_from_base(
                referring_pack,
                referring_base_dir,
                path.as_str(),
                None,
            )?;
            if let Some(actual) = compiler.detect_source_pack_kind(&source)? {
                if actual != expected {
                    return Err(PackError::RefKindMismatch {
                        reference: reference.as_str(),
                        expected,
                        actual,
                    });
                }
            }
            let key = bundle_source_key(&source)?;
            Ok((source, key))
        }
    }
}

fn bundle_source_key(source: &PackSource) -> PackResult<ResolvedPackSourceKey> {
    match source {
        PackSource::Builtin { logical_name, .. } => Ok(ResolvedPackSourceKey::Builtin(
            PackName::parse(logical_name)?,
        )),
        PackSource::File { path, .. } => Ok(ResolvedPackSourceKey::File(absolutize_path(path)?)),
        PackSource::Inline { .. } => Err(PackError::InvalidPackRef {
            input: "inline bundle sources are unsupported".to_owned(),
        }),
    }
}

fn builtin_source_for_kind(expected: PackKind, logical_name: &str) -> Option<PackSource> {
    match expected {
        PackKind::Profile => builtin::profile_source(logical_name),
        PackKind::BoundaryTaxonomy => builtin::boundary_taxonomy_source(logical_name),
        PackKind::ComponentMap => builtin::component_map_source(logical_name),
        PackKind::ScoreModel => builtin::score_model_source(logical_name),
        PackKind::RulePack => builtin::rule_pack_source(logical_name),
        PackKind::QueryPack => builtin::query_pack_source(logical_name),
        PackKind::RecipePack => builtin::recipe_pack_source(logical_name),
    }
}

fn builtin_pack_kind(logical_name: &str) -> Option<PackKind> {
    [
        (
            PackKind::Profile,
            builtin::profile_source(logical_name).is_some(),
        ),
        (
            PackKind::BoundaryTaxonomy,
            builtin::boundary_taxonomy_source(logical_name).is_some(),
        ),
        (
            PackKind::ComponentMap,
            builtin::component_map_source(logical_name).is_some(),
        ),
        (
            PackKind::ScoreModel,
            builtin::score_model_source(logical_name).is_some(),
        ),
        (
            PackKind::RulePack,
            builtin::rule_pack_source(logical_name).is_some(),
        ),
        (
            PackKind::QueryPack,
            builtin::query_pack_source(logical_name).is_some(),
        ),
        (
            PackKind::RecipePack,
            builtin::recipe_pack_source(logical_name).is_some(),
        ),
    ]
    .into_iter()
    .find_map(|(kind, present)| present.then_some(kind))
}

fn resolved_bundle_pack_kind(value: &ResolvedBundlePack) -> PackKind {
    match value {
        ResolvedBundlePack::ScoreModel(_) => PackKind::ScoreModel,
        ResolvedBundlePack::QueryPack(_) => PackKind::QueryPack,
        ResolvedBundlePack::RulePack(_) => PackKind::RulePack,
        ResolvedBundlePack::RecipePack(_) => PackKind::RecipePack,
    }
}

fn register_bundle_pack_id(
    header: &CompiledPackHeader,
    key: &ResolvedPackSourceKey,
    seen_pack_ids: &mut BTreeMap<PackName, ResolvedPackSourceKey>,
) -> PackResult<()> {
    match seen_pack_ids.get(&header.id) {
        Some(existing) if existing != key => Err(PackError::DuplicatePackId {
            kind: header.kind,
            id: header.id.as_str().to_owned(),
        }),
        Some(_) => Ok(()),
        None => {
            seen_pack_ids.insert(header.id.clone(), key.clone());
            Ok(())
        }
    }
}

fn resolved_pack_origin_key(origin: &PackOrigin) -> PackResult<ResolvedPackSourceKey> {
    match origin {
        PackOrigin::Builtin { logical_name } => Ok(ResolvedPackSourceKey::Builtin(
            PackName::parse(logical_name)?,
        )),
        PackOrigin::File { display_path } => Ok(ResolvedPackSourceKey::File(absolutize_path(
            Path::new(display_path),
        )?)),
        PackOrigin::Inline { logical_name } => {
            Ok(ResolvedPackSourceKey::Inline(logical_name.clone()))
        }
    }
}

fn ensure_query_ref_exists(
    referring_pack: &PackName,
    query_ref: &CompiledQueryRef,
    query_pack: &CompiledQueryPack,
) -> PackResult<()> {
    let query_id = compile_query_id(query_pack.header.id.as_str(), &query_ref.id);
    if query_pack.queries.contains_key(&query_id) {
        Ok(())
    } else {
        Err(PackError::UnknownPackReference {
            referring_pack: referring_pack.as_str().to_owned(),
            reference: format!("{}#{}", query_ref.pack.as_str(), query_ref.id),
        })
    }
}

fn compile_pack_set_fingerprint(
    profile: &CompiledProfile,
    topology: &ResolvedProfileTopology,
    score_model: Option<&CompiledScoreModel>,
    rule_packs: &BTreeMap<PackName, CompiledRulePack>,
    query_packs: &BTreeMap<PackName, CompiledQueryPack>,
    recipe_packs: &BTreeMap<PackName, CompiledRecipePack>,
) -> PackResult<Fingerprint> {
    #[derive(Serialize)]
    struct BundleFingerprintInput {
        profile: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        boundary_taxonomy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        component_map: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        score_model: Option<String>,
        rule_packs: BTreeMap<String, String>,
        query_packs: BTreeMap<String, String>,
        recipe_packs: BTreeMap<String, String>,
    }

    let input = BundleFingerprintInput {
        profile: profile.header.semantic_fingerprint.as_str().to_owned(),
        boundary_taxonomy: topology
            .boundary_taxonomy
            .as_ref()
            .map(|pack| pack.header.semantic_fingerprint.as_str().to_owned()),
        component_map: topology
            .component_map
            .as_ref()
            .map(|pack| pack.header.semantic_fingerprint.as_str().to_owned()),
        score_model: score_model.map(|pack| pack.header.semantic_fingerprint.as_str().to_owned()),
        rule_packs: rule_packs
            .iter()
            .map(|(name, pack)| {
                (
                    name.as_str().to_owned(),
                    pack.header.semantic_fingerprint.as_str().to_owned(),
                )
            })
            .collect(),
        query_packs: query_packs
            .iter()
            .map(|(name, pack)| {
                (
                    name.as_str().to_owned(),
                    pack.header.semantic_fingerprint.as_str().to_owned(),
                )
            })
            .collect(),
        recipe_packs: recipe_packs
            .iter()
            .map(|(name, pack)| {
                (
                    name.as_str().to_owned(),
                    pack.header.semantic_fingerprint.as_str().to_owned(),
                )
            })
            .collect(),
    };

    sha256_canonical_json(&input).map_err(|error| PackError::ParseFailure {
        origin: profile.header.origin.display(),
        diagnostics: vec![PackDiagnostic::error(
            "pack.bundle.canonicalization_failed",
            error.to_string(),
            Some(PackLocation {
                origin: profile.header.origin.clone(),
                path: None,
            }),
        )],
    })
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
