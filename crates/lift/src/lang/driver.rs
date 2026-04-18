use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::kernel::{Diagnostic, DiagnosticCode, Locator, RepoPath, Severity, SymbolId};
use crate::lang::{
    compare_symbol_drafts, sort_local_edges, sort_local_symbols, sort_surface_markers,
    symbol_identity_lemma, AdapterDescriptor, AdapterParseOutput, AdapterParseResult, EdgeEndpoint,
    EdgeEndpointDraft, FailedParse, LangError, LangResult, LanguageAdapter, LanguageId,
    LanguageRegistry, LocalEdge, LocalSymbol, LocalSymbolDraft, MissingRequestedLanguage,
    ParseInput, ParseRequest, ParseScope, ParseSet, ParseStats, ParsedUnit, ReferenceTarget,
    ReferenceTargetDraft, SkippedParse, SkippedReason, SurfaceMarker,
};
use crate::repo::{InventoryEntry, RepoSnapshot};

pub(crate) struct ParseDriver {
    registry: LanguageRegistry,
}

impl ParseDriver {
    pub(crate) fn new(registry: LanguageRegistry) -> Self {
        Self { registry }
    }

    pub(crate) fn parse_snapshot(
        &self,
        snapshot: &RepoSnapshot,
        request: &ParseRequest,
    ) -> LangResult<ParseSet> {
        let normalized_request = request.normalized();
        let request_fingerprint =
            normalized_request
                .fingerprint()
                .map_err(|error| LangError::SchemaViolation {
                    schema_id: "lang.parse.request",
                    reason: error.to_string(),
                })?;

        let mut parse_set = ParseSet {
            snapshot_fingerprint: snapshot.fingerprint.clone(),
            request: normalized_request.clone(),
            request_fingerprint,
            units: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            missing_languages: Vec::new(),
            diagnostics: Vec::new(),
            stats: ParseStats::default(),
        };

        let selected_languages = self.selected_languages(&normalized_request, &mut parse_set);
        match &normalized_request.scope {
            ParseScope::Snapshot => {
                for entry in snapshot.inventory.iter() {
                    self.process_entry(
                        snapshot,
                        entry,
                        &selected_languages,
                        false,
                        &mut parse_set,
                    )?;
                }
            }
            ParseScope::Paths(paths) => {
                for path in paths {
                    let Some(entry) = snapshot.entry(path) else {
                        parse_set.stats.skipped_missing_paths += 1;
                        parse_set.skipped.push(SkippedParse {
                            path: path.clone(),
                            file_id: None,
                            reason: SkippedReason::PathNotInSnapshot,
                            detail: Some("requested path not present in snapshot".to_owned()),
                        });
                        continue;
                    };
                    self.process_entry(snapshot, entry, &selected_languages, true, &mut parse_set)?;
                }
            }
        }

        parse_set.sort_all();
        parse_set.refresh_stats();
        Ok(parse_set)
    }

    fn selected_languages(
        &self,
        request: &ParseRequest,
        parse_set: &mut ParseSet,
    ) -> Vec<LanguageId> {
        if request.languages.is_empty() {
            return self
                .registry
                .descriptors()
                .into_iter()
                .map(|descriptor| descriptor.language)
                .collect();
        }

        let mut selected = Vec::new();
        for language in &request.languages {
            if self.registry.adapter_for_language(*language).is_some() {
                selected.push(*language);
                continue;
            }

            parse_set.missing_languages.push(MissingRequestedLanguage {
                language: *language,
                detail: Some(format!(
                    "no registered adapter is available for language '{}'",
                    language.as_str()
                )),
            });
            parse_set.diagnostics.push(error_diagnostic(
                "lang.parse.missing_registered_adapter",
                format!(
                    "requested language '{}' has no registered adapter",
                    language.as_str()
                ),
                None,
            ));
        }

        selected
    }

