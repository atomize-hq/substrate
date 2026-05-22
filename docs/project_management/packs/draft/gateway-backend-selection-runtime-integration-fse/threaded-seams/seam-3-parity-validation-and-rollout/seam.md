---
seam_id: SEAM-3
seam_slug: parity-validation-and-rollout
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-parity-validation-and-rollout.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
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

## Seam Brief (Restated)

- **Goal / value**:
  - Prove that the bounded multi-backend runtime handoff published by `SEAM-2` holds across operator-facing lifecycle behavior, platform validation, and rollout evidence.
  - Preserve `cli:codex` as the regression floor while proving `api:openai` as the first landed additional-backend path and keeping unsupported-backend handling explicit with no fallback.
- **Type**:
  - conformance
- **Scope**
  - In:
    - parity coverage in runtime and shell tests for `cli:codex`, `api:openai`, and explicit unsupported-backend behavior
    - Linux/macOS/Windows validation expectations and smoke/manual evidence
    - rollout and compatibility notes that consume, rather than redefine, the existing operator/runtime parity contracts
    - publication-ready proof surfaces for `THR-03`
  - Out:
    - new backend selection or policy rules owned by `SEAM-1`
    - runtime binding, auth payload, or artifact semantics owned by `SEAM-2`
    - status-schema widening, tuple-surface changes, or a new top-level operator command family
    - treating a new compatibility contract as a prerequisite for validation
- **Touch surface**:
  - `crates/world-service/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/platform-parity-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/compatibility-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/windows-smoke.ps1`
- **Verification**:
  - This seam **consumes** published `C-01`, `C-02`, `C-03`, and `C-04` from `THR-01` and `THR-02`.
  - This seam **owns and realizes** `C-05`, whose canonical baseline already exists under:
    - `docs/contracts/substrate-gateway-runtime-parity.md`
  - Readiness means execution may start without inventing new upstream truth:
    - `SEAM-2` closeout already names `api:openai` as the first landed non-`cli:codex` proof target
    - live shell, world-service, and shared request/auth surfaces still expose `api:openai`, `api_env`, and explicit unsupported-backend behavior
    - remaining work is parity evidence, platform validation, and rollout publication, not new contract definition
  - Verification for this seam centers on:
    - keeping `cli:codex` as the regression floor in `crates/world-service/tests/gateway_runtime_parity.rs` and `crates/shell/tests/world_gateway.rs`
    - exercising `api:openai` end to end as the first additional-backend proof target
    - proving unsupported backends fail explicitly without silent fallback across Linux/macOS/Windows evidence surfaces
    - keeping rollout evidence subordinate to canonical operator/runtime contracts
- **Basis posture**:
  - Currentness:
    - `current` because `SEAM-1` and `SEAM-2` closeouts are landed, `THR-02` was revalidated during promotion, and the live repo still matches the published multi-backend runtime handoff.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none remain at promotion time; `THR-01` is already revalidated and `THR-02` is now revalidated for this seam
  - Downstream blocked seams:
    - none inside this pack
  - Contracts produced:
    - `C-05`
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
  - Canonical contract refs:
    - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`
    - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
    - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
    - `docs/contracts/substrate-gateway-runtime-parity.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` remains pack orientation only and does not satisfy the seam-local review gate by itself.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-03` is the pack's outbound parity and rollout publication handoff, so downstream release or rollout work must consume one explicit closeout-backed proof record rather than inferred test archaeology.
- **Expected contracts to publish**:
  - none as new canonical docs; this seam should attach proof to the existing canonical runtime parity contract and supporting ADR-0046 pack surfaces
- **Expected threads to publish / advance**:
  - `THR-03`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - the named `api:openai` proof target changes
  - unsupported-backend/no-fallback behavior changes
  - platform validation ownership or smoke evidence expectations change
  - rollout notes start implying widened operator or status surfaces
- **Expected closeout evidence**:
  - landed parity coverage in runtime and shell tests
  - landed or refreshed platform validation and smoke/manual evidence
  - a recorded planned-vs-landed delta for any platform-specific proof nuance that does not widen the operator contract
  - explicit `THR-03` publication and promotion-readiness posture for any downstream release or rollout work

## Slice index

- `S1` -> `slice-1-parity-regression-floor-and-backend-matrix.md`
- `S2` -> `slice-2-platform-evidence-and-unsupported-backend-validation.md`
- `S3` -> `slice-3-rollout-proof-and-compatibility-publication.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Threading alignment

- **Dependency edges**:
  - `SEAM-1` -> `SEAM-3` via `THR-01` carrying `C-01` and `C-02`
  - `SEAM-2` -> `SEAM-3` via `THR-02` carrying `C-03` and `C-04`
  - `SEAM-3` -> downstream rollout/release governance via `THR-03` carrying `C-05`
- **Execution posture**:
  - The seam is now `status: landed`: the parity matrix, platform evidence, rollout publication, and seam-exit closeout have all landed against the current basis.
  - `THR-02` remains revalidated for this seam, and `THR-03` is now published from the seam closeout.
- **Slicing strategy**:
  - lock the automated parity matrix first, then platform evidence and unsupported-backend validation, then rollout publication surfaces, then explicit seam exit

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
