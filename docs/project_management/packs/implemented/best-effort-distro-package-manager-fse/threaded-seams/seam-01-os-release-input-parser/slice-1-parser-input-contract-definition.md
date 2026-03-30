---
slice_id: S1
seam_id: SEAM-01
slice_kind: delivery
execution_horizon: active
status: landed
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
    contract: passed
    revalidation: inherited
  post_exec:
    landing: passed
    closeout: passed
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

## Contract freeze for `C-01` and `C-02`

### Selected-input resolution contract (`C-02`)

- Unset or empty `SUBSTRATE_INSTALL_OS_RELEASE_PATH` selects `/etc/os-release`.
- A non-empty `SUBSTRATE_INSTALL_OS_RELEASE_PATH` value replaces `/etc/os-release` only when it is an absolute path to a readable regular file.
- A non-empty alternate path that is relative, missing, unreadable, not a regular file, or cannot be opened yields an unavailable-input state.
- Once `SUBSTRATE_INSTALL_OS_RELEASE_PATH` is non-empty, the installer does not fall back to `/etc/os-release`; invalid alternate-input state degrades to `<unknown>` instead.
- This hook changes only the os-release input source. It does not override explicit package-manager selectors and does not introduce platform behavior outside the Linux hosted-installer path.

### Safe parser contract (`C-01`)

- Parsing is line-oriented and never executes os-release content.
- Only the exact keys `ID` and `ID_LIKE` are read from the selected input.
- Blank lines are ignored.
- Lines whose first non-space character is `#` are ignored.
- Parsing splits on the first `=` only.
- Leading and trailing ASCII whitespace around the raw value is trimmed.
- One surrounding pair of matching single quotes or double quotes is stripped when present.
- Escapes, variable expansion, command substitution, and backticks are never evaluated.
- Parsed values normalize to ASCII lowercase before handoff.
- If `ID` or `ID_LIKE` appears more than once, the last well-formed assignment wins.
- Missing `ID` yields `distro_id=<unknown>`.
- Missing `ID_LIKE` yields `distro_id_like=<unknown>`.

### Handoff boundary

- `SEAM-01` emits only normalized parser/input truth: `distro_id` and `distro_id_like`.
- Unavailable input and missing accepted keys both use the literal sentinel `<unknown>`.
- `SEAM-01` does not own distro-family mapping, stable decision-line wording, warning text, explicit override precedence, or fallback failure taxonomy.

## Verification checklist for contract readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Source-authority cross-check for selected-input and parser rules | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` | Every rule needed by `C-01` and `C-02` is explicit in seam-local docs and matches accepted source authority. |
| Default-input behavior | `scripts/substrate/install-substrate.sh` plus future `tests/installers/pkg_manager_detection_smoke.sh` coverage | Unset or empty hook selects `/etc/os-release` and no alternate-input branch is consulted. |
| Valid alternate input | `scripts/substrate/install-substrate.sh` plus future `tests/installers/pkg_manager_detection_smoke.sh` coverage | An absolute readable regular-file alternate path replaces `/etc/os-release` and becomes the only parser input. |
| Invalid alternate input | `scripts/substrate/install-substrate.sh` plus future `tests/installers/pkg_manager_detection_smoke.sh` coverage | Relative, unreadable, missing, non-regular, or otherwise unusable alternate input degrades to `<unknown>` without reading `/etc/os-release` as fallback. |
| Parser normalization matrix | `scripts/substrate/install-substrate.sh` plus future `tests/installers/pkg_manager_detection_smoke.sh` coverage | Comment-bearing inputs, duplicate assignments, quoted values, mixed case, and missing keys all resolve to the contract-owned normalized outputs without shell execution. |
| Non-authoritative existing smoke evidence | `tests/installers/pkg_manager_container_smoke.sh` | Existing container smoke is recognized as insufficient for `C-01`/`C-02` because it sources `/etc/os-release` directly and does not exercise the alternate-input hook. |

Contract-readiness for this slice is documentary: `gates.pre_exec.contract` passes when the seam-local rules and verification matrix are explicit enough for `S2` and `S3` to implement without reopening parser or hook semantics.
