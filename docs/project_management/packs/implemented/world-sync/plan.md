# world-sync — plan (v4)

Legacy reference (non-authoritative; do not execute from it):

- `docs/project_management/_archived/world-sync-legacy-2026-02-10/`

## Scope

- Feature directory: `docs/project_management/packs/active/world-sync`
- Orchestration branch: `feat/world-sync`
- Authoritative spec ownership map: `docs/project_management/packs/active/world-sync/spec_manifest.md`
- Authoritative impact map: `docs/project_management/packs/active/world-sync/impact_map.md`
- Authoritative CI cadence: `docs/project_management/packs/active/world-sync/ci_checkpoint_plan.md`

## Goal (operator-facing)

Provide a deterministic, auditable “workspace ↔ world” sync workflow plus an internal checkpoint/rollback store:

- `substrate workspace sync` applies pending world-session filesystem changes to the host workspace (directional; policy-driven; guarded).
- `substrate workspace checkpoint` records a host-only snapshot in Substrate’s internal git store.
- `substrate workspace rollback` restores the host workspace to a recorded internal checkpoint.

## Implementation model (high-level; non-git)

- `workspace sync` is **not** implemented as `git push/pull` or by configuring the world as a git remote.
  - Instead, it consumes the world backend’s per-session **pending diff** model (DR-0002) and applies a planned set of filesystem mutations to the host workspace (WS1/WS2/WS4/WS5).
- `workspace checkpoint` / `workspace rollback` use a **host-only** internal git store under `.substrate/git/` (DR-0005; `internal-git-spec.md`) and never touch `.git/`.
  - This uses the standard `git` executable with `--git-dir` pointing into `.substrate/` and `--work-tree` set to the workspace root (no git remotes; no push/pull).
  - This Planning Pack implements checkpoint/rollback only (WS6/WS7), not the broader per-command internal history described in `docs/project_management/future/INTERNAL_GIT.md`.

## Global guardrails (non-negotiable)

- Specs are the single source of truth.
- Planning Pack docs (anything under `docs/project_management/packs/active/world-sync/`) are edited only on `feat/world-sync` (never inside task worktrees).
- Every slice (code/test/integ) MUST fit within the per-task context budget (≤ 108,800 tokens).
- Greenfield by default: no migrations/back-compat for legacy `.substrate-git/` or legacy world-sync CLI (`substrate sync|checkpoint|rollback`). If legacy artifacts exist, world-sync ignores them and uses only `.substrate/`.

## Invariants (must hold in every slice)

- Protected paths are never mutated by sync/checkpoint/rollback:
  - `crates/shell/src/execution/config_model.rs`: `PROTECTED_EXCLUDES = [".git/**", ".substrate/**"]`
- Workspace discovery is authoritative:
  - Workspace root is defined by `.substrate/workspace.yaml` (see `substrate workspace init`).
  - If not in a workspace, workspace-scoped commands exit `2` with actionable guidance to run `substrate workspace init`.
- Exit codes follow taxonomy:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Cross-platform execution model (v4; platform-fix at checkpoints only)

- Behavior platforms (feature smoke required): `linux, macos`
- CI parity platforms (compile parity required): `linux, macos`
- Cross-platform CI dispatch MUST occur only at the bounded checkpoints in `ci_checkpoint_plan.md`.

## Execution gates (enabled)

- Planning quality gate must exist and be `ACCEPT` before any triad starts:
  - `docs/project_management/packs/active/world-sync/quality_gate_report.md`
- Execution preflight task must be completed before any code/test triad starts:
  - Task: `F0-exec-preflight`
  - Report: `docs/project_management/packs/active/world-sync/execution_preflight_report.md`

## Triads (authoritative slice list)

- Checkpoint group CP1 (boundary slice: `WS2`):
  - `WS0` — CLI + gating + dry-run (no mutations)
  - `WS1` — Non-PTY pending-diff discovery + reporting (no mutations)
  - `WS2` — Non-PTY world→host apply (direction=`from_world`) + safety rails
- Checkpoint group CP2 (boundary slice: `WS5`):
  - `WS3` — Auto-sync trigger + policy (non-PTY)
  - `WS4` — PTY pending-diff discovery + reporting
  - `WS5` — Host→world pre-sync + direction expansion (`from_host` / `both`)
- Checkpoint group CP3 (boundary slice: `WS7`):
  - `WS6` — Internal git checkpoint (`substrate workspace checkpoint`)
  - `WS7` — Internal git rollback (`substrate workspace rollback`)

Authoritative task graph:

- `docs/project_management/packs/active/world-sync/tasks.json`
