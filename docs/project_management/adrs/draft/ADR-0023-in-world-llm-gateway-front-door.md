# ADR-0023 — In-World Substrate LLM Gateway (Front Door + Engines)

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/next/llm_gateway_in_world/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/llm_gateway_in_world/plan.md`
- Tasks: `docs/project_management/next/llm_gateway_in_world/tasks.json`
- Spec manifest: `docs/project_management/next/llm_gateway_in_world/spec_manifest.md`
- Specs: `docs/project_management/next/llm_gateway_in_world/specs/*`
- Contract (if present): `docs/project_management/next/llm_gateway_in_world/contract.md`
- Decision Register: `docs/project_management/next/llm_gateway_in_world/decision_register.md`
- Impact Map: `docs/project_management/next/llm_gateway_in_world/impact_map.md`
- Manual Playbook: `docs/project_management/next/llm_gateway_in_world/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

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

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate llm status`:
    - Behavior: prints gateway state (running/not), bind endpoints inside world, active backend kind, and policy mode.
    - Exit codes: `0` success; `4` gateway not available; `5` required-in-world but world not available.
  - `substrate llm env`:
    - Behavior: prints shell exports to route OpenAI/Anthropic-compatible clients through the in-world gateway (intended for `eval \"$(substrate llm env)\"`).
    - `--json`: prints structured JSON with the same values.
    - Exit codes: `0` success; `5` world required but not available.
  - `substrate llm start`:
    - Behavior: idempotently ensure the in-world gateway is running for the current world session.
    - Exit codes: `0` started/already running; `5` world required but not available.
  - `substrate llm stop`:
    - Behavior: stop gateway process for the current world session (if running).
    - Exit codes: `0` stopped/already stopped; `4` stop unsupported for platform/backend.

- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: success
  - `2`: user input/usage error (invalid flags, invalid config)
  - `3`: recoverable runtime error (transient failure; retry may succeed)
  - `4`: required component unavailable (gateway not running; missing binary)
  - `5`: policy/guardrail failure (fail-closed: world required; enforcement cannot be guaranteed)

### Config
- Files and locations (precedence):
  1. Project: `<project_root>/.substrate/config.toml`
  2. Global: `~/.substrate/config.toml`
  3. Environment variables
  4. CLI flags (highest precedence)
- Schema constraints (minimum required):
  - `[llm.gateway]`
    - `enabled = true|false` (default true when worlds enabled)
    - `bind_openai = \"http://127.0.0.1:<port>\"` (inside world)
    - `bind_anthropic = \"http://127.0.0.1:<port>\"` (inside world)
    - `log_dir = \"~/.substrate/llm/logs\"` (inside world)
    - `log_body = false` (default)
    - `redaction = true` (default)
    - `metrics_enabled = false` (default)
  - The gateway must not introduce any `settings.toml`; only `config.toml` is used.

### Platform guarantees
- Linux:
  - Gateway runs inside the world network namespace; outbound egress is subject to world-level enforcement.
  - Fail-closed: do not fall back to a host-level gateway when `SUBSTRATE_WORLD=enabled`.
- macOS:
  - Gateway runs inside the Lima guest world. Host talks to it via the existing Substrate transport to world-agent.
  - Fail-closed: if the guest is not available, `substrate llm start` fails with exit code `5` when world required.
- Windows:
  - Gateway runs inside the WSL world distribution. Host talks to it via the existing Substrate transport to world-agent.
  - Fail-closed: same as macOS.

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
- Sequencing entry: `docs/project_management/next/sequencing.json` → `llm-gateway-in-world` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0017 (Output/Event Contract) must be available for stable event framing and routing attribution.
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
- Manual playbook: `docs/project_management/next/llm_gateway_in_world/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/llm_gateway_in_world/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/llm_gateway_in_world/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/llm_gateway_in_world/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/llm_gateway_in_world/decision_register.md`:
    - DR-0001 (Gateway bind strategy: UDS vs loopback TCP inside world)
    - DR-0002 (Default logging policy: metadata-only vs body logging)
    - DR-0003 (Policy integration point: broker vs gateway-native checks)

