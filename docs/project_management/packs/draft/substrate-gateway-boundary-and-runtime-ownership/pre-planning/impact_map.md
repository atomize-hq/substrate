# substrate-gateway-boundary-and-runtime-ownership — impact map

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

### Create
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`

### Edit
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

### Deprecate
- None

### Delete
- None

## Downstream implementation surfaces (evidence-only for this pack)

These surfaces are implicated by ADR-0040, but the selected scope for this feature is documentation-only boundary clarification. They are not part of this pack's authoring touch set.

- `crates/shell/src/execution/cli.rs`
  - Current `WorldAction` only exposes `Doctor|Enable|Deps|Cleanup|Verify`; any future `world gateway` command family starts here.
- `crates/shell/src/execution/platform/mod.rs`
  - Current world-command dispatch has no gateway branch, so downstream runtime work must route lifecycle/status operations here.
- `crates/shell/src/builtins/mod.rs`
  - Future gateway lifecycle/status builtin registration lands here.
- `crates/shell/src/builtins/`
  - Directory-prefix fallback for the eventual gateway builtin module; exact filenames are not yet defensible from ADR-0040 plus the current spec set.
- `crates/world-agent/src/`
  - Directory-prefix fallback for future lifecycle/status orchestration and host-to-world gateway control transport.
- `crates/world/`
  - Linux world backend integration surface for actual in-world placement behavior.
- `crates/world-mac-lima/`
  - macOS world backend integration surface for equivalent placement behavior.
- `crates/world-windows-wsl/`
  - Windows/WSL world backend integration surface for equivalent placement behavior.
- `docs/COMMANDS.md`
  - Future operator command reference will need the canonical `world gateway` grammar once runtime work exists.
- `docs/CONFIGURATION.md`
  - Future operator docs will need the stable explanation that ADR-0040 adds no new config family and delegates config/policy keys to ADR-0027.
- `docs/ENVIRONMENT_VARIABLES.md`
  - Future operator docs will need the non-secret wiring env vars once the runtime surface exists.
- `docs/USAGE.md`
  - Future usage docs will need examples for gateway status/sync/restart once implemented.
- `docs/WORLD.md`
  - Future world docs will need placement/lifecycle narrative once the gateway runtime is actually integrated.

## Cascading implications (behavior/UX)

### CLI / UX
- Change: ADR-0040 freezes the canonical operator grammar as `substrate world gateway status`, `substrate world gateway sync`, and `substrate world gateway restart`.
  - Direct impact:
    - Future CLI help, playbooks, smoke steps, and runtime docs have one canonical grammar.
  - Cascading impact:
    - Archived `llm_gateway_in_world` material and any later runtime pack must treat reversed `substrate world status|sync gateway` grammar as historical only.
    - The eventual `clap` tree and shell dispatch cannot preserve both grammars as co-equal primaries without splitting operator guidance.
  - Contradiction risks:
    - Mixed command grammar across ADRs, archived packs, and runtime docs would create an inconsistent operator contract and ambiguous implementation target.
- Change: `substrate world gateway status --json` is the only stable JSON wiring-discovery surface frozen by this feature.
  - Direct impact:
    - Operators and downstream tooling must use `status --json` for stable `client_wiring.*` discovery.
  - Cascading impact:
    - `sync` remains lifecycle-oriented; if a later runtime pack adds `sync --json`, it must either reuse the `status` shape inside a clearly additive surface or keep it non-contractual.
  - Contradiction risks:
    - Archived ADR-0023 and archived gateway pack text still implies `sync --json` is equally authoritative; carrying that forward would silently expand scope beyond ADR-0040 and the selected spec set.

### Config / env vars / paths
- Change: ADR-0040 adds no new config or policy file family and keeps `SUBSTRATE_LLM_OPENAI_BASE_URL` plus `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` as stable non-secret wiring names.
  - Direct impact:
    - The new feature-local docs must link to ADR-0027 ownership for file locations, precedence, `llm.*` keys, `llm.secrets.env_allowed`, and `agents.host_credentials.read.allowed_backends` rather than restating them.
  - Cascading impact:
    - Operator docs must describe these base URLs as Substrate-managed gateway endpoints intended for in-world reachability, not upstream provider endpoints and not a guarantee of direct host reachability.
    - Any future global JSON envelope work must wrap, not redefine, the command-local `status --json` schema frozen here.
  - Contradiction risks:
    - `pre-planning/spec_manifest.md` and ADR-0040 both still reference non-existent `docs/project_management/packs/active/llm_and_agent_config_policy_surface/...` paths even though the authoritative pack now lives under `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/...`.
    - Treating the `SUBSTRATE_LLM_*` values as provider URLs would contradict both ADR-0040 and the archived gateway contract.

### Policy / isolation / security posture
- Change: Substrate remains the sole owner of policy gating, world placement, lifecycle, host secret sourcing, and host-to-world secret delivery ownership for integrated gateway operation.
  - Direct impact:
    - This pack can clarify the ownership boundary, but it cannot invent gateway-local admin/config surfaces or detailed auth-field/transport mechanics.
  - Cascading impact:
    - Downstream implementation must look like Substrate orchestrating an in-world gateway runtime, not Substrate consuming a second, broader gateway control plane.
    - Any later runtime pack must continue to treat ADR-0027 as the only owner for config/policy gates and ADR-0041 as the runtime-internals owner.
  - Contradiction risks:
    - Pulling concrete secret-delivery mechanics from archived env-injection or engine materials into this pack would reopen scope in the wrong owner document.
    - Allowing gateway-local config, persistence, or admin mutation surfaces to become required for Substrate-managed operation would violate ADR-0040's anti-second-control-plane rule.
- Change: Linux, macOS, and Windows all inherit the same boundary-ownership contract.
  - Direct impact:
    - The parity spec must state any allowed divergence explicitly instead of implying parity by omission.
  - Cascading impact:
    - Future runtime/test planning cannot silently make Linux truly in-world while macOS or Windows become host-side bridges under the same operator wording.
  - Contradiction risks:
    - ADR-0042's `host_to_world_bridge` language must remain transport-only; if it is interpreted as a second permanent host gateway, ADR-0040's ownership boundary is broken.

### Trace / identity / status semantics
- Change: canonical tracing remains delegated to ADR-0028, and structured-event/identity carry-over remains delegated to ADR-0017, ADR-0041, ADR-0042, and ADR-0045.
  - Direct impact:
    - ADR-0040 docs may surface placement posture, lifecycle state, and non-secret wiring, but they must not mint a parallel trace vocabulary or overload `backend_id` with identity-tuple meaning.
  - Cascading impact:
    - Future status or lifecycle code must reuse the existing correlation and identity vocabulary once those ADRs land, or keep such fields absent until they do.
  - Contradiction risks:
    - Freezing provider/auth/router tuple semantics in `gateway-status-schema-spec.md` now would create a second source of truth before ADR-0041 and ADR-0042 finish their own contract selection.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`
  - Overlap surfaces:
    - gateway command grammar
    - wiring output ownership
    - `status --json` versus `sync --json`
    - secret-delivery narrative
  - Conflict: yes
  - Resolution (explicit):
    - A) preserve ADR-0023 grammar and dual `status`/`sync` JSON authority
    - B) treat ADR-0040 as the forward contract, keep ADR-0023 historical only, and freeze only `status --json` in this feature
    - Selected: B
- ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`
  - Overlap surfaces:
    - gateway status visibility for backends
    - runtime ownership of backend execution
    - secret-delivery details for CLI/API backends
  - Conflict: yes
  - Resolution (explicit):
    - A) keep Substrate-local engine assumptions alive inside this pack
    - B) keep ADR-0040 boundary-only and delegate adapter/runtime internals to ADR-0041
    - Selected: B
- ADR: `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - Overlap surfaces:
    - backend-id stability
    - capability visibility in status
    - adapter/runtime internals
  - Conflict: yes
  - Resolution (explicit):
    - A) invent local adapter/runtime detail inside ADR-0040 docs
    - B) treat ADR-0041 itself as the provisional delegated owner until its planning pack/specs exist on disk
    - Selected: B
- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces:
    - placement posture names
    - router identity hints
    - `host_to_world_bridge` terminology
  - Conflict: potential
  - Resolution (explicit):
    - A) freeze identity-tuple fields directly in `gateway-status-schema-spec.md`
    - B) allow only boundary-safe placement/router hints here and defer tuple semantics to ADR-0042 plus ADR-0043
    - Selected: B
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces:
    - canonical trace vocabulary
    - world placement observability
  - Conflict: no
  - Resolution (explicit):
    - ADR-0040 delegates trace semantics and must not create a feature-local telemetry contract.
