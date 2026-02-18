# Task Triads & Feature Setup Standard

This document explains, step by step, how to create a new feature directory, define triads (code/test/integration), and produce all required files with zero ambiguity.

## Principles
- Every slice of work ships as a triad: code, test, integration.
- Code agent: production code only. No new tests. Runs `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Optional targeted/manual sanity checks are allowed but not required. No unit/integration suite requirement.
  - When the spec changes behavior that invalidates an existing test expectation, the code agent may update the existing test to match the spec (still no new test files or new test cases).
  - Preferred safety practice: run a targeted baseline test set before changes and re-run the same set after to ensure no regressions.
- Test agent: tests only (plus minimal test-only helpers/fixtures/mocks if absolutely needed). No production code. Runs `cargo fmt` and the targeted tests they add/touch; not responsible for full suite.
  - Passing is owned by integration; test-only branches may be red until the code branch lands, but tests must compile and fail deterministically for spec-driven reasons.
- Integration agent: merges code+tests, resolves drift to the spec, ensures behavior matches the spec, runs `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, all relevant tests, and finishes with `make integ-checks` (required). They own the final green state.
- Execution triads must not begin until the Planning Pack has a quality gate report with `RECOMMENDATION: ACCEPT` at `docs/project_management/packs/active/<feature>/quality_gate_report.md` (legacy during migration: `docs/project_management/next/<feature>/quality_gate_report.md`; see `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`).
- If the feature opts into execution gates (`tasks.json` meta: `execution_gates: true`), triads must not begin until the execution preflight gate is completed (see `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`).
- Docs/tasks/session log edits happen **only** on the orchestration branch (never in worktrees).
- Specs are the single source of truth; integration reconciles code/tests to the spec.

## Slice sizing (context budget; non-negotiable)

This repo assumes a typical maximum model context of 272k tokens. Every triad task (code/test/integ) MUST be scoped so a single agent can execute it within 40% of that window.

Hard limit (per task):
- Maximum task context budget: 108,800 tokens (40% of 272,000 tokens).

Enforcement rules (planning-time):
- If a slice cannot be made green deterministically within the per-task context budget, the slice MUST be split into multiple smaller triads before execution begins.
- If a slice requires editing multiple independent subsystems with unrelated acceptance criteria, the slice MUST be split.
- If a slice requires reading most of the Planning Pack to execute, the slice MUST be split.

## Slice IDs (non-negotiable)

Slice IDs must be feature-derived and stable for the lifetime of the feature directory.

Rules:
- Slice ID format: `<SLICE_PREFIX><N>` (e.g., `WCU0`, `WCU1`, `WCU2`).
- The prefix must be derived from the feature name (or explicitly chosen to match it). Do not use generic `C0/C1/...` for new features.
- All artifacts must use the same slice id:
  - spec: `<SLICE_ID>-spec.md`
  - tasks: `<SLICE_ID>-code`, `<SLICE_ID>-test`, `<SLICE_ID>-integ-*`
  - kickoff prompts: `kickoff_prompts/<task-id>.md`

Scaffolding:
- `make planning-new-feature FEATURE=<feature>` auto-derives a prefix from `<feature>` and uses the first slice `<prefix>0`.
- To force a specific prefix: `make planning-new-feature FEATURE=<feature> SLICE_PREFIX=<prefix>`.

## Worktree execution (automation mode)

When tasks are started via triad automation (preferred) and agents run inside an already-created task worktree, follow:
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Creating a New Feature Directory (from scratch)
1. Choose orchestration branch name (e.g., `feat/<feature>`). Create/pull it.
2. Create directory: `docs/project_management/packs/active/<feature>/` (legacy during migration: `docs/project_management/next/<feature>/`).
3. Add files:
   - `plan.md` (runbook/guardrails/triad overview).
   - `tasks.json` (all tasks with ids, worktrees, deps, prompts).
   - `session_log.md` (START/END entries only).
   - Specs: `<SLICE_ID>-spec.md`, ... (one per slice/triad).
   - `kickoff_prompts/` directory with `<task-id>.md` (e.g., `<SLICE_ID>-code.md`, `<SLICE_ID>-test.md`, `<SLICE_ID>-integ.md`).
   - Execution gates (when used):
     - `execution_preflight_report.md`
     - `<SLICE_ID>-closeout_report.md` (e.g., `WCU0-closeout_report.md`)
   - Optional user-facing drafts (e.g., `DRAFT_*.md`).
