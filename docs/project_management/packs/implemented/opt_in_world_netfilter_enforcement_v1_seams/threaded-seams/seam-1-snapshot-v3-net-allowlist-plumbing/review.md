---
seam_id: SEAM-1
review_phase: pre_exec
execution_horizon: future
basis_ref: seam.md#basis
---
# Review Bundle - SEAM-1 Snapshot V3 `net_allowed` contract + host→world-service plumbing

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can any world-service execution path (PTY or non-PTY) still consult `substrate_broker::allowed_domains()` (or equivalent broker-derived state) instead of Snapshot V3 `net_allowed`?
- Can the host construct Snapshot V3 `net_allowed`, but still send a `WorldSpec` that requests allow-all or deny-all in ways that diverge from the snapshot’s canonicalized semantics?
- Can a missing/unknown `net_allowed` field (older snapshot payload) silently change behavior instead of defaulting to a safe, explicit allow-all/deny-all posture?

## R1 - UX / workflow flow

```mermaid
flowchart LR
  U["Operator"] -->|"sets opt-in"| CFG["C-04 world.net.filter (host config)"]
  U -->|"runs command"| SH["substrate shell"]
  SH -->|"policy eval"| BR["broker (policy)"]
  BR -->|"effective net_allowed"| SNAP["C-01 PolicySnapshotV3.net_allowed (canonicalized)"]
  CFG -->|"request isolate_network?"| SPEC["C-02/C-03 WorldSpec (isolate_network + allowed_domains)"]
  SNAP --> REQ["execute request"]
  SPEC --> REQ
  REQ --> WA["world-service (service/pty)"]
  WA -->|"enforce-or-fail request"| WB["world backend (cgroup/netns)"]
```

## R2 - API / service / data flow

```mermaid
flowchart TB
  subgraph Host["Host / Control plane"]
    BR["broker"]
    SS["shell snapshot builder"]
    AT["transport-api-types (Snapshot V3 schema)"]
  end

  subgraph World["World / Data plane"]
    WA["world-service (service + pty)"]
    W["world backend"]
  end

  BR -->|"effective policy"| SS
  AT -->|"C-01 schema + helpers"| SS
  SS -->|"Snapshot V3 + WorldSpec"| WA
  WA -->|"WorldSpec.isolate_network + allowed_domains"| W
```

## Likely mismatch hotspots

- **Upstream contract drift**: `S2` consumes the now-published `C-04` / `THR-03` host gate; any later change to
  `world.net.filter`, override applicability, exported parity, or the three-way gate docs must force revalidation here.
- **Normalization drift**: hostname casefolding/IDNA posture implemented inconsistently across `transport-api-types`, the host snapshot builder, and the world-service request path (decision is recorded in `S1.T1`; all consumers must share one helper).
- **PTY vs non-PTY divergence**: `service.rs` and `pty.rs` construct different `WorldSpec`/allowlist values or source them from different places.
- **Wildcard semantics**: `["*"]` is not canonicalized to exactly `["*"]`, or other wildcard forms slip through when isolation is requested.
- **Back-compat defaults**: missing `net_allowed` in older snapshots fails to `serde(default)` as intended or accidentally flips behavior.

## Pre-exec findings

- Hostname normalization posture (casefolding + IDNA) for `net_allowed` is now explicitly specified in `S1.T1`.
- `SEAM-3` now publishes `C-04` / `THR-03` in the landed config/env surfaces and operator docs, so this seam no longer
  depends on an unpublished upstream gate.
- This seam’s basis is refreshed against the landed host-gate rule: the host requests `isolate_network` only when the
  effective `world.net.filter` gate is enabled and canonicalized `net_allowed` is restrictive; `WORLD_NETFILTER_ENABLE`
  remains downstream runtime gating only.

## Pre-exec gate disposition

- **Review gate**: passed
- **Contract gate**: passed
- **Contract gate concerns**:
  - none; `C-04` / `THR-03` are now published upstream and can be consumed directly by this seam.
- **Revalidation**: passed
- **Revalidation concerns**:
  - none; this seam has been refreshed against the published host-gate docs and config/env evidence.
- **Opened remediations**:
  - none

## Planned seam-exit gate focus

- **What must be true before downstream promotion is legal**:
  - `C-01`..`C-03` are published with tests and documented semantics.
  - `C-04` / `THR-03` remain stable upstream; any change to that host gate forces seam-local revalidation before
    promotion.
  - `THR-01` and `THR-02` are advanced from `identified` to `published` in closeout.
  - world-service no longer consults broker-derived allowlists for routing/enforcement.
- **Which outbound contracts/threads matter most**: `C-01`, `C-02`, `C-03` and `THR-01`, `THR-02`.
- **Which review-surface deltas would force downstream revalidation**:
  - Any change to canonicalization or wildcard rejection rules
  - Any change to `WorldSpec` field meaning under opt-in gating
  - Any change to PTY/non-PTY execute request shape