- ADR: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - Overlap surfaces:
    - structured-event routing vocabulary
    - future nested gateway-backed record attribution
  - Conflict: no
  - Resolution (explicit):
    - ADR-0040 stays boundary/status scoped and does not redefine event-envelope semantics.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/_archived/next/llm_gateway_in_world/`
  - Overlap surfaces:
    - gateway commands
    - client wiring env vars
    - secret-delivery material
    - archived contract/playbook assumptions
  - Conflict: yes
  - Resolution (explicit):
    - A) keep the archived pack as a live source of operator truth
    - B) treat it as historical context only and let ADR-0040 plus the new feature-local specs replace its ownership language
    - Selected: B
- Planning Pack: `docs/project_management/_archived/next/llm_cli_backend_engine/`
  - Overlap surfaces:
    - backend engine assumptions
    - CLI backend auth/runtime story
  - Conflict: yes
  - Resolution (explicit):
    - A) carry forward the archived engine model
    - B) treat it as superseded context and route all adapter/runtime ownership to ADR-0041
    - Selected: B
- Planning Pack: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/`
  - Overlap surfaces:
    - config/policy file locations
    - backend allowlists
    - host secret-read gates
    - stable pack links referenced by ADR-0040 and `pre-planning/spec_manifest.md`
  - Conflict: yes
  - Resolution (explicit):
    - A) duplicate the implemented pack under an `active` path to satisfy stale links
    - B) treat the implemented pack as authoritative, update feature-local links to it, and record broader PM link cleanup as follow-up work
    - Selected: B
- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - canonical trace vocabulary
    - world/session/span semantics
  - Conflict: no
  - Resolution (explicit):
    - ADR-0040 boundary docs link to this pack as the trace owner and avoid introducing gateway-local trace terminology.
- Planning Pack: `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/`
  - Overlap surfaces:
    - structured-event routing vocabulary
    - future nested gateway-backed record semantics
  - Conflict: no
  - Resolution (explicit):
    - ADR-0040 boundary docs remain status/boundary scoped and leave event-envelope ownership with the implemented routing pack.
- Planning Pack: `docs/project_management/packs/draft/json-mode/`
  - Overlap surfaces:
    - `--json` output posture for command surfaces
  - Conflict: potential
  - Resolution (explicit):
    - A) let ADR-0040 define a standalone top-level JSON envelope
    - B) keep ADR-0040 responsible only for the command-local `status --json` data shape and let the global JSON-mode work own any outer envelope
    - Selected: B

## Follow-ups (explicit)
- Decision Register entries required:
  - None. The required A/B choices for this single-output run are recorded above instead of opening `decision_register.md`.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md` — update delegated ADR-0027 owner links from `docs/project_management/packs/active/llm_and_agent_config_policy_surface/...` to `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/...`.
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md` — add promoted feature-local doc links and correct stale ADR-0027 pack paths.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md` — state explicitly that adapter/runtime mechanics remain delegated to ADR-0041 until its draft pack/specs exist.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md` — keep identity-tuple fields optional/delegated and keep `sync --json` out of the frozen schema surface unless scope changes.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json` — keep this feature docs-only and do not schedule production-code work inside the SGBRO0 slice.
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md` — include explicit checks that archived gateway/engine materials are treated as historical-only and that the implemented ADR-0027 pack paths are the ones linked.
- Tightening required later:
  - Tighten the directory-prefix downstream implementation surfaces `crates/shell/src/builtins/`, `crates/world-agent/src/`, `crates/world/`, `crates/world-mac-lima/`, and `crates/world-windows-wsl/` into exact file paths when runtime implementation planning begins.
  - Reconcile stale `packs/active/...` references in PM registry docs such as `docs/project_management/packs/sequencing.json` as separate PM cleanup work; do not duplicate or move implemented packs as part of this feature.