4. Update `plan.md` triad overview to list all triads.
5. Commit the scaffolding on the orchestration branch.

## Required Content (no ambiguity)
### plan.md
- Principles/guardrails, start/end checklists, triad overview, role rules.

### tasks.json (fields)
- Required fields per task:
  - `id`, `name`, `type` (code/test/integration), `phase`, `status`, `description`
  - `references` (array of files/docs to read)
  - `acceptance_criteria` (array of concrete outcomes)
  - `start_checklist` (array of steps)
  - `end_checklist` (array of steps)
  - `worktree`, `integration_task`, `kickoff_prompt`
  - `depends_on` (list), `concurrent_with` (list)
- Slice Spec v2 traceability:
  - `ac_ids` is required for slice triad tasks (`<SLICE_ID>-code`, `<SLICE_ID>-test`, `<SLICE_ID>-integ`) when `tasks.json` `meta.slice_spec_version >= 2`.
- Optional (required when triad automation is enabled; schema v3):
  - `git_branch` (deterministic task branch name)
  - `required_make_targets` (array of make targets for `task_finish`)
  - `merge_to_orchestration` (integration tasks only; boolean; only `true` for the single integration task that is allowed to merge back to orchestration)
- Example entry:
```json
{
  "id": "<SLICE_ID>-code",
  "name": "<SLICE_ID> slice (code)",
  "type": "code",
  "phase": "<SLICE_ID>",
  "status": "pending",
  "description": "Implement <SLICE_ID> spec (production code only).",
  "references": ["docs/project_management/packs/active/<feature>/<SLICE_ID>-spec.md"],
  "ac_ids": ["AC-<SLICE_ID>-01", "AC-<SLICE_ID>-02", "AC-<SLICE_ID>-03"],
  "acceptance_criteria": [
    "Implements the behaviors required by ac_ids (see <SLICE_ID>-spec.md)"
  ],
  "start_checklist": [
    "Checkout feat/<feature>, pull ff-only",
    "Set status to in_progress, log START, commit docs",
    "Run: make triad-task-start-pair FEATURE_DIR=\"docs/project_management/packs/active/<feature>\" SLICE_ID=\"<SLICE_ID>\""
  ],
  "end_checklist": [
    "Run fmt/clippy",
    "From inside the worktree: make triad-task-finish TASK_ID=\"<SLICE_ID>-code\"",
    "Update tasks/session log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)"
  ],
  "worktree": "wt/<feature>-<slice>-code",
  "git_branch": "<feature>-<slice>-code",
  "required_make_targets": ["triad-code-checks"],
  "integration_task": "<SLICE_ID>-integ",
  "kickoff_prompt": "docs/project_management/packs/active/<feature>/kickoff_prompts/<SLICE_ID>-code.md",
  "depends_on": ["<PREV_SLICE_ID>-integ"],
  "concurrent_with": ["<SLICE_ID>-test"]
}
```

### Specs (`<SLICE_ID>-spec.md`)
Must include:
- Scope (explicit behaviors, defaults, error handling, platform guards, protected paths).
- Acceptance (observable outcomes).
- Out of scope (to avoid scope creep).

### Kickoff prompts
Each prompt must include:
- Scope and explicit role boundaries (“prod code only, no tests” for code; “tests only” for test; integration owns aligning to spec).
- Start checklist (always): read plan/tasks/session_log/spec/prompt, verify the task worktree exists and contains `.taskmeta.json`, and repeat the sentinel rule: `Do not edit planning docs inside the worktree.`
- Requirements: what to build/test, protected paths/safety, required commands (code: fmt/clippy only; test: fmt + targeted tests; integration: fmt/clippy/tests + `make integ-checks`), sanity-check expectations.
- End checklist: run required commands; run `make triad-task-finish`; update tasks.json status and add END entry; commit docs (`docs: finish <task-id>`); do not remove the worktree (feature cleanup removes worktrees at feature end).

## Execution gates (recommended for high-fidelity work)

### Feature start: Execution Preflight Gate

Purpose:
- Confirm the Planning Pack is runnable and the smoke/manual validation plan is strong enough before starting any triads.

