---
slice_id: S3
seam_id: SEAM-5
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`THR-05` cannot be advanced because public identity, transport factoring, or event evidence still depends on provider parsing details"
    - landed `C-05` or `C-06` behavior leaks planner/executor role truth into public docs, config, or identity
    - closeout reveals policy deltas that force future Substrate work to re-plan against different boundary assumptions
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-05
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-05
  - C-06
open_remediations: []
---
### S3 - Seam Exit Gate

- **User/system value**: downstream integration work consumes closeout-backed truth about the external boundary instead of assuming the gateway is safe because the service exists.
- **Scope (in/out)**:
  - In: capture landed `C-05` and `C-06` evidence, `THR-05` publication state, boundary verification results, review-surface deltas, stale triggers, remediation disposition, and promotion readiness.
  - Out: unfinished identity or event-boundary work, provider normalization changes, and later Substrate implementation work that belongs outside this seam.
- **Acceptance criteria**:
  - `../../governance/seam-5-closeout.md` records the seam-exit source ref, landed `C-05`/`C-06` evidence, verification evidence, and the `THR-05` publication decision
  - `THR-05` advances from `identified` to `published` only if one logical backend identity and normalized structured events are demonstrated on top of the landed upstream seams
  - downstream stale triggers for future Substrate work are explicit when boundary assumptions change materially
  - promotion readiness is `ready` only if no blocking post-exec issue requires downstream seams to inspect provider parsing or public/internal identity drift to proceed
  - the closeout names the exact owned contract artifacts and runtime evidence sources so future promotion does not rely on pack memory
- **Landed outputs**:
  - seam closeout target: `../../governance/seam-5-closeout.md`
  - owned contract evidence targets: `docs/foundation/substrate-boundary-c05-contract.md` and `docs/foundation/substrate-structured-events-c06-contract.md`
  - runtime evidence anchors: `gateway/src/main.rs`, `gateway/src/cli/mod.rs`, `gateway/src/server/mod.rs`, `gateway/src/providers/openai.rs`, and the `gateway/tests/fixtures/azure_kimi/` regression corpus
- **Dependencies**: `S1`, `S2`, `../../threading.md`, `../../governance/remediation-log.md`, `../../governance/seam-5-closeout.md`, `THR-05`, `C-05`, and `C-06`
- **Verification**:
  - the closeout artifact names the seam-exit source, contract publication state, thread state, planned-versus-landed delta, and promotion readiness
  - pass condition: future Substrate work can later promote on closeout-backed `C-05`/`C-06` truth without reverse-engineering runtime behavior
  - failure conditions are explicit: incomplete boundary verification, unresolved identity leakage, or event behavior that still depends on provider parsing details
- **Rollout/safety**: do not hide unfinished boundary work inside seam exit; if the contract or verification is incomplete, promotion readiness must remain `blocked`.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md`

#### S3.T1 - Capture Landed Boundary Evidence

- **Outcome**: closeout records the owned `C-05` and `C-06` artifacts plus the runtime/config/test evidence that demonstrates the boundary actually landed.
- **Inputs/outputs**: inputs are the landed outputs of `S1` and `S2`; output is a populated `../../governance/seam-5-closeout.md` record with source refs, evidence links, and publication accounting.
- **Thread/contract refs**: `THR-05`, `C-05`, `C-06`
- **Implementation notes**: the closeout must cite both the contract notes and the concrete runtime anchors so future promotion never has to infer which surfaces embody the boundary.

#### S3.T2 - Decide THR-05 Publication And Downstream Stale Triggers

- **Outcome**: `THR-05` ends with an explicit publish-or-hold decision and downstream seams inherit concrete revalidation triggers.
- **Inputs/outputs**: inputs are the landed `C-05`/`C-06` evidence set, boundary verification results, and any unresolved drift; output is the `THR-05` decision plus stale-trigger language inside closeout.
- **Thread/contract refs**: `THR-05`, `C-05`, `C-06`
- **Implementation notes**: only publish if downstream consumers can reason from the owned contracts and evidence without reopening provider parsing or public/internal identity questions.

#### S3.T3 - State Promotion Readiness And Blocker Posture

- **Outcome**: the seam ends with a clear downstream handoff signal instead of an implicit assumption that boundary work is done.
- **Inputs/outputs**: inputs are landing evidence, `THR-05` state, and remediation posture; output is the promotion-readiness and blocker section in `../../governance/seam-5-closeout.md`.
- **Thread/contract refs**: `THR-05`, `C-05`, `C-06`
- **Implementation notes**: if boundary verification is incomplete or downstream work would still need runtime reverse engineering, readiness must stay `blocked` and the closeout must say why.