    fn process_entry(
        &self,
        snapshot: &RepoSnapshot,
        entry: &InventoryEntry,
        selected_languages: &[LanguageId],
        explicit_path: bool,
        parse_set: &mut ParseSet,
    ) -> LangResult<()> {
        parse_set.stats.considered_files += 1;
        let bytes =
            snapshot
                .read_bytes(&entry.path)
                .map_err(|error| LangError::CacheInvariant {
                    reason: error.to_string(),
                })?;
        let input = ParseInput {
            path: &entry.path,
            file_id: &entry.file_id,
            blob_fingerprint: &entry.blob_fingerprint,
            bytes,
        };

        let selection = self.select_candidate(&input, selected_languages);
        match selection {
            CandidateSelection::NoMatch => {
                parse_set.stats.skipped_no_adapter += 1;
                if explicit_path {
                    parse_set.skipped.push(SkippedParse {
                        path: entry.path.clone(),
                        file_id: Some(entry.file_id.clone()),
                        reason: SkippedReason::NoMatchingAdapter,
                        detail: Some("no selected adapter recognized file".to_owned()),
                    });
                }
            }
            CandidateSelection::Failed(failed) => parse_set.failed.push(failed),
            CandidateSelection::Adapter {
                adapter,
                descriptor,
            } => match self.invoke_parse(&adapter, &descriptor, &input) {
                ParseAttempt::Failed(failed) => parse_set.failed.push(failed),
                ParseAttempt::Parsed(output) => {
                    match normalize_parsed_output(&descriptor, &input, output)? {
                        NormalizedFile::Parsed(unit) => parse_set.units.push(unit),
                        NormalizedFile::Failed(failed) => parse_set.failed.push(failed),
                    }
                }
            },
        }

        Ok(())
    }

    fn select_candidate(
        &self,
        input: &ParseInput<'_>,
        selected_languages: &[LanguageId],
    ) -> CandidateSelection {
        for language in selected_languages {
            let Some(adapter) = self.registry.adapter_for_language(*language).cloned() else {
                continue;
            };
            let descriptor = adapter.descriptor();
            match catch_unwind(AssertUnwindSafe(|| adapter.recognizes(input))) {
                Ok(true) => {
                    return CandidateSelection::Adapter {
                        adapter,
                        descriptor,
                    };
                }
                Ok(false) => {}
                Err(payload) => {
                    return CandidateSelection::Failed(FailedParse {
                        path: input.path.clone(),
                        file_id: input.file_id.clone(),
                        blob_fingerprint: input.blob_fingerprint.clone(),
                        language: descriptor.language,
                        adapter: descriptor.name,
                        adapter_version: descriptor.version,
                        diagnostics: vec![error_diagnostic(
                            "lang.parse.adapter_panic",
                            format!(
                                "adapter panicked during recognizes(): {}",
                                panic_message(payload)
                            ),
                            Some(Locator {
                                path: input.path.clone(),
                                span: None,
                                json_pointer: None,
                            }),
                        )],
                    });
                }
            }
        }

        CandidateSelection::NoMatch
    }

    fn invoke_parse(
        &self,
        adapter: &Arc<dyn LanguageAdapter>,
        descriptor: &AdapterDescriptor,
        input: &ParseInput<'_>,
    ) -> ParseAttempt {
        match catch_unwind(AssertUnwindSafe(|| adapter.parse(input))) {
            Ok(AdapterParseResult::Parsed(output)) => ParseAttempt::Parsed(output),
            Ok(AdapterParseResult::Failed { diagnostics }) => {
                ParseAttempt::Failed(normalize_failed_parse(descriptor, input, diagnostics))
            }
            Err(payload) => ParseAttempt::Failed(FailedParse {
                path: input.path.clone(),
                file_id: input.file_id.clone(),
                blob_fingerprint: input.blob_fingerprint.clone(),
                language: descriptor.language,
                adapter: descriptor.name.clone(),
                adapter_version: descriptor.version.clone(),
                diagnostics: vec![error_diagnostic(
                    "lang.parse.adapter_panic",
                    format!(
                        "adapter panicked during parse(): {}",
                        panic_message(payload)
                    ),
                    Some(Locator {
                        path: input.path.clone(),
                        span: None,
                        json_pointer: None,
                    }),
                )],
            }),
        }
    }
}

