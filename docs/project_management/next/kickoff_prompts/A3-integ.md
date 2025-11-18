# Task A3-integ – Integration Agent Kickoff Prompt

Task ID: **A3-integ** (Integrate shell env injection)

Summary:
- Merge the env-injection code (`wt/a3-shell-env-code`) with the new regression tests (`wt/a3-shell-env-test`).
- Tests validate that only Substrate-managed processes receive PATH shim + manager snippets, that `manager_env.sh` chains `SUBSTRATE_MANAGER_INIT` + legacy `.substrate_bashenv`, that `--no-world` skips all of that, and that user overlays (`manager_hooks.local.yaml`) still override the manifest.
- Each test fabricates its own HOME under `target/tests-tmp/substrate-test-*`, drops custom `manager_hooks.yaml`, `.substrate_bashenv`, and `host_bash_env.sh`, then runs `substrate -c ...` with `SUBSTRATE_WORLD=disabled` so no real world provisioning is needed.

Focus files / context:
- `crates/shell/tests/integration.rs` (new helper fixture + three `shell_env_*` tests)
- Existing env code in `crates/shell/src/lib.rs`, `crates/shell/src/pty_exec.rs`, and PTY bootstrap scripts from A3-code
- Docs: `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../data_map.md` for reference if behavior needs re-checking

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shell_env`

Reminders:
- Begin at `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md` on `feat/isolated-shell-plan`, then use worktree `wt/a3-shell-env-integ` to stitch everything together.
- Keep the temporary HOME fixtures confined to the `target/tests-tmp` sandbox; no real HOME files should be touched.
- Verify that the `shell_env_*` tests pass alongside the existing suite and that PATH/manager env behavior matches expectations when run manually if needed.
