# Concrete Audit Report

Generated at: 2026-02-22T21:00:58-05:00

## Summary
- Files audited: 27
- Issues: 14 total (blocker 5 / critical 6 / major 2 / minor 1)

### Highest-risk gaps
1. CA-0001 — Unclear which documents are normative vs planning aids
2. CA-0002 — CONTRACT-2 model config: JSON shape and version selection are not concretely specified
3. CA-0003 — CONTRACT-3 `--emit-json`: missing concrete schema (types/enums/order/error model)
4. CA-0004 — CONTRACT-4 `validate_impact_map.py --emit-json`: base path + normalization + `dir_prefixes` semantics not fully specified
5. CA-0005 — `pm_lift.py` CLI surface is not pinned (positional vs flags, subcommand signatures)
6. CA-0006 — CONTRACT-1 schema requirements do not enumerate the v1 field set or per-field required/null rules
7. CA-0007 — Null semantics for Lift Vector counts are stated inconsistently (“numeric count inputs” vs specific fields)
8. CA-0008 — Behavior when schema/config artifacts are missing is not concretely defined (advisory vs strict)
9. CA-0009 — Prefix expansion is “optional” but lacks a default, determinism boundary, and failure-mode contract
10. CA-0010 — `confidence` is described with examples, but the deterministic mapping rules are not pinned

### Files with highest issue density
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md: 2
- work_lift_v1_seams/README.md: 1
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md: 1
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md: 1
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md: 1
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md: 1
- work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md: 1
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-2-config-backed-scoring-and-validation.md: 1
- work_lift_v1_seams/seam-4-pack-derived-lift-inputs.md: 1
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md: 1

## Issues

### CA-0001 — Unclear which documents are normative vs planning aids
- Severity: blocker
- Category: ownership
- Location: `work_lift_v1_seams/README.md` L5-L7
- Excerpt: “These files are planning aids; they are not normative contracts (authoritative contracts remain in the PM system docs + schemas).”
- Problem: Implementers cannot tell which statements in this directory are binding contracts vs non-authoritative guidance, and the referenced “PM system docs + schemas” are not concretely identified.
- Required to be concrete:
  - Name the authoritative source(s) of truth for each CONTRACT-* (exact repo path(s) and expected stable identifiers/anchors).
  - State whether `work_lift_v1_seams/threading.md` is itself normative for CONTRACT-* semantics or only a pointer/registry.
  - If threaded slices are the “canonical execution plan”, specify what parts are normative (deliverable paths, CLI contracts, schema contracts) vs advisory.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/README.md` L20-L22

### CA-0002 — CONTRACT-2 model config: JSON shape and version selection are not concretely specified
- Severity: blocker
- Category: contract
- Location: `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md` L34-L55
- Excerpt: “Keep v1 rules to tables/constants only: weights, multipliers, triggers, mapping constants, confidence degradation.”
- Problem: The docs describe what the config “contains” but do not pin the exact JSON structure, key names, precedence rules, or the deterministic algorithm for selecting a version (v1 vs v2). This leaves core scoring behavior under-specified.
- Required to be concrete:
  - Specify the exact JSON object structure for `work_lift_model.v1.json` (all required keys, types, and nested shapes).
  - Define precedence/ordering rules for applying weights → multipliers → triggers → mapping → confidence (including ties and rounding).
  - Define the version-selection algorithm as an ordered precedence list (e.g., CLI flag vs vector `model_version` vs default pin), including what happens when inputs disagree.
  - Define the behavior when the requested model version is unavailable (error vs fallback), including exit code and stderr requirements.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threading.md` L19-L30
  - `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md` L111-L140

