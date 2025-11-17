# Task D3-integ – Integration Agent Kickoff Prompt

Task ID: **D3-integ** (Integrate world env toggle audit)

Summary / current state:
- D3-code introduced the shared host-only helper (`scripts/dev/substrate_shell_driver`) plus the Rust helper in `crates/shell/tests/common.rs`. The integration suites (`integration.rs`, `world_enable.rs`, `world_deps.rs`, `shim_doctor.rs`, `shim_health.rs`) now call this helper so they inherit the TMPDIR and `SUBSTRATE_WORLD=disabled` overrides.
- Host-only scripts now call the helper or explicitly export the toggles: `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, `scripts/substrate/dev-shim-bootstrap.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, and `scripts/windows/dev-uninstall-substrate.ps1`. Docs (`docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`) instruct contributors to use the helper; python REPL harnesses still run with `--no-world`.
- D3-test pulled those changes into `wt/d3-world-audit-test` and re-ran the command list below; all tests passed with the existing dead-code warnings from the shared fixtures.

What you need to do:
1. From `feat/isolated-shell-plan`, create/switch to `wt/d3-world-audit-integ`, merge `wt/d3-world-audit-code` + `wt/d3-world-audit-test`, and resolve conflicts in the touched files and planning docs.
2. Re-run the full host-only command suite to confirm the helper behaves consistently:
   - `cargo fmt --all`
   - `cargo test -p substrate-shell integration -- --nocapture`
   - `cargo test -p substrate-shell world_enable`
   - `cargo test -p substrate-shell world_deps`
   - `cargo test -p substrate-shell shim_doctor`
   - `cargo test -p substrate-shell --test shim_health -- --nocapture`
   - `SUBSTRATE_BIN=target/debug/substrate scripts/dev/substrate_shell_driver --no-world -c 'echo helper-ok'`
   - `BIN=target/debug/substrate scripts/validate_phase_d.sh`
   Expect only the existing `DoctorFixture` dead-code warnings.
3. For scripts you cannot execute in CI (e.g., Podman rotation, mac/WSL smoke helpers), verify by inspection that they either shell through the driver with `SUBSTRATE_BIN` overrides or intentionally keep isolation enabled (document any intentional exceptions like `scripts/mac/smoke.sh`).
4. Update planning artifacts: log START/END entries referencing this prompt and set D3-code/D3-test/D3-integ statuses appropriately in `docs/project_management/next/tasks.json`.
5. Ensure `scripts/dev/substrate_shell_driver` remains executable and clearly documented for future contributors. If any remaining entry point still bypasses the helper without exporting the env toggles, treat it as a blocker and capture it in the session log.
