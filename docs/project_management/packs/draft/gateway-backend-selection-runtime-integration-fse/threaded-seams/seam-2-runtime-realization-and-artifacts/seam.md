---
seam_id: SEAM-2
seam_slug: runtime-realization-and-artifacts
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-runtime-realization-and-artifacts.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - `SEAM-1` selection, precedence, or inventory truth changes after this review refresh
    - integrated auth payload or request-schema expectations change outside this seam before landing
    - runtime artifact naming, permissions, or readiness semantics drift outside the planned slice order
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
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-2 - Runtime realization and artifacts

## Seam Brief (Restated)

- **Goal / value**:
  - Realize the integrated adapter runtime path from the now-published selection and policy handoff so Substrate stops treating `cli:codex` as the only executable integrated backend lifecycle.
  - Publish `THR-02` with landed runtime behavior, typed request/auth surfaces, artifact semantics, and drift-guard tests that `SEAM-3` can consume.
- **Type**:
  - integration
- **Scope**
  - In:
    - adapter binding lookup and capability gating in runtime-owned code paths
    - shared request and integrated auth payload widening beyond the current `cli_codex`-only shape
    - adapter-driven config render plus managed runtime artifact semantics
    - launch, readiness, restart, and failure mapping for supported versus unavailable integrated backends
    - shell/world-service/shared-type tests that lock in the selected-backend runtime handoff
  - Out:
    - backend selection order, inventory identity, and auth precedence definition owned by `SEAM-1`
    - parity matrix, rollout proof, and named additional-backend validation owned by `SEAM-3`
    - new operator commands, status-schema widening, tuple-surface changes, or secret-channel redesign
- **Touch surface**:
  - `crates/world-service/src/gateway_runtime.rs`
  - `crates/world-service/src/service.rs`
  - `crates/transport-api-types/src/lib.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/world-service/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
- **Verification**:
  - This seam **consumes** published `C-01` and `C-02` from `THR-01`.
  - This seam **owns and realizes** `C-03` and `C-04`, whose canonical baselines already exist under:
    - `docs/contracts/gateway/backend-adapter-protocol.md`
    - `docs/contracts/gateway/backend-adapter-schema.md`
  - Readiness means execution may start without inventing new upstream contract truth:
    - selected backend id already arrives from shell as a fixed input
    - auth precedence is already pinned by canonical policy docs
    - remaining gaps are runtime lookup, payload shape, artifact semantics, and lifecycle behavior that this seam owns
  - Verification for this seam centers on:
    - adapter lookup and capability validation order in `crates/world-service/src/service.rs` and `crates/world-service/src/gateway_runtime.rs`
    - shared integrated-auth/request shape generalization in `crates/transport-api-types/src/lib.rs`
    - shell request construction staying backend-aware without reopening selection ownership
    - runtime tests proving unsupported backends fail explicitly while supported backends preserve the selected-backend contract across sync/status/restart
- **Basis posture**:
  - Currentness:
    - `current` because `SEAM-1` closeout published `THR-01`, the active seam revalidated against that closeout, and the remaining repo gaps still match this seam's implementation plan.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - `THR-01` is satisfied and revalidated for this seam
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-03`
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`
  - Canonical contract refs:
    - `docs/contracts/gateway/backend-adapter-selection.md`
    - `docs/contracts/gateway/policy-evaluation.md`
    - `docs/contracts/gateway/backend-adapter-protocol.md`
    - `docs/contracts/gateway/backend-adapter-schema.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` remains pack orientation only and does not satisfy the seam-local review gate by itself.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-02` is the controlling upstream handoff for `SEAM-3`, and parity work must consume landed runtime truth rather than infer it from partial implementation.
- **Expected contracts to publish**:
  - none as new canonical docs; this seam should land runtime behavior against existing canonical protocol/schema contracts
- **Expected threads to publish / advance**:
  - `THR-02`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - adapter lookup or capability-gate order changes
  - integrated auth/request payload shape changes
  - runtime artifact naming, permissions, or inspectability changes
  - readiness or restart semantics change relative to the reviewed plan
- **Expected closeout evidence**:
  - landed runtime updates in `crates/world-service/src/gateway_runtime.rs` and `crates/world-service/src/service.rs`
  - landed shared request/auth updates in `crates/transport-api-types/src/lib.rs`
  - landed shell request updates in `crates/shell/src/builtins/world_gateway.rs`
  - landed regression coverage in runtime and shell tests
  - resolution or explicit carry status for `REM-003` and `REM-004`

## Slice index

- `S1` -> `slice-1-binding-lookup-and-capability-gates.md`
- `S2` -> `slice-2-request-auth-and-runtime-artifacts.md`
- `S3` -> `slice-3-lifecycle-conformance-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Threading alignment

- **Dependency edges**:
  - `SEAM-1` -> `SEAM-2` via `THR-01` carrying `C-01` and `C-02`
  - `SEAM-2` -> `SEAM-3` via `THR-02` carrying `C-03` and `C-04`
- **Execution posture**:
  - The seam is now `status: exec-ready`: seam-local review, contract, and revalidation gates all pass, the basis is current, and the remaining work is bounded runtime implementation.
  - `THR-01` has been revalidated for this seam, while `THR-02` remains the outbound publication target for closeout.
- **Slicing strategy**:
  - runtime binding and capability gates first, then request/auth/schema plus artifact behavior, then lifecycle conformance and drift guards, then explicit seam exit

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
