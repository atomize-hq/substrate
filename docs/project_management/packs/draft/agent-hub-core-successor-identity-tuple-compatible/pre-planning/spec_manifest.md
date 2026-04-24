# agent-hub-core-successor-identity-tuple-compatible — spec manifest (pre-planning)

This file enumerates every contract, protocol, policy, telemetry, compatibility, validation, and slice-planning surface required for ADR-0044 and assigns each touched surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- External authoritative docs reused by this feature:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`

## Slice IDs (canonical)

Canonical slice ids selected for this feature:
- Slice prefix: `AHCSITC`
- `AHCSITC0` — operator contract and agent identity presentation lock
- `AHCSITC1` — orchestrator and member session protocol lock
- `AHCSITC2` — fail-closed policy and telemetry publication lock
- `AHCSITC3` — platform parity, compatibility, and validation closure

## Required spec documents (authoritative)

### Current pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the exact required document set for ADR-0044
    - the one-owner-per-surface matrix for every ADR-0044 surface
    - the canonical slice ids and canonical slice spec paths for this feature
    - the explicit list of reused external owners and unselected doc classes
  - Must define:
    - every feature-local document required by this body of work
    - every reused external authority that remains authoritative for unchanged surfaces
    - every follow-up that blocks promotion if the ambiguity remains unresolved

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
  - Role: required pre-planning artifact
  - Owns:
    - the exact create and edit touch set for implementation and doc changes
    - the downstream implications across agent hub, shell, trace, gateway, and policy surfaces
    - the cross-pack conflict map against ADR-0017, ADR-0028, ADR-0040, ADR-0041, ADR-0042, and ADR-0043
  - Must define:
    - the exact implementation paths under `crates/agent-hub`, `crates/shell`, `crates/trace`, and any `crates/agent-api-*` packages touched by the selected slices
    - the exact docs and contract files that require synchronized updates
    - the explicit no-change boundary for config-file family creation and for gateway ownership outside nested request routing

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/ci_checkpoint_plan.md`
  - Role: required pre-planning artifact
  - Owns:
    - the checkpoint cadence for this cross-platform, security-sensitive feature
    - the validation gates that execution tasks and quality gate review must mirror
  - Must define:
    - the checkpoint boundary after `AHCSITC2`
    - the final checkpoint boundary after `AHCSITC3`
    - the exact lint, doc-review, parity-review, and validation gates required before each checkpoint closes

