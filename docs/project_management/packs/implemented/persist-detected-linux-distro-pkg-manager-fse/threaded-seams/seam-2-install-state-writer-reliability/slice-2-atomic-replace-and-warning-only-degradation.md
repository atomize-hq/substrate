---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - published `C-01` or `C-02` changes required merge or canonical-path behavior before closeout
    - installer temp-file or warning-only scaffolding changes before landing
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations:
  - REM-003
candidate_subslices: []
---
### S2 - Atomic replace and warning-only degradation

- **User/system value**: one explicit reliability contract keeps metadata persistence additive and fail-open instead of letting partial writes or parse failures create ambiguous state for downstream conformance.
- **Scope (in/out)**:
  - In: same-directory temp-file rendering complete-document-before-replace behavior warning-only read/write/replace degradation invalid-JSON and non-`1` schema fallback and prior-state preservation rules
  - Out: redefining payload fields, redefining the write matrix itself, smoke assertions, and uninstall cleanup alignment
- **Acceptance criteria**:
  - temp files live in the same directory as the canonical file and the replace step happens only after a complete JSON document exists
  - in-place truncation is not allowed
  - unreadable invalid or non-`1` schema content degrades to warning-only behavior instead of install failure
  - failed temp-file write or replace does not destroy prior canonical content
  - the slice carries `REM-003` only as a visible follow-up and does not broaden execution into uninstaller cleanup
- **Dependencies**:
  - `S1`
  - `../../governance/seam-1-closeout.md`
  - `review.md`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `tests/installers/install_state_smoke.sh`
- **Verification**:
  - pass condition: reliability semantics are explicit enough that downstream smoke coverage can prove the landed warning-only and atomicity story without reconstructing it from code history
  - planned evidence cross-checks the current temp-file and warning logging scaffold in both installers plus the current smoke harness compatibility scenarios
- **Rollout/safety**:
  - preserves installer success posture for metadata issues
  - keeps reliability work separate from the out-of-scope cleanup-reader follow-up
- **Review surface refs**:
  - `review.md` R2
  - `../../review_surfaces.md` R2
  - `../../review_surfaces.md` R4

#### S2.T1 - Freeze same-directory temp-file replacement for `C-04`

- **Outcome**: the writer contract names one temp-file path rule and one replace rule across both installers.
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Acceptance criteria**:
  - temp-file path is `<effective_prefix>/install_state.json.tmp`
  - replace happens only after complete JSON exists
  - in-place truncation is not allowed

#### S2.T2 - Freeze warning-only fallback and prior-state preservation

- **Outcome**: invalid-file and write-failure behavior is explicit before conformance work begins.
- **Thread/contract refs**:
  - `THR-02`
  - `C-04`
- **Acceptance criteria**:
  - invalid JSON unreadable files and non-`1` schema input degrade to warning-only handling
  - temp-file write or replace failure preserves prior canonical content when present
  - metadata failures do not change an otherwise successful installer exit status

## Contract freeze for `C-04`

- Writers render the next document to `<effective_prefix>/install_state.json.tmp`.
- The temp file lives in the same directory as the canonical file.
- Replace occurs only after the temp file contains a complete JSON document.
- In-place truncation of the canonical file is not allowed.
- Invalid unreadable or non-`1` schema input degrades to warning-only handling before rebuild.
- Failed temp-file write or replace preserves prior canonical content when that content exists.
- Metadata read or write failure does not change an otherwise successful installer exit status.

## Verification checklist for `C-04` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Same-directory temp file | hosted and dev installer metadata writers | both installers use `install_state.json.tmp` in the canonical file directory. |
| Replace sequencing | hosted and dev installer metadata writers | replace occurs only after complete JSON is rendered. |
| Warning-only invalid-file handling | hosted and dev installer warning paths | parse failure or schema reset stays warning-only and does not fail install. |
| Prior-state preservation | current smoke compatibility scenarios plus landed writer behavior | write or replace failure does not require downstream seams to assume canonical-state truncation. |

Contract-readiness for this slice is documentary: the reliability contract is concrete enough to implement and later verify without reopening warning posture or atomicity scope.
