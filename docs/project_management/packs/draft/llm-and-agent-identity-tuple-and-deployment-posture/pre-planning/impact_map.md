# llm-and-agent-identity-tuple-and-deployment-posture — impact map

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Spec manifest:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- External contract docs scanned:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- Adjacent ADRs and packs scanned:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/impact_map.md`
  - `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/impact_map.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture-backup/impact_map.md`
  - `docs/project_management/_archived/next/llm_gateway_in_world/impact_map.md`
  - `docs/project_management/_archived/next/llm_cli_backend_engine/spec_manifest.md`

## Touch set (explicit)

### Create
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/identity-contract-and-schema.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/policy-and-observability-alignment.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/platform-rollout-and-validation.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/contracts/substrate-gateway-operator-contract.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/CONFIGURATION.md`
- `docs/TRACE.md`
- `docs/USAGE.md`
- `crates/agent-api-types/src/lib.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/world-agent/src/gateway_runtime.rs`
- `crates/world-agent/src/service.rs`
- `crates/world-agent/tests/gateway_runtime_parity.rs`
- `crates/trace/src/span.rs`
- `crates/trace/src/tests.rs`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior and UX)

### CLI / UX
- Change: existing gateway commands keep their names and exit taxonomy while human-readable output and machine-readable status gain explicit identity-tuple and placement-posture publication.
  - Direct impact:
    - `substrate world gateway status`, `substrate world gateway sync`, and `substrate world gateway restart` remain the only operator entrypoints in this lane.
    - Human-readable output names router identity, placement posture, and tuple semantics without treating `backend_id` as the full explanation.
    - `status --json` widens additively outside `client_wiring.*`.
  - Cascading impact:
    - `GatewayLifecycleResponseV1` becomes the typed carrier for tuple and posture publication.
    - Shell passthrough code and tests must preserve the additive envelope for available and unavailable responses.
    - Operator docs must present `host_to_world_bridge` as transport text, never as router identity.
  - Contradiction risks:
    - Adding tuple fields under `client_wiring.*` breaks the status-schema owner line.
    - Reusing `backend_id` as the tuple carrier breaks ADR-0041 and ADR-0046.
    - Presenting `host_only` as a second standing gateway breaks ADR-0040 and ADR-0042.

### Config / env vars / paths
- Change: this feature reuses the existing config and policy surface and adds no new config keys, env vars, commands, or filesystem contracts.
  - Direct impact:
    - `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` stay the only relevant keys for placement and credential gating.
    - Example credential paths such as `~/.codex/auth.json` stay illustrative text, not new Substrate-owned path contracts.
  - Cascading impact:
    - `docs/CONFIGURATION.md` and the implemented config-policy pack docs must state the no-new-key rule and the backend-id-versus-tuple split in one voice.
    - Future ADR-0043 policy docs can add tuple-axis constraints without reopening config-root ownership.
  - Contradiction risks:
    - Any doc that implies tuple axes live inside backend ids or new config families creates drift.
    - Any doc that promotes example auth paths into required contract paths creates drift.

### Policy / isolation / security posture
- Change: routing hints stay requests validated against existing policy gates, and placement posture stays world-first unless existing policy permits `host_only`.
  - Direct impact:
    - Accepted hints influence effective provider selection.
    - Rejected hints leave `client` unchanged and do not create provider authority.
    - Bridge transport text cannot relax in-world `net_allowed` governance.
  - Cascading impact:
    - The gateway policy-evaluation contract must align with the feature-local policy spec on accepted-hint, rejected-hint, and denial outcomes.
    - Runtime status and diagnostics must distinguish router authority from provider fulfillment when they expose both.
  - Contradiction risks:
    - Rejected hints rewriting `client` or `auth_authority` break ADR-0042.
    - Tuple-axis policy keys defined in this lane would duplicate ADR-0043.
    - A bridge path that changes egress semantics breaks ADR-0027 and the runtime-parity contract.

