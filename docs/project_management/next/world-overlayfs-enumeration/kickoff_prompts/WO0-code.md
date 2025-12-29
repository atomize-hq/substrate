# Kickoff — WO0-code (Stable mount topology + strategy probe + fallback metadata)

Role: Code agent (production code only; no tests).

## Scope (authoritative)
- Implement ADR-0004 + `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md` production code changes only.
- Do not edit planning docs (`plan.md`, `tasks.json`, `session_log.md`, ADRs) from the worktree.

## Start checklist
1. `git checkout feat/world-overlayfs-enumeration && git pull --ff-only`
2. Read:
   - `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
   - `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
   - `docs/project_management/next/world-overlayfs-enumeration/decision_register.md`
   - `docs/project_management/next/world-overlayfs-enumeration/integration_map.md`
3. On `feat/world-overlayfs-enumeration`, set `WO0-code` to `in_progress` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs.
4. Create branch + worktree: `git checkout -b woe-wo0-code` then `git worktree add wt/woe-wo0-code woe-wo0-code`.

## Requirements
- Implement the project-isolation mount choreography change required by ADR-0004 (`mount --move`, not `mount --bind`).
- Implement kernel overlayfs → fuse-overlayfs retry behavior driven by the enumeration probe contract in ADR-0004.
- Emit the required trace fields and doctor JSON keys from ADR-0004 (additive only; do not break existing consumers).
- Implement the required warning-line contract for world-optional fallback to host.

## Commands (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and fix failures.
2. Commit changes in `wt/woe-wo0-code` to branch `woe-wo0-code`.
3. Fast-forward merge back to `feat/world-overlayfs-enumeration`.
4. Update `WO0-code` status to `completed` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs on `feat/world-overlayfs-enumeration`.
5. Remove worktree: `git worktree remove wt/woe-wo0-code`.



Do not edit planning docs inside the worktree.
