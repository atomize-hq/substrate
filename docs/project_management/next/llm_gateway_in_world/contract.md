# contract — llm_gateway_in_world

This document is the operator-facing contract summary for ADR-0023.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
- Config/policy surface: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Output/event routing contract: `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`

## Non-negotiable invariants
- **In-world by default.** When `SUBSTRATE_WORLD=enabled`, the gateway runs inside the world boundary.
- **Fail-closed routing.** If effective policy has `llm.fail_closed.routing=true` and no world boundary is available, gateway start/use fails (no host fallback).
- **Deny-by-default allowlist.** If `llm.allowed_backends=[]`, requests fail closed (no implicit backend selection).
- **No secrets persisted.** Substrate config/policy files must not store API keys/tokens; request/response bodies are not logged by default.
  - Exception (still redacted + policy-gated): backend adapters MAY emit backend-native structured events (e.g., `cli:codex` JSONL ingestion) into session logs after applying Substrate redaction/caps.

## Secret delivery for `api:*` backends (v1.1 preferred)
- Preferred mechanism (Phase 8 additive): host→world secret values are delivered to the in-world gateway/manager via a secret-channel payload and an inherited one-time FD/pipe auth bundle (no secret-bearing env vars in-world by default).
  - See: `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0018) and `docs/project_management/next/llm_gateway_in_world/specs/env_injection.md`.

## Secret delivery for `api:*` backends (v1 legacy)
- Secrets are provided via environment-variable injection (see `docs/project_management/next/llm_gateway_in_world/decision_register.md` DR-0007).
- Inventory/config MAY reference env var *names* only (e.g., `OPENAI_API_KEY`), never values.
- Delivery mechanism:
  - The gateway lifecycle is owned by the world subsystem. v1 command surface:
    - `substrate world sync gateway`
    - `substrate world sync gateway --restart`
  - The sync/restart path gathers the required env var values from the host process environment and passes them to the world-agent as part of the in-world spawn request for the gateway/engine.
  - The in-world gateway/engine receives those values as process environment variables.
  - Values MUST NOT be written to disk by Substrate.
- Missing secrets MUST fail closed with actionable errors that name the missing env var(s) (but never print values).

## Config + policy keys (source of truth: ADR-0027)
- Config (`$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`):
  - `llm.enabled`
  - `llm.gateway.enabled`
  - `llm.gateway.mode`
  - `llm.routing.default_backend`
- Policy (`$SUBSTRATE_HOME/policy.yaml`, `<workspace_root>/.substrate/policy.yaml`):
  - `llm.allowed_backends`
  - `llm.fail_closed.routing`
  - `llm.require_approval`
  - `net_allowed` (egress allowlist; still authoritative)

## Client wiring
- `substrate world status gateway` is the authoritative client wiring surface.
  - Default output: status/health (no wiring exports).
  - `--debug`: prints the required exports (base URLs) to route OpenAI/Anthropic-compatible clients through Substrate.
  - `--json`: always includes non-secret `client_wiring.*` fields.
- This contract intentionally does not freeze the underlying transport mechanics (ports vs proxying), only the client-facing env outputs and behavior guarantees.

## HTTP surfaces (subset; capability-gated)
The gateway exposes a minimal subset of:
- OpenAI-compatible endpoints (e.g., `/v1/chat/completions`).
- Anthropic-compatible endpoints (e.g., `/v1/messages`).

Unsupported endpoints MUST fail clearly (e.g., `404`/`501`) and MUST emit a structured trace/event indicating “unsupported endpoint” (no silent partial success).

## Attribution + tracing (high-level)
Each request MUST emit a structured record with stable correlation fields consistent with ADR-0017 and Phase 8 trace circle-back:
- join/correlation: `orchestration_session_id`, `run_id`, `thread_id` (when applicable), `agent_id` (when applicable)
- isolation: `world_id` when routed in-world
- routing: selected `backend_id` (`<kind>:<name>`), backend kind, and allowlist check result
- policy: allow/deny/require-approval outcome
