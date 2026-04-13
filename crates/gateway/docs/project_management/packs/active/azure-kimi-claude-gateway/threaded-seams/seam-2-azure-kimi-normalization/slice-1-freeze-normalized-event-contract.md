---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Azure evidence shows hidden markers or empty-content behavior that the frozen `C-02` rules do not classify"
    - the contract still leaves downstream seams guessing whether parser provenance is part of the event surface or debug-only evidence
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Normalized Event Contract

- **User/system value**: downstream seams stop planning against raw Azure response shapes and inherit one concrete internal contract for tool, action, and final events.
- **Scope (in/out)**:
  - In: define the `C-02` event vocabulary, required fields, provenance rules, malformed-input behavior, reuse-versus-bypass statement, and verification checklist for explicit `tool_calls`, hidden `reasoning_content`, mixed signals, and no-tool responses.
  - Out: public Anthropic payload mapping, planner/executor routing policy, rollout decisions, Substrate-facing event publication, fixture capture, parser implementation, and closeout/accounting work.
- **Acceptance criteria**:
  - `docs/foundation/azure-kimi-c02-normalized-event-contract.md` is the canonical landed source for `C-02`.
  - the contract names one canonical normalized event vocabulary that covers tool intent, intermediate action/state updates, and final assistant completion without exposing raw Azure sentinel syntax.
  - the contract specifies which provenance fields are part of the internal contract versus retained only for debugging or evidence capture.
  - the contract states how explicit-plus-hidden collisions, malformed markers, and empty-content fallback are resolved.
  - the contract includes a narrow verification checklist with pass/fail conditions for explicit-only, hidden-only, mixed, and no-tool cases.
  - the contract records the reuse-versus-bypass rule relative to `C-01` so later implementation work does not blur Azure-specific logic into generic provider behavior.
- **Dependencies**: `../../threading.md` (`C-01`, `C-02`, `THR-01`, `THR-02`), `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, `docs/foundation/claude-code-mux-extension-boundary.md`, `docs/foundation/claude-code-mux-5a372fb-validation.md`, ADR 0002, and ADR 0003
- **Verification**:
  - a reviewer can explain what event kinds exist, what each kind guarantees, and which fields downstream seams may rely on by reading `docs/foundation/azure-kimi-c02-normalized-event-contract.md`
  - edge cases are explicit: explicit-plus-hidden collisions, hidden markers without tool resolution, malformed sentinel blocks, and empty-content fallback
  - pass condition: `SEAM-3`, `SEAM-4`, and `SEAM-5` can cite `C-02` directly and do not need raw Azure payload examples to define their own contracts
- **Rollout/safety**: keep the contract internal and capability-oriented; do not let it leak public backend identity or routing policy choices.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R1`, `Likely mismatch hotspots`)

#### S1.T1 - Define The Event Vocabulary And Invariants

- **Outcome**: one normalized event model exists for Azure Kimi responses and is landed in a citeable foundation note.
- **Inputs/outputs**: inputs are `C-01`, ADR 0002, ADR 0003, and the `5a372fb` validation note; output is `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, which names event kinds, required fields, ordering guarantees, and invariants.
- **Thread/contract refs**: `THR-02`, `C-02`, `C-01`
- **Implementation notes**: the contract should stay downstream-friendly and must not require consumers to know whether the source path was explicit `tool_calls` or hidden `reasoning_content`; do not pull fixture evidence or parser implementation details into this slice.

#### S1.T2 - Freeze Provenance And Failure Semantics

- **Outcome**: the seam defines how parser provenance, malformed input, and debug evidence are handled without turning raw provider payloads into the consumer contract.
- **Inputs/outputs**: inputs are `C-01`, the review bundle, and the frozen contract note; output is a written rule set for provenance metadata, malformed-marker handling, and empty-content fallback behavior in the contract artifact.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: provenance can exist for debugging and closeout evidence, but raw sentinel strings and Azure chunk layout must remain debug-only and out of the consumer-facing contract.

#### S1.T3 - Freeze The Verification Checklist

- **Outcome**: the producing seam has the narrow contract checklist needed to pass later pre-exec contract review.
- **Inputs/outputs**: inputs are the frozen event vocabulary and known Azure risks; output is a pass/fail checklist in the foundation contract for explicit-only, hidden-only, mixed, and no-tool cases plus the expected normalized output assertions.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: keep the checklist focused on contract readiness, not post-exec publication, fixtures, or closeout; publication evidence belongs to seam exit later.
