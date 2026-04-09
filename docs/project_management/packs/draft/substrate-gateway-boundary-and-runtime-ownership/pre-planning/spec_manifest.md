# substrate-gateway-boundary-and-runtime-ownership — spec manifest (pre-planning)

This file enumerates every contract, schema, ownership, and validation surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

External authoritative inputs (this feature MUST NOT redefine these surfaces):
- Config/policy file families, precedence, gateway enablement keys, backend allowlists, and host secret-read gates:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- Structured event envelope and output-routing semantics when Substrate renders or persists structured events:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/contract.md`
  - `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md`
- Canonical trace vocabulary and correlation fields:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
- External gateway boundary and normalized-event contracts:
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`
  - `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md`
- Exit-code baseline:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Alignment evidence:
  - `.codex/handoffs/2026-04-02-144618-substrate-gateway-architecture-alignment.md`

## Slice IDs (canonical)

ADR-0040 defines one bounded documentation-and-contract slice for this feature. This pack MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `SGBRO` (derived from `substrate-gateway-boundary-and-runtime-ownership`)
- `SGBRO0` — Substrate versus `substrate-gateway` boundary ownership, operator contract consolidation, JSON status schema, and supersession/compatibility rules

## Required spec documents (authoritative)

This ADR is a boundary-ownership clarification pack. It requires:
- one feature-local operator contract doc,
- one stable JSON schema doc for the Substrate-owned status/wiring surface,
- one platform parity doc,
- one compatibility doc,
- one manual validation playbook,
- one canonical slice spec,
- and the standard planning-pack scaffolding below.

