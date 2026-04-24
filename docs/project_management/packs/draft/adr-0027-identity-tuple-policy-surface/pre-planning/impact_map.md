# adr-0027-identity-tuple-policy-surface — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Spec manifest:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` (strict packs only).

### Create
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/plan.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md`

### Edit
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tasks.json`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/CONFIGURATION.md`
- `docs/USAGE.md`
- `docs/TRACE.md`
- `docs/reference/policy/contract.md`
- `docs/reference/policy/README.md`
- `crates/shell/src/execution/policy_cmd.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/broker/src/policy/tests.rs`
- `crates/broker/src/tests.rs`
- `crates/trace/src/span.rs`
- `crates/trace/src/tests.rs`

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
- Change: `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities` stay on the policy inspection surface, with `substrate policy current show --explain` as the authoritative merged-view command.
  - Direct impact: operator guidance, examples, and acceptance criteria move to `substrate policy current show --explain` for tuple-axis inspection instead of treating config and policy as interchangeable roots.
  - Cascading impact: ADR-0043, the implemented ADR-0027 contract pack, `docs/CONFIGURATION.md`, `docs/reference/policy/contract.md`, and `docs/reference/policy/README.md` need aligned examples, key ordering, and provenance wording so operators can find the active constraint set in one place.
  - Contradiction risks: ADR-0043 currently says the config effective view surfaces the new keys, while the codebase already renders them on the policy effective view. Leaving that split in place creates two incompatible operator stories.
- Change: tuple-axis denials remain separate from backend-id selection and publish router/provider/protocol/auth-authority mismatch reasons without relabeling `backend_id`.
  - Direct impact: an operator sees backend ids as adapter gates and tuple fields as semantic routing axes, which keeps troubleshooting aligned with ADR-0042 and the gateway status contract.
  - Cascading impact: `substrate world gateway status`, deny messaging, test fixtures, and troubleshooting docs need one vocabulary for `identity_tuple`, `placement_posture`, and tuple-axis denial causes.
  - Contradiction risks: any output that treats `backend_id` as a provider or auth-authority surrogate breaks ADR-0042, ADR-0044, ADR-0045, and the existing gateway status JSON shape.

### Config / env vars / paths
- Change: this feature stays additive inside the existing policy files and does not introduce a second config system, new policy files, or new environment-variable knobs.
  - Direct impact: operators keep using `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`; they add `llm.constraints.*` entries rather than new file families or per-backend tuple shims.
  - Cascading impact: `tuple-policy-schema-spec.md`, `policy-spec.md`, `docs/CONFIGURATION.md`, `docs/reference/policy/contract.md`, and `docs/reference/policy/README.md` need exact examples for empty-list semantics, invalid token rejection, and the rule that `client` remains outside the standalone policy-key surface in v1.
  - Contradiction risks: the implemented ADR-0027 pack still says tuple-axis keys are follow-on work rather than live policy surface, so current docs understate the already-shipped key family.
- Change: validation examples that mention `~/.codex/auth.json` remain validation-only examples rather than new Substrate-owned path contracts.
  - Direct impact: manual testing can reference real tool auth files without creating a new product path guarantee.
  - Cascading impact: `manual_testing_playbook.md`, ADR-0043, and `docs/USAGE.md` need the same wording so reviewers do not treat the example path as a new config root.
  - Contradiction risks: turning the example path into an owned contract expands scope beyond the ADR and collides with existing auth-source ownership outside Substrate.

### Policy / isolation / security posture
- Change: tuple-axis constraints act as narrowing policy gates layered on top of backend allowlists, fail-closed routing, host credential-read gates, and `net_allowed`.
  - Direct impact: a backend can remain allowed while a request is denied for router, provider, protocol, or auth-authority mismatch, and that denial path must explain the exact mismatch axis.
  - Cascading impact: policy tests, gateway deny tests, docs, and trace publication all need the same ordered evaluation story: backend gate first, tuple-axis narrowing second, secret-read and egress gates preserved, no implicit host fallback, and `host_to_world_bridge` staying transport-only.
  - Contradiction risks: ADR-0043 presents the tuple-axis keys as additive future work, while `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/policy_cmd.rs`, `crates/broker/src/tests.rs`, and `crates/shell/tests/world_gateway.rs` already expose and enforce them. Leaving the ADR and docs unchanged would describe a repo that no longer exists.
- Change: tuple-policy telemetry uses one tuple field family across gateway status, agent-event publication, and trace publication.
  - Direct impact: allow and deny records use the same `identity_tuple` and `placement_posture` object names instead of introducing a second trace-only tuple shape.
  - Cascading impact: `telemetry-spec.md`, `docs/TRACE.md`, `crates/trace/src/span.rs`, and `crates/trace/src/tests.rs` need to pin the exact additive fields and deny-projection rules, while `backend_id` remains a separate correlation and selector field.
  - Contradiction risks:
    - Option A: reuse the existing `identity_tuple` and `placement_posture` field family already present on gateway status and agent-event records, then extend trace publication and denial metadata around that family.
    - Option B: add a second trace-only tuple publication shape that duplicates the same semantics under different field names.
    - Selected option: Option A. It preserves one tuple vocabulary, keeps ADR-0042 and ADR-0028 owner boundaries intact, and avoids parallel trace schemas that would drift out of sync.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - Overlap surfaces: semantic meaning of `client`, `router`, `provider`, `protocol`, `auth_authority`, `identity_tuple`, `placement_posture`, and `host_to_world_bridge`.
  - Conflict: no
  - Resolution (explicit): ADR-0042 remains the semantic owner. ADR-0043 only owns additive policy keys, ordered tuple-axis evaluation, and operator-facing denial semantics.
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces: trace vocabulary, correlation keys, and additive record widening for policy evaluation.
  - Conflict: no
  - Resolution (explicit): ADR-0028 keeps the base trace envelope and correlation model. ADR-0043 reuses that envelope and adds tuple-policy publication through the selected `identity_tuple` and `placement_posture` field family rather than inventing a parallel trace schema.
- ADR: `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - Overlap surfaces: separation between `backend_id` and tuple fields on agent-event records.
  - Conflict: no
  - Resolution (explicit): keep `backend_id` as adapter identity and keep tuple fields additive and optional. ADR-0043 denial and trace text must not repurpose `backend_id` as a tuple surrogate.