### CA-0003 — CONTRACT-3 `--emit-json`: missing concrete schema (types/enums/order/error model)
- Severity: blocker
- Category: contract
- Location: `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` L13-L24
- Excerpt: “`pm_lift.py ... --emit-json` emits a JSON object with stable keys: `model_version`, `lift_score`, `estimated_slices`, `confidence`, `triggers`, `missing_inputs`, `vector`, `derived`.”
- Problem: The contract lists keys and some qualitative expectations, but it does not define a complete machine contract: exact types, allowed enum values, deterministic ordering rules, stable trigger string formats, which `derived` keys are stable vs debug-only, and whether errors ever emit JSON.
- Required to be concrete:
  - Define each required key’s JSON type (e.g., integer vs number, string enum values, object shapes).
  - Define the full allowed value set for `confidence` (no “e.g.”) and the deterministic mapping from inputs → confidence.
  - Define the stable format and allowed value set for `triggers` and `missing_inputs` (including ordering/sort rules).
  - Define whether `--emit-json` emits JSON on error; if not, require stdout empty and specify exit codes per error class.
  - Define which `derived` fields are stable contract fields vs “best-effort debug” (and how consumers should treat them).
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threading.md` L32-L42
  - `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` L35-L50

### CA-0004 — CONTRACT-4 `validate_impact_map.py --emit-json`: base path + normalization + `dir_prefixes` semantics not fully specified
- Severity: blocker
- Category: contract
- Location: `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md` L13-L20
- Excerpt: “CONTRACT-4 ... `create/edit/deprecate/delete`: arrays of normalized path tokens ... `dir_prefixes`: array of directory-prefix tokens ...”
- Problem: The contract names fields but does not define critical semantics needed for deterministic consumption: what paths are relative to (repo root vs pack root), path separator/case rules, deduplication expectations, and the exact derivation rules for `dir_prefixes`.
- Required to be concrete:
  - Define the path base and normalization rules precisely (relative root, allowed separators, case-sensitivity expectations, and canonicalization).
  - Specify whether allowlist arrays may contain duplicates and whether producers must de-duplicate.
  - Specify the exact algorithm for deriving `dir_prefixes` (dedupe rules, ordering, and relationship to per-action lists).
  - Define strict vs legacy behavior in terms of exit codes and stderr messaging (not only “empty arrays”).
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md` L40-L49
  - `work_lift_v1_seams/threading.md` L44-L50

### CA-0005 — `pm_lift.py` CLI surface is not pinned (positional vs flags, subcommand signatures)
- Severity: blocker
- Category: contract
- Location: `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md` L50-L53
- Excerpt: “Include three “how to run” recipes: `pm_lift.py from-intake <path>` ... `pm_lift.py from-impact-map <pack_dir>` ...”
- Problem: Docs and slices reference `pm_lift.py` usage with inconsistent argument styles, leaving the CLI contract ambiguous (subcommand names, required flags, positional parameters, and where `--emit-json` is supported). This blocks writing stable docs, Makefile targets, and tests.
- Required to be concrete:
  - Specify the canonical `pm_lift.py` CLI syntax for each subcommand (command name, required/optional flags, positional args, defaults).
  - Define how `--emit-json` composes with each subcommand (supported everywhere vs specific modes).
  - Define stable exit codes per subcommand and error class (consistent across modes).
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` L29-L31

### CA-0006 — CONTRACT-1 schema requirements do not enumerate the v1 field set or per-field required/null rules
- Severity: critical
- Category: schema
- Location: `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md` L31-L39
- Excerpt: “Schema property names/types match D6. Null allowances match D6.”
- Problem: The slice relies on an external “D6” field list but does not restate the field set or null allowances in a local, reviewable form. It also calls for “additive-friendly” evolution without specifying whether unknown keys are allowed or rejected in v1 validation.
- Required to be concrete:
  - List all v1 fields (full JSON paths) that the schema must define, including which are required vs optional.
  - For each numeric field, specify whether `null` is allowed and whether omission is allowed (and if so, how it differs from `null`).
  - Define the v1 policy for unknown/additive fields (e.g., `additionalProperties` behavior) and how versioning gates stricter validation.
  - Define the stable trigger naming convention for missing inputs (`missing_inputs:*`) using explicit JSON paths.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md` L20-L22
  - `work_lift_v1_seams/threading.md` L7-L18

