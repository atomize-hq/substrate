# Kickoff: C0-code (code)

## Scope
- Production code only; no tests.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. `git checkout feat/tmp-make-scaffold && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. Set `C0-code` status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-code`).
4. Create branch `c0-code`; create worktree `wt/tmp-make-scaffold-c0-code`.

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs.
2. Commit worktree changes.
3. Merge back to orchestration branch (ff-only).
4. Update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-code`); remove worktree.

