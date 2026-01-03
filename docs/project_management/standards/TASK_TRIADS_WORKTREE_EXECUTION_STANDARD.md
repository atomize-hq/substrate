# Task Triads — Worktree Execution Standard (Automation + Concurrent Code/Test)

This standard is for **execution-time** triad work when tasks are started via triad automation (preferred), and the agent is already running **inside a task worktree**.

It is a focused companion to:
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md` (full planning pack + task schema + integration model)

## Operating assumptions (the situation you are in)

When this standard applies, the operator started you with one of:
- `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="<slice>" LAUNCH_CODEX=1` (preferred)
- `make triad-task-start FEATURE_DIR="docs/project_management/next/<feature>" TASK_ID="<task-id>" LAUNCH_CODEX=1`

You are already:
- in a git worktree (usually under `wt/...`)
- on a task branch (not the orchestration branch)
- expected to find `.taskmeta.json` at the worktree root (task id, orchestration branch, etc.)

## Non-negotiable safety rule

- Do not edit planning docs inside the worktree (anything under `docs/project_management/next/`).
  - The task finisher enforces this and will fail if you try.

## Separation of concerns (role boundaries)

All agents receive the **same spec** for the slice and must implement/validate **that exact contract**.

### Code agent (type=`code`)

Your job is to implement the slice’s production behavior per the spec.

Rules:
- Do not add new tests.
- Do not create new test files.
- Only touch tests if required to restore **baseline** test behavior after your code change:
  - Example: a previously-passing test now fails because the spec’s behavior change makes the old expectation invalid.
  - In that case, adjust the **existing** test expectation to match the spec (still no new test cases).
- Do not coordinate merges back to orchestration; integration owns merge/reconcile.

Baseline testing requirement (to avoid silent regressions):
- Before making code changes, run a **baseline** targeted test set relevant to the area you will touch.
- After your code changes, re-run the **same** tests and ensure the results are unchanged (or improved).
  - If baseline was green: it must remain green.
  - If baseline had known failures: do not introduce new failures; keep failure set stable.

Choosing a baseline test set:
- Prefer the smallest suite that exercises the touched behavior (e.g., a single integration test file, a package test target, or a focused module test target).
- Record the exact command(s) you ran and the observed outcome (exit code + key failure summaries) in your task handoff.

### Test agent (type=`test`)

Your job is to write/update tests that encode the spec’s acceptance criteria.

Rules:
- Tests only (plus minimal test-only helpers/fixtures/mocks if needed).
- Do not modify production code.
- Do not try to “make tests pass easily” by weakening assertions or checking implementation details.
- Prefer contract-level assertions and explicit fixtures/mocks when required to express the spec.

Passing expectation:
- Your branch is not expected to be fully green before the code agent lands the behavior.
- Your tests **must** be:
  - compilable/runnable in isolation (no compile errors),
  - deterministic,
  - failing for the right reason (assertion mismatches that will flip green when code implements the spec), not because of missing setup or brittle coupling.

## What “done” means (per task type)

All task types:
- You commit only to your task branch in your worktree.
- You do not delete the worktree (feature cleanup removes worktrees later).
- You do not merge branches; integration handles merge/reconcile to the spec.

### Code task completion
1. Ensure you ran:
   - `cargo fmt`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - your chosen baseline tests (before + after)
2. Commit your changes to the task branch.
3. Run the task finisher from inside the worktree:
   - `make triad-task-finish TASK_ID="<task-id>"`
4. Hand off:
   - baseline test command(s) + outcomes (before + after),
   - any behavior notes that the test agent should align to.

### Test task completion
1. Ensure you ran:
   - `cargo fmt`
   - the targeted tests you added/modified (even if expected to fail due to missing code)
2. Commit your changes to the task branch.
3. Run the task finisher from inside the worktree:
   - `make triad-task-finish TASK_ID="<task-id>"`
4. Hand off:
   - targeted test command(s) + outcomes,
   - what is expected to be red until code is merged,
   - any fixtures/mocks introduced and why.

## Operator-owned steps (do not do these inside the worktree)

The operator (or an orchestration/integration workflow) owns:
- Updating `docs/project_management/next/<feature>/tasks.json` statuses.
- Appending START/END entries to `docs/project_management/next/<feature>/session_log.md`.
- Merging/rebasing task branches into integration and then into orchestration (when applicable).
- Running feature cleanup (`FZ-feature-cleanup`) to remove retained worktrees.

If your kickoff prompt asks you to edit `tasks.json`/`session_log.md` yourself, treat it as **outdated** for automation mode and ask the operator to do it instead.

## When automation is not viable (exception path)

If a slice cannot be executed via automation/concurrent code+test (rare), fall back to the manual flow in:
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

