---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - surface implementation starts re-parsing raw Azure payloads or hidden markers instead of consuming normalized `C-02` events
    - session state, tool-result handling, or surface-visible metadata begin leaking planner/executor role selection or other internal policy truth
    - verification cannot prove Claude Code-compatible streaming and tool loops over `/v1/messages` without relying on raw provider inspection
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver Anthropic Session And Streaming Loop

- **User/system value**: Claude Code gets a usable Anthropic Messages-compatible path backed by normalized events rather than raw provider transport or speculative surface behavior.
- **Scope (in/out)**:
  - In: implement or tighten `/v1/messages` request handling, streaming response shaping, tool-use and tool-result continuation, final response behavior, and the verification surface that proves the public path stays thin over the normalized core.
  - Out: Azure normalization internals, planner/executor routing policy, external identity lock-in, downstream Substrate event/schema work, and seam-exit closeout accounting.
- **Acceptance criteria**:
  - `/v1/messages` consumes landed normalized events and does not require raw Azure response inspection to decide public block behavior
  - tool-use, tool-result follow-up, and final assistant completion behavior work for Claude Code-compatible Anthropic flows
  - verification covers streamed tool loops, non-stream or final-only paths, and session continuation on top of the normalized core
  - surface-level docs or code-local notes preserve the thin-adapter boundary for later Responses work
  - implementation anchors stay separated: provider normalization remains below the surface and internal policy remains above it
- **Dependencies**: `S1`, `../../threading.md`, `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, `gateway/src/providers/openai.rs`, `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, and `docs/foundation/claude-code-mux-extension-boundary.md`
- **Verification**:
  - a reviewer can trace one Claude Code request through `/v1/messages`, normalized event handling, and Anthropic block rendering without crossing into raw provider payload logic
  - pass condition: tool loop, streaming, and final-response behavior are all explainable in terms of `C-03` over `C-02`
  - failure conditions are explicit: raw Azure coupling at the surface, public exposure of internal routing roles, or a Responses-later boundary that now requires core refactoring
- **Rollout/safety**: keep public behavior capability-oriented and avoid turning local transport convenience into the architectural contract.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Likely mismatch hotspots`, `Pre-exec findings`)

#### S2.T1 - Wire The Surface To Normalized Event Semantics

- **Outcome**: the `/v1/messages` path renders public Anthropic behavior from landed normalized events instead of provider-specific response shape.
- **Inputs/outputs**: inputs are `C-02`, the owned `C-03` contract, and current surface anchors; outputs are the relevant route, renderer, and adapter changes plus any code-local documentation needed to keep the boundary explicit.
- **Thread/contract refs**: `THR-03`, `C-03`, `C-02`
- **Implementation notes**: treat provider-specific provenance as debug-only and keep block rendering strictly downstream of normalized event meaning.

#### S2.T2 - Prove The Session And Tool Loop

- **Outcome**: the seam has executable verification for session continuation, tool-result follow-up, and final response completion.
- **Inputs/outputs**: inputs are the public contract and current route/session behavior; outputs are tests, fixtures, or scripted verification paths that cover streamed tool use, follow-up turns, and final-only completion.
- **Thread/contract refs**: `THR-03`, `C-03`
- **Implementation notes**: verification must prove Claude Code-compatible behavior without requiring inspectors to look at raw Azure payload frames.

#### S2.T3 - Preserve The Thin Responses-Later Boundary

- **Outcome**: the first public surface does not hard-code Anthropic-only engine behavior.
- **Inputs/outputs**: inputs are the landed `C-03` contract and implementation anchors; output is explicit code or doc evidence that later Responses work still wraps the same normalized core.
- **Thread/contract refs**: `THR-03`, `C-03`, `C-02`
- **Implementation notes**: do not let helper abstractions make the Anthropic route the only internal data model.
