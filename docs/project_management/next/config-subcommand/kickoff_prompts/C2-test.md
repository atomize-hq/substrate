# Task C2-test (Config show command) – TEST

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `C2-test` to `in_progress`, log START entry, and commit doc update
   (`git commit -am "docs: start C2-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-c2-show-test
   git worktree add wt/cs-c2-show-test cs-c2-show-test
   cd wt/cs-c2-show-test
   ```

## Spec (shared with C2-code)
- Tests invoke `substrate config show`:
  - Validate TOML output matches file contents under a temp HOME.
  - Validate `--json` output parses and mirrors the same data.
  - Ensure missing config path emits hint to run `config init`.
  - Cover redaction hook by simulating a sensitive key (even placeholder).
- Keep tests hermetic via `SUBSTRATE_HOME`/`TMPDIR`.

## Scope & Guardrails
- Test files only. Minimal helpers allowed for fixtures.
- Document any skipped scripts (e.g., installer smoke) with reason.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
```
(Run installer smoke if viable.)

## End Checklist
1. Confirm fmt/tests (and any scripts) are green; note outputs/skips.
2. Commit worktree changes (e.g., `test: cover substrate config show output`).
3. Merge branch into `feat/config-subcommand` (fast-forward).
4. Update `tasks.json` (status → `completed`), append END entry to session log,
   and ensure `C2-integ` prompt exists. Commit doc updates
   (`git commit -am "docs: finish C2-test"`).
5. Remove worktree and hand off.