- ADR: `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
  - Overlap surfaces: tuple-compatible publication rules, omission rules for `provider` and `auth_authority`, and trace/event vocabulary reuse.
  - Conflict: no
  - Resolution (explicit): ADR-0045 stays on toolbox and pure-agent flows. ADR-0043 uses the same tuple field family and preserves the existing omission rule for flows where `provider` or `auth_authority` is unresolved or inapplicable.
- ADR: `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - Overlap surfaces: backend selection, runtime generation, tuple-axis policy deferral text, and backend-id boundary wording.
  - Conflict: yes
  - Resolution (explicit): keep ADR-0046 on backend selection, adapter lookup, and runtime generation only. Edit ADR-0046 so it stops describing tuple-axis policy as not yet implemented, and keep all `llm.constraints.*` ownership, deny behavior, and inspection-surface work in ADR-0043.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
  - Overlap surfaces: tuple vocabulary, top-level `identity_tuple` and `placement_posture` publication, trace/status field placement, and platform parity wording.
  - Conflict: yes
  - Resolution (explicit): the LAITDP pack remains the owner of tuple semantics and the canonical field family, while this feature owns tuple-axis policy keys and denial semantics. Edit the local telemetry and contract specs so they reuse LAITDP field names and do not redefine tuple semantics.
- Planning Pack: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/`
  - Overlap surfaces: gateway runtime generation, backend-id selection, and stale statements about tuple-policy implementation state.
  - Conflict: yes
  - Resolution (explicit): keep the non-overlap boundary already described in that pack’s impact map, then clean up stale implementation-state wording so it no longer contradicts the current repository.
- Planning Pack: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/`
  - Overlap surfaces: stable backend ids, adapter capability gating, and the boundary against tuple-axis policy keys.
  - Conflict: no
  - Resolution (explicit): keep that pack on backend-id contracts only. ADR-0043 remains the only owner of `llm.constraints.*`.

## Follow-ups (explicit)

- Decision Register entries required:
  - `DR-ITPS-01` — lock tuple-policy telemetry to the existing `identity_tuple` and `placement_posture` field family and reject a parallel trace-only tuple schema.
  - `DR-ITPS-02` — lock the operator inspection surface so `substrate policy current show --explain` is the authoritative merged view for `llm.constraints.*`, while `substrate config` remains the config-root inspection surface.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md` — remove the row that assigns tuple-policy visibility to `substrate config show --explain`, and align the manual testing surface with the actual policy-view contract.
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md` — pin the exact rendered deny text, exit-code mapping, and explain-surface wording for tuple-axis mismatches.
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md` — pin the exact allow and deny field set, deny-reason projection, and redaction rules using the selected tuple field family.
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` — replace stale “new surface” framing, remove config and policy conflation, and replace ambiguous modal language with direct contract text.
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md` — remove stale statements that describe tuple-axis policy as unimplemented.
