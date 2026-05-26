# substrate-gateway-backend-adapter-contract — spec manifest (pre-planning)

This file enumerates every contract, protocol, schema, policy-evaluation, compatibility, and platform surface touched by ADR-0041 and assigns each surface to exactly one authoritative document.

Authoring standards:

- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Inputs

- Feature directory: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- External authoritative docs reused by this feature:
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/policy-evaluation.md`
  - `docs/contracts/gateway/runtime-parity.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

This manifest does not require `plan.md`, `tasks.json`, kickoff prompts, execution-ownership registries, or other legacy full-planning artifacts.

## Required documents (authoritative)

### Pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the required doc set for ADR-0041
    - the one-owner-per-surface matrix
    - the explicit list of unselected doc classes
  - Must define:
    - every feature-local doc required by this body of work
    - every externally owned surface reused by this body of work
    - every follow-up that blocks deterministic downstream planning

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the touch set for the selected doc set
    - the downstream implications across ADR-0027, ADR-0040, ADR-0017, and ADR-0028
  - Must define:
    - every feature-local doc expected to be created
    - every external doc that this pack constrains or depends on
    - the exact drift risks that stale external references create

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/minimal_spec_draft.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the cross-document defaults and invariants that every selected local spec must share
  - Must define:
    - the adopted backend-id vocabulary
    - the adopted fail-closed posture vocabulary
    - the adopted boundary rules for external-owner references

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the advisory sequencing for downstream seam planning
  - Must define:
    - the dependency order across adapter selection, protocol/schema, and parity/validation work
    - the conflict points between local docs and externally owned contracts

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/alignment_report.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the wrapper-facing summary of pre-planning gaps and hard gates
  - Must define:
    - every blocker that prevents downstream seam decomposition
    - the exact external docs that need alignment review before promotion

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/ci_checkpoint_plan.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the checkpoint cadence for this cross-platform, security-sensitive doc set
  - Must define:
    - the document-validation gate before seam planning starts
    - the parity-validation gate for Linux, macOS, and Windows surfaces
    - the doc-lint and ambiguity-scan gates for all selected authored docs

### Topic-specific specs required by ADR-0041

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the adapter-contract subset of the user-facing gateway contract that ADR-0041 changes
    - the rule that one stable `<kind>:<name>` backend id maps to one gateway adapter contract identity
    - the rule that the adapter contract does not split one backend id into planner, executor, provider, or wrapper-visible sub-identities
    - the adapter-selection-specific meaning of exit codes `2`, `4`, and `5`
    - the rule that gateway-local session strategy, prompt shaping, provider quirks, and admin surfaces stay internal to `substrate-gateway`
    - the rule that backend identity fields and selection surfaces do not expose secrets
    - the no-second-control-plane invariant for the Substrate boundary
  - Must define:
    - the exact stable backend-id semantics reused by operator-visible selection surfaces
    - the exact failure buckets for invalid adapter selection, unavailable adapter components, and policy denial
    - the exact boundary between external gateway operator docs and this feature-local adapter contract
    - the exact invariant that future adapter additions preserve the same identity contract

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - adapter selection over existing ADR-0027 config and policy inputs
    - allowlist gating before adapter dispatch
    - adapter-missing, adapter-unsupported, and capability-unsatisfied failure posture
    - the rule that backend inventory lookup uses the stable backend id and existing inventory files
    - the rule that gateway-local admin, config, and persistence surfaces are not trusted policy inputs for adapter authorization
  - Must define:
    - the exact ordered evaluation inputs for adapter selection
    - the exact decision flow that distinguishes invalid selection, unavailable dependency, and policy denial
    - the exact absence semantics for missing inventory items and missing adapters
    - the exact relationship to the externally owned gateway policy-evaluation contract

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the gateway-internal adapter protocol for request dispatch and response translation
    - the capability-validation flow used during execution
    - the ordering rules for request normalization, adapter dispatch, event translation, and response emission
    - the session-handle lifecycle rules
    - the versioning posture for adapter protocol evolution
  - Must define:
    - the exact dispatch lifecycle from selected backend id to adapter invocation
    - the exact fail-closed behavior for unsupported extension keys or required capabilities
    - the exact retry posture and ordering rules
    - the exact boundary between local event-translation rules and the externally owned ADR-0017 event envelope
    - the exact boundary between local adapter execution and the externally owned ADR-0028 trace vocabulary

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - adapter descriptor schema
    - capability advertisement schema
    - adopted versioned extension-key schema
    - request payload schema
    - response payload schema
    - adapter error object schema
    - backend-defined session-handle facet schema
  - Must define:
    - every field name, field type, default, and absence rule
    - the exact adopted Unified Agent API subset for capability ids and extension keys
    - the exact session-handle facet keys and bounds
    - the exact canonicalization rules for stable backend ids inside serialized adapter objects

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - Linux, macOS, and Windows parity guarantees for adapter-backed execution
    - the rule that stable backend ids and allowlist semantics remain invariant across platforms
    - the rule that in-world adapter execution occurs through `substrate-gateway` when world execution is required
  - Must define:
    - the exact guarantees that are identical across all three platforms
    - the exact hidden transport or bootstrap divergences that remain out of contract
    - the exact validation evidence required per platform

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - ADR-0024 supersession posture
    - the additive rollout and compatibility policy for this adapter contract
    - the rule that future `cli:*` and `api:*` adapters extend the contract without changing existing backend-id semantics
  - Must define:
    - the exact compatibility promise for existing operator workflows
    - the exact end state for ADR-0024 architectural intent
    - the exact validation evidence that proves no second Substrate control plane has been introduced

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/manual_testing_playbook.md`
  - Role: topic-specific validation spec required by the ADR
  - Owns:
    - the deterministic manual validation procedure for doc-level contract alignment
  - Must define:
    - the review checklist that compares ADR-0041 against ADR-0040, ADR-0027, ADR-0017, ADR-0028, and the gateway contract docs
    - the exact assertions for one-owner-per-surface coverage
    - the exact assertions that prove capability/session semantics stay inside the gateway adapter boundary

### Downstream seam-planning and decomposition artifacts that must exist later

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/adapter-selection-boundary.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `contract.md` and `policy-spec.md`
    - planning for the external-owner references to ADR-0027 and the gateway boundary contracts
  - Must define:
    - the implementation seam for backend-id resolution, allowlist gating, and failure mapping
    - the validation boundary against gateway-local control-plane leakage

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/adapter-protocol-and-schema.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `gateway-backend-adapter-protocol-spec.md` and `gateway-backend-adapter-schema-spec.md`
  - Must define:
    - the implementation seam for adapter registry lookup, capability validation, session-handle surfacing, and event translation
    - the acceptance boundary against ADR-0017 and ADR-0028

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/seam-planning/parity-and-validation.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`
  - Must define:
    - the implementation seam for parity proof, compatibility proof, and doc-level validation evidence
    - the acceptance boundary for Linux, macOS, and Windows parity review

