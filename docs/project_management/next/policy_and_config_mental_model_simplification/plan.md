# Policy + Config Mental Model Simplification (ADR-0003) — Plan

## Context
ADR-0003 defines a single, strict mental model for Substrate config, policy, workspace discovery, naming, and env state.
This Planning Pack turns ADR-0003 into execution-ready triads with explicit scope boundaries and validation.

Authoritative contract:
- `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`

## Guardrails (non-negotiable)
- Orchestration branch: `feat/policy_and_config`
- Planning Pack directory: `docs/project_management/next/policy_and_config_mental_model_simplification/`
- Docs/tasks/session log edits happen only on the orchestration branch (never in worktrees).
- Greenfield: remove all legacy compatibility paths; no aliases; no fallbacks.
- YAML only for config and policy; parsing is strict (unknown keys and type mismatches are hard errors).
- Follow `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.

### Role boundaries (triad workflow)
- Code agent: production code only; no tests; run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Test agent: tests only; no production code; run `cargo fmt` and targeted `cargo test ...`.
- Integration agent: reconcile code+tests to spec; run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and finish with `make integ-checks`; run feature-local smoke scripts and record results in `session_log.md`.

## Triads overview (spec slices)

1) **PCM0 — Workspace + config inventory and CLI**
   - Workspace discovery marker, nested init refusal, `.gitignore` rules.
   - Global/workspace config schema, discovery, merge precedence, strict parsing.
   - `substrate workspace init` and `substrate config *` commands per ADR-0003.

2) **PCM1 — Policy inventory and CLI**
   - Policy schema, strict parsing + invariants, discovery and precedence.
   - `substrate policy *` commands per ADR-0003.

3) **PCM2 — Policy mode + routing semantics**
   - `policy.mode=disabled|observe|enforce` semantics.
   - `cmd_*` evaluation semantics and “requires world” constraints.
   - Host vs world selection rules and fail-closed behavior in enforce mode.
   - Approval “save to policy” write target selection.

4) **PCM3 — Env scripts + world enable home + legacy removals**
   - `$SUBSTRATE_HOME/env.sh` and `$SUBSTRATE_HOME/manager_env.sh` ownership and behavior.
   - `substrate world enable --home` semantics and `--prefix` removal.
   - Removal of legacy filenames/flags/env vars and naming collisions (anchor/isolation).

Specs (single source of truth):
- `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`
- `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`
- `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`
- `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`

## Cross-sprint alignment (explicit)
- Sequencing spine: `docs/project_management/next/sequencing.json` (sprint `policy_and_config_mental_model_simplification`, order `25`).
- This sprint lands before `world_sync`.
- Before any `world_sync` execution triads begin, update world-sync planning docs to match ADR-0003 (minimum: `docs/project_management/next/world-sync/C0-spec.md` and `docs/project_management/next/world-sync/C1-spec.md`).

## Primary code touchpoints (expected)
These are the primary locations expected to change when implementing ADR-0003:
- Config/policy loaders and CLI:
  - `crates/shell/src/execution/settings/builder.rs`
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/execution/config_cmd.rs`
  - `crates/common/src/paths.rs`
  - `crates/broker/src/*` (policy semantics and approvals write targets)
- World enable + env scripts + runtime wiring:
  - `crates/shell/src/builtins/world_enable/*`
  - `crates/shell/src/shim/*` (shim lifecycle; manager env wiring)
  - `crates/world-agent/src/service.rs`
  - `crates/world/src/exec.rs`

## Start checklist (all tasks)
1. `git checkout feat/policy_and_config && git pull --ff-only`
2. Read: `plan.md`, `tasks.json`, `session_log.md`, the relevant `PCM*-spec.md`, and your kickoff prompt.
3. Set task status to `in_progress` in `tasks.json`.
4. Add a START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create the task branch and worktree per the kickoff prompt.
6. Do not edit docs/tasks/session_log.md inside the worktree.

## End checklist (code/test)
1. Run required commands (code: fmt + clippy; test: fmt + targeted tests). Capture outputs for the END entry.
2. Commit worktree changes to the task branch.
3. Merge/fast-forward into the orchestration branch.
4. Update `tasks.json` + add END entry to `session_log.md`; commit docs (`docs: finish <task-id>`).
5. Remove the worktree.

## End checklist (integration)
1. Merge code+test branches into the integration worktree; reconcile to spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
3. Run feature-local smoke scripts:
   - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh`
   - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh`
   - `pwsh -File docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1`
4. Commit integration changes; merge/fast-forward into the orchestration branch.
5. Update `tasks.json` + add END entry to `session_log.md`; commit docs (`docs: finish <task-id>`); remove the worktree.
