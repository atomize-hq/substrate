---
seam_id: SEAM-2
seam_slug: replay-attribution-runtime-surfaces
type: capability
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 changes the classifier field set or layer vocabulary
    - reason-fragment or recorded-host formatting rules change
    - telemetry field names, enum values, or omission rules change
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

# SEAM-2 - Replay attribution runtime surfaces

- **Goal / value**:
  - Publish effective-disable attribution in replay origin summaries, host warnings, and `replay_strategy` so operators and trace consumers see the same winning-layer story.
  - Preserve all existing replay-local opt-out fragments and selection behavior while making effective host fallback intelligible.
- **Scope**
  - In:
    - origin-summary wiring for effective-disable attribution
    - host-warning wiring for effective-disable attribution
    - recorded-host format lock: `host (recorded; <reason>)`
    - additive `replay_strategy` provenance fields and `origin_reason_code` extension values
    - omission rules so replay-local opt-out cases do not emit `world_disable_source`
  - Out:
    - new replay subcommands or JSON envelopes
    - helper placement changes beyond the `SEAM-1` contract
    - smoke-wrapper and manual-playbook authoring
    - broad operator-doc lock-in outside the later conformance seam
- **Primary interfaces**
  - Inputs:
    - `C-01` classifier result from `SEAM-1`
    - `C-02` provenance/redaction contract from `SEAM-1`
    - exact replay fragments and telemetry fields already locked by the source pack
  - Outputs:
    - `[replay] origin: ...` attribution output
    - `[replay] warn: running on host (...)` attribution output
    - `replay_strategy.origin_reason_code`
    - optional `replay_strategy.world_disable_source`
- **Key invariants / rules**:
  - replay-local opt-out fragments remain unchanged
  - effective-disable attribution uses the exact fragments locked by the source contract
  - the recorded-host case prints `host (recorded; <reason>)` exactly
  - `world_disable_source` emits only for effective-disable attribution
  - existing `replay_strategy` fields remain stable and telemetry changes remain additive only
  - replay does not add extra replay lines outside the existing origin summary and host warning
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
    - `THR-01`
    - `THR-02`
  - Transitive blockers:
    - external semantic anchors inherited through `SEAM-1`
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - replay operators
    - trace consumers
    - docs examples
- **Touch surface**:
  - `crates/shell/src/execution/routing/replay.rs`
  - `crates/replay/src/replay/executor.rs`
  - `crates/shell/tests/replay_world.rs`
  - trace examples that later conformance work will lock in
- **Verification**:
  - This seam consumes upstream contracts from `SEAM-1`, so seam-local verification may depend on accepted upstream contract evidence for `C-01` and `C-02`.
  - Later seam-local verification should prove:
    - origin summaries use the exact effective-disable fragments
    - host warnings use the same fragment as the origin summary
    - recorded-host output uses the exact `host (recorded; <reason>)` punctuation
    - `origin_reason_code` uses the expected `world_disabled_*` values
    - `world_disable_source` emits only for effective-disable cases and remains redacted
- **Risks / unknowns**:
  - Risk:
    - runtime copy and telemetry can drift because both are wired in high-churn replay paths
  - De-risk plan:
    - tie implementation to `contract.md` and `telemetry-spec.md` as single-source inputs and assert exact-string plus exact-field tests
  - Risk:
    - omission rules may regress and emit `world_disable_source` for replay-local opt-out cases
  - De-risk plan:
    - add explicit negative coverage for replay-local opt-out paths
  - Risk:
    - a narrow helper contract may still prove insufficient once wired into both stderr and trace output
  - De-risk plan:
    - keep `SEAM-2` provisional until `SEAM-1` publishes evidence and revalidate before promotion
- **Rollout / safety**:
  - Runtime behavior remains bounded to existing replay surfaces.
  - Safety comes from exact-string contracts, additive-only telemetry, and the promise not to change replay selection, backend, timeout, or exit-code behavior.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because it is second on the critical path and should follow a published `SEAM-1` contract rather than re-infer foundation semantics.
  - Which threads matter most
    - `THR-03`
    - `THR-04`
    - plus upstream dependency threads `THR-01` and `THR-02`
  - What the first seam-local review should focus on
    - exact copy fragments
    - recorded-host punctuation
    - telemetry object gating
    - omission rules for replay-local opt-out cases
    - evidence that no additional replay lines or secret-bearing fields were introduced
  - Provisional-deeper-planning note
    - if seam-local work begins before `SEAM-1` publishes, deeper planning should stay provisional and spike-grade only.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-03`
    - `C-04`
  - Threads likely to advance:
    - `THR-03` from `defined` to `published`
    - `THR-04` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - replay origin-summary flow
    - host-warning flow
    - `replay_strategy` event shape
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
- **Source pack anchors**:
  - `contract.md`
  - `telemetry-spec.md`
  - `decision_register.md` DR-0002 and DR-0003
  - `slices/WDRA1/WDRA1-spec.md`
