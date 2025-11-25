# Task J2a-code (Structured output – core commands) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2a-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2a-coverage-code
   git worktree add wt/cs-j2a-coverage-code cs-j2a-coverage-code
   cd wt/cs-j2a-coverage-code
   ```

## Spec (shared with J2a-test)
- Commands in scope: root `substrate -c/-f`, `--version-json`, `--shim-status[ -json]`,
  `--shim-deploy/remove`, `--trace`, `--replay`, `--health`, and global error exits.
- When `--json` (or legacy aliases) is active:
  - Success responses use the shared envelope (`status`,`message`,`data`).
  - Errors emit structured envelopes with details.
- Legacy JSON flags map to the new global flag without regression.
- Docs updated with schema summaries for these commands plus alias behavior.

## Scope & Guardrails
- Production code/docs only; tests handled by J2a-test.
- Preserve text-mode behavior when `--json` is absent.
- Avoid touching world/shim/graph commands (covered in later phases).

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure commands/tests pass; capture outputs.
2. Commit worktree changes (e.g., `feat: add structured output for core commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J2a-code"`). Reference J2a-test prompt.
5. Remove worktree and hand off.
