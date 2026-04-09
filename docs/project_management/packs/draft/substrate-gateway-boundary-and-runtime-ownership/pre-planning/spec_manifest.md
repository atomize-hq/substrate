# substrate-gateway-boundary-and-runtime-ownership — spec manifest

This file enumerates every contract surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

## Required documents (authoritative)

### Planning pack artifacts
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/spec_manifest.md`
  - Owns the required-doc selection and the surface-to-owner map for this feature.
  - Links to every selected feature-local spec and every delegated external owner.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/impact_map.md`
  - Owns the downstream touch set and the cross-ADR dependency map for the selected docs.
  - Links to ADR-0027, ADR-0017, ADR-0028, and ADR-0041 as upstream contract dependencies.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - Owns the execution runbook for authoring the selected docs without reopening delegated surfaces.
  - Links to `tasks.json`, `manual_testing_playbook.md`, and `slices/SGBRO0/SGBRO0-spec.md`.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
  - Owns the triad task graph for the selected slice only.
  - Links to `plan.md`, `manual_testing_playbook.md`, and `slices/SGBRO0/SGBRO0-spec.md`.

### Feature-local authoritative specs
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - Owns the Substrate operator-facing contract added or clarified by ADR-0040:
    - `substrate world gateway sync`
    - `substrate world gateway status`
    - `substrate world gateway restart`
    - `substrate world gateway status --json` as the stable discovery entrypoint
    - exit codes `0|2|3|4|5`
    - stable non-secret wiring env var names
  - Links to ADR-0027 pack docs for config/policy paths, precedence, defaults, and existing key schemas.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md`
  - Owns the ownership matrix for Substrate versus `substrate-gateway`.
  - Owns the explicit delegation rules for config/policy, routing/envelope, trace vocabulary, and gateway runtime internals.
  - Owns the ADR-0023 supersession rule for this feature.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - Owns the stable JSON schema for `substrate world gateway status --json`.
  - Links to `contract.md` for command-level semantics and env-var naming.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - Owns Linux/macOS/Windows placement guarantees, allowed divergences, and required validation evidence.
  - Links to `runtime-boundary-spec.md` for the ownership split and to `contract.md` for CLI-visible failure posture.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - Owns the deterministic doc-review validation cases for this feature.
  - Links to `contract.md`, `runtime-boundary-spec.md`, `gateway-status-schema-spec.md`, and `platform-parity-spec.md`.
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
  - Owns the slice-level scope, acceptance criteria, and out-of-scope guardrails for authoring the docs above.
  - Links to every selected feature-local spec and `manual_testing_playbook.md`.

## Not selected

The ADR does not justify the docs below for this feature. They must not be created unless ADR scope changes.

- `compatibility-spec.md`
  - Not selected because ADR-0040 is additive boundary clarification. ADR-0023 supersession semantics fit in `runtime-boundary-spec.md`.
- `policy-spec.md`
  - Not selected because ADR-0040 does not add or change policy decision rules. ADR-0027 remains authoritative.
- `env-vars-spec.md`
  - Not selected because the only env-var surface clarified here is two stable non-secret wiring names, which is small and operator-facing enough to belong in `contract.md`.
- `<topic>-protocol-spec.md`
  - Not selected because ADR-0040 does not define a new Substrate-local RPC/HTTP/WebSocket/named-pipe contract. Gateway runtime internals remain delegated.
- `telemetry-spec.md`
  - Not selected because ADR-0040 does not redefine structured-event envelopes or canonical trace vocabulary. ADR-0017 and ADR-0028 remain authoritative.
- `filesystem-semantics-spec.md`
  - Not selected because ADR-0040 defines boundary ownership and platform placement posture, not new path-resolution or mount-layout semantics.

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `substrate world gateway sync` command contract | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | command purpose, default output, lifecycle semantics, failure posture, relationship to secret rotation and gateway ensure behavior |
| `substrate world gateway status` command contract | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | stable operator entrypoint rule, human-readable output semantics, availability/posture reporting rules |
| `substrate world gateway restart` command contract | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | restart semantics, explicit lifecycle ownership, secret-rotation trigger semantics |
| `substrate world gateway status --json` serialized surface | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` | top-level object shape, exact field names, required vs optional fields, defaults, and absence semantics |
| `client_wiring.*` JSON fields in `status --json` | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` | exact field paths, types, non-secret rule, omission rules, and mapping to the stable env-var names |
| Stable non-secret wiring env var names `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | names, non-secret classification, semantic meaning, reachability posture, and operator discovery rules |
| Gateway lifecycle/status exit codes `0`, `2`, `3`, `4`, `5` | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` | code-to-meaning mapping, command applicability, and reference to the canonical taxonomy |
| Global/workspace config file families and precedence (`$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`, `$SUBSTRATE_HOME/policy.yaml`, `<workspace_root>/.substrate/policy.yaml`) | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md` | exact file paths, precedence order, and the rule that ADR-0040 adds no new config family |
| Existing gateway-related config/policy key schema and defaults under the `llm.*` families | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md` | exact key paths, allowed values, defaults, and fail-closed rules already defined by ADR-0027 |
| Host secret-read allowlists `llm.secrets.env_allowed` and `agents.host_credentials.read.allowed_backends` | `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md` | exact key names, element types, defaults, deny-by-default behavior, and absence semantics |
| Boundary rule that Substrate owns policy-gated host secret sourcing and host-to-world secret delivery for integrated operation | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md` | ownership statement, fail-closed posture, and the rule that detailed transport/auth mechanics are delegated rather than redefined here |
| Rule that gateway-local config files, local persistence, and admin mutation surfaces are not required for Substrate-managed operation | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md` | explicit exclusion, operator-visible implication, and anti-second-control-plane invariant |
| Gateway backend adapter internals, provider/planner/executor routing, and stable `<kind>:<name>` backend identity at the Substrate boundary | `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` | adapter/runtime ownership, backend identity rules, internal capability/session semantics, and fail-closed adapter selection posture |
| Gateway deployment/auth factoring and in-world-compatible boundary below the public Substrate seam | `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-boundary-c05-contract.md` | replaceable deployment/auth boundary, non-loopback-only posture, and drift guards against leaking internal roles into the public boundary |
| Gateway-generated normalized structured event semantics | `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/substrate-structured-events-c06-contract.md` | normalized event meaning, exclusions for raw provider transport, and downstream drift guards |
| Substrate structured event routing/envelope after gateway events cross into Substrate-managed output | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | output-class separation, envelope/routing rules, and attribution requirements |
| Substrate canonical trace vocabulary, correlation fields, and persistence posture | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | canonical correlation field names, trace record posture, and operator-facing audit semantics |
| Linux/macOS/Windows gateway placement guarantees and allowed divergences | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md` | per-platform placement guarantees, boundary invariants, unavailable behavior, and required validation evidence |
| ADR-0023 supersession and historical-only interpretation for this feature | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md` | exact supersession rule, what remains historical context only, and the no-migration consequence for operators |
| Manual validation coverage for the ownership split and delegated-source links | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md` | deterministic review steps, expected findings, and pass/fail criteria |
| Slice-level acceptance for authoring this docs-only pack | `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md` | exact scope, acceptance criteria, and out-of-scope guardrails for the single selected slice |

