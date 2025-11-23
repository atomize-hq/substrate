# Task R6-test (Bootstrap & builtins decomposition) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R6-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R6-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r6-bootstrap-test
   git worktree add wt/cr-r6-bootstrap-test cr-r6-bootstrap-test
   cd wt/cr-r6-bootstrap-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R6-code)
- Crates: `common`, `shim`, `shell` builtins.
- Add/reshape tests for the split manifest schema/resolver/validator modules,
  including property cases for expansion/overlay merge rules.
- Cover shim exec bootstrap/logging/policy flows with harness fixtures; add
  builtin command coverage aligned to the new per-command modules.

## Scope & Guardrails
- Test-only changes: fixtures, helpers, doctests/property tests as needed.
- Avoid production code edits beyond test-only helpers; keep cfg-gates and
  logging/redaction expectations intact.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-common --all-targets
cargo test -p substrate-shim
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/tests above are green; capture outputs (and skips) for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `test: cover manifest/shim builtin split`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r6-bootstrap-test
   git merge --ff-only wt/cr-r6-bootstrap-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r6-bootstrap-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R6-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R6-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r6-bootstrap-test`) if done
   and hand off per instructions.