enum CandidateSelection {
    NoMatch,
    Failed(FailedParse),
    Adapter {
        adapter: Arc<dyn LanguageAdapter>,
        descriptor: AdapterDescriptor,
    },
}

enum ParseAttempt {
    Failed(FailedParse),
    Parsed(AdapterParseOutput),
}

enum NormalizedFile {
    Parsed(ParsedUnit),
    Failed(FailedParse),
}

fn normalize_parsed_output(
    descriptor: &AdapterDescriptor,
    input: &ParseInput<'_>,
    output: AdapterParseOutput,
) -> LangResult<NormalizedFile> {
    let file_len = input.bytes.len() as u64;
    let (mut diagnostics, mut issues) =
        sanitize_diagnostics(input.path, file_len, output.diagnostics);

    let mut by_local_key = BTreeMap::<String, LocalSymbolDraft>::new();
    let mut symbol_drafts = output.symbols;
    symbol_drafts.sort_by(compare_symbol_drafts);
    for draft in &symbol_drafts {
        if draft.local_key.is_empty() {
            issues.push(error_diagnostic(
                "lang.parse.invalid_local_key",
                "symbol local_key must not be empty".to_owned(),
                Some(Locator {
                    path: input.path.clone(),
                    span: Some(draft.span),
                    json_pointer: None,
                }),
            ));
        }
        if draft.span.end_byte > file_len {
            issues.push(invalid_span_diagnostic(
                input.path,
                Some(draft.span),
                format!(
                    "symbol span [{}..{}) exceeds file length {}",
                    draft.span.start_byte, draft.span.end_byte, file_len
                ),
            ));
        }
        if by_local_key
            .insert(draft.local_key.clone(), draft.clone())
            .is_some()
        {
            issues.push(error_diagnostic(
                "lang.parse.duplicate_local_key",
                format!("duplicate symbol local_key '{}'", draft.local_key),
                Some(Locator {
                    path: input.path.clone(),
                    span: Some(draft.span),
                    json_pointer: None,
                }),
            ));
        }
    }

    if !issues.is_empty() {
        diagnostics.extend(issues);
        diagnostics.sort();
        return Ok(NormalizedFile::Failed(FailedParse {
            path: input.path.clone(),
            file_id: input.file_id.clone(),
            blob_fingerprint: input.blob_fingerprint.clone(),
            language: descriptor.language,
            adapter: descriptor.name.clone(),
            adapter_version: descriptor.version.clone(),
            diagnostics,
        }));
    }

    let mut symbols = Vec::with_capacity(symbol_drafts.len());
    let mut symbol_ids = BTreeMap::<String, SymbolId>::new();
    let mut previous_identity: Option<(
        crate::lang::SymbolKind,
        Vec<String>,
        Option<String>,
        crate::kernel::ByteSpan,
    )> = None;
    let mut duplicate_ordinal = 0usize;
    for draft in &symbol_drafts {
        let current_identity = (
            draft.kind,
            draft.path.clone(),
            draft.name.clone(),
            draft.span,
        );
        if previous_identity.as_ref() == Some(&current_identity) {
            duplicate_ordinal += 1;
        } else {
            duplicate_ordinal = 0;
            previous_identity = Some(current_identity);
        }

        let id = SymbolId::from_identity(&symbol_identity_lemma(
            descriptor.language,
            input.path,
            draft,
            duplicate_ordinal,
        ));
        symbol_ids.insert(draft.local_key.clone(), id.clone());
        symbols.push(LocalSymbol {
            id,
            kind: draft.kind,
            name: draft.name.clone(),
            path: draft.path.clone(),
            span: draft.span,
            visibility: draft.visibility,
        });
    }

    let mut edges = Vec::with_capacity(output.edges.len());
    for edge in output.edges {
        if let Some(span) = edge.span {
            if span.end_byte > file_len {
                issues.push(invalid_span_diagnostic(
                    input.path,
                    Some(span),
                    format!(
                        "edge span [{}..{}) exceeds file length {}",
                        span.start_byte, span.end_byte, file_len
                    ),
                ));
                continue;
            }
        }

        let source = match resolve_edge_endpoint(&symbol_ids, &edge.source) {
            Ok(source) => source,
            Err(local_key) => {
                issues.push(error_diagnostic(
                    "lang.parse.unresolved_local_ref",
                    format!("edge source references missing local_key '{}'", local_key),
                    Some(Locator {
                        path: input.path.clone(),
                        span: edge.span,
                        json_pointer: None,
                    }),
                ));
                continue;
            }
        };
        let target = match resolve_reference_target(&symbol_ids, &edge.target) {
            Ok(target) => target,
            Err(local_key) => {
                issues.push(error_diagnostic(
                    "lang.parse.unresolved_local_ref",
                    format!("edge target references missing local_key '{}'", local_key),
                    Some(Locator {
                        path: input.path.clone(),
                        span: edge.span,
                        json_pointer: None,
                    }),
                ));
                continue;
            }
        };

        edges.push(LocalEdge {
            kind: edge.kind,
            source,
            target,
            span: edge.span,
        });
    }

    let mut surface_markers = Vec::with_capacity(output.surface_markers.len());
    for marker in output.surface_markers {
        if let Some(span) = marker.span {
            if span.end_byte > file_len {
                issues.push(invalid_span_diagnostic(
                    input.path,
                    Some(span),
                    format!(
                        "surface marker span [{}..{}) exceeds file length {}",
                        span.start_byte, span.end_byte, file_len
                    ),
                ));
                continue;
            }
        }

        let symbol = match marker.symbol_local_key {
            Some(local_key) => match symbol_ids.get(&local_key).cloned() {
                Some(symbol) => Some(symbol),
                None => {
                    issues.push(error_diagnostic(
                        "lang.parse.unresolved_marker_ref",
                        format!(
                            "surface marker references missing local_key '{}'",
                            local_key
                        ),
                        Some(Locator {
                            path: input.path.clone(),
                            span: marker.span,
                            json_pointer: None,
                        }),
                    ));
                    continue;
                }
            },
            None => None,
        };

        surface_markers.push(SurfaceMarker {
            kind: marker.kind,
            symbol,
            span: marker.span,
            label: marker.label,
        });
    }

    if !issues.is_empty() {
        diagnostics.extend(issues);
        diagnostics.sort();
        return Ok(NormalizedFile::Failed(FailedParse {
            path: input.path.clone(),
            file_id: input.file_id.clone(),
            blob_fingerprint: input.blob_fingerprint.clone(),
            language: descriptor.language,
            adapter: descriptor.name.clone(),
            adapter_version: descriptor.version.clone(),
            diagnostics,
        }));
    }

    sort_local_symbols(&mut symbols);
    sort_local_edges(&mut edges);
    sort_surface_markers(&mut surface_markers);
    diagnostics.sort();

    let mut unit = ParsedUnit {
        path: input.path.clone(),
        file_id: input.file_id.clone(),
        blob_fingerprint: input.blob_fingerprint.clone(),
        language: descriptor.language,
        adapter: descriptor.name.clone(),
        adapter_version: descriptor.version.clone(),
        unit_fingerprint: crate::kernel::sha256_bytes(b""),
        symbols,
        edges,
        surface_markers,
        diagnostics,
    };
    unit.unit_fingerprint = unit
        .fingerprint()
        .map_err(|error| LangError::SchemaViolation {
            schema_id: "lang.parse.unit",
            reason: error.to_string(),
        })?;

    Ok(NormalizedFile::Parsed(unit))
}