Mechanics:
- Use the task `F0-exec-preflight` (type `ops`) and fill:
  - `docs/project_management/packs/active/<feature>/execution_preflight_report.md`
- Standard:
  - `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

### Slice end: Slice Closeout Gate

Purpose:
- Ensure there is no drift between the slice spec and the shipped behavior, with evidence (commands run + smoke run ids/URLs).

Mechanics:
- At the end of `<triad>-integ`, fill:
  - `docs/project_management/packs/active/<feature>/<triad>-closeout_report.md`
- Standard:
  - `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md`

## Branch/Worktree Naming
- Branch: `<feature-prefix>-<triad>-<short-scope>` (e.g., `ws-c3-autosync-code` for world-sync, `ss-s2-settings-code` for settings-stack style). Use a consistent prefix per feature.
- Worktree: `wt/<branch>` or `wt/<feature-prefix>-<triad>-<short-scope>`.
- Integration worktrees may be created from a dedicated integration branch or directly from the orchestration branch; ensure tasks.json/kickoff prompts specify the expected workflow.

### Automation naming (opt-in)
If the Planning Pack opts into triad automation (`tasks.json` meta: `schema_version >= 3` and `meta.automation.enabled=true`):
- Every code/test/integration task must declare a deterministic branch name in `tasks.json` as `git_branch`.
- Worktree paths remain explicit via `tasks.json` `worktree`.

## Start/End Checklists (copy/paste)
Note: `make triad-task-start` / `make triad-task-finish` require an automation-enabled Planning Pack (`tasks.json` meta: `schema_version >= 3` and `meta.automation.enabled=true`). Legacy packs must either migrate or follow the manual worktree workflow described in their existing prompts.

Start (all tasks):
1. Ensure the orchestration branch exists and is checked out:
  - Automation packs: `make triad-orch-ensure FEATURE_DIR="docs/project_management/packs/active/<feature>"`
   - Legacy packs: `git checkout <orchestration-branch> && git pull --ff-only`
2. Read plan/tasks/session_log/spec/prompt.
3. Set task status to `in_progress` in tasks.json.
4. Add START entry to session_log.md; commit docs (`docs: start <task-id>`).
5. Create task branch + worktrees via the task runner:
  - Code+test (always parallel): `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/<feature>" SLICE_ID="<slice>"`
  - Integration (single task): `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/<feature>" TASK_ID="<task-id>"`
6. Do not edit planning docs inside the worktree.

Optional: also launch Codex headless for both code+test tasks:
- Preferred (for reliable artifact reporting): `docs/project_management/system/prompts/triad_wrappers/triad_wrapper.md`
- `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/<feature>" SLICE_ID="<slice>" LAUNCH_CODEX=1`

Optional: start only the failing platform-fix integration tasks (after smoke results are known):
- `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/active/<feature>" SLICE_ID="<slice>" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

Optional: start the final aggregator integration task for a slice (requires its deps are completed):
- `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/packs/active/<feature>" SLICE_ID="<slice>" LAUNCH_CODEX=1`
  - Note: the final aggregator task id is `<slice>-integ` (the command name contains `integ-final`).

End (code/test):
1. Run required commands (code: fmt/clippy; test: fmt + targeted tests). Capture outputs.
2. From inside the worktree, run the task finisher (commits to the task branch; does not merge to orchestration for code/test):
   - `make triad-task-finish TASK_ID="<task-id>"`
3. Switch to the orchestration branch; update tasks.json status and add the END entry (commands/results/blockers) to session_log.md; commit docs (`docs: finish <task-id>`).
4. Do not remove the worktree (worktrees are retained until feature cleanup).