### CA-0007 — Null semantics for Lift Vector counts are stated inconsistently (“numeric count inputs” vs specific fields)
- Severity: critical
- Category: consistency
- Location: `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md` L20-L22
- Excerpt: “Numeric count inputs may be represented as `null` to mean “unknown” (discovery-time)...”
- Problem: Some docs imply any numeric count may be `null`, while other parts of the plan single out specific fields (e.g., `touch.crates_touched` / `touch.boundary_crossings`). Without a per-field rule set, schema validation and scoring behavior will diverge.
- Required to be concrete:
  - State exactly which Lift Vector fields allow `null` in v1 (by full JSON path).
  - State whether other numeric fields are required to be present and non-null, and what failure mode applies if they are missing/null.
  - State whether `null` and missing are treated equivalently or differently for scoring/triggers/confidence.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/seam.md` L21-L26
  - `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md` L33-L35

### CA-0008 — Behavior when schema/config artifacts are missing is not concretely defined (advisory vs strict)
- Severity: critical
- Category: behavior
- Location: `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-2-config-backed-scoring-and-validation.md` L37-L39
- Excerpt: “if config/schema missing, fail with actionable guidance in strict modes; otherwise fall back conservatively (documented).”
- Problem: Multiple files mention bootstrap fallbacks and strict-mode gating, but none define the exact runtime conditions (what is “strict mode” for the tool), the specific fallback behavior, or the exit-code/stderr contract for missing schema/config.
- Required to be concrete:
  - Define how `pm_lift.py` determines whether it is in advisory vs strict mode (exact flag/env/config source).
  - Define behavior when config is missing in advisory mode (warn vs error), including deterministic stderr text elements and exit code.
  - Define behavior when schema is missing (and whether structural validation is allowed), including exit codes and how `--emit-json` behaves on these failures.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/seam-2-lift-model-config-v1.md` L40-L41
  - `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md` L31-L32

### CA-0009 — Prefix expansion is “optional” but lacks a default, determinism boundary, and failure-mode contract
- Severity: critical
- Category: operational
- Location: `work_lift_v1_seams/seam-4-pack-derived-lift-inputs.md` L8-L11
- Excerpt: “Optionally expand prefixes deterministically from the repo file list (e.g., `git ls-files <prefix>`) ... Degrade confidence when prefix entries are present...”
- Problem: The plan does not specify when prefix expansion runs (default on/off, flag-controlled, strict-mode-only), what constitutes the prefix namespace (repo-root relative vs pack-relative), or how errors/performance concerns are handled (git unavailable, large repos).
- Required to be concrete:
  - Define the default behavior for prefix expansion (always/never/by-flag) and the exact controlling flag/env/config key.
  - Define the prefix resolution base (repo root) and how prefixes are validated/canonicalized before expansion.
  - Define failure behavior when expansion cannot be performed (git missing, command fails), including exit codes and whether scoring proceeds using raw counts only.
  - Define performance constraints (max prefixes, max runtime) and deterministic output expectations under those constraints.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/seam.md` L11-L15
  - `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-2-prefix-expansion-and-derived-counts.md` L1-L17

### CA-0010 — `confidence` is described with examples, but the deterministic mapping rules are not pinned
- Severity: critical
- Category: behavior
- Location: `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` L17-L20
- Excerpt: “`confidence` is a low-cardinality string enum (e.g., `high|medium|low`) and is deterministic for a given input/config.”
- Problem: The contract uses “e.g.” for the enum values and does not specify the deterministic rule set mapping inputs (nulls, missing inputs, prefix entries) to a specific confidence value. Without this, downstream tools cannot safely interpret or gate on confidence.
- Required to be concrete:
  - Enumerate the exact allowed `confidence` values for v1.
  - Define deterministic confidence computation rules (including precedence when multiple degradation reasons apply).
  - Define how prefix entries and missing inputs affect confidence (and whether model config can change this in v1).
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/seam.md` L25-L25
  - `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md` L20-L21

