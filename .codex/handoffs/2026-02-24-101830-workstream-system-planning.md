# Handoff: Workstream system planning (Pass A triage + Pass B refinement)

## Session Metadata
- Created: 2026-02-24 10:18:30
- Project: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
- Branch: ops/work_lift_v1_seams
- Session duration: ~2–3 hours

### Recent Commits (for context)
  - 3d302a3c Review Work Lift doc gaps
  - c47238d3 Review docs for Work Lift updates
  - 6d591f06 Add advisory lift docs and Makefile
  - eef62cd9 Add SEAM-3 goldens tests
  - db87451b Implement seam-3 goldens conformance

## Handoff Chain

- **Continues from**: [2026-02-23-154924-work-lift-v1-doc-sync.md](./2026-02-23-154924-work-lift-v1-doc-sync.md)
  - Previous title: Work Lift v1 doc sync + prompts/templates updated to use new commands
- **Supersedes**: None

> Review the previous handoff for full context before filling this one.

## Current State Summary

We finished tightening Work Lift v1 docs/prompts/templates earlier, and shifted to *planning* the missing “Workstream Triage + Refinement” workflow (D1 in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`). A repo-root implementation plan now exists (uncommitted) describing canonical registries (`docs/project_management/workstreams/`, `docs/project_management/work_items/`), and an Option A lifecycle: discovery/lockdown does **not** create workstreams; instead a dedicated **Pass A Workstream Triage** session runs post-lockdown and pre-planning kickoff, followed by **Pass B Workstream Refinement** after planning-lint is green. Important: next work should remain **planning/docs/spec** work (do not jump into coding/scaffolding yet).

## Codebase Understanding

### Architecture Overview

- The PM system already supports optional pack-level linkage to workstreams/work items via `tasks.json meta.workstream_id` and `meta.work_item_refs`.
- Strict packs (`meta.slice_spec_version >= 2`) already perform existence checks for referenced WS/WI records via `validate_tasks_json.py`, using default registry roots `docs/project_management/workstreams/` and `docs/project_management/work_items/` (overridable via `PM_WORKSTREAMS_ROOT` / `PM_WORK_ITEMS_ROOT`).
- Work Lift v1 is already implemented and is the sizing signal used by triage/refinement:
  - Discovery-time: `make pm-lift-intake FILE=...`
  - Planning-time (strict packs only meaningful): `make pm-lift-pack PACK=...`
  - Optional: `PM_LIFT_ADVISORY=1 make planning-lint FEATURE_DIR=...` (advisory output)

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md | Root decision log | Defines D1 (Pass A/Pass B), v1/v2 boundaries, workstream semantics |
| WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md | Repo-root implementation plan (draft) | Captures Option A timing + “one record per WSID/WIID” + required prompts/validator changes |
| docs/project_management/system/scripts/planning/validate_tasks_json.py | Pack validator | Already validates WS/WI formats and (strict packs) existence; must be updated to require exactly one match per ID |
| docs/project_management/system/scripts/planning/pm_paths.py | Registry root resolver | Documents `PM_WORKSTREAMS_ROOT` / `PM_WORK_ITEMS_ROOT` behavior; helps align docs and tooling |
| docs/project_management/system/standards/planning/PLANNING_WORKFLOW_OVERVIEW.md | Workflow diagram | Needs to include Pass A triage + Pass B refinement steps (planning work item) |
| docs/project_management/system/USER_GUIDE.md | Operator overview | References canonical registries; needs “intake vs canonical record” clarity during rollout |
| docs/project_management/system/standards/planning/PLANNING_README.md | Planning entrypoint | References registries + strict-pack behavior; needs Option A timing notes |

### Key Patterns Discovered

- “Strict gating by `meta.slice_spec_version >= 2`” is the compatibility mechanism for enforcing stronger invariants without breaking legacy packs.
- Registry roots are already part of the PM env contract in code (`pm_paths.py`, validator), but are not consistently documented in the PM system README.
- The project prefers deterministic, tool-computed outputs (e.g., Work Lift via Make targets) over hand-written calculations.

## Work Completed

### Tasks Finished

- [x] Created a repo-root plan for implementing workstreams/work-items registries and D1 Pass A/Pass B lifecycle.
- [x] Updated that plan to adopt Option A timing: Pass A post-lockdown, pre-planning kickoff; Pass B post planning-lint.
- [x] Recorded the key requirement for strict packs: exactly one on-disk record per referenced WSID/WIID.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md | New draft plan (currently untracked) | Single source of truth for the planned workflow + required changes |
| .codex/handoffs/2026-02-24-101830-workstream-system-planning.md | New handoff (currently untracked) | Preserve context + ensure next agent continues planning (no coding yet) |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Option A lifecycle timing | Create WS during discovery vs post-lockdown triage | Keeps discovery focused on shaping ADR/WI; triage belongs right before planning kickoff when details are more concrete |
| Keep `snake_case` in IDs | kebab-case vs snake_case | Already matches the validator regex (`WS-YYYYMM-[a-z0-9_]+`) |
| Keep WI intake; remove WS intake | Use intake dirs for both vs only WI | Workstreams are canonical umbrellas; WIs still benefit from draft intake forms |
| Strict packs require exactly one record per ID | allow multiple `WSID-*.md` matches | Prevent ambiguity and make `meta.workstream_id` meaningfully resolvable |

## Pending Work

### Immediate Next Steps

1. Continue planning (docs/spec only): refine `WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md` into an implementation-ready checklist (no coding yet).
2. Specify the two new “one-session” system prompts (docs-only specs): `workstream_triage.md` (Pass A) and `workstream_refinement.md` (Pass B) including exact inputs/outputs and required command blocks.
3. Identify the exact PM system docs that must be updated to remove friction (workflow overview diagram, discovery coach prompt, system README env roots, schema descriptions).

### Blockers/Open Questions

- None that block planning, but open questions to resolve before coding:
  - Pass A stability: what qualifies a workstream assignment as “locked” vs “provisional”?
  - Heuristics for “candidate workstreams that fit”: what evidence is required (touch-surface overlap, sequencing overlap, operator intent)?
  - Migration: do any existing non-README docs exist under `docs/project_management/intake/workstreams/` that require migration?

### Deferred Items

- Implementing scripts/Make targets and validator enforcement should wait until planning is finalized and the prompts/record formats are agreed.

## Context for Resuming Agent

### Important Context

- Continue **planning/docs/spec** work; do **not** jump into implementation/scaffolding yet.
- Workstreams are **umbrella initiatives** and should not share names with any single contained pack/WI/ADR.
- Option A timing is now the intended lifecycle:
  - Pass A Workstream Triage happens post-lockdown, pre-planning kickoff (select/propose WSID; record coarse lift evidence via `pm-lift-intake`).
  - Pass B Workstream Refinement happens post planning-lint (pack-derived evidence; set `tasks.json meta.workstream_id` once locked).
- Strict packs must resolve exactly one record per WSID/WIID (validator change required before enforcement is real).
- Canonical registries are intended to live at `docs/project_management/workstreams/` and `docs/project_management/work_items/`, but these dirs do not exist yet.

### Assumptions Made

- Workstream IDs and work item IDs remain `snake_case` after the `WS-YYYYMM-` / `WI-YYYYMM-` prefix.
- `tasks.json meta.workstream_id` stays optional for now (may be `null`).
- Work Lift v1 remains stable/immutable; workstream triage uses existing `pm-lift-*` evidence rather than introducing new scoring.

### Potential Gotchas

- Discovery prompts currently still require creating a workstream “intake” file under `docs/project_management/intake/workstreams/` (Option A says this should be removed; plan captures the change needed).
- Strict-pack validator currently permits multiple `WSID-*.md` matches; once we enforce “exactly one match”, any duplicate records will break strict pack validation.
- Planning Pack-derived lift is meaningful only for strict packs (`meta.slice_spec_version >= 2`); legacy packs intentionally emit empty allowlists under `validate_impact_map.py --emit-json`.

## Environment State

### Tools/Services Used

- `git`, `rg`, `sed`, `python3`

### Active Processes

- None.

### Environment Variables

- `PM_ROOT`, `PM_SYSTEM_ROOT`, `PM_ADRS_ROOT`, `PM_PACKS_ROOT`
- `PM_WORKSTREAMS_ROOT`, `PM_WORK_ITEMS_ROOT`
- `PM_DEFAULT_PACK_BUCKET`
- `PM_LIFT_ADVISORY`, `PM_LIFT_STRICT`

## Related Resources

- `WORKSTREAM_SYSTEM_IMPLEMENTATION_PLAN.md`
- `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`
- Prior handoff: `.codex/handoffs/2026-02-23-154924-work-lift-v1-doc-sync.md`

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
