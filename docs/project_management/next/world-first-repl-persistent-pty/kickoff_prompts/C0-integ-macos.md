# Kickoff: C0-integ-macos (integration macOS)

## Scope
- Platform-fix task for macOS smoke results for slice C0 (macOS uses Lima-backed world).
- Smoke script: `docs/project_management/next/world-first-repl-persistent-pty/smoke/macos-smoke.sh`
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c0-integ-macos` on branch `world-first-repl-persistent-pty-c0-integ-macos` and that `.taskmeta.json` exists.
2. Read: manual_testing_playbook.md and this prompt.

## Requirements
- Own macOS smoke for `SMOKE_SLICE_ID=C0` and apply fixes on this branch only.
- Dispatch macOS smoke via `make feature-smoke ... PLATFORM=macos SMOKE_SLICE_ID="C0"` (see tasks.json end checklist).

## End Checklist
1. Ensure macOS smoke is green or fixes are committed.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-macos"`

