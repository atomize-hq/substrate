# substrate-gateway-boundary-and-runtime-ownership — spec manifest (pre-planning)

This file enumerates every contract, status-schema, policy-evaluation, and platform surface touched by ADR-0040 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- Related authoritative contracts that remain external to this pack:
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

## Slice IDs (canonical)

This feature uses feature-derived slice ids per the triad setup standard.

Canonical slice ids selected for this feature:
- Slice prefix: `SGBRO`
- `SGBRO0` — boundary ownership and contract authority lock
- `SGBRO1` — status-schema and `client_wiring.*` lock
- `SGBRO2` — policy-evaluation and trust-boundary lock
- `SGBRO3` — typed runtime and parity lock
- `SGBRO4` — docs-validation, task-graph, and checkpoint lock-in

Planning-support artifacts required by the populated task graph:
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/kickoff_prompts/CP1-ci-checkpoint.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/kickoff_prompts/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO1/kickoff_prompts/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO2/kickoff_prompts/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO3/kickoff_prompts/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO4/kickoff_prompts/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO1/SGBRO1-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO2/SGBRO2-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO3/SGBRO3-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO4/SGBRO4-spec.md`

## Required spec documents (authoritative)

Each entry lists:
- what surfaces it owns, and
- what it must define with singular, testable statements.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
  - Owns:
    - the exact required-doc set for this feature
    - the coverage matrix for every ADR-0040 surface
    - the explicit list of unselected doc classes for this feature
  - Must define:
    - every authoritative doc selected for this body of work
    - every surface that remains externally owned
    - every follow-up required to remove remaining ADR ambiguity before quality gate

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
  - Owns:
    - the explicit touch set for the planning pack and downstream doc alignment
    - the cross-ADR and cross-pack implications of the ownership split
  - Must define:
    - every feature-local file expected to be created or edited
    - every external ADR or pack doc that this feature constrains
    - the contradiction risks created by stale ADR-0023 and stale `packs/active/...` references

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`
  - Owns:
    - the checkpoint grouping for this cross-platform planning pack
    - the exact validation gates required before promotion and before execution triads
  - Must define:
    - the single checkpoint boundary after `SGBRO4`
    - the exact doc-validation gates, lint gates, and cross-platform parity review gates for this pack

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - Owns:
    - the execution runbook for the pack
    - slice ordering and feature-level guardrails
  - Must define:
    - the orchestration branch and task flow for this pack
    - the accepted five-slice execution order `SGBRO0` through `SGBRO4`
    - the validation evidence required at slice closeout

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
  - Owns:
    - the task graph, dependencies, worktree metadata, and acceptance-criteria wiring
  - Must define:
    - triad tasks for `SGBRO0` through `SGBRO4`
    - `ac_ids` wired to the accepted slice specs
    - checkpoint metadata that matches `pre-planning/ci_checkpoint_plan.md`

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/session_log.md`
  - Owns:
    - the append-only planning and execution log for this pack
  - Must define:
    - start and end entries for every task
    - the audit trail for doc-production and validation activity

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/quality_gate_report.md`
  - Owns:
    - the planning quality-gate outcome for this pack
  - Must define:
    - the acceptance decision for the full planning pack
    - any blocking gaps that prevent execution triads

