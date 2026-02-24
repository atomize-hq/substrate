# Handoff: Work Lift v1 doc sync + prompts/templates updated to use new commands

## Session Metadata
- Created: 2026-02-23 15:49:24
- Project: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- Branch: ops/work_lift_v1_seams
- Session duration: ~1–2 hours

### Recent Commits (for context)
  - 6d591f06 Add advisory lift docs and Makefile
  - eef62cd9 Add SEAM-3 goldens tests
  - db87451b Implement seam-3 goldens conformance
  - a24bf584 Verify orthogonal goals alignment
  - cb426f1e Audit and fix work lift docs

## Handoff Chain

- **Continues from**: [2026-02-22-175724-workstream-triage-lift.md](./2026-02-22-175724-workstream-triage-lift.md)
  - Previous title: Workstream Triage + Work Lift (v1)
- **Supersedes**: [list any older handoffs this replaces, or "None"]

> Review the previous handoff for full context before filling this one.

## Current State Summary

This session focused on closing “spec drift” between the root decision log (`WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`) and the shipped Work Lift v1 implementation/contracts under `docs/project_management/system/*`, then propagating the new Work Lift Makefile commands into the discovery prompts and PM system templates. The decision log now reflects v1 reality (confidence enum, null handling, blowup clamp, v1 vs v2 triggers), and all remaining aspirational items are explicitly listed as v2 candidates. Discovery prompts and Planning Pack templates now instruct agents/authors to compute lift via `make pm-lift-*` targets (and optionally `PM_LIFT_ADVISORY=1 make planning-lint ...`) instead of hand-calculating.

## Codebase Understanding

### Architecture Overview

- Work Lift v1 is defined by a small set of explicit contracts:
  - Lift Vector schema (CONTRACT-1): `docs/project_management/system/schemas/work_lift_vector.schema.json`
  - Lift model config (CONTRACT-2): `docs/project_management/system/schemas/work_lift_model.v1.json`
  - `pm_lift --emit-json` output contract (CONTRACT-3): `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`
  - `validate_impact_map --emit-json` output contract (CONTRACT-4): `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md`
- `pm_lift.py` is the reference tool; Makefile targets wrap it for consistent invocation:
  - `make pm-lift-intake FILE=...`
  - `make pm-lift-pack PACK=...`
  - `make pm-lift-diff BASE=... HEAD=...`
  - `make pm-lift-strict FILE=...` or `PACK=...` (opt-in strict wrapper; sets `PM_LIFT_STRICT=1`)
