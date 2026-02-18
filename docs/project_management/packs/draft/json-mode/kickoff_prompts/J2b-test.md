# Task J2b-test (Structured output – world & shim) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2b-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2b-coverage-test
   git worktree add wt/cs-j2b-coverage-test cs-j2b-coverage-test
   cd wt/cs-j2b-coverage-test
   ```

## Spec (shared with J2b-code)
- Validate JSON envelopes for:
  - `substrate world doctor --json` (platform fixtures).
  - `substrate world enable` (success + already enabled).
  - `substrate world deps status/install/sync` (with world enabled/disabled).
  - `substrate shim doctor --json` and `substrate shim repair --json`.
  - `substrate health --json`.
- Cover failure cases: missing manifest, repair denied, world disabled, etc.

## Scope & Guardrails
- Tests only. Use hermetic HOMEs/manifests to simulate states.
- Document any skipped scripts (installer smoke) with reasons.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure tests pass; capture outputs/skips.
2. Commit worktree changes (e.g., `test: add structured output coverage for world/shim`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and ensure J2b-integ prompt exists; commit docs (`git commit -am "docs: finish J2b-test"`).
5. Remove worktree and hand off.
