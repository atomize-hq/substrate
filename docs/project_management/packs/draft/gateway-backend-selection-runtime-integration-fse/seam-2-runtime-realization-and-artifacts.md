---
seam_id: SEAM-2
seam_slug: runtime-realization-and-artifacts
type: integration
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
  stale_triggers:
    - `SEAM-1` publishes selection, precedence, or inventory rules that differ from current assumptions
    - integrated auth payload shape widens beyond the current `cli_codex` variant
    - world-agent runtime launch flow changes before runtime contract publication
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
  - REM-003
  - REM-004
---

# SEAM-2 - Runtime realization and artifacts

- **Goal / value**:
  - Freeze the integrated runtime path once selection succeeds so the repo can move from one hardcoded Codex realization to one adapter-driven realization surface.
  - Make runtime binding, auth handoff classification, and artifact semantics explicit enough that parity and rollout work can later verify them rather than invent them.
- **Scope**
  - In:
    - integrated adapter binding lookup
    - required capability gating
    - missing binding classification
    - missing auth handoff material classification after policy permits the read path
    - adapter-driven config render
    - runtime artifact roots, naming, permissions, and inspectability
    - launch, readiness, and restart order
  - Out:
    - backend selection order, auth precedence, and inventory discoverability rules owned by `SEAM-1`
    - parity matrix, compatibility floor, and rollout posture owned by `SEAM-3`
    - new operator commands, status-schema widening, or tuple-surface work
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - current world-agent runtime manager behavior in `crates/world-agent/src/gateway_runtime.rs`
    - current lifecycle request shape in `crates/agent-api-types/src/lib.rs`
  - Outputs:
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
    - one landed ADR-0046 delta in `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
    - explicit classification for missing binding and missing auth material carried by `THR-02`
- **Key invariants / rules**:
  - one selected backend resolves to one integrated adapter binding
  - capability gating, auth validation, config render, launch, and readiness must have one fixed order
  - runtime artifact semantics must not become implicit side effects of the current Codex-specific launch path
  - this seam must consume, not redefine, `SEAM-1` selection/policy truth
  - this seam must not widen operator commands, `status --json`, or tuple surfaces
- **Dependencies**
  - Direct blockers:
    - `THR-01`
  - Transitive blockers:
    - `SEAM-1` closeout must publish explicit inventory and auth-precedence truth
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - world-agent runtime launch code
    - shared request/response shapes
    - parity and smoke tests
- **Touch surface**:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
  - `crates/world-agent/src/gateway_runtime.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
- **Verification**:
  - This seam consumes upstream contracts `C-01` and `C-02`; verification may depend on accepted upstream evidence for selection order, policy gates, auth precedence, and inventory rules.
  - This seam produces owned contracts `C-03` and `C-04` through the feature-local ADR-0046 docs `gateway-runtime-adapter-protocol-spec.md`, `gateway-runtime-adapter-schema-spec.md`, and `filesystem-semantics-spec.md`. Verification at seam-brief depth is those feature-local deltas becoming concrete enough for seam-local planning and implementation: binding lookup, capability taxonomy, auth handoff shapes, artifact semantics, and runtime ordering.
  - The consumed external authorities under `docs/contracts/*` remain compatibility dependencies; verification here does not require editing them.
  - Later seam-local verification should prove:
    - missing binding and missing auth material are classified explicitly
    - adapter-driven config render replaces the static Codex-only render path
    - managed artifacts have fixed roots, names, and inspectability rules
    - restart preserves the selected backend contract instead of re-deriving behavior ad hoc
- **Canonical contract refs**:
  - Owned feature-local outputs:
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
    - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
  - Consumed external authorities:
    - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
    - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
    - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Risks / unknowns**:
  - Risk:
    - the current request and runtime types only expose `cli_codex`, so widening the integrated path can accidentally entrench one-off variants instead of one adapter-owned shape
  - De-risk plan:
    - freeze the integrated protocol/schema boundary before implementation multiplies special cases
  - Risk:
    - missing binding and missing auth material can collapse into the wrong exit bucket if classification is not explicit
  - De-risk plan:
    - keep those classifications as first-class runtime remediations before seam-local protocol/schema planning
  - Risk:
    - runtime artifact semantics can drift between shell, world-agent, and operator documentation
  - De-risk plan:
    - treat config path, manifest path, and managed log inspectability as one owned runtime-artifact surface
- **Rollout / safety**:
  - This seam stays `next` because its deeper planning is still provisional under the subslice matrix: upstream contract stability is partial, coupling to upstream semantics is still medium-high, and the unresolved items affect authoritative publication surfaces.
  - Safety depends on consuming published `SEAM-1` truth rather than backfilling it inside runtime code.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because it is the immediate downstream consumer of the active seam and the next credible planning target after `THR-01` publishes.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
  - What the first seam-local review should focus on
    - binding lookup semantics
    - missing-binding classification
    - missing-auth classification
    - integrated auth payload widening strategy
    - runtime artifact roots and permissions
    - whether `S00` is needed to freeze protocol/schema truth before implementation slices begin
  - Why deeper planning stays provisional
    - under the provisional matrix, this seam still has contract-authority impact and non-trivial coupling to upstream semantics, so any deeper planning must stay provisional until `SEAM-1` publishes explicit truth
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-03` via `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
    - `C-04` via `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md` and `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
  - Threads likely to advance:
    - `THR-02`
  - Review-surface areas likely to shift after landing:
    - runtime realization flow
    - auth handoff diagram
    - artifact path map
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
  - Seam exit should record the landed feature-local outputs and their compatibility against consumed external authorities; it does not require editing the external authorities themselves.
