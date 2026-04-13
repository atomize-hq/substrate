# IMPORTANT: Substrate Alignment Constraints

Read this before making architecture decisions in this repository.

This project is allowed to move faster than Substrate, but it is not allowed to drift into a shape that Substrate cannot consume later without avoidable rework.

These constraints are mandatory design guardrails for early implementation:

1. Treat this gateway as one logical Substrate backend, not as multiple externally selectable planner/executor/provider backends.
2. Keep Azure Kimi normalization and planner/executor orchestration internal to the gateway.
3. Do not assume the current host-local dev topology is the final Substrate deployment topology.
   This means `localhost` is a development convenience, not an architectural contract. The core engine should not depend on loopback HTTP, always-on host process state, direct host credential access in the request path, or other assumptions that would make an in-world or policy-controlled deployment awkward later.
4. Do not couple shell or REPL output directly to raw provider streams; future Substrate integration must consume normalized structured events.

Authoritative repository-local decisions:

- [ADR 0005](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md)
- [ADR 0006](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md)
- [ADR 0007](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md)

External planning inputs reviewed:

- LLM/policy surface pack:
  [/Users/spensermcconnell/__Sandbox/actions-runner/_work/substrate/substrate/candidate/docs/project_management/packs/active/llm_and_agent_config_policy_surface](/Users/spensermcconnell/__Sandbox/actions-runner/_work/substrate/substrate/candidate/docs/project_management/packs/active/llm_and_agent_config_policy_surface)
- Agent-hub output routing pack:
  [/Users/spensermcconnell/__Sandbox/actions-runner/_work/substrate/substrate/docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing](/Users/spensermcconnell/__Sandbox/actions-runner/_work/substrate/substrate/docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing)

If an implementation choice conflicts with these constraints, stop and resolve the conflict in docs before proceeding.
