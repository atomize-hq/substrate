# Kickoff Prompts (Crate Refactor)

Place per-task kickoff prompts in this directory using the filenames referenced
in `tasks.json` (e.g., `R1-code.md`). Each prompt should include:
- Start checklist (feat/crate-refactor) with branch/worktree names for the task.
- Scope, deliverables, and any guardrails/commands unique to the task.
- Suggested commands/tests to run.
- End checklist and required artifacts.

Guardrails for prompts:
- Code and test prompts must carry the **exact same spec**. Code agents must not
  add or modify tests; test agents write tests based solely on the spec (they
  will not rely on the code branch state).
- Integration prompts must restate that their job is to merge both code and test
  branches into the integration branch/worktree, resolve misalignments, and
  ensure combined behavior matches the spec before updating docs/tasks/logs on
  `feat/crate-refactor`.

Follow the format used in `docs/project_management/next/settings-stack/kickoff_prompts/`.
