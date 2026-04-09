---
seam_id: SEAM-3
seam_slug: typed-runtime-and-platform-parity
type: platform
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
    - THR-03
    - THR-04
  stale_triggers:
    - typed world-agent endpoint ownership changes
    - shell-side exec probing is reintroduced
    - Linux/macOS/Windows parity guarantees or allowed divergence list changes
    - provisioning is pulled back into this pack
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
open_remediations: []
---

# SEAM-3 - Typed runtime and platform parity

- **Goal / value**:
  - Define the typed world-agent lifecycle/status path and platform guarantees that keep gateway lifecycle behavior stable across Linux, macOS, and Windows.
  - Prevent raw exec probing, backend-private quirks, or provisioning assumptions from becoming the operator contract.
- **Scope**
  - In:
    - typed world-agent lifecycle/status ownership
    - shell builtin consumption path
    - shared API type/client alignment
    - Linux/macOS/Windows parity guarantees for placement, lifecycle visibility, and status semantics
    - allowed divergence list and required validation evidence
  - Out:
    - the operator command family definition
    - the owned `status --json` field list and policy decision tables
    - provisioning-script changes or backend warm-flow edits
    - cross-doc/manual-playbook lock-in and checkpoint wiring
- **Primary interfaces**
  - Inputs:
    - `C-01`
    - `C-02`
    - `C-03`
    - selected Option A in `pre-planning/impact_map.md`
    - parity requirements in `pre-planning/ci_checkpoint_plan.md`
  - Outputs:
    - `C-04` for typed lifecycle/status transport ownership and platform parity expectations
    - downstream-ready evidence expectations for docs validation and quality gate work
- **Key invariants / rules**:
  - the CLI consumes a typed lifecycle/status surface rather than rebuilding operator state from runtime-private probes
  - the machine-readable status contract stays detached from gateway binary internals
  - Linux, macOS, and Windows share one operator-facing lifecycle and status contract even if backend implementation differs
  - provisioning remains out of scope for this pack
  - parity docs may describe allowed divergence, but they must not turn those divergences into separate user contracts
- **Dependencies**
  - Direct blockers:
    - `SEAM-2`
    - `THR-02`
    - `THR-03`
  - Transitive blockers:
    - `SEAM-1`
    - `THR-01`
  - Direct consumers:
    - `SEAM-4`
  - Derived consumers:
    - shell builtins
    - shared client types
    - parity docs and quality-gate evidence
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - `crates/world-agent/src/handlers.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/agent-api-client/src/lib.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `docs/WORLD.md`
- **Verification**:
  - This seam consumes upstream contracts `C-01`, `C-02`, and `C-03`; verification may depend on accepted upstream evidence for command behavior, status schema, and fail-closed policy rules.
  - This seam produces owned contract `C-04`. At seam-brief depth, verification is the runtime/parity contract becoming concrete enough for seam-local planning and implementation: typed surface ownership, host/backend call path, parity guarantees, divergence list, and required evidence.
  - Later seam-local verification should prove:
    - the typed world-agent path is authoritative for lifecycle/status operations
    - shell and shared clients consume the same runtime contract
    - Linux, macOS, and Windows guarantees are explicit and testable
    - provisioning remains correctly deferred outside this pack
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Risks / unknowns**:
  - Risk:
    - typed runtime ownership can drift back toward shell-assembled probing
  - De-risk plan:
    - keep Option A explicit in seam-local review and make the typed surface the only accepted operator path
  - Risk:
    - parity language can quietly absorb provisioning or backend-specific layout decisions that belong to a later runtime pack
  - De-risk plan:
    - keep the allowed divergence list and evidence requirements explicit while preserving the out-of-scope provisioning boundary
  - Risk:
    - Linux, macOS, and Windows can expose different lifecycle/status semantics under pressure from implementation shortcuts
  - De-risk plan:
    - require parity evidence to be part of the seam-local review and closeout, not an afterthought
- **Rollout / safety**:
  - This seam should not activate until the upstream status and policy surfaces are published; otherwise runtime details will hard-code provisional semantics.
  - Safety depends on keeping fail-closed and non-trust boundaries inherited from upstream contracts rather than re-deciding them locally.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because typed runtime/parity planning is the immediate downstream consumer once `SEAM-2` publishes schema and policy truth, but it should still wait behind the active seam.
  - Which threads matter most
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - What the first seam-local review should focus on
    - typed endpoint ownership
    - shared client/server contract boundaries
    - allowed divergence list
    - parity evidence requirements
    - proof that provisioning stays out of scope
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-04`
  - Threads likely to advance:
    - `THR-04`
  - Review-surface areas likely to shift after landing:
    - runtime path diagram
    - parity expectations
    - docs/WORLD touch map
  - Downstream seams most likely to require revalidation:
    - `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
