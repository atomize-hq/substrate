---
slice_id: S1
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - adapter protocol or schema publication changes upstream
    - Linux/macOS/Windows guarantee wording changes
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
open_remediations: []
---
### S1 - Platform parity and runtime boundary

#### Goal

Define the Linux/macOS/Windows guarantee matrix and the runtime-boundary proof that later validation must land without widening the already accepted upstream contracts.

#### Dependencies

- `../../governance/seam-1-closeout.md`
- `../../governance/seam-2-closeout.md`
- `../../review_surfaces.md`

#### S1.T1 - Define the cross-platform guarantee matrix

- **Outcome**:
  - `platform-parity-spec.md` states the exact guarantees that must hold on Linux, macOS, and Windows for adapter-backed execution.
- **Files**:
  - `../../platform-parity-spec.md`
  - `../../pre-planning/ci_checkpoint_plan.md`
- **Acceptance criteria**:
  - the stable backend-id and allowlist semantics remain invariant across Linux, macOS, and Windows
  - hidden transport or bootstrap divergence stays out of the public contract unless explicitly published
  - any WSL stance is stated explicitly rather than inferred

#### S1.T2 - Lock the runtime-boundary proof

- **Outcome**:
  - parity proof names the runtime-boundary evidence that must align with `docs/contracts/substrate-gateway-runtime-parity.md`.
- **Files**:
  - `../../platform-parity-spec.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Acceptance criteria**:
  - no second Substrate control plane is introduced
  - runtime-boundary validation cites upstream truth rather than restating it
