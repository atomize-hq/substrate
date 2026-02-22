# WS6-spec — Internal checkpoint (`workspace checkpoint`)

## Scope

- Implement `substrate workspace checkpoint` per `internal-git-spec.md`.

## Behavior (authoritative)

- `workspace checkpoint` behavior is owned by `internal-git-spec.md`.
- WS6 additionally requires:
  - deterministic, actionable stderr messages on failure (exit `2|3|5`),
  - and stable stdout printing of the created checkpoint id on success.

## Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- 0: success (including no-op: nothing changed)
- 2: not in workspace or invalid args
- 3: required dependency unavailable (`git` missing)
- 5: safety refusal

## Acceptance criteria

- `substrate workspace checkpoint` creates internal git commits/tags per `internal-git-spec.md`.
- No-op checkpoints exit `0` and do not create new commits/tags.
- Protected paths are excluded from snapshots.

## Out of scope

- Rollback (WS7).
