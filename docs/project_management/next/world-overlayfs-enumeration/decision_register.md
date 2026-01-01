# Decision Register — world-overlayfs-enumeration (ADR-0004)

This register contains the architectural decisions required by:
- `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`

Each entry presents exactly two viable options (A/B), selects one option, and maps follow-ups to triad task IDs in:
- `docs/project_management/next/world-overlayfs-enumeration/tasks.json`

### DR-0001 — Primary topology change to eliminate enumeration failures

**Decision owner(s):** spenser  
**Date:** 2025-12-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`, `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`

**Problem / Context**
- The current Linux “project cage” enforcement overlays the project directory by bind-mounting the overlay merged directory onto the host project path inside a private mount namespace.
- On some hosts, the overlay mount can succeed but directory enumeration in the merged view is broken, violating the world filesystem contract.

**Option A — Keep bind-mount topology; rely on health check + fallback**
- **Pros:** Minimal change to mount choreography; lower short-term implementation risk.
- **Cons:** Leaves the primary topology in place even if it is the trigger for enumeration failure; increases reliance on the fallback strategy and increases host variance.
- **Cascading implications:** Higher long-term maintenance cost; the “normal path” may become “fallback path” on affected hosts.
- **Risks:** More flaky behavior across kernels/filesystems; more operator confusion when strategy changes frequently.
- **Unlocks:** Fast delivery of “fail-closed” safety posture even if the root cause is not fixed.
- **Quick wins / low-hanging fruit:** Add probe + refuse world when unhealthy without changing mounts.

**Option B — Replace bind-mount enforcement with mount move**
- **Pros:** Removes the bind-mount topology from the project path enforcement step; reduces the number of mountpoints referencing the overlay merged tree; aligns the enforcement step with “single mountpoint” semantics.
- **Cons:** Touches low-level mount choreography in a correctness-critical path; requires careful cleanup and error handling.
- **Cascading implications:** Provides a stronger baseline for future PTY parity and “full cage” work by clarifying the intended mount topology.
- **Risks:** Incorrect move semantics can break cwd resolution or teardown; incorrect error handling can reintroduce host escape risks.
- **Unlocks:** A more predictable world filesystem contract on Linux without routinely depending on the fallback strategy.
- **Quick wins / low-hanging fruit:** Eliminate “double mountpoint” complexity while keeping health check as a safety net.

**Recommendation**
- **Selected:** Option B — Replace bind-mount enforcement with mount move
- **Rationale (crisp):** If enumeration is broken due to the current topology, only changing probing/fallback is insufficient; moving the mount makes the topology simpler and reduces kernel/filesystem edge-case exposure while keeping the probe as a hard safety gate.

**Follow-up tasks (explicit)**
- `WO0-code`: implement the mount choreography update (move instead of bind) and ensure it is applied before command execution.
- `WO0-test`: add regression coverage for enumeration health and strategy/fallback observability.
- `WO0-integ`: run smoke + manual playbook to validate enumeration and observability end-to-end.

### DR-0002 — Fallback behavior when primary strategy is unhealthy

**Decision owner(s):** spenser  
**Date:** 2025-12-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`, `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`

**Problem / Context**
- Substrate needs deterministic behavior when primary kernel overlayfs is unavailable or enumeration-unhealthy.
- When world execution is required, Substrate must fail closed; when world is optional, Substrate must degrade to host with an explicit warning and trace metadata.

**Option A — Fail closed whenever primary overlay is unavailable or unhealthy**
- **Pros:** Simplest strategy model; no additional runtime dependencies.
- **Cons:** Blocks world execution on hosts where a safe in-world fallback exists; reduces portability of world-required flows.
- **Cascading implications:** More hosts require policy/config changes or provisioning to use Substrate’s world workflows.
- **Risks:** Operator workflows become brittle on heterogeneous fleets; encourages disabling world rather than fixing it.
- **Unlocks:** Clearer safety posture with fewer moving parts.
- **Quick wins / low-hanging fruit:** Implement probe + “exit 3” without adding fuse strategy selection.

**Option B — Retry with fuse-overlayfs; degrade to host only when world is optional**
- **Pros:** Better portability; isolates host variance behind deterministic strategy selection; preserves fail-closed posture for world-required flows.
- **Cons:** Adds dependency surface (fuse + fuse-overlayfs binary); requires additional doctor/trace observability and careful error classification.
- **Cascading implications:** Requires consistent instrumentation to make strategy selection reproducible and debuggable.
- **Risks:** Performance and semantic differences between kernel overlayfs and fuse-overlayfs can surprise operators if not visible.
- **Unlocks:** Stronger “world works everywhere Substrate claims to support” posture while remaining fail-closed when required.
- **Quick wins / low-hanging fruit:** Use existing fuse-overlayfs support and add deterministic retry logic driven by the enumeration probe.

**Recommendation**
- **Selected:** Option B — Retry with fuse-overlayfs; degrade to host only when world is optional
- **Rationale (crisp):** This preserves safety when world is required, improves portability when world is optional, and makes failures actionable via explicit fallback reasons and doctor/trace metadata.

**Follow-up tasks (explicit)**
- `WO0-code`: implement deterministic strategy selection and retry chain (`overlay` then `fuse`), plus host fallback warning for world-optional no-viable-strategy.
- `WO0-test`: add tests that force primary failure and assert `fuse` selection and trace/doctor metadata.
- `WO0-integ`: run Linux smoke and verify the trace/doctor contracts in the manual playbook.