### Telemetry / status / diagnostics
- Change: trace spans and gateway status surfaces publish tuple metadata additively while preserving canonical correlation keys and redaction rules.
  - Direct impact:
    - `crates/trace/src/span.rs` and `docs/TRACE.md` gain explicit tuple and posture vocabulary.
    - Gateway status responses and docs gain additive non-secret metadata outside `client_wiring.*`.
  - Cascading impact:
    - Trace readers, status readers, and manual validation docs must accept additive fields without treating them as new join keys.
    - `backend_id` remains present where adapter selection matters and stays separate from tuple semantics.
  - Contradiction risks:
    - Emitting secrets in tuple fields breaks the redaction contract.
    - Treating tuple fields as replacements for canonical correlation keys breaks ADR-0028.
    - Divergent tuple field names between trace and status surfaces create consumer drift.

### Platform parity
- Change: Linux, macOS, and Windows expose the same tuple and placement semantics even though transport plumbing remains platform-specific.
  - Direct impact:
    - No direct edits are implied for `crates/world`, `crates/world-mac-lima`, or `crates/world-windows-wsl` in this lane.
    - Parity proof routes through shared gateway runtime and status surfaces plus feature-local validation docs.
  - Cascading impact:
    - The manual playbook must call out identical operator semantics and hidden transport-divergence boundaries.
    - Runtime parity tests must validate the widened status envelope and the unchanged unavailable shape.
  - Contradiction risks:
    - Platform docs that invent different meanings for `host_only` or `host_to_world_bridge` create contract drift.
    - Platform-specific gateway behavior that changes tuple publication shape breaks the parity guarantee.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - Overlap surfaces: `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, `agents.host_credentials.read.allowed_backends`, and the no-new-config-root rule.
  - Conflict: no
  - Resolution: ADR-0027 remains the only owner of key paths, precedence, and fail-closed defaults. ADR-0042 layers semantics over those keys without adding new ones.

- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces: canonical correlation vocabulary, trace redaction posture, and replay-reader expectations.
  - Conflict: no
  - Resolution: ADR-0028 remains the owner of canonical correlation keys. ADR-0042 adds non-secret tuple metadata without replacing those keys.

- ADR: `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - Overlap surfaces: single-router posture, host-to-world bridge semantics, gateway lifecycle ownership, and `status --json` boundary placement.
  - Conflict: no
  - Resolution: ADR-0040 remains the lifecycle and boundary owner. ADR-0042 layers operator-visible tuple and posture semantics over that boundary.

- ADR: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - Overlap surfaces: `backend_id`, adapter selection semantics, and status diagnostics that mention backend realization.
  - Conflict: yes
  - Resolution:
    - Option A: treat `backend_id` as the operator-visible carrier for client, router, provider, auth authority, and protocol.
    - Option B: keep `backend_id` as the adapter selector and publish tuple metadata as separate fields.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces: tuple-axis policy narrowing, placement-posture gating text, and routing-hint evaluation language.
  - Conflict: yes
  - Resolution:
    - Option A: define tuple-axis policy keys inside this lane.
    - Option B: keep this lane on semantics, status, trace, and existing-gate reuse, then route new tuple-axis keys through ADR-0043 only.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - Overlap surfaces: structured agent-event tuple publication, pure-agent absence rules for `provider` and `auth_authority`, and adapter-versus-tuple semantics on the agent side.
  - Conflict: yes
  - Resolution:
    - Option A: define agent-run presence and absence rules for tuple fields in ADR-0042.
    - Option B: keep base field meaning in ADR-0042 and let ADR-0044 own pure-agent and nested-LLM publication rules.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - Overlap surfaces: toolbox trace vocabulary, control-plane tuple fields, and adapter-versus-tuple separation.
  - Conflict: yes
  - Resolution:
    - Option A: let toolbox trace semantics define alternate meanings for tuple fields and placement posture.
    - Option B: keep tuple meanings stable here and let ADR-0045 define toolbox-specific publication rules that reuse those meanings.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - Overlap surfaces: gateway status payload widening, backend realization text, selected-backend diagnostics, and operator messaging around `backend_id`.
  - Conflict: yes
  - Resolution:
    - Option A: fold tuple publication and placement-posture semantics into ADR-0046 implementation scope.
    - Option B: keep ADR-0046 on backend realization and keep tuple publication and posture semantics in ADR-0042 plus this feature-local doc set.
    - Selected option: B.