## Coverage matrix (surface → authoritative doc)

| Surface                                                                                                                                                                  | Authoritative doc                                                                                                         | What must be explicitly defined                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json` as operator entrypoints | `docs/contracts/gateway/operator-contract.md`                                                                   | command family, operator meaning, stable entrypoint status, and gateway lifecycle exit-code baseline |
| Human-readable gateway availability and wiring discovery posture                                                                                                         | `docs/contracts/gateway/operator-contract.md`                                                                   | operator-visible wording, stable wiring discovery posture, and absent-state contract                 |
| `status --json` top-level envelope and `client_wiring.*` field family                                                                                                    | `docs/contracts/gateway/status-schema.md`                                                                       | field names, field types, and absence semantics                                                      |
| Gateway/world placement evaluation and host-to-world secret delivery boundary                                                                                            | `docs/contracts/gateway/policy-evaluation.md`                                                                   | gateway policy-evaluation rules that ADR-0041 reuses without redefining                              |
| Gateway lifecycle runtime parity and hidden transport divergence                                                                                                         | `docs/contracts/gateway/runtime-parity.md`                                                                      | typed runtime boundary and general gateway lifecycle parity                                          |
| Config file families and precedence for config and policy patches                                                                                                        | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`                               | file locations and precedence order                                                                  |
| Backend inventory file family and filename-to-id matching                                                                                                                | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`                                 | inventory locations, filename/id matching rule, and strictness                                       |
| `llm.routing.default_backend` key path, type, default, and `<kind>:<name>` syntax                                                                                        | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`                                 | schema rule and default                                                                              |
| `llm.allowed_backends` key path, type, default, and deny-by-default storage rule                                                                                         | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`                                 | schema rule and default                                                                              |
| Stable backend-id semantics inside the adapter contract                                                                                                                  | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`                              | one backend id to one adapter identity, no split into planner/provider/wrapper-visible sub-ids       |
| Adapter-selection-specific interpretation of exit codes `2`, `4`, and `5`                                                                                                | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`                              | invalid selection, unavailable adapter component, and policy-denial meanings                         |
| Ban on secrets in backend identity fields and adapter selection surfaces                                                                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`                              | non-secret identity contract                                                                         |
| Ban on exposing gateway-local session strategy, prompt shaping, provider quirks, or admin surfaces as Substrate contract                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`                              | internal-only boundary and no-second-control-plane invariant                                         |
| Adapter selection over config, policy, and inventory inputs                                                                                                              | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`                           | ordered evaluation inputs and decision flow                                                          |
| Allowlist gating before adapter dispatch                                                                                                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`                           | fail-closed allowlist rule                                                                           |
| Missing inventory item, missing adapter, unsupported adapter, and capability-unsatisfied failure posture                                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`                           | absence semantics and failure classification                                                         |
| Ban on trusting gateway-local config, admin, or persistence surfaces for adapter authorization                                                                           | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`                           | trusted-input boundary                                                                               |
| Adapter registry lookup and dispatch lifecycle                                                                                                                           | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` | selection-to-dispatch lifecycle                                                                      |
| Capability validation order and unsupported extension-key failure posture                                                                                                | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` | validation order and fail-closed protocol behavior                                                   |
| Request normalization, response translation, and retry or ordering rules                                                                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` | lifecycle ordering and retry posture                                                                 |
| Session-handle lifecycle rules after adapter execution starts                                                                                                            | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` | creation, propagation, reuse, and invalidation rules                                                 |
| Boundary between local adapter event translation and the externally owned ADR-0017 event envelope                                                                        | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` | local translation scope and external envelope handoff                                                |
| Adapter descriptor object shape                                                                                                                                          | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | field list, types, defaults, and absence rules                                                       |
| Capability advertisement object shape and adopted capability ids                                                                                                         | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | capability ids, object shape, and bounds                                                             |
| Adopted versioned extension-key subset                                                                                                                                   | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | extension-key names, versioning format, and compatibility bounds                                     |
| Request payload object shape                                                                                                                                             | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | field list, types, defaults, and canonicalization                                                    |
| Response payload object shape                                                                                                                                            | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | field list, types, defaults, and success semantics                                                   |
| Adapter error object shape                                                                                                                                               | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | field list, codes, and bounded error detail                                                          |
| Backend-defined session-handle facet shape                                                                                                                               | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`   | field list, names, types, and size bounds                                                            |
| Linux adapter execution guarantee when world execution is required                                                                                                       | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`                  | in-world adapter execution rule and stable backend-id guarantee                                      |
| macOS adapter execution guarantee when world execution is required                                                                                                       | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`                  | in-world adapter execution rule and stable backend-id guarantee                                      |
| Windows adapter execution guarantee when world execution is required                                                                                                     | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`                  | in-world adapter execution rule and stable backend-id guarantee                                      |
| Cross-platform allowlist and backend-id invariance                                                                                                                       | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/platform-parity-spec.md`                  | parity guarantees and hidden divergence list                                                         |
| Structured event envelope semantics and output-class separation                                                                                                          | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`                        | event envelope, output classes, and routing vocabulary                                               |
| Canonical trace vocabulary and correlation semantics                                                                                                                     | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`                                | trace authority, field vocabulary, and correlation rules                                             |
| ADR-0024 supersession posture                                                                                                                                            | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`                    | supersession rule and end state                                                                      |
| Additive rollout posture for existing operator workflows                                                                                                                 | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`                    | compatibility promise and validation evidence                                                        |
| Future `cli:*` and `api:*` adapter expansion without identity drift                                                                                                      | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/compatibility-spec.md`                    | extension rule and invariants                                                                        |
| Manual validation of one-owner-per-surface coverage                                                                                                                      | `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/manual_testing_playbook.md`               | deterministic review procedure and expected assertions                                               |

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:

