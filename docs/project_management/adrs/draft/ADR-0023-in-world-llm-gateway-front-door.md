# ADR-0023 — In-World Substrate LLM Gateway (Front Door + Engines)

## Status
- Status: Draft
- Supersession note: Architectural intent is superseded by `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`. Keep this ADR only as the original gateway-capability draft and historical context for the archived `llm_gateway_in_world` planning set.
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/_archived/next/llm_gateway_in_world/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Successor ADR:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- Plan: `docs/project_management/_archived/next/llm_gateway_in_world/plan.md`
- Tasks: `docs/project_management/_archived/next/llm_gateway_in_world/tasks.json`
- Spec manifest: `docs/project_management/_archived/next/llm_gateway_in_world/spec_manifest.md`
- Specs: `docs/project_management/_archived/next/llm_gateway_in_world/specs/*`
- Contract (if present): `docs/project_management/_archived/next/llm_gateway_in_world/contract.md`
- Decision Register: `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md`
- Impact Map: `docs/project_management/_archived/next/llm_gateway_in_world/impact_map.md`
- Manual Playbook: `docs/project_management/_archived/next/llm_gateway_in_world/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 5dfe844de3d4fc36e3bc0d9a4ae7c7f0b79dacfcb7dc8e918261738fc318ec83

Supersession note:
- Read `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md` as the current architectural source of truth for Substrate versus `substrate-gateway` ownership. This ADR remains useful for the original gateway-capability shape and archived execution planning, but it no longer defines the runtime ownership boundary.

### Changes (operator-facing)
- Substrate-owned LLM gateway runs inside the world boundary
  - Existing: CLI agents and tooling may egress directly, or egress may transit a host-level proxy with unclear policy boundary.
  - New: When `SUBSTRATE_WORLD=enabled`, Substrate runs an in-world LLM gateway (“front door”) and all LLM egress is performed from inside the world boundary.
  - Why: Prevent “policy bypass via localhost” and keep enforcement/audit claims accurate (no silent relocation of egress to host).
  - Links:
    - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md#L1`

## Problem / Context
- Substrate’s posture is “secure execution layer between agents and the machine” (isolation + audit + centralized policy). If LLM egress is moved to host-level components, egress enforcement can silently escape the world boundary.
- A host-level gateway reachable as `127.0.0.1` from inside the world collapses all outbound targets to “localhost,” moving real egress outside the sandbox and undercutting the security story.
- We need a Substrate-owned LLM gateway that is:
  - in-world by default,
  - small-footprint and easy to supervise,
  - able to serve multiple client dialects (OpenAI-style and Anthropic-style),
  - pluggable behind a stable internal contract for Tauri, hooks, and graph ingestion.

## Goals
- Provide a Substrate-managed LLM gateway (“front door”) that runs inside the world boundary whenever worlds are enabled.
- Preserve the security boundary: LLM egress occurs within world network enforcement posture.
- Expose OpenAI-compatible and Anthropic-compatible HTTP surfaces for client CLIs and SDKs.
- Emit a stable, structured event/span contract for every LLM request (IDs, policy decision, backend kind, timing, usage when available).
- Keep footprint small: no DB required by default; JSONL logs only; optional metrics behind feature flags.

## Non-Goals
- Replacing or re-implementing third-party CLIs (Codex/Claude Code/Gemini CLI remain first-class clients/backends).
- Implementing advanced optimization/observability stacks (e.g. ClickHouse pipelines) in v1.
- Providing a public remote/multi-tenant gateway (v1 is local/in-world only).
- Guaranteeing perfect cross-dialect feature parity for every provider-specific field (compatibility is capability-gated).
- Enumerating a canonical “backend registry” list in this ADR (defer to ADR-0024/ADR-0025; add references during Phase 8 circle-back once those contracts are accepted).

## User Contract (Authoritative)

