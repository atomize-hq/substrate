# Task J3a-test (JSON input – simple commands) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J3a-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j3a-input-test
   git worktree add wt/cs-j3a-input-test cs-j3a-input-test
   cd wt/cs-j3a-input-test
   ```

## Spec (shared with J3a-code)
- Tests feed JSON payloads to `graph ingest`, `shim repair`, and the other simple command chosen in code task using:
  - Inline payload (`--json-payload`), file (`--json-input path`), and stdin (`--json-input -`).
- Validate CLI flag overrides and precedence behavior.
- Ensure malformed JSON/missing keys cause structured errors without touching disk.

## Scope & Guardrails
- Tests/fixtures only; use temp directories for manifests/config.
- Document skipped scripts if any.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure tests pass; capture outputs/skips.
2. Commit worktree changes (e.g., `test: add json input coverage for simple commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and ensure J3a-integ prompt exists; commit docs (`git commit -am "docs: finish J3a-test"`).
5. Remove worktree and hand off.
