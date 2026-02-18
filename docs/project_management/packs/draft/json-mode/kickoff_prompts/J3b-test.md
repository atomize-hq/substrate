# Task J3b-test (JSON input – complex workflows) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J3b-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j3b-input-test
   git worktree add wt/cs-j3b-input-test cs-j3b-input-test
   cd wt/cs-j3b-input-test
   ```

## Spec (shared with J3b-code)
- Test JSON payloads for `world deps install/sync` (multi-tool lists, dry-run vs real),
  installer/helper commands, and config commands introduced elsewhere.
- Validate CLI flag override order, error handling, and that no partial writes occur on failure.
- Cover malformed payloads (missing tools, invalid booleans, conflict with CLI).

## Scope & Guardrails
- Tests/fixtures only; use temp HOMEs/manifests.
- Document installer smoke status (run vs skip).

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure tests pass; capture outputs/skips.
2. Commit worktree changes (e.g., `test: add json payload coverage for world deps`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and ensure J3b-integ prompt exists; commit docs (`git commit -am "docs: finish J3b-test"`).
5. Remove worktree and hand off.