### Topic-specific specs required by ADR-0044

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the additive operator-facing contract for `substrate agent list`, `substrate agent status`, and `substrate agent doctor`
    - the operator-visible meaning of the pure-agent versus nested-LLM presentation split on those command surfaces
    - the feature-local config, exit-code, backend-id-derivation, and platform-guarantee rules introduced or narrowed by ADR-0044
  - Must define:
    - the exact command surfaces changed by this feature, including every human-readable and machine-readable list, status, and doctor view exposed by the feature
    - the exact render rules for `backend_id`, `execution.scope`, `role`, capability summary, active sessions, `world_id`, and `world_generation`
    - the exact absence rule for `provider` and `auth_authority` on pure agent runs
    - the exact presence rule for `provider` and `auth_authority` on nested LLM records routed through `substrate_gateway`
    - the exact fail-closed exit posture for invalid orchestrator state on `substrate agent doctor`
    - the exact mapping from `agent_id` to `backend_id = "<kind>:<agent_id>"`
    - the explicit reuse boundary for config-file families and for the canonical exit-code taxonomy

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
  - Role: topic-specific protocol spec required by the ADR
  - Owns:
    - the capability-driven agent-backend session contract used by agent hub
    - the control-plane lifecycle for session start, stop, resume, and fork
    - orchestrator eligibility checks, member dispatch semantics, and world-placement semantics
    - the structured status and event exchange contract between agent hub and agent backends
  - Must define:
    - the exact capability descriptor schema
    - the exact session-handle schema and lifecycle transitions
    - the exact host-scoped orchestrator requirement
    - the exact world-scoped member dispatch rules
    - the exact shared-world reuse and restart contract, including when `world_generation` increments
    - the exact timeout, retry, ordering, and idempotency rules for protocol operations
    - the exact boundary between pure agent records and nested gateway-backed LLM requests

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
  - Role: topic-specific policy spec required by the ADR
  - Owns:
    - the fail-closed evaluation flow for orchestrator selection and member dispatch
    - the exact use of `agents.hub.orchestrator_agent_id` and `agents.allowed_backends` for agent-side gating
    - the boundary between agent-hub control-plane gating and gateway-side nested LLM policy gating
    - the explicit role and capability checks that block implicit control-plane action
  - Must define:
    - the exact ordered inputs for orchestrator selection and eligibility
    - the exact deny conditions for missing orchestrator, denied orchestrator, world-scoped orchestrator, missing session capability, and unavailable required world boundary
    - the exact rule that event-plane observation never authorizes a control-plane action
    - the exact rule that nested LLM requests reuse gateway policy gates from ADR-0043 and do not inherit agent-hub approval by implication
    - the exact rule that `backend_id` remains the only agent-side allowlist key

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
  - Role: topic-specific telemetry spec required by the ADR
  - Owns:
    - the structured event and trace publication rules for pure agent runs
    - the structured event and trace publication rules for nested LLM records triggered by an agent
    - the publication and omission rules for `client`, `router`, `protocol`, `backend_id`, `provider`, `auth_authority`, `world_id`, and `world_generation`
    - the alert and restart visibility rules for world reuse and drift restarts
  - Must define:
    - the exact field placement for pure-agent orchestration records
    - the exact field placement for nested gateway records
    - the exact omission rule for `provider` and `auth_authority` on pure agent runs
    - the exact separation rule between the base agent record and the nested LLM record
    - the exact event classes, trace families, and correlation keys reused from ADR-0017 and ADR-0028
    - the exact redaction rule for secret-adjacent nested request metadata

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
  - Role: topic-specific platform-parity spec required by the ADR
  - Owns:
    - the Linux, macOS, and Windows parity contract for operator-visible agent-hub semantics
    - the platform-specific failure posture when a required world boundary is unavailable
    - the validation evidence required to prove that parity
  - Must define:
    - the exact parity guarantees for list, status, doctor, event, and trace surfaces
    - the exact allowed divergence set
    - the exact world-boundary unavailability behavior per platform
    - the exact validation evidence required on Linux, macOS, and Windows

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
  - Role: topic-specific compatibility spec required by the ADR
  - Owns:
    - the semantic supersession of ADR-0025
    - the additive rollout rule that keeps existing `backend_id` values valid
    - the migration boundary between overloaded backend-centric wording and explicit identity-tuple wording
  - Must define:
    - the exact no-change behavior for existing backend allowlists
    - the exact rule that operators stop reading `backend_id` as provider, auth authority, or protocol identity
    - the exact documentation and rollout end-state for ADR-0025 replacement
    - the exact invariants that later features, including ADR-0045 follow-on work, must preserve

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md`
  - Role: topic-specific validation spec required by the ADR
  - Owns:
    - the deterministic manual validation procedure for ADR-0044
    - the proof that every touched surface has one authoritative owner
    - the cross-platform operator checks for pure-agent and nested-LLM visibility
  - Must define:
    - the exact commands and expected results for `substrate agent list`
    - the exact commands and expected results for `substrate agent status`
    - the exact commands and expected results for `substrate agent doctor`
    - the exact checks that prove host-scoped orchestrator selection, world-scoped member visibility, pure-agent field omission, and nested-LLM field presence
    - the exact cross-doc review checklist that proves one-owner-per-surface alignment

### Planning Pack artifacts required before execution

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/plan.md`
  - Role: required Planning Pack artifact
  - Must define:
    - the execution order for `AHCSITC0`, `AHCSITC1`, `AHCSITC2`, and `AHCSITC3`
    - the exact validation gates and artifact dependencies
    - the orchestration branch `feat/agent-hub-core-successor-identity-tuple-compatible`
    - the exact location of this feature’s pre-planning artifacts under `pre-planning/`

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/tasks.json`
  - Role: required Planning Pack artifact
  - Must define:
    - the schema v4 triad task graph for `AHCSITC0` through `AHCSITC3`
    - deterministic branch and worktree wiring for every task
    - acceptance-criteria traceability back to the selected local specs
    - checkpoint metadata aligned to `pre-planning/ci_checkpoint_plan.md`

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/session_log.md`
  - Role: required Planning Pack artifact
  - Must define:
    - the append-only planning and execution log for this pack
    - the exact start and end logging rule for every task id

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/quality_gate_report.md`
  - Role: required Planning Pack artifact
  - Must define:
    - the planning quality-gate outcome required before triads begin
    - the evidence that lint, determinism, and ownership checks passed for the selected doc set

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC0/AHCSITC0-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the contract and operator-facing identity presentation work for `substrate agent list`, `status`, and `doctor`
    - the acceptance criteria for `backend_id`, `client`, `router`, `protocol`, and absence semantics on pure agent runs
    - the out-of-scope boundary that keeps session lifecycle mechanics in `AHCSITC1`

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC1/AHCSITC1-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the session protocol, orchestrator-selection, member-dispatch, and world-reuse work
    - the acceptance criteria for capability discovery, session-handle lifecycle, host-scoped orchestrator enforcement, and `world_generation` increments
    - the out-of-scope boundary that keeps telemetry publication in `AHCSITC2`

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the policy-gating and telemetry-publication work for the pure-agent and nested-LLM split
    - the acceptance criteria for fail-closed routing, nested gateway-policy reuse, alert publication, and trace/event field placement
    - the out-of-scope boundary that keeps compatibility rollout closure in `AHCSITC3`

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC3/AHCSITC3-spec.md`
  - Role: canonical slice spec required by the standard
  - Must define:
    - the platform-parity proof, compatibility closure, and validation-completion work
    - the acceptance criteria for Linux, macOS, and Windows parity and for ADR-0025 supersession
    - the out-of-scope boundary that keeps operator command-surface detail in `AHCSITC0`

