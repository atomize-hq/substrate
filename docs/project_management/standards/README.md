# Project Management Standards (How to Use This System)

This directory defines the **docs-first planning + triad execution** workflow used in this repo.

The system is designed for:
- zero ambiguity (no TBD/TODO/WIP in execution-critical docs),
- auditable decisions (ADRs + executive summaries),
- reproducible execution (standard tasks.json shape + automation scripts),
- strict role boundaries (code vs test vs integration),
- cross-platform parity via self-hosted CI smoke when needed.

## Two Phases

1) **Planning (docs-only)**
- Output is a Planning Pack under `docs/project_management/next/<feature>/`.
- A separate quality gate must approve the pack before execution begins.

2) **Execution (triads)**
- Work is executed as triads (code/test/integration) using worktrees.
- Docs edits happen only on the orchestration branch.

## Start Here (Entry Points)

**Planning**
- `docs/project_management/standards/PLANNING_README.md`
- `docs/project_management/standards/PLANNING_WORKFLOW_OVERVIEW.md`
- `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`

**Execution**
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

**Tooling and schema**
- `docs/project_management/standards/tasks.schema.json`
- `scripts/planning/validate_tasks_json.py`
- `scripts/planning/new_feature.sh`
- `scripts/planning/new_feature.ps1`
- `scripts/planning/archive_project_management_dir.py` (archive a Planning Pack into `docs/project_management/_archived/` and rewrite internal links)
- `Makefile`

## Quickstart (Recommended Workflow)

### 0) Scaffold a new Planning Pack

Scaffold (bash):
- `make planning-new-feature FEATURE="<feature>" AUTOMATION=1`

Scaffold (PowerShell):
- `make planning-new-feature-ps FEATURE="<feature>" AUTOMATION=1`

Cross-platform scaffolding (P3-008; optional knobs):
- Default cross-platform: `make planning-new-feature FEATURE="<feature>" CROSS_PLATFORM=1 AUTOMATION=1`
- Override scopes explicitly:
  - `make planning-new-feature FEATURE="<feature>" CROSS_PLATFORM=1 AUTOMATION=1 BEHAVIOR_PLATFORMS=linux CI_PARITY_PLATFORMS=linux,macos,windows`

This creates:
- `docs/project_management/next/<feature>/plan.md`
- `docs/project_management/next/<feature>/tasks.json`
- `docs/project_management/next/<feature>/session_log.md`
- `docs/project_management/next/<feature>/kickoff_prompts/*`
- execution gates (when enabled by scaffolder): `execution_preflight_report.md`, `C0-closeout_report.md`
- cross-platform smoke scaffolds if requested

### 1) Write the planning docs (planning agent work)

Fill/update:
- specs (`*-spec*.md`) with explicit scope/acceptance/out-of-scope,
- `tasks.json` with explicit start/end checklists and dependencies,
- kickoff prompts (`kickoff_prompts/*.md`) with role boundaries and required commands.

Validate:
- `make planning-lint FEATURE_DIR="docs/project_management/next/<feature>"`
- `make planning-validate FEATURE_DIR="docs/project_management/next/<feature>"`

### 2) Quality gate (required before execution)

Create/approve:
- `docs/project_management/next/<feature>/quality_gate_report.md` containing `RECOMMENDATION: ACCEPT`

Execution triads must not begin until the recommendation is `ACCEPT`.

### 3) Feature start gate (execution preflight)

Complete the feature-level start gate:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- `docs/project_management/next/<feature>/execution_preflight_report.md`

### 4) Execute slices as triads (automation-enabled packs)

General rules:
- Docs edits happen only on the orchestration branch.
- Worktrees are retained for the duration of the feature and removed only by feature cleanup.

Orchestration branch bootstrap (used by the opening gate):
- `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/<feature>"`

#### Code + test (always parallel)

Start both worktrees:
- Preferred (post-preflight): use `docs/project_management/standards/TRIAD_WRAPPER_PROMPT.md` (runs start-pair with `LAUNCH_CODEX=1` and reports exit codes + last messages + artifact paths).
- `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="C0" LAUNCH_CODEX=1`

Finish each task from inside its worktree (commits to the task branch; does not merge to orchestration):
- `make triad-task-finish TASK_ID="C0-code"`
- `make triad-task-finish TASK_ID="C0-test"`

#### Integration (single merge-back point)

