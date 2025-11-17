# Task D3-integ – Integration Agent Kickoff Prompt

Task ID: **D3-integ** (Integrate world env toggle audit)

Summary / current state:
- D3-code is supposed to ship the shared `substrate_shell_driver` helper, wire tests/scripts through it, and document the toggles; however, neither the helper nor the script updates exist in `feat/isolated-shell-plan` yet.
- The D3-test worktree (`wt/d3-world-audit-test`) attempted to run the expected suites and script checks but was blocked immediately because `scripts/dev/substrate_shell_driver` is missing and every entry point still shells out to `target/.../substrate` directly.
- Until D3-code actually lands the helper + script/doc updates, there is nothing to integrate—the current repo state does not honor the SUBSTRATE_WORLD toggles highlighted in the D3 audit.

What you need to do once the helper exists:
1. Coordinate with the D3-code/D3-test owners (worktrees `wt/d3-world-audit-code` and `wt/d3-world-audit-test`) to ensure the following artifacts are present before merging:
   - `scripts/dev/substrate_shell_driver` plus any Rust test helper (`substrate_shell_driver()` in `crates/shell/tests/common.rs`).
   - Shell/shim/world integration tests switched to the helper (integration.rs, world_enable.rs, world_deps.rs, shim_doctor.rs, shim_health.rs).
   - Scripts updated to use the helper or explicitly export `SUBSTRATE_WORLD=disabled` / `SUBSTRATE_WORLD_ENABLED=0`: `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, `scripts/substrate/dev-shim-bootstrap.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, `scripts/windows/dev-uninstall-substrate.ps1`, and the Python REPL harness docs.
   - Docs refreshed (`docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, session log) to mention the helper.
2. After verifying the helper is wired up, create/switch to `wt/d3-world-audit-integ`, merge the code + test worktrees, resolve conflicts, and run these commands from the integration worktree root:
   - `cargo fmt --all`
   - `cargo test -p substrate-shell integration -- --nocapture`
   - `cargo test -p substrate-shell world_enable`
   - `cargo test -p substrate-shell world_deps`
   - `cargo test -p substrate-shell shim_doctor`
   - `cargo test -p substrate-shell --test shim_health -- --nocapture`
   - `SUBSTRATE_BIN=target/debug/substrate scripts/dev/substrate_shell_driver --no-world -c 'echo helper-ok'`
   - `BIN=target/debug/substrate scripts/validate_phase_d.sh`
   Capture any failures as blockers if the helper does not fully disable world provisioning.
3. Double-check out-of-scope scripts (Podman, Windows) that you cannot run locally to ensure they at least call the helper with `SUBSTRATE_BIN` overrides or set the env vars explicitly, and add comments if needed.
4. Update planning artifacts from the integration worktree: set D3-code/D3-test to `completed` (once unblocked) and D3-integ to `in_progress`/`completed`, append START/END entries referencing this prompt, and summarize command outputs or blockers.

Notes:
- If D3-code still has not delivered the helper, keep D3-test and D3-integ blocked; document the missing artifacts in the session log so stakeholders know why integration cannot proceed.
- The helper must guarantee `SUBSTRATE_WORLD=disabled` / `SUBSTRATE_WORLD_ENABLED=0` before launching the CLI; scripts/tests should rely on the helper instead of duplicating binary lookup logic.
