# world-sync — filesystem semantics spec

Template: `docs/project_management/system/templates/spec/filesystem-semantics-spec.md.tmpl`

## Scope
This spec defines the authoritative filesystem/path semantics for:
- `substrate workspace sync` (preview + apply; pending diffs)
- `substrate workspace checkpoint` / `substrate workspace rollback` (what is included/excluded)

This spec does **not** define:
- CLI flag names/defaults (owned by `contract.md`)
- Internal git commit/tag formats (owned by `internal-git-spec.md`)

## Path model (authoritative)
- All paths in world-sync diffs are interpreted as **workspace-root-relative** paths.
- Path normalization:
  - Normalize to a forward-slash (`/`) separated relative path for matching and logging.
  - Reject absolute paths (leading `/` or platform drive prefixes) as a safety violation (exit `5`).
  - Reject paths containing `..` segments after normalization as a safety violation (exit `5`).

## Protected paths (authoritative; fail-closed)
Protected excludes are injected by config and are treated as a hard safety boundary:
- `.git/**`
- `.substrate/**`

Rules:
- If the raw pending diff contains **any** path that matches a protected pattern, `workspace sync` MUST:
  - perform **no mutations**,
  - print a clear “protected path refusal” message including the offending paths,
  - exit `5`.
- `checkpoint`/`rollback` MUST never include or mutate protected paths:
  - `.substrate/` and `.git/` are excluded from internal git snapshots and are never restored.

## Exclude patterns (authoritative)
Exclude patterns come from:
- Effective config `sync.exclude` (already includes protected excludes at the front), and
- `workspace sync --exclude <PATTERN>` (repeatable; appended for this invocation only).

Pattern grammar (matched against normalized workspace-relative paths):
- `*` matches any sequence of characters except `/`.
- `**` matches any sequence of characters including `/`.
- `?` matches any single character except `/`.
- Patterns are case-sensitive.
- Patterns MUST NOT start with `/` (leading `/` is a usage error; exit `2`).

Matching rule:
- A diff path is excluded if it matches **any** exclude pattern.

## Pending diff semantics (authoritative)
### Pending diff record (authoritative; implementation contract)
Pending diff discovery returns a single “pending diff record” for the current world session that includes:
- `session_started_at`: UTC timestamp of session start (RFC3339, e.g., `2026-02-10T18:38:23Z`)
  - Used as the conflict baseline (see Conflict detection).
- `diff_id`: opaque identifier for the returned pending diff snapshot
  - MUST change whenever the underlying pending diff set changes (updates = `writes ∪ mods`, plus `deletes`).
  - MUST NOT change solely because a path is reclassified between `writes` and `mods` (e.g., due to lower-layer existence changes during host apply).
  - Used to safely acknowledge/clear diffs after apply (see Clear/ack semantics).
- One or two buckets of disjoint path sets:
  - `non_pty`: pending changes from non-PTY world executions
  - `pty` (optional): pending changes from PTY world executions

Each bucket contains three disjoint path sets (lists of normalized workspace-relative paths):
- `writes`: paths created by the world session
- `mods`: paths modified by the world session
- `deletes`: paths removed by the world session

Backend capability contract (authoritative)
- The world backend MUST provide the following capabilities for world-sync to be supported:
  - Discover the pending diff record (including `session_started_at` and `diff_id`).
  - Read the current file type/bytes for any path in `writes|mods` at apply time.
  - Acknowledge/clear a pending diff snapshot by `diff_id` (conditional clear).

Rules:
- Paths MUST be treated as files unless the filesystem indicates a directory at apply time.
- Deletions:
  - When applying a delete entry, Substrate removes the target path.
  - If the target is a directory, Substrate deletes it recursively.

### Content source + permissions (authoritative)
For any path in `writes` or `mods` that is selected for apply (after filtering/conflict policy):
- Substrate MUST read the current content from the world session filesystem at apply time.
- Substrate MUST ensure parent directories exist on the host before writing a file (create missing parents as directories).
- If the source path does not exist at apply time, Substrate MUST exit `1` (unexpected internal error) and stop applying further paths.
- Substrate MUST preserve the source type and permissions:
  - Directories: create as directories (never as files) and preserve execute bits (`0o111`) at minimum.
  - Regular files: write bytes exactly and preserve execute bits (`0o111`) at minimum.

### Coalescing rules (authoritative)
When multiple events affect the same path within a session (or within a single pending bucket), the pending diff MUST be coalesced so each path appears in exactly one of `writes|mods|deletes`.

Coalescing rule (last-op-wins):
- If a path is deleted at any point after it was written/modified, it ends in `deletes`.
- If a path is created (not present at session start) and later modified, it ends in `writes`.
- If a path is deleted and later recreated, it ends in `writes`.
- If a path existed at session start and is modified (and not later deleted), it ends in `mods`.

## Size guards (authoritative; fail-closed)
Before any apply, Substrate validates the pending diff against these hard limits:
- `max_paths = 10000` (writes + mods + deletes)
- `max_bytes_to_copy = 104857600` (100 MiB) for the sum of file sizes of all `writes` and `mods` **after** exclude filtering but **before** any mutation.

Guard behavior:
- If either limit is exceeded, `workspace sync` MUST:
  - perform **no mutations**,
  - print a message containing the observed values and the thresholds,
  - exit `5`.

## Conflict detection (authoritative; DR-0004)
Conflict detection is required only for **apply** operations (not for `--dry-run`):
- A path is considered “in conflict” when:
  - the host workspace path exists, AND
  - its `mtime` is strictly greater than the world session’s `session_started_at` timestamp.

Conflict policy application:
- `prefer_host`:
  - Do not apply conflicting paths.
  - Apply non-conflicting paths.
  - Exit `0` (success) and print a summary including skipped conflicts.
- `prefer_world`:
  - Apply conflicting paths (overwrite/delete).
  - Exit `0`.
- `abort`:
  - If **any** conflict exists, apply nothing and exit `5` with a clear message listing conflicts.

## Special file types (authoritative; fail-closed)
Substrate must refuse to apply changes that would create or overwrite:
- symlinks,
- sockets,
- device nodes,
- FIFOs.

Behavior:
- If the apply set contains a path whose source in the world overlay is not a regular file or directory, Substrate MUST:
  - perform **no mutations**,
  - exit `5`,
  - print the path and detected file type.

## Apply ordering (authoritative)
When applying a diff (after validation passes), Substrate applies in deterministic order:
1) Deletes: sort by path depth descending, then lexicographically descending.
2) Writes + mods: sort paths ascending by string order.

## Atomicity / failure model (authoritative)
- Preflight validation is all-or-nothing:
  - protected path scan
  - exclude filtering
  - size guards
  - conflict detection (when policy=`abort`)
- After validation passes, apply is best-effort:
  - Substrate stops at the first filesystem operation failure, exits `1`, and does not attempt to revert already-applied mutations.

## Clear/ack semantics (authoritative)
After a successful apply (exit `0`), Substrate MUST acknowledge/clear the applied pending diff so subsequent `workspace sync` is a no-op for the applied bucket(s).

Rules:
- `--dry-run` MUST NOT clear diffs.
- Clearing MUST be conditional on the `diff_id` that was applied:
  - If the backend reports a different `diff_id` by the time Substrate attempts to clear, Substrate MUST treat the clear as failed (do not clear “whatever is current”).
  - This prevents dropping new pending changes that arrived concurrently.
