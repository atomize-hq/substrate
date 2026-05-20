# ADR-0041 — Substrate Gateway Backend Adapter Contract (Unified Agent API)

## Status

- Status: Draft
- Supersession note: This ADR supersedes the architectural intent of `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md` while leaving ADR-0024 in place as historical context for the original CLI-backend engine draft and archived planning set.
- Date (UTC): 2026-04-02
- Owner(s): Spenser McConnell (Substrate)

## Scope

- Feature directory: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a successor to ADR-0024 and should be read as a contract clarification for the gateway adapter layer, not as a redefinition of the gateway boundary itself.

- Boundary / runtime ownership prerequisite:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- Config / policy source of truth:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- Output / event / trace foundations:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Gateway evidence:
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md`
- Unified Agent API evidence:
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0013-agent-api-backend-harness.md`
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0015-unified-agent-api-session-extensions.md`
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0017-unified-agent-api-session-thread-id-surfacing.md`
- Handoff evidence:
  - `.codex/handoffs/2026-04-02-144618-substrate-gateway-architecture-alignment.md`
- Follow-on boundary ADR:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 10d71a701199e24edda589d0bac6e8edcaf6c56de561e07c59b90aec3102f0d1

### Changes (operator-facing)

- Replace the bespoke Substrate-local engine assumption with a gateway adapter contract
  - Existing: ADR-0024 frames backend execution as if Substrate should own a distinct `llm-manager` / per-CLI engine layer as the primary implementation path.
  - New: `substrate-gateway` owns a backend-adapter layer that presents stable `<kind>:<name>` backend identities to Substrate and uses capability-driven adapters internally, with Unified Agent API style session/capability semantics preferred for CLI backends.
  - Why: This keeps backend identity stable, avoids coupling policy to provider implementation details, and matches the runtime ownership split clarified in ADR-0040.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0013-agent-api-backend-harness.md`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0015-unified-agent-api-session-extensions.md`
    - `/Users/spensermcconnell/atomize-hq/unified-agent-api/docs/adr/0017-unified-agent-api-session-thread-id-surfacing.md`

## Problem / Context

- ADR-0024 captured the right product goal: subscription-authenticated CLIs should be usable as provider backends without forcing API keys.
- The architecture split has since sharpened:
  - Substrate owns the trusted boundary, backend allowlisting, and world placement / lifecycle.
  - `substrate-gateway` owns the in-world runtime that executes backend adapters.
- The adjacent gateway and Unified Agent API work show a better implementation shape than a bespoke Substrate-local engine layer:
  - one stable backend identity per capability,
  - adapter-level capabilities and session handles,
  - normalized request/event translation,
  - and backend-specific mechanics kept internal to the gateway runtime.
- Without a successor ADR, Substrate risks treating engine internals, provider quirks, or wrapper session details as if they were part of the authoritative policy surface.

## Goals

- Keep backend ids stable in `<kind>:<name>` form and keep selection/allowlisting governed by ADR-0027.
- Define a gateway adapter contract that lets `substrate-gateway` host CLI and API backend adapters without making Substrate aware of per-provider engine seams.
- Prefer Unified Agent API style adapter semantics for CLI backends where available:
  - capability advertising,
  - versioned extension keys,
  - backend-defined session handles,
  - and fail-closed capability gating.
- Preserve the current v1 posture for `cli:codex` as the first required backend adapter while leaving the contract open to additional `cli:*` and `api:*` adapters later.
- Keep normalized structured events and canonical trace vocabulary delegated to ADR-0017 and ADR-0028.

## Non-Goals

- Reintroducing a Substrate-owned `llm-manager` as the primary architecture seam.
- Re-specifying config file families, key paths, precedence, or allowlist semantics already owned by ADR-0027.
- Redefining output rendering, PTY passthrough, or structured event envelope semantics already owned by ADR-0017.
- Redefining canonical trace correlation fields or world-process telemetry already owned by ADR-0028.
- Replacing the boundary / runtime ownership clarification already owned by ADR-0040.
- Defining a public remote or multi-tenant gateway.

## User Contract (Authoritative)

### CLI

- Commands:
  - `substrate world gateway status`: continues to be the authoritative Substrate-owned status surface for gateway availability, routing posture, and backend capability visibility.
  - `substrate world gateway sync`: continues to ensure the in-world gateway runtime is running for the active world session and is the lifecycle entrypoint for adapter-backed operation.
  - `substrate world gateway restart`: continues to be the explicit lifecycle operation for secret rotation or adapter restart.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `2`: invalid configuration, invalid policy, or invalid adapter selection
  - `3`: transient runtime failure
  - `4`: required gateway/backend component unavailable
  - `5`: policy or safety failure

### Config

- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`: operator-controlled selection surface
  2. `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`: allowlists, fail-closed posture, and world-boundary requirements
- Schema:
  - `ADR-0027` remains the authoritative source of truth for key paths, precedence, and defaults.
  - This ADR does not add a new config family.
  - `llm.routing.default_backend` MUST remain a stable `<kind>:<name>` identifier.
  - `llm.allowed_backends` MUST remain deny-by-default and MUST gate adapter selection before runtime execution.
  - Backend inventory remains file-based and one-file-per-backend, with filename/id matching enforced by ADR-0027.
  - Gateway adapter internals, session mode selection, and provider-specific prompt shaping are implementation details of `substrate-gateway`.

### Platform guarantees

- Linux:
  - When worlds are enabled and policy requires in-world execution, backend adapters execute inside the world boundary through `substrate-gateway`.
  - The Substrate-facing backend id remains stable even if the gateway switches internal provider, wrapper, or session strategy.
- macOS:
  - The same backend identity and allowlist contract apply through the macOS world backend.
- Windows:
  - The same backend identity and allowlist contract apply through the Windows world backend.

## Architecture Shape

- Components:
  - `substrate-gateway` runtime: owns the backend adapter registry, request normalization, adapter dispatch, and response/event translation.
  - Gateway adapter implementations: one adapter per backend identity, with capability-driven execution semantics.
  - UAA-style adapter harness concepts: preferred internal shape for CLI backends, including capability validation, versioned session semantics, and backend-defined session handle surfacing.
  - `crates/world-agent`, `crates/broker`, and `crates/trace`: retain the ownership described in ADR-0040 and ADR-0028.
- End-to-end flow:
  - Inputs:
    - Substrate config/policy
    - backend inventory items
    - world availability
    - backend adapter capabilities
    - gateway request payloads
  - Derived state:
    - allowed backend id
    - selected adapter
    - adapter capability set
    - backend/session handle state
    - routing and normalization outcome
  - Actions:
    - Substrate decides whether the backend may run and where it runs
    - `substrate-gateway` resolves the adapter for the chosen backend id
    - the adapter normalizes request/response semantics using capability-driven logic
    - any backend-specific session handle or capability metadata stays inside the gateway runtime contract
    - Substrate consumes only the stable boundary outputs it owns
  - Outputs:
    - dialect response to the client
    - gateway-side normalized backend events
    - Substrate-owned status / wiring / trace records

## Sequencing / Dependencies

- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-gateway-backend-adapter-contract` or the next available gateway backend slot
- Prerequisite integration task IDs:
  - `ADR-0040` must land first so the gateway/runtime ownership split is fixed.
  - `ADR-0027` remains prerequisite for config/policy shape and allowlisting.
  - `ADR-0017` remains prerequisite for structured event routing and output-class separation.
  - `ADR-0028` remains prerequisite for canonical trace and correlation vocabulary.

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
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": false
  },
  "notes": "Contract clarification only; implementation work is intentionally deferred to the gateway/runtime adapter layer."
}
```

<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture

- Fail-closed rules:
  - Backend selection MUST fail closed if `llm.allowed_backends` does not include the requested `<kind>:<name>`.
  - Backend selection MUST fail closed if the adapter is missing, unsupported, or cannot satisfy required capabilities.
  - Substrate MUST not need to trust gateway-local admin or persistence surfaces to authorize backend execution.
  - Secrets must not be stored in Substrate config/policy YAML or exposed through backend identity fields.
- Protected paths/invariants:
  - Stable backend ids are authoritative for allowlisting and routing.
  - Gateway adapter internals, provider quirks, and session mechanics remain inside `substrate-gateway`.
  - Normalized structured events and canonical trace records remain governed by ADR-0017 and ADR-0028.
  - The adapter contract must not reintroduce a second Substrate control plane.

## Validation Plan (Authoritative)

### Tests

- Unit tests:
  - ADR review checklist against the ownership split:
    - Substrate owns selection, allowlisting, lifecycle, and operator-visible status.
    - `substrate-gateway` owns adapter internals and capability-driven request translation.
  - Confirmation that backend ids remain `<kind>:<name>` and do not split into planner/executor/provider ids.
  - Confirmation that capability/session-handle semantics are internal to the gateway adapter contract, not new Substrate policy surfaces.
- Integration tests:
  - Not required for this ADR draft.

### Manual validation

- Compare this ADR against:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - the unified-agent-api UAA ADRs
  - the gateway boundary evidence in `kimi-claude-adapter`
- Confirm there is no remaining ambiguity about who owns:
  - backend identity
  - allowlisting
  - adapter capability discovery
  - session strategy
  - provider normalization
  - operator-visible status

## Rollout / Backwards Compatibility

- This ADR is additive in intent and clarifies the gateway backend adapter contract rather than changing the external backend identity model.
- ADR-0024 should be treated as superseded in architectural intent once this ADR is accepted.
- Existing operator workflows remain valid, but the implementation path is now explicit:
  - Substrate keeps the stable control boundary.
  - `substrate-gateway` hosts adapter internals.
  - Unified Agent API style adapter semantics are preferred for CLI backends where supported.

## Decision Summary

- Decision Register entries (if applicable):
  - None required for this contract clarification draft.
- Options (required; at least two):
  - A) Gateway-hosted, capability-driven backend adapters with Unified Agent API style session semantics.
  - B) Bespoke Substrate-local engine contracts per provider or CLI.
- Selection:
  - Chosen: A
  - Rationale: This preserves stable backend identity, keeps policy surfaces simple, fits the clarified gateway ownership split, and aligns the CLI backend path with the Unified Agent API direction already established in `unified-agent-api`.