### Relevant Planning Packs
- Planning Pack: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/`
  - Overlap surfaces: config roots, policy roots, backend-id format, and fail-closed routing defaults.
  - Conflict: no
  - Resolution: the implemented pack remains authoritative. This feature aligns its semantics to that pack and does not reopen its storage model.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
  - Overlap surfaces: gateway lifecycle owner line, command family, `status --json` envelope boundary, and host-to-world bridge wording.
  - Conflict: yes
  - Resolution:
    - Option A: widen the boundary pack so it owns tuple and posture semantics directly.
    - Option B: keep the boundary pack on lifecycle and envelope ownership, then layer tuple and posture semantics in this feature.
    - Selected option: B.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
  - Overlap surfaces: adapter selection, `backend_id`, capability metadata, and backend-facing schema terms.
  - Conflict: yes
  - Resolution:
    - Option A: redefine adapter docs so `backend_id` also carries operator tuple meaning.
    - Option B: keep adapter docs on selector semantics and publish tuple meaning through this feature-local contract and telemetry docs.
    - Selected option: B.

- Planning Pack: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
  - Overlap surfaces: typed gateway lifecycle responses, shell/world-agent status flows, and selected-backend diagnostics.
  - Conflict: yes
  - Resolution:
    - Option A: let the runtime-integration pack own tuple/posture publication as part of backend realization.
    - Option B: keep runtime realization there and keep tuple/posture publication here, with one additive status envelope.
    - Selected option: B.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces: trace schema stability, reader expectations, and parity validation culture.
  - Conflict: no
  - Resolution: this feature adds metadata to the trace schema without changing the command-span correlation contract locked by the tracing pack.

- Planning Pack: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/`
  - Overlap surfaces: `docs/CONFIGURATION.md` language around precedence and workspace-versus-global behavior.
  - Conflict: no
  - Resolution: tuple and posture docs must preserve the existing precedence language and must not imply new override channels.

- Planning Pack: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture-backup/`
  - Overlap surfaces: an older copy of this feature’s contract, impact map, and playbook surfaces.
  - Conflict: yes
  - Resolution:
    - Option A: treat the backup pack as a parallel planning source.
    - Option B: treat the backup pack as stale evidence only and route all live planning through the resolved feature directory.
    - Selected option: B.

- Planning Pack: `docs/project_management/_archived/next/llm_gateway_in_world/`
  - Overlap surfaces: old gateway command phrasing, earlier host/in-world deployment assumptions, and historical environment-delivery language.
  - Conflict: yes
  - Resolution:
    - Option A: reuse archived wording when publishing new tuple and posture docs.
    - Option B: keep the archive as historical evidence only and align live docs to ADR-0040, ADR-0041, ADR-0042, and ADR-0046.
    - Selected option: B.

- Planning Pack: `docs/project_management/_archived/next/llm_cli_backend_engine/`
  - Overlap surfaces: pre-gateway backend-engine semantics and older backend-label usage.
  - Conflict: yes
  - Resolution:
    - Option A: carry archived engine terminology into the live tuple and posture lane.
    - Option B: keep the archive as historical evidence only and retire overloaded backend wording through the compatibility spec.
    - Selected option: B.

## Follow-ups
- Decision or clarification follow-ups:
  - Freeze the exact top-level status and trace field families in `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`, then align `docs/contracts/substrate-gateway-status-schema.md`, `docs/TRACE.md`, and `crates/agent-api-types/src/lib.rs` to that shape.
  - Freeze the exact human-readable status wording for router posture and tuple display in `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`, then mirror it in `docs/contracts/substrate-gateway-operator-contract.md`, `docs/USAGE.md`, and `crates/shell/src/builtins/world_gateway.rs`.
  - Freeze the exact boundary line for pure-agent and toolbox tuple publication with ADR-0044 and ADR-0045 before downstream seam planning reaches those packs.
- Spec updates required:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` — define exact field names, field types, defaults, and absence rules for trace and status publication.
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md` — define the retirement posture for overloaded-backend wording in archived and backup materials.
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md` — define the parity proof that no direct script edits or platform-backend edits are required in this lane; if that proof fails, revise this impact map with exact paths.