## Unselected doc classes

This manifest does not select these feature-local doc classes:
- Separate `env-vars-spec.md`
  - ADR-0044 introduces no new env var contract.
- Separate `filesystem-semantics-spec.md`
  - ADR-0044 names world identity and restart visibility but introduces no new path, overlay, mount, or protected-path contract.
- Separate schema-only spec file
  - `agent-hub-session-protocol-spec.md` owns the machine-readable capability, session, and status object shapes for this feature.
- Feature-local `decision_register.md`
  - ADR-0044 already records the governing A/B decision for this body of work, and this manifest adds no second independent architectural choice.
- Feature-local smoke scripts
  - ADR-0044 defines documentation-first validation through `manual_testing_playbook.md`.

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Existing file-family and precedence surface for config and policy storage | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | canonical config and policy file families and precedence rules reused by ADR-0044 |
| Existing schema key for `agents.hub.orchestrator_agent_id` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, base type, and base storage semantics |
| Existing schema key for `agents.allowed_backends` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, base type, and base allowlist storage semantics |
| Existing schema key for `llm.allowed_backends` | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, base type, and gateway allowlist storage semantics |
| Existing tuple-axis gateway policy keys under `llm.constraints.*` | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` | key names, defaults, and narrowing semantics reused for nested LLM requests |
| Global tuple-field meaning for `client`, `router`, `provider`, `auth_authority`, and `protocol` | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | normalized semantic meaning of each tuple field |
| Global protocol token `uaa.agent.session` | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | protocol identity reused by agent-hub pure-agent records |
| Existing gateway ownership boundary for nested LLM routing | `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md` | runtime boundary and ownership split reused by ADR-0044 |
| Existing gateway backend-adapter contract reused by nested LLM operations | `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` | adapter responsibilities and gateway/backend boundary reused by ADR-0044 |
| Existing structured event-envelope vocabulary | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | event classes, envelope vocabulary, and routing attribution family reused by ADR-0044 |
| Existing trace-envelope vocabulary and correlation family | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | trace envelope, `backend_id`, `world_id`, and correlation-key baseline reused by ADR-0044 |
| Additive operator contract for `substrate agent list`, `substrate agent status`, and `substrate agent doctor` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md` | exact command behavior, rendered fields, and additive no-new-top-level-command rule |
| Derived agent-hub `backend_id = "<kind>:<agent_id>"` mapping | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md` | exact derivation rule and the rule that role assignment does not encode into `backend_id` |
| Pure-agent absence semantics for `provider` and `auth_authority` on operator-visible command surfaces | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md` | exact omission rule for pure agent records |
| Nested-record presence semantics for `provider` and `auth_authority` on operator-visible command surfaces | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md` | exact presence rule for nested gateway-backed records |
| `substrate agent doctor` fail-closed contract when orchestrator selection is missing, denied, world-scoped, or otherwise ineligible | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md` | exact failure classes and exit posture |
| Capability descriptor object shape for agent backends | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | fields, required capabilities, normalization rules, and absence semantics |
| Session-handle object shape and lifecycle transitions | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | acquisition, persistence, resume, fork, stop, and invalidation rules |
| Host-scoped orchestrator requirement | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | exact eligibility rule and control-plane consequences |
| World-scoped member dispatch model | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | exact dispatch rule, world placement rule, and orchestration-session association |
| Shared-world reuse contract for world-scoped members | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | exact reuse scope, restart boundary, and `world_generation` increment rule |
| Structured status exchange between agent hub and agent backends | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | object shapes, ordering, and polling or streaming contract |
| Structured event exchange between agent hub and agent backends | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md` | event types, ordering, and retry or duplication rules |
| Ordered orchestrator-selection evaluation flow | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md` | exact ordered inputs and fail-closed decision sequence |
| Deny conditions for missing orchestrator, denied orchestrator, world-scoped orchestrator, missing capability, and unavailable required world boundary | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md` | exact deny classes and operator explanation requirements |
| Boundary that event-plane observation never authorizes control-plane action | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md` | exact no-implicit-control rule |
| Boundary that nested LLM calls reuse gateway policy gates and do not inherit agent-hub approval | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md` | exact routing and permission boundary |
| Agent-side allowlist rule that `backend_id` is the only allowlist key | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md` | exact allowlist lookup semantics |
| Pure-agent trace and event publication for `client`, `router`, `protocol`, `backend_id`, `world_id`, and `world_generation` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md` | field placement, omission rules, and correlation rules |
| Nested-LLM trace and event publication for `provider` and `auth_authority` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md` | field placement, presence rule, and separation from base agent record |
| Alert-event publication for world reuse and drift restart visibility | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md` | exact event family, required fields, and visibility rules |
| Redaction and no-secret rule for nested request metadata | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md` | exact redaction and omission rules |
| Linux operator-visible parity for list, status, doctor, events, and traces | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md` | exact Linux guarantees and required evidence |
| macOS operator-visible parity for list, status, doctor, events, and traces | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md` | exact macOS guarantees and required evidence |
| Windows operator-visible parity for list, status, doctor, events, and traces | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md` | exact Windows guarantees and required evidence |
| Platform-specific behavior when a required world boundary is unavailable | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md` | exact fail-closed behavior per platform |
| Semantic supersession of ADR-0025 | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md` | exact replacement boundary and migration rule for operator wording |
| No-change validity of existing `backend_id` allowlists | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md` | exact compatibility guarantee |
| Rule that operators stop inferring provider, auth authority, or protocol from `backend_id` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md` | exact compatibility and migration wording |
| Manual proof that host orchestrator selection is host-scoped | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md` | exact commands, evidence capture, and expected result |
| Manual proof that world-scoped members expose `world_id` and `world_generation` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md` | exact commands, evidence capture, and expected result |
| Manual proof that pure agent runs omit `provider` and `auth_authority` | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md` | exact commands, evidence capture, and expected result |
| Manual proof that nested gateway-backed records publish `provider` and `auth_authority` on the nested record only | `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md` | exact commands, evidence capture, and expected result |

