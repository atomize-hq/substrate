# Task J3a-code (JSON input – simple commands) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J3a-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3a-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j3a-input-code
   git worktree add wt/cs-j3a-input-code cs-j3a-input-code
   cd wt/cs-j3a-input-code
   ```

## Spec (shared with J3a-test)
- Implement JSON payload loader/merging (file, stdin `-`, inline string) and ensure CLI flags override payload keys deterministically.
- Adopt JSON input in simple commands: `graph ingest`, `shim repair`, and at least one additional single-action command (e.g., `world doctor` fixture injection or config helper).
- Structured errors on malformed JSON/missing keys; no side effects.
- Docs updated with payload format, precedence rules, and examples.

## Scope & Guardrails
- Production code/docs only. Save complex workflows (world deps install/sync) for J3b.
- Maintain backward-compatible CLI flags.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure commands/tests pass; capture outputs.
2. Commit worktree changes (e.g., `feat: add json input plumbing for simple commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J3a-code"`). Reference J3a-test prompt.
5. Remove worktree and hand off.
