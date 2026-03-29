---
slice_id: S1
seam_id: SEAM-01
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - parser rule drift from `contract.md`
    - alternate-input hook semantics drift from `DR-0003`
gates:
  pre_exec:
    review: inherited
    contract: pending
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-07
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Parser and input contract definition

- **User/system value**: downstream seams get one seam-local contract for selected-input resolution and parser behavior instead of re-deriving parser semantics from a mixed implementation and source-pack narrative.
- **Scope (in/out)**:
  - In: pin the exact seam-local rules for `C-01` and `C-02`, including path validation, no-fallback posture, parser rule table, emitted normalized fields, and verification checklist
  - Out: implementing manager-family mapping, decision-line wording, explicit override precedence, or exit taxonomy
- **Acceptance criteria**:
  - the slice records the exact rule that unset or empty `SUBSTRATE_INSTALL_OS_RELEASE_PATH` uses `/etc/os-release`, while a non-empty invalid or unreadable alternate path yields `<unknown>` without fallback
  - the slice records the parser rule table for comments, duplicate assignments, quote stripping, lowercase normalization, and `ID` / `ID_LIKE` key filtering
  - the slice lists the narrow verification matrix and pass/fail conditions execution must satisfy before `SEAM-01` can pass `gates.pre_exec.contract`
- **Dependencies**:
  - `../../seam-01-os-release-input-parser.md`
  - `../../threading.md`
  - source authority in `../../../best-effort-distro-package-manager/contract.md`
  - source authority in `../../../best-effort-distro-package-manager/decision_register.md`
- **Verification**:
  - seam-local contract checklist covers valid alternate file, invalid relative path, unreadable file, missing keys, quoted values, duplicate assignments, and comment-bearing inputs
  - pass condition: no rule required by `C-01` or `C-02` remains implicit or delegated to downstream seams
- **Rollout/safety**:
  - prevents unsafe fallback to implementation inference
  - keeps shell-execution risk and downstream contract drift visible before coding starts
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` active seam focus

For a contract-definition slice that produces an owned contract:

- make the contract rules concrete enough that the producer seam can later satisfy `gates.pre_exec.contract`
- include a narrow verification plan with test locations, edge cases, and pass/fail conditions
- do not require the final accepted contract artifact to exist before the producer seam can become `exec-ready`

#### S1.T1 - Freeze seam-local rules for `C-01` and `C-02`

- **Outcome**: the seam-local plan states one exact selected-input rule set, one exact parser rule set, and one exact `distro_id` / `distro_id_like` emission contract.
- **Inputs/outputs**:
  - Inputs: source pack `contract.md`, `decision_register.md`, `BEDPM0-spec.md`
  - Outputs: seam-local contract bullets that later slices consume without reinterpretation
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-01`, `C-02`
- **Implementation notes**:
  - explicitly separate parser/input truth from manager selection and decision-line reporting

#### S1.T2 - Define the verification checklist for contract readiness

- **Outcome**: execution inherits one concrete edge-case matrix instead of inventing parser fixtures ad hoc.
- **Inputs/outputs**:
  - Inputs: source acceptance criteria from `BEDPM0-spec.md`
  - Outputs: test locations, edge cases, and pass/fail rules for unreadable paths, duplicate keys, comments, quotes, and lowercase normalization
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-01`, `C-02`
- **Implementation notes**:
  - point future harness work at `tests/installers/pkg_manager_detection_smoke.sh` without making SEAM-01 own SEAM-06's validation topology