## Determinism checklist

### `contract.md`
- Define the exact command surfaces and exact rendered-field obligations.
- Define the exact omission and presence rules for `provider` and `auth_authority`.
- Define the exact `backend_id` derivation and role-separation rule.
- Define the exact exit-code mapping and config reuse boundary.

### `agent-hub-session-protocol-spec.md`
- Define the exact capability descriptor schema.
- Define the exact session-handle schema and lifecycle state machine.
- Define the exact orchestrator eligibility and member-dispatch rules.
- Define the exact world reuse, restart, timeout, retry, ordering, and idempotency rules.

### `policy-spec.md`
- Define the exact ordered evaluation inputs and deny classes.
- Define the exact fail-closed rules for orchestrator and world-boundary failures.
- Define the exact no-implicit-control rule from event-plane observation.
- Define the exact gateway-policy reuse boundary for nested LLM requests.

### `telemetry-spec.md`
- Define the exact field placement for pure-agent and nested LLM records.
- Define the exact omission and presence rules for every identity field.
- Define the exact world-restart alert record fields.
- Define the exact redaction and correlation-key rules.

### `platform-parity-spec.md`
- Define the exact Linux, macOS, and Windows guarantees.
- Define the exact allowed divergence set.
- Define the exact world-boundary unavailability behavior per platform.
- Define the exact validation evidence required for parity signoff.

