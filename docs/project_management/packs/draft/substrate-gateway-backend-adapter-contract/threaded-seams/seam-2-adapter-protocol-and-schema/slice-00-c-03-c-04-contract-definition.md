---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - adopted capability ids or extension keys change
    - request, response, error, or session-handle fields change
    - ADR-0017 or ADR-0028 owner wording changes
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
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations:
  - REM-002
  - REM-003
---

### S00 - Define the `C-03` and `C-04` contract baseline

#### Goal

Make the owned protocol/schema bundle concrete enough that downstream seams can cite one durable lifecycle and one bounded schema truth instead of inferring them from ADR prose or pre-planning notes.

#### Dependencies

- inbound publication is already satisfied by:
  - `../../governance/seam-1-closeout.md`
- authoritative basis references:
  - `../../threading.md`
  - `../../seam-2-adapter-protocol-and-schema.md`
  - `../../pre-planning/workstream_triage.md`
  - `../../pre-planning/alignment_report.md`

#### S00.T1 - Create the canonical `C-03` protocol baseline

- **Outcome**:
  - the seam-owned protocol surfaces state the deterministic adapter-dispatch lifecycle, fail-closed order, and the exact handoff boundary to ADR-0017 and ADR-0028.
- **Files**:
  - `docs/contracts/gateway/backend-adapter-protocol.md`
  - `../../gateway-backend-adapter-protocol-spec.md`
  - `../../pre-planning/spec_manifest.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - the lifecycle consumes the stable backend-id selection result instead of redefining it
  - unsupported capabilities and required extension keys fail closed
  - the local adapter translation boundary versus ADR-0017 and ADR-0028 is explicit
  - the protocol does not invent a second status or operator owner line
- **Test notes**:
  - doc review against ADR-0017, ADR-0028, `../../governance/seam-1-closeout.md`, and `review.md`

Checklist:

- Implement:
  - author the deterministic dispatch-lifecycle baseline in `docs/contracts/gateway/backend-adapter-protocol.md`
  - mirror the execution baseline and owner checklist in `../../gateway-backend-adapter-protocol-spec.md`
  - pin the exact owner-line handoff to ADR-0017 and ADR-0028 without widening those external owners
- Test:
  - compare the lifecycle against the published `SEAM-1` handoff and current ADR basis
  - keep the backend-harness owner-line regression coverage aligned in:
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/backend_contract.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/backend_contract.rs`
- Validate:
  - confirm downstream parity work can consume `C-03` without reopening selection-boundary ownership

#### S00.T2 - Create the canonical `C-04` schema subset baseline

- **Outcome**:
  - the seam-owned schema surfaces state the exact adopted Unified Agent API subset for capability advertisement, extension keys, request/response payloads, bounded adapter errors, and session-handle facets.
- **Files**:
  - `docs/contracts/gateway/backend-adapter-schema.md`
  - `../../gateway-backend-adapter-schema-spec.md`
  - `../../pre-planning/spec_manifest.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Acceptance criteria**:
  - capability ids and extension keys are explicit
  - request/response omission rules and bounded error detail are explicit
  - session-handle facets stay gateway-contract data rather than policy input
  - no schema family widens without a concrete owner line
- **Test notes**:
  - owner-line review against `REM-002` and the current pre-planning workstream packet

Checklist:

- Implement:
  - record the adopted schema subset and omission rules in `docs/contracts/gateway/backend-adapter-schema.md`
  - mirror the exact adopted capability ids, extension keys, bounded error shape, and session-handle facet in `../../gateway-backend-adapter-schema-spec.md`
- Test:
  - compare the proposed subset against the Unified Agent API evidence set and current gateway docs
  - keep the capability and session-handle regression coverage aligned in:
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/capabilities.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/capabilities.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/session_handle.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/session_handle.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/session_selectors.rs`
- Validate:
  - confirm `SEAM-3` can consume the schema without inventing new protocol ambiguity
