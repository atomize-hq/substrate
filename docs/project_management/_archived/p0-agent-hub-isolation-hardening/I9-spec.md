# I9-spec: Full Cage Robustness (`/tmp` Projects + `world verify` Full Cage)

## Context / bug
- `substrate world verify` runs its full-isolation check from a temporary project under the OS temp dir
  (typically `/tmp` on Linux).
- In `world_fs.isolation=full`, the Linux full-isolation path creates a new rootfs and mounts `/tmp` as a tmpfs
  (per I2-spec). When the host project/cwd lives under `/tmp`, the current full-isolation wrapper can
  `cd` into a host path that is no longer nameable inside the cage, causing failures like:
  - `sh: line ...: cd: /tmp/.../fullcage-project: No such file or directory`

## Scope
- Make `world_fs.isolation=full` command execution robust when the host project directory and/or desired
  working directory is under `/tmp` (or any path that becomes unavailable inside the cage).
- Ensure `substrate world verify` full-isolation check is aligned with the isolation model:
  - Do not require host-side project files to exist after a successful full-isolation run (writes are
    expected to land in the world overlay/copydiff, not mutate the host project).
  - Keep the check focused on in-cage behavior (allowlisted writes succeed; non-allowlisted and
    outside-host paths are blocked).
- Beef up tests around full isolation to prevent regressions and uncover adjacent issues (allowlist
  prefixes, `/tmp` semantics, outside-host access).

## Acceptance
- On a provisioned Linux host, `substrate world verify` passes all checks (including full isolation).
- Full-cage execution does not fail with `cd ... No such file or directory` when the project/cwd is
  under `/tmp`.
- Tests cover at least:
  - Full cage execution from a project rooted under `/tmp` (regression for the `cd` failure).
  - Allowlist prefix patterns like `./writable/*` behave as intended in full isolation.
  - Outside-host read/write attempts are blocked in full isolation.

## Out of Scope
- Changing the overall isolation contract beyond making full isolation usable for `/tmp`-rooted projects.
- Windows parity for full isolation or world verify.
- Exit code taxonomy changes (handle separately).
