# Kickoff: DIWAS1-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/dev-install-world-agent-staging/slices/DIWAS1/DIWAS1-spec.md`
- Contract: `docs/project_management/packs/draft/dev-install-world-agent-staging/contract.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dev-install-world-agent-staging-diwas1-code` on branch `dev-install-world-agent-staging-diwas1-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `contract.md`, the slice spec, and this prompt.
3. Confirm `DIWAS0-integ` is complete (DIWAS1 depends on DIWAS0).

## Requirements
- On Linux, `scripts/substrate/dev-install-substrate.sh --no-world` must still stage `world-agent` into both accepted staged paths.
- Selected dev-install profile must control the staged bridge target, and repeated dev installs must refresh the links.
- Keep provisioning disabled when `--no-world` is set.

## Validation (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Capture a short before/after note for the manual playbook (Cases 1 and 2).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-code"`.

