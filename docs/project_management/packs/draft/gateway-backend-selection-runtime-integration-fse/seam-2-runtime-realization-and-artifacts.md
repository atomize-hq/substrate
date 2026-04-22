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
    - THR-02
  stale_triggers:
    - `SEAM-1` publishes selection, precedence, or inventory rules that differ from current assumptions
    - integrated auth payload/request types change outside this seam before execution planning starts
    - world-agent runtime launch or artifact management changes before this seam lands
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

# SEAM-2 - Runtime realization and artifacts

- **Goal / value**:
  - Land the adapter-driven integrated runtime path once `SEAM-1` publishes stable backend-selection truth so the repo can move from one hardcoded Codex realization to a real multi-backend execution surface.
  - Turn the existing canonical protocol, schema, and policy contracts into working code and test evidence rather than treating them as still-missing publication blockers.
- **Scope**
  - In:
    - integrated adapter binding lookup
    - required capability gating
    - request/auth payload widening beyond the current `cli_codex`-only shape
    - adapter-driven runtime config render
    - runtime artifact roots, naming, permissions, and inspectability
    - launch, readiness, and restart order
    - failure mapping in code for invalid integration, dependency unavailable, and explicit unsupported backend/capability cases
    - automated tests proving the selected backend survives sync/status/restart without collapsing back to Codex-only behavior
  - Out:
    - backend selection order, auth precedence, and inventory discoverability rules owned by `SEAM-1`
    - parity matrix, compatibility floor, and rollout posture owned by `SEAM-3`
    - new operator commands, status-schema widening, tuple-surface work, or ADR-0043 policy expansion
    - replacing the current env-compatible auth carrier with a new secret-channel design
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
    - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
    - current world-agent runtime manager behavior in `crates/world-agent/src/gateway_runtime.rs`
    - current shell-side integrated auth construction in `crates/shell/src/builtins/world_gateway.rs`
    - current lifecycle request shape in `crates/agent-api-types/src/lib.rs`
  - Outputs:
    - generalized integrated auth/request payload support in `crates/agent-api-types/src/lib.rs`
    - shell-side backend-aware request construction in `crates/shell/src/builtins/world_gateway.rs`
    - adapter-driven runtime realization in `crates/world-agent/src/gateway_runtime.rs` and `crates/world-agent/src/service.rs`
    - execution evidence and implementation notes carried by `THR-02`
- **Key invariants / rules**:
  - one selected backend resolves to one integrated adapter binding
  - capability gating, auth validation, config render, launch, and readiness must have one fixed order
  - auth precedence is already owned by `docs/contracts/substrate-gateway-policy-evaluation.md`: complete allowlisted env auth is primary, host credential files are fallback-only when env auth is absent, and partial env auth fails closed
  - current env-compatible delivery remains acceptable for this seam; execution must not block on a secret-channel redesign
  - runtime artifact semantics must be explicit implementation behavior rather than side effects of the current Codex-specific launch path
  - this seam must consume, not redefine, `SEAM-1` selection/policy truth
  - this seam must not widen operator commands, `status --json`, or tuple surfaces as a shortcut to ship runtime support
- **Dependencies**
  - Direct blockers:
    - `THR-01`
  - Transitive blockers:
    - none once `THR-01` publishes the remaining inventory-root and filename/id clarification
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
  - This seam consumes canonical contracts `C-01`, `C-02`, `C-03`, and `C-04`. The repo already has durable contract truth for adapter lookup ordering, capability/error taxonomy, and auth-source precedence; this seam should execute against that truth instead of reopening it.
  - Current pre-exec gate posture is:
    - `review: pending` because seam-local execution planning and slice decomposition have not yet been published.
    - `contract: passed` because `docs/contracts/substrate-gateway-backend-adapter-protocol.md`, `docs/contracts/substrate-gateway-backend-adapter-schema.md`, and `docs/contracts/substrate-gateway-policy-evaluation.md` already cover lookup order, capability gating, bounded error kinds, and env-primary/file-fallback auth precedence. The remaining prerequisite is upstream `THR-01` inventory-root publication, not a new `SEAM-2` contract definition step.
    - `revalidation: pending` because the current code still hardcodes `cli:codex` in request construction and runtime realization, so the seam must prove implementation alignment once `THR-01` settles.
  - Later seam-local verification should prove:
    - selected non-Codex backends no longer disappear behind the current `cli:codex` checks in `crates/shell/src/builtins/world_gateway.rs` and `crates/world-agent/src/gateway_runtime.rs`
    - adapter lookup and capability gating happen against the selected backend before launch
    - invalid integration, dependency unavailable, and unsupported capability/backend outcomes map to the correct runtime behavior without silent fallback
    - adapter-driven config render replaces the static Codex-only render path
    - managed artifacts have fixed roots, names, permissions, and inspectability rules
    - restart preserves the selected backend contract instead of re-deriving behavior ad hoc
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
- **Risks / unknowns**:
  - Risk:
    - the current request and runtime types only expose `cli_codex`, so widening the integrated path can accidentally entrench one-off variants instead of one adapter-owned shape
  - De-risk plan:
    - expand the shared request/auth shape and runtime lookup path together, then lock the behavior with tests before adding backend-specific branches
  - Risk:
    - unsupported backends or missing runtime bindings can collapse into the wrong exit bucket if implementation keeps treating Codex as the only integrated path
  - De-risk plan:
    - prove explicit failure mapping in code and tests using the existing contract taxonomy
  - Risk:
    - the current env-based auth delivery path may tempt implementers to treat carrier redesign as a prerequisite
  - De-risk plan:
    - keep carrier redesign explicitly out of scope and execute against the current policy contract's env-primary/file-fallback rule
  - Risk:
    - runtime artifact semantics can drift between shell, world-agent, and operator documentation
  - De-risk plan:
    - treat config path, manifest path, and managed log inspectability as one owned runtime-artifact surface
- **Rollout / safety**:
  - This seam stays `next` because it is the first execution seam after `SEAM-1`: once selection/inventory truth publishes, the remaining work is code, tests, and artifact validation rather than more contract invention.
  - Safety depends on consuming published `SEAM-1` truth rather than backfilling it inside runtime code, and on preserving the existing fail-closed policy boundary while widening runtime support.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because it is the immediate downstream execution target after `THR-01` publishes.
  - Which threads matter most
    - `THR-01`
    - `THR-02`
  - What the first seam-local review should focus on
    - adapter lookup and capability-gate implementation order
    - integrated auth/request payload widening strategy
    - runtime config render inputs and artifact roots/permissions
    - explicit no-fallback behavior for unsupported backends
    - tests needed to prove sync/status/restart keep the selected backend stable
  - Why deeper planning stays provisional
    - deeper planning stays provisional only until `SEAM-1` publishes the remaining inventory-root and filename/id truth; after that, this seam should move straight into execution
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none required for execution; this seam should consume the existing canonical contracts rather than create new blocker docs
  - Threads likely to advance:
    - `THR-02`
  - Review-surface areas likely to shift after landing:
    - runtime realization flow
    - auth handoff diagram
    - artifact path map
  - Downstream seams most likely to require revalidation:
    - `SEAM-3`
  - Seam exit should record landed code paths, tests, and any feature-local ADR-0046 implementation notes used to verify the change.
