---
seam_id: SEAM-3
seam_slug: parity-and-validation
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-3-parity-and-validation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - backend-id or published status subset semantics change upstream
    - adapter protocol/schema publication changes upstream
    - Linux/macOS/Windows guarantee wording changes
    - ADR-0024 supersession posture changes
    - ADR-0040 alignment stops being evidence-only or widens into direct touch surfaces
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
  promotion_readiness: ready
open_remediations: []
---
# SEAM-3 - Parity and validation

This seam is closed. Its authoritative exit-gate record lives in `../../governance/seam-3-closeout.md`.

## Seam Brief (Restated)

- **Goal / value**:
  - Prove the adapter contract is additive, cross-platform, and compatible with the already accepted ownership split now that the upstream selection, protocol, and schema seams have published concrete truth.
- **Type**:
  - conformance
- **Scope**
  - In:
    - `platform-parity-spec.md`
    - `compatibility-spec.md`
    - `manual_testing_playbook.md`
    - `pre-planning/ci_checkpoint_plan.md`
    - Linux/macOS/Windows guarantee matrix
    - ADR-0024 historical-evidence-only supersession proof
    - ADR-0040 alignment posture for runtime ownership evidence
    - deterministic validation gates for document review, ambiguity scan, and platform proof
  - Out:
    - backend-id selection contract definition
    - adapter lifecycle or schema definition
    - widening the operator contract or status schema beyond accepted upstream truth
    - introducing new runtime ownership that ADR-0040 does not already allow
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/ci_checkpoint_plan.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Verification**:
  - This seam consumes `C-01` through `C-04` from `../../governance/seam-1-closeout.md` and `../../governance/seam-2-closeout.md`.
  - `THR-01` and `THR-02` are now published inbound threads for this seam.
  - ADR-0040 now stays explicit evidence-only basis for this seam:
    - ADR-0040 remains the owner of the Substrate versus `substrate-gateway` runtime boundary.
    - ADR-0041 and `docs/contracts/substrate-gateway-runtime-parity.md` carry the downstream consequences that this seam must prove.
    - direct ADR-0040 edits stay out of scope unless landing evidence uncovers a concrete owner-line mismatch.

## Review bundle

- `review.md` is the authoritative artifact for the current pre-exec posture.
- The pre-exec review passed the falsifiability, contract-consumption, and revalidation checks that allowed this seam to reach `status: exec-ready` before landing.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - the final seam must publish parity, compatibility, and validation proof without inventing a second control plane or widening accepted upstream contracts.
- **Expected threads to advance**:
  - `THR-01`: `published` -> `revalidated`
  - `THR-02`: `published` -> `revalidated`
- **Expected closeout evidence**:
  - the final Linux/macOS/Windows guarantee matrix
  - the ADR-0024 supersession and compatibility proof
  - the resolved ADR-0040 alignment posture
  - the manual validation and checkpoint evidence bundle

## Slice index

- `S1` -> `slice-1-platform-parity-and-runtime-boundary.md`
- `S2` -> `slice-2-compatibility-proof-and-adr-0040-decision.md`
- `S3` -> `slice-3-validation-gate-and-checkpoint-bundle.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeouts consumed here:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- Seam closeout:
  - `../../governance/seam-3-closeout.md`
- Current blocking remediation:
  - none at pre-exec; `REM-004` is resolved in `../../governance/remediation-log.md`
