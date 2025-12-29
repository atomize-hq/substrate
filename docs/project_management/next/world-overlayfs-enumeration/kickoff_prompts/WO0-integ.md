# Kickoff — WO0-integ (Integration: land WO0 with smoke + playbook validation)

Role: Integration agent (owns final alignment to spec and green state).

## Scope (authoritative)
- Integrate `woe-wo0-code` + `woe-wo0-test` and reconcile final behavior to:
  - `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
  - `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Run required validation commands, Linux smoke script, and manual playbook steps.
- Do not edit planning docs from the worktree (only from the orchestration branch).

## Start checklist
1. `git checkout feat/world-overlayfs-enumeration && git pull --ff-only`
2. On `feat/world-overlayfs-enumeration`, set `WO0-integ` to `in_progress` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs.
3. Create branch + worktree: `git checkout -b woe-wo0-integ` then `git worktree add wt/woe-wo0-integ woe-wo0-integ`.
4. Merge `woe-wo0-code` and `woe-wo0-test` into `wt/woe-wo0-integ` and resolve drift to spec.

## Commands (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...` suites (at minimum the ones added/modified by WO0-test)
- `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
- `make preflight`

## End checklist
1. Run required commands and fix failures until green.
2. Commit integration changes to branch `woe-wo0-integ`.
3. Fast-forward merge `woe-wo0-integ` back to `feat/world-overlayfs-enumeration`.
4. Update `WO0-integ` status to `completed` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs on `feat/world-overlayfs-enumeration`.
5. Remove worktree: `git worktree remove wt/woe-wo0-integ`.



Do not edit planning docs inside the worktree.