## Deterministic requirements by document

### `contract.md`
- Define the exact command set owned by this feature: `sync`, `status`, `restart`, and `status --json`.
- Define which outputs are stable operator contract and which outputs are informational only.
- Define exit-code meanings for `0`, `2`, `3`, `4`, and `5`, with no command-specific contradictions.
- Define the stable env-var names, their non-secret classification, and the rule that they point to Substrate-managed gateway endpoints intended for in-world reachability.
- State that this feature introduces no new config file family, no new policy file family, and no required gateway-local admin/config surface.
- Link to ADR-0027 pack docs for config/policy paths, precedence, key schemas, and defaults instead of restating them.

### `runtime-boundary-spec.md`
- Define one ownership matrix that assigns every touched surface to either Substrate, `substrate-gateway`, or an existing delegated contract doc.
- Define the Substrate-owned boundary responsibilities: policy gate, world placement, lifecycle, host secret sourcing, host-to-world secret delivery ownership, operator UX, and canonical tracing.
- Define the `substrate-gateway`-owned responsibilities: in-world front door, backend adapter internals, provider/planner/executor routing, and normalized event generation.
- Define the exact delegated owners for config/policy (`ADR-0027` pack docs), structured event routing (`ADR-0017`), canonical trace vocabulary (`ADR-0028`), and backend adapter internals (`ADR-0041`).
- Define the rule that gateway-local config, persistence, and admin mutation surfaces are not trusted inputs for Substrate-managed operation.
- Define the ADR-0023 supersession boundary with no ambiguous partial ownership language.

