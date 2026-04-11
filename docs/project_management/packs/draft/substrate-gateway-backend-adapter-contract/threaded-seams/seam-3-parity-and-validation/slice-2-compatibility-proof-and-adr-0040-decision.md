---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR-0024 supersession posture changes
    - ADR-0040 evidence-only posture changes
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
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations:
  - REM-004
---
### S2 - Compatibility proof and ADR-0040 decision

#### Goal

Lock the additive compatibility proof and resolve whether ADR-0040 remains evidence-only or must enter the direct touch set before this seam can become `exec-ready`.

#### Dependencies

- `../../governance/remediation-log.md`
- `../../governance/seam-2-closeout.md`
- `../../review_surfaces.md`

#### S2.T1 - Prove ADR-0024 is historical evidence only

- **Outcome**:
  - `compatibility-spec.md` states the additive rollout posture and historical-evidence-only supersession rule for ADR-0024.
- **Files**:
  - `../../compatibility-spec.md`
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
- **Acceptance criteria**:
  - existing operator workflows remain additive and compatible
  - the proof does not restate or widen upstream contracts

#### S2.T2 - Resolve ADR-0040 alignment posture

- **Outcome**:
  - `REM-004` is either resolved by confirming ADR-0040 is evidence-only or converted into a direct touch-surface decision captured in seam-local planning.
- **Files**:
  - `../../compatibility-spec.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `../../governance/remediation-log.md`
- **Acceptance criteria**:
  - the decision is explicit and no longer implied
  - `SEAM-3` can explain whether ADR-0040 remains evidence-only before `exec-ready` is claimed
