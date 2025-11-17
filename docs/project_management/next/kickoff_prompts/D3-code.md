# Task D3-code – Code Agent Kickoff Prompt

Task ID: **D3-code** (Audit world env toggles across crates)

Summary:
- Track down every crate, binary, script, and test harness that launches `substrate` or depends on world enablement. The goal is to ensure each entry point uses a single shared helper (or the existing `ShellConfig`) so `SUBSTRATE_WORLD`, `SUBSTRATE_WORLD_ENABLED`, and `--no-world` overrides work consistently, preventing hangs/timeouts like the recent shell integration issue.
- Update the shared configuration helper and the call sites so tests and host-only flows can disable world provisioning simply by setting env vars. This includes shell integration tests, shim integration tests, installer/uninstaller harnesses, replay tools, world doctor/deps CLIs, and any scripts under `scripts/` or `tests/` that invoke `substrate` binaries.
- Document the helper expectation in `docs/CONFIGURATION.md`, the Workstream D plan/data map, and any scripts/readmes that mention world toggles. Leave breadcrumbs for future contributors (e.g., when writing new scripts/tests, they should call `substrate_shell_driver` or the helper instead of hard-coding PATH logic).

Work items / focus areas:
1. **Shared helper**: finalize where the helper lives. Today we have `ShellConfig::from_args` and the test-only `substrate-shell-driver`. Decide whether to expose a public API (`run_shell_with_cli` + helper) or create a small crate-level utility that other binaries/tests can call to respect `SUBSTRATE_WORLD`/`--no-world` automatically.
2. **Inventory**: search for `Command::new("substrate")`, `cargo run -p substrate`, manual PATH manipulation in tests (e.g., world_deps/world_enable fixtures, installer harness scripts, replay/doctor scripts). Maintain a checklist so nothing is missed.
3. **Apply helper**: update each call site to use the shared helper (or set `SUBSTRATE_WORLD=disabled` explicitly via the helper) instead of bespoke env juggling. Ensure integration tests set `SUBSTRATE_WORLD=disabled` once and never stall waiting for world agents.
4. **Docs/Data Map**: record the helper details and new expectations in `docs/CONFIGURATION.md`, `docs/project_management/next/substrate_isolated_shell_data_map.md`, and any relevant README/script comments.
5. **Validation**: run targeted checks (`cargo check --workspace`, plus any crate-specific tests touched). Leave deeper suite runs to the Test/Integration tasks but ensure your edits build.

Commands to run before handing off:
1. `cargo fmt --all`
2. `cargo check --workspace`
3. Any targeted tests needed to prove individual crates still compile (e.g., `cargo test -p substrate-shell integration -- --nocapture` if you touched the driver code or CLI wiring).

Artifacts to produce:
- Updated helper implementation + call sites.
- Documentation updates (config/data map/workplan).
- Checklist of audited crates/scripts (include in session log).
- Notes about which areas still need test coverage so D3-test can focus there.

Reminders:
- Work in `wt/d3-world-audit-code` and keep coordination artifacts untouched until you switch back to `feat/isolated-shell-plan`.
- Start by marking D3-code `in_progress` in `docs/project_management/next/tasks.json`, adding a START entry to `docs/project_management/next/session_log.md` referencing this prompt.
- Before coding, draft the Test Agent Kickoff Prompt for D3-test (include audit findings, helper details, commands). Record its path when you wrap up.
- Follow the Workstream workflow: after implementing changes and running the commands above, switch back to `feat/isolated-shell-plan`, append the END entry, update task status to `completed`, and log the Test Agent prompt path.

