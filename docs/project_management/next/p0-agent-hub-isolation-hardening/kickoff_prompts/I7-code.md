# Task I7-code (Manual playbook alignment) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I7-spec.md`, and this prompt.
3. Set `I7-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I7-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i7-playbook-align-code
   git worktree add wt/ahih-i7-playbook-align-code ahih-i7-playbook-align-code
   cd wt/ahih-i7-playbook-align-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I7-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I7-spec.md`

## Scope & Guardrails
- Documentation changes only (no production Rust changes; no tests).
- Fix the manual testing playbook so it matches the real schema + I0–I5 specs:
  - Ensure `.substrate-profile` examples include `id` and `name`.
  - Remove/adjust any playbook claims that are not in I0–I5 specs.
  - Keep steps runnable and outcomes actionable (platform notes allowed).

## End Checklist
1. Commit worktree changes.
2. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
3. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I7-code`).
4. Remove worktree.
