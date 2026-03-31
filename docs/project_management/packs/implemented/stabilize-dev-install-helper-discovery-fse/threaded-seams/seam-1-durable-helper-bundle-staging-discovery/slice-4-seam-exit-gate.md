---
slice_id: S4
seam_id: SEAM-1
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - downstream seams consume provisional bundle facts instead of closeout-backed evidence
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
contracts_produced:
  - C-01
  - C-02
  - C-03
contracts_consumed: []
open_remediations:
  - REM-001
candidate_subslices: []
---
### S4 - Seam-exit gate

This slice plans the deterministic handoff that downstream seam promotion may consume once `SEAM-1` has landed and closeout is recorded.

#### Required closeout records

- Update `../../governance/seam-1-closeout.md` with:
  - landed staged bundle inventory for the exact `C-02` path surface under `$SUBSTRATE_HOME`
  - landed managed-asset evidence showing which assets are repo-managed symlinks and whether any Linux guest binaries landed as manifest-tracked copies instead of symlinks
  - landed helper-resolution evidence proving:
    - env override remains first
    - prefix helper beats inferred version-dir helper
    - missing helper candidates remain fail-closed
    - `--home` remains valid and `--prefix` remains invalid
  - thread state updates:
    - `THR-01` -> `published`
    - `THR-02` -> `published`
  - contracts published or changed:
    - `C-01`
    - `C-02`
    - `C-03`
  - any review-surface delta discovered during landing, especially:
    - staged path-list differences
    - helper-order or wording differences
    - managed-asset boundary differences
    - macOS scope drift
  - remediation disposition:
    - record whether `REM-001` is resolved, accepted risk, or carried forward
    - record any ADR-0035 revalidation performed during landing if shared install-script or helper-script surfaces moved after this promotion
  - `seam_exit_gate.status` and `seam_exit_gate.promotion_readiness`

#### Promotion readiness criteria

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-01` and `THR-02` are explicitly recorded as `published`
- the closeout records one landed durable bundle surface and one landed helper-order truth that `SEAM-2` and `SEAM-3` can consume
- any downstream-relevant stale triggers are recorded before promotion:
  - path-list drift
  - helper-order or helper-missing wording drift
  - managed-asset eligibility drift
  - macOS scope drift beyond helper discovery and dry-run proof

#### Evidence checklist

- One closeout-backed inventory of the staged durable bundle surface
- One closeout-backed record of managed symlink versus manifest-tracked copied-binary evidence
- One proof point that `cargo clean` no longer breaks helper discovery when the prefix helper remains
- One recorded test or smoke evidence set for helper precedence and CLI surface
- One planned-vs-landed delta note covering any review-surface movement that downstream seams must revalidate against