### CA-0011 — Strict-mode onramp plan uses placeholders and does not pin invariant set selection
- Severity: critical
- Category: rollout
- Location: `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md` L44-L54
- Excerpt: “Define promotion criteria: N calibration runs over real packs, acceptable false positive rate, documented exceptions.”
- Problem: The strict-mode plan contains unresolved quantities (N calibration runs, acceptable false positive rate) and leaves the enforcement rule set under-specified (how invariants are selected and versioned). This prevents implementers from writing a deterministic strict checker.
- Required to be concrete:
  - Pin the promotion criteria as specific, measurable thresholds (exact N, exact false-positive target, and what constitutes a “run”).
  - Define the initial strict invariant set as an explicit list (by CONTRACT-3 fields) and the exact failure messages/exit codes when violated.
  - Define how the strict invariant set is selected (config file path + schema, or hard-coded list) and how it is versioned.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md` L79-L83

### CA-0012 — Deliverable file paths are underspecified via wildcards/“choose one”/“e.g.” patterns
- Severity: major
- Category: operational
- Location: `work_lift_v1_seams/seam-5-advisory-workflow-integration.md` L34-L38
- Excerpt: “`docs/project_management/system/standards/planning/*` (as needed for “where lift fits”)”
- Problem: Several slices leave key deliverables ambiguous (wildcard paths, “choose one” output formats, and example-only file paths). This makes work ownership and review criteria non-deterministic.
- Required to be concrete:
  - Replace wildcard deliverables with exact file paths for each doc/script/test artifact.
  - When alternatives are listed (“choose one”), pick one and state it as the committed output path/format.
  - When example-only paths are used (“e.g.”), either pin the exact path or explicitly mark the item as non-contractual guidance.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md` L45-L48
  - `work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-2-goldens-and-conformance.md` L41-L44
  - `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md` L41-L44
  - `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md` L40-L43

### CA-0013 — Test invocation is referenced but not specified as an executable command
- Severity: major
- Category: testing
- Location: `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-3-goldens-and-conformance.md` L16-L18
- Excerpt: “A single command (python test invocation for planning scripts) runs: golden cases ... negative cases ...”
- Problem: The plan calls for “a single command” but does not specify the exact command, test runner, working directory assumptions, or required environment variables. This prevents reproducible verification.
- Required to be concrete:
  - Specify the exact command line to run the planning script tests (runner, arguments, and working directory).
  - State any environment prerequisites (python version, dependencies, PATH requirements for git) needed for deterministic execution.
  - Define where fixtures live and the required on-disk layout for tests to pass.
- Suggested evidence order: codebase → docs → external → decision

### CA-0014 — Non-contract language (“roughly”, “may”) appears in goal/success framing without explicit bounds
- Severity: minor
- Category: language
- Location: `work_lift_v1_seams/scope_brief.md` L3-L7
- Excerpt: “PM / planning owners: quickly assess whether an ADR candidate should be split and roughly how many slices/checkpoints it implies.”
- Problem: Phrases like “roughly” and “may” are fine in narrative context, but here they sit adjacent to execution expectations without defining what accuracy/variance is acceptable, which can lead to inconsistent interpretation of success criteria.
- Required to be concrete:
  - Clarify whether “roughly” is purely narrative or implies an accuracy target (and if so, state the target).
  - Where behavior is implied (e.g., splitting decisions), reference the concrete trigger thresholds and mapping rules that govern it.
- Suggested evidence order: codebase → docs → external → decision
- Cross-references:
  - `work_lift_v1_seams/scope_brief.md` L36-L39

## Audited files
- work_lift_v1_seams/README.md
- work_lift_v1_seams/scope_brief.md
- work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md
- work_lift_v1_seams/seam-2-lift-model-config-v1.md
- work_lift_v1_seams/seam-3-pm-lift-core-engine.md
- work_lift_v1_seams/seam-4-pack-derived-lift-inputs.md
- work_lift_v1_seams/seam-5-advisory-workflow-integration.md
- work_lift_v1_seams/seam_map.md
- work_lift_v1_seams/threaded-seams/README.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/seam.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md
- work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-2-human-rubric-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/seam.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-1-contract-2-model-config.md
- work_lift_v1_seams/threaded-seams/seam-2-lift-model-config-v1/slice-2-goldens-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/seam.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-2-config-backed-scoring-and-validation.md
- work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-3-goldens-and-conformance.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/seam.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md
- work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-2-prefix-expansion-and-derived-counts.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/seam.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-2-advisory-report-hook-and-lint-integration.md
- work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-3-strict-mode-onramp-plan.md
- work_lift_v1_seams/threading.md
