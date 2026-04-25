# agent-hub-core-successor-identity-tuple-compatible — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- Spec manifest:
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- External authorities scanned:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"` (strict packs only).

### Create
- `crates/common/src/agent_identity.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

### Edit
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/plan.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/session_log.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/quality_gate_report.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC0/AHCSITC0-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC1/AHCSITC1-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC3/AHCSITC3-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/tasks.json`
- `docs/project_management/packs/sequencing.json`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/CONFIGURATION.md`
- `docs/USAGE.md`
- `docs/TRACE.md`
- `scripts/linux/world-provision.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/smoke.sh`
- `scripts/windows/wsl-warm.ps1`
- `scripts/windows/wsl-smoke.ps1`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/src/execution/agent_inventory.rs`
- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/execution/routing/telemetry.rs`
- `crates/common/src/agent_events.rs`
- `crates/common/src/lib.rs`
- `crates/common/tests/agent_hub_event_envelope_schema.rs`
- `crates/trace/src/span.rs`
- `crates/trace/src/tests.rs`
- `crates/agent-api-types/src/lib.rs`
- `crates/agent-api-client/src/lib.rs`
- `crates/agent-api-core/src/lib.rs`
- `crates/shell/src/execution/invocation/plan.rs`
- `crates/shell/src/execution/mod.rs`
- `crates/shell/tests/agents_validate.rs`
- `crates/shell/tests/agent_hub_trace_persistence.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/world_gateway.rs`

### Deprecate
- `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

### Delete
- None

## Cascading implications (behavior/UX)

### CLI / UX
- Change: `substrate agent` becomes the canonical namespace for inventory, status, doctor, and toolbox surfaces.
  - Option A: promote `substrate agent list`, `substrate agent status`, and `substrate agent doctor` as the primary root and keep `substrate agents validate` as a compatibility leaf during rollout.
  - Option B: keep the existing plural `substrate agents` root and rewrite ADR-0044 and ADR-0045 around that namespace.
  - Selected option: A.
  - Direct impact: operator docs, help text, manual validation, and toolbox docs all converge on one command family.
  - Cascading impact: `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/agents_cmd.rs`, `docs/USAGE.md`, `contract.md`, `compatibility-spec.md`, and `ADR-0045` move together so list, status, doctor, and toolbox do not fork into separate roots.
  - Contradiction risks: the current repo only exposes `substrate agents validate`, while ADR-0026 and ADR-0045 already reserve `substrate agent toolbox ...`; leaving that split in place creates two incompatible operator stories.

- Change: operator-visible list and status output must separate pure agent records from nested gateway-backed LLM records.
  - Option A: emit a base pure-agent record plus a separate correlated nested LLM record when an agent invokes `substrate_gateway`.
  - Option B: mutate the base pure-agent record in place by adding `provider` and `auth_authority` whenever a nested LLM call happens.
  - Selected option: A.
  - Direct impact: `substrate agent list` and `substrate agent status` keep `provider` and `auth_authority` absent on pure agent runs and only show them on the nested record path.
  - Cascading impact: `crates/common/src/agent_events.rs`, `crates/shell/src/execution/agent_events.rs`, `crates/shell/src/execution/routing/telemetry.rs`, `crates/trace/src/span.rs`, and trace tests must preserve two correlated record families instead of one overloaded record.
  - Contradiction risks: enriching the base agent record with nested gateway fulfillment data breaks ADR-0044’s absence rule and collapses the ownership boundary between agent-hub identity and gateway fulfillment.

### Config / env vars / paths
- Change: ADR-0044 reuses the existing config and policy families and introduces no new config root, env var family, or backend inventory file family.
  - Direct impact: operators keep using the existing `agents.hub.orchestrator_agent_id`, `agents.allowed_backends`, `agents.toolbox.*`, and gateway policy keys inside the current config and policy files.
  - Cascading impact: `docs/CONFIGURATION.md`, `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`, and `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` need synchronized wording for host-scoped orchestrator eligibility, backend allowlist meaning, and no-new-family scope.
  - Contradiction risks: ADR-0025 still describes `agents.hub.orchestrator_agent_id` as a future additive key even though the repo already parses and explains it today.

