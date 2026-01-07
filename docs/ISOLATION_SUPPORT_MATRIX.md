# World FS Isolation Support Matrix (Linux vs macOS/Lima)

This document compares `world_fs.isolation` behavior across **Linux** (native) and **macOS** (Lima VM).
It is intentionally grounded in the current implementation (see “Code Pointers”).

Scope:
- `world_fs.mode`: `writable | read_only`
- `world_fs.isolation`: `workspace | full`
- The “world backend” path (world-agent + overlay + mount namespace + optional Landlock)

## Comparison Matrix

| Topic | Linux (native backend) | macOS (Lima-backed backend) |
|---|---|---|
| Where enforcement runs | On the Linux host via the world backend (world-agent) and its Linux kernel primitives. | Inside the Linux guest VM (world-agent runs in the guest); the macOS host only provides transport/forwarding. |
| `world_fs.isolation=workspace` (what it tries to prevent) | Prevents absolute-path escapes back into the host project by placing the overlay at the project path inside a private mount namespace. | Same mechanics, but “host paths” mean *guest-visible mounts* (the project and any other host-shared directories that are mounted into the guest). |
| `workspace`: are non-project paths nameable? | Yes. The process can still name other host paths (it’s not a `pivot_root` cage). | Yes, for any paths that exist inside the guest (including shared host mounts, if present). |
| `workspace`: write protection outside project | Best-effort write blocking via Landlock “write-only allowlist”: allows writes to `/tmp`, `/var/tmp`, `/dev`, `/var/lib/substrate/world-deps`, plus the project dir (`SUBSTRATE_MOUNT_PROJECT_DIR`); other writes are denied by Landlock. | Same Landlock write-only allowlist behavior, but applied inside the guest kernel. Writes to other guest-visible mounts outside the project should be denied by Landlock. |
| `workspace`: read protection outside project | No read restrictions are applied via Landlock in workspace mode (reads remain unrestricted by Landlock). | Same: workspace mode does not apply Landlock read allowlists. |
| `world_fs.isolation=full` (what it tries to prevent) | Makes host paths “not nameable” by constructing a minimal rootfs and `pivot_root`ing into it. | Same `pivot_root`-based cage, but executed inside the guest (so it hides guest-visible host mounts as well). |
| `full`: which mounts exist | Script bind-mounts a minimal set (`/usr`, `/bin`, `/lib*`, `/etc` as read-only; `/dev` read-only; `/proc`; `/tmp` as tmpfs; `/var/lib/substrate/world-deps` read-write) plus the project mounted at both `/project` and the host-absolute project path. | Same mount set inside the guest. |
| `full` + `mode=writable`: project writability | The project is remounted read-only by default; only prefixes derived from `world_fs.write_allowlist` are remounted `rw` (via `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST`). If the allowlist is empty, project writes should fail. | Same semantics inside the guest. |
| `full` + `mode=read_only` | The project mount is remounted read-only; allowlisted prefixes are not remounted writable because `SUBSTRATE_MOUNT_FS_MODE=read_only`. | Same semantics inside the guest. |
| `full`: read_allowlist / write_allowlist usage | In addition to mount semantics, the world-agent can apply Landlock allowlists in full mode: `read_allowlist` feeds `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST`, `write_allowlist` feeds `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST`, and `write_allowlist` also drives the mount-time writable prefix remounting. | Same behavior inside the guest (subject to Landlock support in the guest kernel). |
| Diagnostics surface area | `substrate world doctor` probes host-kernel capabilities (overlayfs, nft, cgroup v2) and includes a Landlock support probe. | `substrate world doctor (macOS)` focuses on Lima + transport + guest service status; it does not currently report Landlock support (Landlock is a guest-kernel concern). |
| Verification harness caveat | `substrate world verify` uses the OS temp dir by default for its scratch projects/logs; this is usually fine on Linux hosts. | `substrate world verify` default root may land under macOS `/var/folders/...`. If that path is not mirrored into the guest, full-isolation verification can fail for environmental reasons; overriding `--root` to a host path that is visible in the guest (e.g. under `$HOME`) avoids this. |

## Code Pointers (Implementation Ground Truth)

Filesystem/mount enforcement (Linux shell script used by the world backend):
- `crates/world/src/exec.rs` (`PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT`)

Landlock policy application:
- `crates/world/src/landlock.rs` (`apply_filesystem_policy`, `apply_write_only_allowlist`)
- `crates/world-agent/src/internal_exec.rs` (`__substrate_world_landlock_exec` wrapper; applies full-mode allowlists, and applies the workspace-mode write-only allowlist when isolation is not full)

How allowlists are resolved and injected (full isolation):
- `crates/world-agent/src/service.rs` (non-PTY `/v1/execute`)
- `crates/world-agent/src/pty.rs` (PTY `/v1/stream`)

macOS transport / “world backend available” vs fallback:
- `crates/shell/src/execution/platform/macos.rs` (doctor checks Lima + guest service)
- `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`execute_world_pty_over_ws_macos`)

Verification harness:
- `crates/shell/src/builtins/world_verify.rs` (`substrate world verify` root defaults + `--root` override)

## Practical Takeaways

- The “macOS version” of `world_fs.isolation` is effectively the **Linux implementation executed inside the Lima guest**, so the key differences are about **what paths are mounted into the guest** and what the **macOS host doctor** can observe.
- The recent “workspace isolation is stronger on Linux (Landlock in both modes)” is implemented in the Landlock exec wrapper by applying a write-only allowlist when `SUBSTRATE_WORLD_FS_ISOLATION != full`.
