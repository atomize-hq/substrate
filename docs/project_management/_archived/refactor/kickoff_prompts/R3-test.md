# Task R3-test (State isolation & binary harness tests) – TEST

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R3-test` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R3-test"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r3-boundaries-test
   git worktree add wt/cr-r3-boundaries-test cr-r3-boundaries-test
   cd wt/cr-r3-boundaries-test
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R3-code)
- Crates: `broker`, `trace`, `world-agent`, `host-proxy`.
- Add tests proving broker/trace contexts are isolated (no shared global state)
  and can run in parallel without cross-talk.
- Cover thin binary entrypoints for `world-agent` and `host-proxy` with
  integration or harness tests that assert main delegates into library
  constructors/run loops.
- Keep fixtures/helpers test-only; align with any refactor surface changes while
  preserving existing CLI/config behavior.

## Scope & Guardrails
- Tests only. Avoid production code changes except minimal hooks required for
  testing. Preserve feature flags/platform cfg-gates and redaction/logging
  expectations.
- No new features or coverage expansion beyond the R3 spec.

## Suggested Commands
```
cargo fmt
cargo test -p substrate-broker
cargo test -p substrate-trace
cargo test -p world-agent
cargo test -p host-proxy
```

## End Checklist
1. Ensure fmt/tests above are green; capture outputs for the END log entry.
2. Commit worktree changes with a descriptive message (e.g., `test: isolate
   broker/trace contexts and thin binaries`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r3-boundaries-test
   git merge --ff-only wt/cr-r3-boundaries-test   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r3-boundaries-test
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the integration
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R3-integ.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R3-test"`).
6. Remove the worktree (`git worktree remove wt/cr-r3-boundaries-test`) if done
   and hand off per instructions.
