# Kickoff: C0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. `git checkout feat/tmp-make-scaffold && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. Set `C0-integ-core` status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-integ-core`).
4. Create branch `c0-integ-core`; create worktree `wt/tmp-make-scaffold-c0-integ-core`.

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run required integration gates:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- If the feature directory contains `smoke/`, run cross-platform smoke via CI (validation-only):
  - `scripts/ci/dispatch_feature_smoke.sh --feature-dir "docs/project_management/next/tmp-make-scaffold" --runner-kind self-hosted --platform all --cleanup`
  - If WSL coverage is required for this feature, add `--run-wsl`.
- If any platform smoke fails, do not attempt platform-specific fixes here. Record failures in the END entry and let the corresponding `*-integ-<platform>` task(s) do the platform-fix work.

## End Checklist
1. Run required commands; capture outputs (including any smoke run ids/URLs).
2. Commit worktree changes.
3. Merge back to orchestration branch (ff-only).
4. Update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-integ-core`); remove worktree.

