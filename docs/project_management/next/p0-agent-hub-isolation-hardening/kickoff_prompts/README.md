# Kickoff Prompts (P0 Agent Hub Isolation Hardening)

Prompts live here with filenames referenced in `tasks.json` (e.g., `I0-code.md`).

Each prompt must include:
- Start checklist (feat/p0-agent-hub-isolation-hardening) with branch/worktree names.
- Spec reference (`I*-spec.md`) shared between code/test for the triad.
- Role guardrails (code: prod only; test: tests only; integration: merge/validate only).
- Required commands (code: fmt+clippy; test: fmt+targeted tests; integration: fmt+clippy+tests+make preflight).
- End checklist (merge back ff-only, update tasks/session log, remove worktree).

