# llm-and-agent-identity-tuple-and-deployment-posture — spec manifest (pre-planning)

This file enumerates every contract, schema, policy, telemetry, compatibility, and platform surface touched by ADR-0042 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
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

This manifest does not require `plan.md`, `tasks.json`, kickoff prompts, execution-ownership registries, or other legacy full-planning artifacts.

## Required documents (authoritative)

### Pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the required-doc set for ADR-0042
    - the one-owner-per-surface matrix
    - the explicit list of reused external authorities
    - the explicit list of unselected doc classes
  - Must define:
    - every local doc required by this feature directory
    - every externally owned surface reused by this feature
    - every follow-up that blocks deterministic downstream seam planning

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the touch set for the selected doc set
    - the downstream implication map across ADR-0027, ADR-0028, ADR-0040, ADR-0041, and ADR-0043
  - Must define:
    - every local doc expected to be created
    - every external contract that later work will need to align before implementation
    - the exact drift risks created by stale external-owner language

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the shared defaults and invariants that every selected local spec must reuse
  - Must define:
    - the adopted identity-tuple vocabulary
    - the adopted placement-posture vocabulary
    - the adopted external-owner boundary rules

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/workstream_triage.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the advisory sequencing for downstream seam planning
  - Must define:
    - the dependency order across contract-and-schema, policy-and-observability, and rollout-and-validation work
    - the cross-doc conflict points that later seam planning must gate

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/alignment_report.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the wrapper-facing summary of pre-planning gaps and hard gates
  - Must define:
    - every blocker that prevents downstream seam planning from starting
    - every external doc that needs alignment review before promotion

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the checkpoint cadence for this cross-platform, security-sensitive doc set
  - Must define:
    - the document-validation gate before seam planning starts
    - the external-alignment gate for status-schema and trace-vocabulary widening
    - the micro-lint and ambiguity-scan gates for all selected authored docs

