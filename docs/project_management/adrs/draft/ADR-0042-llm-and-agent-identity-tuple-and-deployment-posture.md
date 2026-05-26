# ADR-0042 — LLM and Agent Identity Tuple and Deployment Posture

## Status
- Status: Draft
- Date (UTC): 2026-04-02
- Owner(s): Spenser McConnell (Substrate)

## Stable Curated ADR

- Current stable ADR: `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a semantic lock for operator-facing identity and deployment posture. It should be read as a
clarification layer that precedes later Agent Hub updates and any additive config/policy updates.

- Foundational config/policy surface:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/reference/policy/contract.md`
  - `docs/reference/policy/schema.md`
- Semantic planning pack:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- Foundational output/event and trace contracts:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Gateway ownership and adapter contracts:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Additive policy follow-on:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Follow-on agent orchestration ADRs:
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Supporting evidence:
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 51fd84175744539955168c2a9aa657d1fc69c9923a9a84295c22d8697847df20
### Changes (operator-facing)
- Replace overloaded backend-id interpretation with an explicit identity tuple
  - Existing: operator-facing docs and traces can accidentally collapse client origin, router authority, upstream provider, auth authority, and protocol shape into one overloaded backend label.
  - New: Substrate treats these as distinct semantic fields: `client`, `router`, `provider`, `auth_authority`, and `protocol`.
  - Why: A single backend id is not enough to explain what is actually happening when a host client is pointed at `substrate_gateway`, when the gateway fans out to multiple providers, or when subscription-based auth and API-key auth both exist in the same ecosystem.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
    - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

## Problem / Context
- The current architecture has multiple, orthogonal choices that operators need to understand independently:
  - which client/runtime originated the request,
  - which router decided fulfillment,
  - which provider actually served the request,
  - which auth authority made the request permissible or billable,
  - and which protocol/capability surface was being spoken.
- If these are collapsed into one backend label, the operator has to infer meaning from context, which is exactly the confusion we want to remove.
- The tuple is especially important because the same client can route to different providers, and the same router can accept different auth authorities and protocol surfaces across requests.

## Goals
- Lock a stable operator-facing identity model with five explicit fields:
  - `client`
  - `router`
  - `provider`
  - `auth_authority`
  - `protocol`
- Make it clear that `host_to_world_bridge` is transport-only, not a second router or second permanent gateway.
- Preserve the existing Substrate ownership split:
  - Substrate owns policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing.
  - `substrate_gateway` owns runtime routing and fulfillment inside the boundary.
- Keep per-request routing hints explicit and policy-gated.

## Non-Goals
- Defining a new public config family.
- Replacing ADR-0027, ADR-0017, ADR-0028, ADR-0040, or ADR-0041.
- Reintroducing a second permanent host gateway as a peer control plane.
- Specifying exact wire-level header or environment variable names for future request hints.

## User Contract (Authoritative)

### Identity tuple
The operator-visible identity tuple is:

- `client`
  - Meaning: the originating runtime or caller surface that issued the request.
  - Examples: `claude_code`, `codex`, future UAA-backed clients.

- `router`
  - Meaning: the routing authority that accepted the request and selected fulfillment.
  - Example: `substrate_gateway` as the canonical routing boundary.

- `provider`
  - Meaning: the upstream service that actually fulfilled the request.
  - Examples: `openai`, `anthropic`, `azure_openai`.

- `auth_authority`
  - Meaning: the credential or billing authority under which the request was authorized.
  - Examples: a subscription login state, an API key, or a gateway-delegated secret bundle.

- `protocol`
  - Meaning: the request/response contract being spoken.
  - Examples: OpenAI Responses, OpenAI-compatible chat surfaces, Anthropic Messages, or UAA-style capability/session semantics.

Operator-facing rules:
- The tuple fields are metadata-only and must not be used as a heuristic join system.
- `auth_authority` is distinct from both `client` and `provider`.
- `protocol` is capability/contract metadata and should be visible in traces, status output, and diagnostics.
- No secrets should be emitted into trace by default.
- The tuple does not replace the canonical correlation vocabulary and join keys in ADR-0028.