- Change: `backend_id = "<kind>:<agent_id>"` remains the only agent-side allowlist and attribution token.
  - Direct impact: doctor output, list output, trace output, and compatibility docs all present `backend_id` as adapter identity only.
  - Cascading impact: `agent_inventory.rs`, `contract.md`, `compatibility-spec.md`, ADR-0044, ADR-0045, and trace docs need one statement for derived backend ids, tuple fields, and role separation.
  - Contradiction risks: any wording that treats `backend_id` as a proxy for `client`, `router`, `provider`, `auth_authority`, or `protocol` conflicts with ADR-0042, ADR-0043, ADR-0044, ADR-0045, and existing event-envelope tests.

### Policy / isolation / security posture
- Change: orchestrator selection becomes an explicit fail-closed control-plane gate, not an observational hint.
  - Option A: validate host-scoped orchestrator eligibility inside the `substrate agent doctor` and session-protocol path while leaving raw config parsing unchanged.
  - Option B: reject invalid orchestrator placement at config-parse time for every command path.
  - Selected option: A.
  - Direct impact: `substrate agent doctor` reports missing, disabled, denied, or world-scoped orchestrator state as an actionable failure instead of silently accepting a broken control plane.
  - Cascading impact: `agents_cmd.rs`, `agent_inventory.rs`, `agent-api-*`, `policy-spec.md`, `manual_testing_playbook.md`, and platform parity validation all need the same ordered eligibility checks.
  - Contradiction risks: event-plane visibility alone must not authorize orchestration, and nested LLM approval must not leak backward into agent-hub control-plane approval.

