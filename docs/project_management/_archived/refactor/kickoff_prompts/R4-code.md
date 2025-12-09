# Task R4-code (Polish & documentation sweep) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R4-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R4-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r4-polish-code
   git worktree add wt/cr-r4-polish-code cr-r4-polish-code
   cd wt/cr-r4-polish-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R4-test)
- Crates: `trace`, `world-windows-wsl`, `replay`, `common` (+ `CHANGELOG.md`).
- Split `trace` and `world-windows-wsl` single-file crates into modules per the
  analysis while keeping thin lib surfaces and existing public APIs intact.
- Add replay module-level rustdoc with runnable examples and introduce a
  documented `common` prelude if justified by the refactor.
- Update `CHANGELOG.md` for refactor impacts; preserve feature flags, platform
  cfg-gates, and logging/redaction behavior.

## Scope & Guardrails
- Production code and documentation only; avoid adding tests beyond doc
  examples required by new surfaces.
- No new features or platform behavior changes; keep CLI/config stable and
  avoid drive-by edits outside the four crates.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-trace
cargo test -p world-windows-wsl           # on supported platforms; note skips
cargo test -p substrate-replay --all-targets
cargo test -p substrate-common --all-targets
cargo test --doc -p substrate-replay      # ensure new docs compile
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `chore: polish
   trace docs and split world-windows-wsl`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r4-polish-code
   git merge --ff-only wt/cr-r4-polish-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r4-polish-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R4-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R4-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r4-polish-code`) if done and
   hand off per instructions.
