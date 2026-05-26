---
seam_id: SEAM-3
seam_slug: parity-validation-and-rollout
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
    - THR-03
  stale_triggers:
    - `SEAM-1` or `SEAM-2` publishes different selection, runtime, or artifact truth than the parity proof now assumes
    - the named `api:openai` proof target changes or another supported integrated backend is chosen first
    - Linux/macOS/Windows parity expectations, smoke ownership, or unsupported-backend handling change before landing
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

# SEAM-3 - Parity, validation, and rollout

- **Goal / value**:
  - Prove that the multi-backend integrated runtime lands without regressing the existing operator contract or the current `cli:codex` floor.
  - Keep parity, unsupported-backend behavior, and rollout evidence as explicit execution proof rather than pretending they are blocked on another missing contract publication.
  - Execute against the now-named first additional integrated backend proof target `api:openai` instead of planning around an unspecified future backend.
- **Scope**
  - In:
    - `cli:codex` regression floor
    - first additional integrated backend proof target `api:openai`
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
    - `docs/contracts/gateway/runtime-parity.md`
    - current parity tests in `crates/world-service/tests/gateway_runtime_parity.rs`
    - current shell command-path tests in `crates/shell/tests/world_gateway.rs`
  - Outputs:
    - automated parity coverage and regression evidence in `crates/world-service/tests/gateway_runtime_parity.rs` and `crates/shell/tests/world_gateway.rs`
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
    - none at promotion time beyond preserving the named `api:openai` proof target and current platform/runtime truth
  - Direct consumers:
    - none inside this pack
  - Derived consumers:
    - rollout review
    - compatibility docs
    - smoke and manual validation
- **Touch surface**:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/platform-parity-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/compatibility-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/windows-smoke.ps1`
  - `crates/world-service/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
- **Verification**:
  - This seam consumes upstream contracts `C-01`, `C-02`, `C-03`, and `C-04` plus the existing lifecycle/status parity contract in `docs/contracts/gateway/runtime-parity.md`.
  - Current pre-exec gate posture is:
    - `review: passed` because seam-local proof planning, falsification questions, and the parity/rollout review bundle now exist under `threaded-seams/seam-3-parity-validation-and-rollout/`.
    - `contract: passed` because the operator/runtime parity surface is already owned by `docs/contracts/gateway/runtime-parity.md`, and unsupported-backend/no-fallback behavior is already implied by the existing selection/runtime contracts. This seam should validate those truths in code and smoke evidence rather than wait for a new compatibility contract.
    - `revalidation: passed` because `governance/seam-2-closeout.md` publishes `THR-02`, names `api:openai` as the first landed non-Codex proof target, and the live runtime/test surfaces still expose `api_env`, `api:openai`, and explicit unsupported-backend behavior exactly where this seam expects to verify them.
  - Later seam-local verification should prove:
    - `cli:codex` remains non-regressed
    - `api:openai` is exercised end to end as the first additional integrated backend proof target
    - unsupported backends fail explicitly across Linux/macOS/Windows
    - rollout posture does not rely on widened status or tuple surfaces
    - smoke/manual evidence matches the existing operator/runtime parity contracts rather than an invented seam-local compatibility taxonomy
- **Canonical contract refs**:
  - `docs/contracts/gateway/backend-adapter-selection.md`
  - `docs/contracts/gateway/policy-evaluation.md`
  - `docs/contracts/gateway/backend-adapter-protocol.md`
  - `docs/contracts/gateway/backend-adapter-schema.md`
  - `docs/contracts/gateway/runtime-parity.md`
- **Risks / unknowns**:
  - Risk:
    - parity work can overfit to one `api:openai` proof path and stop exercising the explicit unsupported-backend posture that must remain visible
  - De-risk plan:
    - keep the `cli:codex` regression floor and unsupported-backend negative cases in the same verification matrix as the `api:openai` proof path
  - Risk:
    - unsupported-backend behavior can quietly fall back to the current Codex path if explicit compatibility proof never lands
  - De-risk plan:
    - require automated and manual proof that unsupported backends fail explicitly across platforms
  - Risk:
    - parity work can drift back into upstream contract design if runtime realization remains unsettled
  - De-risk plan:
    - keep this seam anchored to the landed `THR-02` handoff and re-run revalidation if runtime truth or the named proof target changes before landing
- **Rollout / safety**:
  - This seam has now landed because parity and rollout proof verified the landed upstream implementation without widening the operator or status surfaces.
  - Safety depends on keeping `cli:codex` as the regression floor while proving explicit no-fallback behavior for unsupported backends.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because the upstream runtime implementation landed, `THR-02` is now revalidated, and the remaining work is bounded conformance and rollout execution.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - What the first seam-local review should focus on
    - `api:openai` as the landed proof target
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
