---
seam_id: SEAM-3
seam_slug: cross-surface-parity-and-drift-guards
type: conformance
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 changes replay-routing or tracing-validation contracts after closeout
    - SEAM-2 changes abnormal-terminal-loss wording, exit status, or regression harness assumptions after closeout
    - WPEP playbook, REPLAY docs, TRACE docs, or USAGE docs drift before the conformance seam revalidates them
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
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-3 - Cross-surface parity and drift guards

- **Goal / value**:
  Turn the landed contracts from the first two seams into durable operator guidance and regression guards so the same ambiguity does not recur through docs or smoke drift.
- **Scope**
  - In:
    - update and lock the authoritative playbook and smoke expectations in the active WPEP pack
    - align `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, and any touched contract docs to the landed runtime behavior
    - add or adjust regression coverage that proves replay routing parity and REPL abnormal-exit behavior stay aligned with docs
    - capture downstream stale triggers if the landed reality differs from the extracted review surfaces
  - Out:
    - new runtime behavior beyond what `SEAM-1` and `SEAM-2` already published
    - opportunistic cleanup outside replay, tracing, and REPL conformance boundaries
- **Primary interfaces**
  - Inputs:
    - published replay-routing and tracing-validation contracts from `SEAM-1`
    - published interactive terminal-loss contract from `SEAM-2`
    - landed smoke and regression surfaces across replay, trace, and REPL docs
  - Outputs:
    - cross-surface docs lock-in
    - regression and smoke drift guards
    - downstream stale-trigger record if plan and landed reality diverge
- **Key invariants / rules**:
  - pack-level review surfaces remain orientation only; seam-local review is still required before execution
  - conformance must consume landed truth rather than re-decide runtime behavior
  - docs and smoke scripts must assert the same contract that runtime tests pin
  - this seam is not a cleanup bucket for unfinished runtime work from upstream seams
- **Dependencies**
  - Direct blockers:
    - none once `SEAM-1` and `SEAM-2` closeouts remain current
  - Transitive blockers:
    - any late ADR or decision-record change that modifies the published contracts from the first two seams after this seam decomposes
    - platform-specific verification gaps reopened by later runtime or harness drift
  - Direct consumers:
    - none inside this pack
  - Derived consumers:
    - future maintainers using replay, tracing, or REPL docs and smoke evidence as authoritative guidance
- **Touch surface**:
  `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`, `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`, `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, and the regression suites that pin replay-routing and REPL abnormal-terminal-loss behavior.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove every doc or smoke assertion maps directly to a landed upstream contract, and that any remaining drift is captured as an explicit stale trigger or remediation rather than left implicit.
- **Risks / unknowns**:
  - Risk: upstream seams land with partial docs/test updates, forcing conformance to absorb unfinished runtime cleanup.
  - De-risk plan: keep seam-local review tied to the published closeouts for `THR-01` and `THR-02`, and refresh stale triggers whenever the upstream runtime or validation surfaces drift.
  - Risk: the active WPEP pack changes in parallel and invalidates the extracted basis.
  - De-risk plan: treat the basis as current at promotion time, then reopen revalidation only if the WPEP pack or upstream closeouts drift again.
- **Rollout / safety**:
  This seam should only lock and verify previously landed behavior. It reduces operator confusion and future regression risk without expanding product scope.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` only because it has landed and left the forward planning window after closing the final conformance pass for `THR-01` and `THR-02`.
  - Which threads matter most: `THR-01`, `THR-02`
  - What the first seam-local review should focus on: mapping each smoke or doc assertion back to landed upstream evidence and rejecting any attempt to use conformance to finish missing runtime work
- **Expected seam-exit concerns**:
  - Contracts likely to publish: none beyond conformance evidence and stale-trigger records
  - Threads likely to advance: `THR-01`, `THR-02`
  - Review-surface areas likely to shift after landing: pack-level workflow diagrams versus the final landed docs and smoke topology
  - Downstream seams most likely to require revalidation: future replay, tracing, or shell-resilience follow-on packs
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
