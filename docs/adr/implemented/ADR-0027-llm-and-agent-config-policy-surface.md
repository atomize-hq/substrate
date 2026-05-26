# ADR-0027 — LLM and Agent Config/Policy Surface

## Status

- Status: Implemented
- Original date (UTC): 2026-02-03
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Substrate's LLM and agent surfaces must reuse the existing layered config and policy files rather
than introducing a second config system.

The stable decision is:

- config lives in `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`
- policy lives in `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`
- agent inventory lives in `$SUBSTRATE_HOME/agents/<agent_id>.yaml` and
  `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- unknown keys are hard errors
- invalid values are hard errors
- routing remains fail-closed by default
- backend allowlists remain deny-by-default
- backend ids remain adapter selectors only, not overloaded identity labels
- secrets must not be stored in Substrate YAML patches

## Stable Owned Surface

The stable operator-facing and schema-facing references for this ADR are:

- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`
- `docs/reference/policy/tuple_constraints.md` for the additive ADR-0043 extension

## Current Implementation Anchors

The decision is materially implemented and enforced through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/policy_model.rs`
- `crates/broker/src/policy.rs`
- `crates/broker/src/effective_policy.rs`
- `crates/broker/src/tests.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Historical predecessors kept for context:
  - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`

## Historical Note

The original ADR includes planning-pack scope, slice references, and feature-local execution
context that do not belong in the stable ADR tree. Keep using the curated references above for
current contract truth.
