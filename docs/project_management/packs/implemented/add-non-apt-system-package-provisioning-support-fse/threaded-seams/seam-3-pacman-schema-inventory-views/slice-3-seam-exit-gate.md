---
slice_id: S3
seam_id: SEAM-3
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - any downstream seam begins planning against APT-only inventory method assumptions
  - any downstream seam treats `install.pacman` ordering or non-runnable scope as non-authoritative
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the deterministic handoff that downstream seam promotion may consume once `SEAM-3` has landed and closeout is recorded.

#### Required closeout records

- Update `../../governance/seam-3-closeout.md` with:
  - landed evidence summary for `C-03` publication and stable artifact location(s)
  - thread state update: `THR-03` -> `published`
  - additive compatibility record:
    - `version: 1` preserved
    - no translation layer introduced
    - pacman-backed packages remain non-runnable prerequisites in v1
  - inventory validation and view evidence:
    - valid pacman-backed package acceptance
    - invalid-state rejection with exit `2`
    - view rendering that preserves `pacman` and authored `install.pacman` order
  - review-surface delta list versus `../../review_surfaces.md`
  - remediation disposition:
    - confirm `SEAM-3` owns no open blocking remediations at closeout
    - reference `REM-003` only as downstream context owned by `SEAM-4`
  - `seam_exit_gate.status` and `seam_exit_gate.promotion_readiness`

#### Promotion readiness criteria (for downstream seams consuming `THR-03`)

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-03` is explicitly recorded as `published`
- Any downstream-relevant stale triggers are recorded (method vocabulary, `install.pacman` shape, non-runnable scope, and view rendering)

#### Evidence checklist (what must be captured)

- The published `C-03` contract artifact location(s)
- A short validation and view evidence summary that downstream seams can cite without re-deriving schema semantics
- A planned-vs-landed delta note confirming:
  - schema support stayed additive on `version: 1`
  - `pacman` remained explicit in inventory views
  - non-runnable v1 pacman scope remained intact
