# ADR-0043 — ADR-0027 Identity Tuple Policy Surface

## Status
- Status: Draft
- Date (UTC): 2026-04-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a minimal additive follow-on to ADR-0027. It keeps the existing file families and extends the policy surface so operators can express the ADR-0042 identity model clearly without overloading backend ids.

- Semantic model:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Config/policy foundation:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Expected planning-pack outputs:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/spec_manifest.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/impact_map.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md`
- Event/trace foundations:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Gateway ownership and adapter contracts:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Follow-on agent orchestration ADRs:
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: cdbcd3b47a2dcc20fdce2662742614eb1a8484429895bd7d7052405de93b8cba
### Changes (operator-facing)
- Add tuple-axis policy constraints without creating a new config system
  - Existing: `llm.allowed_backends` and `agents.allowed_backends` gate backend ids, but they do not let operators state router, provider, protocol, or auth-authority constraints independently.
  - New: Substrate adds tuple-axis narrowing constraints under `llm.constraints` so operators can separately constrain `router`, `provider`, `protocol`, and `auth_authority` while keeping backend ids as adapter/backend gates only.
  - Why: This removes the last ambiguity from the operator model without inventing new files or collapsing the router, client, provider, and auth authority into one label.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

## Problem / Context
- ADR-0042 establishes the semantic identity tuple:
  - `client`
  - `router`
  - `provider`
  - `auth_authority`
  - `protocol`
  - plus placement posture (`in_world`, `host_only`) and a transport-only `host_to_world_bridge`
- ADR-0027 already owns the file families, precedence, and fail-closed posture, but its current policy surface only expresses adapter/backend allowlists and world fallback.
- Without additive tuple-axis policy keys, operators are forced to use backend labels to mean too many things at once, especially when:
  - one client can speak multiple protocols,
  - one router can fan out to multiple providers,
  - and the auth authority may be a subscription login state, an API key, or a gateway-delivered secret bundle.
- The goal is to let operators say “this backend is allowed, but only for these providers/protocols/auth authorities” without creating a second config system.

## Goals
- Preserve ADR-0027 as the single config/policy root for LLM and agent work.
- Add minimal additive policy keys that express the tuple axes separately from adapter/backend ids.
- Keep `llm.allowed_backends` and `agents.allowed_backends` intact as backend/adapter gates.
- Keep secrets out of YAML; only names, ids, and references are allowed.
- Preserve fail-closed behavior with no implicit host fallback.
- Keep tuple-axis policy semantics distinct from the existing deny-by-default backend allowlists.

## Non-Goals
- Creating new config or policy files.
- Replacing `llm.allowed_backends` or `agents.allowed_backends`.
- Changing ADR-0042’s semantic model.
- Defining exact wire-level header names or env var names for per-request hints.
- Allowing secrets to be stored in config or policy YAML.

## User Contract (Authoritative)

### CLI
- This ADR introduces no new commands.
- Existing config/policy commands remain the operator surface:
  - `substrate config ...`
  - `substrate policy ...`
- Existing `--explain` and effective-view behavior SHOULD surface the new tuple-axis keys alongside the existing backend allowlists.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (unless explicitly overridden here)
  - This ADR introduces no new exit codes.

### Config
- Files and locations (precedence):
  1. `$SUBSTRATE_HOME/config.yaml`
  2. `<workspace_root>/.substrate/workspace.yaml`
- Policy files and locations (unchanged):
  1. `$SUBSTRATE_HOME/policy.yaml`
  2. `<workspace_root>/.substrate/policy.yaml`
- Schema:
  - No new config files.
  - No new policy files.
  - v1 does not require new config keys to express the tuple; the minimal additive change is policy-side tuple-axis constraints under `llm.constraints`.
  - Existing keys remain authoritative:
    - `llm.allowed_backends`
    - `agents.allowed_backends`
    - `llm.fail_closed.routing`
    - `agents.fail_closed.routing`
    - `llm.secrets.env_allowed`
    - `agents.host_credentials.read.allowed_backends`
    - `net_allowed`
  - New additive policy keys (authoritative; minimal set):
    - `llm.constraints.routers: [string]`
      - Meaning: normalized router ids allowed for LLM fulfillment.
      - Examples: `substrate_gateway`, `direct_provider_path`.
      - Default (effective): `[]`.
      - Constraint semantics:
        - If empty: routing is unconstrained beyond backend/adapter gating and fail-closed posture.
        - If non-empty: the selected effective `router` MUST be in this list.
    - `llm.constraints.providers: [string]`
      - Meaning: normalized upstream provider ids allowed for LLM fulfillment.
      - Examples: `openai`, `anthropic`, `azure_openai`.
      - Default (effective): `[]`.
      - Constraint semantics:
        - If empty: routing is unconstrained beyond backend/adapter gating and fail-closed posture.
        - If non-empty: the selected effective `provider` MUST be in this list.
    - `llm.constraints.protocols: [string]`
      - Meaning: normalized protocol ids allowed for LLM requests.
      - Examples: `openai.responses`, `openai.chat_completions`, `anthropic.messages`.
      - Default (effective): `[]`.
      - Constraint semantics:
        - If empty: routing is unconstrained beyond backend/adapter gating and fail-closed posture.
        - If non-empty: the selected effective `protocol` MUST be in this list.
    - `llm.constraints.auth_authorities: [string]`
      - Meaning: normalized credential or billing authority ids allowed for LLM requests.
      - Examples: `codex_subscription`, `claude_pro_subscription`, `anthropic_api_key`, `gateway_delegated_secret`.
      - Default (effective): `[]`.
      - Constraint semantics:
        - If empty: routing is unconstrained beyond backend/adapter gating and fail-closed posture.
        - If non-empty: the selected effective `auth_authority` MUST be in this list.
  - Canonical tokenization:
    - `router`, `provider`, and `auth_authority` values MUST use normalized lowercase snake_case ids.
    - `protocol` values MUST use normalized lowercase dotted ids.
    - Human-readable labels MAY appear in prose, but policy values and operator-visible status/trace outputs MUST use the normalized ids above.
  - Config additions under `llm.routing` are not required for v1.
    - If a later operator-facing default-provider knob is introduced, it must remain additive and must not replace the policy constraints above.

### Platform guarantees
- Linux:
  - Policy continues to fail closed when `llm.fail_closed.routing=true` and a world boundary is unavailable.
  - Host-side credential reads remain policy-gated and are only a preparation step for Substrate-owned secret delivery.
- macOS:
  - Same policy semantics as Linux.
- Windows:
  - Same policy semantics as Linux.

## Architecture Shape
- Components:
  - `crates/shell`: config/policy surfaces and `--explain` presentation.
  - `crates/broker`: policy evaluation and allow/deny decisions.
  - `crates/world-agent` and world backends: secret delivery and boundary enforcement.
  - `substrate_gateway`: consumes router/provider/protocol/auth-authority constraints when routing requests.
- End-to-end flow:
  - Inputs:
    - client request metadata
    - backend/adapter selection
    - tuple-axis policy constraints
    - `net_allowed`
    - host credential read permissions
  - Derived state:
    - allowed backend/adapter ids
    - allowed router ids
    - allowed provider ids
    - allowed protocol ids
    - allowed auth-authority ids
    - effective placement posture
  - Actions:
    - validate requested router/provider/protocol/auth authority against policy
    - keep `llm.allowed_backends` and `agents.allowed_backends` as backend/adapter gates
    - allow `host_to_world_bridge` only as transport, never as a second control plane
  - Outputs:
    - allow/deny decisions
    - operator-visible provenance via `--explain`
    - canonical trace records with tuple metadata where applicable

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `adr-0027-identity-tuple-policy-surface` (or next available config/policy slot)
- Prerequisite integration task IDs:
  - ADR-0042 must exist before this additive policy surface is finalized.
  - ADR-0027 remains the source of truth for file families, precedence, and fail-closed semantics.
  - ADR-0040 and ADR-0041 remain the gateway ownership and adapter prerequisites.
  - ADR-0017 and ADR-0028 remain the event/trace prerequisites.

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
  "notes": "Minimal additive policy-surface clarification; adds three policy keys under `llm` without changing config file families."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - Empty backend allowlists remain deny-by-default (existing ADR-0027 posture).
  - `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities` are narrowing constraints:
    - when non-empty they MUST be enforced (deny if not matched),
    - when empty they impose no additional restriction beyond backend/adapter gating.
  - `llm.fail_closed.routing` and `agents.fail_closed.routing` still govern host fallback behavior.
  - `host_to_world_bridge` does not create a new standing host gateway.
- Protected paths/invariants:
  - No secrets in YAML.
  - Only names, ids, and references are allowed in policy/config.
  - `net_allowed` remains the network egress boundary at the world layer.
  - `agents.host_credentials.read.allowed_backends` and `llm.secrets.env_allowed` remain the only explicit host-side secret-read gates for this path.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - tuple-axis constraint parsing for routers, providers, protocols, and auth authorities
  - fail-closed behavior with empty backend allowlists and empty tuple-axis constraints
  - precedence between backend/adapter allowlists and tuple-axis constraints
- Integration tests:
  - `substrate config show --explain` and `substrate policy current show --explain` surface tuple-axis keys and provenance
  - denied router/provider/protocol/auth-authority combinations produce clear operator explanations

### Manual validation
- Review the two concrete cases in this ADR:
  - Claude Code pointed at `substrate_gateway` and routed to OpenAI vs Anthropic under policy control.
  - Codex + Responses API + `~/.codex/auth.json`, with `auth_authority` expressed separately from `provider`.

## Rollout / Backwards Compatibility
- This ADR is additive.
- Existing backend allowlists remain in place and keep their adapter/backend gating role.
- Operators who do not add the new tuple-axis constraints keep the current deny-by-default backend posture, with no additional tuple-axis narrowing.
- Future additions must preserve the separation between backend ids and tuple-axis constraints.

## Decision Summary
- Options (required; at least two):
  - A) Keep policy expressed only in terms of backend/adapter ids and rely on overloaded labels or context for router/provider/protocol/auth-authority meaning.
  - B) Add tuple-axis constraints for router, provider, protocol, and auth authority while preserving backend/adapter allowlists for runtime selection.
- Selection:
  - Chosen: B
  - Rationale: this is the minimal additive extension that makes the tuple model operable without changing config file families or introducing a second policy system.
