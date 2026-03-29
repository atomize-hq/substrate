---
seam_id: SEAM-01
seam_slug: os-release-input-parser
status: landed
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-01-os-release-input-parser.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - os-release parsing rules change
    - alternate-input hook semantics change
    - `<unknown>` sentinel semantics change
    - downstream persistence pack diverges from inherited input truth
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---
# SEAM-01 - os-release Input And Parser Contract

## Seam Brief (Restated)

- **Goal / value**: establish one trustworthy Linux input-selection and parser contract so every later package-manager decision stage starts from deterministic `distro_id`, `distro_id_like`, and `<unknown>` semantics.
- **Type**: domain
- **Scope**
  - In:
    - selected-input resolution for `/etc/os-release` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
    - absolute-path, readable-regular-file, and no-fallback semantics for the alternate-input hook
    - line-oriented extraction of `ID` and `ID_LIKE`
    - normalization, quote stripping, duplicate-key handling, and comment handling
    - `<unknown>` sentinel behavior for missing, unreadable, or unusable selected input
    - handoff of normalized fields to later selection/reporting seams without executing os-release content
  - Out:
    - distro-family mapping and availability-based manager selection
    - decision-line wording and suppression rules
    - explicit override precedence and failure taxonomy
    - wrapper/doc propagation and validation topology ownership
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- **Verification**:
  - execution starts only after the seam-local artifacts make `C-01` and `C-02` concrete enough to implement without reopening parser or hook semantics
  - selected-input resolution must prove unset or empty uses `/etc/os-release`, a valid alternate file replaces `/etc/os-release`, and an invalid or unreadable alternate path yields `<unknown>` without fallback
  - parser verification must cover comment lines, duplicate assignments, surrounding quotes, lowercase normalization, missing keys, and the rule that only `ID` and `ID_LIKE` are read
  - final accepted or published contract artifacts are reserved for seam-exit evidence and closeout, not pre-exec readiness
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: none
  - Required threads: none; this seam is the producer for `THR-01` and `THR-07`
  - Stale triggers:
    - os-release parsing rules change
    - alternate-input hook semantics change
    - `<unknown>` sentinel semantics change
    - downstream persistence pack diverges from inherited input truth
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams:
    - `SEAM-02`
    - `SEAM-03`
    - `SEAM-04`
    - `SEAM-06`
    - downstream pack `persist-detected-linux-distro-pkg-manager`
  - Contracts produced:
    - `C-01`
    - `C-02`
  - Contracts consumed: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4` / `slice-4-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: every later seam and the downstream persistence pack consume parser/input truth, so promotion cannot rely on inferred implementation behavior or unpublished parser semantics.
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01` to `published`
  - `THR-07` to `published`
- **Likely downstream stale triggers**:
  - parser normalization rules change
  - alternate-input path validation or no-fallback semantics change
  - `<unknown>` emission rules change
- **Expected closeout evidence**:
  - landed installer evidence for selected-input resolution and degradation behavior
  - landed evidence that only `ID` and `ID_LIKE` are parsed without shell execution
  - contract publication accounting for `C-01` and `C-02`
  - downstream stale-trigger emission for `SEAM-02`, `SEAM-03`, `SEAM-04`, `SEAM-06`, and the persistence pack if parser/input semantics shifted during landing

## Slice index

- `S1` -> `slice-1-parser-input-contract-definition.md`
- `S2` -> `slice-2-selected-input-resolution.md`
- `S3` -> `slice-3-safe-parser-normalized-fields.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-01-closeout.md`
