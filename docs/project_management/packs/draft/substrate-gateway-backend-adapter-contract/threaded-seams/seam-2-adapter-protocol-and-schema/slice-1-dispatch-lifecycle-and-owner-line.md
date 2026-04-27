---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - backend-id dispatch lifecycle changes
    - ADR-0017 owner wording changes
    - ADR-0028 owner wording changes
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations:
  - REM-003
---

### S1 - Lock dispatch lifecycle and external owner line

#### Goal

Turn the protocol boundary into one deterministic lifecycle that consumes the published selection result and hands off event and trace semantics cleanly to their external owners.

#### Dependencies

- `S00` defines the owned contract bundle
- basis authorities:
  - `../../governance/seam-1-closeout.md`
  - `../../pre-planning/alignment_report.md`
  - `../../pre-planning/workstream_triage.md`

#### S1.T1 - Write the ordered dispatch lifecycle

- **Outcome**:
  - the seam-owned protocol docs state the exact order in which a selected backend id becomes adapter lookup, capability validation, normalized request handling, and normalized response emission.
- **Files**:
  - `../../gateway-backend-adapter-protocol-spec.md`
  - `../../pre-planning/workstream_triage.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - the lifecycle starts from the published `SEAM-1` backend-id result
  - capability validation occurs before runtime execution
  - fail-closed behavior is explicit for unsupported capability or extension requirements
  - the flow does not widen policy inputs or status ownership
- **Test notes**:
  - cross-check with the published `SEAM-1` closeout and existing gateway review surfaces

Checklist:

- Implement:
  - document the ordered lifecycle and fail-closed checkpoints in `../../gateway-backend-adapter-protocol-spec.md`
  - keep `docs/contracts/substrate-gateway-backend-adapter-protocol.md` as the durable owner of the lifecycle boundary
  - document which upstream selection outputs are consumed as fixed inputs
- Test:
  - compare the order against the seam review diagrams and pre-planning alignment notes
- Validate:
  - confirm the lifecycle can be cited without reopening selection semantics

#### S1.T2 - Pin the ADR-0017 / ADR-0028 owner line

- **Outcome**:
  - the seam-owned protocol docs state exactly where local adapter translation stops and externally owned event-envelope and trace semantics begin.
- **Files**:
  - `../../gateway-backend-adapter-protocol-spec.md`
  - `../../pre-planning/alignment_report.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - local translation responsibilities are explicit
  - ADR-0017 and ADR-0028 remain the external owners of envelope and canonical trace vocabulary semantics
  - the protocol does not silently redefine those external contracts
- **Test notes**:
  - compare wording against ADR-0017, ADR-0028, and `review.md`

Checklist:

- Implement:
  - write the exact local-to-external owner boundary
  - map each owned protocol surface to the correct upstream authority
  - record that repository packaging changes for the standalone gateway or UAA repos do not change this owner line by themselves
- Test:
  - verify each lifecycle stage has one owner against:
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/backend_contract.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/backend_contract.rs`
- Validate:
  - confirm `REM-003` can resolve without widening other seams
