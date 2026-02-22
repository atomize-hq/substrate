# Workstream Triage + Lift Model — Decision Log

Last updated (UTC): 2026-02-22

This file lives at the **repo root on purpose** so we can track evolving workflow decisions without losing context or scattering notes across the project management system.

## Goals

- Maintain a **time-free** sizing and split heuristic that maps to our execution primitives (ADRs, slices, checkpoints), not wall-clock.
- Make workstream grouping stable: workstreams are **umbrella initiatives** spanning multiple ADRs/packs/work items (not tied 1:1 to an ADR codename).
- Enable incremental adoption with minimal breakage:
  - Advisory-first where possible
  - Strict enforcement gated by Planning Pack strict mode (`meta.slice_spec_version >= 2`)

## Terminology

- **ADR**: decision record; should represent **one behavior delta**.
- **Slice**: execution unit; each slice typically maps to a triad (code/test/integ).
- **Workstream**: umbrella initiative grouping multiple ADRs, packs, and work items.
- **Work Item (WI)**: execution/backlog unit that is not an ADR (implementation/cleanup/follow-up).
- **Lift Vector / Lift Score**: deterministic, time-free sizing signal computed from structured inputs.

## Decisions

### D1 — Workstream lifecycle is two-pass

Date: 2026-02-22

- We adopt **Workstream Triage + Workstream Refinement** as two passes:
  - **Pass A (Workstream Triage)** happens when an ADR is accepted (or queued). It groups the ADR into an umbrella workstream and records coarse size/risk signals.
  - **Pass B (Workstream Refinement)** happens after the Planning Pack is created and planning-lint is green. It replaces coarse estimates with pack-derived counts (actual slices/checkpoints/touch-set).

Rationale:
- ADRs usually arrive one at a time.
- Exact slice counts aren’t fully known until pack planning is done, but ADRs still need grouping and rough sizing.

### D2 — Use a deterministic, time-free Lift model (v1)

Date: 2026-02-22

- We drop “hours/days” from sizing inputs and instead use:
  - **Lift Vector** (counts + booleans)
  - **Lift Score** (deterministic formula)
  - **Split Triggers** (hard thresholds that force decomposition)
- Mapping is expressed in execution units:
  - Lift → estimated slices
  - Slices → checkpoint group count (4–8 slices per checkpoint as a default planning guideline)

### D3 — Discovery-time Lift Vector storage: JSON block + frontmatter summary

Date: 2026-02-22

- Discovery-time Lift Vector lives as an **embedded JSON block** (comment-fenced) inside intake/ADR markdown so it is deterministic to parse and works in either file type.
- A small **frontmatter summary** stores computed outputs for quick scanning (and can be updated by scripts):
  - `lift_model_version`
  - `lift_score`
  - `lift_estimated_slices`
  - `lift_confidence` (`low|medium|high`)
  - (optional) `lift_split_triggers` (array of strings)

Example (frontmatter summary):
```yaml
lift_model_version: 1
lift_score: 18
lift_estimated_slices: 2
lift_confidence: low
lift_split_triggers: []
```

Example (embedded JSON block):
````md
<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 0,
    "edit_files": 0,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": null,
    "boundary_crossings": null
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": ""
}
```
<!-- PM_LIFT_VECTOR:END -->
````

Notes:
- `null` is allowed for counts we can’t estimate deterministically at discovery-time; the script should reflect that in confidence/scoring behavior.
- The script (not the LLM) should compute `lift_score`, `lift_estimated_slices`, and split trigger flags when possible.

### D4 — Strict enforcement is gated by Planning Pack strict mode

Date: 2026-02-22

- Strict enforcement gates on `tasks.json meta.slice_spec_version >= 2`.
- Legacy packs remain compatible.
- Initial rollout should be advisory-first, then enforce only the most reliable invariants.

### D5 — Workstream IDs are initiative-themed (not ADR-codename themed)

Date: 2026-02-22

- Canonical Workstream registry IDs use: `WS-YYYYMM-<initiative_slug>`
- Workstreams are umbrellas spanning multiple ADRs/packs/WIs.
- Any “WS-<adr_codename>” values that appear in intake batching artifacts should be treated as **intake-local grouping tags**, not canonical workstream IDs, unless explicitly migrated.

## Open Questions / TODOs

- Define the canonical **Lift rubric doc** location and versioning scheme (root vs system docs).
- Define the deterministic formula (weights) as a config file or as a versioned constant.
- Decide how to handle `null` fields in scoring (skip, default, or degrade confidence).
- Decide which invariants become hard gates first (likely: `behavior_deltas == 1`, `estimated_slices <= 3` for a single ADR candidate).
- Decide whether to backfill Lift summaries for existing queued/active items.
