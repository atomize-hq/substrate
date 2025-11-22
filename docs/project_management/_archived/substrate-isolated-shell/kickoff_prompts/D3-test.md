# Task D3-test – Test World Env Toggle Audit

## Summary
- Validate the new `substrate_shell_driver` helpers (Rust test support + `scripts/dev/substrate_shell_driver`) to ensure host-only flows enforce `SUBSTRATE_WORLD=disabled` / `SUBSTRATE_WORLD_ENABLED=0`.
- Confirm every entry point mentioned in the D3-code audit (shell integration suite, world_enable/world_deps tests, shim doctor/health fixtures, python REPL harnesses, `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, Linux/Windows dev shim scripts) either calls the helper or explicitly sets the env toggles / `--no-world`.
- Check documentation updates (`docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, D3 sections of the plan/session log) covering the helper guidance so contributors know to rely on `substrate_shell_driver` instead of invoking `target/debug/substrate` directly.
- Track any remaining gaps so D3-integ can follow up (e.g., isolation-enabled smoke scripts that intentionally keep worlds alive).

## Focus Files / Context
- `crates/shell/tests/common.rs` (`substrate_shell_driver()`), plus `tests/integration.rs`, `tests/world_enable.rs`, `tests/world_deps.rs`, `tests/shim_doctor.rs`, and `tests/shim_health.rs`.
- `scripts/dev/substrate_shell_driver` helper; scripts updated to use it / set env: `scripts/validate_phase_d.sh`, `scripts/podman/log_rotation_sanity.sh`, `scripts/substrate/dev-shim-bootstrap.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, `scripts/windows/dev-uninstall-substrate.ps1`.
- Python REPL harnesses in `scripts/dev/async_repl_*.py` (should continue passing `--no-world`).
- Docs: `docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, plus any README/snippet referencing world toggles.

## Required Commands
1. `cargo fmt --all`
2. `cargo test -p substrate-shell integration -- --nocapture`
3. `cargo test -p substrate-shell world_enable`
4. `cargo test -p substrate-shell world_deps`
5. `cargo test -p substrate-shell shim_doctor`
6. `cargo test -p substrate-shell --test shim_health -- --nocapture`
7. `SUBSTRATE_BIN=target/debug/substrate scripts/dev/substrate_shell_driver --no-world -c 'echo helper-ok'`
8. `BIN=target/debug/substrate scripts/validate_phase_d.sh`

## Reminders
- Work inside `wt/d3-world-audit-test`, mark D3-test as `in_progress` in `docs/project_management/next/tasks.json`, and log START/END entries referencing this prompt in `docs/project_management/next/session_log.md`.
- Compare the D3-code audit checklist (session log) against actual patches—flag any entry point still launching `substrate` directly without the helper/env exports.
- For scripts that cannot run in CI (Podman/WSL smoke helpers), validate that `substrate_shell_driver` is invoked with `SUBSTRATE_BIN` overrides and that comments point to the helper.
- Capture any gaps so D3-integ can address them (world smoke scripts that intentionally keep isolation enabled are acceptable but should be noted).
