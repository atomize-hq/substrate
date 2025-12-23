# YAML Settings Migration Plan (TOML → YAML)

## Context

Substrate currently uses TOML for the layered runtime settings stack:
- `~/.substrate/config.toml`
- `.substrate/settings.toml`
- CLI: `substrate config init/show/set`

Most Substrate-owned runtime artifacts are already YAML (policies + manifests). We are standardizing to
**YAML-only** across Substrate-owned runtime config/manifests/policies.

This track migrates only the **settings stack** from TOML to YAML (smaller footprint than converting all
policies/manifests to TOML).

## Guardrails

- Orchestration branch: `feat/yaml-settings-migration`
- Docs/tasks/session log edits happen **only** on the orchestration branch (never in worktrees).
- Each slice ships as a triad: **code**, **test**, **integration**.
- Follow `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.

### Role boundaries

- Code agent: production code only. No tests.
  - Required: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`
- Test agent: tests only (plus minimal test-only helpers). No production code.
  - Required: `cargo fmt`; targeted `cargo test ...`
- Integration agent: merges code/test, reconciles to spec, runs:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant `cargo test ...`
  - `make preflight` (required)

## Triads Overview

Single triad (kept intentionally small):

1) **Y0 — Migrate config/settings stack to YAML**

## Start Checklist (all tasks)

1. `git checkout feat/yaml-settings-migration && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `Y0-spec.md`, and your kickoff prompt.
3. Set task status → `in_progress` in `tasks.json`.
4. Add START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create task branch and worktree: `git worktree add wt/<worktree> <branch>`.
6. Do not edit docs/tasks/logs in worktrees.

## End Checklist (code/test)

1. Run required commands (code: fmt/clippy; test: fmt + targeted tests). Capture outputs.
2. Commit worktree changes to the task branch.
3. Switch back to orchestration branch; update `tasks.json` + add END entry to `session_log.md`; commit docs.
4. Remove worktree.

## End Checklist (integration)

1. Merge code+test branches into the integration worktree; reconcile to spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.
3. Commit integration changes; fast-forward into orchestration branch.
4. Update `tasks.json` + `session_log.md`; commit docs; remove worktree.

