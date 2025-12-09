# Task C1-code (Config CLI foundation & init) – CODE

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, the backlog
   entry in `docs/BACKLOG.md`, and this prompt.
3. Set `C1-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the doc update
   (`git commit -am "docs: start C1-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cs-c1-config-code
   git worktree add wt/cs-c1-config-code cs-c1-config-code
   cd wt/cs-c1-config-code
   ```
   Do **not** edit docs/tasks/session logs from the worktree.

## Spec (shared with C1-test)
- Add a `config` subcommand group to the CLI with an `init` verb available before
  shell/REPL execution.
- `substrate config init`:
  - Creates `~/.substrate/config.toml` and required parent directories.
  - Writes default `[install]` and `[world]` tables (align with current schema).
  - Supports `--force` to regenerate even if the file exists.
- Shell startup (and installer scripts) log a clear hint (“run `substrate config
  init`”) when the config is missing instead of silently failing.
- Command exits zero on success, non-zero on error; errors bubble with context.
- Docs (`docs/CONFIGURATION.md`, `docs/USAGE.md`) reference the new command and
  describe when to run it.

## Scope & Guardrails
- Production code plus necessary doc changes only. **Tests are owned by C1-test.**
- Touch CLI parsing, invocation plumbing, settings helpers, and installer
  scripts as needed; avoid unrelated refactors.
- Keep behavior consistent across Unix/Windows; ensure SUBSTRATE_HOME override
  works for tests.

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
```
(Record any additional commands or skips in the END log entry.)

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the log.
2. Commit worktree changes with a descriptive message
   (e.g., `feat: add substrate config init command`).
3. Merge the task branch back into `feat/config-subcommand` (fast-forward only).
4. Update `tasks.json` (status → `completed`) and append an END entry to
   `session_log.md` with commands/results/blockers; commit the doc updates on
   `feat/config-subcommand` (`git commit -am "docs: finish C1-code"`).
5. Remove the worktree (`git worktree remove wt/cs-c1-config-code`) and hand off
   per instructions.
