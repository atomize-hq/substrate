# gateway-backend-selection-runtime-integration — spec manifest (pre-planning)

This file enumerates every contract, protocol, schema, env-var, filesystem, compatibility, and platform surface touched by ADR-0046 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- External authoritative docs reused by this feature:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

This manifest does not require `plan.md`, `tasks.json`, kickoff prompts, execution-ownership registries, or other legacy full-planning artifacts.

## Required documents (authoritative)

### Pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/spec_manifest.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the required doc set for ADR-0046
    - the one-owner-per-surface matrix
    - the explicit list of selected and unselected doc classes
  - Must define:
    - every feature-local doc required by this body of work
    - every reused external owner that remains authoritative
    - every follow-up that blocks deterministic downstream planning

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the touch set for the selected doc set
    - the downstream implications across shell, broker, world-service, and cross-platform validation
  - Must define:
    - every feature-local doc expected to be authored
    - every external doc that this pack constrains or depends on
    - the exact drift risks that stale external references create

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/minimal_spec_draft.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the cross-document defaults and invariants shared by the selected local specs
  - Must define:
    - the adopted backend-selection vocabulary
    - the adopted fail-closed vocabulary
    - the adopted adapter-runtime realization vocabulary

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/workstream_triage.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the advisory sequencing for downstream seam planning
  - Must define:
    - the dependency order across selection and policy, runtime realization, and parity validation
    - the conflict points between local specs and reused external contracts

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/alignment_report.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the wrapper-facing summary of pre-planning gaps and hard gates
  - Must define:
    - every blocker that prevents downstream seam decomposition
    - the exact external docs that need alignment review before promotion

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/ci_checkpoint_plan.md`
  - Role: pre-planning artifact produced in this lane
  - Owns:
    - the checkpoint cadence for this cross-platform, security-sensitive doc set
  - Must define:
    - the document-validation gate before seam planning starts
    - the parity-validation gate for Linux, macOS, and Windows
    - the doc-lint and ambiguity-scan gates for all selected authored docs

### Topic-specific specs required by ADR-0046

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the gateway-lifecycle subset of the user-facing contract that ADR-0046 changes
    - the rule that `status`, `sync`, and `restart` realize the selected backend through inventory, policy, and one integrated adapter binding
    - the rule that the command family and backend selector remain unchanged
    - the operator-visible exit-code meanings for invalid integration, transient failure, dependency unavailable, and policy denial in this feature
  - Must define:
    - the exact selected-backend realization semantics for `substrate world gateway status`, `sync`, and `restart`
    - the exact rule that no new CLI flag family or top-level command family is introduced
    - the exact exit-code mapping for this feature, with references to the canonical taxonomy
    - the exact boundaries against the existing operator contract and status schema docs

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the integrated-lifecycle evaluation flow over existing config, policy, and inventory inputs
    - backend allowlist gating before integrated adapter dispatch
    - host env-read and host-credential-read gating for adapter-specific auth material sourcing
    - the trusted-input boundary that excludes gateway-local config, admin mutation, and token persistence from authorization
  - Must define:
    - the exact ordered evaluation inputs for backend realization
    - the exact decision flow that distinguishes invalid integration, dependency unavailable, and policy denial
    - the exact absence semantics for missing inventory entries, disallowed backends, blocked env reads, and blocked host credential reads
    - the exact relationship to the existing gateway policy-evaluation contract

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the internal env-var boundary used by shell request construction and world-service runtime launch
    - the adapter-specific auth env-var family used by integrated auth handoff
    - the redaction and absence rules for all env-driven auth material
  - Must define:
    - the exact name, producer, consumer, type, default, and allowed values for each internal env var
    - the exact precedence between env-based auth handoff and host credential file reads
    - the exact redaction rules for secret-bearing env vars
    - the exact boundary between internal env vars and the stable operator-facing wiring env outputs

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - integrated adapter registry lookup after backend selection and allowlist evaluation have already succeeded
    - required capability gating for the integrated lifecycle
    - auth handoff resolution order
    - runtime config rendering order
    - launch, restart, and readiness-probe ordering for the selected backend
  - Must define:
    - the exact lifecycle from selected backend id to one integrated adapter binding
    - the exact fail-closed behavior for missing bindings, unsupported adapters, missing capabilities, and unsatisfied auth handoff material
    - the exact ordering for auth handoff resolution, config rendering, process launch, and readiness confirmation
    - the exact boundary against the existing general gateway adapter protocol contract and runtime parity contract

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - integrated adapter binding metadata
    - required capability-set metadata for integrated lifecycle use
    - adapter-specific auth handoff payload shapes
    - adapter-specific runtime config payload shapes
    - integrated lifecycle result and failure-shape additions that are local to this feature seam
  - Must define:
    - every field name, field type, default, and absence rule for integrated adapter binding metadata
    - the exact schema for auth handoff variants used by supported integrated backends
    - the exact schema for runtime config payloads rendered from adapter-owned metadata
    - the exact error-shape fields used to classify unsupported adapter selection, missing capabilities, and auth handoff failures

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - backend inventory file lookup and filename-to-id invariants as they apply during integrated runtime realization
    - generated runtime artifact paths for config, manifest, and managed runtime logs
    - file permission and inspectability rules for managed runtime artifacts
    - host credential file path rules used by adapter-specific auth handoff
  - Must define:
    - the exact path roots, naming rules, and lifecycle of generated runtime artifacts
    - the exact permissions and authorized-read expectations for config, manifest, and log files
    - the exact path and absence semantics for host credential files used by integrated auth handoff
    - the exact rule that gateway-local persistence does not authorize execution

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - Linux, macOS, and Windows parity guarantees for backend selection and integrated runtime realization
    - allowed transport and bootstrap divergence hidden behind one operator-facing contract
    - required platform validation evidence for lifecycle, readiness, and restart behavior
  - Must define:
    - the exact guarantees that are identical across Linux, macOS, and Windows
    - the exact hidden divergences that remain outside the operator contract
    - the exact validation evidence required on each platform

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md`
  - Role: topic-specific spec required by the ADR
  - Owns:
    - the additive rollout posture for extending integrated lifecycle realization beyond `cli:codex`
    - the regression baseline for `cli:codex`
    - the rule that unsupported backends fail explicitly rather than collapsing back to Codex-specific behavior
  - Must define:
    - the exact backward-compatibility promise for existing `cli:codex` operators
    - the exact acceptance condition for adding the first non-`cli:codex` integrated backend
    - the exact validation evidence that proves unsupported backends fail explicitly

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md`
  - Role: topic-specific validation spec required by the ADR
  - Owns:
    - the deterministic manual validation flow for selected-backend realization and parity proof
    - the operator validation matrix for `cli:codex`, one additional integrated backend, invalid backend, blocked backend, missing binding, and missing auth cases
    - the validation references for platform smoke scripts
  - Must define:
    - the exact ordered manual steps and expected outputs for Linux, macOS, and Windows
    - the exact assertions for exit-code buckets, status transitions, and restart behavior
    - the exact smoke-script paths and the evidence each script must capture

### Downstream seam-planning and decomposition artifacts that must exist later

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/backend-selection-and-policy.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `contract.md`, `policy-spec.md`, and `env-vars-spec.md`
    - planning for shell-side backend selection, policy evaluation, and auth-material sourcing
  - Must define:
    - the implementation seam for selected-backend realization and policy gating
    - the acceptance boundary against gateway-local authorization drift

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/runtime-realization-and-artifacts.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `gateway-runtime-adapter-protocol-spec.md`, `gateway-runtime-adapter-schema-spec.md`, and `filesystem-semantics-spec.md`
    - planning for world-service adapter binding, config rendering, auth handoff, and runtime artifact management
  - Must define:
    - the implementation seam for adapter binding lookup, capability gating, readiness probing, and artifact lifecycle
    - the acceptance boundary against external adapter-protocol drift

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/seam-planning/parity-validation-and-rollout.md`
  - Role: downstream seam-planning artifact deferred until after pre-planning
  - Intended ownership scope:
    - planning for `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`
    - planning for cross-platform validation evidence and rollout proof
  - Must define:
    - the implementation seam for Linux, macOS, and Windows parity proof
    - the acceptance boundary for `cli:codex` regression coverage and first additional backend coverage

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json` as the stable operator command family | `docs/contracts/substrate-gateway-operator-contract.md` | command names, baseline operator meaning, stable entrypoint set, and stable absent-state semantics |
| `status --json` envelope and `client_wiring.*` field family | `docs/contracts/substrate-gateway-status-schema.md` | field names, field types, and absence semantics |
| Stable non-secret wiring env outputs `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` | `docs/contracts/substrate-gateway-operator-contract.md` | env names, non-secret posture, and ownership boundary |
| Gateway policy-evaluation rules for `llm.gateway.mode`, `llm.fail_closed.routing`, host-to-world secret delivery, and no-host-fallback posture | `docs/contracts/substrate-gateway-policy-evaluation.md` | evaluation meaning, fail-closed posture, and trust boundary |
| Typed runtime authority, readiness truth, operator-facing parity contract, and hidden transport divergence | `docs/contracts/substrate-gateway-runtime-parity.md` | typed runtime ownership, readiness contract, and transport-divergence boundary |
| Stable `<kind>:<name>` backend-id grammar and the ordered selection boundary before adapter dispatch | `docs/contracts/substrate-gateway-backend-adapter-selection.md` | backend-id format, selection order, invalid-versus-denied distinction, and trusted-input boundary |
| General gateway adapter dispatch lifecycle after a backend id has been selected | `docs/contracts/substrate-gateway-backend-adapter-protocol.md` | general lookup, validation, dispatch, and event-translation rules reused by this feature |
| General gateway adapter capability subset, extension subset, request shape, bounded event/completion shape, and bounded adapter error shape | `docs/contracts/substrate-gateway-backend-adapter-schema.md` | adopted capability ids, extension keys, request envelope, bounded event/completion payloads, and bounded adapter error vocabulary |
| Config file families and precedence for global and workspace config and policy files | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | file locations, precedence order, and overlay rules |
| Inventory file family and filename-to-id match rule | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | file locations, filename rules, and schema-backed identity consistency |
| `llm.routing.default_backend` key path, type, and default behavior | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, type, default, and schema constraints |
| `llm.allowed_backends` key path, type, and deny-by-default storage rule | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, type, default, and schema constraints |
| `llm.secrets.env_allowed` key path, type, and overlay rule | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, type, default, and overlay constraints |
| `agents.host_credentials.read.allowed_backends` key path, type, and overlay rule | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key path, type, default, and overlay constraints |
| The rule that the selected backend remains config-driven and no new CLI flag family is introduced for integrated lifecycle realization | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md` | backend-selection input source, no-new-CLI rule, and operator-visible lifecycle semantics |
| Operator-visible exit-code mapping for success, invalid integration, transient failure, dependency unavailable, and policy denial | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md` | exit-code table, command-family semantics, and links to the canonical taxonomy |
| The rule that `status`, `sync`, and `restart` remain valid for any supported integrated backend | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md` | supported-backend semantics and unsupported-backend behavior |
| The ordered evaluation flow over config, inventory, allowlists, env-read gates, and host-credential-read gates for backend realization | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md` | evaluation order, decision points, and one-owner classification rules |
| The distinction between invalid integration, dependency unavailable, and policy denial for selection and auth sourcing failures | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md` | failure-bucket mapping and absence semantics |
| The rule that gateway-local config, admin mutation, and token persistence do not authorize integrated backend realization | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md` | trusted-input boundary and non-authoritative local state |
| Shell-to-world request env vars `SUBSTRATE_LLM_GATEWAY_ENABLED`, `SUBSTRATE_LLM_GATEWAY_MODE`, and `SUBSTRATE_LLM_DEFAULT_BACKEND` | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md` | env names, producers, consumers, allowed values, defaults, and absence semantics |
| World-agent launch env vars `SUBSTRATE_LLM_GATEWAY_MODE`, `SUBSTRATE_LLM_GATEWAY_CONFIG_PATH`, `SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE`, and `SUBSTRATE_GATEWAY_BINARY` | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md` | env names, producers, consumers, allowed values, defaults, and redaction posture |
| Adapter-specific auth env vars such as `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID` and `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN` | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md` | env names, value types, precedence, and redaction requirements |
| Integrated adapter registry lookup after backend selection and allowlist evaluation | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md` | lookup order, one-binding rule, and unsupported-binding posture |
| Required capability gating before runtime launch | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md` | required-capability evaluation order and fail-closed behavior |
| Auth handoff source resolution, validation order, and handoff delivery sequencing | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md` | source precedence, validation order, delivery order, and blocked-source posture |
| Runtime config rendering, launch, readiness probe, and restart ordering | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md` | render-before-launch rule, readiness truth, restart ordering, and failure mapping |
| Integrated adapter binding metadata and required capability-set schema | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md` | field names, field types, defaults, and absence rules |
| Adapter-specific auth handoff payload variants | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md` | payload names, required fields, optional fields, and canonicalization rules |
| Adapter-specific runtime config payload shapes | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md` | payload names, field types, defaults, and absence rules |
| Integrated lifecycle-local failure object additions used to classify unsupported adapter selection, missing capabilities, and auth handoff failures | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md` | field names, field types, and classification tokens |
| Host credential file path used by backend-specific auth sourcing | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md` | path, permissions expectations, parse constraints, and absence semantics |
| Generated runtime artifact root, runtime manifest path, rendered config path, and managed log paths | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md` | path roots, naming rules, file lifecycle, and cleanup rules |
| Runtime artifact permissions and operator inspectability for managed log and manifest files | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/filesystem-semantics-spec.md` | permission modes, group ownership, and authorized-read expectations |
| Linux backend-selection and integrated-runtime realization guarantee | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md` | parity guarantee, hidden Linux transport details, and required validation evidence |
| macOS backend-selection and integrated-runtime realization guarantee | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md` | parity guarantee, hidden macOS transport details, and required validation evidence |
| Windows backend-selection and integrated-runtime realization guarantee | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/platform-parity-spec.md` | parity guarantee, hidden Windows transport details, and required validation evidence |
| Regression baseline that preserves existing `cli:codex` lifecycle behavior | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md` | baseline backend, unchanged operator surface, and required regression evidence |
| Explicit failure posture for unsupported backends with no fallback to Codex-specific behavior | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md` | unsupported-backend classification and no-fallback rule |
| Manual validation matrix for `cli:codex`, one additional integrated backend, invalid backend, blocked backend, missing binding, and missing auth cases | `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/manual_testing_playbook.md` | steps, expected outputs, and evidence capture |

