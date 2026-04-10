# FSE Planning / Research / Alignment Standard

Use this standard when the next unit of work is planning, research, documentation, or decomposition preparation rather than code execution.

Goal:
- produce an FSE-native pre-planning pack with zero ambiguity about scope, contract surfaces, downstream seams, and alignment with adjacent work.

## 1) When to use this standard

Use an FSE pre-planning pass when one or more are true:
- multiple subsystems interact,
- cross-platform parity matters,
- a stable user-facing contract must be defined,
- the repo already contains partial plans that need alignment,
- downstream decomposition needs a clean seam map before execution planning begins.

If you already have crisp downstream docs and only need execution-task authoring, use the downstream execution subsystem that owns that work instead of extending pre-planning.

## 2) Core principles

- Docs are the source of truth for planning.
- Zero ambiguity. Behavior statements must be singular and testable.
- Every major decision is explicit.
- Cross-pack alignment is required.
- Pre-planning stops before execution wiring.
- This lane must not require `tasks.json`, `plan.md`, kickoff prompts, or legacy execution ownership artifacts.
- This lane must not require pre-full-planning convergence, post-full-planning reconcile, or legacy full-planning follow-up artifacts.

## 3) Required outputs

When you run an FSE pre-planning pass for a feature, produce these artifacts under:
- `docs/project_management/packs/<bucket>/<feature>/pre-planning/`

Required:
1. `spec_manifest.md`
2. `impact_map.md`
3. `minimal_spec_draft.md`
4. `workstream_triage.md`
5. `alignment_report.md`

Conditional:
6. `ci_checkpoint_plan.md` when platform scope or verification cadence warrants explicit checkpoint intent

Supporting evidence lives under:
- `docs/project_management/packs/<bucket>/<feature>/logs/`

Compatibility-only inactive surfaces:
- legacy pre-full-planning convergence outputs,
- legacy post-full-planning reconcile outputs,
- legacy full-planning follow-up artifacts.

If these surfaces still exist in the repo or a feature pack, treat them as compatibility scaffolding outside the supported `pm-fse-pre-planning-from-adr` contract.

### 3.1 `spec_manifest.md`

Purpose:
- select required docs,
- assign authoritative ownership,
- distinguish pre-planning outputs from downstream deferred docs.

Standard:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

### 3.2 `impact_map.md`

Purpose:
- enumerate touched surfaces,
- document cascading implications and contradiction risks,
- align with queued work.

Standard:
- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`

### 3.3 `minimal_spec_draft.md`

Purpose:
- provide cross-cutting defaults, invariants, and a draft downstream seam skeleton.

Rules:
- it is pre-planning only,
- it is superseded later by downstream planning or decomposition,
- it does not create execution tasks.

### 3.4 `ci_checkpoint_plan.md`

Purpose:
- capture advisory checkpoint intent for later multi-platform verification.

Rules:
- it defines checkpoint groups, gates, and rationale,
- it does not define task IDs or execution wiring.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

### 3.5 `workstream_triage.md`

Purpose:
- propose downstream planning workstreams,
- recommend seam-oriented sequencing or restructuring,
- capture sequencing risks and unknowns.

Rules:
- it is advisory,
- it does not own execution artifacts,
- it does not become a legacy execution registry.

## 4) Alignment duties

Every pre-planning pass must align against:
- the ADR set,
- the current repo state,
- queued or draft ADRs,
- active, draft, queued, and archived planning packs that touch the same surfaces.

The result must make conflicts explicit and record resolutions or follow-ups.
This alignment duty is satisfied by the active pre-planning artifacts above, not by legacy convergence or reconcile surfaces.

## 5) Decision discipline

Pre-planning may capture explicit follow-ups and A/B resolution notes inside the authored docs.

Rules:
- no unresolved `TBD`, `TODO`, or `open question` placeholders,
- no vague “future work” language without a concrete follow-up,
- no multiple contradictory contract options left implicit.

## 6) Lint-like rules

Run ambiguity and hard-ban scans over the authored pre-planning outputs.

Forbidden in authored pre-planning docs:
- `TBD`, `TODO`, `WIP`, `TBA`
- `open question`
- `etc.`
- `should`, `could`, `might`, `maybe` in behavior or contract statements

Encouraged checks:
```bash
rg -n "\\b(TBD|TODO|WIP|TBA)\\b" "$FEATURE_DIR/pre-planning"
rg -n "open question" "$FEATURE_DIR/pre-planning"
rg -n "\\betc\\.\\b|and so on" "$FEATURE_DIR/pre-planning"
rg -n "\\b(should|could|might|maybe|optionally)\\b" "$FEATURE_DIR/pre-planning"
```

## 7) Ready for downstream planning or decomposition

The FSE pre-planning pack is ready to hand off when all are true:
- `spec_manifest.md` defines required docs and ownership with no implicit surfaces,
- `impact_map.md` defines explicit touched surfaces and conflict resolutions,
- `minimal_spec_draft.md` defines cross-cutting invariants and a usable draft downstream seam skeleton,
- `ci_checkpoint_plan.md` exists when needed and expresses checkpoint intent without task wiring,
- `workstream_triage.md` proposes actionable downstream workstreams without legacy execution-registry semantics,
- `alignment_report.md` summarizes remaining follow-ups and hard gates,
- no compatibility-only convergence, reconcile, or legacy full-planning follow-up artifact is required for readiness,
- ambiguity and hard-ban scans pass for the pre-planning outputs.
