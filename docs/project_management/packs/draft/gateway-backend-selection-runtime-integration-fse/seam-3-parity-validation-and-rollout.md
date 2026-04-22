---
seam_id: SEAM-3
seam_slug: parity-validation-and-rollout
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
    - THR-01
    - THR-02
  stale_triggers:
    - `SEAM-1` or `SEAM-2` publishes different classification or artifact truth than assumed here
    - the first additional integrated backend baseline is fixed elsewhere
    - Linux/macOS/Windows parity expectations or smoke ownership change
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
  planned_location: S99
  status: pending
open_remediations:
  - REM-005
---

# SEAM-3 - Parity, validation, and rollout

- **Goal / value**:
  - Publish the proof and rollout posture for expanding integrated lifecycle support beyond the current Codex-only baseline without regressing the existing operator contract.
  - Keep parity and compatibility as explicit downstream verification work rather than allowing unsupported-backend behavior or platform drift to emerge accidentally.
- **Scope**
  - In:
    - `cli:codex` regression floor
    - first additional integrated backend baseline
    - Linux/macOS/Windows validation matrix
    - explicit unsupported-backend posture
    - compatibility and rollout evidence
    - manual testing and smoke-script expectations that consume published upstream truth
  - Out:
    - backend selection and policy rule definition
    - runtime binding lookup, auth payload schema, and artifact semantics
    - tuple metadata work or `status --json` widening
    - editing the external authority docs during extraction
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - current parity tests in `crates/world-agent/tests/gateway_runtime_parity.rs`
    - current shell command-path tests in `crates/shell/tests/world_gateway.rs`
  - Outputs:
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
    - one future parity and rollout contract carried by `THR-03`
    - one named additional-backend proof target
    - one explicit no-fallback posture for unsupported integrated backends
- **Key invariants / rules**:
  - `cli:codex` remains the regression floor until another backend is explicitly supported
  - parity proof consumes published upstream truth rather than reopening it
  - unsupported backends must fail explicitly rather than silently collapsing back to a Codex path
  - platform divergence may exist under the hood, but the operator-facing contract remains one lifecycle surface
  - this seam must not widen operator commands or status schema as a shortcut to expose rollout state
- **Dependencies**
  - Direct blockers:
    - `THR-01`
    - `THR-02`
  - Transitive blockers:
    - a named first additional integrated backend baseline must exist before parity proof is authoritative
  - Direct consumers:
    - none inside this pack
  - Derived consumers:
    - rollout review
    - compatibility docs
    - smoke and manual validation
- **Touch surface**:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/windows-smoke.ps1`
  - `crates/world-agent/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
- **Verification**:
  - This seam consumes upstream contracts `C-01`, `C-02`, `C-03`, and `C-04`; verification may depend on accepted upstream evidence for selection truth, runtime realization truth, and published classifications.
  - This seam produces owned contract `C-05` through the feature-local ADR-0046 docs `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`. Verification at seam-brief depth is that those feature-local deltas become concrete enough for seam-local planning and implementation: baseline backend set, explicit failure matrix, and platform evidence obligations.
  - The consumed external authorities under `docs/contracts/*` remain compatibility dependencies; verification here does not require editing them.
  - Later seam-local verification should prove:
    - `cli:codex` remains non-regressed
    - one additional integrated backend is named and exercised end to end
    - unsupported backends fail explicitly across Linux/macOS/Windows
    - rollout posture does not rely on widened status or tuple surfaces
- **Canonical contract refs**:
  - Owned feature-local outputs:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
  - Consumed external authorities:
    - `docs/contracts/substrate-gateway-runtime-parity.md`
    - `docs/contracts/substrate-gateway-operator-contract.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Risks / unknowns**:
  - Risk:
    - parity planning can start without a fixed additional-backend baseline and then hardcode the wrong fixtures, smoke assertions, or rollout promise
  - De-risk plan:
    - keep the baseline as an explicit future-seam remediation rather than assuming a backend id here
  - Risk:
    - unsupported-backend behavior can quietly fall back to the current Codex path if explicit compatibility proof never lands
  - De-risk plan:
    - require the future seam to publish a no-fallback validation matrix across platforms
  - Risk:
    - parity work can drift back into upstream contract design if runtime realization remains unsettled
  - De-risk plan:
    - keep this seam future-only until `THR-01` and `THR-02` are published
- **Rollout / safety**:
  - This seam remains `future` because parity and rollout proof should verify landed upstream truth rather than race ahead of it.
  - Safety depends on holding the rollout surface behind published runtime truth and an explicit additional-backend baseline.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `future` because it depends on both upstream contract publication and a missing additional-backend baseline.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - What the first seam-local review should focus on
    - baseline backend choice
    - compatibility matrix
    - unsupported-backend behavior
    - Linux/macOS/Windows evidence expectations
    - keeping tuple/status widening out of the proof seam
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-05` via `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`, `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`, and `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
  - Threads likely to advance:
    - `THR-03`
  - Review-surface areas likely to shift after landing:
    - parity matrix
    - rollout diagram
    - smoke coverage map
  - Downstream seams most likely to require revalidation:
    - downstream execution or release-governance packs rather than another seam inside this pack
  - Seam exit should record the landed feature-local outputs and their compatibility against consumed external authorities; it does not require editing the external authorities themselves.
