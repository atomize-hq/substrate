# WS5-spec — Host→world pre-sync + direction expansion + PTY apply

## Scope
- Implement `direction=from_host` and `direction=both` for `workspace sync`.
- Implement PTY diff apply (world→host) under `direction=from_world` and `both`.
- Add the “host→world pre-sync” reconciliation step required for correctness when an overlay upper shadows lower host changes.

## Behavior (authoritative)

### Direction semantics
`workspace sync` resolves an effective direction:
- `from_world`: apply pending diffs from world overlay to host (non-PTY + PTY when present)
- `from_host`: reconcile host changes into the world overlay (pre-sync only; no host mutations)
- `both`: perform `from_host` reconciliation first, then `from_world` apply

### `from_host` reconciliation
Behavior:
- Compute the set of “overlay-shadowed” paths from the pending diff record:
  - the union of `writes|mods|deletes` across the `non_pty` bucket and (if present) the `pty` bucket.
- For each such path:
  - If the host path is newer than `session_started_at`, it is a conflict (DR-0004).
  - Conflict policy determines whether the host version replaces the upper version (`prefer_host`), the upper version is kept (`prefer_world`), or the operation aborts (`abort`).

Mutations:
- `from_host` MUST NOT mutate the host workspace.
- `from_host` MUST update the world session writable layer (backend-specific) so subsequent world reads reflect the selected policy:
  - `prefer_host`: discard the world’s pending change for the path so the world session observes the host version.
  - `prefer_world`: keep the world’s pending change for the path.
  - `abort`: perform no mutations and exit `5` if any conflict exists.

Output:
- `--dry-run` MUST print a deterministic reconciliation plan summary (counts, and whether conflicts exist).
- `--verbose` MUST print deterministic per-path decisions (keep/discard/conflict).

### PTY apply
When applying `from_world`:
- Include both non-PTY and PTY pending diff sets.
- Enforce the same safety rails and ordering defined in `filesystem-semantics-spec.md`.

### Exit codes
- 0: success (including no-op)
- 3: world backend required but unavailable
- 4: backend capability unsupported
- 5: safety refusal (protected path, size guard, or abort-on-conflict)

## Acceptance criteria
- `workspace sync --dry-run --direction from_host` is supported and reports reconciliation plan.
- `workspace sync --direction from_host` mutates overlay state but does not mutate host workspace.
- `workspace sync --direction both` runs reconciliation then apply.
- PTY-originated diffs can be applied to host under `from_world` and `both`.

## Out of scope
- Internal checkpoint/rollback (WS6/WS7).
