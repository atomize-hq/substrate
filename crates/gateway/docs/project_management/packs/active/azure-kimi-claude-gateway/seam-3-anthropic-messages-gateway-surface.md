---
seam_id: SEAM-3
seam_slug: anthropic-messages-gateway-surface
type: capability
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
    - any change to `docs/foundation/azure-kimi-c02-normalized-event-contract.md` that alters normalized tool/action/final semantics or `source_origin` guarantees requires `SEAM-3` revalidation
    - client-surface work starts depending on raw Azure payload framing, hidden sentinel syntax, or planner/executor role selection instead of the normalized core
    - the client-surface boundary in `docs/foundation/claude-code-mux-extension-boundary.md` moves in a way that collapses the thin-adapter posture for later Responses work
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
  planned_location: S3
  status: passed
open_remediations: []
---

# SEAM-3 - Anthropic Messages Gateway Surface

- **Goal / value**: deliver the first user-facing Claude Code-compatible gateway surface using Anthropic Messages semantics while keeping the core engine client-agnostic.
- **Scope**
  - In:
    - Anthropic Messages-compatible ingress and streaming boundary
    - mapping normalized internal tool/action/final events to Anthropic-compatible client behavior
    - session continuation and tool result loop semantics for Claude Code
    - local development transport as a replaceable outer layer rather than a permanent architecture assumption
  - Out:
    - OpenAI Responses delivery
    - external exposure of planner/executor or provider role selection
    - raw provider passthrough or provider-specific event leakage
- **Primary interfaces**
  - Inputs:
    - `C-01` foundation boundary
    - `C-02` normalized internal event contract
    - Claude Code Anthropic Messages compatibility requirements from the ADR set
  - Outputs:
    - `C-03` Anthropic-compatible external API contract
    - named session and streaming invariants for downstream conformance work
- **Key invariants / rules**:
  - Anthropic Messages is the first delivery target, but it must not become the core engine's only internal shape
  - public backend identity remains singular and capability-oriented
  - local HTTP is an outer transport convenience, not the architecture's defining boundary
  - surface behavior must consume normalized events, not raw provider streams
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` via `THR-01`
    - `SEAM-2` via `THR-02`
  - Transitive blockers:
    - none
  - Direct consumers:
    - Claude Code and other Anthropic-compatible clients
  - Derived consumers:
    - `SEAM-5`
    - future OpenAI Responses adapter work
- **Touch surface**:
  - Anthropic-compatible routes and streaming adapters
  - session state boundaries
  - tool result and final response shaping
  - public-facing docs for the first client contract
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify Claude Code can use the gateway through an Anthropic-compatible Messages path backed by normalized internal events.
  - Verify the core boundary remains usable for a later OpenAI Responses adapter without provider-normalization refactors.
  - Verify tool and session semantics do not depend on raw provider stream artifacts.
- **Risks / unknowns**:
  - Risk: `claude-code-mux` may already provide some of this surface, but Azure-specific normalization may force deeper internal changes than expected.
  - De-risk plan: seam-local review should compare reused upstream surface behavior against the `C-02` contract before detailed slicing.
  - Risk: public-surface work may accidentally freeze transport assumptions that conflict with the in-world-compatible boundary.
  - De-risk plan: keep transport and auth concerns explicitly outside the normalized core and document the replaceable outer-layer boundary.
- **Rollout / safety**:
  - expose Anthropic compatibility first, but keep OpenAI Responses as a later thin outer adapter rather than a forked engine
  - guard against user-visible drift by basing surface behavior on `C-02` rather than provider-specific ad hoc branches
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` because the seam has landed, published `THR-03`, and left the forward planning window after promoting `SEAM-4` into active work
  - Which threads matter most: `THR-01`, `THR-02`, `THR-03`
  - What the authoritative seam-local review now focuses on: public streaming semantics, tool loop behavior, and the exact surface/core boundary that keeps future Responses work a thin outer adapter
- **Expected seam-exit concerns**:
  - Contracts published: `C-03`
  - Threads advanced: `THR-03`
  - Review-surface areas shifted by landing: `R1`, `R3`
  - Downstream seams most likely to require revalidation after later drift: `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
