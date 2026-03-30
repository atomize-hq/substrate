---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - published `C-01` or `C-02` changes the canonical file path or payload contract before closeout
    - shared installer-branch structure changes before landing
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
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Successful-Linux write matrix and no-write boundaries

- **User/system value**: one explicit producer matrix keeps hosted and dev installers aligned and prevents downstream smoke or docs work from guessing which successful Linux runs write canonical state.
- **Scope (in/out)**:
  - In: hosted install hosted `--no-world` dev install and dev `--no-world` successful-Linux writes, hosted `--dry-run` no-write behavior, non-Linux no-write behavior, and create/update expectations when no prior file exists
  - Out: temp-file reliability details, warning-only failure wording, smoke assertions, and operator-doc rewrites
- **Acceptance criteria**:
  - successful Linux hosted and dev flows create or update `<effective_prefix>/install_state.json` even when no group or linger events exist
  - hosted `--dry-run` creates neither the canonical file nor the temp file nor a metadata-only parent directory
  - non-Linux flows do not gain new `host_state.platform.*` writes
  - the slice states that the writer consumes `C-01` and `C-02` without reopening payload or canonical-path ownership
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `review.md`
  - `../../../persist-detected-linux-distro-pkg-manager/contract.md`
  - `../../../persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
- **Verification**:
  - pass condition: no successful Linux branch needed by the feature can still skip canonical-file creation because of event-only metadata writes
  - planned evidence cross-checks the hosted and dev installers plus the current smoke harness coverage shape
- **Rollout/safety**:
  - prevents downstream work from encoding split writer rules across installer variants
  - keeps payload ownership and failure-posture work in their proper slices
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R3

#### S1.T1 - Remove event-only write gating from successful Linux flows

- **Outcome**: successful Linux producer paths no longer depend on incidental group or linger events to create or update canonical metadata.
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - hosted and dev installers both attempt canonical metadata persistence after successful Linux install completion
  - no-event success still writes or updates the canonical file

#### S1.T2 - Freeze the no-write matrix

- **Outcome**: dry-run and non-Linux boundaries are explicit and testable before downstream conformance work starts.
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Acceptance criteria**:
  - hosted `--dry-run` remains no-write
  - non-Linux flows remain no-write for `host_state.platform.*`

## Contract freeze for `C-03`

- Hosted install success writes the canonical file.
- Hosted install with `--no-world` success writes the canonical file.
- Dev install success writes the canonical file.
- Dev install with `--no-world` success writes the canonical file.
- Hosted `--dry-run` does not create the canonical file its temp file or a metadata-only parent directory.
- Non-Linux runs do not add new `host_state.platform.*` writes.
- Canonical-file creation and update do not depend on group or linger events being present.

## Verification checklist for `C-03` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Hosted successful-Linux matrix | `scripts/substrate/install-substrate.sh` | successful Linux branches always reach canonical metadata write setup even when no host-state events exist. |
| Dev successful-Linux matrix | `scripts/substrate/dev-install-substrate.sh` | dev install keeps the same producer rule as hosted install. |
| Dry-run boundary | `scripts/substrate/install-substrate.sh` | hosted `--dry-run` remains no-write for canonical metadata and temp files. |
| Non-Linux boundary | hosted and dev installer platform gates | non-Linux flows do not gain new `host_state.platform.*` writes. |

Contract-readiness for this slice is documentary: the matrix is explicit enough that implementation can land one stable writer rule before `SEAM-3` adds smoke assertions.
