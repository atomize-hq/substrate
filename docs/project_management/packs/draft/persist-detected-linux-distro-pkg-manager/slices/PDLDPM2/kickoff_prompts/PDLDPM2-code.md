# Kickoff: PDLDPM2-code (code)

## Scope
- Production and operator-doc changes only.
- Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-code` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-code` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM2 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2"`.

## Requirements
- Implement only the non-test changes required by the PDLDPM2 AC set.
- Keep smoke-harness assertions in the test task.
- Reconcile `docs/INSTALLATION.md` to the accepted metadata path, `schema_version = 1`, and shared hosted-plus-dev producer wording.
- Do not add new tests or new test files.
- Record a targeted baseline command before edits and rerun it after edits.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Capture the baseline command and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM2-code"` from inside the worktree.
3. Leave the worktree in place.