### `gateway-status-schema-spec.md`
- Define the full JSON schema for `substrate world gateway status --json`.
- Define the exact top-level field names for availability, policy posture, lifecycle state, and `client_wiring.*`.
- Define which fields are required, which fields are optional, and what omission means for each optional field.
- Define canonical field types, default values when fields are present, and the no-secrets rule for all serialized values.
- Define whether the schema is versionless or explicitly versioned, and define forward/backward compatibility posture for additive fields.
- State explicitly that this feature does not freeze a separate `substrate world gateway sync --json` schema.

### `platform-parity-spec.md`
- Define Linux, macOS, and Windows placement guarantees in singular, testable language.
- Define which divergences are allowed and which are forbidden across platforms.
- Define the unavailable/prerequisite-missing posture for each platform as it affects the operator contract.
- Define the required validation evidence for each platform, even though this feature is documentation-only.
- Define the invariant that the same Substrate-owned boundary contract applies across all three platforms.

### `manual_testing_playbook.md`
- Define the exact review steps needed to prove the ownership split is internally consistent.
- Define the exact docs to compare for each step: ADR-0040, `contract.md`, `runtime-boundary-spec.md`, `gateway-status-schema-spec.md`, `platform-parity-spec.md`, ADR-0027 pack docs, ADR-0017, ADR-0028, and ADR-0041.
- Define expected pass/fail outcomes for each review step.
- Define the follow-up path when a delegated surface is missing, duplicated, or contradicted.

### `slices/SGBRO0/SGBRO0-spec.md`
- Define that this slice is docs-only and must not introduce production-code scope.
- Define the exact docs to author or update within the feature pack.
- Define the acceptance criteria for a complete ownership matrix, complete status schema selection, and complete platform parity coverage.
- Define the explicit out-of-scope list: policy-schema redesign, gateway runtime protocol design, and telemetry-schema redesign.

### `impact_map.md`
- Define the exact upstream inputs and downstream planning artifacts affected by the selected docs.
- Define the dependency edges to ADR-0027, ADR-0017, ADR-0028, and ADR-0041.
- Define whether any adjacent packs need coordination because this feature changes ownership language they consume.

### `plan.md`
- Define the document-authoring sequence needed to produce the selected specs without reopening delegated contracts.
- Define the order of operations for drafting the contract, boundary matrix, status schema, parity spec, playbook, and slice tasking.
- Define the guardrail that canonical tracked docs outside the feature pack are evidence-only during this feature's drafting phase.

### `tasks.json`
- Define the single selected slice id `SGBRO0` and keep every task traceable to `slices/SGBRO0/SGBRO0-spec.md`.
- Define acceptance criteria that reference doc completeness rather than implementation behavior.
- Define any orchestration/dependency edges needed to keep delegated-source review ahead of local doc authoring.

## Follow-ups

- ADR-0040 references gateway secret-delivery and decision docs only indirectly. Until the ADR-0041 planning pack publishes its own spec set, `runtime-boundary-spec.md` should link `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` as the provisional authoritative owner for detailed gateway adapter/runtime mechanics and should not invent a substitute feature-local mechanics spec.
- After this staged manifest is promoted, update ADR-0040 `Related Docs` to link `spec_manifest.md`, `contract.md`, `runtime-boundary-spec.md`, `gateway-status-schema-spec.md`, and `platform-parity-spec.md`.
