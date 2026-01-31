# Integration Map — Warn on `config global show` when workspace config overrides

This map ties the planning artifacts to concrete code surfaces and validation points.

Deprecated:
- Replaced by `docs/project_management/next/warn-config-global-show-workspace-overrides/impact_map.md`

## Primary code surfaces

### CLI implementation
- `crates/shell/src/execution/config_cmd.rs`
  - `run_global_show` (target for the stderr note + workspace detection)

### Workspace detection
- `crates/shell/src/execution/workspace.rs`
  - `find_workspace_root` (enabled workspace detection)
  - `workspace_marker_path` (resolves `<root>/.substrate/workspace.yaml`)

### Patch parsing
- `crates/shell/src/execution/config_model.rs`
  - `parse_config_patch_yaml` (parse workspace patch; failures MUST be treated as “override active” without failing)

## Tests

- Add/extend a focused test module under `crates/shell/tests/` that:
  - constructs a temp workspace (`substrate workspace init ...` or direct file writes),
  - captures stderr for `substrate config global show`,
  - asserts note presence/absence and stdout invariants (including `--json` parseability).

Existing helpers worth reusing:
- `crates/shell/tests/shared.rs` (temporary home/workspace helpers)

## Manual validation

- `docs/project_management/next/warn-config-global-show-workspace-overrides/manual_testing_playbook.md`

## Smoke validation (automation)

Feature smoke scripts (must mirror the manual playbook):
- Linux: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/windows-smoke.ps1`

CI dispatch entrypoint:
- `scripts/ci/dispatch_feature_smoke.sh --feature-dir docs/project_management/next/warn-config-global-show-workspace-overrides`