- Change: world-scoped member reuse and restart visibility stay explicit across status, alerts, and trace.
  - Direct impact: operator-visible status must show `world_id` and `world_generation` for world-scoped members and must surface drift restarts and restart-required states coherently.
  - Cascading impact: `crates/common/src/agent_events.rs`, `crates/shell/src/execution/routing/telemetry.rs`, `crates/trace/src/span.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, and `telemetry-spec.md` need one publication rule for world reuse and restart visibility.
  - Contradiction risks: ADR-0017 and current REPL alerts publish `world_generation` inside event data today; any new top-level publication path needs one owner decision before code lands.

### Telemetry / diagnostics
- Change: pure-agent and nested gateway records reuse ADR-0017 event vocabulary and ADR-0028 trace vocabulary.
  - Direct impact: the feature gains no second event envelope and no second trace family.
  - Cascading impact: `crates/common/src/agent_events.rs`, `crates/common/tests/agent_hub_event_envelope_schema.rs`, `crates/trace/src/span.rs`, `crates/trace/src/tests.rs`, and `docs/TRACE.md` must align on field placement, omission rules, and correlation keys.
  - Contradiction risks: any successor implementation that creates an agent-hub-only trace shape or infers nested semantics from `backend_id` will drift away from ADR-0017 and ADR-0028.

### Implementation placement
- Change: the successor implementation path is anchored in existing crates instead of introducing a new crate during this planning boundary.
  - Option A: create a new `crates/agent-hub` crate immediately and move registry, session routing, and status rendering into that new package.
  - Option B: land the successor work in existing `crates/shell`, `crates/common`, and `crates/agent-api-*` surfaces first, then add a dedicated crate later only if the codebase shows an unavoidable seam.
  - Selected option: B.
  - Direct impact: the strict touch set stays exact today because the relevant files already exist in the repo.
  - Cascading impact: `spec_manifest.md`, ADR-0044, and ADR-0045 need synchronized wording so later planning does not assume a `crates/agent-hub` path that does not exist.
  - Contradiction risks: keeping `crates/agent-hub` as the implied owner in planning docs while execution work lands in existing crates will fracture ownership and triad scoping.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - Overlap surfaces: agent-event envelope, routing attribution, world restart alerts, `backend_id`, `world_id`, and trace join keys.
  - Conflict: no
  - Resolution (explicit): ADR-0017 remains the event-envelope and routing owner; ADR-0044 reuses that vocabulary and only adds successor agent-hub semantics plus operator command surfaces.

- ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
  - Overlap surfaces: `substrate agent list`, `substrate agent status`, `substrate agent doctor`, derived `backend_id`, host-scoped orchestrator selection, shared-world reuse, and restart alert taxonomy.
  - Conflict: yes
  - Resolution (explicit): treat ADR-0025 as superseded evidence, deprecate it as a live source, and carry forward only the compatible rules that ADR-0044 retains.

- ADR: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
  - Overlap surfaces: `substrate agent toolbox ...` namespace, orchestrator identity, session visibility, and toolbox dependence on agent-hub inventory.
  - Conflict: yes
  - Resolution (explicit): ADR-0045 is the live toolbox contract, and ADR-0044 supplies the agent-hub inventory and session semantics that toolbox status and env surfaces depend on.

- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - Overlap surfaces: `agents.hub.orchestrator_agent_id`, `agents.allowed_backends`, `agents.toolbox.*`, and the current config and policy file families.
  - Conflict: no
  - Resolution (explicit): ADR-0027 remains the owner of config roots, precedence, and stored key paths; ADR-0044 narrows how those existing keys drive agent-hub control-plane behavior.

- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces: `client`, `router`, `protocol`, `provider`, `auth_authority`, placement posture, and the rule that `backend_id` is not a tuple surrogate.
  - Conflict: no
  - Resolution (explicit): ADR-0042 remains the semantic owner of tuple meaning; ADR-0044 reuses those tuple semantics for pure agent runs and nested gateway-backed records.

- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces: nested gateway-policy reuse, tuple-aware deny explanations, and telemetry wording that keeps `backend_id` separate from tuple meaning.
  - Conflict: no
  - Resolution (explicit): ADR-0043 remains the owner of tuple-axis policy gates; ADR-0044 consumes those gates for nested LLM requests and does not create a second policy surface.

- ADR: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - Overlap surfaces: toolbox command namespace, orchestrator identity, session state, `backend_id`, trace correlation, and pure-agent omission rules.
  - Conflict: yes
  - Resolution (explicit): ADR-0044 supplies the successor agent-hub identity and session state that ADR-0045 depends on, and ADR-0045 stays on toolbox transport and trace semantics instead of redefining hub inventory ownership.

- ADR: `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - Overlap surfaces: backend selection, runtime generation, and the boundary between agent-hub approval and gateway nested-request approval.
  - Conflict: no
  - Resolution (explicit): ADR-0046 remains on gateway runtime integration; ADR-0044 keeps nested requests on the existing gateway policy path and does not widen gateway ownership.

### Relevant Planning Packs (landed + live draft lineage)
- Planning Pack: `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/`
  - Overlap surfaces: agent-event envelope, `backend_id`, `world_id`, world restart alerts, and trace flattening.
  - Conflict: no
  - Resolution (explicit): that pack remains the transport and trace owner, while ADR-0044 adds successor hub inventory, status, doctor, and nested-record interpretation on top of the existing event plane.

- Planning Pack: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
  - Overlap surfaces: tuple vocabulary, placement posture, and the rule that `backend_id` stays adapter-only.
  - Conflict: no
  - Resolution (explicit): that pack remains the tuple-semantics owner; ADR-0044 consumes the tuple vocabulary for pure agent and nested gateway records.

