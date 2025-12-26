# Task I8-code (I1 noise reduction) – CODE

## Start Checklist (feat/p0-agent-hub-isolation-hardening)
1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `I8-spec.md`, and this prompt.
3. Set `I8-code` to `in_progress`, append START entry to `session_log.md`, commit docs (`docs: start I8-code`).
4. Create branch/worktree:
   ```
   git checkout -b ahih-i8-i1-noise-code
   git worktree add wt/ahih-i8-i1-noise-code ahih-i8-i1-noise-code
   cd wt/ahih-i8-i1-noise-code
   ```
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Spec (shared with I8-test)
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I8-spec.md`

## Scope & Guardrails
- Production code only (no tests).
- Reduce world-routing noise to meet I8/I1 requirements:
  - Exactly one warning on fallback-allowed runs when world is unavailable.
  - Exactly one error on required-world runs when world is unavailable.
- Do not weaken enforcement semantics; keep actionable hints.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```

## End Checklist
1. Confirm required checks are green; capture outputs for log.
2. Commit worktree changes.
3. Merge back to `feat/p0-agent-hub-isolation-hardening` (ff-only).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs (`docs: finish I8-code`).
5. Remove worktree.
