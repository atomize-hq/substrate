# Task D2-code – Code Agent Kickoff Prompt

Task ID: **D2-code** (Enhance doctor/health reporting)

Summary:
- Extend the existing `substrate shim doctor` functionality per Workstream D2 in `docs/project_management/next/substrate_isolated_shell_execution_plan.md` (§Phase D) and the data map entries for `ShimDoctorReport`. The doctor output should aggregate manager manifest results, the latest hint telemetry, and world backend health so that a single command surfaces host + guest readiness.
- Add a consolidated health command (e.g., `substrate health` or `substrate doctor`) that wraps `shim doctor`, `world doctor`, and `world deps status` checks. Emit a structured JSON payload (matching `docs/project_management/next/substrate_isolated_shell_data_map.md`) plus a human-readable summary that highlights actionable failures.
- Update CLI help, docs (`docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/INSTALLATION.md`, and any troubleshooting guides) so users know how to capture health snapshots (`substrate shim doctor --json`, `substrate health --json`, etc.) for support bundles.
- Ensure the new command respects pass-through env toggles (`SUBSTRATE_WORLD_ENABLED`, `SUBSTRATE_MANAGER_INIT_DEBUG`, etc.) and reads config metadata produced by the installer work (C3). Instrument code paths so integration/tests can stub HOME/config with the same fixtures used in `crates/shell/tests/shim_doctor.rs`.

Focus files / context:
- `crates/shell/src/commands/shim_doctor.rs` (extend structs/output, add aggregation helpers).
- `crates/shell/src/lib.rs` and new CLI module(s) for the top-level health command.
- `crates/shell/tests/shim_doctor.rs` + new tests covering aggregated output.
- Documentation + release notes: `docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/INSTALLATION.md`, `CHANGELOG.md`.
- Reference data: `docs/project_management/next/substrate_isolated_shell_plan.md` (§Workstream D2) and `docs/project_management/next/substrate_isolated_shell_data_map.md`.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shim_doctor`
3. `cargo test -p substrate-shell --test shim_health` (or whichever new integration test module exercises the aggregated command)
4. Capture a sample report via `target/debug/substrate shim doctor --json` and the new health command (text + `--json`) using a temp HOME so logs can mention concrete output.

Reminders:
- Work out of `wt/d2-health-code`, set task status to `in_progress` in `docs/project_management/next/tasks.json`, and append START/END entries to the session log before/after coding.
- Coordinate with the Test agent by documenting fixture requirements, env toggles, and any tricky platform-specific paths (e.g., `world doctor` timeouts, `SUBSTRATE_WORLD_SOCKET` overrides) inside the session log END entry.
- Surface any schema additions in `docs/TRACE.md` / `docs/CONFIGURATION.md` if the JSON payload changes, and keep the plan/data map updated when introducing new fields.
