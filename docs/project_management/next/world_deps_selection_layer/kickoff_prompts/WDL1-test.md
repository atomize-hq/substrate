# WDL1-test kickoff — Install classes tests

Read first:
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`

Add tests for:
- Manifest v2 validation rules (class required; mutually-exclusive fields).
- Routing: system_packages and manual are blocked with correct exit codes/messages.
- Ensure no runtime path can execute `apt`/package-manager recipes.

Required checks:
- `cargo fmt`
- Targeted `cargo test ...` suites

