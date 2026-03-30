---
slice_id: S2
seam_id: SEAM-03
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - `PKG_MANAGER` precedence changes
    - explicit-success reporting for source `env` changes
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
contracts_produced:
  - C-05
contracts_consumed:
  - C-01
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S2 - Env selector handling

- **User/system value**: environment-driven explicit selection stays deterministic when the CLI flag is absent and does not fall through to lower-precedence stages.
- **Scope (in/out)**:
  - In: `PKG_MANAGER` selection when `--pkg-manager` is absent, allowed-value validation, `pkg_manager.source=env`, and explicit-success handoff into the inherited decision line
  - Out: flag parsing, explicit failure taxonomy ownership, ordered `PATH` fallback, warning line, and exit `4`
- **Acceptance criteria**:
  - `PKG_MANAGER` applies only when `--pkg-manager` is absent
  - a valid env selection that is available in `PATH` sets `pkg_manager.source=env`
  - a valid env selection never falls through to os-release mapping or `PATH` probing
  - explicit-success reporting reuses the inherited decision-line template with source `env`
- **Dependencies**:
  - `S1`
  - `scripts/substrate/install-substrate.sh`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves env wins over os-release mapping and path probe when the flag is absent
  - pass condition: env-driven success uses only the allowed manager vocabulary and the inherited decision-line template
- **Rollout/safety**:
  - keeps env semantics explicit without widening into fallback ownership
  - preserves `SEAM-02` reporting vocabulary and timing
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` next seam focus

#### S2.T1 - Honor `PKG_MANAGER` after flag absence is established

- **Outcome**: the env-selector stage becomes concrete enough to implement without ambiguity or hidden precedence drift.
- **Inputs/outputs**:
  - Inputs: `PKG_MANAGER`, allowed manager vocabulary
  - Outputs: selected manager plus `pkg_manager.source=env`
- **Thread/contract refs**:
  - `THR-02`, `THR-03`
  - `C-05`
- **Implementation notes**:
  - keep env selection below the flag and above os-release mapping or path probe
