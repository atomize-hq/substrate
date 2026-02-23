# Contradictions Audit Report

## Meta
- Generated at: 2026-02-22T19:53:38-05:00
- Files audited: 26 (all files under `work_lift_v1_seams/`)
- Scan used: no

## Summary
- Total issues: 4
- By severity: blocker=0, critical=2, major=2, minor=0
- High-confidence contradictions: 2

## Issue index
| ID | Severity | Confidence | Type | Subject | Files |
|---|---|---|---|---|---|
| CX-0001 | critical | high | schema | Lift Vector v1: which numeric fields may be null | work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md; work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md |
| CX-0002 | critical | high | scope_mismatch | Lift Vector v1: is `model_version` required or defaultable when missing? | work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md; work_lift_v1_seams/seam-3-pm-lift-core-engine.md |
| CX-0003 | major | medium | api | CLI contract: `pm_lift.py from-git-diff` argument form | work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md; work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md |
| CX-0004 | major | medium | scope_mismatch | Pack-derived lift: `from-impact-map` expectations vs legacy-mode `validate_impact_map --emit-json` semantics | work_lift_v1_seams/scope_brief.md; work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md |

## Issues

### CX-0001 — Lift Vector v1 null allowances conflict
- Severity: critical
- Confidence: high
- Type: schema
- Subject: Lift Vector v1: which numeric fields may be null
- Scope: env=all, version=v1, feature_flag=none, timeline=planned
- Statement A: `work_lift_v1_seams/seam-1-lift-vector-schema-and-rubric.md:20-22`
  - Excerpt: Missing numeric inputs may be represented as `null` (general rule).
- Statement B: `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md:34-36`
  - Excerpt: Allow `null` only for `touch.crates_touched` and `touch.boundary_crossings`.
- Why this conflicts: The seam pack simultaneously describes `null` as broadly permitted for missing numeric inputs and as narrowly permitted for only two specific fields.
- What must be true:
  - The exhaustive v1 set of fields that permit `null`, and whether `null` is allowed in authored input or only via tool-internal normalization.
  - Tightened documentation so every seam refers to the same `null` policy (or explicitly scopes exceptions).
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision

### CX-0002 — `model_version` requiredness vs defaulting
- Severity: critical
- Confidence: high
- Type: scope_mismatch
- Subject: Lift Vector v1: is `model_version` required or defaultable when missing?
- Scope: env=all, version=v1, feature_flag=none, timeline=planned
- Statement A: `work_lift_v1_seams/threaded-seams/seam-1-lift-vector-schema-and-rubric/slice-1-contract-1-schema.md:31-32`
  - Excerpt: `model_version` is a required top-level key.
- Statement B: `work_lift_v1_seams/seam-3-pm-lift-core-engine.md:41-43`
  - Excerpt: selection rule may “default to 1 if missing”.
- Why this conflicts: A required key cannot be “missing” in authored Lift Vector blocks unless the schema/tool has an explicit legacy/bootstrapping exception.
- What must be true:
  - One canonical rule for whether authored Lift Vector blocks must include `model_version`.
  - If defaulting is supported, the exact scope (advisory vs strict) and how it interacts with schema validation (pre-normalization vs schema allowing omission).
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision

### CX-0003 — `from-git-diff` CLI surface is inconsistent
- Severity: major
- Confidence: medium
- Type: api
- Subject: CLI contract: `pm_lift.py from-git-diff` argument form
- Scope: env=all, version=v1, feature_flag=none, timeline=planned
- Statement A: `work_lift_v1_seams/threaded-seams/seam-3-pm-lift-core-engine/slice-1-contract-3-emit-json.md:29-32`
  - Excerpt: `pm_lift.py from-git-diff --git-range HEAD~1..HEAD --emit-json`
- Statement B: `work_lift_v1_seams/threaded-seams/seam-5-advisory-workflow-integration/slice-1-advisory-workflow-docs-and-make-targets.md:50-53`
  - Excerpt: `pm_lift.py from-git-diff <base> <head>`
- Why this conflicts: The docs specify two different argument conventions for the same subcommand, making downstream Makefile targets and workflow docs ambiguous unless both forms are intentionally supported and documented.
- What must be true:
  - The canonical `from-git-diff` CLI definition (required args/flags and accepted forms).
  - If both forms are supported, an explicit equivalence mapping and precedence rules.
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision

### CX-0004 — Pack-derived lift support scope is unclear for legacy packs
- Severity: major
- Confidence: medium
- Type: scope_mismatch
- Subject: Pack-derived lift: `from-impact-map` expectations vs legacy-mode `validate_impact_map --emit-json` semantics
- Scope: env=all, version=v1, feature_flag=none, timeline=planned
- Statement A: `work_lift_v1_seams/scope_brief.md:21-26`
  - Excerpt: `pm_lift.py` computes advisory lift from Planning Pack `impact_map.md` via `validate_impact_map.py --emit-json`.
- Statement B: `work_lift_v1_seams/threaded-seams/seam-4-pack-derived-lift-inputs/slice-1-contract-4-impact-map-emit-json.md:16-17`
  - Excerpt: in legacy mode, `--emit-json` returns empty allowlists and empty `dir_prefixes`.
- Why this conflicts: If legacy packs are in scope for `from-impact-map`, empty allowlists imply “no derived inputs” unless a separate legacy-derived counting path exists; the seam pack does not state which behavior is intended.
- What must be true:
  - A concrete eligibility rule for pack-derived lift (strict/new-format only vs legacy-supported).
  - If legacy packs are supported, the exact data source/path for derived counts when validator output is empty.
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision

