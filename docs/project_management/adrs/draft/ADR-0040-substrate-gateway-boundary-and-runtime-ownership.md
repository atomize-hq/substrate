# ADR-0040 — Substrate Gateway Boundary and Runtime Ownership

## Status
- Status: Draft
- Supersession note: This ADR supersedes the architectural intent of `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md` while leaving ADR-0023 in place as historical context for the original gateway-capability draft and archived planning set.
- Date (UTC): 2026-04-02
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a successor to ADR-0023 and should be read as an ownership clarification, not a rewrite of the
underlying gateway capability.

- Superseded intent:
  - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
- Foundational config/policy surface:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Foundational output/routing and trace contracts:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Evidence from the adjacent gateway runtime:
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md`
- Handoff evidence:
  - `.codex/handoffs/2026-04-02-144618-substrate-gateway-architecture-alignment.md`
- Follow-on engine/backend layer ADR:
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: f912ab8be5245a70ba603bc6547c3b62e05c59492af2c7c9d27c3d898f664a50

### Changes (operator-facing)
- Clarify the runtime boundary between Substrate and `substrate-gateway`
  - Existing: ADR-0023 describes the gateway as if Substrate owns the full in-world runtime contract, including operational concerns that now belong to the gateway implementation itself.
  - New: Substrate owns the trusted control boundary - policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing - while `substrate-gateway` owns the in-world front door, provider/planner/executor internals, and normalized event generation.
  - Why: This matches the current architecture split and prevents gateway internals from leaking into Substrate policy or config surfaces.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
    - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
    - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`
    - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`

## Problem / Context
- The original ADR-0023 framing was directionally correct, but the runtime split has become clearer:
  - Substrate is the owner of the trust boundary and the operator-facing contract.
  - `substrate-gateway` is the runtime that executes inside that boundary.
- The adjacent gateway work already codifies this split:
  - one stable external backend identity to Substrate,
  - an in-world-compatible deployment boundary,
  - and normalized structured events instead of raw provider streams.
- Without an explicit successor ADR, the repo risks reintroducing a "second control plane" pattern where gateway-local configuration, local admin endpoints, or internal trace behavior become de facto Substrate contracts.

## Goals
- Make the Substrate / `substrate-gateway` ownership split explicit and durable.
- Preserve the existing Substrate contract around policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing.
- Preserve the gateway contract around a single stable backend identity, in-world compatibility, internal provider/planner/executor routing, and normalized structured event generation.
- Define how current standalone gateway concerns are governed when the gateway is integrated with Substrate.

## Non-Goals
- Defining the full backend engine contract for `ADR-0024`.
- Re-specifying the config/policy key surface already owned by `ADR-0027`.
- Replacing the output/routing contract already owned by `ADR-0017`.
- Replacing the canonical trace vocabulary already owned by `ADR-0028`.
- Designing a public remote or multi-tenant gateway.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate world gateway sync`: Substrate ensures the in-world gateway is running for the active world session, performs policy-gated secret delivery, and exposes operator-visible wiring/status.
  - `substrate world gateway status`: Substrate reports gateway availability, policy posture, and client wiring in a Substrate-owned format.
  - `substrate world gateway restart`: Substrate restarts the gateway as an explicit lifecycle operation, including secret rotation flows.
  - `substrate world gateway status --json`: structured Substrate-owned status output; the authoritative operator surface for gateway wiring.
- Client wiring contract:
  - `substrate world gateway status --json` is the authoritative Substrate-owned wiring surface and MUST include non-secret `client_wiring.*` fields for the gateway endpoints Substrate wants operators and in-world clients to use.
  - Human-readable wiring output MAY be abbreviated by default, but `substrate world gateway status` MUST remain the stable operator entrypoint for discovering gateway wiring.
  - Stable non-secret wiring env var names remain:
    - `SUBSTRATE_LLM_OPENAI_BASE_URL`
    - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
  - These values point to Substrate-managed gateway endpoints, not upstream provider endpoints.
  - These base URLs are intended for in-world reachability (clients/backends executing inside the world boundary), not as a guarantee of direct host reachability.
- Secret delivery contract boundary:
  - This ADR intentionally preserves only the ownership rule that Substrate owns policy-gated host secret sourcing and host-to-world secret delivery for integrated operation.
  - Exact secret transport mechanics, canonical auth field naming, and compatibility-path details remain governed by ADR-0027 and the referenced gateway secret-delivery/decision docs rather than being redefined here.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (unless explicitly overridden here)
  - `0`: success
  - `2`: invalid configuration, invalid policy, or invalid integration state
  - `3`: transient runtime failure
  - `4`: required gateway/world component unavailable
  - `5`: policy or safety failure

