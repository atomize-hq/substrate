# substrate-gateway-backend-adapter-contract — impact map

Authoring standards:

- `docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Spec manifest:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`

## Touch set (explicit)

### Create

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/manual_testing_playbook.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/adapter-selection-boundary.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/adapter-protocol-and-schema.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/parity-and-validation.md`

### Edit

- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/contracts/substrate-gateway-operator-contract.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/WORLD.md`
- `docs/USAGE.md`
- `crates/transport-api-types/src/lib.rs`
- `crates/transport-api-client/src/lib.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/tests/gateway_runtime_parity.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`

### Deprecate

- `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`

### Delete

- None

## Cascading implications (behavior and UX)

### CLI / UX

- Change: `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart` remain the only operator entrypoints while backend selection stays expressed as a stable `<kind>:<name>` id behind the gateway boundary.
  - Direct impact: no new top-level CLI surface lands in Substrate for adapter selection or adapter administration.
  - Cascading impact: human-readable status text, JSON status output, and manual playbooks must keep exit `4` reserved for required gateway or world component absence and must keep invalid adapter selection, unsupported capability, and policy denial outside that absent-state bucket.
  - Contradiction risks: archived command orderings such as `substrate world sync gateway` and `substrate world status gateway` remain wrong, and any operator text that treats provider ids or wrapper ids as stable CLI-facing identities recreates ADR-0024 drift.

### Config / env vars / paths

- Change: adapter selection continues to consume the existing ADR-0027 config and policy roots, the existing one-file-per-backend inventory model, and the existing backend-id grammar.
  - Direct impact: no new config family, no new policy family, and no new backend filename grammar land in this feature.
  - Cascading impact: the feature-local contract and policy docs must define adapter-missing, adapter-unsupported, and capability-unsatisfied behavior without inventing new YAML keys; the Substrate-owned non-secret wiring exports remain `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`; host credential reads remain gated by `agents.host_credentials.read.allowed_backends`.
  - Contradiction risks: any draft or spec that points at `docs/project_management/packs/active/llm_and_agent_config_policy_surface/*` in this checkout creates link drift because the live pack path is `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*`; any text that overloads `llm.routing.default_backend` with router, provider, auth-authority, or protocol meaning conflicts with ADR-0042 and ADR-0043.

### Policy / isolation / security posture

- Change: allowlisting, fail-closed routing, host-to-world secret delivery, and trusted-input boundaries remain Substrate-owned, while adapter internals remain `substrate-gateway` implementation details.
  - Direct impact: invalid integration state, dependency unavailability, and policy denial remain separate outcomes for adapter-backed execution.
  - Cascading impact: Substrate status/lifecycle code must refuse to trust gateway-local config, admin mutation surfaces, token persistence, or session storage as policy inputs; adapter capability checks must map to explicit failure classes instead of collapsing into gateway absence; world-required routing remains fail-closed when policy demands in-world execution.
  - Contradiction risks: any host fallback path that bypasses world-required policy, any control path that authorizes execution from gateway-local state, and any leak of secret or session-handle data into operator-visible status or logs breaks the contract.

### Runtime / protocol / trace

- Change: adapter protocol semantics, capability advertisement, extension keys, request and response schema, and session-handle lifecycle move into new feature-local specs while normalized event envelopes and canonical trace vocabulary remain owned by ADR-0017 and ADR-0028.
  - Direct impact: the feature must author exact local protocol and schema docs before any typed runtime surface widens.
  - Cascading impact: `GatewayLifecycleResponseV1` and `substrate world gateway status --json` need one owner decision for any additive adapter metadata before code lands; backend ids remain `<kind>:<name>` selectors in trace and events; tuple fields such as `client`, `router`, `provider`, `auth_authority`, and `protocol` remain external vocabulary rather than new local schema keys.
  - Contradiction risks: widening `status --json` without an owning schema doc, redefining `backend_id`, or introducing adapter-native event fields that compete with ADR-0017 or ADR-0028 recreates multi-owner drift.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs

