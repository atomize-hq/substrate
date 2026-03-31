---
seam_id: SEAM-2
seam_slug: shim-doctor-disabled-aware-reporting
type: capability
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - C-01 or THR-01 changes after SEAM-1 closeout
    - json-mode or attribution work reshapes shim payloads or field paths
    - hidden world backend or world-deps probe paths survive refactors in shim_doctor/report.rs
    - exact disabled-mode copy contract changes without synchronized test/doc updates
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-2 - Shim doctor disabled-aware reporting

- **Goal / value**:
  Make `substrate shim doctor` truthful for host-only installs by publishing explicit disabled/skipped behavior with no probes, while preserving fail-visible behavior when the world is enabled and broken.
- **Scope**
  - In:
    - Gate shim-doctor world/world-deps collection on `effective_world_enabled`
    - Short-circuit all world backend and world-deps probes when disabled
    - Emit `.world.status` and `.world_deps.status` with the source pack's additive enum contract
    - Omit legacy error/details/report fields in disabled mode where the source contract requires omission
    - Render the exact disabled-mode shim-doctor lines and suppress `Error:` for disabled/skipped states
  - Out:
    - Health summary derivation and docs alignment
    - Cross-platform smoke orchestration beyond seam-local proving
    - Why-disabled attribution beyond the base status contract
- **Primary interfaces**
  - Inputs:
    - `effective_world_enabled` from `SEAM-1`
    - world backend probe results when enabled
    - world-deps snapshot/apply probe results when enabled
    - shim-doctor output renderer and JSON serializer
  - Outputs:
    - canonical world/world-deps shim status contracts (`C-02`, `C-03`)
    - disabled-mode shim operator contract (`C-04`)
- **Key invariants / rules**:
  - Disabled mode must not spawn `substrate world doctor --json`
  - Disabled mode must not make world-agent socket/pipe calls for diagnostics
  - Disabled mode must not compute world-deps applied state
  - Disabled mode must publish `.world.status = "disabled"` and `.world_deps.status = "skipped_disabled"`
  - Disabled mode must omit probe-derived `world.error`, `world.details`, and `world_deps.report`
  - Enabled mode must never emit `disabled` or `skipped_disabled`
  - Status enums, not legacy booleans/strings alone, are the canonical machine-readable classifier
  - The exact disabled-mode text contract is:
    - `World backend:`
    - `  Status: disabled`
    - `  Reason: skipped because world diagnostics are disabled by effective config`
    - `World deps:`
    - `  Status: skipped (world disabled)`
    - `  Reason: skipped because world diagnostics are disabled by effective config`
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` / `THR-01`
  - Transitive blockers:
    - Any unresolved precedence ambiguity or exit-code drift in diagnostics routing
    - Adjacent JSON/copy work on the same shim-doctor surfaces
  - Direct consumers:
    - `SEAM-3`
    - `SEAM-4`
  - Derived consumers:
    - automation reading shim JSON
    - future attribution and json-envelope packs
- **Touch surface**:
  `crates/shell/src/builtins/shim_doctor/report.rs`, `crates/shell/src/builtins/shim_doctor/output.rs`, `crates/shell/tests/shim_doctor.rs`.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove disabled text lines, disabled JSON enum values, disabled-mode omission rules, and the no-probe boundary even when probe fixtures exist and would otherwise report failures.
- **Risks / unknowns**:
  - Risk: rendering could still branch on `.world.ok` and accidentally mark disabled state as healthy or unavailable.
  - De-risk plan: drive rendering from `.world.status` / `.world_deps.status` and assert the exact text lines in integration tests.
  - Risk: omission rules may be implemented partially, leaving one legacy error surface behind and reintroducing false negatives downstream.
  - De-risk plan: assert absence of every forbidden disabled-mode field (`world.error`, `world.details`, `world_deps.report`, etc.).
- **Rollout / safety**:
  JSON remains additive-only, and enabled-mode failure visibility is preserved. The seam reduces backend traffic when disabled by preventing unnecessary probes.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now the active seam because `SEAM-1` published `C-01` / `THR-01`, and its seam-local plan has been refreshed against that closeout-backed handoff.
  - Which threads matter most: `THR-01`, `THR-02`, `THR-03`, `THR-04`.
  - What the first seam-local review should focus on: hidden probe paths, exact disabled-mode copy, and schema omission compatibility.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-02`, `C-03`, `C-04`
  - Threads likely to advance: `THR-02`, `THR-03`, `THR-04`
  - Review-surface areas likely to shift after landing: shim-doctor world section, world-deps section, JSON field presence/absence, and downstream status-driven branching
  - Downstream seams most likely to require revalidation: `SEAM-3`, `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
