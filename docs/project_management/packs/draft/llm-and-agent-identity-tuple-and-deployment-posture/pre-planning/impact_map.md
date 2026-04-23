# llm-and-agent-identity-tuple-and-deployment-posture — impact map

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

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

## Canonical slice order

Canonical slice ids selected for this feature:
- `LAITDP0` — identity contract and schema lock
- `LAITDP1` — policy and observability alignment lock
- `LAITDP2` — platform rollout and validation lock

## Touch set (explicit)

### Create
- None

### Edit
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/quality_gate_report.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/kickoff_prompts/CP1-ci-checkpoint.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/kickoff_prompts/CP2-ci-checkpoint.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/kickoff_prompts/FZ-feature-cleanup.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/LAITDP0-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/kickoff_prompts/LAITDP0-code.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/kickoff_prompts/LAITDP0-test.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP0/kickoff_prompts/LAITDP0-integ.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/LAITDP1-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-code.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-test.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-integ-core.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-integ-linux.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-integ-macos.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-integ-windows.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP1/kickoff_prompts/LAITDP1-integ.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/LAITDP2-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-code.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-test.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-integ-core.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-integ-linux.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-integ-macos.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-integ-windows.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/slices/LAITDP2/kickoff_prompts/LAITDP2-integ.md`
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
- `crates/common/tests/agent_hub_event_envelope_schema.rs`
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
    - Shell passthrough code and tests preserve the additive envelope for available and unavailable responses.
    - Operator docs present `host_to_world_bridge` as transport text, never as router identity.
  - Contradiction risks:
    - adding tuple fields under `client_wiring.*`
    - reusing `backend_id` as the tuple carrier
    - presenting `host_only` as a second standing gateway
  - Full-planning slice owner:
    - `LAITDP0`

### Config / env vars / paths
- Change: this feature reuses the existing config and policy surface and adds no new config keys, env vars, commands, or filesystem contracts.
  - Direct impact:
    - `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` stay the only relevant keys for placement and credential gating.
    - Example credential paths such as `~/.codex/auth.json` stay illustrative text, not new Substrate-owned path contracts.
  - Cascading impact:
    - `docs/CONFIGURATION.md` and the implemented config-policy pack docs state the no-new-key rule and the backend-id-versus-tuple split in one voice.
    - Future ADR-0043 policy docs can add tuple-axis constraints without reopening config-root ownership.
  - Contradiction risks:
    - any doc that implies tuple axes live inside backend ids or new config families
    - any doc that promotes example auth paths into required contract paths
  - Full-planning slice owner:
    - `LAITDP0`

### Policy / isolation / security posture
- Change: routing hints stay requests validated against existing policy gates, and placement posture stays world-first unless existing policy permits `host_only`.
  - Direct impact:
    - accepted hints influence effective provider selection
    - rejected hints leave `client` unchanged and do not create provider authority
    - bridge transport text does not relax in-world `net_allowed` governance
  - Cascading impact:
    - the gateway policy-evaluation contract aligns with the feature-local policy spec on accepted-hint, rejected-hint, and denial outcomes
    - runtime status and diagnostics distinguish router authority from provider fulfillment when they expose both
  - Contradiction risks:
    - rejected hints rewriting `client` or `auth_authority`
    - tuple-axis policy keys defined in this lane
    - a bridge path that changes egress semantics
  - Full-planning slice owner:
    - `LAITDP1`

### Telemetry / status / diagnostics
- Change: trace spans and gateway status surfaces publish tuple metadata additively while preserving canonical correlation keys and redaction rules.
  - Direct impact:
    - `crates/trace/src/span.rs` and `docs/TRACE.md` gain explicit tuple and posture vocabulary
    - gateway status responses and docs gain additive non-secret metadata outside `client_wiring.*`
  - Cascading impact:
    - trace readers, status readers, and manual validation docs accept additive fields without treating them as new join keys
    - `backend_id` remains present where adapter selection matters and stays separate from tuple semantics
  - Contradiction risks:
    - emitting secrets in tuple fields
    - treating tuple fields as replacements for canonical correlation keys
    - divergent tuple field names between trace and status surfaces
  - Full-planning slice owner:
    - `LAITDP1`

### Platform parity
- Change: Linux, macOS, and Windows expose the same tuple and placement semantics even though transport plumbing remains platform-specific.
  - Direct impact:
    - no direct edits are implied for `crates/world`, `crates/world-mac-lima`, or `crates/world-windows-wsl` in this lane
    - parity proof routes through shared gateway runtime and status surfaces plus feature-local validation docs
  - Cascading impact:
    - the manual playbook calls out identical operator semantics and hidden transport-divergence boundaries
    - runtime parity tests validate the widened status envelope and the unchanged unavailable shape
  - Contradiction risks:
    - platform docs that invent different meanings for `host_only` or `host_to_world_bridge`
    - platform-specific gateway behavior that changes tuple publication shape
  - Full-planning slice owner:
    - `LAITDP2`
