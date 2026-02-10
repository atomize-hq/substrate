# C7-spec: Rollback CLI via .substrate-git

## Scope
- Expose `substrate rollback` supporting:
  - `last` (undo last Substrate commit)
  - `checkpoint <id>` (restore to checkpoint tag/metadata)
  - `session <id>` (restore to session-close tag)
- Behavior:
  - Restore files in worktree from `.substrate-git` commit/tag; protect `.git`, `.substrate-git`, `.substrate`, sockets/dev nodes.
  - Rollback is host-only; it does not refresh the world overlay. After rollback, the CLI prints one line instructing the user to start a new world session to pick up the restored host state.
  - Record a follow-up commit noting the rollback when `sync.use_internal_git=true`.
- Safety:
  - Refuse rollback when the workspace is a git repo with a dirty worktree unless `--force` is provided.
  - Clear messaging when tags/commits missing (e.g., retention/squash scenarios).
- Platform support: if internal git is missing or corrupt, exit `4` with a clear message.

## Acceptance
- `substrate rollback last` restores previous Substrate commit and logs result; honors clean-tree guard and protected paths.
- `substrate rollback checkpoint|session` resolves tags and restores or errors clearly when missing.
- World overlay is refreshed or a warning emitted; no silent divergence.
- Rollback creates a metadata commit recording the action (when internal git enabled).

## Out of Scope
- Advanced conflict resolution beyond restore semantics.
- Remote operations.
