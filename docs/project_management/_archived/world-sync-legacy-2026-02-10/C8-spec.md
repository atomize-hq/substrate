# C8-spec: World-side Handling of `.substrate-git`

## Scope
- Define the world-side contract for `.substrate-git` in a world-sync workspace.
- Ensure user commands running inside a world do not mutate internal git state.

## Contract (must be enforced)
- `.substrate-git/` is a protected path for sync and must never be applied as part of host↔world sync in any direction.
- Internal git operations are host-only:
  - `substrate checkpoint` and `substrate rollback` run on the host and use the host internal git directory created by C0/C6.
  - World execution paths must not create, mutate, or depend on `.substrate-git/` contents.
- If a world execution path cannot avoid touching `.substrate-git/`, it must fail closed with an explicit error and must perform no mutations.

## Acceptance
- World-side executions do not create or modify `.substrate-git/` files.
- `substrate sync` never applies diffs that touch `.substrate-git/` in any direction; the command exits non-zero with an explicit message if only protected paths are present in the diff.
- `substrate checkpoint` and `substrate rollback` operate on the host internal git only and never require world backend availability.

## Out of Scope
- Any world-side internal git mirroring or commit alignment.
