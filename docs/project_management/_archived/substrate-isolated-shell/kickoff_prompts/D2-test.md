# Task D2-test – Test Agent Kickoff Prompt

Task ID: **D2-test** (Test doctor/health enhancements)

Summary:
- Consume the D2-code implementation + plan/data map references, then design automated coverage for the richer `substrate shim doctor` output and the new aggregated health command introduced in Workstream D2.
- Ensure tests validate both the text/table view and the JSON payload: manager states, last hint metadata, PATH diagnostics, config paths, and the stitched-in world health/deps results. Failures should emit clear diffs when expected vs. actual reports diverge.
- Exercise `--json`, `--verbose`, and failure scenarios (e.g., missing world agent socket, `world deps status` returning pending installs, manager init skipped). The fixtures should stub HOME, config, trace logs, and world-agent responses the same way earlier shim doctor tests simulate manager manifests.
- Provide harness helpers (probably under `crates/shell/tests/shim_doctor.rs` or a new `shim_health.rs`) so Integration can re-run the entire suite locally and CI can reuse the same entry point.

Focus files / context:
- `crates/shell/tests/shim_doctor.rs` (extend existing fixtures/assertions).
- New test module(s) for the aggregated `substrate health` command; reuse helpers from Workstreams B2/C2 for fake world doctor/deps results.
- Trace/log fixtures under `tests/fixtures/` if needed for hint records.
- Documentation cross-checks (`docs/USAGE.md`, `docs/CONFIGURATION.md`) to make sure described outputs line up with assertions.

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shim_doctor`
3. `cargo test -p substrate-shell --test shim_health -- --nocapture` (adjust test binary/filter to match the new module name)

Reminders:
- Work inside `wt/d2-health-test`, set D2-test status to `in_progress` in `docs/project_management/next/tasks.json`, and log START/END entries referencing this prompt path (`docs/project_management/next/kickoff_prompts/D2-test.md`).
- Coordinate expectations with the Code agent via the session log: required env vars (`SUBSTRATE_MANAGER_INIT`, `SUBSTRATE_WORLD_SOCKET`, `SUBSTRATE_WORLD_ENABLE_SCRIPT`, etc.), fake world doctor responses, and any helper scripts.
- Before handing off to Integration, document the exact `cargo test` commands plus any sample CLI runs so the next agent can reproduce the reports verbatim.
