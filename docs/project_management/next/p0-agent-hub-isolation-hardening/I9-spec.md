# I9-spec: Full Cage Robustness (`/tmp` Projects + `world verify` Full Cage)

## Context / bug
- `substrate world verify` runs its full-cage check from a temporary project under the OS temp dir
  (typically `/tmp` on Linux).
- In `world_fs.cage=full`, the Linux full-cage path creates a new rootfs and mounts `/tmp` as a tmpfs
  (per I2-spec). When the host project/cwd lives under `/tmp`, the current full-cage wrapper can
  `cd` into a host path that is no longer nameable inside the cage, causing failures like:
  - `sh: line ...: cd: /tmp/.../fullcage-project: No such file or directory`

## Scope
- Make `world_fs.cage=full` command execution robust when the host project directory and/or desired
  working directory is under `/tmp` (or any path that becomes unavailable inside the cage).
- Ensure `substrate world verify` full-cage check is aligned with the isolation model:
  - Do not require host-side project files to exist after a successful full-cage run (writes are
    expected to land in the world overlay/copydiff, not mutate the host project).
  - Keep the check focused on in-cage behavior (allowlisted writes succeed; non-allowlisted and
    outside-host paths are blocked).
- Beef up tests around full cage to prevent regressions and uncover adjacent issues (allowlist
  prefixes, `/tmp` semantics, outside-host access).

## Acceptance
- On a provisioned Linux host, `substrate world verify` passes all checks (including full cage).
- Full-cage execution does not fail with `cd ... No such file or directory` when the project/cwd is
  under `/tmp`.
- Tests cover at least:
  - Full cage execution from a project rooted under `/tmp` (regression for the `cd` failure).
  - Allowlist prefix patterns like `./writable/*` behave as intended in full cage.
  - Outside-host read/write attempts are blocked in full cage.

## Out of Scope
- Changing the overall isolation contract beyond making full cage usable for `/tmp`-rooted projects.
- Windows parity for full cage or world verify.
- Exit code taxonomy changes (handle separately).

