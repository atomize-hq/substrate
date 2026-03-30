---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR-0032 or related docs reintroduce competing feature-directory authority before closeout
    - canonical-path wording or alias semantics change before closeout
    - upstream detection ownership boundary for selected or source values changes before closeout
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
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S2 - Canonical path and authority boundary

- **User/system value**: runtime and conformance seams inherit one canonical file-location rule and one ownership boundary, so they do not need to reconcile stale ADR paths, dual metadata locations, or local package-manager semantics.
- **Scope (in/out)**:
  - In: canonical on-disk file path, operator-facing alias relationship, one-file-only rule, upstream authority boundary for copied-through detection outputs, and repo-surface checks that downstream work must honor
  - Out: runtime write triggers, temp-file replace execution, smoke evidence, and operator doc rewrites themselves
- **Acceptance criteria**:
  - the slice freezes `<effective_prefix>/install_state.json` as the only canonical on-disk file and `$SUBSTRATE_HOME/install_state.json` as the default-prefix alias, not a second metadata location
  - the slice states that this feature introduces no new env var, no second metadata file, and no local vocabulary authority for `pkg_manager.selected` or `pkg_manager.source`
  - the slice records the accepted authority-path override (`contract.md` + `DR-0005`) and the exact repo surfaces that downstream work must continue to treat as canonical
  - the verification checklist makes downstream writer and doc seams consume one exact path rule
- **Dependencies**:
  - `S1`
  - `../../threading.md`
  - `review.md`
  - `../../../persist-detected-linux-distro-pkg-manager/contract.md`
  - `../../../persist-detected-linux-distro-pkg-manager/decision_register.md`
  - `../../governance/remediation-log.md`
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- **Verification**:
  - pass condition: downstream seams can point to one canonical file rule, one authority boundary, and one feature directory without manual reconciliation during execution
  - planned evidence cross-checks ADR, contract docs, installer path assignments, and `docs/INSTALLATION.md` wording targets
- **Rollout/safety**:
  - prevents future code and docs from diverging across multiple path stories
  - keeps upstream detection semantics externally owned instead of letting this seam create a second vocabulary authority
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R3

#### S2.T1 - Freeze the canonical path and alias contract for `C-02`

- **Outcome**: `SEAM-1` publishes one file-path rule that both installers and later docs must follow.
- **Inputs/outputs**:
  - Inputs: source `contract.md`, `decision_register.md`, ADR-0032, current installer path assignments
  - Outputs: seam-local path and alias bullets plus a reconciliation checklist for stale path references
- **Thread/contract refs**:
  - `THR-01`, `THR-03`
  - `C-02`
- **Implementation notes**:
  - make the default-prefix alias explicit without promoting it to a second canonical file
- **Acceptance criteria**:
  - no seam-local artifact implies both `<effective_prefix>/install_state.json` and another independent metadata file
  - the path story holds for both hosted and dev installers
- **Test notes**:
  - future execution must inspect both installer scripts and `docs/INSTALLATION.md`
- **Risk/rollback notes**:
  - leaving dual-authority wording unresolved blocks downstream promotion because it makes runtime and docs work ambiguous

#### S2.T2 - Publish the authority-boundary checklist for downstream seams

- **Outcome**: `SEAM-2` and `SEAM-3` receive one explicit boundary map for what stays owned by upstream detection, what stays owned by the writer seam, and what `SEAM-1` must publish.
- **Inputs/outputs**:
  - Inputs: `threading.md`, source contract docs, current installer and doc surfaces, `REM-001`
  - Outputs: seam-local ownership bullets and a revalidation checklist for downstream consumers
- **Thread/contract refs**:
  - `THR-01`, `THR-03`
  - `C-01`, `C-02`
- **Implementation notes**:
  - keep smoke and doc execution work out of this slice; this slice only defines the handoff boundary
- **Acceptance criteria**:
  - downstream seams know which unresolved blocker belongs to `SEAM-1` and which doc drift is already tracked as `REM-002`
  - no local contract text redefines supported package-manager spellings or source vocabulary
- **Test notes**:
  - future closeout must show that outbound thread publication reflects this authority map
- **Risk/rollback notes**:
  - if the authority map changes during execution, `THR-01` and `THR-03` must be marked stale for downstream revalidation

## Authority freeze for `C-02`

- The only canonical on-disk file for this feature is `<effective_prefix>/install_state.json`.
- `$SUBSTRATE_HOME/install_state.json` is an operator-facing alias for the same path only when the effective prefix is the default user-scoped install root.
- This feature does not create a second metadata file, a feature-local config file, or a new env-var override.
- The hosted and dev installer paths must converge on the same effective-prefix semantics even if their string interpolation differs (`${PREFIX}/install_state.json` versus `${PREFIX%/}/install_state.json`).
- Upstream distro and package-manager detection remain externally owned; the persistence writer copies emitted strings but does not restate or refine their meaning.
- `install_state.json` remains the only persisted metadata file touched by this feature.

## Verification checklist for `C-02` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Feature-directory authority | `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `../../governance/remediation-log.md` | Either ADR-0032 is reconciled to the resolved pack path or an explicit override is recorded strongly enough for `REM-001` to close. |
| Canonical path convergence | `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | Both installers and the contract point to the same effective-prefix `install_state.json` location. |
| Alias discipline | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/INSTALLATION.md` | `$SUBSTRATE_HOME/install_state.json` is described only as the default-prefix alias, not as a separate second path. |
| Upstream vocabulary ownership | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` | Supported manager spellings and `pkg_manager.source` values stay externally owned and are not redefined locally. |

Contract-readiness for this slice is documentary: `SEAM-1` can move toward `exec-ready` only when the path rule, alias discipline, and authority boundary are explicit enough that downstream seams no longer need to reconcile stale ADR or docs inputs by hand.
