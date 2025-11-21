# Task A1-test – Test Agent Kickoff Prompt

Task ID: **A1-test** (Test manager manifest parser)

Summary:
- Validate the new `crates/common/src/manager_manifest.rs` module that loads `config/manager_hooks.yaml` plus an optional overlay (`~/.substrate/manager_hooks.local.yaml`).
- Ensure env/tilde expansion on detect file paths and `detect.env` values works as expected, including overlay path handling for `~/.substrate/manager_hooks.local.yaml`.
- Confirm overlay entries override or extend the base manifest correctly and that managers are sorted by `priority` (ascending) then name.
- Exercise validation paths: duplicate manager names, invalid regex patterns under `errors`, missing manifest files/keys.
- Cover platform-aware helpers such as `ManagerManifest::resolve_for_platform` (ensuring POSIX sessions only expose `init.shell` while Windows keeps `init.powershell`).

Focus files / context:
- `crates/common/src/manager_manifest.rs` (new module)
- `crates/common/src/lib.rs` (re-export and any helper wiring)
- Reference docs: `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../data_map.md`, `.../dependency_graph.md`

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-common manager_manifest`
3. `cargo clippy -p substrate-common -- -D warnings` (if time allows, to keep the crate tidy)

Reminders:
- Start from `AI_AGENT_START_HERE.md`, follow the logging + task workflow, and update `docs/project_management/next/session_log.md` for START/END entries.
- Work inside the `wt/a1-manifest-test` worktree when implementing tests.
- Document any new fixtures or helper files you add.

Please keep this prompt linked in the session log once tests are implemented so future agents have traceability.