- Gateway operator contract doc
  - Existing gateway operator surfaces already have one owner in `docs/contracts/gateway/operator-contract.md`.
- Status-schema spec
  - Existing machine-readable gateway status surfaces already have one owner in `docs/contracts/gateway/status-schema.md`.
- Gateway runtime parity contract
  - Existing lifecycle and hidden-transport parity surfaces already have one owner in `docs/contracts/gateway/runtime-parity.md`.
- Env-vars spec
  - ADR-0041 does not add a new env-var family. Existing stable wiring env vars remain owned by the gateway operator contract.
- Telemetry spec
  - ADR-0041 delegates normalized structured event envelope semantics to ADR-0017 and canonical trace vocabulary to ADR-0028.
- Filesystem-semantics spec
  - ADR-0041 does not add new filesystem rules. Backend inventory file semantics remain owned by ADR-0027 schema docs.
- Decision register
  - ADR-0041 records one architectural A/B decision already. This manifest resolves document ownership without a new decision register.
- Smoke scripts
  - ADR-0041 is a contract-clarification body of work. The required validation artifact is `manual_testing_playbook.md`.

## Determinism checklist

Before promotion, the selected docs must define:

- every reused external-owner boundary, with one local reference point and one external owner
- every stable backend-id rule, including storage owner, selection owner, and serialization owner
- every adapter-selection failure bucket, including invalid selection, dependency unavailable, and policy denial
- every capability and extension-key field used by the adopted adapter contract subset
- every request, response, error, and session-handle object field, including absence semantics
- every handoff boundary to ADR-0017 event-envelope ownership and ADR-0028 trace ownership
- every Linux, macOS, and Windows guarantee for adapter-backed execution
- every compatibility rule that keeps existing operator workflows intact while ADR-0024 is treated as superseded architectural intent
- every manual validation assertion required to prove one owner exists for every touched surface

## Follow-ups

- Edit ADR-0041 `Related Docs` to use `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*`. The staged manifest uses the implemented path because that is the live repo path in this checkout.
- The S00 protocol/schema baselines are now pinned in `gateway-backend-adapter-protocol-spec.md` and `gateway-backend-adapter-schema-spec.md`; downstream planning should treat `C-03` and `C-04` as concrete, with no additional pre-planning ambiguity remaining for the adopted capability subset, extension-key subset, session-handle facet fields, bounded adapter error detail, or request/response/error object naming.
