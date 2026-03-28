---
slice_id: S1
seam_id: SEAM-4
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to doctor endpoint schema or field naming"
    - "Any change to requested-state derivation from Snapshot/WorldSpec inputs"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
contracts_produced:
  - C-07
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S1 - Publish the additive doctor netfilter status contract

- **User/system value**: operators get one stable JSON block that explains whether filtering was requested, whether it is enabled, whether the service env guard is present, and what failed last.
- **Scope (in/out)**:
  - In:
    - extend the world doctor schema with a dedicated netfilter status block
    - define the contract for `requested`, `enabled`, `world_netfilter_enable_present`, and `last_failure_reason`
    - ensure the contract remains additive for existing doctor consumers
  - Out:
    - terminal smoke playbook coverage (`SEAM-5`)
    - new runtime enforcement semantics (`SEAM-2`)
- **Acceptance criteria**:
  - `crates/agent-api-types` publishes additive doctor fields for the netfilter status block.
  - `crates/world-agent` returns one coherent block with the four required signals populated or intentionally null.
  - Existing doctor schema/version compatibility remains intact for callers that ignore the new block.
- **Dependencies**:
  - published routing handoff in `../../governance/seam-1-closeout.md`
  - published host gate semantics in `../../governance/seam-3-closeout.md`
  - published runtime guard/failure taxonomy in `../../governance/seam-2-closeout.md`
- **Verification**:
  - schema and handler tests for additive JSON fields
  - fixtures asserting nullable-but-present failure-reason behavior where appropriate
- **Rollout/safety**:
  - additive JSON only; no existing field removals or semantic repurposing
- **Review surface refs**:
  - `review.md`

#### S1.T1 - Extend the doctor schema with a netfilter status block

- **Outcome**: the world doctor contract has one explicit place for netfilter observability.
- **Files**:
  - `crates/agent-api-types/src/lib.rs`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `C-07`
- **Acceptance criteria**:
  - the schema names requested/enabled/guard/failure fields explicitly
  - the fields remain additive relative to current doctor JSON consumers
- **Test notes**:
  - add or refresh doctor-schema assertions in shell/world-agent test fixtures

Checklist:
- Implement: additive doctor schema fields
- Test: schema/fixture assertions
- Validate: existing doctor consumers can ignore the new block safely

#### S1.T2 - Populate requested and guard state from published upstream truth

- **Outcome**: the doctor block reflects the actual routed request posture rather than inferred or duplicated logic.
- **Files**:
  - `crates/world-agent/src/handlers.rs`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `C-07`
- **Acceptance criteria**:
  - `requested` reflects the landed Snapshot/WorldSpec routing semantics
  - `world_netfilter_enable_present` reflects the landed `WORLD_NETFILTER_ENABLE` guard posture
  - the block does not conflate requested state with runtime enablement
- **Test notes**:
  - use world-doctor handler or fixture coverage to pin each state transition

Checklist:
- Implement: request/guard field derivation
- Test: handler coverage for requested vs guard permutations
- Validate: operator can distinguish host-gate issues from service-env issues
