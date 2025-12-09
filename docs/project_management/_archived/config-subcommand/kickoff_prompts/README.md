# Kickoff Prompts (Config Subcommand)

Store per-task kickoff prompts here using the filenames referenced in
`tasks.json` (e.g., `C1-code.md`). Each prompt must include:
- Start checklist (feat/config-subcommand) with explicit branch/worktree names.
- Scope/acceptance criteria mirrored between code/test counterparts.
- Required commands/tests/scripts and guardrails for the task.
- End checklist plus artifacts to capture (commits, prompts, logs).

Guardrails:
- Code vs test prompts **must describe the exact same spec** so the two roles
  work independently. Code agents avoid editing tests; test agents derive tests
  solely from the prompt.
- Integration prompts must focus on merging the paired branches/worktrees,
  reconciling conflicts, running fmt/clippy/tests, and updating docs/tasks/logs
  on `feat/config-subcommand`.
