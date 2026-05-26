# substrate-gateway-boundary-and-runtime-ownership — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- Spec manifest:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/"` (strict packs only).

### Create
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/quality_gate_report.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/session_log.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`

### Edit
- `crates/transport-api-client/src/lib.rs`
- `crates/transport-api-types/src/lib.rs`
- `crates/shell/src/builtins/mod.rs`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/execution/platform/mod.rs`
- `crates/world-service/src/handlers.rs`
- `crates/world-service/src/lib.rs`
- `crates/world-service/src/service.rs`
- `docs/CONFIGURATION.md`
- `docs/TRACE.md`
- `docs/USAGE.md`
- `docs/WORLD.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
- `docs/project_management/packs/sequencing.json`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: `substrate world gateway status`, `substrate world gateway sync`, and `substrate world gateway restart` become first-class world subcommands instead of archived draft command sketches.
  - Direct impact:
    - Operators get one stable command family for gateway availability, wiring discovery, and restart flows.
    - `substrate world gateway status --json` becomes the single machine-readable operator surface for gateway status and wiring.
  - Cascading impact:
    - `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/platform/mod.rs`, `crates/shell/src/builtins/world_gateway.rs`, `docs/USAGE.md`, and `crates/shell/tests/world_gateway.rs` must land together.
    - `sync` and `status --json` must share one `client_wiring.*` vocabulary so text and JSON surfaces never diverge.
  - Contradiction risks:
    - `docs/project_management/_archived/next/llm_gateway_in_world/contract.md` still publishes `substrate world status gateway` and `substrate world sync gateway`; without a historical-evidence-only banner, the archive can read like a second operator contract.
    - Backend-specific status text from ADR-0041 cannot leak into the base gateway status surface before this pack locks the base envelope.

- Change: gateway lifecycle and status transport runs through a dedicated world-service surface instead of raw exec probing.
  - Option A: add typed gateway lifecycle and status endpoints to world-service and consume them through `crates/transport-api-types/src/lib.rs` and `crates/transport-api-client/src/lib.rs`.
  - Option B: shell builtins issue ad hoc exec probes through the existing execute transport and assemble status JSON locally.
  - Selected option: A.
  - Direct impact:
    - CLI behavior stays stable across Linux, macOS, and Windows because Substrate owns one typed lifecycle and status surface.
    - Gateway status JSON stays detached from gateway binary internals and detached from per-platform shell probing.
  - Cascading impact:
    - `crates/world-service/src/lib.rs`, `crates/world-service/src/handlers.rs`, `crates/world-service/src/service.rs`, `crates/transport-api-types/src/lib.rs`, and `crates/transport-api-client/src/lib.rs` move as a set.
    - The manual playbook must verify that `status`, `sync`, and `restart` all exercise the same internal lifecycle state and failure taxonomy.
  - Contradiction risks:
    - Exec-probe status would make operator JSON depend on runtime-private commands and would reintroduce the second-control-plane drift ADR-0040 blocks.
    - Platform-specific probes would fracture parity and would force `docs/WORLD.md` to document transport quirks as user contract.

### Config / env vars / paths
- Change: ADR-0040 reuses ADR-0027 file families and key paths and introduces no new config family.
  - Direct impact:
    - Operators continue to use `$SUBSTRATE_HOME/config.yaml`, `<workspace_root>/.substrate/workspace.yaml`, `$SUBSTRATE_HOME/policy.yaml`, and `<workspace_root>/.substrate/policy.yaml`.
    - Existing keys stay authoritative: `llm.gateway.enabled`, `llm.gateway.mode`, `llm.routing.default_backend`, `llm.allowed_backends`, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends`.
  - Cascading impact:
    - `docs/CONFIGURATION.md`, `contract.md`, and `policy-spec.md` must state that gateway bind endpoints, gateway-local admin state, and gateway-local config files are not operator-controlled config surfaces.
    - `tasks.json` and `plan.md` must preserve that reuse boundary so later slices do not invent a `gateway.yaml` or a second policy file family.
  - Contradiction risks:
    - Any requirement for gateway-local config or admin setup as a normal Substrate-managed prerequisite would violate ADR-0040.
    - Any implementation that treats gateway-local persistence as a trusted policy input would violate ADR-0040 and ADR-0027 together.

