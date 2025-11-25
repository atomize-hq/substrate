# Task C1-test (Config CLI foundation & init) – TEST

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`,
   `docs/BACKLOG.md`, and this prompt.
3. Set `C1-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the doc update
   (`git commit -am "docs: start C1-test"`).
4. Create the task branch/worktree:
   ```
   git checkout -b cs-c1-config-test
   git worktree add wt/cs-c1-config-test cs-c1-config-test
   cd wt/cs-c1-config-test
   ```
   Keep docs/tasks/session log edits on `feat/config-subcommand`, not in
   the worktree.

## Spec (shared with C1-code)
- Exercise `substrate config init` through the shell driver:
  - File created with `[install]` + `[world]` defaults under a temp HOME.
  - `--force` rewrites the file when user edits exist.
- When config is missing, launching `substrate` (or installer helper) should
  emit a hint instructing users to run `substrate config init`; tests capture
  stderr/stdout to assert the message.
- Ensure SUBSTRATE_HOME overrides are respected so tests stay isolated.
- Do not touch production code except for minor test-only helpers (e.g., fixture
  builders).

## Scope & Guardrails
- Test files only: `crates/shell/tests`, `tests/installers/*`, fixtures.
- Keep prompts/spec mirrored with C1-code; assume code branch is not visible.
- Document skipped commands (e.g., installer smoke) with justification.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
./tests/installers/install_smoke.sh   # run when platform permits; otherwise note skip
```

## End Checklist
1. Ensure required fmt/tests above are green (or skipped with notes); capture
   outputs for the log.
2. Commit worktree changes with a descriptive message
   (e.g., `test: cover substrate config init scaffolding`).
3. Merge the branch back into `feat/config-subcommand` (fast-forward).
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` with commands/results/blockers, and author the
   `C1-integ` kickoff prompt if it does not already exist.
5. Commit the doc updates on `feat/config-subcommand`
   (`git commit -am "docs: finish C1-test"`), remove the worktree, and hand off.
