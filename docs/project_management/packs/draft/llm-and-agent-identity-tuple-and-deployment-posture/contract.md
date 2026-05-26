# llm-and-agent-identity-tuple-and-deployment-posture — contract

This document is the pack-local operator-facing contract summary for ADR-0042.

Authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

## Contract surface

This pack locks:
- one operator-visible identity tuple with the field names `client`, `router`, `provider`, `auth_authority`, and `protocol`
- one placement-posture vocabulary with the tokens `in_world`, `host_only`, and `host_to_world_bridge`
- one machine-readable object-name baseline: `identity_tuple` and `placement_posture`
- one current router-id baseline for this lane: `substrate_gateway` and `direct_provider_path`
- one human-readable wording baseline for status and diagnostics that later docs reuse verbatim
- one illustrative-only rule for example credential paths such as `~/.codex/auth.json`

## Identity tuple meanings

- `client`
  - Meaning: the originating runtime or caller surface that issued the request.
  - Examples: `codex`, `claude_code`

- `router`
  - Meaning: the routing authority that accepted the request and selected fulfillment.
  - Current locked router ids in this lane:
    - `substrate_gateway`: request routing passed through the Substrate-managed gateway boundary.
    - `direct_provider_path`: Substrate authorized direct provider fulfillment with no `substrate_gateway` mediation. This id records routing authority only. It does not describe a standing daemon or a second control plane.

- `provider`
  - Meaning: the upstream service that fulfilled the request after routing completed.
  - Examples: `openai`, `anthropic`, `azure_openai`

- `auth_authority`
  - Meaning: the credential or billing authority under which the request was authorized.
  - Examples: `codex_subscription`, `anthropic_api_key`, `gateway_delegated_secret`

- `protocol`
  - Meaning: the request and response contract or capability surface spoken by the routed request.
  - Examples: `openai.responses`, `anthropic.messages`, `substrate.agent.session`

Field invariants:
- `backend_id` remains an adapter selector only in the existing `<kind>:<name>` grammar. It does not replace any tuple field.
- `auth_authority` remains distinct from both `client` and `provider`.
- `protocol` remains capability metadata. It does not grant routing or credential authority.
- `provider` and `auth_authority` remain independently omittable when the effective value is unresolved or not applicable to the reported surface.

## Placement-posture meanings

- `in_world`
  - Meaning: fulfillment runs inside the world boundary.

- `host_only`
  - Meaning: fulfillment runs on the host because effective policy permitted host execution.
  - `host_only` describes execution placement only. It does not define router identity.

- `host_to_world_bridge`
  - Meaning: host-scoped control surfaces reach in-world resources through transport glue.
  - `host_to_world_bridge` is transport only. It does not define router identity and it does not create a second control plane.

Placement invariants:
- `router=direct_provider_path` requires `host_only`.
- `router=substrate_gateway` is valid with `in_world` or `host_only`.
- `host_only` does not imply `direct_provider_path`.
- `host_to_world_bridge` only describes transport attached to in-world fulfillment.

## Rendering baseline

Human-readable status and diagnostics reuse the following labels:
- `originating client`
- `routing authority`
- `fulfillment provider`
- `auth authority`
- `protocol`
- `deployment posture`
- `bridge transport`

Rendering rules:
- Machine-readable surfaces publish normalized ids only.
- Human-readable surfaces keep the normalized id unchanged when they render a prose label.
- Omitted optional tuple fields stay omitted in machine-readable output and absent in human-readable output. Surfaces do not backfill placeholder text such as `unknown`.
- Human-readable surfaces do not rename `router` to `backend`, do not rename `provider` to `client`, and do not rename `host_only` to `host gateway`.
- `~/.codex/auth.json` and similar credential-source paths remain illustrative examples only. A rendered example path is never a required Substrate-owned filesystem contract.

## Routing and authority rules

- A routing hint is a request only. It is not authority.
- Accepted routing hints change effective provider selection only after policy validation.
- Rejected routing hints do not rewrite `client`, `router`, or `auth_authority`.
- Rejected routing hints do not create implicit provider authority.
- Direct provider fulfillment remains policy-gated. It is not inferred from `provider`, `protocol`, or placement posture alone.
- `host_to_world_bridge` does not relax the in-world `net_allowed` governance owned by downstream parity and policy surfaces.

## Publication and owner boundaries

- `identity-tuple-schema-spec.md` owns the machine-readable object names `identity_tuple` and `placement_posture`, their field lists, token grammar, and omission rules.
- `telemetry-spec.md` owns placement and projection of `identity_tuple` and `placement_posture` onto status, diagnostics, and trace surfaces.
- `docs/contracts/gateway/status-schema.md` remains the owner of the top-level `status --json` envelope and the `client_wiring.*` field family.
- `docs/contracts/gateway/operator-contract.md` remains the owner of the gateway command family and exit-code taxonomy reuse.
- `docs/contracts/gateway/policy-evaluation.md` remains the owner of policy evaluation over existing ADR-0027 keys.
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` and `SCHEMA.md` remain the owners of config roots, policy roots, precedence, and backend-id grammar.
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` remains the owner of tuple-axis policy keys under `llm.constraints.*`.
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` remains the owner of canonical correlation vocabulary and join keys.

## Concrete examples

### Gateway-routed in-world fulfillment

- `client=codex`
- `router=substrate_gateway`
- `provider=openai`
- `auth_authority=codex_subscription`
- `protocol=openai.responses`
- `deployment posture=in_world`

### Host-only direct provider fulfillment

- `client=codex`
- `router=direct_provider_path`
- `provider=openai`
- `auth_authority=codex_subscription`
- `protocol=openai.responses`
- `deployment posture=host_only`

Rule:
- this path records direct provider fulfillment without `substrate_gateway` mediation and without a second standing host gateway

### Pre-provider-selection publication

- `client=codex`
- `router=substrate_gateway`
- `protocol=openai.responses`
- `provider` omitted
- `auth_authority` omitted
- `deployment posture=in_world`

Rule:
- omission records unresolved or not-applicable values only. It does not change the meaning of the remaining tuple fields.
