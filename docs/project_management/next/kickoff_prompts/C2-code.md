# Task C2-code – Code Agent Kickoff Prompt

Task ID: **C2-code** (Implement `substrate world deps` CLI)

Summary:
- Add the `substrate world deps` command group with `status`, `install`, and `sync` actions that read a manifest of host/guest tool definitions, detect what’s available on the host vs. inside the world backend, and run the installer snippets in the guest when requested. Follow the workflow outlined in `docs/project_management/next/world-deps-sync.md` and the execution plan (§4 "World Dependency Sync").
- Reuse the manifest/overlay concepts introduced for manager init: ship a base manifest in the repo (e.g., `scripts/substrate/world-deps.yaml`), support a user-local override (e.g., `~/.substrate/world-deps.local.yaml`), and honor `SUBSTRATE_WORLD_DEPS_MANIFEST` for tests. The CLI should surface clear status output (host command found?, guest command found?, recipe provider) and actionable follow-ups when installs fail.
- Hook the CLI into the existing world plumbing. `status` should work even if the world is disabled (printing a warning and exiting non-zero or degraded), while `install`/`sync` should bail when `--no-world`/`SUBSTRATE_WORLD=disabled` is set. When the world is active, reuse the same world backend API (`world-agent` / `world-backend-factory`) to run the install recipes inside the guest, stream logs (especially in `--verbose` mode), and wait for completion/errors.
- Ensure installer integration is ready: expose a helper that `scripts/substrate/install-substrate.sh --sync-deps` can call to kick off `substrate world deps sync --all` (or at least print the reminder) once shims/world are provisioned.

Focus files / context:
- `crates/shell/src/lib.rs`, `crates/shell/src/commands/mod.rs`, and a new module like `crates/shell/src/commands/world_deps.rs` for CLI argument parsing, manifest helpers, host/guest detection, and recipe execution.
- Manifest & helper utilities: reuse `crates/common` (possibly adding a new `deps_manifest` module alongside `manager_manifest`). Respect the data-map docs for `SUBSTRATE_WORLD_DEPS_MANIFEST`, `SUBSTRATE_HOME`, etc.
- `world-backend-factory`, `world-agent`, or helper structs that already encapsulate guest execution; you may need lightweight wrappers to run commands inside Lima/WSL via `/v1/execute` with root privileges.
- Planning docs: `docs/project_management/next/substrate_isolated_shell_plan.md` (§4, §5.5), `docs/project_management/next/substrate_isolated_shell_data_map.md` (env + config fields), `docs/project_management/next/world-deps-sync.md`, and `docs/project_management/next/substrate_isolated_shell_execution_plan.md` (Phase C milestones).

Commands to run:
1. `cargo fmt --all`
2. `cargo check -p substrate-shell`

Reminders:
- Begin on `feat/isolated-shell-plan`: read `AI_AGENT_START_HERE.md`, flip C2-code to `in_progress` in `docs/project_management/next/tasks.json`, and append a START entry in `session_log.md` before editing. Do all code work inside `wt/c2-world-deps-code`; don’t touch coordination artifacts from that worktree.
- Capture nuances for the Test Agent: manifest format, expected CLI output, guest command invocation approach, how to simulate host/guest availability, any TODOs you’re punting on. Record this prompt path in the session log when you finish.
- Handle edge cases: missing manifest entries, unsupported providers, users running with `--no-world`, guest command failures, partial installs, and dry-run/verbose output. The CLI should clearly report which tools are missing and what command it ran (or would run) to fix them.
- Finish checklist: (1) commit/push `wt/c2-world-deps-code`, (2) switch back to `feat/isolated-shell-plan`, (3) append END log entry + set task status to `completed`, (4) add/link the Test Agent Kickoff Prompt, (5) commit/push coordination file updates.
