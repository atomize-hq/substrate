---
slice_id: S2
seam_id: SEAM-01
slice_kind: delivery
execution_horizon: active
status: landed
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - alternate-input path validation changes
    - no-fallback semantics change
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-01
  - THR-07
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Selected-input resolution and degradation

- **User/system value**: the installer chooses one input source deterministically and turns bad alternate-input state into explicit `<unknown>` fields instead of silent host fallback or ambiguous partial reads.
- **Scope (in/out)**:
  - In: selected-input resolver for `/etc/os-release` versus `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, absolute-path validation, readable-regular-file validation, and no-fallback degradation behavior
  - Out: parsing `ID` / `ID_LIKE`, manager-family mapping, decision-line emission, and override precedence
- **Acceptance criteria**:
  - unset or empty `SUBSTRATE_INSTALL_OS_RELEASE_PATH` selects `/etc/os-release`
  - a non-empty absolute path to a readable regular file replaces `/etc/os-release`
  - a non-empty path that is relative, unreadable, missing, or not a regular file yields `<unknown>` fields and does not read `/etc/os-release`
  - the resolver exports enough internal state for the parser stage to know whether it received a selected file or an unavailable-input condition
- **Dependencies**:
  - `S1`
  - `scripts/substrate/install-substrate.sh`
  - source contract `SUBSTRATE_INSTALL_OS_RELEASE_PATH` rules
- **Verification**:
  - targeted shell fixtures cover unset or empty env, valid alternate file, relative path, unreadable file, and directory path
  - landed slice coverage lives in `tests/installers/pkg_manager_detection_smoke.sh`
  - pass condition: all invalid alternate-input cases degrade without a second file read and without branching into manager selection logic
- **Rollout/safety**:
  - isolates the risky boundary where an invalid test hook could otherwise pull host state back into the run
  - keeps Linux-only behavior local to the installer path
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R1

#### S2.T1 - Implement selected-input resolver in `install-substrate.sh`

- **Outcome**: the installer chooses exactly one os-release input source or an explicit unavailable-input state before any parser logic runs.
- **Inputs/outputs**:
  - Inputs: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, `/etc/os-release`
  - Outputs: selected path or unavailable-input state handed to the parser stage
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-02`
- **Implementation notes**:
  - preserve the rule that a set alternate path disables `/etc/os-release` fallback entirely

#### S2.T2 - Wire `<unknown>` degradation handoff for later seams

- **Outcome**: downstream stages inherit deterministic unavailable-input semantics instead of rechecking filesystem state.
- **Inputs/outputs**:
  - Inputs: selected-input resolver result
  - Outputs: normalized placeholder state that the parser and later selection/reporting seams can consume
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-01`, `C-02`
- **Implementation notes**:
  - keep this handoff purely about parser/input truth; do not emit decision-line or mapping behavior here
