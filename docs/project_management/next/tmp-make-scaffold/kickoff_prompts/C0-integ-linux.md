# Kickoff: C0-integ-linux (integration platform-fix — linux)

## Scope
- Ensure the slice behaves correctly on **linux**.
- This task is allowed to make production-code and/or test changes as needed to achieve cross-platform parity, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: **linux**.
2. `git checkout feat/tmp-make-scaffold && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
4. Set `C0-integ-linux` status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-integ-linux`).
5. Create branch `c0-integ-linux`; create worktree `wt/tmp-make-scaffold-c0-integ-linux`.

## Requirements
- First, validate platform smoke via CI for this platform:
  - `scripts/ci/dispatch_feature_smoke.sh --feature-dir "docs/project_management/next/tmp-make-scaffold" --runner-kind self-hosted --platform linux --cleanup`
- If this is the Linux task and WSL coverage is required (see `tasks.json` meta: `wsl_required` + `wsl_task_mode`):
  - Bundled (default): include `--run-wsl` in the Linux smoke dispatch.
- If smoke passes: record run id/URL in the END entry and do not change code.
- If smoke fails:
  1) Fix the issue in this worktree (platform-specific guards, path handling, deps, etc.) while keeping the spec contract intact.
  2) Run the appropriate local checks for your change (fmt/clippy and targeted tests).
  3) Re-run the CI smoke for this platform until green.

## End Checklist
1. Ensure smoke is green for linux and capture the run id/URL.
2. Commit worktree changes (if any).
3. Merge back to orchestration branch (ff-only).
4. Update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-integ-linux`); remove worktree.
