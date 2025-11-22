# Task A3-test – Test Agent Kickoff Prompt

Task ID: **A3-test** (Test shell env injection)

Summary:
- Validate the per-session shell environment changes: PATH/shims + manager snippets only apply to Substrate-managed processes while the host shell remains untouched.
- Cover generation + sourcing of the new `manager_env.sh` shim (sources `SUBSTRATE_MANAGER_INIT` and legacy `.substrate_bashenv`) and ensure PTY bootstrap reads it before any user rc scripts.
- Exercise the new `--no-world` flag / config path so pass-through runs skip shim + manager injection entirely while still executing commands.
- Keep `manager_hooks.local.yaml` overlays functioning (manifest override path + env) and ensure coordination between `SUBSTRATE_MANAGER_INIT` + `SUBSTRATE_MANAGER_ENV` exports.

Focus files / context:
- `crates/shell/src/lib.rs` (`ShellConfig`, session env builders, `configure_manager_init`, CLI flag plumbing)
- `crates/shell/src/async_repl.rs` (flag propagation, manager env wiring)
- `crates/shell/src/pty_exec.rs` (new PTY bootstrap sourcing order)
- `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../data_map.md`, `.../kickoff_prompts/A3-code.md` for background

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shell_env`

Reminders:
- Begin at `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md` on `feat/isolated-shell-plan`, then switch into `wt/a3-shell-env-test` for all code/test edits.
- Document any temporary HOME/dotfile fixtures and capture the Integration Agent Kickoff Prompt for `A3-integ` before finishing.
