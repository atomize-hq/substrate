# Task C2-test – Test Agent Kickoff Prompt

Task ID: **C2-test** (Test `substrate world deps` CLI)

Summary:
- Validate the new `substrate world deps` subcommands added in C2-code. The tests should exercise `status`, `install`, and `sync` flows using temporary manifests and fake guest executors so we can assert host/guest detection, recipe invocation, `--all` behavior, `--verbose` streaming, and failure messaging. Cover both healthy runs and edge cases outlined in `docs/project_management/next/world-deps-sync.md`.
- Build a harness similar to `crates/shell/tests/world_enable.rs`: create temp HOME/PREFIX layouts, seed manifests (base + overlay), inject overrides via `SUBSTRATE_WORLD_DEPS_MANIFEST`, and fake the world backend by pointing the CLI at helper scripts or mock sockets/log files. Ensure tests can run offline without touching a real Lima/WSL VM.
- Confirm the CLI respects toggles: `--no-world`, `SUBSTRATE_WORLD=disabled`, missing manifests, unsupported providers, and dry-run/verbose flags. Tests should assert that `status` warns when the world is disabled but still reports host info, while `install`/`sync` return actionable errors in that mode.

Focus files / context:
- `crates/shell/tests/world_deps.rs` (new) plus any shared fixtures/helpers under `crates/shell/tests/`. You may also need lightweight mocks in `tests/common` to simulate the world backend (e.g., intercept calls to a helper script via `SUBSTRATE_WORLD_DEPS_EXECUTOR`).
- Coordinate with modules introduced by C2-code: manifest loader (likely in `crates/common`), CLI plumbing in `crates/shell/src/commands/world_deps.rs`, and any helper scripts placed under `scripts/substrate/`.
- Planning references: `docs/project_management/next/world-deps-sync.md`, `docs/project_management/next/substrate_isolated_shell_plan.md` (§4), and `docs/project_management/next/substrate_isolated_shell_data_map.md` (env vars for manifests + overrides).

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell world_deps`

Reminders:
- Start from `feat/isolated-shell-plan`: follow `AI_AGENT_START_HERE.md`, mark C2-test as `in_progress` in `docs/project_management/next/tasks.json`, and append a START entry to the session log before jumping into `wt/c2-world-deps-test`. Leave coordination files untouched in the test worktree.
- Consume the C2-code kickoff summary + implementation notes to understand flags, expected outputs, and testing hooks. Record any additional assumptions (e.g., helper env vars) for the Integration Agent when you finish.
- Keep tests deterministic: avoid relying on network access or real apt installs. Use stub scripts that append to log files / touch marker files to prove recipes ran. Validate stdout/stderr messages (status tables, install confirmations, error hints) and config mutations.
- Finish checklist mirrors prior tasks: (1) commit/push `wt/c2-world-deps-test`, (2) return to `feat/isolated-shell-plan`, (3) append END log entry + update task status, (4) add/link the Integration Agent Kickoff Prompt (`docs/project_management/next/kickoff_prompts/C2-integ.md` placeholder), (5) commit/push coordination updates.
