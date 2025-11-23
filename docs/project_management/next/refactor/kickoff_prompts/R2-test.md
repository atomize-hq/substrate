# Task R2-test (Shell decomposition & PTY channelization) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R2-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R2-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r2-shell-test
   git worktree add wt/cr-r2-shell-test cr-r2-shell-test
   cd wt/cr-r2-shell-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R2-code)
- Crate: `shell`.
- Move tests out of `crates/shell/src/lib.rs` into `crates/shell/tests/`
  integration/unit modules aligned with the new execution/repl/builtins/scripts
  modules.
- Add coverage for the channel-based PTY manager (resize/write/close) and for
  module seams introduced by the decomposition; no global mutex patterns.
- Preserve CLI behavior and update fixtures as needed; keep docs references
  aligned with the new module layout.
- Maintain existing cfg-gates/feature flags and logging/redaction patterns.

## Scope & Guardrails
- **Tests only. Do not modify production code** (except minimal test-only
  helpers/fixtures if strictly required).
- Stay scoped to shell tests and PTY channel coverage; avoid drive-by changes
  in other crates.
- Mirror the shared spec; do not rely on visibility into the code branch beyond
  documented behavior.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
./tests/installers/install_smoke.sh   # document scenarios/results in END entry
```

## End Checklist
1. Ensure fmt/tests you ran are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: extract
   shell tests and cover pty manager`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r2-shell-test
   git merge --ff-only wt/cr-r2-shell-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r2-shell-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and create/confirm the
   integration prompt path (`docs/project_management/next/refactor/kickoff_prompts/R2-integ.md`).
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R2-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r2-shell-test`) if done and
   hand off per instructions.
