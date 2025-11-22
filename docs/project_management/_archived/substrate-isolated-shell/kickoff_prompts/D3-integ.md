# Task D3-integ – Integration Agent Kickoff Prompt

Task ID: **D3-integ** (Integrate world env toggle audit)

Summary / current state:
- D3-code added the shared host-only helper (`scripts/dev/substrate_shell_driver`) plus the Rust-side `substrate_shell_driver()` fixture (crates/shell/tests/common.rs). All shell integration suites (`integration.rs`, `world_enable.rs`, `world_deps.rs`, `shim_doctor.rs`, `shim_health.rs`) now call the helper and rely on the shared TMPDIR wiring.
- Host-only scripts now invoke the helper or explicitly export `SUBSTRATE_WORLD=disabled`/`SUBSTRATE_WORLD_ENABLED=0`: `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, `scripts/substrate/dev-shim-bootstrap.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, and `scripts/windows/dev-uninstall-substrate.ps1`. Python REPL harnesses continue to pass `--no-world` directly.
- Docs updated (`docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`) with guidance to call the helper; D3-test validated the helper workflow and the required commands listed below.

What you need to do:
1. From `feat/isolated-shell-plan`, create/switch to the integration worktree (`wt/d3-world-audit-integ`). Merge the code (`wt/d3-world-audit-code`) and test (`wt/d3-world-audit-test`) changes, resolving any conflicts in the files above plus planning docs.
2. Re-run the full command suite to ensure everything passes with the helper in place:
   - `cargo fmt --all`
   - `cargo test -p substrate-shell integration -- --nocapture`
   - `cargo test -p substrate-shell world_enable`
   - `cargo test -p substrate-shell world_deps`
   - `cargo test -p substrate-shell shim_doctor`
   - `cargo test -p substrate-shell --test shim_health -- --nocapture`
   - `SUBSTRATE_BIN=target/debug/substrate scripts/dev/substrate_shell_driver --no-world -c 'echo helper-ok'`
   - `BIN=target/debug/substrate scripts/validate_phase_d.sh`
   Expect the usual doctor_fixture dead-code warnings only.
3. Spot-check other scripts that intentionally keep isolation enabled (e.g., mac smoke or world doctor scripts). These still call `target/debug/substrate` directly by design—document any deviations if you discover scripts that should have switched to the helper but did not.
4. Update planning artifacts from the integration worktree: mark D3-code + D3-test as completed, set D3-integ to `in_progress`/`completed`, and log START/END entries that reference this prompt and the command outputs.
5. Ensure `scripts/dev/substrate_shell_driver` remains executable and that contributors know to set `SUBSTRATE_BIN` when invoking it from containers/CI (see Podman script for an example). Capture any regressions if the helper fails to honor `--no-world` or the env toggles.

Notes:
- Leave the existing helper-focused warnings alone unless they cause failures; the tests share fixtures that aren’t built in every binary.
- If any script or test still shells out directly without the helper or env overrides, treat it as a blocker and document in the session log for follow-up.
