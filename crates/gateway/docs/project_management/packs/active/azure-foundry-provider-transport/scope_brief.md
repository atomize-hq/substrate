---
pack_id: azure-foundry-provider-transport
pack_version: v1
pack_status: extracted
source_ref: follow-on to azure-kimi-claude-gateway closeouts plus current Azure transport/config/runtime anchors in gateway/
execution_horizon:
  active_seam: SEAM-2
  next_seam: null
---

# Scope Brief - Azure Foundry Provider Transport

- **Goal**: make Azure a first-class runtime provider in this gateway by closing the remaining transport, auth, config, and live-verification work without reopening the already-landed normalization, Anthropic surface, planner/executor, or external-boundary contracts.
- **Why now**: the prior `azure-kimi-claude-gateway` pack landed the normalized event core, Anthropic `/v1/messages` surface, planner/executor policy, and boundary contracts, but the current runtime still treats Azure through a generic OpenAI-compatible path that assumes `Authorization: Bearer ...` auth and appends `/chat/completions` directly in `gateway/src/providers/openai.rs`. That leaves the live Azure Foundry path incomplete even though the rest of the gateway contracts are already frozen.
- **Primary user(s) + JTBD**: gateway operators with real Azure Foundry credentials need to configure and verify the gateway against Azure-hosted Kimi so Claude Code can actually use `Kimi-K2-Thinking` for think/planner traffic and `Kimi-K2.5` for default/execution traffic through the landed Anthropic-compatible gateway surface.
- **In-scope**:
  - Azure `api-key` versus generic bearer-auth handling at the provider/runtime boundary
  - Azure deployment URL shape and request-target resolution
  - `api-version` support in runtime request construction
  - provider/runtime request construction for Azure Foundry while preserving landed `C-02`, `C-03`, `C-04`, `C-05`, and `C-06` contracts
  - config schema and config/example surfaces needed for real Azure runtime setup
  - routing-ready model/provider mapping examples for `Kimi-K2-Thinking` and `Kimi-K2.5`
  - live smoke-test path against real Azure Kimi endpoints
  - operator-facing verification and troubleshooting surfaces for Azure runtime setup
- **Out-of-scope**:
  - re-planning or re-implementing the landed normalization seam unless a fresh stale trigger proves `C-02` no longer matches reality
  - redesigning the Anthropic `/v1/messages` public surface
  - redesigning planner/executor policy or exposing planner/executor as public backend identity
  - speculative downstream Substrate integration work beyond what Azure runtime transport forces directly
  - generic provider refactors that are not required to make Azure Foundry work
- **Success criteria**:
  - the pack identifies a bounded active seam that can make Azure runtime transport concrete instead of leaving it implicit inside the generic OpenAI path
  - the pack identifies a bounded next seam that makes live smoke testing and operator verification first-class outputs rather than post-hoc cleanup
  - the resulting plan preserves the landed `C-02` through `C-06` contracts and uses them as basis instead of reopening them
  - an operator can follow the eventual seam outputs to express Azure auth, deployment URL shape, `api-version`, provider mapping, and Kimi deployment selection without guessing
  - the gateway has an explicit planned path to prove real Azure traffic through `/v1/messages` using `Kimi-K2-Thinking` and `Kimi-K2.5`
- **Constraints**:
  - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway` is the upstream landed basis and should be consumed through closeout-backed truth, not re-planned as greenfield work
  - `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/foundation/substrate-boundary-c05-contract.md`, and `docs/foundation/substrate-structured-events-c06-contract.md` remain authoritative constraints
  - `docs/foundation/claude-code-mux-extension-boundary.md` and `docs/foundation/claude-code-mux-5a372fb-validation.md` still constrain where Azure-specific work may attach and what remained unresolved after the foundation seam
  - `docs/adr/0002-model-azure-kimi-as-a-first-class-provider-normalization-problem.md` and `docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md` prohibit collapsing Azure into a thin generic adapter or hard-coding localhost-only assumptions into the core
  - the practical validation path must run through the landed Anthropic-compatible gateway contract, not through a side-channel provider test that bypasses the public gateway behavior Claude Code consumes
- **External systems / dependencies**:
  - Azure Foundry runtime endpoints hosting `Kimi-K2-Thinking` and `Kimi-K2.5`
  - operator-owned Azure credentials and deployment identifiers
  - Claude Code as the first real client using the landed gateway surface
  - existing gateway provider/config/runtime surfaces in `gateway/src/providers/`, `gateway/src/cli/`, `gateway/src/server/`, and `gateway/config/`
- **Known unknowns / risks**:
  - the exact Azure runtime URL shape, auth header posture, and `api-version` handling required by the live Azure Kimi endpoints may not match the generic current provider assumptions
  - a naive Azure implementation may bleed transport-specific branching into non-Azure OpenAI-compatible providers
  - operator docs/examples may drift from the actual runtime contract if config/schema work and smoke/verification work are split poorly
  - live smoke testing depends on real credentials and deployments that may not be available in normal CI
  - Azure runtime failures may surface as opaque 4xx/5xx errors unless operator-facing diagnostics are made explicit
- **Assumptions**:
  - `SEAM-2` through `SEAM-5` of `azure-kimi-claude-gateway` remain landed and authoritative basis for this follow-on pack
  - the current Azure gap is runtime transport and operator verification, not the normalized event model or Anthropic public surface
  - `Kimi-K2-Thinking` remains the intended think/planner deployment and `Kimi-K2.5` remains the intended default/execution deployment under the landed internal policy contract
  - the default horizon policy still applies here: only one active seam and one next seam, with no deep planning for future seams
