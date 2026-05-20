# ADR-0027 — LLM + Agent Config/Policy Surface (Existing Files, New Keys)

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Plan: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/plan.md`
- Tasks: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/tasks.json`
- CI checkpoints: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`
- Spec manifest: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md`
- Specs:
  - Contract: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
  - Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
  - Phase 3a slice: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md`
  - Phase 3b slice: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Decision Register: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md`
- Impact Map: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/impact_map.md`
- Manual playbook: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md`
- Existing config/policy layering model:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
- Profiles (future; must remain compatible):
  - `docs/project_management/adrs/draft/ADR-0020-profiles-config-policy-snapshots.md`
- Identity / tuple follow-ons (additive; must remain compatible):
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Current successor ADRs that consume this surface:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- Historical predecessor ADRs (superseded semantically; useful as origin context only):
  - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Global JSON mode plan (separate track; do not duplicate):
  - `docs/project_management/packs/draft/json-mode/json_mode_plan.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 36c3baa794d0878fae02ffa4dfa7bb9dbeed355c4aa7e4a0a554df7dd1164435
### Changes (operator-facing)
- LLM + agent behavior is configured and governed via the existing config/policy files (new keys only)
  - Existing: There is no repo-wide, stable config/policy surface for LLM gateway routing, CLI agent backends, or agent role selection, which invites ad-hoc files/env vars and inconsistent enforcement boundaries.
  - New: LLM + agent configuration and policy governance live in the existing layered surfaces:
    - config patches: `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`
    - policy patches: `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`
    - with explicit schemas, precedence, and fail-closed behavior.
  - Why: Keep Substrate’s enforcement/audit claims accurate and avoid a “second config system” as LLM + agent features land (gateway, CLI backends, agent hub, orchestration toolbox).
  - Links:
    - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md#L1`
    - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md#L1`
    - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md#L1`
    - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md#L1`
- Phase 8 additive clarification:
  - This ADR remains the source of truth for config/policy file families, precedence, fail-closed posture, backend allowlists, and host-side secret-read gates.
  - ADR-0042 defines the operator-facing identity tuple (`client`, `router`, `provider`, `auth_authority`, `protocol`).
  - ADR-0043 extends this ADR additively with tuple-axis policy constraints so backend ids do not need to carry tuple meaning by themselves.

## Problem / Context
- The next major body of work adds:
  - an in-world LLM gateway + routing layer, and
  - multiple agent backends (CLI and API) that must be role-swappable (orchestrator/executor).
- These features require:
  - operator-controlled enable/disable and routing decisions (config),
  - fail-closed “can this operation occur at all?” rules (policy),
  - stable provenance for “why did this happen?” (explainability),
  - and compatibility with the existing layered resolution system (workspace/global, CLI/env overrides, profiles).
- If we create new config/policy files ad hoc per feature, we will:
  - multiply precedence rules,
  - create drift between shell/shim/world-agent, and
  - increase the probability of boundary errors (e.g., LLM egress happening outside the world boundary without an explicit operator decision).

## Goals
- Define the authoritative config/policy surface for LLM + agent work that:
  - uses existing config/policy files (no new “root” config/policy file families),
  - adds new keys in a schema-first, strict, fail-closed way,
  - keeps precedence and explainability consistent with the existing model,
  - is compatible with future “profiles” (complete config/policy snapshots).
- Ensure “LLM egress boundary” and “agent role boundary” are explicit and governable:
  - operators can force LLM operations to require a world boundary (policy),
  - operators can restrict which backends are eligible (policy),
  - operators can select which backend is used (config).

## Non-Goals
- Implementing the LLM gateway, routing engine, or agent hub (those are separate ADRs).
- Defining a new config file format (e.g., TOML) or introducing new config roots.
- Storing secrets (API keys, tokens) in Substrate config files.
- Defining the global JSON-mode envelope for all commands (tracked by `docs/project_management/packs/draft/json-mode/json_mode_plan.md`).

## User Contract (Authoritative)

### CLI
This ADR defines the config/policy *key paths* and file locations used by LLM + agent features. Operators use the existing patch-management CLIs to set them:

- Config patch management:
  - `substrate config global init|show|set|reset`
  - `substrate config workspace show|set|reset`
  - `substrate config show` (effective merged config; YAML by default; JSON with `--json`; provenance on stderr with `--explain`)
- Policy patch management:
  - `substrate policy global init|show|set|reset`
  - `substrate policy workspace show|set|reset`
  - `substrate policy current show` (effective merged policy; YAML by default; JSON with `--json`; provenance on stderr with `--explain`)

