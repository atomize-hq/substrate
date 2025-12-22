# I2-spec: Full Cage (Non-PTY) via Mount Namespace + pivot_root

## Scope
- Implement full caging for non-PTY executions on Linux:
  - New mount namespace, build minimal rootfs, mount/bind only allowed paths, then `pivot_root`.
  - Ensure the process cannot access host paths outside the cage (by name).
- Minimum viable “full cage” rootfs:
  - project root mounted at a stable location (e.g., `/project`)
  - required system mounts for typical dev tools:
    - `/usr`, `/bin`, `/lib*`, `/etc` as read-only binds (or documented minimal set)
  - `/proc` mounted fresh
  - `/tmp` writable (tmpfs or bind)
  - minimal `/dev` requirements documented (or bind `/dev` read-only if needed)
- Enforce `world_fs.mode`:
  - `read_only` must prevent writes to the project (and to any other non-whitelisted paths).
- Enforce allowlists (if present in schema) at least for project writes (full allowlist enforcement may be Landlock-based in I4; this spec must be explicit about what is enforced by pivot_root alone).
- Capability/privilege detection:
  - If full cage is required and cannot be created → fail closed with a clear diagnostic.
  - If full cage is optional and cannot be created → degrade with a warning (only if policy allows).

## Acceptance
- With `world_fs.cage=full`, commands cannot write to or read from arbitrary host paths outside the allowed mounts.
- With `world_fs.mode=read_only`, project writes fail (both relative and absolute paths).
- Clear failure mode on hosts without required privileges (no silent degrade when required).

## Out of Scope
- PTY/interactive parity — I3.
- Landlock allowlist enforcement — I4.

