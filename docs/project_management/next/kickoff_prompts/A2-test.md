# Task A2-test – Test Agent Kickoff Prompt

Task ID: **A2-test** (Test manager init module)

Summary:
- Exercise the new `crates/shell/src/manager_init.rs` module that loads the manager manifest, evaluates detection signals (files, commands, env vars, optional scripts), honors skip env vars, and emits snippets/telemetry.
- Validate `ManagerInitConfig::from_env` parsing for `SUBSTRATE_SKIP_MANAGER_INIT`, `SUBSTRATE_SKIP_MANAGER_INIT_LIST`, and `SUBSTRATE_MANAGER_INIT_DEBUG` plus case-insensitive handling.
- Cover detection success/failure across the helpers (`detect_files`, `detect_commands`, env match, script execution), skip-list behavior, and snippet assembly order/headers when managers are detected or skipped.
- Ensure `write_snippet` writes atomically (creating parent dirs) and `telemetry_payload` reports the expected shape; mock manifest paths using temp dirs/files.
- Shell wiring: `configure_manager_init` (`crates/shell/src/lib.rs`) now writes `~/.substrate/manager_init.sh`, respects overlay path `~/.substrate/manager_hooks.local.yaml`, accepts `SUBSTRATE_MANAGER_MANIFEST` override, logs telemetry, and exports `SUBSTRATE_MANAGER_INIT`. Add coverage around these helpers without touching coordination artifacts.

Focus files / context:
- `crates/shell/src/manager_init.rs`
- `crates/shell/src/lib.rs` (helper fns: `configure_manager_init`, `manager_manifest_base_path`, logging helpers)
- Planning docs: `docs/project_management/next/substrate_isolated_shell_plan.md`, `.../data_map.md`, `.../dependency_graph.md`

Commands to run:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell manager_init`
3. (Optional sanity) `cargo clippy -p substrate-shell -- -D warnings`

Reminders:
- Start from `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md` on the coordination branch, then work inside `wt/a2-manager-init-test`.
- Document any fixtures or helper scripts you add and capture the Integration Agent Kickoff Prompt when finished.
