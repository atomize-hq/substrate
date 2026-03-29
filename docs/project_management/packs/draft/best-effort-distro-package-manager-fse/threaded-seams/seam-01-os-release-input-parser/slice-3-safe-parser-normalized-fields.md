---
slice_id: S3
seam_id: SEAM-01
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - `ID` / `ID_LIKE` extraction rules change
    - quote stripping or duplicate-key handling changes
    - lowercase normalization or `<unknown>` emission changes
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
  - THR-07
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S3 - Safe parser and normalized field handoff

- **User/system value**: later seams receive deterministic `distro_id` and `distro_id_like` values from a parser that never executes os-release content and never broadens beyond the two allowed keys.
- **Scope (in/out)**:
  - In: line parsing, comment skipping, first-`=` splitting, ASCII whitespace trimming, one-pair quote stripping, lowercase normalization, duplicate-key resolution, and `<unknown>` emission for missing fields
  - Out: family-table matching, selected-manager reporting, warning text, and explicit override handling
- **Acceptance criteria**:
  - the parser reads only `ID` and `ID_LIKE`
  - blank lines and comment lines are ignored
  - duplicate keys resolve by last well-formed assignment wins
  - quoted values normalize correctly without evaluating escapes, variable expansions, command substitutions, or backticks
  - missing keys emit `<unknown>` for the corresponding normalized field
- **Dependencies**:
  - `S1`
  - `S2`
  - `scripts/substrate/install-substrate.sh`
- **Verification**:
  - fixture matrix covers comments, duplicate assignments, mixed case, quoted values, and partial-key presence
  - pass condition: repeated runs over the same selected input yield identical normalized fields and no shell execution path exists
- **Rollout/safety**:
  - this is the core security boundary for the feature
  - deterministic field emission prevents downstream seams from inventing recovery behavior
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` active seam focus

#### S3.T1 - Implement the safe `ID` / `ID_LIKE` parser

- **Outcome**: the installer extracts only the two allowed keys with deterministic normalization and no shell execution behavior.
- **Inputs/outputs**:
  - Inputs: selected os-release content or unavailable-input state
  - Outputs: normalized `distro_id` and `distro_id_like`
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-01`
- **Implementation notes**:
  - parse line-by-line, split on the first `=`, trim, unquote once, lowercase, and ignore unsupported keys

#### S3.T2 - Expose the parser handoff boundary for later selection/reporting seams

- **Outcome**: later seams consume one stable normalized-field interface instead of peeking back into raw os-release input.
- **Inputs/outputs**:
  - Inputs: parser outputs and unavailable-input sentinel state
  - Outputs: internal variables or helper outputs consumed by later mapping and reporting work
- **Thread/contract refs**:
  - `THR-01`, `THR-07`
  - `C-01`, `C-02`
- **Implementation notes**:
  - do not embed family mapping or decision-line logic in this slice; the output is normalized parser truth only
