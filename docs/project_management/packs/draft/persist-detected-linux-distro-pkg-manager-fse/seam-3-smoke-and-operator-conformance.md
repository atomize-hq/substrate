---
seam_id: SEAM-3
seam_slug: smoke-and-operator-conformance
type: conformance
status: proposed
execution_horizon: future
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-02
    - THR-03
  stale_triggers:
    - any landed SEAM-1 closeout changes field names path wording or compatibility rules
    - any landed SEAM-2 closeout changes write matrix temp-file semantics or warning-only behavior
    - smoke harness scenarios or CI checkpoint surfaces move before this seam enters the horizon
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-3 - Smoke and operator conformance

- **Goal / value**:
  - Lock drift guards, operator wording, and checkpoint evidence around the landed persistence behavior so the feature remains provable and supportable after the runtime seams complete.
- **Scope**
  - In:
    - Linux smoke assertions for no-event success, exact persisted platform fields, missing os-release degradation, and additive compatibility
    - `docs/INSTALLATION.md` wording for canonical path, shared hosted-plus-dev producer scope, `schema_version = 1`, and the four persisted platform fields
    - Validation command and evidence expectations carried from the old `plan.md`
    - Checkpoint-ready evidence for Linux behavior smoke plus cross-platform compile/test parity
    - Session-log and pack-closeout surfaces that summarize the resulting evidence
  - Out:
    - Changing runtime writer mechanics or field ownership
    - Expanding non-Linux runtime behavior for this feature
    - Adding a manual validation playbook
    - Solving unrelated world-deps or helper-staging changes in shared files
- **Primary interfaces**
  - Inputs:
    - `C-01` and `C-02` from `SEAM-1`
    - `C-03` and `C-04` from `SEAM-2`
    - Existing smoke harness and operator documentation surfaces
  - Outputs:
    - `C-05` smoke evidence contract
    - `C-06` operator wording contract
    - Final pack evidence posture for closeout
- **Key invariants / rules**:
  - Smoke coverage remains Linux behavior only
  - Cross-platform parity is compile/test evidence rather than non-Linux runtime persistence
  - Documentation must not drift from the actual canonical path, field names, or write/no-write matrix
  - No separate manual playbook is required if the smoke harness covers the accepted scenarios explicitly
  - `CP1` remains evidence inside this seam rather than a new seam or alternate control plane
- **Dependencies**
  - Direct blockers:
    - `THR-02` must publish landed writer truth from `SEAM-2`
    - `THR-03` must publish landed schema/path truth from `SEAM-1`
  - Transitive blockers:
    - Shared-file conflicts in `docs/INSTALLATION.md`, `plan.md`, `tasks.json`, and `tests/installers/install_state_smoke.sh`
  - Direct consumers:
    - none inside this pack
  - Derived consumers:
    - Operators and support maintainers
    - Future maintainers relying on drift guards and pack closeout evidence
- **Touch surface**:
  - `tests/installers/install_state_smoke.sh`
  - `docs/INSTALLATION.md`
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
- **Verification**:
  - Because this seam **consumes** upstream contracts, verification may depend on accepted upstream evidence from both `SEAM-1` and `SEAM-2`.
  - The first seam-local review should try to falsify:
    - whether the smoke harness still misses any of the four required Linux branches
    - whether docs still use stale wording for canonical path or schema naming
    - whether checkpoint evidence is still attached to the same contract the smoke harness is proving
    - whether cross-platform parity is being overstated as behavior rather than compile/test evidence
  - A passing pre-exec posture should leave pack closeout able to rely on concrete evidence instead of source-plan intention.
- **Risks / unknowns**:
  - Risk:
    - `docs/INSTALLATION.md` already had wording drift in the source pack.
  - De-risk plan:
    - Carry it explicitly as `REM-002` so operator wording is not treated as an optional polish pass.
  - Risk:
    - Shared-file sequencing with adjacent packs can stale the smoke harness or docs between extraction and execution.
  - De-risk plan:
    - Revalidate both threads before this seam enters the horizon.
  - Risk:
    - Conformance work can accidentally become a cleanup bucket for unresolved runtime behavior.
  - De-risk plan:
    - Keep this seam strictly downstream of landed writer truth and reject net-new runtime behavior in seam-local planning.
- **Rollout / safety**:
  - This seam should add evidence and documentation accuracy, not new runtime surface area.
  - It is the pack's main drift-guard seam and should capture any delta between planned and landed behavior before pack closeout.
- **Downstream decomposition context**:
  - This seam is `future` because the source pack positioned smoke/docs/CP1 work after contract and runtime semantics were already stable.
  - The most important threads are `THR-02` and `THR-03`.
  - The first seam-local review should focus on exact smoke scenario coverage, documentation wording accuracy, and whether checkpoint evidence still matches the current contract.
  - Source-plan lineage: primarily `PDLDPM2` plus the old `plan.md`, `tasks.json`, and checkpoint plan surfaces.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-05`
    - `C-06`
  - Threads likely to advance:
    - `THR-02` from `published` to `revalidated` and then toward `closed`
    - `THR-03` from `published` to `revalidated` and then toward `closed`
  - Review-surface areas likely to shift after landing:
    - final validation and checkpoint evidence flow
    - operator-facing wording about the canonical path and producer scope
  - Downstream seams most likely to require revalidation:
    - none inside this pack, but pack closeout will need to evaluate downstream stale triggers and carried-forward follow-ups
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
