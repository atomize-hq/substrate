# Workspace Sync — Filesystem Model (Current Linux Reality)

Scope: this document explains how `substrate workspace sync` behaves today on Linux given the
current overlay-based world implementation. It is developer-facing and grounded in the current
shell, world-service, and overlayfs code paths.

If you are looking for the operator-facing command contract, see
`docs/reference/cli/workspace_sync.md`.

## Mental model (one sentence)

On Linux, the world filesystem is the live host workspace as the overlay lower layer plus an
optional world-only overlay upper layer for pending changes; `workspace sync` controls when the
world-only layer is applied back to the host and when conflicting shadowed paths are discarded.

## Glossary

- Host workspace: the real directory on the developer machine (for example `/home/<user>/repo`).
- World session: a reusable isolated context created by `world-service` for a compatible
  `WorldSpec`.
- Overlay lower: the live host workspace directory used as the overlay baseline.
- Overlay upper/work: where world-only writes and deletes are materialized.
- Overlay merged: what world processes see when reading or writing the project directory.
- Pending diff: the current workspace-relative set of writes, mods, and deletes represented by the
  overlay upper/work layer.
- Shadowed path: a host path whose current host state is hidden by a pending world-side overlay
  entry for the same relative path.

## Stable surfaces for this behavior

- Operator overview: `docs/reference/cli/workspace_sync.md`
- Related operator history commands: `docs/reference/cli/workspace_history.md`
- Current implementation details and code pointers: this document
- Auto-sync hook behavior: `crates/shell/src/execution/auto_sync.rs`

## Where the behavior lives (code pointers)

Shell (`workspace sync` CLI, filtering, conflict policy, apply, and clear):
- `crates/shell/src/execution/workspace_cmd.rs`

World agent (pending diff record, conditional clear, reconciliation):
- `crates/world-service/src/service.rs`
  - `pending_diff()` builds the record and computes `diff_id`
  - `pending_diff_clear()` conditionally clears by `diff_id`
  - `pending_diff_reconcile()` discards selected overlay paths by `diff_id`

World backend and overlay tracking:
- `crates/world/src/lib.rs`
  - `LinuxLocalBackend::{pending_diff, clear_pending_diff, discard_pending_paths}`
- `crates/world/src/session.rs`
  - `SessionWorld::clear_pending_diff()` resets the overlay upper/work layer
- `crates/world/src/overlayfs/mod.rs`
  - overlay mount/unmount and `discard_paths()`
- `crates/world/src/overlayfs/utils.rs`
  - pending diff computation and whiteout interpretation

## Stable rules enforced today

### 1. All diff paths are workspace-root-relative

`workspace sync` treats all diff and reconciliation paths as workspace-root-relative strings.

Normalization rules enforced by the shell and world-service:
- normalize separators to forward slashes
- strip a leading `./`
- reject empty paths
- reject absolute paths
- reject Windows drive-prefixed absolute paths
- reject any path containing a `..` segment

Behavior:
- an invalid pending diff path causes a fail-closed sync refusal
- an invalid `discard_paths` reconciliation request is rejected by the agent

### 2. Protected paths are a hard fail-closed boundary

Protected excludes are injected automatically:
- `.git/**`
- `.substrate/**`

If the raw pending diff contains any protected path, `workspace sync` refuses before mutation.
The shell reports the offending paths and exits without applying or clearing anything.

This same protection also prevents checkpoint and rollback from mutating those paths.

### 3. Exclude patterns are applied after protected-path refusal

Effective excludes come from:
- protected excludes injected by the shell
- config `sync.exclude`
- CLI `workspace sync --exclude <PATTERN>`

Pattern behavior:
- `*` matches within a path segment
- `**` matches across `/`
- `?` matches a single non-`/` character
- patterns are case-sensitive
- patterns must not start with `/`

The shell removes duplicate patterns while preserving order. Excluded paths are skipped from the
apply set and counted in sync output, but excluded paths do not override protected-path refusal.

### 4. Pending diffs are bucketed and normalized before decisions

The pending diff record returned by the agent contains:
- `session_started_at`
- `diff_id`
- a required `non_pty` bucket
- an optional `pty` bucket

Each bucket contains disjoint normalized path lists:
- `writes`
- `mods`
- `deletes`

The shell:
- normalizes and validates raw paths
- rejects protected paths
- applies exclude filtering
- sorts and de-duplicates bucket contents
- computes a combined apply set from non-PTY and PTY buckets

Implementation note:
- `diff_id` is intentionally stable across harmless reclassification of a path between `writes`
  and `mods`
- it changes when the effective updates or deletes set changes
- this prevents false clear failures after host apply changes the lower-layer existence check

### 5. Host changes are usually visible in world immediately

Because overlay lower is the live host workspace, host create, modify, and delete operations are
normally visible in world reads immediately.

The main exception is a shadowed path:
- if the world has a pending upper-layer entry for a path, the merged overlay can continue showing
  the world version instead of the host version until that upper-layer entry is discarded or the
  pending diff is cleared

### 6. World changes accumulate as overlay upper entries

World-side filesystem changes under the project root are represented in overlay upper/work until
they are cleared or selectively discarded:
- creates and modifications materialize as upper-layer files or directories
- deletes materialize as overlay whiteouts

Pending diff computation walks the overlay upper layer and interprets whiteouts accordingly.

### 7. Direction semantics are asymmetric

`workspace sync` resolves an effective direction:

- `from_world`
  - apply pending world diffs to the host
  - host is mutated
  - both non-PTY and PTY pending diffs are included when present