### Topic-specific specs required by ADR-0042

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the operator-visible meaning of `client`, `router`, `provider`, `auth_authority`, and `protocol`
    - the operator-visible meaning of `in_world`, `host_only`, and `host_to_world_bridge`
    - the additive identity and placement wording rendered over existing gateway command surfaces
    - the rule that tuple metadata does not collapse into backend ids
    - the rule that example credential-source paths are illustrative rather than new Substrate-owned path contracts
    - the additive exit-taxonomy reuse rule for identity-tuple visibility on existing entrypoints
  - Must define:
    - the exact operator meaning of each tuple field
    - the exact operator meaning of each placement posture token
    - the exact additive human-readable status wording expected from existing gateway commands
    - the exact rule that no new commands, flags, config files, or exit codes are introduced by this feature
    - the exact rule that `host_to_world_bridge` is transport-only and not a second standing control plane

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the machine-readable identity-tuple object shape
    - the machine-readable placement-posture object shape
    - the canonical token grammar for tuple ids
    - the absence semantics for tuple and posture fields on published machine-readable surfaces
    - the compatibility posture for future tuple-field additions
  - Must define:
    - every field name, field type, default, and absence rule for the tuple object
    - every allowed placement-posture value and any related boolean or enum facets
    - the exact lowercase snake_case grammar for `client`, `router`, `provider`, and `auth_authority`
    - the exact lowercase dotted grammar for `protocol`
    - the exact boundary between the inner tuple object and externally owned status or trace envelopes

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - routing-hint evaluation semantics
    - the rule that routing hints remain requests until policy validates them
    - the rule that rejected hints do not rewrite `client` and do not create implicit provider authority
    - the direct-provider permission boundary over existing ADR-0027 keys
    - the bridge transport-only enforcement rule at the policy layer
    - the boundary between this feature and ADR-0043 tuple-axis policy keys
  - Must define:
    - the ordered evaluation inputs reused from ADR-0027
    - the exact decision flow for accepted hints, rejected hints, and policy denial
    - the exact interaction between placement posture, router identity, provider selection, and fail-closed routing
    - the exact rule that this feature reuses existing keys and does not author new policy key paths
    - the exact handoff line where ADR-0043 owns `llm.constraints.*`

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - additive trace field families for identity-tuple metadata
    - additive machine-readable status and diagnostic field placement for identity-tuple metadata outside `client_wiring.*`
    - redaction rules for tuple metadata
    - the rule that secrets remain absent from trace, status, and diagnostics by default
    - the boundary between tuple metadata and the externally owned ADR-0017 and ADR-0028 envelopes
  - Must define:
    - the exact field families and field names used for tuple and placement metadata
    - the exact field types, defaults, and absence semantics for those fields
    - the exact redaction and non-secret rules for every additive field
    - the exact rule that tuple metadata augments but does not replace canonical correlation keys
    - the exact consumer-impact notes for trace readers, status readers, and diagnostics

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - Linux parity guarantees for tuple semantics and placement posture semantics
    - macOS parity guarantees for tuple semantics and placement posture semantics
    - Windows parity guarantees for tuple semantics and placement posture semantics
    - the rule that `host_to_world_bridge` does not change in-world `net_allowed` enforcement
    - the allowed divergence list for hidden transport or bootstrap details that do not change operator semantics
  - Must define:
    - the exact guarantees that remain identical across Linux, macOS, and Windows
    - the exact platform-specific details that stay out of the public contract
    - the exact validation evidence required per platform

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the overloaded-backend-label retirement posture
    - the additive rollout rule for tuple vocabulary in new operator-facing docs and diagnostics
    - the end-state rule for historical docs that still use overloaded backend language
    - the rule that tuple vocabulary stays distinct from backend ids and future policy-axis keys
  - Must define:
    - the exact compatibility promise for existing operators
    - the exact end condition for retiring overloaded backend terminology from new docs and new diagnostics
    - the exact validation evidence that proves the feature did not create a second standing gateway concept

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`
  - Role: topic-specific validation spec required by the ADR
  - Owns:
    - the deterministic manual validation procedure for doc-level alignment
    - the example-based review over the Claude Code and Codex scenarios named by ADR-0042
    - the one-owner-per-surface proof across local docs and reused external docs
  - Must define:
    - the exact review steps that compare local docs against ADR-0027, ADR-0017, ADR-0028, ADR-0040, ADR-0041, and ADR-0043
    - the exact assertions that prove tuple fields stay distinct from backend ids
    - the exact assertions that prove `host_to_world_bridge` remains transport-only
    - the exact assertions that prove no new config family or exit-code family was introduced

### Downstream seam-planning artifacts that must exist later

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/identity-contract-and-schema.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `contract.md` and `identity-tuple-schema-spec.md`
  - Must define:
    - the execution seam for operator wording, canonical tokenization, and machine-readable object modeling
    - the acceptance boundary against externally owned gateway command and status envelopes

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/policy-and-observability-alignment.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `policy-spec.md` and `telemetry-spec.md`
  - Must define:
    - the execution seam for routing-hint evaluation, tuple-field publication, and redaction enforcement
    - the acceptance boundary against ADR-0027, ADR-0017, ADR-0028, and ADR-0043

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/seam-planning/platform-rollout-and-validation.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`
  - Must define:
    - the execution seam for parity proof, terminology rollout, and doc-validation evidence
    - the acceptance boundary for Linux, macOS, and Windows parity review

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
| Operator-visible meaning of `client`, `router`, `provider`, `auth_authority`, and `protocol` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | singular operator meaning for each tuple field |
| Operator-visible meaning of `in_world`, `host_only`, and `host_to_world_bridge` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | singular operator meaning for each posture token and the transport-only meaning of the bridge |
| Additive human-readable status and diagnostics wording for tuple and placement metadata on existing gateway entrypoints | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | rendered wording, no-new-command rule, and exit-taxonomy reuse |
| Example credential-source paths named by ADR-0042, including `~/.codex/auth.json`, as illustrative examples rather than new Substrate-owned path contracts | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | illustrative-only rule and boundary against new filesystem contract scope |
| Machine-readable identity-tuple object shape | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | field list, field types, defaults, and absence semantics |
| Machine-readable placement-posture object shape | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | value set, related facets, defaults, and absence semantics |
| Canonical token grammar for tuple ids | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | lowercase snake_case rules for `client`, `router`, `provider`, and `auth_authority`; lowercase dotted rule for `protocol` |
| Machine-readable compatibility posture for future tuple-field additions | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md` | additive-field rules and non-breaking extension boundary |
| Routing-hint request semantics and provider-selection decision flow | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | evaluation inputs, ordering, and accepted-hint behavior |
| Routing-hint rejection semantics, including no `client` rewrite and no implicit provider authority | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | rejected-hint behavior and failure posture |
| Direct-provider fulfillment permission boundary and its interaction with router identity and placement posture | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | explicit permission rule, policy gate, and router/posture interaction |
| Bridge transport-only enforcement at the policy layer | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md` | no-second-control-plane rule and no implicit authority escalation |
| Tuple-axis policy key paths under `llm.constraints.*` | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` | key names, defaults, narrowing behavior, and the boundary against ADR-0042 |
| Additive trace field family for tuple metadata | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | field names, field types, defaults, and absence semantics for trace publication |
| Additive machine-readable status and diagnostics field placement for tuple metadata outside `client_wiring.*` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md` | field placement, publication rules, and boundary against the externally owned status envelope |
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
- define exact names, values, and constraints for every serialized boundary it owns
- define failure posture and taxonomy reuse where the surface is operator-visible
- define security and redaction invariants for any published metadata
- define the boundary against reused external owners so local docs do not restate external truth
- define platform guarantees where the surface is exposed on Linux, macOS, and Windows

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:

- `env-vars-spec.md`
  - ADR-0042 introduces no new environment variables.
  - Existing env and host-credential read gates remain owned by `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` and `docs/contracts/substrate-gateway-policy-evaluation.md`.

- `<topic>-protocol-spec.md`
  - ADR-0042 names protocol identity tokens as metadata.
  - ADR-0042 does not define wire-level request or response shapes, headers, retries, or version negotiation.

- `filesystem-semantics-spec.md`
  - ADR-0042 introduces no new Substrate-owned file paths, file formats, protected-path rules, or path-resolution rules.
  - Example credential-source paths remain illustrative under `contract.md`.

- `decision_register.md`
  - ADR-0042 records one body-of-work decision in the ADR itself.
  - The unresolved items in this manifest are follow-up clarifications, not a second architecture-selection track.

## Follow-ups

- Resolve the canonical router identity for host-only direct-provider fulfillment. ADR-0042 names `direct_provider_path` in one example and also states that host-only is a mode of the router rather than a second standing authority.
- Resolve absence semantics for `provider` and `auth_authority` when routing-hint validation fails before provider selection and when agent-only flows surface tuple metadata.
- Align additive `status --json` tuple fields with `docs/contracts/substrate-gateway-status-schema.md` before downstream runtime or typed-model work widens the published JSON envelope.
- Align additive trace-field placement with `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` before downstream runtime work publishes new trace fields.
