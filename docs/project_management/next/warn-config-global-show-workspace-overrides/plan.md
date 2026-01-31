# Plan — Warn on `config global show` when workspace config overrides

This plan is anchored by:
- `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/decision_register.md`

## Goal

Make `substrate config global show` self-explanatory when run inside a workspace that has non-empty
workspace config overrides, by emitting a single stderr note that points the operator at the
effective config view: `substrate config show --explain`.

## Guardrails (non-negotiable)

- **No behavior/semantics changes**: config precedence and merge behavior are unchanged.
- **No stdout contamination**: `config global show` stdout MUST remain *only* the serialized global patch (YAML/JSON).
- **Stderr-only note**: the new warning MUST be emitted on stderr and MUST be a single line.
- **No new failure modes**: `config global show` MUST NOT begin failing due to unreadable or invalid workspace config.
- **No overlap-key enumeration in v1**: do not attempt to list “which keys are overridden”; keep the UX minimal and stable.

## Platform scope (planning pack)

- Behavior platforms (smoke required): `linux`, `macos`, `windows`
- CI parity platforms (compile/test parity): `linux`, `macos`, `windows`
- WSL: not required for this feature.

## Slice plan (triads)

- `C0`: implement workspace-override note + tests + smoke validation.

## Planning Pack artifact index (this directory)

- `docs/project_management/next/warn-config-global-show-workspace-overrides/plan.md`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/session_log.md`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/spec_manifest.md`
- Spec (slice-level): `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md`
- Contract: `docs/project_management/next/warn-config-global-show-workspace-overrides/contract.md`
- Decision Register: `docs/project_management/next/warn-config-global-show-workspace-overrides/decision_register.md`
- Impact Map: `docs/project_management/next/warn-config-global-show-workspace-overrides/impact_map.md`
- CI checkpoint plan: `docs/project_management/next/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md`
- Manual Playbook: `docs/project_management/next/warn-config-global-show-workspace-overrides/manual_testing_playbook.md`
- Smoke scripts:
  - `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh`
  - `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh`
  - `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/windows-smoke.ps1`
- Kickoff prompts: `docs/project_management/next/warn-config-global-show-workspace-overrides/kickoff_prompts/`

## Implementation outline (authoritative)

### 1) Add workspace-override detection in `config global show`

Target:
- `crates/shell/src/execution/config_cmd.rs` → `run_global_show`

Approach:
1. Resolve `cwd` and attempt to find an enabled workspace root via:
   - `workspace::find_workspace_root(&cwd)`
2. If a workspace root exists:
   - Resolve the workspace config path:
     - `workspace::workspace_marker_path(&workspace_root)` (this is `<root>/.substrate/workspace.yaml`)
   - Attempt to read and parse that file as a config patch:
     - `config_model::parse_config_patch_yaml(&raw_yaml)`
3. Determine whether a workspace override is “active” for the purpose of printing the note:
   - Parse **success + empty patch** ⇒ override **inactive** ⇒ no note.
   - Parse **success + non-empty patch** ⇒ override **active** ⇒ print note.
   - Parse **failure** (invalid YAML) ⇒ override **treated as active** ⇒ print note (but MUST NOT fail the command).
4. If override is active, emit the new stderr note (spec-defined text).
5. Preserve the existing “global patch is empty (no overrides)” note **only** when the workspace-override note is NOT emitted.

### 2) Tests

Target:
- `crates/shell/tests/` (add a focused test module for `config global show` stderr behavior)

Tests MUST cover:
- Outside workspace: no workspace-override note.
- Inside workspace + empty workspace patch: no workspace-override note.
- Inside workspace + non-empty workspace patch: workspace-override note present; stdout is patch only.
- `--json` mode: stdout parses as JSON even when stderr note is present.
- Workspace patch invalid YAML: command still succeeds and emits the workspace-override note.

### 3) Manual + smoke validation

- Manual: `manual_testing_playbook.md` is authoritative.
- Smoke scripts MUST mirror the manual playbook cases and assert:
  - note present/absent conditions
  - stdout is uncontaminated and JSON parseable for `--json`

## CI dispatch policy (recommended-first)

Checkpoint tasks and platform-fix tasks use:
- `scripts/ci-audit/ci_audit.sh` (recommended before dispatch)
- `scripts/ci/dispatch_ci_testing.sh` (compile/test parity)
- `scripts/ci/dispatch_feature_smoke.sh` (feature smoke scripts)

Evidence (CI audit ledgers + run ids) lives under:
- `docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/` (gitignored)