### CLI
- Commands:
  - Gateway lifecycle is owned by the world subsystem (session world management + deps provisioning).
    - v1 requirement (legacy): the “ensure gateway running” path MUST pass any required secret values to the in-world gateway/engine spawn request over the existing world-agent transport (see `docs/project_management/_archived/next/llm_gateway_in_world/specs/env_injection.md`).
    - Phase 8 additive upgrade (preferred): secret values MUST NOT live in the in-world gateway/manager process environment by default. Host→world secret delivery SHOULD use a secret-channel payload and an inherited one-time FD/pipe bundle into the gateway/manager (see `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md` (DR-0018)).
  - `substrate world status gateway`:
    - Behavior: prints per-world-session gateway state (running/not), bind endpoints inside the world, active backend kind, and policy mode.
    - Default output: status + health only (no client wiring exports by default).
    - `--debug`: includes a stable “client wiring” section that shows how to route OpenAI/Anthropic-compatible clients through the in-world gateway (base URLs / exports).
      - v1 env var names (values point to Substrate gateway, not upstream providers):
        - `SUBSTRATE_LLM_OPENAI_BASE_URL`
        - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
      - Reachability: these base URLs are intended to be reachable from inside the world boundary (in-world clients/backends), not directly from the host.
    - `--json`: prints structured JSON including client wiring fields (non-secret).
    - Exit codes: `0` success; `4` gateway not available; `5` world required but not available (fail-closed).
  - `substrate world sync gateway`:
    - Behavior: idempotently ensure the in-world gateway is running for the current world session; performs secret delivery as needed (legacy env injection or v1.1 FD/pipe auth bundle).
    - Output: on success, prints a status/health summary by default.
      - `--debug`: prints the same client wiring section as `substrate world status gateway --debug`.
      - `--json`: includes the same `client_wiring.*` fields as `substrate world status gateway --json`.
    - Exit codes: same taxonomy as `substrate world status gateway` with `3` reserved for transient start failures.
  - `substrate world sync gateway --restart`:
    - Behavior: restart the gateway for the current world session; used for secret rotation.

- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `2`: user input/usage error (invalid flags, invalid config)
  - `3`: recoverable runtime error (transient failure; retry may succeed)
  - `4`: required component unavailable (gateway not running; missing binary)
  - `5`: policy/guardrail failure (fail-closed: world required; enforcement cannot be guaranteed)

### Config
This ADR does not define new config file families or bespoke gateway config files. It MUST use the Phase 3 surface defined by ADR-0027.

- Source of truth (config/policy key paths + precedence + defaults):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/_archived/next/llm_and_agent_config_policy_surface/SCHEMA.md`

Files and locations (existing YAML layering model):
- Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
- Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`

Minimum required config keys (from ADR-0027):
- `llm.enabled: bool` (default: `false`)
- `llm.gateway.enabled: bool` (default: `false`)
- `llm.gateway.mode: in_world|host_only` (default: `in_world`; `host_only` permitted only when policy allows host fallback)
- `llm.routing.default_backend: <kind>:<name>` (default: empty; no implicit backend selection)

Operational parameters (bind endpoints, logging destinations, metrics) are intentionally NOT part of the ADR-0027 config surface in v1.
- `substrate world status gateway --json` (and `--debug` for human output) is the authoritative “client wiring” output (base URLs / exports) and may change implementation details (ports/transports) without reshaping config.

### Correlation + trace contract (Phase 8 alignment; authoritative spines)

This ADR intentionally does not re-define correlation vocabulary or envelope semantics. All persisted records MUST explicitly defer to the Phase 8 spines:
- Canonical correlation vocabulary + required/optional matrix: ADR-0028 (`docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`)
- Structured event framing/routing attribution (envelope) when applicable: ADR-0017 (`docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`)