fn normalize_failed_parse(
    descriptor: &AdapterDescriptor,
    input: &ParseInput<'_>,
    diagnostics: Vec<Diagnostic>,
) -> FailedParse {
    let file_len = input.bytes.len() as u64;
    let (mut valid, mut issues) = sanitize_diagnostics(input.path, file_len, diagnostics);
    if valid.is_empty() && issues.is_empty() {
        valid.push(error_diagnostic(
            "lang.parse.failed",
            "adapter reported parse failure".to_owned(),
            Some(Locator {
                path: input.path.clone(),
                span: None,
                json_pointer: None,
            }),
        ));
    }
    valid.append(&mut issues);
    valid.sort();

    FailedParse {
        path: input.path.clone(),
        file_id: input.file_id.clone(),
        blob_fingerprint: input.blob_fingerprint.clone(),
        language: descriptor.language,
        adapter: descriptor.name.clone(),
        adapter_version: descriptor.version.clone(),
        diagnostics: valid,
    }
}

fn sanitize_diagnostics(
    path: &RepoPath,
    file_len: u64,
    diagnostics: Vec<Diagnostic>,
) -> (Vec<Diagnostic>, Vec<Diagnostic>) {
    let mut valid = Vec::new();
    let mut issues = Vec::new();

    for diagnostic in diagnostics {
        if locator_span_invalid(diagnostic.subject.as_ref(), path, file_len) {
            issues.push(invalid_span_diagnostic(
                path,
                diagnostic.subject.as_ref().and_then(|locator| locator.span),
                "diagnostic subject span exceeds file length".to_owned(),
            ));
            continue;
        }
        if diagnostic
            .related
            .iter()
            .any(|related| locator_span_invalid(Some(&related.locator), path, file_len))
        {
            let span = diagnostic
                .related
                .iter()
                .find_map(|related| (related.locator.path == *path).then_some(related.locator.span))
                .flatten();
            issues.push(invalid_span_diagnostic(
                path,
                span,
                "diagnostic related span exceeds file length".to_owned(),
            ));
            continue;
        }
        valid.push(diagnostic);
    }

    valid.sort();
    issues.sort();
    (valid, issues)
}

