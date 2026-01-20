# Decision Register — Full Isolation Landlock ↔ OverlayFS Compatibility

This decision register scopes decisions required to restore the `world_fs.write_allowlist` contract
in `world_fs.isolation=full` when Linux Landlock enforcement is enabled and the active filesystem
strategy uses overlayfs backing dirs (`upperdir` / `workdir`).

Related ADR:
- `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

---

### DR-0001 — Full Isolation: Landlock Runtime Allowlist for OverlayFS Backing Dirs

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

**Problem / Context**
- In full isolation on Linux, the command process runs under an applied Landlock policy.
- When using overlayfs, project writes that appear to target `/project/<prefix>` are serviced via
  overlayfs backing dirs (`upperdir` / `workdir`).
- If Landlock does not allow the overlay backing dirs, allowlisted project writes fail with `EPERM`
  even when mount-level allowlist remounting is correct.
- This runtime state must not be represented in the policy snapshot schema/hash (it is per-session).

**Option A — Derive overlay backing dirs at runtime via procfs mount inspection**
- **Pros:** No policy surface area expansion; works with policy snapshots; robust under version skew; keeps runtime-derived state out of snapshot hashes.
- **Cons:** Requires Linux-only parsing logic for `/proc/self/mountinfo` (preferred) or `/proc/self/mounts`; needs careful error messaging.
- **Cascading implications:** World-agent Landlock wrapper must read mount metadata and extend allowlists dynamically.
- **Risks:** Incorrect parsing could over-allow or under-allow; mitigate with fixture tests and fail-closed behavior.
- **Unlocks:** Generalizes to other filesystem strategies that have internal write roots (future).
- **Quick wins / low-hanging fruit:** Implement a helper in `crates/world` and reuse it from the Landlock exec wrapper.

**Option B — Plumb overlay backing dirs via explicit env vars from the world backend**
- **Pros:** Avoids procfs parsing; uses already-known overlay state paths from the backend.
- **Cons:** Requires new env var contracts and additional plumbing across execution paths (non-PTY + PTY); increases “ambient” runtime surface area.
- **Cascading implications:** Requires keeping env propagation consistent across platforms/backends; requires versioning if treated as a stable contract.
- **Risks:** Env propagation drift could silently break enforcement; needs strong invariants and tests.
- **Unlocks:** Enables backends to provide strategy-specific internal roots without procfs dependence.
- **Quick wins / low-hanging fruit:** Add env var(s) only as an internal implementation detail, not as a documented stable interface.

**Recommendation**
- **Selected:** Option A — Derive overlay backing dirs at runtime via procfs mount inspection
- **Rationale (crisp):** Overlay backing dirs are session-specific runtime state; procfs-based derivation keeps the policy snapshot deterministic while restoring allowlisted writability under Landlock.

**Follow-up tasks (explicit)**
- Implement Linux-only mount inspection helper (prefer `/proc/self/mountinfo`) in `crates/world`.
- Extend `crates/world-agent/src/internal_exec.rs` to include derived overlay backing dirs in the full isolation Landlock policy write allowlist.
- Add fixture-based unit tests for mount parsing and an integration test gated on Landlock support that asserts allowlisted writes succeed under full isolation.

