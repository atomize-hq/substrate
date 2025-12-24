# WDL1-code kickoff — Install classes

Read first:
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/decision_register.md`

Primary implementation targets:
- `crates/common/src/manager_manifest/schema.rs` (schema bump to v2 + validation)
- `crates/common/src/world_deps_manifest.rs` (surface install class metadata)
- `crates/shell/src/builtins/world_deps/*` (routing and UX)
- Substrate-owned manifests:
  - `config/manager_hooks.yaml`
  - `scripts/substrate/world-deps.yaml`

Hard requirement:
- Runtime `world deps sync/install` must never execute OS package managers.

Required checks:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

