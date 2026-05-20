---
seam_id: SEAM-2
seam_slug: adapter-protocol-and-schema
type: integration
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - backend-id selection semantics or the published status subset change upstream
    - adopted capability ids or extension keys change
    - request, response, error, or session-handle fields change
    - ADR-0017 event-envelope owner wording changes
    - ADR-0028 trace owner wording changes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations:
  - REM-002
  - REM-003
---

# SEAM-2 - Adapter protocol and schema

- **Goal / value**:
  - Define one deterministic adapter lifecycle and one bounded schema inventory so gateway adapters can execute behind the stable selection boundary without widening ADR-0017 or ADR-0028 ownership.
- **Scope**
  - In:
    - `gateway-backend-adapter-protocol-spec.md`
    - `gateway-backend-adapter-schema-spec.md`
    - adapter registry lookup and dispatch order
    - capability-validation order and fail-closed extension-key handling
    - request normalization and response emission ordering
    - adapter error object shape
    - backend-defined session-handle facet schema
    - the exact owner line between local adapter translation and ADR-0017 / ADR-0028
  - Out:
    - stable backend-id selection rules already owned by `SEAM-1`
    - cross-platform guarantee proof
    - ADR-0024 supersession proof and ADR-0040 alignment decisions
    - operator command-family semantics
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
    - Unified Agent API evidence referenced by ADR-0041
  - Outputs:
    - `C-03`
    - `C-04`
- **Key invariants / rules**:
  - unsupported capabilities and required extension keys fail closed
  - session handles remain gateway-contract data rather than Substrate policy or operator identity data
  - local adapter translation may normalize events and traces, but ADR-0017 and ADR-0028 remain the external owners of envelope and canonical vocabulary semantics
  - the protocol must not reintroduce provider-specific sub-identities into the stable backend-id surface
  - future `cli:*` and `api:*` adapters extend this contract additively under the same stable identity model
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` closeout must remain the current published source for the stable selection contract and the adapter-visible status publication boundary
  - Transitive blockers:
    - ADR-0017 or ADR-0028 wording drift could collapse the intended local-to-external handoff line
    - unresolved Unified Agent API subset choices could force a schema split or resequencing
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - `substrate-gateway` runtime implementation
    - shared adapter clients
    - future manual validation and parity proof
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md`
  - likely downstream code surfaces once execution begins:
    - `crates/agent-api-types/src/lib.rs`
    - `crates/agent-api-client/src/lib.rs`
    - `crates/world-agent/src/handlers.rs`
    - `crates/shell/src/builtins/world_gateway.rs`
- **Verification**:
  - This seam consumes upstream contracts `C-01` and `C-02`, so verification may depend on accepted upstream evidence for the stable selection and publication boundary.
  - This seam produces owned contracts `C-03` and `C-04`.
  - At seam-brief depth, readiness is that the dispatch lifecycle, field inventory, fail-closed capability rules, and ADR-0017 / ADR-0028 handoff lines are concrete enough for seam-local planning and implementation.
  - `SEAM-2` is closed because `THR-02` is now published and the seam-local contract baseline landed in:
    - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
    - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
    - `gateway-backend-adapter-protocol-spec.md`
    - `gateway-backend-adapter-schema-spec.md`
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
- **Risks / unknowns**:
  - Risk:
    - landing could drift away from the now-pinned adopted capability subset, extension-key subset, bounded error shape, or session-handle bounds
  - De-risk plan:
    - keep `REM-002` open as non-blocking landing and closeout tracking tied to the canonical contract docs and schema spec
  - Risk:
    - landing could blur the local-to-external owner line and silently reopen ADR-0017 or ADR-0028 semantics
  - De-risk plan:
    - keep `REM-003` open as non-blocking landing and closeout tracking tied to the canonical protocol doc and seam-local protocol spec
- **Rollout / safety**:
  - this seam should only activate after the upstream selection contract is fixed
  - safety depends on failing closed for unsupported capabilities and keeping gateway-private mechanics out of Substrate policy surfaces
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `future` because the seam is already closed and no longer sits in the forward planning window after publishing `THR-02`
  - Which threads matter most
    - `THR-01`
    - `THR-02` as outbound work to publish after this seam lands
  - What the first seam-local review should focus on
    - dispatch lifecycle
    - fail-closed capability order
    - adopted Unified Agent API subset
    - session-handle boundary
    - exact ADR-0017 / ADR-0028 handoff wording
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-03`
    - `C-04`
  - Threads likely to advance:
    - `THR-02`
  - Review-surface areas likely to shift after landing:
    - adapter dispatch flow
    - normalized response/event boundary
    - schema inventory for adapter-visible payloads
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
