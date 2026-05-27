# Workspace Internal Git Model

Scope: this document explains the current internal git store used by `substrate workspace
checkpoint` and `substrate workspace rollback`. It is developer-facing and grounded in the current
workspace command implementation.

If you are looking for the operator-facing command contract, see
`docs/reference/cli/workspace_history.md`.

## Stable surfaces for this behavior

- Operator contract:
  - `docs/reference/cli/workspace_history.md`
- Related sync surface:
  - `docs/reference/cli/workspace_sync.md`
  - `docs/internals/world/workspace_sync_filesystem_model.md`

## Where the behavior lives

Shell workspace command implementation:

- `crates/shell/src/execution/workspace_cmd.rs`

The current implementation shells out to the system `git` binary with explicit internal-repo
arguments instead of discovering a repository from `cwd`.

## Internal store layout

Substrate's internal history store uses:

- GIT_DIR:
  - `<workspace_root>/.substrate/git/repo.git/`
- WORK_TREE:
  - `<workspace_root>/`

Rules:

- the internal git dir is separate from the user's `.git/`
- the internal store is a Substrate-owned implementation detail under `.substrate/`
- checkpoint and rollback must not rely on the user's repo config, hooks, or identity

## Initialization

When checkpoint or rollback needs the internal store and `HEAD` does not exist yet, Substrate
initializes the internal repo.

Current invariants:

- git is invoked with explicit `--git-dir` and `--work-tree`
- internal commits use deterministic Substrate-owned author and committer identity
- signing is disabled for internal commits and tags

This keeps internal workspace-history operations independent from user-global git configuration.

## Snapshot boundary

Internal checkpoints include workspace content except:

- `.git/**`
- `.substrate/**`

Those exclusions are part of the behavior contract, not just an incidental implementation detail.

Consequences:

- user git metadata is never snapshotted or restored
- Substrate runtime state is never snapshotted or restored as workspace history

## Checkpoint identifiers

Checkpoint ids are lightweight tags with a sortable UTC timestamp shape:

- `cp/<YYYYMMDDTHHMMSSZ>`

Example:

- `cp/20260210T183823Z`

Ordering:

- the most recent checkpoint is the lexicographically greatest checkpoint id
- `workspace rollback last` resolves using that ordering

## Checkpoint behavior

Current checkpoint flow:

1. resolve the workspace root
2. ensure the internal repo is initialized
3. stage all included paths while excluding protected paths
4. if nothing changed versus internal `HEAD`, exit successfully as a no-op
5. create an internal commit
6. create a lightweight checkpoint tag
7. print the checkpoint id

Checkpoint does not depend on the user's `.gitignore` for correctness.

## Rollback behavior

Current rollback flow:

1. resolve the target checkpoint id or `last`
2. enforce safety rails unless `--force` is present
3. resolve the target commit from the checkpoint tag
4. restore the internal work tree to the target commit
5. when forced, delete non-protected workspace paths that are outside the target snapshot

Rollback invariants:

- rollback does not create a new checkpoint
- rollback does not mutate `.git/**` or `.substrate/**`
- rollback uses the internal git store as the source of truth for the target snapshot

## Safety rails

Without `--force`, rollback refuses when:

- the user repo exists and is dirty according to `git status --porcelain`
- non-protected workspace paths exist that are not present in the target checkpoint snapshot

With `--force`, rollback may delete non-protected workspace paths to make the workspace match the
target checkpoint.

## Snapshot presence model

A path counts as present in the target checkpoint snapshot when it is:

- a file path recorded in the target tree, or
- a parent directory required to contain one of those recorded file paths

This is why rollback can safely distinguish between:

- paths that should exist after restore, and
- non-protected extra paths that should be removed only under `--force`

## Relationship to workspace sync

Workspace history and workspace sync solve different problems:

- `workspace sync`
  - reconciles pending world overlay changes with the host workspace
- `workspace checkpoint` / `workspace rollback`
  - snapshot and restore host workspace state through Substrate's internal git store

The protected-path boundary is shared across both surfaces:

- `.git/**`
- `.substrate/**`
