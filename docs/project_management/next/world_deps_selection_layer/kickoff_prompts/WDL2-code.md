# WDL2-code kickoff — System packages provisioning

Read first:
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

Primary implementation targets:
- `crates/shell/src/builtins/world_deps/*` (new `provision` action)
- Platform scripts for smoke coverage:
  - `scripts/mac/smoke.sh`
  - `scripts/windows/wsl-smoke.ps1`
  - Linux: extend `scripts/linux/world-socket-verify.sh` or add a dedicated world-deps smoke invoked by CI

Hard requirement:
- Linux host must not mutate OS packages; `provision` must fail with exit 4 + manual guidance.

Required checks:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

