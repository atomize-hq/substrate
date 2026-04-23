---
slice_id: S3
seam_id: SEAM-1
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - shell tests, supporting ADR-0046 docs, or review surfaces drift from canonical `C-01` and `C-02`
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
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S3 - Add drift guards and adoption evidence for `THR-01`

- **User/system value**:
  - Gives reviewers and downstream seams one reliable proof bundle that shell behavior, shell tests, and supporting docs all adopt the published contracts the same way.
- **Scope (in/out)**:
  - In: supporting doc alignment, shell test coverage, review-surface refresh, and publication-evidence notes
  - Out: new selection or policy semantics beyond what `S00`-`S2` already define
- **Acceptance criteria**:
  - supporting ADR-0046 docs align behind the canonical `docs/contracts/` refs
  - shell tests cover the main selection and auth-precedence invariants
  - review surfaces and closeout evidence targets are updated so `THR-01` publication is auditable
- **Dependencies**:
  - `S1`
  - `S2`
  - `THR-01`
  - `C-01`
  - `C-02`
- **Verification**:
  - `cargo test -p substrate-shell world_gateway -- --nocapture` or narrower targeted coverage matching the landed code paths
- **Rollout/safety**:
  - drift guards should fail fast when docs and shell behavior diverge
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`
  - `../../governance/seam-1-closeout.md`

#### S3.T1 - Align supporting ADR-0046 docs to the canonical owner line

- **Outcome**:
  - feature-local ADR-0046 docs become supporting implementation surfaces only and clearly defer to canonical `C-01`/`C-02` refs.
- **Inputs/outputs**:
  - Inputs: canonical contract docs, `contract.md`, `policy-spec.md`, `env-vars-spec.md`
  - Outputs: aligned supporting docs and review-surface references
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
  - `C-02`
- **Implementation notes**:
  - keep planning IDs out of canonical contract docs
  - keep feature-local docs descriptive of implementation detail, not competing contract owners
- **Acceptance criteria**:
  - downstream planning cannot mistake supporting ADR-0046 docs for canonical publication targets
- **Test notes**:
  - doc diff review against canonical refs
- **Risk/rollback notes**:
  - if supporting docs drift, downstream seams will re-open stale authority questions immediately

Checklist:
- Implement:
  - align supporting ADR-0046 docs to the canonical owner line
- Test:
  - review for contradictory wording against `docs/contracts/`
- Validate:
  - confirm `THR-01` publication evidence has one durable source of truth

#### S3.T2 - Install test and closeout drift guards for `THR-01`

- **Outcome**:
  - the seam has explicit test and closeout evidence targets for selection and policy invariants.
- **Inputs/outputs**:
  - Inputs: landed shell behavior, canonical docs, `../../governance/seam-1-closeout.md`
  - Outputs: updated test coverage and ready-to-fill closeout evidence targets
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
  - `C-02`
- **Implementation notes**:
  - capture the exact evidence downstream seams need: contract paths, shell tests, and any review-surface deltas
- **Acceptance criteria**:
  - closeout can name landed evidence without reconstructing it after the fact
- **Test notes**:
  - preserve:
    - `world_gateway_invalid_integration_uses_exit_code_2`
    - `world_gateway_transient_runtime_failures_use_exit_code_3`
    - `world_gateway_policy_failures_use_exit_code_5`
    - `world_gateway_empty_default_backend_uses_exit_code_2`
  - add:
    - unsupported backend rejection before socket contact
    - env wins over auth file when both sources are present
    - env-allowed denial with no file fallback
- **Risk/rollback notes**:
  - missing drift guards will make `THR-01` publication non-deterministic even if docs look correct

Checklist:
- Implement:
  - add or update test and closeout evidence hooks for `THR-01`
- Test:
  - run targeted shell gateway coverage
- Validate:
  - confirm the seam-exit slice has concrete evidence to consume
