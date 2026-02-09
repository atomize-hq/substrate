# contract — llm_cli_backend_engine

This document is the operator-facing contract summary for ADR-0024.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
- Config/policy surface: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

## Non-negotiable invariants
- **Subscription-first.** CLI backends rely on the CLI’s own authentication state; Substrate does not store subscription tokens.
- **In-world enforcement.** When LLM operations are routed in-world, CLI backend invocations execute inside the same world boundary and are subject to `net_allowed` and other effective policy constraints.
- **Fail-closed routing.** If effective policy has `llm.fail_closed.routing=true` and no world boundary is available, requests fail closed (no host fallback).
- **Deny-by-default allowlist.** If `llm.allowed_backends=[]`, routing to CLI backends is denied by default.
  - Credential reuse note: subscription auth state MAY be forwarded/mounted into the world (policy-gated; no secrets in Substrate YAML). See `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/next/llm_cli_backend_engine/decision_register.md` (DR-0004).

## Backend identity + registration (ADR-0027)
CLI backends are registered as agent inventory items:
- Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `~/.substrate/agents/<agent_id>.yaml`)
- Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

Backend ids used by LLM routing have format `<kind>:<name>`.
For CLI engines, this ADR defines the mapping:
- `cli:<agent_id>` → agent inventory item with `id: <agent_id>` and:
  - `config.kind: cli`
  - `config.capabilities.llm: true`
  - `config.cli.mode: persistent|per_request` (optional; defaults via `agents.defaults.cli.mode`)

v1 requirement:
- The initial implementation MUST support `cli:codex`. Additional `cli:*` backends are a planned extension.

## Streaming + translation (capability-gated)
- The engine may be unable to provide true incremental streaming if the underlying CLI does not expose it.
- Streaming semantics (true stream vs synthetic stream vs non-stream) are finalized via `docs/project_management/next/llm_cli_backend_engine/decision_register.md`.

## Attribution + tracing (high-level)
Each routed request MUST record:
- `backend_id` (`cli:<agent_id>`) and a stable backend session identifier (when persistent)
- `world_id` when routed in-world
- allowlist/policy gate outcomes (allowed/denied; require-approval if applicable)
