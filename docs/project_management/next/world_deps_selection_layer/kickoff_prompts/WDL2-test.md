# WDL2-test kickoff — System packages provisioning tests

Read first:
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

Add tests for:
- Deterministic package list computation (stable ordering, de-dup).
- Platform gating behavior (Linux unsupported → exit 4; Lima/WSL supported → attempts apt).
- Exit-code mapping for backend unavailable vs unsupported vs config errors.

Required checks:
- `cargo fmt`
- Targeted `cargo test ...`

