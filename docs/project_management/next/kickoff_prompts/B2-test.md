# Task B2-test – Test Agent Kickoff Prompt

Task ID: **B2-test** (Test shim doctor/repair CLI)

Summary:
- Add integration coverage for `substrate shim doctor` in both human-readable and `--json` modes, verifying that manifest-derived hints and PATH diagnostics align with the data map expectations.
- Exercise `substrate shim repair --manager <name>` with temporary homes to ensure the repair snippet is appended (with backup file) and that repeated invocations remain idempotent.
- Document the current `cargo test -p substrate-shell` status (expected to pass once B2-code lands) so the integration agent has a clear verification target.

Focus files / context:
- `crates/shell/tests/` (existing CLI + shim deployment tests)
- Manager manifest + init helpers (`crates/shell/src/manager_init.rs`, `crates/common/src/manager_manifest.rs`)
- Planning docs in `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../execution_plan.md`, and `.../data_map.md`

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shim_doctor`

Reminders:
- Begin at `AI_AGENT_START_HERE.md`, update coordination files on `feat/isolated-shell-plan`, and log START/END entries before touching tests.
- Work in `wt/b2-doctor-test`, keep coordination artifacts untouched there, and consume the Test Agent Kickoff Prompt left by B2-code for any implementation specifics.
- Before finishing, craft the Integration Agent Kickoff Prompt for `B2-integ` and record its location in the session log.
