# ADR-0046 — Realize ADR-0041 Inventory-Backed Backend Selection and Multi-Adapter Integrated Runtime

## Status
- Status: Draft
- Date (UTC): 2026-04-21
- Owner(s): Spenser McConnell (Substrate)

## Stable Curated ADR

- Current stable ADR: `docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope
- Feature directory: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a thin implementation follow-on to ADR-0041. It keeps the ADR-0041 backend-id contract intact and defines how Substrate realizes that contract in the integrated gateway lifecycle for more than `cli:codex`.

- Prerequisite boundary and contract ADRs:
  - `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Config / policy source of truth:
  - `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/reference/policy/contract.md`
  - `docs/reference/policy/schema.md`
- Existing contract docs this ADR realizes:
  - `docs/contracts/gateway/backend-adapter-selection.md`
  - `docs/contracts/gateway/backend-adapter-protocol.md`
  - `docs/contracts/gateway/backend-adapter-schema.md`
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/contracts/gateway/status-schema.md`
- Explicitly deferred follow-ons:
  - `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Expected planning-pack outputs:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/plan.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/tasks.json`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/spec_manifest.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 0c49fe896ae4eebf17fad20a6079dffa13006612a50c40496f0edec500ad8437

### Changes (operator-facing)
- Replace the integrated gateway's single hardcoded `cli:codex` path with inventory-backed backend realization
  - Existing: the integrated lifecycle rejects any default backend other than `cli:codex`, renders one static runtime config, and resolves one Codex-specific auth handoff path.
  - New: the integrated lifecycle resolves the selected backend through config, policy, and inventory, looks up one adapter binding, validates required capabilities fail-closed, and renders runtime config/auth handoff from adapter-owned metadata.
  - Why: ADR-0041 already established stable backend ids and one-backend-id-to-one-adapter semantics, but the current implementation still realizes only the first proof path.
  - Links:
    - `crates/world-service/src/gateway_runtime.rs#L34`
    - `crates/world-service/src/gateway_runtime.rs#L91`
    - `crates/world-service/src/gateway_runtime.rs#L548`
    - `crates/world-service/src/gateway_runtime.rs#L766`
    - `crates/shell/src/builtins/world_gateway.rs#L323`
- Keep tuple-policy, widened status metadata, and secret-channel redesign out of this implementation ADR
  - Existing: the repo has adjacent draft ADRs for identity tuple posture and tuple-axis policy, but those surfaces are not yet implemented and are separable from the immediate backend-selection/runtime gap.
  - New: this ADR limits itself to realizing ADR-0041 selection, adapter lookup, capability gating, and integrated runtime generation, while deferring tuple-axis policy, additive status-schema widening, and stronger secret transport redesign to their owning ADRs.
  - Why: this keeps the active implementation seam small enough to plan and land without reopening adjacent semantic surfaces.
  - Links:
    - `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
    - `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
    - `crates/shell/src/execution/config_model.rs#L215`
    - `crates/shell/src/execution/policy_model.rs#L68`

## Problem / Context
- ADR-0041 established the contract that Substrate selects one stable backend id in `<kind>:<name>` form, applies deny-by-default allowlisting, and hands one allowed backend id to a gateway adapter/runtime boundary.
- The integrated lifecycle and world-service plumbing have landed, but the current implementation is still effectively a single-path proof:
  - `crates/world-service/src/gateway_runtime.rs` hardcodes `cli:codex` as the only supported integrated backend.
  - integrated runtime config generation is static and Codex-specific.
  - integrated auth handoff is Codex-specific.
  - the shell only synthesizes integrated auth payloads for `cli:codex`.
- That leaves a gap between the ADR-0041 contract surface and the integrated runtime behavior:
  - backend selection is not yet inventory-backed at realization time,
  - adapter lookup is not yet a first-class implementation seam,
  - required capabilities are not yet enforced through a bounded adapter contract,
  - and runtime config generation is not yet adapter-driven.
- The immediate planning need is not another contract-only ADR for backend ids. It is a narrow implementation ADR that turns the existing contract into executable behavior without expanding into tuple-policy, status-schema, or secret-channel redesign work.

## Goals
- Realize ADR-0041 backend selection in the integrated gateway lifecycle using the existing config, policy, and inventory surfaces.
- Define one implementation seam where a selected backend id resolves to one adapter binding with explicit required capabilities.
- Replace static Codex-only integrated runtime config rendering with adapter-driven runtime config generation.
- Define a bounded integrated auth handoff model that can support more than `cli:codex` without making auth semantics part of the backend id.
- Keep the operator command family unchanged while making `status`, `sync`, and `restart` valid for any supported integrated backend.

