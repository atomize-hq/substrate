# gateway-backend-selection-runtime-integration — impact map

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- Spec manifest:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/spec_manifest.md`
- External contract docs scanned:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- Adjacent ADRs and packs scanned:
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/_archived/next/llm_gateway_in_world/`
  - `docs/project_management/_archived/next/llm_cli_backend_engine/`

## Touch set (explicit)

### Create
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/backend-selection-and-policy.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/runtime-realization-and-artifacts.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/parity-validation-and-rollout.md`

### Edit
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- `docs/contracts/substrate-gateway-operator-contract.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/CONFIGURATION.md`
- `docs/USAGE.md`
- `crates/transport-api-types/src/lib.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/tests/gateway_runtime_parity.rs`
- `crates/gateway/src/auth/`
- `crates/gateway/src/providers/`
- `crates/gateway/tests/`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior and UX)

### Selected-backend realization replaces the hardcoded Codex path
- Direct impact:
  - `substrate world gateway status`, `sync`, and `restart` continue as the only operator entrypoints.
  - The selected backend comes from effective config and policy, then resolves through inventory and one integrated adapter binding instead of one hardcoded `cli:codex` branch.
  - `status` stops reporting generic gateway availability for unsupported integrated selections when integrated gateway mode is enabled. It reports the selected-backend outcome.
- Cascading impact:
  - `crates/shell/src/builtins/world_gateway.rs` must stop returning `None` for every non-`cli:codex` backend in `resolve_integrated_auth_payload`.
  - `crates/world-service/src/gateway_runtime.rs` must stop rejecting every backend other than `cli:codex` in `GatewayControlSettings::from_request_env` and `render_integrated_config`.
  - `crates/shell/tests/world_gateway.rs` cases that treat `api:openai` as a generic available status path must move to the new classification matrix: supported backend, blocked backend, invalid backend, missing inventory, missing adapter, and missing auth.
- Contradiction risks:
  - Silent collapse back to the Codex template would violate ADR-0046 and ADR-0041.
  - A backend-specific command fork would violate the operator contract in `docs/contracts/substrate-gateway-operator-contract.md`.
  - Any additive `status --json` field family without a schema-owner update would violate `docs/contracts/substrate-gateway-status-schema.md`.

### Inventory-backed selection creates a new filesystem and discoverability dependency
- Direct impact:
  - Integrated realization now depends on one file-based backend inventory surface in addition to effective config and policy.
  - The implementation must enforce backend-id grammar, filename-to-id consistency, and missing-inventory classification before adapter dispatch.
- Cascading impact:
  - `filesystem-semantics-spec.md` must define the exact global and workspace inventory roots, filename rules, absence semantics, and generated runtime artifact paths.
  - `docs/CONFIGURATION.md` must publish the operator-facing discovery path for the backend inventory roots and the rule that gateway-local persistence does not authorize execution.
  - `crates/world-service/src/service.rs` and `crates/world-service/src/gateway_runtime.rs` must agree on where inventory lookup happens so `status`, `sync`, and `restart` share one resolution order.
- Contradiction risks:
  - The current repo documents agent inventory roots and deps inventory roots, but it does not publish backend inventory roots. Leaving that gap open would make the ADR claim inventory-backed realization without an operator discoverability path.
  - Treating gateway-local config files as the inventory source would violate `docs/contracts/substrate-gateway-policy-evaluation.md`.

### Auth handoff stops being Codex-only and becomes backend-aware
- Direct impact:
  - `GatewayCliCodexIntegratedAuthV1`, `GatewayIntegratedAuthPayloadV1`, and `GatewayRuntimeStartContext.integrated_auth` no longer describe the full integrated auth surface.
  - The shell path must source auth material per backend using the existing gates: `llm.secrets.env_allowed` and `agents.host_credentials.read.allowed_backends`.
  - The world-service path must validate adapter-owned auth handoff kinds fail-closed before launch.
- Cascading impact:
  - `crates/transport-api-types/src/lib.rs` must widen the integrated auth payload model.
  - `crates/shell/src/builtins/world_gateway.rs` must classify missing env reads, blocked env reads, blocked host credential reads, incomplete handoff payloads, and unsupported handoff kinds into the existing exit-code buckets.
  - `crates/gateway/src/auth/`, `crates/gateway/src/providers/`, and `crates/gateway/tests/` enter the touch set because the integrated gateway consumer side is Codex-shaped today.
- Contradiction risks:
  - Pulling tuple semantics into auth payload names would violate ADR-0042.
  - Replacing the existing policy gates with backend-specific ad hoc reads would violate ADR-0027.
  - Extending the secret channel design beyond the bounded handoff required here would violate the ADR non-goals.

### Runtime config rendering and capability gating move under one adapter-driven seam
- Direct impact:
  - `render_integrated_config` stops emitting one static Codex/OpenAI template.
  - `start_runtime` must accept one adapter-owned config payload and one adapter-owned auth payload for the selected backend.
  - The runtime manager keeps one lifecycle contract while the adapter-specific config and auth internals vary by binding.
- Cascading impact:
  - `gateway-runtime-adapter-protocol-spec.md` must define the exact order: selection passed in, adapter lookup, capability gate, auth resolution, config render, launch, readiness probe, restart.
  - `gateway-runtime-adapter-schema-spec.md` must define binding metadata, capability-set metadata, auth payload variants, config payload variants, and failure shapes.
  - `crates/world-service/tests/gateway_runtime_parity.rs` must move from the current Codex-only fixture to one matrix with at least one additional supported backend.
- Contradiction risks:
  - Reusing the adapter contract docs as implementation notes without freezing the integrated lifecycle subset would blur owner lines between ADR-0041 and ADR-0046.
  - Launch-time fallback from missing adapter binding to `unavailable` would collapse dependency-unavailable and invalid-integration outcomes.

### Operator docs and validation surfaces expand from one backend proof to a backend matrix
- Direct impact:
  - `cli:codex` remains the regression baseline.
  - The first additional integrated backend becomes part of the operator proof for `status`, `sync`, and `restart`.
  - The playbook and smoke scripts need explicit assertions for invalid backend, blocked backend, missing inventory, missing adapter binding, and missing auth.
- Cascading impact:
  - `compatibility-spec.md` must pin the regression promise for `cli:codex` and the explicit failure posture for unsupported backends.
  - `platform-parity-spec.md` and the three smoke scripts must capture Linux, macOS, and Windows evidence under the same command family.
  - `docs/USAGE.md` must state that the command family realizes the selected backend rather than a single baked-in runtime.
- Contradiction risks:
  - Status-schema widening for tuple metadata or placement metadata belongs to ADR-0042, not this feature.
  - Tuple-axis policy constraints belong to ADR-0043, not this feature.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs
- ADR: `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - Overlap surfaces: command family, lifecycle owner line, `status --json`, readiness, runtime parity, and host-to-world secret delivery ownership.
  - Conflict: no
  - Resolution: ADR-0040 remains the owner of the lifecycle boundary and the status envelope. ADR-0046 realizes backend selection and runtime rendering under that boundary.

