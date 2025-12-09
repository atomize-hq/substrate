# C6-spec: .substrate-git Integration (Commits & Checkpoints)

## Scope
- Initialize and manage an internal git repo at `.substrate-git/.git` with worktree=`.`; ensure `.substrate-git` is ignored by user git.
- After successful world→host sync (manual or auto), stage and commit changed paths to `.substrate-git` with metadata (session/span/command, direction, conflict policy, dry-run indicator).
- Settings:
  - `use_internal_git` (bool, default true)
  - `enforce_clean_tree_before_sync` (bool, default false): if true, fail sync when worktree has external dirt (outside Substrate changes).
- Add `substrate checkpoint` command to record a commit without running sync (no-op when no changes).
- Clear error handling: if git unavailable/corrupt, surface error and skip commit without blocking sync apply (unless clean-tree guard triggers).
- No rollback commands yet.
- No host→world commits unless that path mutates host (only record world→host effects).

## Acceptance
- Internal git auto-initializes when needed; `.substrate-git` never touches user `.git`.
- Sync operations produce commits when `use_internal_git=true` and changes exist; skipped with clear message when disabled or no changes.
- Clean-tree guard blocks sync apply if enabled and worktree dirty (outside Substrate), with explicit message.
- `substrate checkpoint` records a commit (or reports no-op) using current tree state; honors `use_internal_git`.
- Logging includes commit hash when created.

## Out of Scope
- Rollback/restore CLI.
- Remote pushes/pulls.
