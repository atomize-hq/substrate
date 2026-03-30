---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - upstream detection contract changes field names, selected-manager vocabulary, or `<unknown>` semantics before closeout
    - additive-merge assumptions for legacy or unknown keys change before closeout
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
  - THR-03
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Persisted schema and additive merge contract

- **User/system value**: downstream writer and conformance work inherit one exact persisted payload contract instead of inferring field names, merge behavior, or sentinel handling from scattered source docs.
- **Scope (in/out)**:
  - In: exact `host_state.platform.*` nesting, required fields, `schema_version = 1` invariants, additive compatibility, preservation of legacy and unknown keys, and a contract-readiness verification matrix
  - Out: successful-Linux branch selection, temp-file replace mechanics, warning-only runtime behavior, and doc wording rewrites
- **Acceptance criteria**:
  - the slice freezes one exact `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` schema for `C-01`
  - the slice states that `pkg_manager.selected` and `pkg_manager.source` are copied verbatim from the upstream detection contract and are never re-derived locally
  - the slice states that `host_state.group`, `host_state.linger`, and unknown keys survive rewrites unchanged while `schema_version` remains integer `1`
  - the verification checklist names the exact repo surfaces and pass/fail conditions needed for `SEAM-1` to later pass `gates.pre_exec.contract`
- **Dependencies**:
  - `../../seam-1-persisted-platform-metadata-contract.md`
  - `../../threading.md`
  - `review.md`
  - `../../../persist-detected-linux-distro-pkg-manager/contract.md`
  - `../../../persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - `../../../persist-detected-linux-distro-pkg-manager/decision_register.md`
- **Verification**:
  - pass condition: no field path, merge rule, or upstream vocabulary boundary needed by `C-01` remains implicit or delegated to downstream seams
  - planned evidence cross-checks `install-state-schema-spec.md`, `contract.md`, and `tests/installers/install_state_smoke.sh` for one consistent schema story
- **Rollout/safety**:
  - prevents `SEAM-2` from implementing a writer against ambiguous field names or local package-manager semantics
  - keeps runtime reliability and failure behavior explicitly outside this seam
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R4

## Subslice decomposition disposition

- This slice remains crisp at the current execution grade, so no authoritative standalone sub-slices are needed after the basis refresh to `current`.
- `candidate_subslices` remains empty because this slice publishes the authoritative shared contract `C-01`; splitting the contract-definition work would not reduce ambiguity further.
- Revisit authoritative sub-slicing only if later closeout preparation materially expands the scope beyond the current contract freeze and verification matrix.

For a contract-definition slice that produces an owned contract:

- make the contract rules concrete enough that the producer seam can later satisfy `gates.pre_exec.contract`
- include a narrow verification plan with test locations, edge cases, and pass/fail conditions
- do not require the final accepted contract artifact to exist before the producer seam can become `exec-ready`

#### S1.T1 - Freeze the exact persisted platform schema for `C-01`

- **Outcome**: `SEAM-1` publishes one exact nested payload contract for Linux platform metadata under `install_state.json`.
- **Inputs/outputs**:
  - Inputs: source `contract.md`, `install-state-schema-spec.md`, `decision_register.md`
  - Outputs: seam-local schema bullets and examples that later execution can implement without reinterpretation
- **Thread/contract refs**:
  - `THR-01`, `THR-03`
  - `C-01`
- **Implementation notes**:
  - keep the scope limited to payload truth and additive compatibility; do not fold runtime write-branch behavior into this slice
- **Acceptance criteria**:
  - exact field paths, required-field rules, and `<unknown>` copy-through behavior are explicit
  - `pkg_manager.selected` and `pkg_manager.source` remain externally owned strings
- **Test notes**:
  - future execution must point at `tests/installers/install_state_smoke.sh` and installer-script fixtures for schema verification
- **Risk/rollback notes**:
  - any late field rename or new local vocabulary rule would invalidate both downstream seams and requires revalidation

#### S1.T2 - Define the additive-merge verification matrix

- **Outcome**: later execution inherits a narrow checklist for preserving legacy state while writing the new platform block.
- **Inputs/outputs**:
  - Inputs: existing `install_state.json` compatibility rules from source docs and current smoke coverage shape
  - Outputs: pass/fail matrix for legacy-field preservation, unknown-key preservation, and fresh-file creation
- **Thread/contract refs**:
  - `THR-01`, `THR-03`
  - `C-01`
- **Implementation notes**:
  - verification remains about payload shape and merge expectations, not atomicity or warning-only runtime failure behavior
- **Acceptance criteria**:
  - the matrix names exact evidence surfaces for preserved legacy fields and additive compatibility
  - the slice makes clear which runtime behaviors stay owned by `SEAM-2`
- **Test notes**:
  - use current `tests/installers/install_state_smoke.sh` scenarios as a baseline, then extend them downstream for platform fields
- **Risk/rollback notes**:
  - if the existing file-upgrade story changes, this slice becomes stale and must not be used for exec-ready promotion

## Contract freeze for `C-01`

- `schema_version` remains the integer `1`.
- The new platform block lives only under `host_state.platform`.
- `host_state.platform` contains both `os_release` and `pkg_manager` when present.
- `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` copy the detector's normalized outputs verbatim, including the literal `<unknown>` sentinel when the detector emitted it.
- `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` copy the external detection contract's emitted strings verbatim and must not be re-derived or renamed by the persistence writer.
- Existing `host_state.group` and `host_state.linger` JSON values remain preserved verbatim.
- Unknown keys remain preserved verbatim unless another authoritative spec for that key says otherwise.
- This slice does not redefine temp-file replacement, invalid-file fallback, or warning-only failure posture; those runtime behaviors remain downstream writer scope.

## Verification checklist for `C-01` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Exact field-path cross-check | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | One exact nested path exists for each required field and no alternate field names remain in seam-local planning. |
| Upstream vocabulary ownership | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` plus `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` | `pkg_manager.selected` and `pkg_manager.source` are treated as copied-through outputs, not local enums with independent authority. |
| Additive compatibility | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` and `tests/installers/install_state_smoke.sh` | Existing `host_state.group`, `host_state.linger`, and unknown keys remain preserved across rewrites. |
| Missing os-release sentinel handling | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | `<unknown>` persistence is explicit for both `os_release.id` and `os_release.id_like`; no local fallback distro value is introduced. |
| Fresh-file shape | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | A newly created Linux metadata file still stays on `schema_version = 1` and contains only the additive platform block plus any existing legacy host-state content. |

Contract-readiness for this slice is documentary: `gates.pre_exec.contract` can pass only when the payload contract and verification matrix are explicit enough for downstream implementation without reopening field ownership or merge semantics.