- Change: `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` stay the only stable non-secret wiring env outputs.
  - Direct impact:
    - Operators and in-world clients keep one stable discovery path for gateway endpoints.
    - Human-readable output stays free to abbreviate, but the JSON surface and env names stay fixed.
  - Cascading impact:
    - `gateway-status-schema-spec.md` must lock the `client_wiring.*` field family.
    - `contract.md` and `docs/USAGE.md` must pin the exact meaning of both env names and must keep them pointed at Substrate-managed gateway endpoints rather than upstream providers.
  - Contradiction risks:
    - Archived gateway planning still contains transport-specific wording and alternate command ordering; lifting that wording into product docs would fracture the discovery contract.
    - ADR-0042 additive tuple metadata cannot displace `client_wiring.*` as the wiring authority.

### Policy / isolation / security posture
- Change: fail-closed in-world placement stays active when policy requires it, and host secret sourcing stays Substrate-owned.
  - Direct impact:
    - Operators get one deterministic exit boundary: `2` invalid config or invalid integration state, `3` transient runtime failure, `4` missing world or gateway component, `5` policy or safety denial.
    - `host_only` remains blocked when `llm.fail_closed.routing=true`.
  - Cascading impact:
    - `policy-spec.md`, `contract.md`, `docs/WORLD.md`, and the world-service lifecycle surface must all agree on placement failure, dependency failure, and denial wording.
    - The manual playbook must verify that no host fallback path appears when policy requires in-world execution.
  - Contradiction risks:
    - Any host fallback when policy requires in-world execution would violate ADR-0040 and ADR-0042.
    - Any status text that treats gateway-local token persistence or gateway-local admin state as required trust inputs would create a second control plane.

- Change: Substrate trace remains the canonical operator telemetry surface and gateway-local trace stays implementation-local.
  - Direct impact:
    - Operators keep using Substrate trace and Substrate status output for audit and troubleshooting.
  - Cascading impact:
    - `docs/TRACE.md`, `platform-parity-spec.md`, and `manual_testing_playbook.md` must state that gateway-local trace files are not required and are not authoritative.
    - ADR-0017 and ADR-0028 stay the only authorities for structured event routing and canonical trace vocabulary.
  - Contradiction risks:
    - If gateway docs redefine trace field names or correlation semantics, queued identity and adapter work will inherit two trace authorities.

- Change: provisioning stays outside this pack’s implementation boundary.
  - Option A: edit `scripts/linux/`, `scripts/mac/`, `scripts/windows/`, and backend-specific provisioning surfaces in this pack.
  - Option B: leave provisioning changes for a later gateway runtime pack after the runtime binary layout and service shape are fixed.
  - Selected option: B.
  - Direct impact:
    - This pack locks the ownership boundary, command surface, status envelope, and validation rules first.
  - Cascading impact:
    - Later runtime work must consume this pack’s contract before touching platform provisioning or world backend warm flows.
    - `platform-parity-spec.md` must name the evidence expected before any script or backend change is accepted.
  - Contradiction risks:
    - Pulling provisioning into this pack would freeze service names and file layout before ADR-0041 and the runtime pack finalize adapter and runtime ownership.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - Overlap surfaces:
    - `substrate world gateway status|sync|restart`
    - `client_wiring.*`
    - host-to-world secret delivery
    - gateway runtime ownership
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0040 is the ownership authority.
    - ADR-0023 stays historical context only.
    - Archived `llm_gateway_in_world` pack artifacts stay evidence only and do not define the current runtime owner or current command spelling.

- ADR: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - Overlap surfaces:
    - gateway lifecycle commands
    - status output
    - backend capability visibility
    - adapter-backed runtime behavior
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0040 owns the Substrate-facing lifecycle boundary, policy posture, wiring discovery, and operator-visible status contract.
    - ADR-0041 owns backend adapter identity, capability semantics, and gateway runtime internals.
    - Base status availability and `client_wiring.*` fields land here before backend-capability additions land in ADR-0041 follow-on work.

- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces:
    - `substrate world gateway status --json`
    - placement posture language
    - router identity
    - no-second-host-gateway invariant
  - Conflict: yes
  - Resolution (explicit):
    - Option A: this pack owns the full status JSON including tuple metadata and placement-posture metadata.
    - Option B: this pack owns the status envelope, gateway availability, policy posture, absence semantics, and `client_wiring.*`, while ADR-0042 owns tuple metadata and additive placement-posture metadata.
    - Selected option: B.

- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - Overlap surfaces:
    - policy evaluation for router, provider, protocol, and auth-authority constraints
    - `status --json` explanations of policy posture
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0040 consumes ADR-0027 and ADR-0043 policy keys.
    - ADR-0040 does not define new policy schema.
    - `policy-spec.md` in this pack documents evaluation over existing keys only.

- ADR: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - Overlap surfaces:
    - router terminology
    - control-plane versus event-plane wording
    - no-second-execution-plane posture
  - Conflict: yes
  - Resolution (explicit):
    - Gateway docs keep `substrate_gateway` as the fulfillment router.
    - Toolbox docs keep `agent_toolbox` as a control-plane router only.
    - `docs/TRACE.md` and later status docs must keep those meanings separate.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/`
  - Overlap surfaces:
    - config and policy file locations
    - key paths and defaults
    - host secret-read gates
  - Conflict: no
  - Resolution (explicit):
    - That pack stays the authority for config and policy storage, schema, and precedence.
    - This pack consumes those surfaces and does not redefine them.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - canonical trace vocabulary
    - operator trace expectations
    - validation posture for trace docs
  - Conflict: no
  - Resolution (explicit):
    - That pack remains the trace authority.
    - This pack adds no new trace schema and only clarifies the gateway-versus-Substrate ownership boundary in `docs/TRACE.md`.

- Planning Pack: `docs/project_management/_archived/next/llm_gateway_in_world/`
  - Overlap surfaces:
    - command family
    - wiring env vars
    - secret-delivery narrative
    - status JSON ownership
  - Conflict: yes
  - Resolution (explicit):
    - Treat the archived pack as historical evidence only; the live operator contract is `docs/contracts/gateway/operator-contract.md`.
    - Carry forward only the stable wiring env names and the in-world gateway intent.
    - Do not carry forward Substrate-owned gateway runtime crates, archived command ordering, or archived transport details as current contract.

- Planning Pack: `docs/project_management/_archived/next/llm_cli_backend_engine/`
  - Overlap surfaces:
    - host credential read gating
    - secret delivery into the in-world runtime
    - backend status hints
  - Conflict: yes
  - Resolution (explicit):
    - Carry forward only the existing policy gates and the later secret-channel posture.
    - Do not carry forward the archived `llm-manager` ownership model or env-injection-as-primary-contract language into the current Substrate boundary pack.

## Follow-ups (explicit)

- Decision Register entries required:
  - None. The selected A/B resolutions above are boundary locks for this pre-planning artifact. Add a feature-local decision register later only if planning introduces a new operator-visible option beyond these selections.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md` — add an explicit internal-surface note for `crates/transport-api-types/src/lib.rs` and `crates/transport-api-client/src/lib.rs` so the dedicated world-service transport choice is reflected in the authoritative ownership map.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md` — lock the exact command spelling, the exit `2|3|4|5` split, and the rule that `status --json` is the operator wiring authority.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` — lock the `client_wiring.*` field family and the additive ownership boundary against ADR-0042 tuple metadata.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md` — state that provisioning is outside this pack and list the evidence required before any platform script or backend change lands.
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` — keep related-doc links aligned with `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*` when cross-ADR alignment work opens.
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` — keep related-doc links aligned with `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*` when cross-ADR alignment work opens.
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` — keep related-doc links aligned with `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*` when cross-ADR alignment work opens.
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` — normalize self-links away from `packs/active/...` so downstream ADRs stop copying stale pack paths.
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/spec_manifest.md` — normalize self-links away from `packs/active/...` so downstream ADRs stop copying stale pack paths.
