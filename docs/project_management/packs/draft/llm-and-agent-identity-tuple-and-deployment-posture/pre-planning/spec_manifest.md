# llm-and-agent-identity-tuple-and-deployment-posture — spec manifest (pre-planning)

This file enumerates the pre-planning artifacts, canonical slice ids, and authoritative ownership boundaries for ADR-0042.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- External authoritative docs reused by this feature:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Slice IDs (canonical)

Canonical slice ids selected for this feature:
- Slice prefix: `LAITDP`
- `LAITDP0` — identity contract and schema lock
- `LAITDP1` — policy and observability alignment lock
- `LAITDP2` — platform rollout and validation lock

## Required spec documents (authoritative)

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/alignment_report.md`

### Current pre-planning artifact roles

- `pre-planning/spec_manifest.md`
  - Owns:
    - the required pre-planning doc set for ADR-0042
    - the one-owner-per-surface matrix
    - the explicit list of reused external authorities
  - Must define:
    - every pre-planning artifact required for the standard lane
    - every externally owned surface reused by this feature
    - every blocker that full planning must resolve

- `pre-planning/impact_map.md`
  - Owns:
    - the touch set for the current pre-planning pass
    - the downstream implication map across ADR-0027, ADR-0028, ADR-0040, ADR-0041, and ADR-0043
  - Must define:
    - every local doc expected to be created or refined during planning
    - every external contract that later work aligns before implementation
    - the drift risks created by stale external-owner language

- `pre-planning/minimal_spec_draft.md`
  - Owns:
    - the shared defaults and invariants that every selected local spec reuses
    - the draft three-slice backbone `LAITDP0` through `LAITDP2`
  - Must define:
    - the adopted identity-tuple vocabulary
    - the adopted placement-posture vocabulary
    - the draft slice boundaries that full planning expands

- `pre-planning/workstream_triage.md`
  - Owns:
    - the standard `PM_PWS_INDEX` authority for accepted slice order
    - the advisory sequencing for full planning workstreams
  - Must define:
    - the accepted slice order `LAITDP0`, `LAITDP1`, `LAITDP2`
    - the cross-doc conflict points that full planning gates

- `pre-planning/ci_checkpoint_plan.md`
  - Owns:
    - the checkpoint cadence for this cross-platform, security-sensitive doc set
  - Must define:
    - the `CP1` boundary after `LAITDP1`
    - the `CP2` boundary after `LAITDP2`
    - the document-validation, micro-lint, ambiguity-scan, and parity-review gates

- `pre-planning/alignment_report.md`
  - Owns:
    - the wrapper-facing summary of pre-planning gaps and hard gates
  - Must define:
    - every blocker that prevents full planning from starting
    - every external doc that needs alignment review before promotion

### Deferred full-planning outputs

Full planning will create or refine these docs:
- `contract.md`
- `identity-tuple-schema-spec.md`
- `policy-spec.md`
- `telemetry-spec.md`
- `platform-parity-spec.md`
- `compatibility-spec.md`
- `manual_testing_playbook.md`
- `plan.md`
- `tasks.json`
- `session_log.md`
- `quality_gate_report.md`
- `slices/LAITDP0/LAITDP0-spec.md`
- `slices/LAITDP1/LAITDP1-spec.md`
- `slices/LAITDP2/LAITDP2-spec.md`

## Contract lock decisions adopted for downstream planning

- `identity_tuple` and `placement_posture` are the canonical machine-readable object names owned by `identity-tuple-schema-spec.md`.
- `direct_provider_path` records host-only direct provider fulfillment without `substrate_gateway` mediation and requires `host_only` when tuple and posture objects are published together.
- `provider` and `auth_authority` use independent omission semantics. Omission is encoded by field absence, never by `null`, empty strings, or placeholder tokens.
- `telemetry-spec.md` owns placement and projection of `identity_tuple` and `placement_posture`. It does not redefine object names, token grammar, or omission rules.

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json` as existing operator entrypoints | `docs/contracts/substrate-gateway-operator-contract.md` | command family, stable operator meaning, and baseline exit-code taxonomy for the gateway boundary |
| `status --json` top-level envelope and `client_wiring.*` field family | `docs/contracts/substrate-gateway-status-schema.md` | top-level JSON shape, `status`, `client_wiring.*`, and absence semantics for the published wiring surface |
| Gateway policy-evaluation flow over `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` | `docs/contracts/substrate-gateway-policy-evaluation.md` | fail-closed routing, secret-source precedence, trusted-input boundary, and policy outcome classes |
| Gateway runtime lifecycle parity and hidden transport divergence | `docs/contracts/substrate-gateway-runtime-parity.md` | typed runtime boundary, lifecycle/status parity, and hidden transport divergence list |
| Config and policy file families, patch locations, and precedence order | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | file locations, precedence order, and the deny-by-default policy posture already in force |
| Existing key-path definitions for `llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key paths, types, defaults, and merge strategy |
| Backend id format and the rule that backend ids stay adapter/runtime selectors rather than tuple substitutes | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | `<kind>:<name>` format and the non-equivalence boundary against tuple semantics |
| Operator-visible meaning of `client`, `router`, `provider`, `auth_authority`, and `protocol` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | singular operator meaning for each tuple field, including the current router ids `substrate_gateway` and `direct_provider_path` |
| Operator-visible meaning of `in_world`, `host_only`, and `host_to_world_bridge` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | singular operator meaning for each posture token, the transport-only meaning of the bridge, and the rule that `direct_provider_path` requires `host_only` |
| Additive human-readable status and diagnostics wording for tuple and placement metadata on existing gateway entrypoints | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | rendered wording, no-new-command rule, and exit-taxonomy reuse |
| Example credential-source paths named by ADR-0042, including `~/.codex/auth.json`, as illustrative examples rather than new Substrate-owned path contracts | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | illustrative-only rule and boundary against new filesystem contract scope |
| Machine-readable identity-tuple object shape | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | canonical object name `identity_tuple`, required and optional fields, token grammar, and omission semantics |
| Machine-readable placement-posture object shape | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | canonical object name `placement_posture`, `execution` value set, bridge facet, defaults, and omission semantics |
| Canonical token grammar for tuple ids | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | lowercase snake_case rules for `client`, `router`, `provider`, and `auth_authority`; lowercase dotted rule for `protocol` |
| Machine-readable compatibility posture for future tuple-field additions | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | additive-field rules and non-breaking extension boundary |
| Routing-hint request semantics and provider-selection decision flow | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | evaluation inputs, ordering, and accepted-hint behavior |
| Routing-hint rejection semantics, including no `client` rewrite and no implicit provider authority | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | rejected-hint behavior and failure posture |
| Direct-provider fulfillment permission boundary and its interaction with router identity and placement posture | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | explicit permission rule, policy gate, and router/posture interaction |
| Bridge transport-only enforcement at the policy layer | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | no-second-control-plane rule and no implicit authority escalation |
| Tuple-axis policy key paths under `llm.constraints.*` | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` | key names, defaults, narrowing behavior, and the boundary against ADR-0042 |
| Additive trace field family for tuple metadata | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | placement and projection rules for `identity_tuple` and `placement_posture` in trace publication |
| Additive machine-readable status and diagnostics field placement for tuple metadata outside `client_wiring.*` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | top-level placement of `identity_tuple` and `placement_posture`, publication rules, and boundary against the externally owned status envelope |
| Redaction and non-secret rules for tuple metadata in trace, status, and diagnostics | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | redaction rules, secret absence, and safe publication bounds |
| Canonical trace vocabulary and correlation keys reused by this feature | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | correlation vocabulary, join keys, and trace-authority boundary |
| Rule that tuple metadata augments rather than replaces the canonical trace vocabulary | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | augmentation-only rule and collision avoidance with canonical keys |
| Structured event-envelope and output-routing vocabulary reused by this feature | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | event envelope, output classes, and routing vocabulary |
| Linux parity guarantee for tuple semantics and placement posture semantics | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md` | Linux guarantee and platform-specific validation evidence |
| macOS parity guarantee for tuple semantics and placement posture semantics | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md` | macOS guarantee and platform-specific validation evidence |
| Windows parity guarantee for tuple semantics and placement posture semantics | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md` | Windows guarantee and platform-specific validation evidence |
| Rule that `host_to_world_bridge` does not alter in-world `net_allowed` governance | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md` | bridge and egress invariants across platforms |
| Historical overloaded-backend-label retirement posture | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md` | terminology retirement rules and migration posture for new operator-facing docs |
| End-state rule for new docs, status surfaces, and diagnostics using tuple vocabulary | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md` | rollout end condition and proof that tuple vocabulary stays distinct from backend ids |
| Deterministic validation of the Claude Code and Codex examples named by ADR-0042 | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md` | example-based assertions and expected conclusions |
| Deterministic validation that each surface has one authoritative owner | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md` | cross-doc review procedure and expected ownership proof |

## Determinism checklist

Every selected local doc must:
- define inputs and precedence order when more than one input exists
- define defaults and absence semantics
- define data-model constraints for every serialized boundary
- define error model and failure posture
- define ordering and concurrency rules where behavior depends on sequencing
- define security and redaction invariants
- define Linux, macOS, and Windows guarantees where platform parity matters
