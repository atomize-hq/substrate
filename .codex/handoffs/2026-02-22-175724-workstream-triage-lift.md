# Handoff: Workstream Triage + Work Lift (v1)

## Session Metadata
- Created: 2026-02-22 17:57:24
- Project: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- Branch: testing
- Session duration: ~2–4 hours (estimate)

### Recent Commits (for context)
  - 271db259 Document Lift decision sections
  - 6c988081 Plan workstream grouping agent
  - 1bdffef7 chore: path file name replacement
  - 8d560996 Add ADR guardrail for legacy paths
  - 640a0d84 Plan post-ADR workstream capacity

## Handoff Chain

- **Continues from**: None (fresh start)
- **Supersedes**: None

> This is the first handoff for this task.

## Current State Summary

We converged on a two-pass “Workstream Triage + Workstream Refinement” model and a time-free sizing heuristic (“Work Lift”) that maps to execution primitives (ADRs, slices, checkpoints), not hours. The canonical decision log is at `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` and now includes D1–D10 (Lift Vector v1 fields, Lift Score v1 formula, split triggers, Lift→slices/checkpoints mapping, directory-prefix expansion discount/cap, and the intended rubric artifact locations under `docs/project_management/system/*`). An initial `pm_lift.py` sketch exists and can compute lift from a pack’s `impact_map.md` touch set (with deterministic directory expansion) and from an intake/ADR JSON block.

## Codebase Understanding

### Architecture Overview

- Impact Map Touch Set strict validation is already a central enforcement primitive: strict packs (`tasks.json meta.slice_spec_version >= 2`) require explicit repo-relative tokens under `impact_map.md`, and `task_finish.sh` consumes `validate_impact_map.py --emit-json`.
- Directory/prefix tokens (ending with `/`) are first-class in the Touch Set model today; they flow through `validate_impact_map.py` → `dir_prefixes` → `task_finish` enforcement. Lift computation needs to treat prefixes carefully to avoid swamping scores.
- Work Lift is meant to be deterministic and tunable; the long-term plan is to move weights/triggers to machine-readable config and keep LLMs in “fill vector only” mode.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md | Canonical decision log for workstream triage + lift | Source of truth for v1 model + next steps |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pm_lift.py | Initial lift estimator sketch/tool | Starting point for implementation and integration |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/validate_impact_map.py | Strict Touch Set validator | pm_lift uses `--emit-json`; task_finish uses it for enforcement |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md | Defines Touch Set rules and strict gating | Dictates what impact_map authors may/should do |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/triad/task_finish.sh | Enforces touch-set allowlist at finish time | Downstream consumer; avoid breaking semantics |

### Key Patterns Discovered

- “Strict gating by `meta.slice_spec_version >= 2`” is the established compat mechanism for PM enforcement: legacy packs must not break; strict packs can require stronger invariants.
- Validators are designed to emit actionable remediation errors and (where needed) `--emit-json` outputs for downstream enforcement.

## Work Completed

### Tasks Finished

- [x] Documented Lift v1 core mechanics (vector, formula, split triggers, mapping) and directory-prefix expansion policy in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`.
- [x] Added a working sketch for `pm_lift.py` capable of:
  - `from-impact-map` (Touch Set → lift, with per-prefix discounted expansion)
  - `from-intake` (comment-fenced JSON block → lift)
  - `from-git-diff` (diff-derived touch counts → lift; calibration scaffold)

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md | Added D6–D10 (full v1 mechanics + artifact plan + pm_lift sketch) | Preserve canonical decisions and avoid context loss |
| /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pm_lift.py | Added initial lift estimator tool | Start deterministic computation and future automation |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Two-pass workstream management | Single-pass vs two-pass | ADRs arrive incrementally; slice counts become firm after pack lint |
| Time-free “Work Lift” model | Hours/days estimates | Agentic parallelism makes time non-deterministic; lift maps to slices/checkpoints |
| Discovery-time storage | YAML-only vs JSON block vs JSON+summary | JSON is deterministic; frontmatter summary is scannable |
| Strict enforcement gating | Always-on vs `slice_spec_version>=2` | Preserve legacy packs; enable stronger rules for strict packs |
| Directory prefixes in Touch Set | Disallow vs allow+constrain vs expand | Touch-set enforcement already supports prefixes; lift uses discounted/capped expansion per prefix |

## Pending Work

### Immediate Next Steps

1. Read `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` end-to-end and confirm the v1 model is acceptable as “canonical until revised”.
2. Decide D10 implementation detail: create the rubric artifacts under `docs/project_management/system/*` (schema + model config + human rubric) and align `pm_lift.py` to read them.
3. Plan integration as advisory-only first (make targets / optional lint output), and explicitly avoid “just start executing” at session start — begin with a short next-steps proposal and confirmation.

### Blockers/Open Questions

- [ ] Define a subsystem taxonomy if we want `touch.boundary_crossings` to be filled deterministically and consistently.
- [ ] Decide how `pm_lift.py` should update/maintain the frontmatter summary fields (idempotent rewrite rules).
- [ ] Decide how/when to promote split triggers from advisory to enforced (and where enforcement lives: lint vs separate validator).

### Deferred Items

- Calibration loop (pm_lift_calibrate) deferred until we have enough completed packs with reliable metadata to tune weights.

## Context for Resuming Agent

### Important Context

- Do not jump straight into implementing enforcement next session. First: propose a short plan (rubric artifacts + schema/config + minimal make target integration) and get confirmation.
- Directory prefixes are already an accepted Touch Set primitive (enforced by `validate_impact_map.py` and consumed by `task_finish.sh`). Lift must treat them carefully, hence the per-prefix discounted/capped expansion rule:
  - `EXPAND_DISCOUNT=0.20`, `EXPAND_CAP=10` per prefix entry (max 2.0 effective “files” per prefix).
- Null scoring rule is deliberate: treat `null` as 0 for computation, but force `lift_confidence=low` and emit missing-input triggers so we don’t invent precision.
- The approved rubric artifact locations (for DR-0010 context) are recorded in D10 inside `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`:
  - `docs/project_management/system/schemas/work_lift_vector.schema.json`
  - `docs/project_management/system/schemas/work_lift_model.v1.json`
  - `docs/project_management/system/standards/shared/WORK_LIFT_RUBRIC.md`

### Assumptions Made

- Using `meta.slice_spec_version >= 2` as the main strict gating flag remains acceptable for Lift-related enforcement (aligns with existing strict/legacy patterns).
- Directory-prefix expansion uses current HEAD (`git ls-files`) for lift estimation only; it does not attempt to predict future touched files.

### Potential Gotchas

- `validate_impact_map.py --emit-json` emits JSON on stdout and warnings/errors on stderr; callers must not parse stderr.
- Touch Set strict mode forbids globs and requires directory tokens to end with `/`; edit/deprecate/delete entries must exist on disk.
- The current `pm_lift.py` is a sketch: it can compute from impact_map touch sets and intake JSON blocks, but it does not yet update frontmatter or load the model config (it tries, but config files don’t exist yet).

## Environment State

### Tools/Services Used

- `python3`, `jq`, `git`, `rg`
- PM validators under `docs/project_management/system/scripts/planning/`

### Active Processes

- None

### Environment Variables

- `PM_ROOT`, `PM_ADRS_ROOT`, `PM_PACKS_ROOT`, `PM_WORKSTREAMS_ROOT`, `PM_WORK_ITEMS_ROOT` (optional; used by PM tooling)

## Related Resources

- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/pm_lift.py
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/scripts/planning/validate_impact_map.py
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
