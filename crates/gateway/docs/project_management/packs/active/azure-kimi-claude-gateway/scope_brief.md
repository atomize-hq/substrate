---
pack_id: azure-kimi-claude-gateway
pack_version: v1
pack_status: extracted
source_ref: repo-local ADR set, IMPORTANT_SUBSTRATE_ALIGNMENT.md, and Azure Kimi handoff chain
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - Azure Kimi Claude Gateway

- **Goal**: deliver a `claude-code-mux`-based gateway that makes Azure-hosted Kimi usable through an Anthropic Messages-compatible surface for Claude Code while keeping Azure normalization, planner/executor orchestration, and Substrate-facing boundary concerns internal and structured.
- **Why now**: the current CCR and one-shot adapter path proves Azure connectivity but fails structurally on Azure Kimi hidden tool intent in `reasoning_content`; the repo needs a real gateway plan before implementation diverges into thin patches.
- **Primary user(s) + JTBD**: Claude Code operators need a gateway that can drive Azure Kimi without losing tool behavior; future Substrate integrators need the gateway to remain one logical backend with an in-world-compatible and structured-event-compatible boundary.
- **In-scope**:
  - adopt the archived `claude-code-mux` codebase as the implementation foundation in this repo, stabilize it at baseline behavior, then perform the repo-local identity renaming pass before deeper modifications
  - model Azure Foundry Kimi as a first-class provider normalization problem, including hidden tool intent parsing from `reasoning_content`
  - prioritize an Anthropic Messages-compatible gateway surface as the first external delivery target
  - preserve internal planner/executor orchestration as internal policy rather than external backend identity
  - preserve a future Substrate path around one logical backend identity, replaceable deployment transport, and normalized structured events
- **Out-of-scope**:
  - shipping OpenAI Responses support as part of the active execution horizon
  - exposing separate external backend identities for planner, executor, provider parser, or model role selection
  - treating host-local loopback topology as the permanent architecture
  - leaking raw provider stream chunks as the downstream contract
- **Success criteria**:
  - the repo has a real adopted `claude-code-mux` baseline that is proven working near baseline state before identity renames and feature changes proceed
  - the baseline identity pass renames the adopted gateway to repo-local names, including the `substrate-gateway` crate identity and gateway/config naming surfaces
  - the repo has an explicit note on what commit `5a372fb` does or does not solve for Azure
  - Azure Kimi normalization has a bounded provider seam that can normalize both explicit `tool_calls` and hidden tool intent markers into one internal representation
  - the first public gateway path is Anthropic Messages-compatible for Claude Code without baking Anthropic-only structures into the core
  - planner/executor routing remains internal policy, not public backend identity
  - downstream integration direction remains one logical backend identity, in-world-compatible boundary, and structured events instead of raw provider streams
- **Constraints**:
  - `IMPORTANT_SUBSTRATE_ALIGNMENT.md` is mandatory and constrains external identity, deployment boundary, and event-shape decisions
  - ADR 0001 fixes `claude-code-mux` as the intended implementation foundation
  - ADR 0002 and the handoffs make Azure hidden-tool parsing the central provider-normalization risk
  - ADR 0004 makes Anthropic Messages the first delivery target while keeping OpenAI Responses as a later adapter seam
  - ADR 0005 through ADR 0007 prohibit exposing internal routing roles, loopback-only assumptions, and raw provider streams as the downstream contract
- **External systems / dependencies**:
  - upstream `claude-code-mux`
  - Azure Foundry Moonshot chat-completions endpoint and Azure Kimi model behavior
  - Claude Code as the first external client contract
  - future Substrate policy and agent-hub consumers for external integration direction
- **Known unknowns / risks**:
  - whether upstream commit `5a372fb` materially addresses the Azure-specific hidden-tool failure mode
  - whether `Kimi-K2.5` also needs hidden-tool normalization beyond the current `Kimi-K2-Thinking` evidence
  - which identity surfaces beyond the crate/package name must be renamed to fully disconnect the repo from the old project naming without destabilizing the baseline
  - whether Anthropic surface delivery should initially land with direct normalized execution before dual-model planner/executor policy is layered in
  - how much structured-event detail downstream integrations need without coupling to provider transport
- **Assumptions**:
  - Azure remains the mandatory upstream provider context for this feature
  - the two handoffs and the CCR log evidence summarized inside them remain the best current truth for Azure hidden-tool behavior
  - `SEAM-1`, `SEAM-2`, `SEAM-3`, and `SEAM-4` closeouts were the current promotion inputs for `SEAM-5`, and `SEAM-5` has now landed enough to publish `THR-05` and leave no active seam in this pack
  - future OpenAI Responses support is intentionally deferred into a later adapter seam outside the active horizon