No separate feature-local protocol, policy, env-vars, telemetry, filesystem-semantics, decision-register, or smoke-script doc is selected.
- No feature-local protocol doc is selected because ADR-0040 does not define a new host↔gateway, gateway↔provider, or shell↔world wire protocol. Those protocols remain delegated to the external gateway runtime and follow-on ADRs.
- No feature-local policy doc is selected because ADR-0040 preserves the ADR-0027 policy surface rather than changing its schema, precedence, or evaluation rules.
- No feature-local env-vars doc is selected because the only env-var surface owned here is the small operator-facing pair of stable non-secret wiring names, and `contract.md` can own them without a separate env-vars spec.
- No feature-local telemetry doc is selected because ADR-0040 explicitly preserves external ownership for normalized event semantics and canonical trace vocabulary.
- No feature-local filesystem-semantics doc is selected because ADR-0040 introduces no new stable filesystem layout, mount rule, overlay rule, or protected-path contract beyond existing config/policy file families and the explicit statement that gateway-local persistence is not a Substrate contract surface.
- No feature-local decision register is selected because ADR-0040 already records the only material A/B architectural selection. The remaining open items are under-specified contract details that must be resolved inside the selected docs or recorded as follow-ups.
- No feature-local smoke scripts are selected because this pack establishes documentation and authority boundaries only; runtime smoke belongs to follow-on implementation packs once those packs own concrete behavior.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
  - Owns (authoritative):
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the canonical slice ids and canonical slice-spec path
    - the follow-ups required to remove ADR ambiguity before quality gate
  - Must define (deterministic items):
    - an exhaustive surface inventory with exactly one owner per ADR-touched surface
    - the explicit list of unselected doc classes and why they remain unselected
    - the rule that external gateway runtime surfaces stay delegated rather than being copied into feature-local docs
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
  - Owns (authoritative):
    - the exact touch set and downstream-document implications for this feature
    - the exact list of existing docs that must be reconciled once this pack is implemented
  - Must define (deterministic items):
    - the exact feature-local docs created by `SGBRO0`
    - the exact external documents that this pack must align against without taking ownership of them
    - the exact follow-on ADR/doc surfaces that remain blocked on this pack, including ADR-0024 successor work
  - Links to (non-authoritative):
    - all feature-local docs listed in this manifest

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - checkpoint grouping and CI gate cadence for this schema-v4 pack
  - Must define (deterministic items):
    - the single checkpoint boundary ending at `SGBRO0`
    - the exact relationship between checkpoint completion and `tasks.json` `meta.checkpoint_boundaries`
    - the explicit rule that no runtime smoke gate is required for this docs-only boundary pack
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - Owns (authoritative):
    - execution order, guardrails, and validation evidence requirements for this pack
  - Must define (deterministic items):
    - the orchestration branch `feat/substrate-gateway-boundary-and-runtime-ownership`
    - the canonical locations for:
      - `pre-planning/spec_manifest.md`
      - `pre-planning/impact_map.md`
      - `pre-planning/ci_checkpoint_plan.md`
    - the single-slice order `SGBRO0`
    - the exact validation evidence required before quality gate, including document-alignment checks against ADR-0027, ADR-0017, ADR-0028, and the gateway boundary/event notes
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-json-schema-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/compatibility-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json` (already exists)
  - Owns (authoritative):
    - task ids, dependency graph, automation metadata, and slice acceptance traceability
  - Must define (deterministic items):
    - `meta.checkpoint_boundaries` and alignment with `pre-planning/ci_checkpoint_plan.md`
    - one triad for `SGBRO0`:
      - `SGBRO0-code`
      - `SGBRO0-test`
      - `SGBRO0-integ`
    - references to the canonical slice-spec path under `slices/SGBRO0/`
    - `ac_ids` pointing to `AC-SGBRO0-*`
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/session_log.md`
  - Owns (authoritative):
    - the append-only planning and execution log for this pack
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - the rule that every task start and end is recorded with timestamp and task id
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/quality_gate_report.md`
  - Owns (authoritative):
    - the planning quality-gate outcome for starting execution triads
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - the rule that triads MUST NOT start unless `RECOMMENDATION: ACCEPT` is present
    - evidence that every delegated surface remains delegated and every feature-local surface has one owner
  - Links to (non-authoritative):
    - every required artifact referenced by the recommendation

### Feature contract and validation docs (required)

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - Owns (authoritative):
    - the Substrate-owned operator contract for integrated gateway use
    - the explicit ownership split between Substrate and `substrate-gateway`
    - stable non-secret wiring env-var names and their meaning
    - exit-code meanings and fail-closed invariants for the gateway lifecycle/status commands
  - Must define (deterministic items):
    - in-scope commands:
      - `substrate world gateway sync`
      - `substrate world gateway status`
      - `substrate world gateway restart`
      - `substrate world gateway status --json`
    - the exact ownership table:
      - Substrate owns policy gating, world placement, lifecycle, host-side secret sourcing and host-to-world secret delivery, operator UX, and canonical tracing
      - `substrate-gateway` owns the in-world front door, provider normalization, planner/executor routing internals, and normalized structured event generation
    - the exact stable non-secret wiring env-var names:
      - `SUBSTRATE_LLM_OPENAI_BASE_URL`
      - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
    - the exact meaning of those values:
      - they point to Substrate-managed gateway endpoints
      - they do not point to upstream provider endpoints
      - they are intended for in-world reachability and do not guarantee direct host reachability
    - the exact exit-code mapping:
      - `0` success
      - `2` invalid configuration, invalid policy, or invalid integration state
      - `3` transient runtime failure
      - `4` required gateway or world component unavailable
      - `5` policy or safety failure
    - the exact boundary invariants:
      - when world placement is required by effective policy, Substrate MUST NOT fall back to a host-level gateway
      - Substrate-owned secret delivery is the only trusted path for integrated secret material
      - gateway-local config files, admin mutation surfaces, token persistence, or internal tracing MUST NOT become required trust inputs for Substrate-managed operation
    - explicit delegated ownership references for:
      - config/policy key semantics and precedence
      - structured event envelope/routing semantics
      - canonical trace/correlation vocabulary
      - public external gateway identity/deployment boundary
  - Links to (non-authoritative):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-json-schema-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/compatibility-spec.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-json-schema-spec.md`
  - Owns (authoritative):
    - the stable JSON output contract for the Substrate-owned gateway status surface
  - Must define (deterministic items):
    - the authoritative JSON schema for `substrate world gateway status --json`
    - the exact relationship between `substrate world gateway status --json` and any JSON output emitted by `substrate world gateway sync`
    - the exact required top-level fields representing:
      - gateway availability
      - policy posture
      - non-secret `client_wiring.*` endpoint data
    - the exact `client_wiring.*` field paths used for the OpenAI-compatible and Anthropic-compatible base URLs
    - absence semantics when:
      - the gateway is unavailable
      - policy denies operation
      - a world session exists but wiring has not yet been established
    - the exact rule that all fields in this schema are non-secret
    - at least one example payload for:
      - gateway available
      - gateway unavailable because a required component is missing
      - policy failure
  - Links to (non-authoritative):
    - `docs/project_management/system/templates/spec/schema-spec.md.tmpl`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - Owns (authoritative):
    - Linux/macOS/Windows guarantees for the integrated gateway boundary
  - Must define (deterministic items):
    - Linux guarantee:
      - when worlds are enabled and policy requires in-world execution, the gateway runs inside the world boundary
      - Substrate-owned secret delivery is the only trusted integrated secret path
    - macOS guarantee:
      - the gateway is integrated through the macOS world backend
      - the same Substrate-owned trust-boundary rules apply
    - Windows guarantee:
      - the gateway is integrated through the Windows world backend
      - the same Substrate-owned trust-boundary rules apply
    - the exact allowed divergence rule:
      - no platform may broaden the trust boundary or introduce a platform-specific second control plane
    - the exact validation evidence expected per platform for later implementation packs
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/compatibility-spec.md`
  - Owns (authoritative):
    - supersession and compatibility rules introduced by ADR-0040
  - Must define (deterministic items):
    - the exact supersession rule:
      - ADR-0040 supersedes ADR-0023 in architectural intent
      - ADR-0023 remains historical context only for the original gateway-capability draft and archived planning set
    - the exact workflow preservation rule:
      - existing operator workflows remain valid
      - their ownership is clarified rather than redefined
    - the exact non-migration rule:
      - this feature introduces no new config family
      - this feature does not rename the stable non-secret wiring env-var names
      - this feature does not redefine backend-id grammar or structured-event framing
    - the exact “no second control plane” rule for integrated operation:
      - gateway-local standalone conveniences may exist
      - they remain implementation-specific and non-authoritative for Substrate-managed operation
    - the exact end condition for treating ADR-0023 as historical-only in planning references
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - Owns (authoritative):
    - deterministic manual validation for this boundary-ownership pack
  - Must define (deterministic items):
    - the exact document review inputs:
      - ADR-0040
      - feature-local selected docs from this manifest
      - ADR-0027 documents
      - ADR-0017 documents
      - ADR-0028 documents
      - gateway C-05/C-06 boundary notes
      - `.codex/handoffs/2026-04-02-144618-substrate-gateway-architecture-alignment.md`
    - the exact manual checks proving:
      - every touched surface is assigned to exactly one authoritative doc
      - no feature-local doc restates or contradicts delegated surfaces from ADR-0027, ADR-0017, ADR-0028, or the gateway notes
      - `contract.md`, `gateway-status-json-schema-spec.md`, `platform-parity-spec.md`, and `compatibility-spec.md` agree with each other
      - every follow-up listed in `pre-planning/spec_manifest.md` is either resolved or explicitly blocked before quality gate
  - Links to (non-authoritative):
    - every selected doc above

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
  - Owns (authoritative):
    - the vertical-slice scope and acceptance criteria for the entire boundary-ownership clarification pack
  - Must define (deterministic items):
    - acceptance criteria `AC-SGBRO0-*` proving:
      - every ADR-0040-touched surface is assigned to one authoritative doc
      - `contract.md` expresses the ownership split without redefining ADR-0027, ADR-0017, ADR-0028, C-05, or C-06 surfaces
      - `gateway-status-json-schema-spec.md` fully defines the stable JSON surface with absence semantics
      - `platform-parity-spec.md` fully defines Linux/macOS/Windows guarantees without platform-specific trust-boundary broadening
      - `compatibility-spec.md` fully defines ADR-0023 supersession and workflow-preservation posture
      - `manual_testing_playbook.md` validates the document set deterministically
    - explicit non-goals matching ADR-0040:
      - no full backend engine contract
      - no redefinition of ADR-0027 config/policy keys
      - no replacement of ADR-0017 output routing
      - no replacement of ADR-0028 canonical trace vocabulary
      - no public remote or multi-tenant gateway design
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-json-schema-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/compatibility-spec.md`

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0040 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI commands in scope (`substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`) | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | command names, command purpose, operator-entrypoint rules, non-goals, and linkage to JSON schema |
| Human-readable status/wiring output | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | stable operator entrypoint, abbreviated human output posture, and relation to `--json` |
| Structured JSON status/wiring output | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-json-schema-spec.md` | full schema, examples, absence semantics, and compatibility posture |
| Stable non-secret wiring env-var names | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | exact names, meaning, endpoint interpretation, and in-world reachability posture |
| Exit-code meanings for gateway lifecycle/status commands | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | taxonomy reference plus exact meanings for `0`, `2`, `3`, `4`, `5` |
| Substrate-owned trust-boundary responsibilities | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | exact ownership of policy, world placement, lifecycle, secret delivery, operator UX, and canonical tracing |
| `substrate-gateway` runtime-owned responsibilities | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | exact ownership of front door, provider normalization, planner/executor routing internals, and normalized event generation |
| Config file families and precedence for integrated gateway operation | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | file locations, precedence summary, and fail-closed operator posture |
| Gateway-related config key semantics (`llm.enabled`, `llm.gateway.enabled`, `llm.gateway.mode`, `llm.routing.default_backend`) | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | types, defaults, constraints, and backend-id interpretation |
| Gateway-related policy key semantics (`llm.fail_closed.routing`, `llm.allowed_backends`, `llm.secrets.env_allowed`, `agents.host_credentials.read.allowed_backends`) | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | types, defaults, deny-by-default posture, and host-read gating semantics |
| Public gateway identity and replaceable deployment boundary | `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md` | one stable backend identity, capability-oriented naming, and non-loopback-only deployment posture |
| Normalized gateway structured-event semantics | `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md` | normalized event meaning, explicit exclusions, and downstream-consumer contract |
| Substrate structured-event envelope and output-routing contract (when Substrate emits or persists structured gateway-related records) | `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md` | envelope fields, routing rules, and correlation-ready structure |
| Canonical trace field vocabulary and correlation semantics | `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` | top-level field names, correlation keys, and canonical trace record semantics |
| Linux/macOS/Windows guarantees and allowed divergences | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md` | per-platform guarantees, explicit divergence limits, and required evidence |
| ADR-0023 supersession and workflow-preservation posture | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/compatibility-spec.md` | exact supersession rule, historical-only boundary, and no-migration posture |
| Gateway-local config/admin/token-persistence surfaces are not authoritative for integrated Substrate operation | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | explicit exclusion from Substrate-managed trust inputs |
| Manual validation for the boundary-ownership pack | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md` | deterministic review steps and expected outcomes |
| Slice acceptance and non-goal guardrails | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md` | `AC-SGBRO0-*`, out-of-scope items, and acceptance boundaries |

## Determinism checklist (must be satisfied before quality gate)

For the selected docs above, confirm they explicitly define:
- every in-scope command and every delegated surface boundary
- defaults and absence semantics for every field in the JSON status surface
- exactly one owner for every touched surface, including externally delegated surfaces
- exit-code posture and fail-closed behavior
- platform guarantees and the exact rule for prohibited trust-boundary broadening
- compatibility posture, including what remains historical context only
- validation steps that prove the selected docs do not contradict ADR-0027, ADR-0017, ADR-0028, C-05, or C-06

## Follow-ups

- ADR-0040 makes `substrate world gateway status --json` authoritative, but it does not yet pin the full stable field set beyond non-secret `client_wiring.*`; `gateway-status-json-schema-spec.md` must resolve that field set explicitly.
- ADR-0040 implies `sync` exposes operator-visible status/wiring, but it does not yet say whether `substrate world gateway sync --json` is identical to `status --json` or wraps the same payload in an action result; the schema spec must resolve this singularly.
- `compatibility-spec.md` must state the exact end condition for removing ADR-0023 from current-contract references while still preserving it as historical context.