### `compatibility-spec.md`
- Define the exact ADR-0025 replacement boundary.
- Define the exact no-change guarantee for existing backend allowlists.
- Define the exact migration wording that removes backend-id overloading.
- Define the exact invariants later follow-on features must preserve.

### `manual_testing_playbook.md`
- Define the exact command matrix and expected outputs.
- Define the exact evidence required for pure-agent omission checks.
- Define the exact evidence required for nested-record presence checks.
- Define the exact one-owner-per-surface review checklist.

### `plan.md`
- Define the exact slice order and checkpoint order.
- Define the exact document-dependency order.
- Define the exact validation gates and promotion flow.

### `tasks.json`
- Define exact triad tasks for `AHCSITC0`, `AHCSITC1`, `AHCSITC2`, and `AHCSITC3`.
- Define exact acceptance criteria linked to the selected local specs.
- Define exact validation commands and checkpoint metadata.
- Define exact branch and worktree wiring for every task.

### `slices/AHCSITC0/AHCSITC0-spec.md`
- Define the exact command-surface and operator-identity presentation scope.
- Define the exact acceptance criteria for the pure-agent record split.
- Define the exact out-of-scope boundary for session-lifecycle mechanics.

### `slices/AHCSITC1/AHCSITC1-spec.md`
- Define the exact protocol and world-reuse scope.
- Define the exact acceptance criteria for orchestrator and member session lifecycle.
- Define the exact out-of-scope boundary for telemetry publication.

### `slices/AHCSITC2/AHCSITC2-spec.md`
- Define the exact policy and telemetry scope.
- Define the exact acceptance criteria for fail-closed routing and nested-record separation.
- Define the exact out-of-scope boundary for compatibility closure.

### `slices/AHCSITC3/AHCSITC3-spec.md`
- Define the exact parity, compatibility, and validation-completion scope.
- Define the exact acceptance criteria for ADR-0025 supersession and parity proof.
- Define the exact out-of-scope boundary for detailed command-surface rendering.

## Follow-ups

- `agent-hub-session-protocol-spec.md` needs to name the exact machine-readable status objects used by `substrate agent status` and every JSON variant exposed by the feature.
- `telemetry-spec.md` needs to name the exact restart and alert event family that carries world-reuse and world-generation transitions.
- `impact_map.md` needs to confirm whether the implementation reuses an existing `crates/agent-hub` package path or creates the successor module in a new exact crate path.
