# llm_and_agent_config_policy_surface — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/llm_and_agent_config_policy_surface/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Spec manifest:
  - `docs/project_management/next/llm_and_agent_config_policy_surface/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

### Create
- `docs/project_management/next/llm_and_agent_config_policy_surface/spec_manifest.md` — required spec ownership map (planning v4)
- `docs/project_management/next/llm_and_agent_config_policy_surface/impact_map.md` — impact map (planning v4)
- `docs/project_management/next/llm_and_agent_config_policy_surface/LACP0-spec.md` — Phase 3a slice spec + acceptance criteria
- `docs/project_management/next/llm_and_agent_config_policy_surface/LACP1-spec.md` — Phase 3b slice spec + acceptance criteria
- (planning-pack completion; required before execution triads begin)
  - `docs/project_management/next/llm_and_agent_config_policy_surface/plan.md`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/tasks.json`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/kickoff_prompts/*`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/session_log.md`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/quality_gate_report.md`
- (implementation-time; code/test surface)
  - `crates/shell/tests/adr_0027_llm_agents_schema.rs` (or similar) — config/policy parsing + dotted-update coverage for new keys
  - `crates/broker/tests/adr_0027_effective_policy.rs` (or similar) — effective policy resolution + provenance for new keys
  - `crates/shell/tests/adr_0027_agent_inventory.rs` (or similar) — strict agent file parsing + overlay broadening rejection

### Edit
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` — update `Related Docs` to include `spec_manifest.md`, `impact_map.md`, plan/tasks, and slice specs; keep links internally consistent
- `docs/project_management/next/llm_and_agent_config_policy_surface/SCHEMA.md` — keep `policy_overlay` composition rules aligned with DR-0007 and keep the overlay allowed-key subset explicit
- `docs/CONFIGURATION.md` — document new key families and strictness posture for `llm.*`, `agents.*`, and `workflow.router.*`
- `crates/shell/src/execution/config_model.rs` — extend strict config schema/patch model to include `llm.*` and `agents.*`
- `crates/shell/src/execution/config_cmd.rs` — ensure dotted update application supports all new config keys (including enums) and `--explain` includes them
- `crates/shell/src/execution/policy_model.rs` — extend shell-side effective policy model + explain rendering to include new policy keys
- `crates/shell/src/execution/policy_cmd.rs` — ensure effective policy display includes new keys deterministically (YAML and JSON)
- `crates/broker/src/effective_policy.rs` — extend policy patch schema + effective policy resolution/provenance for new keys
- `crates/broker/src/policy.rs` — extend strict policy schema types to include new key families
- `crates/broker/src/profile.rs` — ensure default/built-in policy/config structures remain valid with new keys (no missing defaults)

### Deprecate
- None (ADR-0027 is additive: new keys only; no legacy key families are removed).

### Delete
- None.

## Cascading implications (behavior/UX)

### CLI / UX
- Change: operators can enable/configure LLM + agent features via existing patch-management CLIs (new keys only).
  - Direct impact:
    - No new file families to learn; operators set `llm.*` / `agents.*` keys using familiar commands.
  - Cascading impact:
    - Error messages for strict schema violations must be actionable (unknown key path, invalid enum value, file path).
    - `--explain` must remain stable so operators can debug precedence without guessing.
  - Contradiction risks:
    - Any component or doc that implies a separate config/policy file family for LLM/agents would contradict this ADR and must be corrected.

### Config / env vars / paths
- Change: new key families are introduced into the strict config schema (`llm.*`, `agents.*`) with disabled-by-default defaults.
  - Direct impact:
    - Effective config gains new sections with deterministic defaults.
  - Cascading impact:
    - Workspace-vs-env override rules must remain consistent (workspace config present ⇒ ignore `SUBSTRATE_OVERRIDE_*`).
    - Future profiles snapshots (ADR-0020) must include these keys without special-casing.
  - Contradiction risks:
    - Partial/loose schema acceptance (allowing unknown keys) would undermine fail-closed posture and create drift.

- Change: a new agent inventory directory model becomes part of the operator-visible file layout.
  - Direct impact:
    - Operators can define agent backends in `$SUBSTRATE_HOME/agents/*.yaml` and workspace `.substrate/agents/*.yaml`.
  - Cascading impact:
    - Strict file schema and filename/id matching must be enforced to keep inventory deterministic.
    - Docs and tooling that reference “no secrets in YAML” must remain consistent with inventory rules.
  - Contradiction risks:
    - Allowing agent file overlays to broaden policy would silently create per-agent privilege escalation.

### Policy / isolation / security posture
- Change: policy gains explicit allowlists + fail-closed routing controls for LLM/agents, plus additive `workflow.router.*` gates.
  - Direct impact:
    - Deny-by-default posture becomes explicit and explainable (`allowed_backends=[]` denies; router disabled by default).
  - Cascading impact:
    - Agent `policy_overlay` must be restriction-only and validated to reject broadening attempts.
    - Downstream components (gateway/engine/hub/router) must treat these keys as the sole source of truth (no ad-hoc env var bypasses).
  - Contradiction risks:
    - Implementations that treat policy as “feature flags” (e.g., adding `policy.llm.enabled`) would contradict DR-0004 and must not ship.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - Overlap surfaces: config/policy key paths for gateway enablement/routing; secret sourcing allowlists.
  - Conflict: no (dependent).
  - Resolution (explicit): ADR-0023 must consume `llm.*` keys from ADR-0027 and must not define new config/policy file families.
- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Overlap surfaces: backend id format; agent inventory file format for `cli:*` backends.
  - Conflict: no (dependent).
  - Resolution (explicit): ADR-0024 must not redefine backend id grammar or inventory directory layout.
- ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - Overlap surfaces: `agents.*` key paths, agent inventory parsing/selection, `policy_overlay` tightening.
  - Conflict: no (dependent).
  - Resolution (explicit): hub-core ADRs define runtime behavior but defer to ADR-0027 for config/policy storage surface and overlay rules.
- ADR: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
  - Overlap surfaces: `agents.toolbox.*` config keys and policy gating expectations.
  - Conflict: no (dependent).
  - Resolution (explicit): toolbox ADR consumes key paths defined here and does not define new key roots.
- ADR: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
  - Overlap surfaces: `workflow.router.*` policy gates.
  - Conflict: no (dependent; Phase 8 additive).
  - Resolution (explicit): router ADR must treat `workflow.router.*` as policy-owned gates and remain fail-closed by default.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/next/llm_gateway_in_world/`
  - Overlap surfaces: `llm.*` enable/routing keys; secret allowlist keys.
  - Conflict: potential if gateway plans introduce new operator-facing key roots or duplicate policy gates.
  - Resolution (explicit): use ADR-0027 keys only; record any additional required keys as additive ADR-0027 updates (Decision Register A/B) rather than local one-offs.
- Planning Pack: `docs/project_management/next/llm_cli_backend_engine/`
  - Overlap surfaces: backend id format; agent inventory schema; host credential read gate.
  - Conflict: potential if engine plans introduce backend-specific key families in config/policy.
  - Resolution (explicit): backend-specific configuration lives in inventory files; global/workspace config/policy remains shape-stable per ADR-0027.
- Planning Pack: `docs/project_management/next/agent_hub_core/`
  - Overlap surfaces: agent inventory discovery and overlay semantics.
  - Conflict: no (dependent).
  - Resolution (explicit): agent hub consumes inventory and overlay rules; it must not redefine overlay composition semantics.
- Planning Pack: `docs/project_management/next/host_event_bus_router_daemon/`
  - Overlap surfaces: router policy gates and allowlists.
  - Conflict: no (dependent).
  - Resolution (explicit): router planning owns runtime behavior but policy gate surfaces remain defined here.
- Planning Pack: `docs/project_management/next/workflow-engine/`
  - Overlap surfaces: workflow ids and cross-workspace safety model.
  - Conflict: no (dependent).
  - Resolution (explicit): workflow work must align ids/allowlists with `workflow.router.*` gating keys and must not create competing enablement keys.

## Follow-ups (explicit)

- Decision Register entries required:
  - None (ADR-0027 decision register already captures key A/B decisions; new decisions are added only when a new operator-facing surface is introduced).
- Spec updates required (if any):
  - `docs/project_management/next/llm_and_agent_config_policy_surface/spec_manifest.md` — keep the required spec list in lockstep with created files (no implied specs).
  - `docs/project_management/next/llm_and_agent_config_policy_surface/manual_testing_playbook.md` — extend to include agent inventory strictness and overlay broadening rejection cases.
