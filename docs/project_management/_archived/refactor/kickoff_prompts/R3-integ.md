# Task R3-integ (Integrate state/binary boundaries) – INTEGRATION

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Confirm `R3-code` and `R3-test` are completed.
4. Set `R3-integ` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update
   (`git commit -am "docs: start R3-integ"`).
5. Create the task branch and worktree:
   ```
   git checkout -b cr-r3-boundaries-integ
   git worktree add wt/cr-r3-boundaries-integ cr-r3-boundaries-integ
   cd wt/cr-r3-boundaries-integ
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared baseline)
- Crates: `broker`, `trace`, `world-agent`, `host-proxy`.
- Integrate the context-based broker/trace changes and thin binary patterns
  from `R3-code` with the isolation/binary harness tests from `R3-test`.
- Ensure broker/trace contexts remain explicit (no shared global state), and
  `world-agent`/`host-proxy` mains stay thin delegators without changing
  CLI/config/logging behavior.

## Scope & Guardrails
- Merge only the R3 code and test branches; do not expand coverage or modify
  production behavior beyond the shared R3 spec.
- Preserve feature flags, platform cfg-gates, and redaction/logging patterns;
  avoid drive-by changes in other crates.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-broker
cargo test -p substrate-trace
cargo test -p world-agent
cargo test -p host-proxy
```
Capture outputs for the END log entry.

## End Checklist
1. Resolve merges between R3 code/test branches inside the integration
   worktree.
2. Ensure fmt/clippy/tests (above) are green and results are recorded.
3. Commit worktree changes with a descriptive message
   (e.g., `chore: integrate R3 state/binary boundaries`).
4. Return to repo root and merge the integration branch:
   ```
   git checkout cr-r3-boundaries-integ
   git merge --ff-only wt/cr-r3-boundaries-integ   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r3-boundaries-integ
   ```
5. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and create/confirm kickoff
   prompts for `R4-code` and `R4-test`.
6. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R3-integ"`).
7. Remove the worktree (`git worktree remove wt/cr-r3-boundaries-integ`) and
   hand off per instructions.
