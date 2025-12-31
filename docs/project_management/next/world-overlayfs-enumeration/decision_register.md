# Decision Register — World OverlayFS Enumeration Reliability (ADR-0004)

This register contains the non-trivial architectural decisions required by ADR-0004. Each entry is exactly A/B with tradeoffs and one selection.

## DR-0001 — Primary fix for overlay enumeration failures

### Option A (keep current topology; add only fallback probing)
- Keep mounting overlayfs in the world backend and bind-mount the merged overlay root onto the project path in a per-command private mount namespace.
- Detect “enumeration unhealthy” and fall back to fuse-overlayfs/copydiff.

Tradeoffs:
- Lower implementation risk (incremental).
- Higher long-term complexity and variance (multiple strategies used routinely depending on host quirks).
- Leaves the primary architecture vulnerable to additional kernel/filesystem edge cases.

### Option B (change topology; make overlay enumeration intrinsically correct)
- Restructure Linux world project mounting so the overlay mount topology is stable under bind mounts and mount namespaces.
- Ensure the “lower snapshot” used for overlay lowerdir is not invalidated by overmounting the project path.
- Keep fallback probing as a last-resort safety net, not a normal-path dependency.

Tradeoffs:
- Higher implementation effort (touches mount choreography and strategy plumbing).
- Stronger, more predictable contract: world behaves like a real filesystem everywhere Substrate claims to support.

Selection: **Option B**

## DR-0002 — Fallback behavior on overlay strategy health check failure

### Option A (fail closed on any overlay strategy failure)
- If overlayfs is unhealthy/unavailable, treat world execution as unavailable and apply fail-closed behavior whenever world is selected.

Tradeoffs:
- Simpler mental model; fewer backends.
- Worse portability and UX on hosts with partial overlay support; blocks work even when a safe in-world fallback exists.

### Option B (retry with an in-world fallback; fail closed only when required)
- If kernel overlayfs is unhealthy/unavailable, retry with fuse-overlayfs.
- If no viable in-world strategy exists:
  - Fail closed only when world is required.
  - Otherwise, fall back to host with explicit warning + trace annotation.

Tradeoffs:
- More implementation complexity.
- Better portability and resilience while keeping a strict posture for “world required” flows.

Selection: **Option B**