Exit codes:
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `0`: success
- `2`: user/config/policy error (unknown keys, invalid YAML, invalid values, schema violations)
- `1`: unexpected/runtime failure (I/O errors, internal errors)

### Config

#### Files and locations (existing)
- Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
- Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`

#### Effective config precedence (existing; applies to new keys)
Per key (highest to lowest):
1. CLI flags (when applicable for a given key; world-related flags only)
2. Workspace config patch (when inside an enabled workspace)
3. Environment override inputs (`SUBSTRATE_OVERRIDE_*`) (ignored when inside an enabled workspace)
4. Global config patch
5. Built-in defaults

#### Schema additions (new keys)
All keys below are part of the config schema and MUST be strict (unknown keys rejected).

##### `llm` (LLM gateway + routing configuration)
- `llm.enabled: bool`
  - Meaning: whether LLM features are enabled at all for the effective config.
  - Default: `false` (must be explicitly enabled by config/profile).

- `llm.gateway.enabled: bool`
  - Meaning: whether Substrate may run/ensure the local gateway front door for this scope.
  - Default: `false` (must be explicitly enabled by config/profile).

- `llm.gateway.mode: in_world|host_only`
  - Meaning:
    - `in_world`: gateway must run inside the world boundary when world is enabled.
    - `host_only`: gateway runs on host (for host-only environments); permitted only when effective policy has `llm.fail_closed.routing=false`.
  - Default: `in_world`.

- `llm.routing.default_backend: string`
  - Meaning: identifier of the default backend used by the gateway/router when no explicit override is provided.
  - Format: `<kind>:<name>` (e.g., `cli:codex`, `cli:claude_code`, `api:openai`).
  - Interpretation note: this selects a backend/adapter id only. It MUST NOT be treated as a collapsed encoding of `client`, `router`, `provider`, `auth_authority`, or `protocol`.

Constraints:
- Config files MUST NOT contain secrets. Backend authentication must rely on:
  - the CLI backend’s own subscription/login state, and/or
  - environment variables for API backends (names defined by the backend contract, not by this ADR).

##### `agents` (agent subsystem defaults; agent inventory lives in `agents/`)
- `agents.enabled: bool`
  - Meaning: whether the agent hub registry/routing layer is enabled for the effective config.
  - Default: `false` (explicit enable required).

- `agents.defaults.execution.scope: host|world`
  - Meaning: default execution scope for agents when an agent file omits an explicit scope.
  - Default: `world`.

- `agents.defaults.cli.mode: persistent|per_request`
  - Meaning: default CLI session strategy for CLI-based agents when an agent file omits it.
  - Default: `persistent`.

##### `agents.hub` (additive; Phase 5 Agent Hub)
- `agents.hub.orchestrator_agent_id: string`
  - Meaning: selects the agent inventory item id assigned `role=orchestrator` for the current process/session (Agent Hub successor semantics are defined by ADR-0044).
  - Default: empty string.
  - Constraint: if `agents.enabled=true`, this key MUST be set and MUST refer to an eligible allowlisted agent inventory item (enforced by Agent Hub; ADR-0044).
- `agents.hub.world_restart.on_drift: auto_restart|fail_closed`
  - Meaning: how Agent Hub handles “world-relevant drift” during a long-running orchestration session (ADR-0044).
  - Default: `auto_restart`.

##### `agents.toolbox` (additive; Phase 5 internal toolbox; MCP protocol)
- `agents.toolbox.enabled: bool`
  - Meaning: whether the internal orchestration toolbox may run at all for the effective config (toolbox successor semantics are defined by ADR-0045).
  - Default: `false`.
- `agents.toolbox.bind.transport: uds|tcp`
  - Meaning: preferred bind transport for the toolbox endpoint (ADR-0045).
  - Default: `uds`.

Notes:
- Detailed agent runtime behavior (roles, tool gating, steering) is defined by the Agent Hub ADRs. This ADR defines the config/policy storage surface and the inventory directory pattern that those ADRs depend on.

#### Agent inventory (new; file-based)
Agent definitions are stored as inventory items, one file per agent, mirroring the deps inventory model (ADR-0011):
- Global agent defs: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `~/.substrate/agents/<agent_id>.yaml`)
- Workspace agent defs: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

Safety and strictness requirements:
- The filename-derived `<agent_id>` MUST match the `id:` field inside the YAML exactly.
- Agent files MUST be strict (unknown keys rejected).
- Agent files MUST NOT contain secrets.
- Agent files MAY include an embedded `policy_overlay`, but it MUST be restriction-only (it can only tighten effective policy; never broaden).

### Policy

#### Files and locations (existing)
- Global policy patch: `$SUBSTRATE_HOME/policy.yaml` (default: `~/.substrate/policy.yaml`)
- Workspace policy patch: `<workspace_root>/.substrate/policy.yaml`

#### Policy schema additions (new keys)
All keys below are part of the policy schema and MUST be strict (unknown keys rejected).

##### `llm` (LLM operation gating; enforced in gateway/manager)
- `llm.fail_closed.routing: bool`
  - Meaning: when `true`, any LLM operation MUST fail closed if it cannot be executed inside a world boundary (no host fallback).
  - Default: `true`.

- `llm.require_approval: bool`
  - Meaning: when `true`, LLM operations require approval in enforce mode (approval mechanism defined elsewhere).
  - Default: `false`.

- `llm.allowed_backends: [string]`
  - Meaning: allowlist of backend ids permitted for LLM operations (empty means “no backends allowed”).
  - Elements format: `<kind>:<name>`.

- `llm.secrets.env_allowed: [string]`
  - Meaning: allowlist of secret env var *names* that Substrate is permitted to read from the host process environment for host→world secret delivery to the in-world gateway/engine.
  - Default: `[]` (deny-by-default; no secret host env reads allowed for LLM secret delivery).
  - Note: env var names only; values must never be stored in Substrate YAML; missing names fail closed with actionable errors.

Phase 8 additive note:
- ADR-0043 extends this policy family with tuple-axis narrowing constraints under `llm.constraints`:
  - `llm.constraints.routers`
  - `llm.constraints.providers`
  - `llm.constraints.protocols`
  - `llm.constraints.auth_authorities`
- Those keys narrow the effective `router`, `provider`, `protocol`, and `auth_authority` independently of backend/adapter allowlists.
- In v1, `client` remains an operator-visible semantic field defined by ADR-0042, but is not a standalone policy key in this ADR.

##### `agents` (agent backend gating; enforced in agent hub)
- `agents.allowed_backends: [string]`
  - Meaning: allowlist of backend ids eligible for assignment/routing (empty means “no backends allowed”).
  - Elements format: `<kind>:<name>`.

- `agents.fail_closed.routing: bool`
  - Meaning: when `true`, agent executions configured/routed to run in-world MUST fail closed if a world boundary is not available (no host fallback).
  - Default: `true`.

- `agents.host_credentials.read.allowed_backends: [string]`
  - Meaning: allowlist of backend ids permitted to read host credential material as part of a backend adapter’s host-side preparation step (e.g., reading an existing CLI login state so required auth fields can be delivered to an in-world component over a Substrate-owned secret channel, rather than persisting them).
  - Elements format: `<kind>:<name>`.
  - Default: `[]` (deny-by-default).
  - Note: This gates host credential reads only; network egress remains governed by `net_allowed` at the world boundary.

##### `workflow.router` (indirect execution via router daemon; enforced in router)

Phase 8 additive note: ADR-0029 introduces an indirect execution path (trace event → request → action). This must be explicitly policy-gated, fail-closed by default, and explainable.
This `workflow.router` namespace is unrelated to the LLM identity-tuple field `router` defined by ADR-0042. It governs the workflow router daemon only and MUST NOT be read as the semantic routing authority for LLM fulfillment.

- `workflow.router.enabled: bool`
  - Meaning: when `true`, router-derived requests/actions are eligible for evaluation/execution (still gated by the additional allowlists below).
  - Default: `false` (fail-closed; router is disabled unless explicitly enabled).

- `workflow.router.allow_cross_workspace: bool`
  - Meaning: when `true`, router may enqueue/execute actions targeting a different workspace than the source event workspace (subject to target allowlists).
  - Default: `false` (fail-closed; no cross-workspace routing).

- `workflow.router.allowed_rule_ids: [string]`
  - Meaning: allowlist of router `rule_id` values eligible to trigger requests/actions.
  - Default: `[]` (deny-by-default).

- `workflow.router.allowed_workflow_ids: [string]`
  - Meaning: allowlist of workflow ids eligible for router-triggered `workflow.run` actions.
  - Default: `[]` (deny-by-default).

- `workflow.router.allowed_target_workspace_ids: [string]`
  - Meaning: allowlist of workspace ids eligible as routing targets.
  - Default: `[]` (deny-by-default).

#### Interaction with existing policy keys
- Network egress control remains policy-owned via `net_allowed`:
  - LLM gateways/backends that make outbound requests MUST have those requests governed by the effective policy `net_allowed` allowlist.
  - An empty `net_allowed` means “no outbound hosts allowed” (deny-all).

### Platform guarantees
- Linux/macOS/Windows:
  - The config/policy file shapes and key paths are identical.
  - The default posture for LLM and agent operations is fail-closed (disabled unless enabled by config and allowed by policy allowlists/requirements).

## Architecture Shape
- Components impacted (high level):
  - `crates/shell`:
    - extend config schema + patch schema to include `llm.*` and `agents.*`
    - extend dotted update support for the new keys (for `config set` and `reset`)
  - `crates/broker`:
    - extend policy schema + patch schema to include `llm.*` and `agents.*`
    - ensure effective policy resolution and `--explain` provenance include the new keys
  - `crates/world-agent` / world backends:
    - continue to enforce network allowlists via `net_allowed` (used by LLM egress paths)
  - LLM gateway/manager + agent hub components (defined in other ADRs):
    - consume effective config/policy and enforce fail-closed rules described above

- End-to-end flow (config/policy relevant):
  - Inputs:
    - config patches (global/workspace), policy patches (global/workspace)
    - environment override inputs (when no workspace exists), CLI flags (world-related)
  - Derived state:
    - effective config (includes `llm.*` and `agents.*`)
    - effective policy (includes `llm.*`, `agents.*`, and `net_allowed`)
  - Actions:
    - LLM request and agent routing paths consult policy gates before executing
  - Outputs:
    - allow/deny (and approval) decisions are observable in structured events/spans (defined in other ADRs)

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `llm-and-agent-config-policy-surface` (to be scheduled)
- Dependencies:
  - LLM gateway ADRs (front door + engines) must use this ADR’s config/policy shape.
  - Agent hub ADRs must use this ADR’s config/policy shape.
  - Profiles ADR (ADR-0020) must treat these keys as part of “complete config/policy snapshots”.
  - ADR-0042 defines the operator-facing tuple semantics and must not be collapsed back into backend ids.
  - ADR-0043 is the additive policy-surface extension for tuple-axis constraints within this config/policy root.

## Security / Safety Posture
- Fail-closed rules:
  - LLM and agent operations are disabled by default:
    - config defaults to disabled (`llm.enabled=false`, `agents.enabled=false`), and
    - policy defaults to deny-by-default allowlists (`llm.allowed_backends=[]`, `agents.allowed_backends=[]`).
  - If `llm.fail_closed.routing=true` and a world boundary is not available, LLM operations MUST fail closed (no host fallback).
  - If `llm.allowed_backends` / `agents.allowed_backends` are empty, routing MUST fail closed (no implicit backend selection).
- Protected paths/invariants:
  - Config files must not store secrets; secrets must be provided via backend-owned mechanisms (subscription state) or environment variables.
  - Any logging of request/response bodies must be explicitly enabled and must honor the repo’s redaction posture (details live in LLM gateway ADRs).
  - The five-part identity tuple from ADR-0042 remains semantically distinct from backend ids.
  - In v1, policy can constrain `router`, `provider`, `protocol`, and `auth_authority` via ADR-0043, while `client` remains operator-visible metadata rather than a standalone policy key.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - config schema validation for new keys (unknown keys rejected; invalid enums rejected)
  - policy schema validation for new keys (unknown keys rejected; lists parse as lists)
- Integration tests:
  - `substrate policy current show --explain` includes tuple-policy keys and provenance as the authoritative merged policy surface
  - `substrate config show --explain` remains config-root inspection only and does not become the authoritative tuple-policy provenance surface
  - fail-closed behavior: with defaults (config disabled + allowlists empty), LLM/agent entrypoints refuse to run (exact behavior defined in the feature ADRs)

### Manual validation
- Manual playbook: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md`
  - DR-0001: config/policy live in existing YAML patch files (new keys only)
  - DR-0002: backend ids use `<kind>:<name>` string format
  - DR-0004: policy expresses requirements + allowlists (no `policy.*.enabled`)
  - DR-0005: agent definitions are inventory items under `agents/<agent_id>.yaml`
  - DR-0006: per-agent restriction-only `policy_overlay` is embedded in agent files (not separate policy files)
  - DR-0007: `policy_overlay` composition is restriction-only AND semantics (cannot broaden)
  - DR-0008: host/world fallback is governed by `*.fail_closed.routing` (no implicit host fallback)
  - DR-0009: overlays may further restrict host credential read allowlists (subset-only)
  - DR-0010: overlays may further restrict secret env var allowlists (subset-only)