Canonical token format:
- `client`, `router`, `provider`, and `auth_authority` MUST use normalized lowercase snake_case ids.
  - Examples:
    - `client=claude_code`
    - `router=substrate_gateway`
    - `provider=azure_openai`
    - `auth_authority=codex_subscription`
- `protocol` MUST use a normalized lowercase dotted id, optionally ending with a version token.
  - Examples:
    - `openai.responses`
    - `anthropic.messages`
    - `substrate.agent.session`
    - `mcp.toolbox.v1`
- Human-readable prose labels MAY appear in surrounding text, but operator-visible status, policy, and trace surfaces MUST use the normalized ids above.

### Deployment / placement posture
The placement model has two postures and one transport adjunct:

- `in_world`
  - The canonical fulfillment posture.
  - `substrate_gateway` runs inside the world boundary when worlds are enabled and policy requires in-world execution.

- `host_only`
  - A fallback posture for host-only environments or explicit policy-permitted host execution.
  - This is a deployment mode, not a second permanent gateway.

- `host_to_world_bridge`
  - A transport bridge used when host-scoped control surfaces must reach in-world resources.
  - This is not a router identity and not a second control plane.

Non-negotiable interpretation:
- We do not run a second permanent host gateway alongside the in-world gateway.
- If host execution is permitted, it is a mode of the router, not a separate standing authority.
- If host-side orchestration needs to reach in-world fulfillment, it does so through bridge transport, not by duplicating gateway ownership.

### Routing hints
- A per-request routing hint may request a provider selection.
- The router validates that request against policy and capability.
- If the hint is accepted, it influences the effective provider selection.
- If the hint is rejected, it does not change the `client` identity and does not become an implicit provider authority.

### CLI
- This ADR introduces no new commands.
- Existing status/wiring surfaces (owned elsewhere) SHOULD be extended additively to make the tuple and placement posture operator-visible:
  - `substrate world gateway status --json`: report the effective placement posture and router identity; include capability and protocol hints where safe and available.
  - `substrate world gateway sync`: remains the lifecycle entrypoint for ensuring the in-world router is available and for policy-gated secret delivery.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - This ADR introduces no new exit codes.

### Config
- This ADR introduces no new config files and no new config keys.
- Source of truth for config/policy files, precedence, and fail-closed semantics remains:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Router/provider/protocol/auth-authority policy constraints are introduced additively by:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Placement posture is expressed via existing config/policy:
  - `llm.gateway.mode: in_world|host_only`
  - `llm.fail_closed.routing` (no host fallback when true)
  - `agents.host_credentials.read.allowed_backends` and `llm.secrets.env_allowed` (policy gates for host-side credential reads and host env reads used in host-to-world secret delivery)

### Platform guarantees
- Linux/macOS/Windows:
  - The tuple semantics are the same across platforms.
  - Default posture remains world-first and fail-closed when `*.fail_closed.routing=true` (per ADR-0027).
  - A `host_to_world_bridge` (when present) is transport-only and must not change whether egress is governed by in-world `net_allowed` enforcement.

### Concrete examples

#### Claude Code pointed at `substrate_gateway`
- `client`: `claude_code`
- `router`: `substrate_gateway`
- `provider`: may vary by request or config, such as `openai`, `anthropic`, or another supported provider
- `auth_authority`: Claude subscription login state, Claude gateway token, or API credential path depending on the chosen mode
- `protocol`: the surface being spoken, such as Anthropic Messages or compatible gateway protocol semantics

Why this matters:
- The operator can point Claude Code at one router and still have multiple possible upstream providers.
- The client stays `claude_code` even when the provider changes.
- The router is the authority that decides whether the provider request is allowed.

#### Codex using Responses API and `~/.codex/auth.json`
- `client`: `codex`
- `router`: `substrate_gateway` or `direct_provider_path`, depending on deployment posture and policy
- `provider`: `openai`
- `auth_authority`: Codex subscription auth, derived from `~/.codex/auth.json`, or another approved OpenAI credential authority
- `protocol`: `openai.responses`

Why this matters:
- The Responses API is a protocol choice, not the same thing as client identity.
- The credential source in `~/.codex/auth.json` is an auth authority, not the provider itself.
- The same client can remain `codex` while the effective provider/protocol/auth path is made explicit.
- If direct provider routing is permitted, that permission must be explicit in policy; it must not be inferred from the provider or protocol alone.

