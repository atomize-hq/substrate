# Task J2a-test (Structured output – core commands) – TEST

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2a-test` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2a-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2a-coverage-test
   git worktree add wt/cs-j2a-coverage-test cs-j2a-coverage-test
   cd wt/cs-j2a-coverage-test
   ```

## Spec (shared with J2a-code)
- Use the shell driver to run `substrate --version-json`, `--shim-status[ -json]`,
  `--shim-deploy/remove`, `--trace`, `--replay`, and `--health` with `--json` enabled.
- Assert every command emits the envelope (status/message/data) with expected payloads.
- Cover error cases (e.g., missing span for `--trace`, replay failure) to ensure structured errors.
- Verify legacy flags still work (e.g., `--version-json` toggles the same behavior).

## Scope & Guardrails
- Tests/fixtures only. Keep runs hermetic via temp HOME/TMPDIR.
- Document any skipped scripts with justification.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure tests pass; capture outputs/skips.
2. Commit worktree changes (e.g., `test: add structured output coverage for core commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and ensure J2a-integ prompt exists; commit docs (`git commit -am "docs: finish J2a-test"`).
5. Remove worktree and hand off.
