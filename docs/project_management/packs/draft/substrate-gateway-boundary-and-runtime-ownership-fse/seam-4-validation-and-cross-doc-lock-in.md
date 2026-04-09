---
seam_id: SEAM-4
seam_slug: validation-and-cross-doc-lock-in
type: conformance
status: proposed
execution_horizon: future
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - manual testing assertions drift from landed operator/runtime contracts
    - `plan.md`, `tasks.json`, or checkpoint boundaries drift from the accepted seam and slice ordering
    - docs/CONFIGURATION, docs/USAGE, docs/WORLD, or docs/TRACE restate stale ownership or schema wording
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---

# SEAM-4 - Validation and cross-doc lock-in

- **Goal / value**:
  - Prove that every ADR-0040 surface has one owner and that docs, manual validation, plan/task wiring, and quality-gate evidence all reflect the same landed truth.
  - Prevent late drift where the contract is correct in one document but stale in operator docs, planning automation, or cross-platform validation evidence.
- **Scope**
  - In:
    - `manual_testing_playbook.md`
    - `plan.md`, `tasks.json`, `session_log.md`, and `quality_gate_report.md`
    - `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md`
    - checkpoint boundary alignment and one-owner-per-surface review
    - correction tracking for stale cross-ADR and stale `packs/active/...` references that remain relevant to the feature
  - Out:
    - new command semantics
    - new status-schema fields or policy rules
    - runtime transport design changes
    - provisioning or backend-warm implementation changes
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - `pre-planning/ci_checkpoint_plan.md`
    - `pre-planning/impact_map.md`
    - accepted workstream and slice ordering from `pre-planning/workstream_triage.md`
  - Outputs:
    - one-owner-per-surface manual validation evidence
    - aligned docs and planning automation
    - pack closeout evidence that the ownership boundary stayed coherent end to end
- **Key invariants / rules**:
  - manual validation must prove exactly one owner per surface
  - docs must consume the landed contracts rather than restate stale ADR or archived-pack language
  - task/checkpoint wiring must match the accepted planning spine and eventual seam-local reality
  - this seam does not redefine upstream contracts; it locks conformance around them
- **Dependencies**
  - Direct blockers:
    - `SEAM-3`
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - Transitive blockers:
    - none beyond the upstream contract and runtime seams
  - Direct consumers:
    - none inside this pack; this is the terminal conformance seam
  - Derived consumers:
    - promotion and quality-gate reviewers
    - future runtime or docs follow-on packs
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/quality_gate_report.md`
  - `docs/CONFIGURATION.md`
  - `docs/USAGE.md`
  - `docs/WORLD.md`
  - `docs/TRACE.md`
- **Verification**:
  - This seam consumes upstream contracts `C-01`, `C-02`, `C-03`, and `C-04`; verification may depend on accepted upstream evidence for command semantics, schema ownership, policy boundaries, and parity guarantees.
  - This seam does not introduce a new durable contract surface. At seam-brief depth, verification is the conformance plan becoming concrete enough for seam-local planning and implementation: review checklist, doc touch set, checkpoint wiring, and quality-gate evidence shape.
  - Later seam-local verification should prove:
    - every ADR-0040 surface has one and only one owner
    - manual playbook assertions match the landed contracts
    - docs/CONFIGURATION, docs/USAGE, docs/WORLD, and docs/TRACE reflect the same truth
    - `plan.md`, `tasks.json`, and checkpoint boundaries match the accepted planning spine and landed seam ordering
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Risks / unknowns**:
  - Risk:
    - docs and quality-gate artifacts can lag the landed contract wording even when implementation is correct
  - De-risk plan:
    - keep the manual playbook and docs touch set inside the same seam so conformance is reviewed as one unit
  - Risk:
    - `tasks.json` and checkpoint boundaries can drift from the accepted five-slice planning spine
  - De-risk plan:
    - treat slice/checkpoint alignment as first-class validation evidence in seam-local review
  - Risk:
    - stale `packs/active/...` links in adjacent ADRs can keep leaking incorrect ownership cues into follow-on work
  - De-risk plan:
    - track link normalization as explicit conformance evidence or carried-forward follow-up, not as an implicit cleanup hope
- **Rollout / safety**:
  - This seam should only activate after the upstream contracts and runtime/parity expectations are concrete enough to verify directly.
  - Safety comes from making drift visible before promotion rather than after downstream packs inherit stale wording.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `future` because conformance only makes sense once upstream seams have published authoritative truth to compare against.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - What the first seam-local review should focus on
    - one-owner-per-surface checklist
    - cross-doc wording parity
    - plan/task/checkpoint alignment
    - stale-link normalization posture
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none; this seam locks conformance around previously published contracts
  - Threads likely to advance:
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - Review-surface areas likely to shift after landing:
    - manual validation checklist
    - doc touch map
    - quality-gate evidence
  - Downstream seams most likely to require revalidation:
    - none inside this pack; later follow-on packs should reopen only if conformance evidence finds drift
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
