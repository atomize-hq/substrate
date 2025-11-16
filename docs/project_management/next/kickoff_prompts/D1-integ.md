# Task D1-integ – Integration Agent Kickoff Prompt

Task ID: **D1-integ** (Integrate Tier-2 managers)

Summary / current state:
- Code worktree `wt/d1-managers-code` should carry the refreshed `config/manager_hooks.yaml` with mise/rtx, rbenv, sdkman, bun, volta, goenv, and the other Tier-2 managers wired up (detect/env/script blocks plus repair + guest metadata). Tests expect those names + hints to exist in the shipping manifest once merged.
- Test worktree `wt/d1-managers-test` adds coverage in three areas:
  - `crates/common/src/manager_manifest.rs`: `tier2_managers_include_complete_metadata` loads a manifest containing all new managers and asserts env/tilde expansion, repair hints, and guest install metadata (apt/custom) are preserved. Keep those schema fields synchronized with the production manifest.
  - Shell: new integration test file `crates/shell/tests/manager_init.rs` plus an expanded `shell_env_injects_manager_snippets` scenario ensure tier-2 managers produce snippet blocks and that `manager_env.sh` still sources `SUBSTRATE_MANAGER_INIT` + legacy `.substrate_bashenv`. `crates/shell/tests/shim_doctor.rs` now checks JSON output to verify Bun/Volta states and recorded hints.
  - Shim: `crates/shim/tests/integration.rs` gained `tier2_manager_hint_logging_records_entry` so bun failures emit `manager_hint` records when hints are enabled.
- All commands below were run successfully from `wt/d1-managers-test`, but `cargo test -p substrate-shell manager_init` filters by the substring “manager_init” and therefore skips the new integration test target. Run the additional `--test manager_init` invocation so the new file executes.

What you need to do:
1. From `feat/isolated-shell-plan`, create or switch into `wt/d1-managers-integ` and merge both `wt/d1-managers-code` and `wt/d1-managers-test`. Resolve conflicts in `crates/common/src/manager_manifest.rs`, `config/manager_hooks.yaml`, `crates/shell/tests/*`, and `crates/shim/tests/integration.rs` so the tests reference the exact manifest entries that land in code.
2. Ensure the manifest actually contains the Tier-2 entries referenced by the tests (names + repair hints + guest metadata). Update either side as needed so the tests’ expectations (env expansion paths, hint strings, install recipes) match the manifest contents that will ship.
3. Run the verification commands from the integrated worktree:
   - `cargo fmt --all`
   - `cargo test -p substrate-common manager_manifest`
   - `cargo test -p substrate-shell manager_init`
   - `cargo test -p substrate-shell --test manager_init` (executes the new integration test binary; the filtered command above will not)
   - `cargo test -p substrate-shim`
   Capture at least one run of each command; if the manifest/code merge is still in progress, keep the stderr so the next agent understands any remaining failures.
4. Optional but recommended: manually inspect `~/.substrate/manager_init.sh` / `manager_env.sh` emitted by `shell_env_injects_manager_snippets` (or `substrate -c 'env'` in a temp HOME) to confirm the additional managers show up ahead of legacy bashenv content.

Finish checklist:
1. Commit/push the merged results inside `wt/d1-managers-integ` after the commands above succeed.
2. Return to `feat/isolated-shell-plan`, log the END entry for D1-integ, update `docs/project_management/next/tasks.json`, and reference this prompt path plus the executed commands in `docs/project_management/next/session_log.md`.
3. Leave the tree ready for Phase D2 (doctor/health) by keeping manifests/tests in sync and documenting any deviations in the session log.
