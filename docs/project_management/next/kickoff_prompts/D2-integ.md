# Task D2-integ – Integration Agent Kickoff Prompt

Task ID: **D2-integ** (Integrate doctor/health reporting)

Summary / current state:
- Code worktree `wt/d2-health-code` adds the aggregated `substrate health` command plus richer `substrate shim doctor` output (world doctor snapshots, world-deps summary, hint metadata plumbing, docs/CHANGELOG updates, etc.).
- Test worktree `wt/d2-health-test` extends the doctor fixtures + adds `crates/shell/tests/shim_health.rs` to cover the new JSON/human summaries, failure modes (invalid fixtures, skip-manager env, missing guest tools), and the text doctor output when world sections fail. Tests rely on temp HOMEs under `target/tests-tmp` and seed `~/.substrate/health/{world_doctor.json,world_deps.json}` fixtures as part of `DoctorFixture`.
- Both worktrees expect the shared commands below to pass; no code/test changes have been merged back to `feat/isolated-shell-plan` yet.

What you need to do:
1. From `feat/isolated-shell-plan`, create/switch to `wt/d2-health-integ`, merge `wt/d2-health-code` + `wt/d2-health-test`, and resolve overlaps in `crates/shell/tests/common.rs`, `crates/shell/tests/shim_doctor.rs`, `crates/shell/tests/shim_health.rs`, CLI wiring (`crates/shell/src/commands/*.rs`, `crates/shell/src/lib.rs`), docs, and CHANGELOG. Keep the health fixtures + helper struct identical between tests.
2. Run the required commands from the integration worktree root:
   - `cargo fmt --all`
   - `cargo test -p substrate-shell shim_doctor`
   - `cargo test -p substrate-shell --test shim_health -- --nocapture`
   These cover both the doctor regressions and the new health binary; they also ensure the world fixture helpers remain deterministic.
3. (Optional but recommended) Capture a sample CLI run for reference: `target/debug/substrate shim doctor --json` and `target/debug/substrate health --json` using a temp HOME (set `TMPDIR`/`HOME` like the tests do) so docs/examples match the aggregated payload.
4. Verify docs (`docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/INSTALLATION.md`, plan/data-map entries) align with the merged behavior. Update wording if conflicts arise during merge.

Notes:
- The fixtures live under `$HOME/.substrate/health/` inside each test HOME; deleting or corrupting them intentionally simulates missing world doctor/deps responses (tests assert the error messaging). Keep that behavior intact so CI doesn’t accidentally reach for real world agents.
- The helper sets `SUBSTRATE_WORLD=disabled` by default; the health command now takes `--json` only, so there is no `--verbose` flag to exercise.

Finish checklist:
1. Once the integration worktree is green, merge/push back to `feat/isolated-shell-plan`.
2. While on the coordination worktree, append START/END entries for D2-integ to `docs/project_management/next/session_log.md`, update `docs/project_management/next/tasks.json` (mark D2-code and D2-test completed if not already, set D2-integ accordingly), and reference this prompt plus the exact commands above.
3. Ensure any captured sample outputs/logs are linked in the session log or PR description so downstream reviewers can reproduce the doctor/health snapshots.
