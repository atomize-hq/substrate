---
seam_id: SEAM-2
seam_slug: replay-attribution-runtime-surfaces
type: capability
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 changes the classifier field set or layer vocabulary
    - reason-fragment or recorded-host formatting rules change
    - telemetry field names, enum values, or omission rules change
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
  planned_location: reserved_final_slice
  status: passed
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
    - bind `SEAM-2` directly to the published `SEAM-1` closeout and revalidate again before landing if the helper/result contract drifts
- **Rollout / safety**:
  - Runtime behavior remains bounded to existing replay surfaces.
  - Safety comes from exact-string contracts, additive-only telemetry, and the promise not to change replay selection, backend, timeout, or exit-code behavior.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because `governance/seam-1-closeout.md` now publishes the upstream `C-01` / `C-02` handoff this seam consumes.
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
  - Active-planning note
    - seam-local planning now binds directly to the published `SEAM-1` closeout and should treat later helper/result drift as a stale-trigger revalidation event, not as a missing-input blocker.
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
