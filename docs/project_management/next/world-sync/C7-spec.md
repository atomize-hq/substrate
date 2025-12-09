# C7-spec: Rollback CLI via .substrate-git

## Scope
- Expose `substrate rollback` supporting:
  - `last` (undo last Substrate commit)
  - `checkpoint <id>` (restore to checkpoint tag/metadata)
  - `session <id>` (restore to session-close tag)
- Behavior:
  - Restore files in worktree from `.substrate-git` commit/tag; protect `.git`, `.substrate-git`, `.substrate`, sockets/dev nodes.
  - If world isolation is active, refresh world overlay to match restored host state (or mark dirty if refresh fails).
  - Record a follow-up commit noting the rollback (unless disabled via `use_internal_git=false`).
- Safety:
  - Refuse rollback when worktree dirty unless `--force` or clean-tree guard disabled.
  - Clear messaging when tags/commits missing (e.g., retention/squash scenarios).
- Platform support: degrade gracefully where internal git missing.

## Acceptance
- `substrate rollback last` restores previous Substrate commit and logs result; honors clean-tree guard and protected paths.
- `substrate rollback checkpoint|session` resolves tags and restores or errors clearly when missing.
- World overlay is refreshed or a warning emitted; no silent divergence.
- Rollback creates a metadata commit recording the action (when internal git enabled).

## Out of Scope
- Advanced conflict resolution beyond restore semantics.
- Remote operations.
