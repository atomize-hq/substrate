# Task J2c-test (Structured output – graph & advanced) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2c-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2c-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2c-coverage-test
   git worktree add wt/cs-j2c-coverage-test cs-j2c-coverage-test
   cd wt/cs-j2c-coverage-test
   ```

## Spec (shared with J2c-code)
- Validate JSON envelopes for:
  - `substrate graph ingest/status/what-changed` (success + invalid span/path).
  - `substrate world deps install/sync` (tools present/missing, dry-run vs actual).
  - `substrate shim repair` (success + denied).
  - Any installer/helper/config commands touched in J2c-code.
- Confirm schema matches docs and errors remain structured.

## Scope & Guardrails
- Tests only; reuse fixtures for manifests/span logs.
- Document skipped scripts (e.g., installer smoke) with reasons.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure tests pass; capture outputs/skips.
2. Commit worktree changes (e.g., `test: add structured output coverage for graph/advanced commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and ensure J2c-integ prompt exists; commit docs (`git commit -am "docs: finish J2c-test"`).
5. Remove worktree and hand off.
