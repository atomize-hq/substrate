# Kickoff: BEDPM3-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-code` on branch `best-effort-distro-package-manager-bedpm3-code` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" SLICE_ID="BEDPM3"`.

## Requirements
- Implement only the production-code behavior in BEDPM3.
- Use a baseline installer command before and after edits:
  - if `tests/installers/pkg_manager_detection_smoke.sh` exists, run `bash tests/installers/pkg_manager_detection_smoke.sh`
  - otherwise run `bash tests/installers/pkg_manager_container_smoke.sh`
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Capture the baseline command and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-code"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
