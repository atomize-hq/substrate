# WDL0-test kickoff — Selection config + UX tests

Read first:
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`

Add tests for:
- Selection file parsing + precedence (workspace overrides global).
- Unconfigured-state no-op (exit 0, no world calls; assert via mocks where possible).
- `--all` scope behavior.
- Unknown tools in selection → exit 2.

Required checks:
- `cargo fmt`
- Targeted `cargo test -p substrate-shell ...` suites you touch

