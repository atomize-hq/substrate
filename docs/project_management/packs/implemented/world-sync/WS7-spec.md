# WS7-spec — Internal rollback (`workspace rollback`)

## Scope
- Implement `substrate workspace rollback` per `internal-git-spec.md`.

## Behavior (authoritative)
- `workspace rollback` behavior is owned by `internal-git-spec.md`.
- WS7 additionally requires:
  - deterministic target resolution (`last` and explicit checkpoint ids),
  - safety rails are enforced by default and bypassed only by `--force`.

## Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 0: success
- 2: invalid target or not in workspace
- 3: required dependency unavailable (`git` missing)
- 5: safety refusal (dirty workspace without `--force`, or would-delete-noncheckpointed-paths without `--force`)

## Acceptance criteria
- `substrate workspace rollback last` restores the workspace per `internal-git-spec.md`.
- `--force` is required for any destructive delete of non-checkpointed paths.
- Protected paths are never mutated.

## Out of scope
- Advanced multi-checkpoint branching workflows in internal git.

