# I2-spec: Full Cage (Non-PTY) via Mount Namespace + pivot_root

## Scope
- Implement full caging for non-PTY executions on Linux:
  - New mount namespace, build minimal rootfs, mount/bind only allowed paths, then `pivot_root`.
  - Ensure the process cannot access host paths outside the cage (by name).
- Minimum viable “full cage” rootfs:
  - project root mounted at a stable location (e.g., `/project`)
  - required system mounts for typical dev tools:
    - `/usr`, `/bin`, `/lib`, `/lib64`, `/etc` as read-only bind mounts
  - `/proc` mounted fresh
  - `/tmp` mounted as tmpfs (writable)
  - `/dev` bind-mounted read-only
  - `/var/lib/substrate/world-deps` bind-mounted read-write
- Enforce `world_fs.mode`:
  - `read_only` must prevent writes to the project (and to any other non-whitelisted paths).
- Enforce allowlists for the project mount:
  - Mount the project as read-only by default.
  - For each `world_fs.write_allowlist` entry that resolves under the project root, bind-mount the matching path as writable inside the cage.
- Capability/privilege detection:
  - If `world_fs.cage=full` cannot be created, Substrate must fail closed with a clear diagnostic (I1).

## Acceptance
- With `world_fs.cage=full`, commands cannot write to or read from arbitrary host paths outside the allowed mounts.
- With `world_fs.mode=read_only`, project writes fail (both relative and absolute paths).
- Clear failure mode on hosts without required privileges (no silent degrade when required).

## Out of Scope
- PTY/interactive parity — I3.
- Landlock allowlist enforcement — I4.
