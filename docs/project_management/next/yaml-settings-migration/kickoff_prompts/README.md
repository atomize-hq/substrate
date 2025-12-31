# Kickoff Prompts (YAML Settings Migration)

Prompts live here with filenames referenced in `tasks.json` (e.g., `Y0-code.md`).

Each prompt must include:
- Start checklist (feat/yaml-settings-migration) with branch/worktree names.
- Spec reference (`Y0-spec.md`) shared between code/test tasks.
- Role guardrails (code: prod only; test: tests only; integration: merge/validate only).
- Required commands (code: fmt+clippy; test: fmt+targeted tests; integration: fmt+clippy+tests+make integ-checks).
- End checklist (merge back ff-only, update tasks/session log, remove worktree).



Do not edit planning docs inside the worktree.