Integration tasks should set `merge_to_orchestration` in `tasks.json`:
- platform-fix integration tasks: `false` (never merge back to orchestration)
- final aggregator integration task: `true` (the only task that merges back to orchestration)

Start integration worktree:
- `make triad-task-start FEATURE_DIR="docs/project_management/next/<feature>" TASK_ID="C0-integ-core"`

Optional: run an end-to-end integration orchestration wrapper (integ-core -> smoke -> platform-fix -> final) with artifact reporting:
- `docs/project_management/standards/TRIAD_INTEGRATION_WRAPPER_PROMPT.md`

Dispatch cross-platform smoke:
- If behavior platforms are exactly `linux,macos,windows`: `make feature-smoke FEATURE_DIR="docs/project_management/next/<feature>" PLATFORM=all WORKFLOW_REF="feat/<feature>"`
- Otherwise, dispatch per platform (repeat): `make feature-smoke FEATURE_DIR="docs/project_management/next/<feature>" PLATFORM=<platform> WORKFLOW_REF="feat/<feature>"`
- Add WSL coverage when required: `RUN_WSL=1` (Linux smoke, or `PLATFORM=wsl` when `wsl_task_mode="separate"`)

If smoke fails, start only the failing platform-fix tasks:
- Single smoke run id case (`PLATFORM=all`): `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="C0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
- Multi-run case (per-platform smoke): `make triad-task-start-platform-fixes FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="C0" PLATFORMS="<csv>" LAUNCH_CODEX=1`

After all failing platform-fix tasks are green, start the final aggregator:
- `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="C0" LAUNCH_CODEX=1`

Finish integration from inside the worktree:
- `make triad-task-finish TASK_ID="C0-integ"`

Guardrail: the finisher will only merge back to orchestration when the task has `merge_to_orchestration=true`.

### 5) Feature end cleanup (worktree retention model)

At feature end, remove retained worktrees and optionally prune branches:
- Dry-run: `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/<feature>" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Real: `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/<feature>" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## tasks.json Versions and Opt-Ins (Mental Model)

- **v1** (`meta.schema_version: 1`): basic task structure (no cross-platform model; no automation).
 - **v2** (`meta.schema_version: 2`): cross-platform integration model splits scope (P3-008):
   - `meta.behavior_platforms_required`: platforms with behavioral guarantees (smoke scripts required here)
   - `meta.ci_parity_platforms_required`: platforms that must be green in CI parity gates (platform-fix tasks required here; legacy: `meta.platforms_required`)
   - optional WSL via `meta.wsl_required`/`meta.wsl_task_mode` (behavior-scoped; do not include `"wsl"` in the platform arrays)
- **v3** (`meta.schema_version: 3` + `meta.automation.enabled=true`): execution automation is enabled.
  - Required structured fields are enforced by `scripts/planning/validate_tasks_json.py`.
  - Integration tasks must include `merge_to_orchestration` to make merge-back behavior explicit.

## Hard Guardrails (Non-Negotiable)

- Do not edit planning docs inside task worktrees.
- Code/test tasks commit to their own branches; they do not merge back to orchestration.
- Integration merge-back is allowed only from the designated final aggregator (`merge_to_orchestration=true`).
  - If orchestration is behind, the finisher fast-forwards.
  - If orchestration advanced with docs/status commits, the finisher creates a merge commit while preserving the orchestration branch’s Planning Pack files under the feature dir.
- Do not delete per-task worktrees; cleanup removes worktrees at feature end.
- Headless Codex launch (when used) runs via `codex exec --dangerously-bypass-approvals-and-sandbox` and captures output artifacts.

## Where to Look When Something Fails

- Planning lint/validate failures: `make planning-lint` and `make planning-validate`
- Automation workflow and commands: `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- Cross-platform flow and automation diagrams: `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`
- CI smoke dispatch expectations: `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

## End-to-End Smoke Scripts (for debugging the workflow)

These are intentionally operator-facing scripts to exercise the full system (planning pack scaffolding, triad worktrees, Codex headless launch, CI smoke dispatch, and final merge-back):
- Phase 1 (scaffold + C0 code/test parallel): `scripts/e2e/triad_e2e_phase1.sh`
- Phase 2 (integration + CI smoke + optional platform-fix tasks + final aggregator): `scripts/e2e/triad_e2e_phase2.sh`
- Combined runner: `scripts/e2e/triad_e2e_all.sh`
