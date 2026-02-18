# world-sync — internal git spec

Template reference: `docs/project_management/standards/templates/spec/schema-spec.md.tmpl` (adapted; internal git is a filesystem+workflow contract).

## Scope
This spec defines the deterministic contract for Substrate’s internal checkpoint/rollback store.

Owned surfaces:
- Internal git directory path + initialization behavior
- Checkpoint identifiers (tag names) and ordering
- What is included/excluded in snapshots
- Rollback safety rails and exact restore semantics

Clarification (non-negotiable):
- Substrate leverages the standard `git` executable as an implementation tool, but it does **not** reuse or modify the user’s `.git/`.
- The internal store is a separate git directory under `.substrate/` (see “Internal git directory”), so there is no collision with user repos.

Non-owned surfaces:
- CLI flag names/defaults (owned by `contract.md`)
- Sync diff semantics (owned by `filesystem-semantics-spec.md`)

Non-goals (explicit; out of scope for this Planning Pack):
- Per-command commit history for every mutating command (command-level commits).
- “Undo last command” / command-scoped rollback.
- Session-level tagging (“session closed”) and session-scoped rollback points.
- History compaction / squashing / retention policies and `git gc` workflows.
- Persisted metadata indexes mapping trace spans/command ids ↔ internal git commits.
- Concurrency model for multiple agents/sessions writing to the same internal store.

Note:
- The broader product intent for internal git (including per-command history, compaction, and richer rollback UX) is described in:
  - `docs/project_management/future/INTERNAL_GIT.md` (PRD input; non-authoritative for this Planning Pack).

## Internal git directory (authoritative)
- Git directory (GIT_DIR):
  - `<workspace_root>/.substrate/git/repo.git/`
- Work tree (WORK_TREE):
  - `<workspace_root>/`

Rules:
- The internal git dir MUST be created by `substrate workspace init` (directory only).
- The internal git dir MUST NOT be created by any other command as a side effect.

## Initialization (authoritative; DR-0005)
When `checkpoint` or `rollback` runs:
- If `<workspace_root>/.substrate/git/repo.git/HEAD` does not exist, Substrate MUST initialize the internal repo using:
  - `git --git-dir "<.../repo.git>" --work-tree "<workspace_root>" init --initial-branch=main`

Git execution invariants (authoritative):
- Substrate MUST invoke `git` with explicit `--git-dir` and `--work-tree` (never relying on `cwd` discovery).
- Substrate MUST ensure internal operations do not require user-global git identity or signing configuration:
  - Set a deterministic author/committer identity for internal commits:
    - `GIT_AUTHOR_NAME=Substrate`
    - `GIT_AUTHOR_EMAIL=substrate@localhost`
    - `GIT_COMMITTER_NAME=Substrate`
    - `GIT_COMMITTER_EMAIL=substrate@localhost`
  - Disable signing for internal commits and tags (e.g., `-c commit.gpgsign=false` and `-c tag.gpgSign=false`).

## Snapshot inclusion/exclusion (authoritative)
Internal checkpoints include every path under `<workspace_root>/` **except**:
- `.git/**` (user repo metadata, if present)
- `.substrate/**` (Substrate workspace state and internal git itself)

These exclusions are enforced by Substrate (not by relying on the user repo’s `.gitignore`).

## Checkpoint ID (authoritative)
Checkpoint IDs are tag names with a monotonically increasing UTC timestamp:
- Format: `cp/<YYYYMMDDTHHMMSSZ>`
- Example: `cp/20260210T183823Z`

Ordering:
- “Most recent” (`last`) is the lexicographically greatest checkpoint id because the timestamp format is sortable.

## Checkpoint operation (authoritative)
`substrate workspace checkpoint` performs:
1) Ensure workspace root exists.
2) Ensure internal git is initialized (see Initialization).
3) Stage all included paths (see Snapshot inclusion/exclusion).
   - This means “stage the entire work tree” minus the protected excludes.
   - Implementation MUST NOT rely on `.gitignore` for correctness.
   - Example (non-normative) staging invocation:
     - `git --git-dir "<.../repo.git>" --work-tree "<workspace_root>" add -A -- . ':!.git' ':!.substrate'`
4) If there are no staged changes vs current internal HEAD:
   - Print `no-op` message and exit `0`.
5) Create a commit:
   - Commit message:
     - If `--message` provided: `checkpoint: <CHECKPOINT_ID> <message>`
     - Else: `checkpoint: <CHECKPOINT_ID>`
6) Create a lightweight tag at that commit:
   - Tag name: `<CHECKPOINT_ID>`
7) Print the created checkpoint id.

## Rollback operation (authoritative)
`substrate workspace rollback <target>` restores the workspace to a checkpoint.

Target resolution:
- `last` resolves to the most recent checkpoint id (Ordering).
- `<CHECKPOINT_ID>` must exist as a tag; otherwise exit `2` with a clear message.

Safety rails:
- If `.git/` exists and `git status --porcelain` is non-empty:
  - Without `--force`: exit `5` and perform no mutations.
  - With `--force`: proceed.
- If there exist any non-protected paths in the workspace that are **not present** in the target checkpoint snapshot:
  - Without `--force`: exit `5` and perform no mutations.
  - With `--force`: these paths are deleted as part of rollback.

Definition: “present in the target checkpoint snapshot” (authoritative)
- Let `snapshot_files` be the set of workspace-relative file paths in the target commit:
  - Example (non-normative) enumeration:
    - `git --git-dir "<.../repo.git>" --work-tree "<workspace_root>" ls-tree -r --name-only "<target_commit>"`
- Let `snapshot_required_paths` be:
  - every path in `snapshot_files`, plus
  - every parent directory of every path in `snapshot_files`.
- A workspace path is “present in the snapshot” iff it is in `snapshot_required_paths`.

Restore semantics (exact):
1) Compute the target commit from the checkpoint tag.
2) Update the internal repo’s `main` branch to the target commit and restore the work tree to match.
3) Delete any non-protected workspace paths that are not present in the target checkpoint snapshot (only when `--force`).

Rollback behavior notes (authoritative):
- Rollback MUST NOT create a new checkpoint commit or tag as a side effect.
- Example (non-normative) restore sequence:
  - `git --git-dir "<.../repo.git>" --work-tree "<workspace_root>" checkout -f main`
  - `git --git-dir "<.../repo.git>" --work-tree "<workspace_root>" reset --hard "<target_commit>"`

Exit code:
- `0` on success.
- `2` on invalid target.
- `3` if `git` is unavailable.
- `5` on safety-rail refusal.