End (integration):
1. Merge code+test task branches into the integration worktree; resolve drift to spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`. Capture outputs.
3. From inside the worktree, run the task finisher (commits to the task branch; merge-back to orchestration happens only when `merge_to_orchestration=true` for that task):
   - `make triad-task-finish TASK_ID="<task-id>"`
   Notes:
   - Prefer not to merge the orchestration branch into task branches. The finisher merges the task branch back to orchestration even if orchestration advanced with docs/status commits; it preserves the orchestration branch’s Planning Pack files under the feature dir.
4. On the orchestration branch, update tasks.json/session_log.md with END entry; commit docs (`docs: finish <task-id>`).
5. Do not remove the worktree (worktrees are retained until feature cleanup).

### Feature cleanup (worktree retention model)
Worktrees are removed only by a feature-level cleanup task (recommended id: `FZ-feature-cleanup`) using:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/<feature>" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Cleanup consumes the deterministic registry created by `task_start`:
- `<git-common-dir>/triad/features/<feature>/worktrees.json`

Guardrails:
- Cleanup refuses to remove dirty worktrees or delete unmerged/unpushed branches unless forced (`FORCE=1`).

### Headless Codex launch (optional)
`task_start` may launch Codex headless from inside the worktree using the kickoff prompt as stdin. Canonical invocation shape:
- `codex exec --dangerously-bypass-approvals-and-sandbox --cd <worktree> --output-last-message <path> - < <kickoff_prompt.md>`

Optional flags (when needed):
- `--profile <profile>` (selects a codex config profile)
- `--model <model>` (selects a model)
- `--json` (emit JSONL events to stdout; redirect to a file for auditability)

Automation wrapper:
- `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/<feature>" TASK_ID="<task-id>" LAUNCH_CODEX=1`
- `make triad-task-start-complete FEATURE_DIR="docs/project_management/packs/active/<feature>" SLICE_ID="<slice>"` (code+test in parallel, then runs the integration task wired via `<slice>-code.integration_task`; writes wrapper summary; does not run CI checkpoint ops tasks)

