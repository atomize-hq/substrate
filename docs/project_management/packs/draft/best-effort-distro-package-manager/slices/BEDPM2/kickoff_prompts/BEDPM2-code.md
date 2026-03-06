# Kickoff: BEDPM2-code (code)

## Scope
- Production code only.
- Read `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`, and `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm2-code` on branch `best-effort-distro-package-manager-bedpm2-code` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM2 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" SLICE_ID="BEDPM2"`.

## Requirements
- Implement only the production changes required by the BEDPM2 AC set.
- Keep macOS and Windows behavior unchanged.
- Do not add new tests or new test files.
- Record a targeted baseline command before edits and rerun it after edits.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Capture the baseline command and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM2-code"` from inside the worktree.
3. Leave the worktree in place.
