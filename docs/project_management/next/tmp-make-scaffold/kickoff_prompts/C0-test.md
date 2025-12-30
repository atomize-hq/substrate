# Kickoff: C0-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. `git checkout feat/tmp-make-scaffold && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. Set `C0-test` status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-test`).
4. Create branch `c0-test`; create worktree `wt/tmp-make-scaffold-c0-test`.

## Requirements
- Add/modify tests that enforce the spec’s acceptance criteria.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. Commit worktree changes.
3. Merge back to orchestration branch (ff-only).
4. Update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-test`); remove worktree.

