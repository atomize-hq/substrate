# Task R1-integ (Integrate panic remediation) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R1-code` and `R1-test` are completed.
4. Set `R1-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update
   (`git commit -am "docs: start R1-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r1-panics-integ
   git worktree add wt/cr-r1-panics-integ cr-r1-panics-integ
   cd wt/cr-r1-panics-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared baseline)
- Crates: `broker`, `world`, `telemetry-lib`, `forwarder`.
- Panic remediation: library `.unwrap()` paths replaced with `Result` +
  `anyhow::Context`; no new panics; public APIs preserved or documented.
- Tests from `R1-test` validate poisoned locks/error paths return `Result` errors
  (no panics). Ensure code and tests align with the shared spec.
- Keep logging/redaction consistent with `crates/common/src/log_schema.rs`.

## Scope & Guardrails
- Integrate the R1 code and test branches; resolve conflicts; do not expand
  scope beyond R1.
- Ensure merged baseline retains behavior across all four crates and platforms.
- Update docs/tasks/session log on `feat/crate-refactor` only (not in worktree).

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p broker
cargo test -p world
cargo test -p telemetry-lib
cargo test -p forwarder
```
Log any additional targeted suites you run.

## End Checklist
1. Resolve merges between code/test branches inside the integration worktree.
2. Ensure fmt/clippy and the targeted tests above are green; capture outputs for
   the END log entry.
3. Commit worktree changes with a descriptive message (e.g., `chore: integrate R1 panic remediation`).
4. Return to repo root, merge the integration branch:
   ```
   git checkout cr-r1-panics-integ
   git merge --ff-only wt/cr-r1-panics-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r1-panics-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and create the next code/test
   kickoff prompts for R2 (per role rules).
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R1-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r1-panics-integ`) and hand
   off per instructions.
