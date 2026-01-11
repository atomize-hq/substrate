# linux_guest_rootfs_backend — Decision Register (Stub)

This decision register is required by `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`.

Status: stub (intentionally incomplete).

Planned decision entries (to be written before execution begins):
- DR-0001 — Linux guest rootfs image format and builder (A: debootstrap; B: OCI rootfs unpack)
- DR-0002 — Rootfs persistence model for system packages (A: persistent overlay upper; B: rebuild-on-demand)
- DR-0003 — Storage locations and ownership (A: `/var/lib/substrate/world-rootfs`; B: `$SUBSTRATE_HOME`-scoped)
- DR-0004 — Warm/provision command surface (A: CLI `substrate world rootfs warm`; B: script-only `scripts/linux/world-rootfs-warm.sh`)
- DR-0005 — How `world deps provision` detects readiness (A: doctor capability; B: on-demand probing)

