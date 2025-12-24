# World Sync & Internal Git Plan

## Context
- Goal: deliver deterministic host ↔ world filesystem sync with clear directionality, conflict policy, and protected-path rules, then layer in `.substrate-git` as the internal history/rollback store.
- Execution model: triads (code/test/integration) per slice. Code and test run in parallel on separate task branches/worktrees; integration merges and runs full verification.
- Per-triad spec files (`C*-spec.md`) are the single source of truth for scope/acceptance. Code/Test/Integration must align to the spec; integration is responsible for reconciling any drift.

## Global Guardrails
- Orchestration branch: `feat/world-sync`. Docs/tasks/session log live only here; never edit them in worktrees.
- Work happens in dedicated task branches + worktrees per task (names in tasks.json).
- Code agent: writes production code only. No tests. Not required to run unit/integration suites—must run fmt/clippy and validate functionality per spec.
- Test agent: writes tests/fixtures/mocks/harnesses only (and tiny test-only helpers). No production code changes. Runs relevant tests they add/touch.
- Integration agent: merges code+test branches, resolves mismatches, ensures functionality matches the spec, and runs full verification ending with `make preflight` (after fmt/clippy/tests). They own the final green state even if code/test drifted.
- Protected paths: `.git`, `.substrate-git`, `.substrate`, sockets, device files must never be mutated by sync.
- Each task must fit comfortably < 40–50% of 272k context (~110–140k tokens). Keep changes scoped and testable.

## Exit codes (stable taxonomy)

World-sync commands use these exit codes:
- `0`: success, including intentional no-op (no diffs, auto-sync disabled)
- `2`: configuration or usage error (including “workspace not initialized; run substrate init”)
- `3`: world backend unavailable when the command requires it (sync operations only)
- `4`: operation not supported on this platform or not implemented yet
- `5`: safety-rail refusal (protected paths, size guard, clean-tree guard)

## Common Start Checklist (all tasks)
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read this plan, `tasks.json`, `session_log.md`, your task’s kickoff prompt, and the relevant `C*-spec.md`.
3. Update `tasks.json` status → `in_progress`; append START entry to `session_log.md`; commit doc-only change on `feat/world-sync` (`docs: start <task-id>`).
4. Create task branch (see tasks.json) and worktree (from repo root, e.g., `git worktree add wt/<worktree> <branch>`).
5. Do not edit docs/tasks/logs inside worktrees.

## Common End Checklist
- Code/Test tasks:
  1. Run required checks per kickoff (code: fmt/clippy only; test: fmt + relevant tests). Capture outputs for END log.
  2. Commit worktree changes.
  3. Merge/cherry-pick into task branch if needed; from repo root merge back into `feat/world-sync` (ff-only).
  4. Update `tasks.json` status, append END entry to `session_log.md` (commands, results, blockers), create required kickoff prompts for downstream tasks, commit docs (`docs: finish <task-id>`).
  5. Remove worktree if done.
- Integration tasks:
  1. Merge code+test branches into integration worktree; resolve conflicts/mismatches to spec.
  2. Run fmt/clippy + required test suites; finish with `make preflight`. Record outputs.
  3. Merge integration branch back to `feat/world-sync` (ff-only), update `tasks.json`/`session_log.md`, commit docs, remove worktree.

## Triads Overview
- C0: Init + gating (require `substrate init`, create `.substrate/` and `.substrate-git/`, host/world readiness guards).
- C1: Config/CLI surface (no behavior change).
- WDL0–WDL2: World-deps selection layer (executes between C1 and C2; see `docs/project_management/next/world_deps_selection_layer/plan.md`).
- C2: Manual world→host sync (non-PTY) with conflict/filter controls.
- C3: Auto-sync (non-PTY) on session close + safety rails.
- C4: PTY overlay diff + manual/auto world→host sync.
- C5: Host→world pre-sync and directional semantics.
- C6: `.substrate-git` integration (host path) for sync commits/checkpoints.
- C7: Rollback CLI via `.substrate-git`.
- C8: World-side `.substrate-git` bootstrap/bridge (ensure world repo exists/aligns, agent-side ops).
- C9: Init UX & migration (interactive defaults, existing workspace migration, improved gating UX).
