# Kickoff: C0-integ (final integration / aggregator)

## Scope
- Merge any platform-fix branches, re-run integration gates, and complete the C0 closeout report.
- Spec: `docs/project_management/next/policy-patch-only-broker-effective-resolution/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c0-integ` on branch `policy-patch-only-broker-effective-resolution-c0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan/tasks/session_log/spec/this prompt.

## End Checklist
1. Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `make integ-checks`.
2. Re-run CI compile parity and behavioral smoke per `tasks.json` and record run ids/URLs.
3. Complete `docs/project_management/next/policy-patch-only-broker-effective-resolution/C0-closeout_report.md`.
4. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ"`.

