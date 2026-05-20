---
slice_id: S2
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - adopted capability ids change
    - extension-key subset changes
    - request, response, error, or session-handle fields change
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
  - C-04
contracts_consumed:
  - C-01
open_remediations:
  - REM-002
---

### S2 - Lock the adopted schema subset and fail-closed capability rules

#### Goal

Define one bounded schema inventory for adapter-visible protocol data so later implementation and parity work can validate against an explicit subset instead of an inferred API surface.

#### Dependencies

- `S00` defines the owned contract bundle
- basis authorities:
  - `../../governance/seam-1-closeout.md`
  - `../../pre-planning/workstream_triage.md`
  - Unified Agent API evidence referenced by ADR-0041

#### S2.T1 - Pin the adopted capability and extension-key subset

- **Outcome**:
  - the seam-owned schema docs state the supported capability ids, required extension keys, and fail-closed behavior for unsupported requests.
- **Files**:
  - `../../gateway-backend-adapter-schema-spec.md`
  - `../../pre-planning/spec_manifest.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Acceptance criteria**:
  - capability ids and extension keys are explicit
  - unsupported capabilities fail closed
  - no provider-specific identity leaks back into the stable backend-id surface
- **Test notes**:
  - compare the subset against the Unified Agent API evidence set and current gateway assumptions

Checklist:

- Implement:
  - record the supported capability and extension-key inventory in `../../gateway-backend-adapter-schema-spec.md`
  - keep `docs/contracts/substrate-gateway-backend-adapter-schema.md` as the durable owner of the adopted subset
  - record the fail-closed rules for unsupported requests
- Test:
  - verify every supported item is named explicitly against:
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/capabilities.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/capabilities.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/session_selectors.rs`
- Validate:
  - confirm downstream consumers can distinguish supported versus rejected schema features

#### S2.T2 - Pin payload, error, and session-handle schema

- **Outcome**:
  - the seam-owned schema docs state the request/response payload fields, bounded adapter error detail, and backend-defined session-handle facets with explicit omission rules.
- **Files**:
  - `../../gateway-backend-adapter-schema-spec.md`
  - `../../pre-planning/workstream_triage.md`
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Acceptance criteria**:
  - omission defaults and additive rules are explicit
  - bounded error detail is concrete
  - session-handle facets remain gateway-contract data rather than policy input
- **Test notes**:
  - compare wording against the seam review bundle and the current gateway boundary docs

Checklist:

- Implement:
  - write the concrete payload, error, and session-handle subset
  - record omission rules and additive constraints
  - keep bounded adapter error detail limited to the adopted safe UAA error classes and safe reserved messages
- Test:
  - verify every field family has one owner and one bounded purpose against:
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/session_handle.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/session_handle.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c1_codex_exec_policy.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c5_claude_add_dirs_runtime_rejection.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c3_explicit_cancellation.rs`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c3_explicit_cancellation_claude_code.rs`
- Validate:
  - confirm `REM-002` can resolve without leaking policy or operator state into the schema
