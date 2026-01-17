# Kickoff: C1-integ (final integration / aggregator)

## Scope
- Merge any platform-fix branches, re-run integration gates, update `docs/CONFIGURATION.md` per spec, and complete the C1 closeout report.
- Spec: `docs/project_management/next/policy-patch-only-broker-effective-resolution/C1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c1-integ` on branch `policy-patch-only-broker-effective-resolution-c1-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan, tasks, session log, spec, this prompt.

## End Checklist
1. Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `make integ-checks`.
2. Re-run CI compile parity and behavioral smoke per `tasks.json` and record run ids/URLs.
3. Complete `docs/project_management/next/policy-patch-only-broker-effective-resolution/C1-closeout_report.md`.
4. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ"`.

