# Planning Pre-Planning Research Wrapper Standard

This standard describes the FSE pre-planning orchestration workflow and the intended contracts between:
- the scripted orchestrator,
- the focused planning agents,
- and the wrapper or operator.

Canonical wrapper prompt:
- `docs/project_management/system/fse/prompts/planning/pre_planning_wrapper.md`

Canonical automation entrypoint:
- Start from ADR-only:
  - `make pm-fse-pre-planning-from-adr ADR="docs/project_management/adrs/draft/ADR-000X-<kebab-title>.md" [FEATURE="<feature>"] [BUCKET=draft]`
- Start from an existing pack:
  - `docs/project_management/system/fse/scripts/planning/pre_planning_research_orchestrate.sh --feature-dir "docs/project_management/packs/<bucket>/<feature>"`

## Goal

Produce the minimal, high-signal FSE pre-planning artifacts needed to ground downstream planning and decomposition:

Tracked under `<FEATURE_DIR>/pre-planning/`:
- `spec_manifest.md`
- `impact_map.md`
- `minimal_spec_draft.md`
- `ci_checkpoint_plan.md`
- `workstream_triage.md`
- `alignment_report.md`

Untracked evidence under `<FEATURE_DIR>/logs/`:
- per-step scratch, handoff, staged candidates, stderr, and wrapper summaries

Non-goals:
- creating `tasks.json`,
- creating `plan.md`,
- creating kickoff prompts,
- creating legacy execution ownership surfaces,
- creating or depending on pre-full-planning convergence outputs,
- creating or depending on post-full-planning reconcile outputs,
- creating or depending on legacy full-planning follow-up artifacts,
- authoring execution-valid task graphs.

## Preconditions

1) One of:
   - ADR-only start via `make pm-fse-pre-planning-from-adr ...`
   - existing feature pack under `docs/project_management/packs/<bucket>/<feature>/`

2) ADR inputs are resolvable for the feature.
   - The wrapper or operator must know which ADRs seed the run.
   - ADR resolution must not depend on `tasks.json`.

3) Orchestration runs from a clean orchestration checkout.
   - `git status --porcelain=v1` must be empty before starting the orchestrator.

## Stable step logs

Pre-planning research uses stable, pack-local step log directories under `<FEATURE_DIR>/logs/`:
- `spec-manifest/`
- `impact-map/`
- `min-spec-draft/`
- `CI-checkpoint/`
- `workstream-triage/`

Each step directory contains:
- `stderr.log`
- `codex.pid`
- `handoff.md`
- `last_message.md`

Rationale:
- `docs/project_management/packs/**/logs/` is gitignored.
- Stable sentinels let downstream steps self-gate without relying on latest-pointer files.

## Orchestration pattern

Pre-planning research is a 5-step chain:
1. `spec_manifest`
2. `impact_map`
3. `min_spec_draft`
4. `ci_checkpoint`
5. `workstream_triage`

Overlap model:
- The orchestrator starts steps in a staggered, overlapped manner, triggered by upstream `handoff.md`.
- Downstream agents may start early, but must:
  - write to their own step `logs/` only while upstream is still running,
  - delay tracked writes until the upstream step completes successfully.

Default poll interval for sentinel gating: 60 seconds.

Hard rules:
- Stop on failure. If any step fails, do not proceed to later steps.
- Keep reruns explicit and bounded via `START_AT=<step>`.
- The active lane ends at the tracked `pre-planning/` artifacts listed above.
- Legacy convergence or reconcile surfaces are compatibility scaffolding only and are not part of the supported `pm-fse-pre-planning-from-adr` contract.

## Reruns

When rerunning from a mid-chain step, existing stable step log dirs are archived by renaming:
- `<FEATURE_DIR>/logs/<step>/` → `<FEATURE_DIR>/logs/<step>_run_N/`

This prevents stale sentinels from being reused and preserves audit history.

## Validation guidance

During FSE pre-planning, prefer focused validation:
- `bash docs/project_management/system/fse/scripts/planning/micro_lint.sh --feature-dir "<FEATURE_DIR>" --agent <agent> -- <owned-paths...>`
- `python3 docs/project_management/system/fse/scripts/planning/validate_spec_manifest.py --feature-dir "<FEATURE_DIR>"`
- `python3 docs/project_management/system/fse/scripts/planning/validate_impact_map.py --feature-dir "<FEATURE_DIR>"`

If `ci_checkpoint_plan.md` is present, validate it against the FSE-native checkpoint contract rather than legacy task wiring.

Avoid assuming the broader legacy planning lint is green during pre-planning.

## Compatibility-only inactive surfaces

If the repo or a feature pack still contains legacy pre-full-planning convergence, post-full-planning reconcile, or other full-planning follow-up surfaces:
- treat them as compatibility scaffolding only,
- treat them as inactive for the active pre-planning lane,
- do not require them for `pm-fse-pre-planning-from-adr`,
- do not block pre-planning completion on their absence or staleness.

## Workstream sizing evidence

During FSE pre-planning, derive downstream sizing from the authored FSE artifacts themselves:
- `spec_manifest.md` for contract and surface breadth,
- `impact_map.md` for dependency density and boundary crossings,
- `minimal_spec_draft.md` for candidate ordering and split pressure,
- `ci_checkpoint_plan.md` for checkpoint grouping pressure.

Do not invoke legacy lift or intake commands during this lane:
- `make pm-lift-pack ...`
- `make pm-lift-intake ...`

If triage needs a durable evidence artifact, store it under:
- `<FEATURE_DIR>/logs/workstream-triage/planning_pressure_assessment.md`
