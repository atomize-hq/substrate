# ADR-0025 — Agent Hub Core (Role-Swappable Agent Backends)

## Status

- Status: Historical
- Original date (UTC): 2026-02-09
- Curated into `docs/adr/historical/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

This curated ADR is kept only as historical context. The project-management ADR remains as the
planning-rich source retained for compatibility while `docs/project_management/**` is retired.

## Historical Decision Snapshot

This draft proposed an Agent Hub registry and routing layer where CLI or API backends could assume
orchestrator or member roles without hardcoding those roles into backend types.

The historical shape matters because it established the original direction for:

- deterministic agent registration and routing
- concurrent attribution for multi-agent execution
- role assignment as a control-plane concern instead of a backend-type distinction

## Why Historical

This framing was superseded before it became the durable semantic contract.

The newer direction keeps the orchestration goals while clarifying identity and backend semantics
through:

- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

In particular, the successor direction stops overloading `backend_id` with role, provider, or
protocol meaning.

## Historical Note

Keep the original draft for archived Agent Hub origin context, not as the current identity or
orchestration contract.