### Config / policy implications
- `ADR-0027` remains the source of truth for where config/policy lives and how fail-closed gating works.
- This ADR does not define new file families or a new config root.
- Any future additive config/policy update should keep the tuple fields distinct rather than overloading a single backend label.
- Host credential reads and secret delivery remain policy-gated by ADR-0027 and the gateway contracts.

## Architecture Shape
- Components:
  - `substrate_gateway`: routing authority and in-world fulfillment boundary.
  - `crates/world-service` and world backends: transport and lifecycle into the world boundary.
  - `crates/broker`: policy gating and explainability.
  - `crates/trace`: canonical trace persistence and correlation vocabulary.
  - `crates/shell`: operator surfaces for status, wiring, and diagnostics.

- End-to-end flow:
  - A client issues a request.
  - The router evaluates placement posture, protocol, requested provider, auth authority, and policy.
  - The router either fulfills in-world, fulfills host-only, or rejects the request.
  - The tuple is recorded as metadata for operator visibility.
  - The trace layer preserves canonical joins and redaction rules from ADR-0017 and ADR-0028.

- Boundary rule:
  - `host_to_world_bridge` is transport-only.
  - It may connect host-scoped control surfaces to world-scoped fulfillment.
  - It must not become a second router or a parallel canonical gateway.

## Security / Safety Posture
- Fail-closed rules:
  - No heuristic joins across the tuple.
  - No secrets in trace by default.
  - `auth_authority` must be explicit when a request path depends on subscription state or credentials.
  - Provider selection hints are only requests until policy validates them.

- Protected invariants:
  - The tuple fields are metadata, not a join replacement for ADR-0017/ADR-0028 correlation fields.
  - `client` cannot be inferred from `provider`.
  - `provider` cannot be inferred from `auth_authority`.
  - `protocol` cannot be used to smuggle authority or bypass routing policy.
  - A host-only posture does not imply a second standing host gateway.
  - `router` cannot be inferred from `provider`, `protocol`, or deployment posture; if multiple routers are permitted, policy must say so explicitly.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-and-agent-identity-tuple-and-deployment-posture` (or next available identity-tuple slot)
- Prerequisite integration task IDs:
  - This ADR must land before the successor rewrite/supersession of `ADR-0025` (Agent Hub core) so agent orchestration identity and tool gating do not overload backend ids.
  - This ADR should land before any additive updates to ADR-0027 that introduce operator-facing tuple visibility in config/policy or status output.
- Prerequisites and dependencies:
  - `ADR-0040` and `ADR-0041` for the gateway ownership and adapter split.
  - `ADR-0027` for fail-closed config/policy plumbing, host credential reads, and secret delivery rules.
  - `ADR-0017` and `ADR-0028` for event-plane and trace vocabulary alignment.

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 0,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 1
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": false
  },
  "notes": "Semantic lock only; no implementation work is required by this ADR."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Validation Plan (Authoritative)
### Tests
- Unit tests: not required for this ADR (semantic clarification only).
- Integration tests: not required for this ADR (semantic clarification only).

### Manual validation
- Review against the concrete cases in this ADR:
  - Claude Code pointed at `substrate_gateway`.
  - Codex using Responses API with `~/.codex/auth.json`.
- Confirm that every tuple field has a distinct operator meaning and that none of them collapse into another field.
- Confirm that the host-to-world bridge is described only as transport.
- Confirm that no new config family or new public root is introduced.

## Rollout / Backwards Compatibility
- This ADR is a semantic clarification, not an implementation change.
- Existing historical docs that used overloaded backend labels can remain as historical context, but new operator-facing language should use the tuple.
- Later ADRs may add additive config/policy or trace visibility, but they must preserve the separation between client, router, provider, auth authority, and protocol.

## Decision Summary
- Options (required; at least two):
  - A) Keep overloaded backend ids as the primary operator story, rely on context and per-backend heuristics.
  - B) Lock an explicit five-part identity tuple plus a transport-only `host_to_world_bridge` concept for placement and reachability.
- Selection:
  - Chosen: B
  - Rationale: this is the minimum shape that keeps operator reasoning stable when one client can speak multiple protocols, route to multiple providers, and authenticate through different authorities without creating a second permanent gateway.