fn locator_span_invalid(locator: Option<&Locator>, path: &RepoPath, file_len: u64) -> bool {
    locator
        .filter(|locator| locator.path == *path)
        .and_then(|locator| locator.span)
        .is_some_and(|span| span.end_byte > file_len)
}

fn resolve_edge_endpoint(
    symbol_ids: &BTreeMap<String, SymbolId>,
    source: &EdgeEndpointDraft,
) -> Result<EdgeEndpoint, String> {
    match source {
        EdgeEndpointDraft::FileRoot => Ok(EdgeEndpoint::FileRoot),
        EdgeEndpointDraft::Symbol { local_key } => symbol_ids
            .get(local_key)
            .cloned()
            .map(EdgeEndpoint::Symbol)
            .ok_or_else(|| local_key.clone()),
    }
}

fn resolve_reference_target(
    symbol_ids: &BTreeMap<String, SymbolId>,
    target: &ReferenceTargetDraft,
) -> Result<ReferenceTarget, String> {
    match target {
        ReferenceTargetDraft::LocalSymbol { local_key } => symbol_ids
            .get(local_key)
            .cloned()
            .map(ReferenceTarget::LocalSymbol)
            .ok_or_else(|| local_key.clone()),
        ReferenceTargetDraft::QualifiedName { parts } => Ok(ReferenceTarget::QualifiedName {
            parts: parts.clone(),
        }),
        ReferenceTargetDraft::FilePath { path } => {
            Ok(ReferenceTarget::FilePath { path: path.clone() })
        }
        ReferenceTargetDraft::ExternalPackage { package, symbol } => {
            Ok(ReferenceTarget::ExternalPackage {
                package: package.clone(),
                symbol: symbol.clone(),
            })
        }
        ReferenceTargetDraft::Opaque { value } => Ok(ReferenceTarget::Opaque {
            value: value.clone(),
        }),
    }
}

fn error_diagnostic(code: &str, message: String, subject: Option<Locator>) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::parse(code).expect("lang diagnostic code should be valid"),
        severity: Severity::Error,
        message,
        subject,
        related: Vec::new(),
        help: None,
    }
}

fn invalid_span_diagnostic(
    path: &RepoPath,
    span: Option<crate::kernel::ByteSpan>,
    message: String,
) -> Diagnostic {
    error_diagnostic(
        "lang.parse.invalid_span",
        message,
        Some(Locator {
            path: path.clone(),
            span,
            json_pointer: None,
        }),
    )
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    let payload = match payload.downcast::<String>() {
        Ok(message) => return *message,
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<&'static str>() {
        Ok(message) => return (*message).to_owned(),
        Err(payload) => payload,
    };
    format!("non-string panic payload ({:?})", payload.type_id())
}