## Non-Goals
- Adding tuple-axis policy keys from ADR-0043.
- Widening `substrate world gateway status --json` to carry ADR-0042 tuple metadata.
- Redesigning host-to-world secret delivery beyond the minimal adapter-specific handoff needed for this seam.
- Making gateway lifecycle automatic in normal execute, stream, or REPL flows.
- Replacing the stable backend-id contract or the one-file-per-backend inventory posture from ADR-0027 / ADR-0041.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate world gateway status`: remains the authoritative operator surface for the integrated gateway lifecycle and now evaluates the currently selected backend through the inventory-backed adapter binding path when integrated gateway mode is enabled.
  - `substrate world gateway sync`: ensures the integrated gateway runtime is realized for the selected backend if that backend is valid, allowed, supported by an integrated adapter, and has a satisfiable auth handoff.
  - `substrate world gateway restart`: restarts the realized integrated runtime for the selected backend using the same adapter binding and auth-handoff rules as `sync`.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `2`: invalid configuration, invalid inventory state, malformed backend selection, or unsupported integrated adapter selection
  - `3`: transient runtime failure while rendering config, launching, probing, or restarting the integrated runtime
  - `4`: required gateway or adapter dependency unavailable
  - `5`: policy or safety failure, including deny-by-default backend gating or blocked host credential read / env read
- Command-family rules:
  - This ADR adds no new top-level operator commands.
  - The selected backend continues to come from the effective config/policy/inventory surfaces rather than a new CLI flag family.
  - The integrated lifecycle MUST distinguish invalid selection, dependency unavailable, and policy denial exactly as ADR-0041 requires.

### Config
- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`: operator-controlled gateway enablement, mode, and default backend selection
  2. `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`: backend allowlists, fail-closed posture, env-read gates, and host-credential-read gates
- Schema:
  - `ADR-0027` remains the authoritative source of truth for key paths, precedence, and defaults.
  - This ADR does not add a new config family.
  - `llm.routing.default_backend` remains the single backend selector for integrated runtime realization.
  - `llm.allowed_backends` remains the deny-by-default backend gate that must pass before adapter dispatch.
  - `llm.secrets.env_allowed` and `agents.host_credentials.read.allowed_backends` remain the policy gates for host-side auth-material sourcing.
  - Backend inventory remains file-based and one-file-per-backend, with filename/id matching enforced before adapter realization.
  - A backend id MUST NOT be realized through an unregistered hardcoded special case outside the integrated adapter registry.

### Platform guarantees
- Linux:
  - When worlds are enabled and policy requires in-world execution, integrated backend realization occurs inside the world boundary through `substrate-gateway`.
  - Any supported integrated backend uses the same selection and allowlist rules as `cli:codex`; only adapter/runtime metadata differs.
- macOS:
  - The same backend-selection and adapter-gating contract applies through the macOS world path, even when lifecycle ownership is still guest-managed.
- Windows:
  - The same backend-selection and adapter-gating contract applies through the Windows world path.

## Architecture Shape
- Components:
  - `crates/shell/src/execution/config_model.rs`: continues to own `llm.routing.default_backend` as the selected backend surface.
  - `crates/shell/src/execution/policy_model.rs` and `crates/broker`: continue to own allowlist and auth-read policy gates.
  - `crates/shell/src/builtins/world_gateway.rs`: resolves the integrated auth payload and passes only allowed, adapter-specific auth material into lifecycle requests.
  - `crates/world-service/src/gateway_runtime.rs`: owns integrated adapter registry lookup, capability gating, runtime config rendering, and process launch for the selected backend.
  - backend inventory and adapter registry metadata: together define whether a backend is realizable, which capability set it requires, and which config/auth renderer it uses.
