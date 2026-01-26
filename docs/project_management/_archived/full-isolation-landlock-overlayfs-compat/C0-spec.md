# C0-spec — Full isolation Landlock ↔ OverlayFS backing dirs allowlist

Authoritative ADR:
- `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Behavior is Linux-kernel-specific; this slice is validated on:
  - Linux hosts, and
  - macOS hosts via the Lima Linux guest (world backend).
- Restore the operator contract for `world_fs.write_allowlist` in:
  - `world_fs.isolation=full`
  - `world_fs.mode=writable`
  - Landlock supported by the running kernel
  - overlayfs is the active world filesystem strategy
- Extend the full-isolation Landlock write allowlist with runtime-derived overlayfs internal write roots:
  - `upperdir`
  - `workdir`
- Add fixture-based unit tests for mountinfo parsing in `crates/world`.
- Add or update world-agent integration coverage to ensure allowlisted writes succeed under full isolation when Landlock is supported.

## Non-Goals
- Policy snapshot schema changes or policy snapshot hash behavior changes.
- Any new allowlist syntax or matching semantics.
- Any behavior changes on macOS or Windows.

## Inputs (authoritative)
- Environment:
  - `SUBSTRATE_MOUNT_PROJECT_DIR`: absolute project mountpoint inside the active mount namespace.
- Procfs:
  - `/proc/self/mountinfo` (preferred and required for this slice).
- Policy-derived inputs (already present; no schema changes):
  - `world_fs.write_allowlist` (project-relative glob patterns).
  - The resolved Landlock allowlists injected by the world-agent (`SUBSTRATE_WORLD_FS_LANDLOCK_*_ALLOWLIST`).

## Behavior (authoritative)

### Runtime derivation: overlayfs backing dirs from mountinfo
- When all conditions are true:
  - isolation is full
  - filesystem mode is writable
  - Landlock is supported and will be applied for the exec
  - `SUBSTRATE_MOUNT_PROJECT_DIR` is set
- The world-agent Landlock exec wrapper MUST:
  1. Read `/proc/self/mountinfo` as text.
  2. Find the mount entry whose mountpoint equals `SUBSTRATE_MOUNT_PROJECT_DIR`.
     - If multiple entries match exactly, select the entry with the greatest numeric mount id.
  3. Require that the selected entry has:
     - `fs_type == "overlay"`, and
     - `super_options` contains both `upperdir=<path>` and `workdir=<path>` keys.
  4. Interpret the `upperdir` and `workdir` values as absolute paths in the current mount namespace.
  5. Decode mountinfo escape sequences in these values:
     - `\\040` → space
     - `\\011` → tab
     - `\\012` → newline
     - `\\134` → backslash
  6. Return the two resolved paths as the overlayfs internal write roots for this exec.

### Landlock allowlist extension
- When overlayfs internal write roots are derived successfully, the Landlock exec wrapper MUST extend the Landlock write allowlist by adding exactly these two directories:
  - `upperdir`
  - `workdir`
- These derived paths MUST NOT be added to:
  - policy snapshot schema
  - policy snapshot hash inputs
  - any on-disk policy/config files

### Enforcement invariants (must hold)
- Allowlisted project writes succeed:
  - If `world_fs.write_allowlist` covers a project-relative prefix, writes under that prefix succeed (subject to normal command behavior).
- Non-allowlisted project writes are denied:
  - If the target path is not covered by `world_fs.write_allowlist`, the write is denied.
- Derived internal write roots are scoped:
  - The added Landlock write allowlist entries are limited to the active session’s derived `upperdir` and `workdir` paths.
  - The implementation MUST NOT add broad allowlists such as `/var/lib/substrate/overlay` or `/var/lib/substrate`.

## Error handling (authoritative)

### Landlock unsupported
- If Landlock is not supported by the running kernel, this slice introduces no new failure mode:
  - the wrapper does not derive overlay backing dirs,
  - full isolation enforcement continues using mount semantics only.

### Landlock supported, derivation fails (fail closed)
- If Landlock is supported and any required derivation input is missing or invalid:
  - `SUBSTRATE_MOUNT_PROJECT_DIR` is missing/empty
  - `/proc/self/mountinfo` cannot be read
  - no mount entry matches the mountpoint
  - the matching entry is not `fs_type="overlay"`
  - `upperdir` or `workdir` is missing
- The exec MUST fail closed:
  - the command does not execute
  - the failure is surfaced as an actionable “missing prerequisites / not supported” failure for this environment:
    - exit code: `4`
- The error message MUST include:
  - the mountpoint value (`SUBSTRATE_MOUNT_PROJECT_DIR`)
  - the specific missing requirement (no match, wrong fs_type, missing upperdir/workdir)
  - a short remediation hint: “this full-isolation exec requires deriving overlayfs backing dirs from /proc/self/mountinfo”

## Validation plan (authoritative)

### Unit tests (Linux-only)
- Add fixture-based tests for mountinfo parsing in `crates/world`:
  - A fixture with an overlay mount entry for the project mountpoint returns the expected `upperdir` and `workdir`.
  - A fixture with a non-overlay entry for the project mountpoint returns a deterministic error.
  - A fixture missing `upperdir` or `workdir` returns a deterministic error.

### Integration tests (Linux-only)
- Update or add `crates/world-agent` tests so that when overlayfs is available and Landlock is supported:
  - allowlisted writes succeed in full isolation (`world_fs.mode=writable`, `world_fs.isolation=full`)
  - denied writes remain denied
  - the host project directory is not mutated by allowlisted writes (writes remain in overlay backing dirs)

### Smoke (behavior platforms: Linux + macOS)
- Linux smoke (`docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh`) MUST:
  - require that `substrate world doctor --json` reports `world.landlock.supported=true` and `world.world_fs_strategy.primary="overlay"`
  - run an allowlisted write that fails on the pre-fix behavior and succeeds after this slice
  - run a denied write that remains denied
- macOS smoke (`docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh`) MUST:
  - require that `substrate world doctor --json` reports `world.landlock.supported=true` and `world.world_fs_strategy.primary="overlay"`
  - run the same allowlisted write + denied write checks as the Linux smoke, via the macOS host → Lima guest world backend path

## Out of scope
- Any changes to how `world_fs.write_allowlist` patterns are interpreted or canonicalized.
- Any changes to macOS (Lima) or Windows (WSL) world backends.
