# Task D3-test – Test Agent Kickoff Prompt

Task ID: **D3-test** (Test world env toggle audit)

Summary:
- Validate the new `substrate_shell_driver` helpers (Rust test support + `scripts/dev/substrate_shell_driver`) that guarantee `SUBSTRATE_WORLD=disabled` / `SUBSTRATE_WORLD_ENABLED=0` for host-only flows. Ensure shell/shim integration tests, world_enable/world_deps fixtures, and developer scripts rely on the helper instead of duplicating binary lookup logic.
- Confirm every entry point called out in the D3-code audit honors the toggles: shell integration suite, world_enable/world_deps tests, shim doctor/health fixtures, the python REPL harnesses, `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, and the dev shim bootstrap/uninstall scripts (Linux + Windows). These should either invoke the driver script or explicitly set the env vars / `--no-world`.
- Check documentation updates (`docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, D3 sections of the plan/session log) for the new helper guidance so future contributors know to call `substrate_shell_driver` instead of reaching for `target/debug/substrate` directly.
- Capture any remaining gaps so D3-integ can chase them (e.g., world-specific smoke scripts that intentionally keep isolation enabled).

Focus files / context:
- `crates/shell/tests/common.rs` (new `substrate_shell_driver()`), plus the tests that switched to it: `tests/integration.rs`, `tests/world_enable.rs`, `tests/world_deps.rs`, `tests/shim_doctor.rs`, and `tests/shim_health.rs`.
- `scripts/dev/substrate_shell_driver` (new executable wrapper) and the scripts updated to use it / set env: `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, `scripts/substrate/dev-shim-bootstrap.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, and `scripts/windows/dev-uninstall-substrate.ps1`.
- Python REPL harnesses under `scripts/dev/async_repl_*.py` should keep passing `--no-world`; ensure the helper guidance matches their usage.
- Docs: `docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, plus any README/snippet referencing world toggles.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell integration -- --nocapture`
3. `cargo test -p substrate-shell world_enable`
4. `cargo test -p substrate-shell world_deps`
5. `cargo test -p substrate-shell shim_doctor`
6. `cargo test -p substrate-shell --test shim_health -- --nocapture`
7. `SUBSTRATE_BIN=target/debug/substrate scripts/dev/substrate_shell_driver --no-world -c 'echo helper-ok'`
8. `BIN=target/debug/substrate scripts/validate_phase_d.sh` (verifies the script uses the helper without hanging on world provisioning)

Reminders:
- Work inside `wt/d3-world-audit-test`, set D3-test status to `in_progress` in `docs/project_management/next/tasks.json`, and log START/END entries referencing this prompt (`docs/project_management/next/kickoff_prompts/D3-test.md`).
- Compare the D3-code audit checklist (session log) against actual patches—if any entry point still launches `substrate` directly without the helper/env exports, flag it as a blocker.
- For scripts that cannot run in CI (e.g., Podman or WSL smoke helpers), at least validate that `substrate_shell_driver` is invoked with `SUBSTRATE_BIN` overrides and that comments point future contributors to the helper.