Non-negotiable rules (Phase 8):
- No heuristic correlation joins: the gateway/manager/engines MUST NOT guess or derive join keys from request bodies, prompt content, or transport details; they MUST either mint correlation ids or accept them from a trusted integration boundary (and otherwise omit them).
- `backend_id` is the only backend-selection identifier (`<kind>:<name>`); it MUST be persisted on request records when a concrete backend is selected and MUST NOT be inferred from `agent_id`.
- `agent_id` is attribution for the actor/principal (when the gateway is invoked on behalf of an agent); it MUST be treated as distinct from backend selection.
- `channel` (if present on any structured event envelope emitted alongside gateway operations) is a routing hint only: safe/capped/non-secret; never a join key; never used for policy gating.

### Platform guarantees
- Linux:
  - Gateway runs inside the world network namespace; outbound egress is subject to world-level enforcement.
  - Fail-closed: when effective policy has `llm.fail_closed.routing=true`, do not fall back to a host-level gateway when `SUBSTRATE_WORLD=enabled`.
- macOS:
  - Gateway runs inside the Lima guest world. Host talks to it via the existing Substrate transport to world-agent.
  - Fail-closed: if the guest is not available, gateway use fails with exit code `5` when `llm.fail_closed.routing=true`.
- Windows:
  - Gateway runs inside the WSL world distribution. Host talks to it via the existing Substrate transport to world-agent.
  - Fail-closed: same as macOS (when `llm.fail_closed.routing=true`).

## Architecture Shape
- Components:
  - `crates/llm-gateway` (new): binary “front door” HTTP server for OpenAI+Anthropic dialects; minimal parsing; emits structured events/spans.
  - `crates/llm-manager` (new): backend router + policy gate + logging abstraction; selects engine by config.
  - `crates/world-agent` (existing): supervises starting/stopping gateway inside the world session; exposes health/status over existing agent transport.
  - `crates/trace` (existing): stores canonical spans/events for LLM requests in JSONL; optional fs diff integration not required here.
  - `crates/broker` (existing): optional policy decision integration point (allow/deny/require-approval) for LLM operations.

- End-to-end flow:
  - Inputs:
    - HTTP requests (OpenAI-compatible and Anthropic-compatible)
    - Effective config (project/global/env/flags)
    - Policy state (observe/enforce)
  - Derived state:
    - `orchestration_session_id`, `thread_id`, `run_id`
    - chosen backend kind (`passthrough|api|cli`)
    - policy decision (allow/deny/require-approval)
  - Actions:
    - route request to engine via `llm-manager`
    - stream response back to client
    - write structured span/event to trace pipeline
  - Outputs:
    - dialect-correct HTTP response/stream
    - structured JSONL span/event record
    - optional metrics (if enabled)

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-gateway-in-world` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0017 (Output/Event Contract) must be available for stable event framing and routing attribution.
  - ADR-0028 (Correlation vocabulary + required/optional matrix) must be available as the canonical join-key spine for persisted trace records (no heuristic joins).
  - Agent transport parity work must be available for macOS (Lima) and Windows (WSL) bridging.

## Security / Safety Posture
- Fail-closed rules:
  - If `SUBSTRATE_WORLD=enabled` and the world is unavailable, gateway start/use fails (no host fallback).
  - If policy requires “strict egress control,” gateway must refuse to start on platforms where enforcement cannot be guaranteed.
- Protected paths/invariants:
  - Gateway logs and state live under `~/.substrate/llm/` inside world context with user-only permissions.
  - Do not log request/response bodies by default; logging bodies requires explicit config.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Dialect parsing (OpenAI, Anthropic) → canonical internal request representation.
  - Policy decision application (allow/deny) is deterministic and emits a stable event.
- Integration tests:
  - Start gateway inside a test world session and issue a minimal request through each dialect endpoint.
  - Validate fail-closed behavior when world is required but unavailable.

### Manual validation
- Manual playbook: `docs/project_management/_archived/next/llm_gateway_in_world/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/_archived/next/llm_gateway_in_world/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/next/llm_gateway_in_world/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/next/llm_gateway_in_world/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md`:
    - DR-0001 (Gateway bind strategy: UDS vs loopback TCP inside world)
    - DR-0002 (Default logging policy: metadata-only vs body logging)
    - DR-0003 (Policy integration point: broker vs gateway-native checks)
