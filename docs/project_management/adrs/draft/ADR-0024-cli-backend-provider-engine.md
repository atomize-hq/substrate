# ADR-0024 — CLI Backend Provider Engine (Subscription-First Cross-Routing)

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/next/llm_cli_backend_engine/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/llm_cli_backend_engine/plan.md`
- Tasks: `docs/project_management/next/llm_cli_backend_engine/tasks.json`
- Spec manifest: `docs/project_management/next/llm_cli_backend_engine/spec_manifest.md`
- Specs: `docs/project_management/next/llm_cli_backend_engine/specs/*`
- Decision Register: `docs/project_management/next/llm_cli_backend_engine/decision_register.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- Treat Codex/Claude/Gemini CLIs as LLM “provider backends”
  - Existing: Cross-provider routing typically requires API keys or external proxies; subscription-authenticated CLIs cannot be used as provider backends in a controlled way.
  - New: Substrate LLM manager can satisfy OpenAI/Anthropic dialect requests by routing to a configured CLI backend session (Codex/Claude Code/Gemini CLI), preserving subscription-first usage.
  - Why: Enable cross-provider routing without forcing API keys, while keeping enforcement/audit inside the world boundary.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md#L1`

## Problem / Context
- Substrate’s product strategy explicitly supports users leveraging their existing Codex/Claude/Gemini subscriptions via CLIs.
- A Substrate-owned LLM gateway must support cross-provider routing without requiring API keys. The practical method is to treat authenticated CLIs as provider backends: the gateway receives a request in one dialect and fulfills it via a CLI session from another provider.
- This cannot depend on “token scraping.” Usage metadata is best-effort unless the CLI provides it.

## Goals
- Implement a `cli` backend kind for `llm-manager` that can fulfill canonical LLM requests via CLI sessions (Codex, Claude Code, Gemini CLI).
- Support cross-routing:
  - OpenAI-style request → fulfill via Claude Code CLI backend.
  - Anthropic-style request → fulfill via Codex CLI backend.
- Preserve streaming semantics where supported; otherwise emit a bounded, explicit “buffered stream” behavior.
- Keep authentication subscription-first: the CLI’s own authentication is used; Substrate does not require API keys for this backend.
- Emit stable attribution for every routed request: `run_id`, `backend_kind=cli`, `backend_agent_id`, `provider_hint`, `policy_decision`.

## Non-Goals
- Extracting or reverse-engineering tokens or proprietary billing details from CLIs (“token scraping”).
- Replacing the CLIs; Substrate orchestrates them as external tools.
- Guaranteeing lossless translation for every provider-specific request field; features are capability-gated.

## User Contract (Authoritative)

### CLI
- This ADR adds no new top-level CLI commands beyond those defined by ADR-0023.
- When `substrate llm status` is invoked and a CLI backend is enabled, status output includes:
  - backend availability (binary found / authenticated status unknown / last invocation success),
  - session mode (per-request vs persistent),
  - declared capabilities (stream support, tool-call support).

### Config
- Files and locations (precedence): same as ADR-0023, `config.toml` only.
- Schema constraints (minimum required):
  - `[llm.backends.cli]`
    - `enabled = true|false`
  - `[llm.backends.cli.codex]`
    - `binary = \"<path>\"` (default: resolve via PATH)
    - `mode = \"persistent\"|\"per_request\"`
    - `world_required = true` (default true)
  - `[llm.backends.cli.claude_code]` (same fields)
  - `[llm.backends.cli.gemini_cli]` (same fields)
  - `[llm.routing]`
    - rules that select `backend_kind=cli` based on request dialect, model patterns, project config, or explicit user override.

### Platform guarantees
- Linux/macOS/Windows:
  - CLI backends execute inside the world boundary whenever `world_required=true`.
  - If `world_required=true` and the world is unavailable, routing to that CLI backend fails closed.

## Architecture Shape
- Components:
  - `crates/llm-manager` (new/existing from ADR-0023): adds `CliBackendEngine`.
  - `crates/cli-agents` (existing/new): minimal runner wrappers for Codex/Claude/Gemini CLIs with consistent spawn/session semantics.
  - `crates/trace`: logs a structured span per request including backend agent identity and routing decision.

- End-to-end flow:
  - Inputs:
    - canonical request from `llm-gateway` front door (dialect normalized)
    - routing decision from config/policy
  - Derived state:
    - chosen CLI backend (`codex|claude_code|gemini_cli`)
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
      - `backend_kind=cli`
      - `backend_agent_id`
      - `run_id`, `orchestration_session_id`

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `llm-cli-backend-engine` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0023 must land first (gateway + manager shape).
  - ADR-0017 must be available for stable event framing and attribution.

## Security / Safety Posture
- Fail-closed rules:
  - If routing selects a CLI backend but the binary is not present, request fails (exit code `4` at CLI boundary; HTTP error at gateway boundary).
  - If `world_required=true` and world is unavailable, request fails (policy exit code `5` semantics).
- Protected paths/invariants:
  - Do not copy/emit CLI credentials; Substrate does not persist subscription tokens.
  - Do not log request/response bodies by default; redact before persisting if body logging enabled.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Canonical request → CLI prompt contract generation.
  - CLI output parsing → canonical response mapping.
- Integration tests:
  - Use a stub CLI backend (test binary) that implements deterministic echo/stream behavior.
  - Validate cross-routing: OpenAI endpoint → stub “Claude backend” → OpenAI response.
  - Validate fail-closed when world required but unavailable.

### Manual validation
- Manual playbook: `docs/project_management/next/llm_cli_backend_engine/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/llm_cli_backend_engine/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/llm_cli_backend_engine/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/llm_cli_backend_engine/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/llm_cli_backend_engine/decision_register.md`:
    - DR-0001 (CLI session strategy: persistent vs per-request)
    - DR-0002 (Streaming behavior when CLI lacks streaming: buffer+rechunk vs non-stream)
    - DR-0003 (CLI prompt contract format: JSON envelope vs plain text template)

