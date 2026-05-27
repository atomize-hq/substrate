# `substrate workspace checkpoint` and `workspace rollback`

This page defines the stable operator-facing contract for Substrate's workspace history commands.

If you want the deeper implementation details for the internal git store, see:
`docs/internals/world/workspace_internal_git_model.md`.

## What these commands do

- `substrate workspace checkpoint`
  - records a snapshot of the current workspace into Substrate's internal history store
- `substrate workspace rollback <target>`
  - restores the workspace to a previously recorded checkpoint

These commands do not use or mutate the user's `.git/` repository. They use a separate
Substrate-owned store under `.substrate/`.

## Internal store path

Substrate stores workspace-history state at:

- git dir: `<workspace_root>/.substrate/git/repo.git/`
- work tree: `<workspace_root>/`

Protected paths remain outside the snapshot and restore surface:

- `.git/**`
- `.substrate/**`

## `substrate workspace checkpoint`

Purpose:

- capture the current workspace state into the internal Substrate checkpoint store

Flags:

- `--message <TEXT>`
  - optional operator-supplied suffix for the checkpoint commit message
- `--verbose`
  - prints additional checkpoint details

Behavior:

- if the internal history store is not initialized yet, Substrate initializes it on first use
- the checkpoint snapshot includes workspace files except protected paths
- if nothing changed since the last internal checkpoint, the command succeeds as a no-op
- the printed checkpoint id is the stable target you can later pass to rollback

Exit codes:

- `0`
  - success, including no-op
- `2`
  - not in a workspace or invalid flag value
- `3`
  - required dependency unavailable, such as `git`
- `5`
  - safety-rail refusal

## `substrate workspace rollback <target>`

Purpose:

- restore the workspace to a previously recorded Substrate checkpoint

Targets:

- `last`
  - restore the most recent checkpoint
- `<CHECKPOINT_ID>`
  - restore a specific checkpoint id printed by checkpoint

Flags:

- `--force`
  - allows rollback to proceed when safety rails would otherwise refuse
- `--verbose`
  - prints additional rollback details

Behavior:

- rollback restores the workspace to the selected checkpoint from Substrate's internal store
- rollback never mutates the user's `.git/`
- rollback does not create a new checkpoint as a side effect
- when `--force` is not present, rollback refuses if safety rails trigger

## Safety rails

Rollback is intentionally conservative.

Without `--force`, rollback refuses when:

- the user repository `.git/` exists and `git status --porcelain` is non-empty
- non-protected workspace paths exist that are not present in the target checkpoint snapshot

With `--force`, rollback may delete non-protected paths that are outside the target checkpoint
snapshot in order to make the workspace match the checkpoint.

## Snapshot and restore boundaries

Checkpoint and rollback both operate on the workspace root while excluding:

- `.git/**`
- `.substrate/**`

This means:

- Substrate does not overwrite the user's git metadata
- Substrate does not snapshot or restore its own runtime state as part of workspace history

## Related commands

- `substrate workspace sync`
  - applies or reconciles pending world changes; it is not a checkpointing surface

For the current sync behavior, see:

- `docs/reference/cli/workspace_sync.md`