- Planning Pack: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
  - Overlap surfaces: tuple-aware deny semantics, telemetry wording, and `backend_id` separation from tuple meaning.
  - Conflict: no
  - Resolution (explicit): that pack owns tuple-policy semantics and deny publication; ADR-0044 only names the boundary that nested LLM requests reuse those gates.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership-fse/`
  - Overlap surfaces: gateway ownership boundary, the seam-2 `status --json` / `client_wiring.*` owner line, typed runtime/status parity, and the rule that gateway ownership does not spread into a second control plane.
  - Conflict: no
  - Resolution (explicit): treat the `-fse` seam pack as the live planning lineage; ADR-0044 reuses its gateway ownership boundary for nested request routing and does not create new gateway operator commands or gateway-local config roots.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
  - Overlap surfaces: backward-compatible publication path for the gateway operator, policy-evaluation, runtime-parity, and status-schema contracts already mirrored out of the seam pack.
  - Conflict: no
  - Resolution (explicit): use this legacy pack path only when the published contract docs are the authority being cited; live planning lineage stays with `substrate-gateway-boundary-and-runtime-ownership-fse`.

- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
  - Overlap surfaces: backend adapter identity, capability gating, and `backend_id` stability.
  - Conflict: no
  - Resolution (explicit): that pack remains the backend-adapter owner, while ADR-0044 keeps `backend_id` as the agent-side allowlist and attribution token derived from agent inventory.

- Planning Pack: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/`
  - Overlap surfaces: inventory-backed backend selection, capability gating, adapter-driven runtime realization, and the nested-request execution path that later pure-agent sessions will call through.
  - Conflict: no
  - Resolution (explicit): ADR-0046 remains the live gateway-runtime follow-on. ADR-0044 depends on its stable backend-id and nested-request boundary, but does not implement the multi-adapter gateway runtime seam itself.

- Planning Pack: `docs/project_management/_archived/next/agent_hub_core/`
  - Overlap surfaces: backend-id derivation, explicit orchestrator selection, shared-world reuse, and restart alert taxonomy.
  - Conflict: yes
  - Resolution (explicit): treat the archived pack as evidence-only and carry forward only the decisions that ADR-0044 still adopts.

- Planning Pack: `docs/project_management/_archived/next/orchestration_mcp_toolbox/`
  - Overlap surfaces: historical toolbox dependency on an agent-hub inventory and session source.
  - Conflict: yes
  - Resolution (explicit): treat the archived toolbox pack as historical evidence only; the live toolbox dependency chain is ADR-0045 plus ADR-0044.

## Downstream sequence after this pack

This pack closes the successor Agent Hub contract boundary only. Continuing toward fuller AI functionality still requires:

- `docs/project_management/packs/draft/orchestration-toolbox-mcp/` or the eventual ADR-0045 successor pack directory
  - Why next: ADR-0045 depends on ADR-0044’s canonical `substrate agent ...` namespace plus the locked orchestrator/session/back-end-id semantics from this pack before toolbox status/env/tool-call surfaces can land cleanly.
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/`
  - Why next: ADR-0046 is the adjacent runtime-realization seam that moves the integrated gateway beyond the current `cli:codex` proof path and makes nested multi-backend LLM execution real rather than merely contract-compatible.
- Execution of the ADR-0042 and ADR-0043 packs before AHCSITC implementation work if those upstream semantic packs remain unlanded
  - Why prerequisite: ADR-0044 consumes tuple meaning and tuple-policy gates; it does not redefine them.

## Follow-ups (explicit)

- Decision Register entries required:
  - `DR-AHCSITC-01` — lock the canonical CLI namespace on `substrate agent ...` and define the compatibility posture for `substrate agents validate`.
  - `DR-AHCSITC-02` — lock successor implementation placement on existing `crates/shell`, `crates/common`, and `crates/agent-api-*` surfaces for this feature boundary.
  - `DR-AHCSITC-03` — lock nested LLM publication on a separate correlated record instead of mutating the base pure-agent record.

- Spec updates required (if any):
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md` — pin the exact publication path for `world_generation` before execution work changes the envelope or trace flattening.
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md` — tighten any additional new test-file paths if execution work introduces dedicated command or session-protocol tests beyond the existing test files listed in the touch set.
