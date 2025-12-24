# WDL0-code kickoff — Selection config + UX

Read first:
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/decision_register.md`

Primary implementation targets (expected):
- `crates/shell/src/builtins/world_deps/*` (selection gating + UX + exit codes)
- `crates/common/src/paths.rs` (new selection file paths; YAML-only naming must align with Y0)

Non-goals:
- Do not implement install classes (WDL1).
- Do not implement provisioning (WDL2).

Required checks:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