- ADR: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - Overlap surfaces: stable backend ids, one-backend-id-to-one-adapter binding, capability gating, adapter protocol, and adapter schema.
  - Conflict: no
  - Resolution: ADR-0041 remains the contract owner. ADR-0046 realizes those contracts inside the shell and world-service lifecycle path without redefining backend-id semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - Overlap surfaces: `llm.routing.default_backend`, `llm.allowed_backends`, `llm.secrets.env_allowed`, `agents.host_credentials.read.allowed_backends`, file families, and fail-closed posture.
  - Conflict: no
  - Resolution: ADR-0027 remains the only config and policy owner. ADR-0046 adds no new config keys and no new policy keys.

- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Overlap surfaces: pre-gateway engine assumptions and Codex-first backend realization.
  - Conflict: yes
  - Resolution:
    - Option A: revive ADR-0024 semantics and treat ADR-0046 as an engine-follow-on.
    - Option B: keep ADR-0024 as historical evidence and route all live implementation truth through ADR-0041 plus ADR-0046.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces: identity tuple vocabulary, placement posture, and additive status metadata.
  - Conflict: yes
  - Resolution:
    - Option A: add tuple metadata to `status --json` and bake tuple semantics into backend realization docs here.
    - Option B: keep tuple metadata and placement-posture widening in ADR-0042 and keep ADR-0046 limited to backend selection, auth handoff, runtime render, and parity validation.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces: tuple-axis policy constraints under `llm.constraints.*`.
  - Conflict: yes
  - Resolution:
    - Option A: add tuple-axis policy constraints inside ADR-0046 local specs.
    - Option B: reuse only the existing backend and secret-read gates here and leave tuple-axis policy to ADR-0043.
    - Selected option: B.

### Relevant Planning Packs
- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
  - Overlap surfaces: operator contract, policy-evaluation owner line, runtime parity baseline, and `status --json` envelope.
  - Conflict: yes
  - Resolution:
    - Option A: widen the boundary pack so it owns selected-backend realization internals.
    - Option B: keep the boundary pack as the lifecycle/status owner and treat ADR-0046 as the realization seam under that owner line.
    - Selected option: B.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
  - Overlap surfaces: selection contract, capability contract, protocol contract, schema contract, compatibility baseline, and parity contract.
  - Conflict: yes
  - Resolution:
    - Option A: widen the adapter-contract pack so it absorbs the shell/world-service implementation seam.
    - Option B: keep that pack as contract truth and use ADR-0046 for the shell/world-service realization seam.
    - Selected option: B.

- Planning Pack: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/`
  - Overlap surfaces: config roots, policy roots, allowlists, fail-closed posture, and secret-read gates.
  - Conflict: no
  - Resolution: the implemented pack remains authoritative. ADR-0046 depends on it and must not drift from it.

- Planning Pack: `docs/project_management/_archived/next/llm_gateway_in_world/`
  - Overlap surfaces: archived gateway command wording, early secret-delivery framing, and early runtime assumptions.
  - Conflict: yes
  - Resolution:
    - Option A: reuse archived command and runtime wording when operator docs are updated.
    - Option B: keep the archive evidence-only and align operator docs only to the live gateway contracts.
    - Selected option: B.

- Planning Pack: `docs/project_management/_archived/next/llm_cli_backend_engine/`
  - Overlap surfaces: pre-gateway backend engine assumptions and earlier Codex-specific control planes.
  - Conflict: yes
  - Resolution:
    - Option A: pull archived engine semantics back into the live implementation seam.
    - Option B: keep the archive evidence-only and route live compatibility language through `compatibility-spec.md`.
    - Selected option: B.

## Follow-ups
- Pin the first supported non-`cli:codex` integrated backend id. That decision tightens `crates/gateway/src/auth/`, `crates/gateway/src/providers/`, and `crates/gateway/tests/` from directory-prefix entries to exact files.
- Define the exact backend inventory roots and filename rules in `filesystem-semantics-spec.md`, then mirror that wording in `docs/CONFIGURATION.md`.
- Keep `status --json` unchanged unless the status-schema owner explicitly widens `docs/contracts/substrate-gateway-status-schema.md`.
- Keep tuple metadata and tuple-axis policy keys out of this feature-local doc set. Route those surfaces through ADR-0042 and ADR-0043 only.
