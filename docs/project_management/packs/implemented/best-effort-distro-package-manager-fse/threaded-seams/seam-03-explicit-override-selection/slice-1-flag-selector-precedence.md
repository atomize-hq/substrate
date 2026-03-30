---
slice_id: S1
seam_id: SEAM-03
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - selector precedence changes
    - supported manager vocabulary changes
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
### S1 - Flag selector precedence

- **User/system value**: a CLI flag can force the package manager deterministically and outrank lower-precedence selectors without reopening parser, mapping, or fallback ownership.
- **Scope (in/out)**:
  - In: `--pkg-manager` parsing, allowed-value validation, precedence over `PKG_MANAGER`, `pkg_manager.source=flag`, and explicit-success handoff into the inherited decision line
  - Out: env-only selection, explicit failure remediation branches beyond the narrow flag validation boundary, ordered `PATH` fallback, warning line, and exit `4`
- **Acceptance criteria**:
  - when both `--pkg-manager` and `PKG_MANAGER` are present, the flag wins
  - a valid flag selection that is available in `PATH` sets `pkg_manager.source=flag`
  - a valid flag selection never falls through to os-release mapping or `PATH` probing
  - explicit-success reporting reuses the inherited decision-line template with source `flag`
- **Dependencies**:
  - `../../seam-03-explicit-override-selection.md`
  - `../../threading.md`
  - `../seam-02-family-mapping-reporting/seam.md`
  - `../../../best-effort-distro-package-manager/contract.md`
  - `../../../best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- **Verification**:
  - fixture coverage proves a valid flag beats env, os-release, and path-probe branches
  - pass condition: flag-driven success remains deterministic and uses only the allowed manager vocabulary
- **Rollout/safety**:
  - keeps operator-forced behavior fail-closed and explicit
  - preserves `SEAM-02` ownership for the base decision-line wording
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` next seam focus

#### S1.T1 - Parse and prioritize `--pkg-manager`

- **Outcome**: the highest-precedence selector becomes concrete enough to implement without ambiguity.
- **Inputs/outputs**:
  - Inputs: CLI args, supported manager vocabulary
  - Outputs: selected manager plus `pkg_manager.source=flag`
- **Thread/contract refs**:
  - `THR-02`, `THR-03`
  - `C-05`
- **Implementation notes**:
  - do not consult lower-precedence env, os-release, or path-probe stages after a valid flag selection
