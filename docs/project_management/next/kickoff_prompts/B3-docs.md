# Task B3-docs – Code Agent Kickoff Prompt

Task ID: **B3-docs** (Update documentation and configuration references)

Summary:
- Refresh every public-facing guide so they describe the pass-through shim model, manager auto-init, shim doctor/repair CLI, and the upcoming `world enable/world deps` workflow (at least at the conceptual level).
- Document `config/manager_hooks.yaml`, overlay support, and all new env vars/flags (`SUBSTRATE_SKIP_MANAGER_INIT`, `SUBSTRATE_MANAGER_INIT_SHELL`, `SUBSTRATE_MANAGER_MANIFEST`, `--no-world`, etc.) using the data map + execution plan as the source of truth.
- Update onboarding docs (README/INSTALLATION/USAGE) with clear steps for installing, running doctor/repair, and verifying PATH isolation; ensure diagrams or callouts explain that host dotfiles remain untouched.
- Keep CHANGELOG/release notes in sync and add breadcrumbs back to the plan so later phases know where these instructions live.

Focus files / context:
- `README.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`, `docs/CONFIGURATION.md`
- Any supporting references listed in the file audit (plan, data map, file audit itself, AI agent onboarding docs)
- Recent implementation notes from `docs/project_management/next/substrate_isolated_shell_plan.md` + `.../data_map.md`

Commands to run:
1. `cargo fmt --all` (parity check even though this task is docs-only)
2. `npx markdownlint-cli README.md docs/INSTALLATION.md docs/USAGE.md docs/CONFIGURATION.md docs/project_management/next/substrate_isolated_shell_plan.md docs/project_management/next/substrate_isolated_shell_data_map.md`

Reminders:
- Start at `AI_AGENT_START_HERE.md`, mark this task as `in_progress` in `docs/project_management/next/tasks.json`, and add a START entry in `docs/project_management/next/session_log.md` before editing.
- Work inside git worktree `wt/b3-docs`; leave coordination files alone there except for logging the new kickoff references when you return to `feat/isolated-shell-plan`.
- Ensure each doc clearly states how to use `substrate shim doctor`, what `--no-world` does, how manifests/overlays are located, and how to point CLI/testing at temporary HOME directories.
- Capture any follow-up questions (e.g., pending world CLI behavior) in the session log so later phases can resolve them, and remember to link the Integration Agent to this prompt once the task is complete.
