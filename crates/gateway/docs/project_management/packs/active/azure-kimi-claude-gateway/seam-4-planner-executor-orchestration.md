---
seam_id: SEAM-4
seam_slug: planner-executor-orchestration
type: integration
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
    - governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-04
  stale_triggers:
    - any later `SEAM-3` contract or closeout change that alters public streaming semantics, session/tool loop guarantees, or the thin-adapter boundary for future Responses work
    - any later change to `C-02` normalized tool/action/final semantics that alters the planner or executor handoff basis
    - any later routing evidence that requires public model-role exposure or provider-aware policy branches
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
  planned_location: S3
  status: pending
open_remediations: []
---

# SEAM-4 - Planner Executor Orchestration

- **Goal / value**: define and implement internal routing policy that can use `Kimi-K2-Thinking` for planning and `Kimi-K2.5` for execution without leaking those roles into the external backend identity or the provider parser.
- **Scope**
  - In:
    - internal model or route selection policy
    - state handoff between planning and execution turns
    - policy configuration and diagnostics
    - explicit separation between normalization and orchestration
  - Out:
    - owning provider parsing or raw Azure normalization
    - exposing separate public planner and executor backends
    - adding OpenAI Responses as a public client surface
- **Primary interfaces**
  - Inputs:
    - `C-01` foundation boundary
    - `C-02` normalized internal event contract
    - landed `C-03` public surface assumptions where relevant
  - Outputs:
    - `C-04` internal policy contract
    - named state-handoff and routing invariants
- **Key invariants / rules**:
  - planner/executor routing is internal policy, not public backend identity
  - provider normalization stays below this seam and must not depend on policy decisions
  - external clients must not need to reason about internal model-role selection
  - diagnostics may expose internal decisions for debugging, but public config examples remain capability-oriented
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` via `THR-01`
    - `SEAM-2` via `THR-02`
  - Transitive blockers:
    - `SEAM-3` may inform public-surface assumptions that this seam must not violate
  - Direct consumers:
    - `SEAM-3`
    - `SEAM-5`
  - Derived consumers:
    - operators and future policy/config integration work
- **Touch surface**:
  - routing policy modules
  - session-state handoff logic
  - internal config surfaces
  - diagnostics and observability hooks
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify a planning-to-execution handoff can occur through normalized events without the parser layer making policy decisions.
  - Verify the owned policy contract is concrete enough for downstream planning and does not leak separate public backend roles.
  - Verify the public gateway contract can remain singular even as internal model selection changes.
- **Risks / unknowns**:
  - Risk: routing policy may sprawl into user-visible behavior that really belongs to the Anthropic surface seam.
  - De-risk plan: seam-local review should isolate which semantics are public contract versus internal execution policy.
  - Risk: policy assumptions may become stale quickly if Azure model behavior shifts.
  - De-risk plan: encode revalidation triggers around model-role evidence and keep the seam-local basis explicit when new public-surface or routing evidence lands.
- **Rollout / safety**:
  - keep orchestration policy behind internal boundaries until the normalized event contract is stable
  - avoid freezing operator-facing config around internal role names
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `active` because `SEAM-3` landed and published `THR-03`, making the public-surface boundary current enough for planner/executor policy to move into seam-local execution planning
  - Which threads matter most: `THR-01`, `THR-02`, `THR-04`
  - What the seam-local review now focuses on: session ownership, route selection rules, handoff through normalized events, and whether any public config or surface assumption leaks internal model roles
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-04`
  - Threads likely to advance: `THR-04`
  - Review-surface areas likely to shift after landing: `R1`, `R2`, `R3`
  - Downstream seams most likely to require revalidation: `SEAM-3`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
