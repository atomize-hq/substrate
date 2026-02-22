# WS2-spec — Non-PTY world→host apply (direction=from_world) + safety rails

## Scope

- Implement `substrate workspace sync` apply semantics for non-PTY pending diffs when `direction=from_world`.
- Enforce safety rails (protected paths, exclude filtering, size guards, conflict policy).
- Clear pending non-PTY diffs after successful apply.

## Behavior (authoritative)

### Direction support

- `direction=from_world` is supported for apply and dry-run.
- `direction=from_host` and `direction=both` remain unsupported until WS5 (exit `4`).

### Apply preflight (must be all-or-nothing)

For `workspace sync --direction from_world` (without `--dry-run`):

1. Retrieve pending non-PTY diff (same as WS1).
2. Validate and plan per `filesystem-semantics-spec.md`:
   - reject protected paths (exit `5`)
   - apply exclude filtering
   - enforce size guards (exit `5`)
   - detect conflicts per DR-0004 and apply `sync.conflict_policy`
3. If `conflict_policy=abort` and any conflict exists:
   - perform no mutations and exit `5`.

### Apply execution

When preflight passes:

- Apply deletes then writes/mods per the ordering in `filesystem-semantics-spec.md`.
- If a filesystem operation fails mid-apply:
  - exit `1` (unexpected error),
  - do not attempt to revert already-applied mutations.

### Clearing pending diffs

After a successful apply (exit `0`):

- The applied pending diff MUST be acknowledged/cleared using the `diff_id` returned by discovery (see `filesystem-semantics-spec.md` Clear/ack semantics).
- If the clear/ack step fails (e.g., diff_id mismatch or backend error):
  - Substrate MUST exit `1` and print an actionable message whose output contains (case-insensitive):
    - `applied but pending diffs were not cleared`
  - Substrate MUST NOT attempt to “clear whatever is current”.

### Output contract

- Non-verbose: print a deterministic summary including:
  - applied path counts by bucket,
  - skipped-by-exclude count,
  - skipped-by-conflict count (when `prefer_host`).
- Verbose: print a deterministic per-path decision line for every path in the raw pending diff set.

### Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- 0: apply succeeded (including partial skip under `prefer_host`)
- 1: unexpected internal error (e.g., filesystem operation failure mid-apply)
- 3: world backend required but unavailable
- 4: unsupported direction or backend capability
- 5: safety refusal (protected path present, size guard, or abort-on-conflict)

## Acceptance criteria

- With a non-empty pending non-PTY diff, `workspace sync --direction from_world` applies changes to the host workspace.
- If the raw pending diff includes any protected path, apply refuses (exit `5`) and does not mutate.
- Size guard refusal is enforced (exit `5`) with threshold+observed values in the message.
- Conflict policy behaves per `filesystem-semantics-spec.md`.
- After a successful apply, subsequent sync is a no-op (exit `0`, empty pending diff).

## Out of scope

- Auto-sync triggers (WS3).
- PTY pending diff discovery/apply (WS4/WS5).
- Host→world pre-sync (WS5).
