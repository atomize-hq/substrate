# C6-spec: .substrate-git Integration (Commits & Checkpoints)

## Scope
- Initialize and manage the internal git directory at `.substrate-git/repo.git`; ensure `.substrate-git` is ignored by user git.
- After successful world→host sync (manual or auto), stage and commit changed paths to the internal repo using `--git-dir .substrate-git/repo.git --work-tree .`, with metadata (session/span/command, direction, conflict policy).
- Settings:
  - `sync.use_internal_git` (bool, default true)
  - `sync.enforce_clean_tree_before_sync` (bool, default false): if true, fail sync when worktree is dirty before applying changes.
- Add `substrate checkpoint` command to record a commit without running sync (no-op when no changes).
- Clear error handling:
  - If internal git is unavailable or corrupt, `substrate sync` must still apply changes and must exit based on the sync apply outcome; it must print a single warning that internal git recording was skipped.
  - If `sync.enforce_clean_tree_before_sync=true` and the worktree is dirty, `substrate sync` must exit `5` and must not apply changes.
- No rollback commands yet.
- No host→world commits unless that path mutates host (only record world→host effects).

## Acceptance
- Internal git auto-initializes when needed; `.substrate-git` never touches user `.git`.
- Sync operations produce commits when `sync.use_internal_git=true` and changes exist; otherwise they print a single line explaining why no commit was created.
- Clean-tree guard blocks sync apply when `sync.enforce_clean_tree_before_sync=true` and the workspace is a git repo with a dirty worktree; it exits `5` with an explicit message.
- `substrate checkpoint` records a commit when changes exist; if no changes exist it prints “no changes” and exits `0`.
- Logging includes commit hash when created.

## Out of Scope
- Rollback/restore CLI.
- Remote pushes/pulls.