- `from_host`
  - reconcile host-newer shadowed paths back into the world overlay
  - host is never mutated
  - this is not an upload or copy operation because the host is already the lower-layer baseline

- `both`
  - run `from_host` reconciliation first
  - re-fetch the pending diff snapshot
  - then run `from_world` apply on the updated snapshot

Important nuance for `from_host`:
- the shell computes conflicts only for shadowed paths whose host mtime is newer than
  `session_started_at`
- `prefer_host` discards the world overlay entry for those conflicting paths only
- `prefer_world` keeps the overlay entry
- `abort` refuses if any such conflict exists

### 8. Conflict detection uses world session start time

For reconciliation and apply, a host path is considered conflicting when:
- the host path exists, and
- its mtime is strictly greater than `session_started_at`

This is a coarse policy gate, not a three-way merge.

Conflict policy behavior:
- `prefer_host`
  - `from_host`: discard conflicting overlay entries so the world observes the host version
  - `from_world`: skip conflicting paths from apply
- `prefer_world`
  - keep world pending changes and apply them to host when relevant
- `abort`
  - refuse the operation if any conflict exists

### 9. Apply preflight is fail-closed

Before any host mutation, the shell performs all-or-nothing validation:
- protected path refusal
- exclude filtering
- `max_paths = 10000`
- `max_bytes_to_copy = 104857600` across selected writes and mods
- abort-on-conflict when policy is `abort`
- backend capability checks for pending diff clear and world file reads

If any preflight guard fails, no host mutations occur.

### 10. Only regular files and directories can be applied

For selected writes and mods, the shell reads current world metadata first.

Apply accepts:
- directories
- regular files

Apply refuses fail-closed for other file types, including:
- symlinks
- sockets
- device nodes
- FIFOs

This is why overlayfs symlink entries are still tracked in pending diffs: sync must see them and
refuse safely rather than silently dropping them.

### 11. Apply ordering and content preservation are deterministic

After validation passes, apply order is deterministic:
1. Deletes, sorted by path depth descending and then lexicographically descending
2. Writes and mods, sorted lexicographically ascending

Apply behavior:
- deletes remove files directly and directories recursively
- directory writes create parent directories as needed
- file writes read current world bytes at apply time and write them atomically to host
- execute bits are preserved on Unix for directories and regular files

Failure model:
- after preflight passes, apply is best-effort
- the shell stops at the first filesystem failure
- already-applied mutations are not rolled back

### 12. Clearing pending diffs is conditional on `diff_id`

After a successful `from_world` apply, the shell attempts to clear the applied snapshot by sending
the same `diff_id` back to `pending_diff_clear()`.

Rules:
- `--dry-run` never clears
- the agent clears only if the current pending diff still hashes to the same `diff_id`
- a mismatch means new or changed pending overlay state arrived concurrently

If clear fails:
- host mutations remain applied
- the command exits as a failure
- the shell prints `applied but pending diffs were not cleared`

The shell never clears "whatever is current."

## Dry-run and verbose behavior

`workspace sync --dry-run` does not mutate host or overlay state.

Current output shape:
- `from_host` dry-run prints a reconciliation plan summary
- `from_world` dry-run prints non-PTY, optional PTY, and combined pending diff summaries
- `--verbose` includes `session_started_at`, `diff_id`, and per-path decisions

For apply:
- non-verbose output reports applied counts plus skipped-by-exclude and skipped-by-conflict totals
- verbose output includes per-path apply or skip decisions

## Common "surprise" scenarios (expected today)

### Scenario: "I synced a world file to host; deleting it on host also deletes it in world"

This is expected.

Once a world-created file is applied to host and the pending diff is cleared, that file becomes
part of the shared host lower layer. Deleting it on host deletes it from what the world sees.

### Scenario: "`from_host` doesn't upload host-only new files to world"

Also expected.

Host-only files are already visible in world through the lower layer. `from_host` only exists to
resolve cases where a pending world overlay entry is hiding the host version of a shadowed path.

### Scenario: "A host edit still loses to the world version until I reconcile or clear"

Also expected.

If the path is shadowed by a pending upper-layer entry, the world can keep seeing the world version
until `from_host` discards that upper entry or a later clear resets the overlay state.

## Auto-sync (`sync.auto_sync=true`) — current reality

When `sync.auto_sync=true`, the shell opportunistically runs the same sync engine after successful
commands:

- one-shot execution path
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
- async REPL path
  - `crates/shell/src/repl/async_repl.rs`

Behavior:
- `sync.direction=from_host` results in no auto-sync apply
- `sync.direction=from_world` or `both` runs the sync engine automatically
- failures surface as `auto-sync failed: ...` and propagate a non-zero exit code

## Debugging workflows

### Preview decisions safely

- `substrate workspace sync --dry-run --verbose`

Useful details:
- `session_started_at`
- `diff_id`
- non-PTY and PTY counts
- excluded counts
- conflict decisions

### Inspect Linux overlay storage

Overlay base directory selection:
- `crates/world/src/overlayfs/layering.rs` (`choose_base_dir()`)

Per-session overlay directories:
- `crates/world/src/session.rs` (world ids and overlay ownership)

### Inspect why a clear or reconcile refused

Look for:
- `diff_id mismatch (concurrent changes detected)`
- protected path refusal output
- unsupported file type refusal output

These messages map directly to the shell's fail-closed branches in
`crates/shell/src/execution/workspace_cmd.rs`.