## Required non-doc validation artifacts

These artifacts are required by ADR-0046 validation, but they are not selected as authoritative docs:

- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/smoke/windows-smoke.ps1`

`manual_testing_playbook.md` owns the required assertions for these scripts. `ci_checkpoint_plan.md` owns when they run in the planning cadence.

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:

- Gateway operator contract doc
  - Existing operator command-family semantics already have one owner in `docs/contracts/substrate-gateway-operator-contract.md`.
- Status-schema spec
  - Existing machine-readable gateway status surfaces already have one owner in `docs/contracts/substrate-gateway-status-schema.md`.
- Standalone gateway policy-evaluation contract
  - Existing gateway policy-evaluation semantics already have one owner in `docs/contracts/substrate-gateway-policy-evaluation.md`.
- Telemetry spec
  - ADR-0046 does not add a new trace field, log field, or structured-event envelope field. Existing trace vocabulary and envelope ownership remain external.
- Decision register
  - ADR-0046 records one A/B decision already and states that no decision register is required for this draft.

## Determinism checklist

Before promotion, the selected docs must define:

- every reused external-owner boundary, with one local reference point and one external owner
- every operator-visible exit bucket for `status`, `sync`, and `restart`
- every config and policy input that influences backend realization, with one precedence order
- every internal env var used for request construction, runtime launch, and auth handoff
- every adapter binding, capability-set, auth payload, and runtime config schema field used by the integrated lifecycle seam
- every runtime artifact path, permission rule, and inspectability expectation
- every Linux, macOS, and Windows guarantee for realization, readiness, and restart behavior
- every compatibility rule that preserves `cli:codex` as the regression baseline while allowing explicit support for additional integrated backends
- every manual-validation and smoke-script assertion required to prove one owner exists for every touched surface

## Follow-ups

- Pin the exact first non-`cli:codex` integrated backend id used for snapshots, integration fixtures, smoke scripts, and manual validation. ADR-0046 requires one additional integrated backend but does not name it.
- Pin one deterministic classification rule for a missing integrated adapter binding. ADR-0046 currently uses `unsupported integrated adapter selection` in the user contract and `dependency-unavailable classification` in the validation plan.
- Pin one deterministic classification rule for missing auth handoff material when policy allows the read path but the required env var or credential file is absent.
- Pin one deterministic transport rule for auth handoff delivery into the integrated runtime: env-only, file-only, or a fixed mixed model with explicit precedence.
- Update ADR-0046 `Related Docs` to remove legacy expected outputs (`plan.md`, `tasks.json`, feature-root `spec_manifest.md`) and align the pack references with the pre-planning lane selected in this manifest.
