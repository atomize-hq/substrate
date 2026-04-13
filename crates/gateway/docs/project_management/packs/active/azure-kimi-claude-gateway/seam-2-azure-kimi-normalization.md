---
seam_id: SEAM-2
seam_slug: azure-kimi-normalization
type: integration
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - any later change to the frozen `C-02` hidden-marker semantics or Kimi-family variant coverage requires downstream revalidation
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
  planned_location: S4
  status: passed
open_remediations: []
---

# SEAM-2 - Azure Kimi Provider Normalization

- **Goal / value**: isolate Azure Foundry Kimi behavior behind a provider boundary that converts both explicit `tool_calls` and hidden tool intent in `reasoning_content` into stable internal tool/action/final events.
- **Scope**
  - In:
    - Azure chat-completions provider or provider-mode boundary
    - parsing of hidden Kimi sentinel markers from `reasoning_content`
    - normalization of explicit and hidden tool intent into one internal event model
    - regression fixtures and probes for observed Azure Kimi hidden-tool patterns
  - Out:
    - Anthropic Messages surface delivery
    - OpenAI Responses delivery
    - public backend identity or external policy surface
    - planner/executor routing policy beyond what is required to keep normalization independent
- **Primary interfaces**
  - Inputs:
    - `C-01` foundation boundary from `SEAM-1`
    - Azure Kimi response behavior captured in the handoffs
    - ADR constraints forbidding raw sentinel syntax outside the provider seam
  - Outputs:
    - `C-02` normalized internal event contract
    - provider-specific fixtures and parsing invariants
    - explicit statement of what upstream `claude-code-mux` behavior was reused versus bypassed
- **Key invariants / rules**:
  - downstream code must not depend on Azure sentinel markers or raw provider chunk shapes
  - request compatibility must not rely on unsupported Azure `thinking` or `reasoning` request fields
  - planner/executor routing policy must remain outside the parser layer
  - normalized events must be concrete enough for later Anthropic and Substrate-facing seams to consume without re-parsing provider text
- **Dependencies**
  - Direct blockers:
    - none; `THR-01` is already published from `SEAM-1`
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Derived consumers:
    - future OpenAI Responses adapter work
- **Touch surface**:
  - Azure provider adapter code
  - reasoning parser and normalization helpers
  - event model definitions
  - fixture corpus and regression tests
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify explicit `tool_calls` and hidden tool markers normalize into one named internal representation.
  - Verify the owned event contract is concrete enough for downstream seam-local planning.
  - Verify regression fixtures cover the observed hidden-tool marker patterns and any confirmed `Kimi-K2.5` variants.
- **Risks / unknowns**:
  - Risk: `Kimi-K2.5` may share enough hidden-tool behavior that the parser contract must cover more than the current evidence set.
  - De-risk plan: run targeted probes and record fixture additions as contract-shaping inputs.
  - Risk: provider normalization may be tempted to absorb planner/executor state decisions.
  - De-risk plan: require seam-local review to falsify any design that mixes parsing with orchestration policy.
- **Rollout / safety**:
  - land parser and normalization work behind the provider boundary before changing public gateway semantics
  - preserve raw provider inspection paths for debugging without turning them into downstream contracts
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` because the normalization seam has landed and published `THR-02`; downstream seams now consume its closeout-backed truth rather than keep it in the forward planning window
  - Which threads matter most: `THR-01`, `THR-02`
  - What the first seam-local review should focus on: owned event contract boundaries, parser invariants, and fixture sufficiency versus speculative orchestration logic
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-02`
  - Threads likely to advance: `THR-02`
  - Review-surface areas likely to shift after landing: `R1`, `R2`, `R3`
  - Downstream seams most likely to require revalidation: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