### Contract, schema, policy, parity, and validation docs (required)

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - Owns:
    - the CLI contract for `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart`
    - the human-readable operator contract for gateway availability, policy posture, and wiring discovery
    - the stable semantics of `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
    - the exit-code contract for gateway lifecycle and status commands
    - the operator-facing ownership split between Substrate and `substrate-gateway`
  - Must define:
    - command meanings, output posture, and absent-state behavior
    - the exact success, config-error, dependency-error, unsupported, and policy-failure boundaries for exit codes `0`, `2`, `3`, `4`, and `5`
    - the rule that wiring base URLs point to Substrate-managed gateway endpoints rather than upstream provider endpoints
    - the rule that gateway-local config, admin, and token-persistence surfaces are not required for Substrate-managed operation

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - Owns:
    - the structured output schema for `substrate world gateway status --json`
    - the `client_wiring.*` field family
    - the non-secret output guarantees and absence semantics for gateway status
  - Must define:
    - the top-level JSON object shape
    - every required and optional field, with types and conditional presence rules
    - the exact structure of `client_wiring.*`
    - the rule that status JSON remains the authoritative Substrate-owned wiring surface
    - the additive-field boundary against ADR-0042 metadata

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
  - Owns:
    - the gateway-integration evaluation rules that consume ADR-0027 config and policy inputs without redefining their schema
    - the fail-closed placement rules for in-world execution
    - the trust-boundary rules for host secret sourcing and host-to-world secret delivery
    - the rule that gateway-local admin, persistence, and config surfaces are not trusted policy inputs
  - Must define:
    - which existing keys govern gateway placement and secret delivery
    - the exact decision flow that distinguishes invalid integration state, dependency unavailability, and policy denial
    - the rule that Substrate does not authorize host-level gateway fallback when in-world placement is required
    - the rule that policy explanations remain Substrate-owned operator surfaces

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - Owns:
    - Linux/macOS/Windows guarantees for gateway placement, lifecycle visibility, and status semantics
    - any allowed backend divergence across world backends
  - Must define:
    - the parity guarantees for each platform
    - the allowed divergence list, if any
    - the exact validation evidence required for each platform

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - Owns:
    - the deterministic manual validation procedure for the ownership split and status/wiring contract
  - Must define:
    - the doc-review checklist that compares ADR-0040 against ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042
    - the exact assertions that verify each surface has one owner
    - the exact assertions that confirm no gateway-local admin or config surface has become a Substrate contract

### Slice spec (required)

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
  - Owns:
    - the boundary ownership contract set and canonical slice-order lock
  - Must define:
    - `AC-SGBRO0-*` entries for the accepted order, support-artifact presence, and task-graph coherence

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO1/SGBRO1-spec.md`
  - Owns:
    - the status-schema and `client_wiring.*` planning lock
  - Must define:
    - `AC-SGBRO1-*` entries for status-schema task wiring and prompt paths

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO2/SGBRO2-spec.md`
  - Owns:
    - the policy-evaluation and trust-boundary planning lock
  - Must define:
    - `AC-SGBRO2-*` entries for policy task wiring and prompt paths

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO3/SGBRO3-spec.md`
  - Owns:
    - the typed runtime and parity planning lock
  - Must define:
    - `AC-SGBRO3-*` entries for runtime/parity task wiring and prompt paths

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO4/SGBRO4-spec.md`
  - Owns:
    - the docs-validation, task-graph, and checkpoint lock-in
  - Must define:
    - `AC-SGBRO4-*` entries for the final task graph, checkpoint boundary, and validator-backed support artifacts

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `substrate world gateway sync` / `status` / `restart` command contract | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | command meaning, operator-visible outcome, absence semantics, and exit-code mapping |
| Human-readable gateway availability, policy posture, and wiring discovery | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | stable output posture and the rule that `status` is the operator entrypoint |
| `substrate world gateway status --json` schema | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` | top-level fields, types, required/optional rules, absence semantics |
| `client_wiring.*` JSON fields | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` | field names, field types, non-secret constraints, and canonical examples |
| Stable non-secret wiring env vars: `SUBSTRATE_LLM_OPENAI_BASE_URL`, `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | names, value semantics, intended consumer, and the rule that values target Substrate-managed gateway endpoints |
| Exit codes `0`, `2`, `3`, `4`, `5` for gateway lifecycle and status surfaces | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | taxonomy mapping and feature-specific failure boundaries |
| Config file locations and precedence | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` | global/workspace config and policy file paths plus precedence order |
| Gateway-related key names, types, defaults, and schema constraints (`llm.gateway.mode`, `llm.fail_closed.routing`, `llm.secrets.env_allowed`, `agents.host_credentials.read.allowed_backends`) | `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` | key paths, allowed values, defaults, and schema constraints |
| Gateway-integration evaluation over existing ADR-0027 keys | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md` | decision flow, fail-closed posture, and trust-boundary rules |
| Host secret sourcing and host-to-world secret delivery ownership | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md` | trusted source boundary, policy gates, and rejection rules |
| Ban on trusting gateway-local config, admin mutation, or token-persistence surfaces | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md` | explicit non-contract rule and failure posture when such surfaces are absent |
| Linux/macOS/Windows gateway placement and lifecycle guarantees | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md` | per-platform guarantees, allowed divergence list, and validation evidence |
| Substrate-owned operator boundary (`policy`, `world placement`, `lifecycle`, `secret delivery`, `operator UX`, `canonical tracing`) | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | ownership summary and non-goal boundary against gateway internals |
| `substrate-gateway` runtime internals (`front door`, `provider/planner/executor routing`, normalized event generation) | `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` | backend-adapter and runtime-internal ownership |
| Structured event routing and output-class separation | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | event-plane semantics and routing ownership |
| Canonical trace vocabulary and correlation semantics | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | trace fields, correlation vocabulary, and trace authority |
| Identity-tuple metadata and placement-posture semantics beyond `client_wiring.*` | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | `client`, `router`, `provider`, `auth_authority`, `protocol`, and posture semantics |
| Manual validation of one-owner-per-surface coverage | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md` | deterministic checklist and expected assertions |
| Slice-local delivery and acceptance criteria | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md` through `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO4/SGBRO4-spec.md` | `AC-SGBRO0-*` through `AC-SGBRO4-*`, sequencing, checkpoint boundary, and validator-backed support artifacts |

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:
- Protocol spec
  - ADR-0040 does not define a new stable host↔gateway RPC, HTTP request/response framing, WebSocket framing, or IPC envelope as a Substrate-owned contract.
- Telemetry spec
  - ADR-0040 leaves structured event routing to ADR-0017 and canonical trace vocabulary to ADR-0028.
- Env-vars spec
  - The only feature-local env-var surfaces are the two stable, non-secret wiring names, and `contract.md` owns their semantics.
- Filesystem-semantics spec
  - ADR-0040 does not define new mount, overlay, diff-collection, or protected-path path-resolution rules.
- Compatibility spec
  - ADR-0040 supersedes architectural intent and does not define a migration mechanism, staged rollout, or temporary compatibility layer.
- Decision register
  - ADR-0040 already records the single A/B architectural decision for this body of work, and this manifest resolves the remaining document-ownership choices directly.
- Smoke scripts
  - This feature is a boundary-ownership and planning-pack clarification body of work. The validation artifact is a deterministic manual playbook rather than runtime smoke automation.

## Determinism checklist

Before quality gate, the selected docs must define:
- every CLI surface touched by ADR-0040, including absent-state behavior and exit-code mapping
- every JSON field in `status --json`, including conditional presence rules and example payloads
- every reused config and policy input, with one external schema owner and one local evaluation owner
- every fail-closed policy rule and every dependency-unavailable rule
- every platform guarantee for Linux, macOS, and Windows
- every validation step required to confirm there is one owner per surface

## Follow-ups

- ADR-0040 `Related Docs` now references `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*`. Keep that live path current if the external owner moves again.
- `gateway-status-schema-spec.md` must state the hard boundary against ADR-0042: this pack owns gateway availability, policy posture, and `client_wiring.*`; ADR-0042 owns identity-tuple and placement-posture semantics beyond that field family.
