# world-sync — filesystem semantics spec

Template: `docs/project_management/standards/templates/spec/filesystem-semantics-spec.md.tmpl`

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
Pending diffs are represented as three disjoint path sets:
- `writes`: paths created by the world session
- `mods`: paths modified by the world session
- `deletes`: paths removed by the world session

Rules:
- Paths MUST be treated as files unless the filesystem indicates a directory at apply time.
- Deletions:
  - When applying a delete entry, Substrate removes the target path.
  - If the target is a directory, Substrate deletes it recursively.

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
1) Deletes: sort paths descending by string order (deepest paths are deleted first in typical lexical hierarchies).
2) Writes + mods: sort paths ascending by string order.

## Atomicity / failure model (authoritative)
- Preflight validation is all-or-nothing:
  - protected path scan
  - exclude filtering
  - size guards
  - conflict detection (when policy=`abort`)
- After validation passes, apply is best-effort:
  - If any filesystem operation fails mid-apply, Substrate exits `1` and does not attempt to revert already-applied mutations.

