# ADR-0024 — CLI Backend Provider Engine (Subscription-First Cross-Routing)

## Status
- Status: Draft
- Supersession note: The architectural intent of this ADR is superseded by `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`, which preserves the stable backend-id / allowlisting contract while replacing the bespoke Substrate-local engine assumption with a gateway-hosted adapter contract.
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/_archived/next/llm_cli_backend_engine/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/_archived/next/llm_cli_backend_engine/plan.md`
- Tasks: `docs/project_management/_archived/next/llm_cli_backend_engine/tasks.json`
- Spec manifest: `docs/project_management/_archived/next/llm_cli_backend_engine/spec_manifest.md`
- Specs: `docs/project_management/_archived/next/llm_cli_backend_engine/specs/*`
- Contract (if present): `docs/project_management/_archived/next/llm_cli_backend_engine/contract.md`
- Decision Register: `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 2d6a70cef4e8fe641fba8e9639f71fdde3d661b4eedcfaae9906851b58276b13
### Changes (operator-facing)
- Treat subscription-authenticated CLIs (starting with Codex) as LLM “provider backends”
  - Existing: Cross-provider routing typically requires API keys or external proxies; subscription-authenticated CLIs cannot be used as provider backends in a controlled way.
  - New: Substrate LLM manager can satisfy OpenAI/Anthropic dialect requests by routing to a configured CLI backend session (`cli:codex` in v1), preserving subscription-first usage.
  - Why: Enable cross-provider routing without forcing API keys, while keeping enforcement/audit inside the world boundary.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md#L1`

## Problem / Context
- Substrate’s product strategy explicitly supports users leveraging their existing subscription-authenticated CLIs.
- A Substrate-owned LLM gateway must support cross-provider routing without requiring API keys. The practical method is to treat authenticated CLIs as provider backends: the gateway receives a request in one dialect and fulfills it via a CLI session from another provider.
- This cannot depend on “token scraping.” Usage metadata is best-effort unless the CLI provides it.

## Goals
- Implement a `cli` backend kind for `llm-manager` that can fulfill canonical LLM requests via CLI sessions.
- Support dialect decoupling (v1):
  - OpenAI-style request → fulfill via `cli:codex`.
  - Anthropic-style request → fulfill via `cli:codex`.
- Future (planned): cross-provider routing by adding additional `cli:*` adapters (e.g., Claude Code, Gemini CLI) behind the same canonical adapter contract.
- Preserve streaming semantics where supported; otherwise emit a bounded, explicit “buffered stream” behavior.
- Keep authentication subscription-first: the CLI’s own authentication is used; Substrate does not require API keys for this backend.
- Emit stable attribution for every routed request, aligned to Phase 8 correlation vocabulary:
  - `backend_id` (`cli:<name>`; explicit backend selection; never inferred from `agent_id`)
  - correlation (`orchestration_session_id`, `run_id`, optional `thread_id`) when tied to orchestration
  - policy decision metadata (allow/deny/require-approval)

## Non-Goals
- Extracting or reverse-engineering tokens or proprietary billing details from CLIs (“token scraping”).
- Replacing the CLIs; Substrate orchestrates them as external tools.
- Guaranteeing lossless translation for every provider-specific request field; features are capability-gated.
- Enumerating a canonical “backend registry” list (backends are inventory-defined + allowlisted; Phase 8 circle-back may add a non-normative appendix mapping example ids → contracts).
- Shipping multiple CLI backend adapters in v1 (initial implementation only requires `cli:codex`; other `cli:*` backends are a planned extension once the adapter contract is proven).

## User Contract (Authoritative)

### CLI
- This ADR adds no new top-level CLI commands beyond those defined by ADR-0023.
- When `substrate world gateway status` is invoked and a CLI backend is enabled, status output includes:
  - backend availability (binary found / authenticated status unknown / last invocation success),
  - session mode (per-request vs persistent),
  - declared capabilities (stream support, tool-call support).

### Config
This ADR MUST use the Phase 3 surface defined by ADR-0027 for:
- config/policy key paths + precedence + defaults, and
- agent inventory for registering CLI backends (one file per backend).

Sources of truth:
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/reference/policy/schema.md`

Config (selection surface; ADR-0027):
- `llm.enabled: bool`
- `llm.gateway.enabled: bool`
- `llm.routing.default_backend: <kind>:<name>` (e.g., `cli:codex`)

Policy (constraints surface; ADR-0027):
- `llm.allowed_backends: [<kind>:<name>]` (deny-by-default allowlist)
- `llm.fail_closed.routing: bool`
- `net_allowed` remains authoritative for outbound egress enforcement inside the world boundary.

CLI backends are registered via agent inventory files (ADR-0027):
- Global: `$SUBSTRATE_HOME/agents/<inventory_id>.yaml` (default `~/.substrate/agents/<inventory_id>.yaml`)
- Workspace: `<workspace_root>/.substrate/agents/<inventory_id>.yaml`

Normative mapping for this ADR:
- A CLI backend id of the form `cli:<name>` MUST resolve to an agent inventory item with `id: <name>` where:
  - `config.kind: cli`
  - `config.capabilities.llm: true`
  - `config.cli.binary: <string>` (optional; default resolve via PATH)
  - `config.cli.mode: persistent|per_request` (optional; default via `agents.defaults.cli.mode`)

### Platform guarantees
- Linux/macOS/Windows:
  - CLI backends execute inside the world boundary whenever LLM operations are configured to run in-world.
  - Fail-closed: if effective policy has `llm.fail_closed.routing=true` and the world is unavailable, routing to CLI backends fails closed (no host fallback).

## Architecture Shape
- Components:
  - `crates/llm-manager` (new/existing from ADR-0023): adds `CliBackendEngine`.
  - `crates/cli-agents` (existing/new): minimal runner wrappers for CLI backends (v1: Codex) with consistent spawn/session semantics.
  - `crates/trace`: logs structured request spans/events with explicit correlation keys and `backend_id` (routing decision) per the ADR-0028 vocabulary.

### Correlation + trace alignment (Phase 8)

This ADR does not define its own correlation vocabulary. Any trace records and/or structured events emitted as part of CLI-backend routing MUST explicitly defer to:
- ADR-0028 (canonical correlation vocabulary + required/optional matrix): `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- ADR-0017 (structured event envelope + routing attribution) when emitting structured events: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

- End-to-end flow:
  - Inputs:
    - canonical request from `llm-gateway` front door (dialect normalized)
    - routing decision from config/policy
  - Derived state:
    - chosen CLI backend (`codex` in v1; future: other `cli:*` adapters)
    - `backend_session_id` (Substrate-managed identifier)
  - Actions:
    - prepare CLI invocation:
      - build CLI-specific prompt contract from canonical request
      - inject tool endpoints (e.g., MCP) when enabled and allowed
    - execute CLI:
      - per-request spawn OR persistent session channel
    - translate CLI output → canonical response → dialect response
  - Outputs:
    - dialect response to client
    - span/event record including:
      - `backend_id` (`cli:<name>`)
      - correlation (`orchestration_session_id`, `run_id`, optional `thread_id`) when applicable

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-cli-backend-engine` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0023 must land first (gateway + manager shape).
  - ADR-0017 must be available for stable event framing and attribution.
  - ADR-0028 must be available as the canonical correlation vocabulary + required/optional matrix.

## Security / Safety Posture
- Fail-closed rules:
  - If routing selects a CLI backend but the binary is not present, request fails (exit code `4` at CLI boundary; HTTP error at gateway boundary).
  - If effective policy has `llm.fail_closed.routing=true` and the world is unavailable, request fails (policy exit code `5` semantics).
- Protected paths/invariants:
  - Do not copy/emit CLI credentials; Substrate does not persist subscription tokens.
    - v1 (Codex legacy): required auth fields are extracted from host login state and delivered to the in-world gateway/manager at spawn time (no auth files are present in-world). This MUST be explicit, policy-gated (`agents.host_credentials.read.allowed_backends`), and MUST NOT log secret values (see `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md` DR-0006, DR-0008, and Phase 8 DR-0012).
  - Do not log request/response bodies by default; redact before persisting if body logging enabled.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Canonical request → CLI prompt contract generation.
  - CLI output parsing → canonical response mapping.
- Integration tests:
  - Use a stub CLI backend (test binary) that implements deterministic echo/stream behavior.
  - Validate routing: OpenAI endpoint → stub “Codex backend” → OpenAI response.
  - Validate fail-closed when world required but unavailable.

### Manual validation
- Manual playbook: `docs/project_management/_archived/next/llm_cli_backend_engine/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/_archived/next/llm_cli_backend_engine/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/next/llm_cli_backend_engine/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/next/llm_cli_backend_engine/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md`:
    - DR-0001 (CLI session strategy: persistent vs per-request)
    - DR-0002 (Streaming behavior when CLI lacks streaming: buffer+rechunk vs non-stream)
    - DR-0003 (CLI prompt contract format: JSON envelope vs plain text template)
