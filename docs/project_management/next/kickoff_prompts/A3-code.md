# Task A3-code – Code Agent Kickoff Prompt

Task ID: **A3-code** (Implement per-session shell env injection)

Summary:
- Update the shell runtime so shims/managers are injected only for Substrate-managed processes while preserving the host login shell.
- Generate `manager_env.sh` that sources `SUBSTRATE_MANAGER_INIT` alongside the legacy bashenv snippets and ensure PTY bootstraps read it before user rc files.
- Wire the `--no-world` flag so the manager init path/export is skipped entirely during pass-through runs, and keep `manager_hooks.local.yaml` overlays working.

Focus files:
- `crates/shell/src/lib.rs`
- `crates/shell/src/async_repl.rs`
- `crates/shell/src/pty_exec.rs`
- `config/manager_hooks.yaml` (reference path helper / docs if behavior changes)

Commands to run:
1. `cargo fmt --all`
2. `cargo check -p substrate-shell`
3. `cargo test -p substrate-shell manager_init`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, update `docs/project_management/next/tasks.json` + `session_log.md`, and capture START/END entries on `feat/isolated-shell-plan`.
- Work inside `wt/a3-shell-env-code`, keep coordination artifacts off that worktree, and craft the Test Agent Kickoff Prompt for task `A3-test` before finishing.
- Review the isolated shell plan/data map docs plus `docs/project_management/next/kickoff_prompts/A2-integ.md` for the latest integrated context.
