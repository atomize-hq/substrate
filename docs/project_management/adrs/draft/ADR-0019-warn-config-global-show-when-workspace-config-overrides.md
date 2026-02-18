# ADR-0019 — Warn on `config global show` when workspace config overrides

Date (UTC): 2026-01-30

Status: Proposed

Owners: Substrate maintainers

## Executive Summary (Operator)

ADR_BODY_SHA256: 6b8c510ead88e391788963e6bd8b965273c4aaa8312d91df9ea245126c78ad2b
### Changes (operator-facing)
- Workspace override note for `substrate config global show`
  - Existing: `substrate config global show` prints the global config patch, which can be misinterpreted as the effective config when run inside a workspace.
  - New: when run inside an enabled workspace whose workspace config override is active (non-empty or unparseable), the command emits a single stderr note routing operators to `substrate config show --explain`; stdout and exit codes remain unchanged.
  - Why: reduce operator confusion while preserving script safety (stdout patch-only; stderr-only note).
- Explicit write-scope note for implicit-scope `substrate config set`
  - Existing: `substrate config set ...` updates the workspace config patch by default, but the command output does not explicitly state which scope/file was modified.
  - New: when the user does not explicitly specify the scope (i.e., uses `substrate config set ...` rather than `substrate config global set ...`), emit a single stderr note that names the updated scope and the on-disk config path.
  - Why: prevent “I changed config but don’t know where” confusion while keeping stdout stable for scripting.
- Existing note interaction
  - Existing: the command emits an informational note when the global patch is empty.
  - New: when the workspace-override note is emitted, the global-empty note is suppressed for that invocation.
  - Why: keep stderr high-signal and avoid double-notes.

Links:
- Plan: `docs/project_management/next/warn-config-global-show-workspace-overrides/plan.md`
- Tasks: `docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json`
- Spec manifest: `docs/project_management/next/warn-config-global-show-workspace-overrides/spec_manifest.md`
- Spec: `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md`
- Contract: `docs/project_management/next/warn-config-global-show-workspace-overrides/contract.md`
- Impact map: `docs/project_management/next/warn-config-global-show-workspace-overrides/impact_map.md`
- CI checkpoint plan: `docs/project_management/next/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md`
- Manual Playbook: `docs/project_management/next/warn-config-global-show-workspace-overrides/manual_testing_playbook.md`

## Context

`substrate config global show` prints the global config patch. When the current directory is inside a workspace,
the effective config for that directory may differ due to workspace overrides (workspace patch has higher precedence).

Today, operators can misinterpret the global patch output as “the config I’m using here”, especially when debugging
unexpected behavior in a workspace. The effective view already exists (`substrate config show --explain`); this ADR
adds a high-signal note to route operators to the correct command when a workspace override is active.

Additionally, `substrate config set ...` (without an explicit `global` or `workspace` subcommand) writes to the
workspace config patch. Because the command prints the effective merged config after the update, operators can miss
which scope/file was actually modified. A single-line stderr note stating the write scope + path makes this explicit
at the point of action.

Related ADRs / background:
- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`

## Goals

- Make it obvious (at the point of use) that `config global show` is not the effective config view when workspace overrides apply.
- Make it obvious (at the point of update) which config scope/file was modified for implicit-scope `config set`.
- Maintain script safety:
  - stdout remains patch-only (no extra text),
  - exit codes remain unchanged for successful show operations.
- Avoid introducing new failure modes for `config global show` (it must keep working even if workspace config is invalid).

## Non-goals

- Do not change config precedence, merge semantics, or supported config keys.
- Do not add per-key “overlap” enumeration (which keys are overridden); keep the note minimal and stable.
- Do not change `policy global show` behavior in this ADR (separate work item if needed).
- Do not change the default scope resolution for `substrate config set ...` (it remains workspace-scoped).

## Decision

### When to warn

`substrate config global show` emits the note **iff**:
1. The current working directory is inside an enabled workspace, and
2. The workspace override is “active”:
   - the workspace patch parses and is non-empty, OR
   - the workspace patch fails to parse (treated as active for warning purposes, without failing the command).

Rationale:
- This matches the backlog intent (“only when an override applies”) while remaining robust in invalid-YAML cases.

### What the note says

The note MUST be a single stderr line with the exact template (dynamic path substituted):

`substrate: note: workspace config <WORKSPACE_CONFIG_PATH> overrides global config here; run 'substrate config show --explain' to view the effective config for this directory`

Where `<WORKSPACE_CONFIG_PATH>` is `<workspace_root>/.substrate/workspace.yaml`.

### Interaction with existing notes

If the workspace-override note is emitted, suppress the existing “global config patch is empty (no overrides)” note for that invocation.
This keeps stderr output high-signal (no double-notes) while leaving stdout unchanged.

### Explicit write-scope note for implicit `config set` (new)

`substrate config set <KEY>=<VALUE> ...` updates the workspace config patch. When invoked without an explicit scope,
the command MUST emit a single stderr note stating the write target:

`substrate: note: write target is workspace config <WORKSPACE_CONFIG_PATH> (implicit scope); run 'substrate config workspace show' to view the workspace patch`

Where `<WORKSPACE_CONFIG_PATH>` is `<workspace_root>/.substrate/workspace.yaml`.

Notes:
- This note is stderr-only; stdout and exit codes remain unchanged.
- This note is only required for implicit-scope `substrate config set ...` (not `substrate config global set ...`).

## User Contract

Authoritative contract text lives in:
- `docs/project_management/next/warn-config-global-show-workspace-overrides/contract.md`

Key invariants:
- stderr-only note, single-line
- stdout remains patch-only YAML/JSON
- `config global show` does not begin failing due to workspace config parse errors

## Implementation Notes

Primary implementation location:
- `crates/shell/src/execution/config_cmd.rs` → `run_global_show`

Mechanics:
- Use `workspace::find_workspace_root(&cwd)` to detect an enabled workspace.
- Read `<workspace_root>/.substrate/workspace.yaml` and attempt `config_model::parse_config_patch_yaml(...)`.
- Treat parse failures as “override active” for warning purposes, but do not fail the command.

## Test Plan

- Unit/integration tests under `crates/shell/tests/` to cover:
  - outside workspace (no note),
  - empty workspace patch (no note),
  - non-empty patch (note),
  - invalid YAML patch (note + success),
  - `--json` stdout parseability.
- Manual playbook + smoke scripts (linux/macos/windows) mirror these cases.

## Backwards Compatibility

This change is additive and stderr-only:
- It may affect tooling that asserts stderr is empty for `config global show` when run inside a workspace with overrides.
- stdout and exit codes remain unchanged for successful operations.