### Config
- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`: operator-controlled config patches for enabling and selecting gateway behavior
  2. `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`: policy patches governing allowlists, fail-closed posture, and world-boundary requirements
- Schema:
  - `ADR-0027` remains the authoritative source of truth for key paths, precedence, and defaults.
  - This ADR does not add a new config family.
  - Gateway-local config files, admin mutation surfaces, and local persistence paths are not Substrate contract surfaces.
  - Any gateway-local config or admin surface is implementation-specific to `substrate-gateway` and must not be required for Substrate-managed operation.
  - Host-side secret-read gates and backend-selection constraints continue to come from ADR-0027, including:
    - `llm.secrets.env_allowed`
    - `agents.host_credentials.read.allowed_backends`

### Platform guarantees
- Linux:
  - When worlds are enabled and policy requires in-world execution, the gateway runs inside the world boundary.
  - Substrate-owned secret delivery is the only trusted path for integrated secret material.
- macOS:
  - The gateway is integrated through the macOS world backend and remains subject to the same Substrate-owned boundary contract.
- Windows:
  - The gateway is integrated through the Windows world backend and remains subject to the same Substrate-owned boundary contract.

## Architecture Shape
- Components:
  - `crates/world-agent` and the world backend layers: own lifecycle orchestration, placement, and transport into the world boundary.
  - `crates/broker` and config/policy resolution: own allowlists, fail-closed posture, and policy explanation.
  - `crates/trace`: owns canonical trace persistence and the Substrate trace vocabulary.
  - `substrate-gateway` runtime: owns the in-world gateway front door, provider normalization, planner/executor routing, and normalized event generation.
- End-to-end flow:
  - Inputs:
    - Substrate config/policy
    - world availability
    - Substrate-owned secret delivery payloads
    - gateway HTTP requests
  - Derived state:
    - allowed backend selection
    - world/session placement
    - request routing and normalized event emission
    - Substrate-canonical trace records
  - Actions:
    - Substrate decides whether the gateway may run and where it runs
    - `substrate-gateway` processes requests inside the world boundary
    - `substrate-gateway` emits normalized structured events
    - Substrate records canonical trace and exposes operator status/diagnostics
  - Outputs:
    - in-world gateway responses
    - normalized gateway events
    - Substrate-owned status, wiring, and trace records

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-gateway-boundary-ownership` or the next available gateway boundary slot
- Prerequisite integration task IDs:
  - `ADR-0027` remains prerequisite for config/policy shape
  - `ADR-0017` remains prerequisite for structured event routing and output-class separation
  - `ADR-0028` remains prerequisite for canonical trace and correlation vocabulary
  - `ADR-0024` must be rewritten or superseded after this boundary ADR lands so the engine/backend layer reflects the new ownership split

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 0,
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
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": false
  },
  "notes": "Boundary clarification only; implementation work is intentionally deferred to the engine/backend ADR and runtime follow-up."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - Substrate must not fall back to a host-level gateway when world placement is required and policy demands in-world execution.
  - Secret delivery for integrated operation must remain policy-gated and Substrate-owned.
  - Gateway-local token persistence or admin mutation surfaces must not become required trust inputs for Substrate-managed operation.
- Protected paths/invariants:
  - Substrate canonical trace is authoritative for operator-facing audit and correlation.
  - `substrate-gateway` may maintain its own internal tracing for development or implementation purposes, but that tracing is not the operator contract and must not replace Substrate trace semantics.
  - Gateway internal config/admin surfaces, if any, are implementation details and must not leak into Substrate config semantics.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - ADR review checklist against the ownership split:
    - Substrate owns policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing.
    - `substrate-gateway` owns runtime internals and normalized event generation.
  - Confirmation that `ADR-0027`, `ADR-0017`, and `ADR-0028` remain the only normative sources for their respective contracts.
- Integration tests:
  - Not required for this ADR draft.

### Manual validation
- Compare this ADR against:
  - `.codex/handoffs/2026-04-02-144618-substrate-gateway-architecture-alignment.md`
  - the three gateway boundary ADRs in `kimi-claude-adapter`
  - the Substrate config/policy, output routing, and trace ADRs
- Confirm there is no remaining ambiguity about who owns:
  - policy
  - world placement
  - lifecycle
  - secret delivery
  - operator UX
  - canonical tracing

## Rollout / Backwards Compatibility
- This ADR is additive in intent and clarifies ownership rather than changing the gateway capability itself.
- ADR-0023 should be treated as superseded in architectural intent once this ADR is accepted.
- Existing operator workflows remain valid, but their ownership is now explicit:
  - Substrate-owned surfaces remain the source of truth for integrated use.
  - gateway-local standalone conveniences remain gateway implementation concerns.

## Decision Summary
- Decision Register entries (if applicable):
  - None required for this boundary clarification draft.
- Options (required; at least two):
  - A) Substrate owns the trusted boundary and `substrate-gateway` owns the in-world runtime internals.
  - B) `substrate-gateway` owns a broader control surface and Substrate consumes it as a client.
- Selection:
  - Chosen: A
  - Rationale: This matches the current architecture evidence, keeps trust and policy anchored in Substrate, and prevents gateway implementation details from becoming part of the Substrate contract surface.
