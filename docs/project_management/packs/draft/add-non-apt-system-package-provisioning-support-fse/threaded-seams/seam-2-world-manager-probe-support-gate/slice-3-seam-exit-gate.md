---
slice_id: S3
seam_id: SEAM-2
slice_kind: seam_exit_gate
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
  - any downstream seam begins planning against host-derived manager detection
  - any downstream seam treats `/etc/os-release` as non-authoritative for family selection
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced:
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the deterministic handoff that downstream seam promotion may consume once `SEAM-2` has landed and closeout is recorded.

#### Required closeout records

- Update `../../governance/seam-2-closeout.md` with:
  - landed evidence summary for `C-02` publication and stable artifact location(s)
  - thread state update: `THR-02` -> `published`
  - explicit platform/backend posture record:
    - Linux host-native provisioning unsupported -> exit `4`
    - macOS Lima guest supported -> manager selection via `C-02`
    - Windows WSL provisioning unsupported -> exit `4`
  - supported/unsupported probe evidence:
    - Debian-family success lane (`apt`) on the default macOS Ubuntu guest
    - Arch-family pacman-success lane evidence if still required (manual-only lane)
    - contradiction/unmapped lanes (exit `4`) with deterministic reason labels
  - “review surface delta” list versus `../../review_surfaces.md` (capture any drift as downstream stale triggers)
  - remediation disposition:
    - confirm `SEAM-2` owns no open blocking remediations at closeout
    - reference `REM-001` / `REM-002` only as downstream context (owned by `SEAM-6`)
  - `seam_exit_gate.status` and `seam_exit_gate.promotion_readiness`

#### Promotion readiness criteria (for downstream seams consuming `THR-02`)

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-02` is explicitly recorded as `published`
- Any downstream-relevant stale triggers are recorded (mapping rules, contradiction policy, platform posture, exit `4` semantics)

#### Evidence checklist (what must be captured)

- The published `C-02` contract artifact location(s)
- A short supported/unsupported evidence summary that downstream seams can cite without re-deriving semantics
- A “planned vs landed” delta note confirming:
  - probe runs in-world (not on host)
  - `/etc/os-release` remains authoritative for family selection
  - contradictions and unmapped cases remain fail-closed

