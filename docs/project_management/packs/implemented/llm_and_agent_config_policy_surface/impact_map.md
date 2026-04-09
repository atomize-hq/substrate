# llm_and_agent_config_policy_surface — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:

- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` (semantic follow-on)
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` (policy follow-on)
- Spec manifest:
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

### Create

- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md` — required spec ownership map (planning v4)
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/impact_map.md` — impact map (planning v4)
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md` — Phase 3a slice spec + acceptance criteria
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md` — Phase 3b slice spec + acceptance criteria
- (planning-pack completion; required before execution triads begin)
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/plan.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/tasks.json`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/kickoff_prompts/*`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/session_log.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/quality_gate_report.md`
- (implementation-time; code/test surface)
  - `crates/shell/tests/adr_0027_llm_agents_schema.rs` (or similar) — config/policy parsing + dotted-update coverage for new keys
  - `crates/broker/tests/adr_0027_effective_policy.rs` (or similar) — effective policy resolution + provenance for new keys
  - `crates/shell/tests/adr_0027_agent_inventory.rs` (or similar) — strict agent file parsing + overlay broadening rejection

### Edit

- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` — update `Related Docs` to include `spec_manifest.md`, `impact_map.md`, plan/tasks, and slice specs; keep links internally consistent
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md` — keep `policy_overlay` composition rules aligned with DR-0007 and keep the overlay allowed-key subset explicit
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
    - Treating `llm.routing.default_backend` as if it encodes the full operator-facing tuple would create semantic drift once ADR-0042/0043 are in play.

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
    - Confusing `workflow.router.*` with the ADR-0042 tuple field `router` would conflate workflow-daemon gating with LLM routing authority.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)

- ADR: `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - Overlap surfaces: gateway/world-boundary ownership, host-to-world secret-delivery ownership, and operator-visible gateway status/wiring posture.
  - Conflict: no (successor dependency).
  - Resolution (explicit): ADR-0040 owns the Substrate versus `substrate-gateway` runtime boundary, while this pack remains the source of truth for config/policy file families, allowlists, and fail-closed gates.
- ADR: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - Overlap surfaces: backend id stability, backend selection/allowlisting, and adapter-facing secret-read gates.
  - Conflict: no (successor dependency).
  - Resolution (explicit): ADR-0041 consumes the `<kind>:<name>` backend-id surface and agent inventory model from ADR-0027; it MUST NOT redefine backend id grammar or inventory directory layout.
- ADR: `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - Overlap surfaces: `agents.*` key paths, agent inventory parsing/selection, backend-id semantics on the agent side, and `policy_overlay` tightening.
  - Conflict: no (successor dependency).
  - Resolution (explicit): ADR-0044 defines runtime Agent Hub behavior but defers to ADR-0027 for config/policy storage, inventory layout, and overlay-composition rules.
- ADR: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - Overlap surfaces: `agents.toolbox.*` config keys, orchestrator-backend allowlisting, and toolbox-disabled fail-closed posture.
  - Conflict: no (successor dependency).
  - Resolution (explicit): ADR-0045 consumes the `agents.toolbox.*` key paths defined here and does not define new config/policy roots.
- ADR: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
  - Overlap surfaces: `workflow.router.*` policy gates.
  - Conflict: no (dependent; Phase 8 additive).
  - Resolution (explicit): router ADR must treat `workflow.router.*` as policy-owned gates and remain fail-closed by default.
- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces: operator-facing interpretation of backend ids versus `client`/`router`/`provider`/`auth_authority`/`protocol`.
  - Conflict: no, but terminology drift is possible.
  - Resolution (explicit): this pack must continue to own config/policy roots and backend-id surfaces, while ADR-0042 owns tuple semantics.
- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces: additive `llm.constraints.*` policy keys.
  - Conflict: no, if ownership stays split.
  - Resolution (explicit): this pack MUST NOT absorb ADR-0043 implementation scope; it only stays terminology-compatible and avoids implying backend ids carry tuple meaning by themselves.

### Historical predecessor ADRs (origin context only; superseded semantically)

- ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - Status in this cross-scan: historical predecessor; superseded semantically by ADR-0040.
- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Status in this cross-scan: historical predecessor; superseded semantically by ADR-0041.
- ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - Status in this cross-scan: historical predecessor; superseded semantically by ADR-0044.
- ADR: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
  - Status in this cross-scan: historical predecessor; superseded semantically by ADR-0045.

### Related Phase 8 tracks (cross-cutting; use ADRs/registry)

- Phase 8 registry (cross-cutting lock): `docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md`
- Gateway boundary/runtime ownership: `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- Gateway backend adapter contract: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Agent hub successor: `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- Orchestration toolbox successor: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- Router daemon: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- Identity tuple semantics: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Tuple-axis policy surface: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Workflow engine: `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

## Follow-ups (explicit)

- Decision Register entries required:
  - None (ADR-0027 decision register already captures key A/B decisions; new decisions are added only when a new operator-facing surface is introduced).
- Spec updates required (if any):
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md` — keep the required spec list in lockstep with created files (no implied specs).
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md` — extend to include agent inventory strictness and overlay broadening rejection cases.
