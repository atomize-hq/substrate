# Task J1-test (Structured mode scaffold) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J1-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J1-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j1-structured-test
   git worktree add wt/cs-j1-structured-test cs-j1-structured-test
   cd wt/cs-j1-structured-test
   ```

## Spec (shared with J1-code)
- Tests drive the new `--json`, `--json-input`, and `--json-payload` flags:
  - Non-interactive commands return JSON envelopes with status/message/data.
  - Failure paths produce structured errors.
  - Interactive REPL invoked with `--json` yields a friendly rejection.
- Validate placeholder JSON input parsing by feeding sample payloads and ensuring they reach a stub handler (even if unused by commands yet).

## Scope & Guardrails
- Test files/fixtures only. Coordinate via shell driver/fixtures.
- Keep tests hermetic using SUBSTRATE_HOME/TMPDIR overrides.
- Document skips (e.g., installer scripts) with rationale.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure commands/tests pass; log outputs/skips.
2. Commit worktree changes (e.g., `test: cover structured mode flags`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and author the J1-integ
   kickoff prompt if missing; commit docs (`git commit -am "docs: finish J1-test"`).
5. Remove worktree and hand off.