## Role Command Requirements
- Code: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`; optional targeted/manual sanity checks allowed but not required; no unit/integration suite requirement.
- Test: `cargo fmt`; targeted `cargo test ...` for tests added/modified; no production code; no responsibility for full suite.
- Integration: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`; run relevant tests (at least new/affected suites) and finish with `make integ-checks` (required full-suite gate). Integration must reconcile code/tests to the spec.
  - If the feature includes a manual validation playbook and smoke scripts (see `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`), integration must run the required validation gates and record results (including run ids/URLs for CI) in the feature `session_log.md`:
    - Cross-platform CI dispatch is scheduled by `ci_checkpoint_plan.md` (bounded CI checkpoints). Do not dispatch cross-platform CI/smoke from every slice.
    - **Behavior platforms** (P3-008): smoke scripts are required here; dispatch via `make feature-smoke`.
    - **CI parity platforms** (P3-008): smoke is not required for CI parity-only platforms; use compile parity/CI Testing gates instead.
    - Use the advisory CI audit + evidence ledger tooling to reduce redundant multi-OS runs while preserving safety:
      - Before dispatch: `scripts/ci-audit/ci_audit.sh` (use `--ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl"`).
      - After dispatch: `scripts/ci-audit/ci_audit_record.sh` (record `tested_sha` + run id/URL; do not commit the ledger).
      - Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke when the audit shows `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.
    - Preferred cross-platform mechanism: `make feature-smoke` (self-hosted runners), e.g.:
      - Behavior platforms (preferred): `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=behavior RUN_WSL=1 WORKFLOW_REF="feat/<feature>"`
      - Debugging single platform: `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux RUN_WSL=1 WORKFLOW_REF="feat/<feature>"`
    - Use direct local execution only when the platform matches the current machine (e.g., `bash "$FEATURE_DIR/smoke/linux-smoke.sh"` on Linux).

## Cross-platform integration task model (platform-fix)

For cross-platform Planning Packs (`tasks.json` meta: `cross_platform: true`), the integration structure depends on schema version (see also `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md`):

- Schema v2/v3 (legacy): the full platform-fix structure may exist per slice:
  - `X-integ-core`: merges `X-code` + `X-test`, gets primary-platform green, and runs local integration gates (plus a local behavioral smoke preflight when possible).
  - `CPk-ci-checkpoint` (ops task; planned boundary): dispatches compile parity + Feature Smoke for the checkpoint slice’s `X-integ-core` commit (uses `CI_CHECKOUT_REF` / `SMOKE_CHECKOUT_REF`), then starts platform-fix tasks only when needed.
  - `X-integ-linux|macos|windows` (and optional `X-integ-wsl`): platform-fix tasks that:
    - validate via `make feature-smoke` for the platform when it is a behavior platform (dispatch from the orchestration/task ref; never from `main`/`testing`),
    - otherwise validate via CI parity gates (compile parity / CI Testing),
    - apply fixes on the corresponding platform machine/worktree only if smoke fails,
    - re-run smoke until green.
  - `X-integ` (final): merges any platform-fix branches, runs `make integ-checks`, completes slice closeout, and merges back to orchestration.

- Schema v4+ (recommended): boundary-only platform-fix:
  - Normal slices: `X-code`, `X-test`, `X-integ` (single merge task).
  - Checkpoint-boundary slices only (listed in `meta.checkpoint_boundaries`, must match `ci_checkpoint_plan.md` boundaries): `B-integ-core`, `B-integ-<platform>`, `B-integ`.

Kickoff prompt templates for this model:
- Core integration: `docs/project_management/system/templates/kickoff/kickoff_integ_core.md.tmpl`
- Platform-fix integration: `docs/project_management/system/templates/kickoff/kickoff_integ_platform.md.tmpl`
- Final aggregator integration: `docs/project_management/system/templates/kickoff/kickoff_integ_final.md.tmpl`

## Context Budget & Triad Sizing
- Agents typically have a 272k token context window. Every triad task (code/test/integ) MUST be scoped so a single agent can execute it within 40% of that window (≤ 108,800 tokens) while holding the spec, plan, and relevant code/tests and history in-context.
- If a task risks breaching that budget (large migration, many platforms, or broad refactors), split into additional triads or narrower phases before kickoff. If you are uncertain, split.
- Use specs to keep scope crisp; avoid “grab bag” triads. Aim for small, testable chunks with clear acceptance criteria.

Practical sizing rules (recommended):
- Define each slice as **one behavior delta** (one “Existing → New → Why”) with a small, explicit contract surface.
- Prefer **more slices** over bigger slices. Small slices are the main hedge against context compaction and multi-agent handoffs.

Good slice boundaries:
- A single CLI command/subcommand behavior change (including its config/flags, errors, and tests).
- One config key/precedence rule end-to-end (parse → validate → apply → observe).
- One platform-sensitive behavior (plus the platform guards) with explicit parity expectations.
- One failure mode/edge case class with tests (e.g., redaction, permissions, missing deps).

Split triggers (if any are true, split into additional slices):
- The slice has multiple independent UX flows (multiple “Existing → New → Why” deltas).
- The slice touches multiple major subsystems (e.g., shim + broker + world-agent + world backend) without a narrow, single-threaded data flow.
- The acceptance criteria list is “wide” (many unrelated bullets) instead of “deep” (few bullets with clear constraints).
- The work requires heavy refactors plus new behavior in the same slice (split “refactor enabling change” from “feature behavior”).
- Cross-platform parity requires substantial platform-specific behavior differences (split to isolate platform-fix work).

Anti-patterns (avoid):
- Slice titles/descriptions that contain “and”/“plus”/“misc”.
- Slices that include “implement everything” for a feature.
- Slices where tests are the only definition of the behavior (spec must be explicit).

## Protected Paths
If relevant to the feature (e.g., sync/FS operations), explicitly list in specs/prompts: `.git`, `.substrate-git`, `.substrate`, sockets, device nodes, and any feature-specific exclusions.

## Typical Triad Ordering (example: world-sync)
- WS0: Init & gating
- WS1: Config/CLI surface
- WS2: Manual path A
- WS3: Auto path A
- WS4: Additional path (e.g., PTY)
- WS5: Opposite direction
- WS6: Internal system (host)
- WS7: Rollback/CLI
- WS8: Internal system (world/bridge)
- WS9: UX/migration polish
Adjust counts to keep each triad task (code/test/integ) ≤ 40% of a 272k context window (≤ 108,800 tokens).

## Session Log Usage
- Only START/END entries. Include: timestamp (UTC), agent role, task ID, commands run (fmt/clippy/tests/scripts), results (pass/fail, temp roots if applicable), worktree/commits touched, prompts created/verified, blockers/next steps.
- Use a consistent template (can copy the settings-stack template) and do not edit from worktrees.

## Adding New Triads (step-by-step)
1. Create spec file (`<SLICE_ID>-spec.md`) with scope/acceptance/out-of-scope.
2. Add tasks (code/test/integ) to tasks.json with worktrees/branches/deps/prompts.
3. Create kickoff prompts for code/test/integ.
4. Update plan.md triad overview.
5. Commit docs on orchestration branch.