- ADR: `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - Overlap surfaces: operator command family, typed lifecycle and status runtime boundary, host-to-world secret delivery ownership, and the no-second-control-plane rule.
  - Conflict: no
  - Resolution: ADR-0040 remains the owner of the Substrate versus `substrate-gateway` boundary; this feature owns the adapter contract that sits under that boundary and must not redefine operator command semantics or world-placement authority.

- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - Overlap surfaces: backend-id grammar, `llm.routing.default_backend`, `llm.allowed_backends`, backend inventory files, and `agents.host_credentials.read.allowed_backends`.
  - Conflict: no
  - Resolution: ADR-0027 remains the single owner for config and policy roots plus backend-id grammar; this feature consumes those surfaces and does not add YAML keys or alternate inventory models.

- ADR: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - Overlap surfaces: structured event envelope, output-class separation, and operator-visible event attribution.
  - Conflict: no
  - Resolution: adapter protocol docs must bind local event-translation rules to ADR-0017 and must not define a second event envelope.

- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces: `backend_id`, canonical correlation vocabulary, and trace-family ownership.
  - Conflict: no
  - Resolution: adapter protocol and schema docs must reference ADR-0028 vocabulary exactly and must not redefine trace-field names, requiredness, or correlation rules.

- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Overlap surfaces: `cli:codex` as the first backend, CLI session semantics, subscription-authenticated fulfillment, and archived adapter-contract text that assumes a Substrate-local engine seam.
  - Conflict: yes
  - Resolution:
    - Option A: rewrite the ADR-0024-era archived planning set so every file mirrors the gateway-hosted adapter contract.
    - Option B: keep ADR-0024 and its archived planning set as frozen historical evidence, deprecate ADR-0024 as an architectural source, and make `compatibility-spec.md` the live supersession map.
    - Selected option: B

- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces: stable backend ids versus tuple fields `client`, `router`, `provider`, `auth_authority`, and `protocol`.
  - Conflict: yes
  - Resolution:
    - Option A: absorb tuple semantics into ADR-0041 local docs and status surfaces.
    - Option B: keep ADR-0041 limited to stable backend ids plus adapter protocol and defer tuple semantics to ADR-0042.
    - Selected option: B

- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces: tuple-axis policy constraints under `llm.constraints.*`.
  - Conflict: yes
  - Resolution:
    - Option A: extend ADR-0041 local policy docs to define router, provider, protocol, and auth-authority YAML constraints.
    - Option B: keep ADR-0041 on backend-id selection and adapter capability gating only, and leave tuple-axis policy keys to ADR-0043.
    - Selected option: B

### Relevant Planning Packs

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
  - Overlap surfaces: operator contract, policy-evaluation boundary, runtime parity, typed lifecycle routes, and world-backed status semantics.
  - Conflict: yes
  - Resolution:
    - Option A: widen the boundary pack and `status --json` ownership to carry adapter capability metadata under ADR-0041.
    - Option B: keep the boundary pack and committed contract docs as-is, keep `status --json` envelope ownership unchanged, and place adapter capability and session semantics in the new ADR-0041 local docs until a new status-schema owner is declared.
    - Selected option: B

- Planning Pack: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/`
  - Overlap surfaces: backend-id grammar, allowlists, inventory path rules, and host credential read gating.
  - Conflict: yes
  - Resolution:
    - Option A: normalize every stale `packs/active/llm_and_agent_config_policy_surface/*` reference across all adjacent ADRs and docs in the same pass.
    - Option B: fix ADR-0041 and the new ADR-0041 local docs to point at `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*`, then log broader registry cleanup as a follow-up.
    - Selected option: B

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces: backend-id trace vocabulary, event attribution, and platform-parity evidence for world-backed execution.
  - Conflict: no
  - Resolution: ADR-0041 consumes the existing trace vocabulary and platform evidence; its local docs must reference those contracts and must not redefine the trace layer.

- Planning Pack: `docs/project_management/_archived/next/llm_cli_backend_engine/`
  - Overlap surfaces: archived adapter contract, CLI session mode assumptions, and `cli:codex` v1 fulfillment shape.
  - Conflict: yes
  - Resolution: treat the archived pack as evidence-only, keep it out of current ownership, and route all live contract truth through ADR-0041 plus `compatibility-spec.md`.

- Planning Pack: `docs/project_management/_archived/next/llm_gateway_in_world/`
  - Overlap surfaces: gateway command ordering history, client wiring history, secret-delivery history, and host-reachability language.
  - Conflict: yes
  - Resolution: treat the archived pack as evidence-only, preserve the current `substrate world gateway ...` command ordering, preserve in-world wiring posture, and refuse archived host-reachability or command-order assumptions.

## Follow-ups

- Decision or clarification follow-ups:
  - Define the owning document for any additive adapter metadata on `substrate world gateway status --json` before editing `crates/transport-api-types/src/lib.rs`.
  - Scaffold pre-planning packs for `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/` and `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/` before parallel planning starts on ADR-0042 or ADR-0043.

- Spec updates required:
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` — replace stale `packs/active/llm_and_agent_config_policy_surface/*` links with `packs/implemented/llm_and_agent_config_policy_surface/*`.
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md` — record ADR-0024 and `docs/project_management/_archived/next/llm_cli_backend_engine/*` as historical evidence-only surfaces.
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` — define the exact fail-closed capability-validation order and the exact boundary to ADR-0017 and ADR-0028.
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md` — pin the adopted Unified Agent API subset, the extension-key set, and the session-handle facet schema.
