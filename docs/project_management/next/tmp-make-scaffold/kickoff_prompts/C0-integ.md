# Kickoff: C0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. `git checkout feat/tmp-make-scaffold && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. Set `C0-integ` status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-integ`).
4. Create branch `c0-integ`; create worktree `wt/tmp-make-scaffold-c0-integ`.

## Requirements
- Merge the relevant integration branches for this slice:
  - The core integration branch (e.g., `*-integ-core`) and any platform-fix integration branches (`*-integ-linux|macos|windows|wsl`) that produced commits.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Run cross-platform smoke via CI to confirm the merged result is green:
  - `scripts/ci/dispatch_feature_smoke.sh --feature-dir "docs/project_management/next/tmp-make-scaffold" --runner-kind self-hosted --platform all --cleanup`
  - If WSL coverage is required for this feature, add `--run-wsl`.
- Complete the slice closeout gate report:
  - `docs/project_management/next/tmp-make-scaffold/<SLICE>-closeout_report.md` (e.g., `docs/project_management/next/tmp-make-scaffold/C0-closeout_report.md`)

## End Checklist
1. Ensure all required platforms are green (include run ids/URLs).
2. Commit worktree changes.
3. Merge back to orchestration branch (ff-only).
4. Update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-integ`); remove worktree.
