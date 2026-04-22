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
    contract: passed
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---

# SEAM-3 - Parity, validation, and rollout

- **Goal / value**:
  - Prove that the multi-backend integrated runtime lands without regressing the existing operator contract or the current `cli:codex` floor.
  - Keep parity, unsupported-backend behavior, and rollout evidence as explicit execution proof rather than pretending they are blocked on another missing contract publication.
- **Scope**
  - In:
    - `cli:codex` regression floor
    - first additional integrated backend proof target once `SEAM-2` lands it in code
    - Linux/macOS/Windows validation matrix
    - explicit unsupported-backend posture
    - compatibility and rollout evidence
    - manual testing and smoke-script expectations that consume published upstream truth
  - Out:
    - backend selection and policy rule definition
    - runtime binding lookup, auth payload schema, and artifact semantics
    - tuple metadata work or `status --json` widening
    - inventing a new compatibility contract as a prerequisite to validation
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - `docs/contracts/substrate-gateway-runtime-parity.md`
    - current parity tests in `crates/world-agent/tests/gateway_runtime_parity.rs`
    - current shell command-path tests in `crates/shell/tests/world_gateway.rs`
  - Outputs:
    - automated parity coverage and regression evidence in `crates/world-agent/tests/gateway_runtime_parity.rs` and `crates/shell/tests/world_gateway.rs`
    - updated smoke/manual validation expectations in the ADR-0046 implementation pack surfaces
    - rollout evidence and proof artifacts carried by `THR-03`
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
    - a named first additional integrated backend must exist in landed code before cross-backend parity proof is authoritative
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
  - This seam consumes upstream contracts `C-01`, `C-02`, `C-03`, and `C-04` plus the existing lifecycle/status parity contract in `docs/contracts/substrate-gateway-runtime-parity.md`.
  - Current pre-exec gate posture is:
    - `review: pending` because the seam-local proof plan and evidence checklist have not yet been decomposed.
    - `contract: passed` because the operator/runtime parity surface is already owned by `docs/contracts/substrate-gateway-runtime-parity.md`, and unsupported-backend/no-fallback behavior is already implied by the existing selection/runtime contracts. This seam should validate those truths in code and smoke evidence rather than wait for a new compatibility contract.
    - `revalidation: pending` because parity proof is downstream of `THR-01`, `THR-02`, and the first landed non-Codex backend.
  - Later seam-local verification should prove:
    - `cli:codex` remains non-regressed
    - one additional integrated backend is named and exercised end to end
    - unsupported backends fail explicitly across Linux/macOS/Windows
    - rollout posture does not rely on widened status or tuple surfaces
    - smoke/manual evidence matches the existing operator/runtime parity contracts rather than an invented seam-local compatibility taxonomy
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Risks / unknowns**:
  - Risk:
    - parity work can start before a real second backend exists and then hardcode fake fixtures or promises
  - De-risk plan:
    - treat the additional-backend target as an execution dependency chosen by landed implementation, then build parity evidence around that real backend
  - Risk:
    - unsupported-backend behavior can quietly fall back to the current Codex path if explicit compatibility proof never lands
  - De-risk plan:
    - require automated and manual proof that unsupported backends fail explicitly across platforms
  - Risk:
    - parity work can drift back into upstream contract design if runtime realization remains unsettled
  - De-risk plan:
    - keep this seam future-only until `THR-01` and `THR-02` are published and a real additional backend exists to validate
- **Rollout / safety**:
  - This seam remains `future` because parity and rollout proof should verify landed upstream implementation rather than race ahead of it.
  - Safety depends on keeping `cli:codex` as the regression floor while proving explicit no-fallback behavior for unsupported backends.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `future` because it depends on upstream runtime implementation landing first, not because it needs another round of contract authoring.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - What the first seam-local review should focus on
    - baseline backend choice once implementation exists
    - compatibility matrix and no-fallback assertions
    - Linux/macOS/Windows evidence expectations
    - smoke/manual proof ownership
    - keeping tuple/status widening out of the proof seam
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none required for this seam to execute; validation should consume existing canonical contracts and attach proof to the implementation pack
  - Threads likely to advance:
    - `THR-03`
  - Review-surface areas likely to shift after landing:
    - parity matrix
    - rollout diagram
    - smoke coverage map
  - Downstream seams most likely to require revalidation:
    - downstream execution or release-governance packs rather than another seam inside this pack
  - Seam exit should record landed automated/manual validation evidence, the named additional-backend proof target, and any supporting ADR-0046 implementation notes used to land the rollout proof.
