# Task R3-code (Global state & binary boundaries) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R3-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R3-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r3-boundaries-code
   git worktree add wt/cr-r3-boundaries-code cr-r3-boundaries-code
   cd wt/cr-r3-boundaries-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R3-test)
- Crates: `broker`, `trace`, `world-agent`, `host-proxy`.
- Replace broker/trace global singletons with context-based handles and explicit
  initialization; preserve public APIs or add clear deprecations.
- Enforce thin-binary pattern for `world-agent` and `host-proxy` (logic in lib;
  main delegates to constructors/run loops) while keeping CLI/config behavior
  unchanged.
- Update lifecycle/configuration docs as needed for any intentional surface
  adjustments; maintain platform cfg-gates and redaction/logging patterns.

## Scope & Guardrails
- Production code only; avoid adding tests beyond minimal helpers required by
  refactors. Keep feature flags/CLI surfaces stable.
- No drive-by changes outside the four crates. Preserve backward compatibility
  or document any deltas in-line and in docs.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-broker
cargo test -p substrate-trace
cargo test -p world-agent
cargo test -p host-proxy
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry.
2. Commit worktree changes with a descriptive message (e.g., `refactor: enforce
   broker/trace contexts and thin binaries`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r3-boundaries-code
   git merge --ff-only wt/cr-r3-boundaries-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r3-boundaries-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R3-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R3-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r3-boundaries-code`) if done
   and hand off per instructions.