- Pack-derived lift is meaningful only for strict packs (`tasks.json.meta.slice_spec_version >= 2`) because legacy packs intentionally emit empty allowlists under `validate_impact_map.py --emit-json`.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md | Root decision log | Updated to match shipped v1 and mark v2 candidates |
| WORK_LIFT_V1_SPEC_DRIFT.md | Drift report (decision log vs implementation) | Catalog of v1 drift and rationale |
| docs/project_management/system/scripts/planning/pm_lift.py | Reference implementation | Source for actual scoring/emit-json behavior |
| docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md | CONTRACT-3 spec | Defines machine keys/types/semantics |
| Makefile | Make targets for lift | `pm-lift-*` command entry points |
| docs/project_management/system/prompts/discovery/*.md | Discovery prompts | Updated to use `make pm-lift-*` commands |
| docs/project_management/system/templates/planning_pack/* | Planning templates | Updated to include `make pm-lift-pack` + advisory lint |

### Key Patterns Discovered

- Prefer pointing humans/agents at Make targets over raw script invocations (reduces drift).
- “Strict gating by `meta.slice_spec_version >= 2`” remains the main compatibility mechanism for Planning Pack enforcement/derivations.
- When documenting behavior, align with the pinned contracts (CONTRACT-1..4) and keep v1 immutable; new semantics go in v2.

## Work Completed

### Tasks Finished

- [x] Wrote a consolidated spec-drift report between decision log and shipped v1 implementation.
- [x] Updated the root decision log to match shipped v1 behavior and explicitly list remaining items as v2 candidates.
- [x] Updated discovery prompts to compute Work Lift via `make pm-lift-*` commands (plus optional strict checks).
- [x] Updated Planning Pack templates to include Work Lift evidence and `PM_LIFT_ADVISORY=1 make planning-lint ...`.
- [x] Added Work Lift as a preflight requirement in the execution preflight report template.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| WORK_LIFT_V1_SPEC_DRIFT.md | Added drift index + per-drift evidence + resolution options | Make drift explicit and reviewable |
| WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md | Updated to match shipped v1 (confidence enum, null handling, clamp, v1 vs v2 triggers); added v2 candidates section | Make decision log non-misleading and aligned with contracts |
| docs/project_management/system/prompts/discovery/brainstorm_to_adr.md | Replaced hand-calculated lift guidance with `make pm-lift-intake` / optional `make pm-lift-strict`; linked rubric path | Ensure discovery uses real tooling + pinned semantics |
| docs/project_management/system/prompts/discovery/feature_discovery_coach.md | Same as above + apply to candidate ADRs/WIs | Keep feature discovery outputs consistent with v1 |
| docs/project_management/system/prompts/discovery/adr_lockdown.md | Added `pm-lift-intake`, `pm-lift-pack`, `pm-lift-strict`, and lint/advisory commands | Make lockdown workflow runnable + evidence-driven |
| docs/project_management/system/templates/planning_pack/plan.md.tmpl | Added Work Lift advisory section with `make pm-lift-pack` and `PM_LIFT_ADVISORY=1 make planning-lint ...` | Make lift visible in pack runbook |
| docs/project_management/system/templates/planning_pack/session_log.md.tmpl | Added planned commands for lift + advisory lint | Encourage consistent evidence capture |
| docs/project_management/system/templates/planning_pack/impact_map.md.tmpl | Added note to compute pack-derived lift via `make pm-lift-pack` | Connect touch-set authoring to lift computation |
| docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md | Added recommended Work Lift evidence commands (strict packs) | Make lift part of gate evidence (when eligible) |
| docs/project_management/system/templates/planning_pack/PLANNING_SESSION_LOG_TEMPLATE.md | Added rule to record lift commands + exit codes | Enforce traceability in logs |
| docs/project_management/system/templates/planning_pack/execution_preflight_report.md.tmpl | Added preflight checklist item + section for Work Lift evidence | Ensure lift computed before execution begins |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Keep implementation behavior; update decision log + docs to match | Change tooling vs fix docs | Tooling is already pinned/contracted; decision log should reflect reality |
| Mark “medium confidence”, “narrow prefix exception”, aggregation triggers, and frontmatter writer as v2 | Ship in v1 vs defer to v2 | These require new semantics/taxonomy/aggregation and would break v1 immutability |
| Prefer Make targets in prompts/templates | Raw script commands vs Make wrappers | Reduces command drift and standardizes usage |

## Pending Work

### Immediate Next Steps

1. If desired, sweep remaining PM templates/docs for hand-calculated lift references and replace with `make pm-lift-*` (use `rg -n "Lift Score|ceil\\(score/12\\)|pm_lift" docs/project_management/system`).
2. Run a quick “docs correctness” smoke locally: ensure `make pm-lift-intake`, `make pm-lift-pack`, and `make pm-lift-strict` recipes in updated docs match current Makefile flags.
3. Optionally add/confirm a short section in a central planning README that points to `pm-lift-*` targets (only if you want a single entrypoint beyond prompts/templates).

### Blockers/Open Questions

- None.

### Deferred Items

- V2 candidates (explicitly listed in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`):
  - `confidence = medium`
  - “narrow prefix” exception
  - workstream aggregation triggers (`estimated_total_slices > 8`, `touch.boundary_crossings > 4`)
  - idempotent frontmatter summary writer

## Context for Resuming Agent

### Important Context

- Work Lift v1 is “done” from a contract perspective; avoid changing v1 semantics in config/docs—introduce v2 instead.
- Pack-derived lift is meaningful only for strict packs (`meta.slice_spec_version >= 2`); legacy packs’ `validate_impact_map.py --emit-json` intentionally emits empty allowlists.
- Prompts/templates should reference Make targets (preferred) and the CONTRACT-3 output keys: `lift_score`, `estimated_slices`, `confidence`, `missing_inputs`, `triggers`.

### Assumptions Made

- The Makefile `pm-lift-*` targets are the canonical developer-facing interface for invoking lift during discovery/planning.
- Strict mode remains opt-in (`PM_LIFT_STRICT=1`), and preflight/gate templates should request lift evidence only when packs are strict/eligible.

### Potential Gotchas

- The discovery prompts now assume you can run Make targets from repo root; agents running elsewhere may need explicit reminder to be in a git checkout at repo root.
- `pm-lift-pack` depends on `validate_impact_map.py --emit-json`; for legacy packs this will not reflect the authored touch set.
- Intake files must contain the `PM_LIFT_VECTOR` markers and a fenced `json` block; otherwise `pm-lift-intake` fails.

## Environment State

### Tools/Services Used

- `git`, `rg`, `python3`, `make`

### Active Processes

- None.

### Environment Variables

- `PM_LIFT_ADVISORY` (optional; enables advisory report hook during `make planning-lint`)
- `PM_LIFT_STRICT` (opt-in; strict checker wrapper)
- Make vars used by docs/recipes: `FEATURE_DIR`, `PACK`, `FILE`, `BASE`, `HEAD`, `EMIT_JSON`

## Related Resources

- `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`
- `WORK_LIFT_V1_SPEC_DRIFT.md`
- `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`
- `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_STRICT_MODE.md`

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
