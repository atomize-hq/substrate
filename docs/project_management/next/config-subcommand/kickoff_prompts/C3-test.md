# Task C3-test (Config set command) – TEST

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `C3-test` to `in_progress`, log START entry, and commit doc update
   (`git commit -am "docs: start C3-test"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-c3-set-test
   git worktree add wt/cs-c3-set-test cs-c3-set-test
   cd wt/cs-c3-set-test
   ```

## Spec (shared with C3-code)
- Tests cover `substrate config set`:
  - Single-key updates (each supported field) and multi-key runs updating
    anchor mode/path + install fields simultaneously.
  - Validation errors (invalid enum, non-boolean) must exit non-zero and leave
    files untouched.
  - `--json` output parsed to confirm reported keys/values.
  - Atomicity: simulate crash via temporary dir to ensure file not partially
    written.
- Confirm precedence stack unaffected: CLI flag still overrides file even after
  `config set`, captured via environment-driven test.

## Scope & Guardrails
- Tests only; do not modify production code beyond helper hooks.
- Use hermetic HOMEs and temp dirs to avoid polluting real configs.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```
(Installer smoke optional; document if skipped.)

## End Checklist
1. Ensure tests/commands succeed; capture outputs/skips.
2. Commit worktree changes (e.g., `test: cover substrate config set CLI`).
3. Merge into `feat/config-subcommand`.
4. Update `tasks.json` (status → `completed`), append END entry to session log,
   ensure `C3-integ` prompt exists, and commit docs
   (`git commit -am "docs: finish C3-test"`).
5. Remove worktree and hand off.
