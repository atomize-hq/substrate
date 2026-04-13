---
slice_id: S3
seam_id: SEAM-2
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - fixture evidence drifts from the owned `C-11` contract or from consumed `C-12` invariants
    - upstream provider `/v1/responses` behavior shifts enough to require updating the Responses contract and fixtures together
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-12
contracts_produced:
  - C-11
contracts_consumed:
  - C-12
open_remediations: []
---
### S3 - Lock Responses Fixtures And Contract Publication

- **User/system value**: `/v1/responses` lands with durable, closeout-backed evidence (fixtures + tests) that `THR-12` can publish, and downstream conformance can consume without reverse-engineering the runtime adapter.
- **Scope (in/out)**:
  - In: add a minimal but durable fixture and regression suite for `/v1/responses` sync and stream behavior aligned to `C-11`, and prepare publication accounting for `THR-12`.
  - Out: full pack conformance ownership (belongs to `SEAM-3`) and expanding beyond the contracted Responses subset.
- **Acceptance criteria**:
  - fixtures exist for:
    - sync Response objects (text-only, tool-call-only, mixed)
    - streaming event subset coverage (text delta paths, tool-call delta paths, done events, and shared lifecycle/output-item events)
    - tool-loop continuation using `function_call_output`
  - fixtures and tests assert required fields, ordering, and rejection posture without being brittle to optional fields the contract allows to vary
  - the seam can close out with one explicit “published” or “blocked” decision for `THR-12` based on the landed evidence set
  - fixture coverage is explicitly sufficient to prove the ADR 0008-compatible event subset and `call_id` continuation rules without reading provider parsing code
- **Dependencies**: `S00`, `S1`, `S2`, `gateway/tests/*` (or equivalent), `THR-12`, `C-11`, `C-12`
- **Verification**:
  - tests fail deterministically on event ordering drift, missing required events, tool-loop threading drift, or built-in tool leakage
  - pass condition: the closeout can cite one compact fixture set as publication evidence for `THR-12`
- **Rollout/safety**: keep fixtures compatibility-scoped and deterministic; do not introduce live upstream network dependencies for this publication baseline.
- **Review surface refs**: `../../review_surfaces.md` (`R3`) and `review.md` (`Planned seam-exit gate focus`)

#### S3.T1 - Add Sync Response Object Fixtures

- **Outcome**: sync Response objects are locked into fixture-backed truth that matches `C-11`.
- **Inputs/outputs**: inputs are the sync adapter behavior from `S1`; outputs are fixtures and tests that assert required Response object fields and output item mapping.
- **Thread/contract refs**: `THR-12`, `C-11`
- **Implementation notes**: focus assertions on required fields and ordering; treat optional fields per the contract’s tolerant posture.
- **Acceptance criteria**: fixtures cover text-only, tool-call-only, mixed output, and usage present/absent where relevant.
- **Test notes**: keep tests local and deterministic; no network calls.
- **Risk/rollback notes**: weak or missing fixtures will block `THR-12` publication and force downstream seams to infer truth from code.

Checklist:
- Implement: add sync fixtures and tests
- Test: ensure stable ordering and required fields
- Validate: confirm tool arguments remain JSON strings
- Cleanup: keep fixture names and locations easy for conformance to reuse later

#### S3.T2 - Add Streaming Event-Subset Fixtures

- **Outcome**: the streaming event subset and per-shape ordering rules are locked into fixture-backed truth that matches `C-11`.
- **Inputs/outputs**: inputs are the event adapter behavior from `S2`; outputs are streaming fixtures and tests that assert required event presence, ordering, and payload minimum fields.
- **Thread/contract refs**: `THR-12`, `C-11`
- **Implementation notes**: assert `data.type` naming and required payload shapes; tolerate optional fields per contract rules.
- **Acceptance criteria**: fixtures cover text deltas, tool-call argument deltas/done, completion semantics, and `[DONE]`-style termination equivalents if contracted.
- **Test notes**: add at least one mixed text+tool streaming fixture.
- **Risk/rollback notes**: event streaming is the highest compat risk; missing event-subset evidence will block publication.

Checklist:
- Implement: add streaming fixtures and tests
- Test: assert required event presence and ordering for each stream shape
- Validate: confirm chain-of-thought remains suppressed in event payloads
- Cleanup: keep fixtures small and deterministic