- End-to-end flow:
  - Inputs:
    - effective `llm.gateway.*` config
    - effective `llm.routing.default_backend`
    - effective backend allowlists and auth-read policy gates
    - backend inventory entry for the selected backend id
    - integrated adapter registry metadata for that backend
  - Derived state:
    - selected backend id
    - inventory-backed backend existence and identity consistency
    - integrated adapter binding
    - required capability set
    - adapter-specific auth handoff kind
    - adapter-specific runtime config payload
  - Actions:
    - validate gateway mode and selected backend
    - resolve the selected backend through inventory and policy
    - resolve exactly one integrated adapter binding for the selected backend id
    - fail closed if the binding is missing, unsupported, or lacks required capabilities
    - synthesize the integrated runtime config and auth payload using adapter-owned rendering logic
    - launch and probe `substrate-gateway` using the rendered config
  - Outputs:
    - one realized integrated runtime for the selected backend
    - typed status / sync / restart results using the existing operator surface
    - adapter-specific config and auth handoff artifacts that stay inside the Substrate-to-gateway runtime boundary

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `gateway-backend-selection-runtime-integration` or the next available gateway implementation slot
- Prerequisite integration task IDs:
  - `ADR-0040` remains prerequisite for the Substrate versus `substrate-gateway` ownership split.
  - `ADR-0041` remains prerequisite for stable backend ids, allowlisting, and one-backend-id-to-one-adapter semantics.
  - `ADR-0027` remains prerequisite for config/policy families and inventory posture.
  - `ADR-0042` and `ADR-0043` are explicit follow-ons, not prerequisites for this implementation ADR.

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 10,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 4,
    "boundary_crossings": 3
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 2, "new_test_cases": 12 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 1, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": false
  },
  "notes": "Implementation-focused follow-on to ADR-0041. Primary lift is in shell/world-service adapter realization and verification, not in new config or operator-surface design."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - Integrated runtime realization MUST fail closed if the selected backend id is malformed, unknown, inventory-inconsistent, or not allowlisted.
  - Integrated runtime realization MUST fail closed if no integrated adapter binding exists for the selected backend id.
  - Integrated runtime realization MUST fail closed if the adapter binding declares required capabilities that are unavailable or unsatisfied.
  - Integrated runtime realization MUST fail closed if required auth handoff material is unavailable or blocked by policy.
- Protected paths/invariants:
  - Stable backend ids remain the only authoritative backend selectors at the Substrate boundary.
  - Adapter-specific provider names, models, and auth semantics remain internal runtime metadata and must not become policy selectors by accident.
  - Gateway-local config or persistence must not authorize execution.
  - This ADR must not smuggle ADR-0042 tuple semantics or ADR-0043 tuple-policy keys into the backend-id realization path.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - `crates/world-service/src/gateway_runtime.rs`:
    - selected backend resolves through one adapter binding
    - unsupported backend ids fail with invalid-integration classification
    - missing adapter bindings fail with dependency-unavailable classification
    - adapter-specific config rendering is snapshot-tested for more than one backend
    - adapter-specific auth handoff validation fails closed
  - `crates/shell/src/builtins/world_gateway.rs`:
    - integrated auth payload resolution is backend-aware
    - env-read and host-credential-read gates remain enforced per backend
- Integration tests:
  - `crates/world-service/tests/gateway_runtime_parity.rs`:
    - `status -> unavailable`, `sync -> available`, `status -> available` works for at least one non-`cli:codex` integrated backend fixture
    - restart semantics preserve backend binding and runtime-id stability
  - shell command-path tests:
    - effective `llm.routing.default_backend` changes drive backend realization without new CLI flags

### Manual validation
- Validate on Linux first:
  - set `llm.routing.default_backend` to `cli:codex` and confirm no regression in `status`, `sync`, and `restart`
  - set `llm.routing.default_backend` to a second supported integrated backend and confirm the lifecycle realizes that backend through the same command family
  - confirm invalid backend, blocked backend, and missing-adapter cases land in distinct exit-code buckets
- Validate cross-platform command reachability on macOS and Windows using the same selected-backend matrix, without requiring new operator commands.

### Manual playbook (if required)
- Manual playbook: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`

### Smoke scripts (if required)
- Linux: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- `cli:codex` remains the first required integrated backend and must remain the regression baseline throughout rollout.
- This ADR is additive to ADR-0041:
  - it does not change backend-id grammar,
  - it does not change config or policy file families,
  - and it does not require a new operator command family.
- Backwards-compatible rollout means:
  - existing `cli:codex` behavior keeps working,
  - additional integrated backends become realizable behind the existing selection and allowlist surfaces,
  - and unsupported backends fail explicitly instead of silently collapsing back to Codex-specific behavior.

## Decision Summary
- Decision Register entries (if applicable):
  - None required for this ADR draft.
- Options (required; at least two):
  - A) Add a narrow implementation ADR that realizes ADR-0041 through inventory-backed adapter binding, capability gating, and adapter-driven runtime config generation.
  - B) Reopen ADR-0041 and absorb the remaining implementation details, tuple-policy questions, and status-schema follow-ons into one larger planning body.
- Selection:
  - Chosen: A
  - Rationale: the contract truth already exists in ADR-0041 and the closed seam pack. The open work is a bounded implementation seam, so the repo benefits more from a thin follow-on ADR than from reopening settled contract planning.
