# Kickoff — WO0-test (Tests: enumeration probe + fallback + metadata)

Role: Test agent (tests only; no production code).

## Scope (authoritative)
- Add tests required by ADR-0004 + `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`.
- Do not modify production code outside of minimal test-only helpers that are strictly required for deterministic testing.
- Do not edit planning docs from the worktree.

## Start checklist
1. `git checkout feat/world-overlayfs-enumeration && git pull --ff-only`
2. Read:
   - `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
   - `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
3. On `feat/world-overlayfs-enumeration`, set `WO0-test` to `in_progress` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs.
4. Create branch + worktree: `git checkout -b woe-wo0-test` then `git worktree add wt/woe-wo0-test woe-wo0-test`.

## Requirements
- Add Linux coverage for:
  - Enumeration correctness (created file appears via directory enumeration in world view).
  - Forced primary-strategy enumeration failure driving fuse-overlayfs fallback selection.
  - Trace metadata fields and allowed fallback-reason enums from ADR-0004.
  - Doctor JSON keys required by ADR-0004.

## Commands (required)
- `cargo fmt`
- Run targeted `cargo test ...` commands for the tests you add/modify.

## End checklist
1. Run required commands and fix failures.
2. Commit changes in `wt/woe-wo0-test` to branch `woe-wo0-test`.
3. Fast-forward merge back to `feat/world-overlayfs-enumeration`.
4. Update `WO0-test` status to `completed` in `docs/project_management/next/world-overlayfs-enumeration/tasks.json` and commit docs on `feat/world-overlayfs-enumeration`.
5. Remove worktree: `git worktree remove wt/woe-wo0-test`.



Do not edit planning docs inside the worktree.
