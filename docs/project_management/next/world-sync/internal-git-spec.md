# world-sync — internal git spec

Template reference: `docs/project_management/standards/templates/spec/schema-spec.md.tmpl` (adapted; internal git is a filesystem+workflow contract).

## Scope
This spec defines the deterministic contract for Substrate’s internal checkpoint/rollback store.

Owned surfaces:
- Internal git directory path + initialization behavior
- Checkpoint identifiers (tag names) and ordering
- What is included/excluded in snapshots
- Rollback safety rails and exact restore semantics

Non-owned surfaces:
- CLI flag names/defaults (owned by `contract.md`)
- Sync diff semantics (owned by `filesystem-semantics-spec.md`)

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

Restore semantics (exact):
1) Compute the target commit from the checkpoint tag.
2) Replace workspace contents for all paths tracked by the internal checkpoint to match the target commit.
3) Delete any non-protected workspace paths that are not present in the target checkpoint snapshot (only when `--force`).

Exit code:
- `0` on success.
- `2` on invalid target.
- `3` if `git` is unavailable.
- `5` on safety-rail refusal.

