# Task B2-code – Code Agent Kickoff Prompt

Task ID: **B2-code** (Implement shim doctor/repair CLI)

Summary:
- Extend `substrate`'s CLI to expose `substrate shim doctor` (text + `--json`) so operators can inspect manager manifests, current PATH state, and pending repair hints directly from the host.
- Add `substrate shim repair --manager <name>` that looks up the manager manifest entry, writes/updates the appropriate repair snippet in `~/.substrate_bashenv`, and logs telemetry about any file mutations (with backup before overwrite).
- Thread new CLI wiring through the existing manager init/manifest plumbing so doctor/repair share the manifest loader and honor `SUBSTRATE_MANAGER_MANIFEST` overrides.

Focus files / context:
- `crates/shell/src/lib.rs`, `crates/shell/src/commands` (command registration, argument parsing, output formatting)
- Manager manifest + init helpers from `crates/common/src/manager_manifest.rs` and `crates/shell/src/manager_init.rs`
- Planning docs in `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../substrate_isolated_shell_data_map.md`, and `.../execution_plan.md`

Commands to run:
1. `cargo fmt --all`
2. `cargo check -p substrate-shell`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, update `docs/project_management/next/tasks.json` + `session_log.md` on `feat/isolated-shell-plan`, and record START/END entries before/after coding.
- Work entirely inside `wt/b2-doctor-code`; leave coordination artifacts untouched in that worktree.
- Capture any CLI usage notes or new env toggles in the docs referenced above, and craft the Test Agent Kickoff Prompt for `B2-test` before wrapping up.
