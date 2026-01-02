# Policy + Config Precedence (ADR-0005) — Plan

## Context
This Planning Pack implements the precedence correction defined by:
- `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`

Baseline semantics live in:
- `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`

## Guardrails (non-negotiable)
- Orchestration branch: `feat/policy_and_config_precedence`
- Planning Pack directory: `docs/project_management/next/policy_and_config_precedence/`
- Docs/tasks/session log edits happen only on the orchestration branch (never in worktrees).
- Execution gates are enabled (`tasks.json` meta: `execution_gates: true`): complete `F0-exec-preflight` before starting `PCP0-code` / `PCP0-test`.
- Config parsing remains strict (unknown keys and type mismatches are hard errors).
- Exit codes follow: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- Cross-platform parity is required for the precedence contract (Linux/macOS/Windows).

## Triads overview (spec slices)

1) **PCP0 — Workspace config precedence over env**
   - Reorder effective-config precedence so workspace config overrides `SUBSTRATE_*` env vars when a workspace exists.
   - Preserve existing CLI-flag precedence (CLI flags remain highest precedence).
   - Preserve strict parsing and legacy `.substrate/settings.yaml` hard error behavior.

Specs (single source of truth):
- `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`

## Cross-platform integration model
- Model: schema v2 cross-platform integration tasks (core + per-platform + final).
  - `PCP0-integ-core`: merge code+tests, run `make integ-checks`, and dispatch cross-platform smoke via CI.
  - `PCP0-integ-{linux,macos,windows}`: platform-fix tasks (no-op if already green; only used if CI smoke fails).
  - `PCP0-integ`: final aggregator merges any platform fixes and re-confirms all required platforms are green.
- Cross-platform validation mechanism:
  - Preferred: GitHub Actions self-hosted runners via `make feature-smoke` (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
  - Local smoke execution is valid only on the matching platform (Linux script on Linux, macOS script on macOS, Windows script on Windows).

## Primary code touchpoints (expected)
- Effective config resolution:
  - `crates/shell/src/execution/config_model.rs`
- CLI surfaces that consume effective config:
  - `crates/shell/src/execution/settings/builder.rs`
  - `crates/shell/src/execution/invocation/plan.rs`
- CLI tests locking precedence:
  - `crates/shell/tests/config_show.rs`
  - `crates/shell/tests/config_set.rs` (if impacted)

## Start checklist (all tasks)
1. `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, and your kickoff prompt.
3. Set task status to `in_progress` in `tasks.json`.
4. Add a START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create the task worktree per the kickoff prompt (prefer triad automation where available).
6. Do not edit planning docs inside the worktree.

## End checklist (code/test)
1. Run required commands (code: fmt + clippy; test: fmt + targeted tests). Capture outputs for the END entry.
2. Commit worktree changes to the task branch.
3. Merge/fast-forward into the orchestration branch.
4. Update `tasks.json` + add END entry to `session_log.md`; commit docs (`docs: finish <task-id>`).
5. Do not remove the worktree (worktrees are retained until feature cleanup).

## End checklist (integration)
1. `PCP0-integ-core`: merge code+tests, run `cargo fmt`, `cargo clippy`, relevant tests, then `make integ-checks`.
2. `PCP0-integ-core`: dispatch cross-platform smoke via GitHub Actions and record run URLs/ids in `session_log.md`.
3. If any platforms fail smoke: complete the failing `PCP0-integ-{platform}` tasks (platform-fix work) before starting `PCP0-integ`.
4. `PCP0-integ`: merge any platform fixes, run required checks, re-run cross-platform smoke, and complete `PCP0-closeout_report.md`.
5. Update `tasks.json` + add END entry to `session_log.md`; commit docs (`docs: finish <task-id>`).
