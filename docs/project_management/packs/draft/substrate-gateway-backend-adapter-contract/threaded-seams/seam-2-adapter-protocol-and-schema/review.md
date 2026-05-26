---
seam_id: SEAM-2
review_phase: pre_exec
execution_horizon: active
basis_ref: seam.md#basis
---

# Review Bundle - SEAM-2 Adapter protocol and schema

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can the protocol still redefine the published backend-id selection boundary instead of consuming `THR-01` as fixed upstream truth?
- Can unsupported capabilities or required extension keys still fall through to permissive behavior instead of failing closed?
- Can request/response payloads, adapter errors, or session-handle facets still widen without one concrete adopted Unified Agent API subset?
- Can local adapter translation still silently redefine ADR-0017 event-envelope or ADR-0028 trace ownership instead of handing off to those external owners explicitly?
- Can seam-local planning still treat `THR-02` as an inbound prerequisite instead of as outbound publication work owned by this seam?

## R1 - Consumed upstream handoff

```mermaid
flowchart LR
  S1["SEAM-1 closeout"] --> THR1["THR-01 published"]
  THR1 --> C1["C-01 stable backend-id contract"]
  THR1 --> C2["C-02 status boundary"]
  C1 --> Dispatch["SEAM-2 dispatch lifecycle"]
  C2 --> Dispatch
  Dispatch --> THR2["THR-02 publication target"]
```

## R2 - Protocol and schema ownership split

```mermaid
flowchart TB
  Protocol["Local adapter protocol"] --> Owner["Explicit owner line"]
  Owner --> ADR17["ADR-0017 event-envelope owner"]
  Owner --> ADR28["ADR-0028 trace owner"]
  Protocol --> Schema["Adopted Unified Agent API subset"]
  Schema --> Caps["Capabilities + extension keys"]
  Schema --> Payloads["Request/response payloads"]
  Schema --> Errors["Bounded adapter error detail"]
  Schema --> Session["Session-handle facets"]
```

## Likely mismatch hotspots

- `REM-002` stays open only as post-baseline execution and closeout tracking for the adopted Unified Agent API subset.
- `REM-003` stays open only as post-baseline execution and closeout tracking for the local-to-external owner line.
- Any seam-local draft that treats `THR-02` as required inbound state would falsely block activation and invert producer/consumer ownership.

## Pre-exec findings

- Upstream revalidation is satisfied: `../../governance/seam-1-closeout.md` records `seam_exit_gate.status: passed`, `promotion_readiness: ready`, and `THR-01` as published.
- The seam is safe to activate and decompose because the consumed upstream contract truth is now landed and current.
- Review is concrete enough to falsify the intended lifecycle, schema, and owner-line shape.
- The contract gate is now satisfied:
  - `C-03` is pinned by `docs/contracts/gateway/backend-adapter-protocol.md` and `../../gateway-backend-adapter-protocol-spec.md`.
  - `C-04` is pinned by `docs/contracts/gateway/backend-adapter-schema.md` and `../../gateway-backend-adapter-schema-spec.md`.
- `REM-002` and `REM-003` remain open only as non-blocking execution and closeout tracking; they no longer block `status: exec-ready`.

## Pre-exec gate disposition

- **Review gate**: passed
  - the seam-local review now makes the dispatch lifecycle, schema inventory, and owner-line risks falsifiable.
- **Contract gate**: passed
  - the adopted schema subset and owner line are now concrete in owner artifacts.
- **Revalidation gate**: passed
  - `SEAM-1` closeout is landed and still matches the seam brief basis.
- **Opened remediations**:
  - none; existing remediation entries remain authoritative.
- **Current readiness posture**:
  - `SEAM-2` is active and `status: exec-ready`.
  - `THR-02` stays `defined` until this seam lands and closes out its owned contracts.

## Verification evidence model

- `C-03` evidence should anchor to the protocol spec sections that define the deterministic lifecycle, local-to-external owner line, and runtime-adjacent adoption surfaces.
- `C-04` evidence should anchor to the schema spec sections that define the adopted capability ids, extension-key subset, bounded payload inventory, adapter error payload, and session-handle facet.
- `THR-02` evidence should anchor to the seam-2 closeout record fields for canonical artifact paths, landed delta, stale triggers, remediation disposition, and recorded publication state.
- `SEAM-3` should consume the recorded evidence model, not new contract prose, when it performs downstream parity and validation review.

## Planned seam-exit gate focus

- **What must be true before downstream promotion is legal**:
  - `C-03` has one deterministic dispatch lifecycle and one explicit ADR-0017 / ADR-0028 owner line.
  - `C-04` has one bounded adopted schema subset with fail-closed capability and extension-key behavior.
  - `THR-02` is published from closeout with schema, lifecycle, and stale-trigger evidence recorded.
- **Which outbound contracts/threads matter most**:
  - `C-03`
  - `C-04`
  - `THR-02`
- **Which review-surface deltas would force downstream revalidation**:
  - any change to capability ids or extension-key subset
  - any change to payload, error, or session-handle schema
  - any change to ADR-0017 or ADR-0028 owner-line wording
