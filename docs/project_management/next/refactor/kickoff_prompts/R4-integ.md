# Task R4-integ (Integrate polish/doc sweep) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R4-code` and `R4-test` are completed.
4. Set `R4-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update
   (`git commit -am "docs: start R4-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r4-polish-integ
   git worktree add wt/cr-r4-polish-integ cr-r4-polish-integ
   cd wt/cr-r4-polish-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared baseline)
- Crates: `trace`, `world-windows-wsl`, `replay`, `common`.
- Integrate R4 polish code (module splits, docs) with R4 test additions
  (doctests/property tests/fixtures) without changing behavior beyond the
  shared spec.
- Preserve platform cfg-gates and redaction/logging expectations; ensure
  fixtures match the refactored module layouts.

## Scope & Guardrails
- Merge only the R4 code and test branches; avoid new features or coverage
  beyond the R4 spec.
- Keep CLI/config behavior intact; no production changes except conflict
  resolution required for integration.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-trace
cargo test -p world-windows-wsl          # gated on Windows; note skips
cargo test -p substrate-replay --all-targets
cargo test --doc -p substrate-replay
cargo test -p substrate-common --all-targets
```
Capture outputs for the END log entry, including any platform skips.

## End Checklist
1. Resolve merges between R4 code/test branches inside the integration
   worktree.
2. Ensure fmt/clippy/tests (above) are green; log results and any skips.
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R4 polish/doc validation`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r4-polish-integ
   git merge --ff-only wt/cr-r4-polish-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r4-polish-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt is referenced.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R4-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r4-polish-integ`) and hand
   off per instructions.
