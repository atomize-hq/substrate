---
slice_id: S3
seam_id: SEAM-4
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - any downstream seam assumes mixed-manager provisioning can partially mutate the world
  - any downstream seam treats request-profile routing or pacman command shape as non-authoritative
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the deterministic handoff that downstream seam promotion may consume once `SEAM-4` has landed and closeout is recorded.

#### Required closeout records

- Update `../../governance/seam-4-closeout.md` with:
  - landed evidence summary for `C-04` publication and stable artifact location(s)
  - thread state update: `THR-04` -> `published`
  - fail-closed compatibility record:
    - mixed-manager rejection happens before mutation
    - request-profile routing stays internal
    - pacman execution uses the exact command shape with no fallback, retries, or AUR-helper widening
  - provisioning evidence:
    - normalized requirement derivation
    - mixed-manager exit `4`
    - dry-run / verbose rendering
    - no-op behavior for empty detected-manager requirement sets
  - review-surface delta list versus `../../review_surfaces.md`
  - remediation disposition:
    - confirm `SEAM-4` owns no open blocking remediations at closeout
    - record `REM-003` as resolved revalidation evidence rather than a carry-forward blocker
  - `seam_exit_gate.status` and `seam_exit_gate.promotion_readiness`

#### Promotion readiness criteria (for downstream seams consuming `THR-04`)

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-04` is explicitly recorded as `published`
- Any downstream-relevant stale triggers are recorded (normalization rules, request-profile posture, pacman command shape, and dry-run / verbose rendering)

#### Evidence checklist (what must be captured)

- The published `C-04` contract artifact location(s)
- A short provisioning evidence summary that downstream seams can cite without re-deriving execution semantics
- A planned-vs-landed delta note confirming:
  - mixed-manager provisioning stayed fail-closed
  - pacman execution stayed exact and manager-specific
  - request-profile routing remained internal
