**Warning: Pre-Planning Only. This draft exists only to align full-planning inputs and will be deleted or retired during full planning.**

# Minimal spec draft

## Scope + authority

This draft defines only the cross-cutting alignment backbone for ADR-0043:
- default precedence for the tuple-axis policy surface
- source-of-truth inspection surface and file roots
- failure posture and security invariants
- exit-code posture
- shared naming and ordering constraints that every downstream slice spec must reuse

This draft does not define:
- slice-specific implementation behavior
- detailed YAML schema tables
- trace field inventories
- task breakdown, worktree wiring, or execution sequencing beyond the draft slice skeleton

Authoritative inputs for this draft:
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`

## Defaults + precedence

Tuple-axis policy constraints remain an additive extension of the existing ADR-0027 policy surface under:
- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

Source-of-truth files and views:
- Global policy patch: `$SUBSTRATE_HOME/policy.yaml`
- Workspace policy patch: `<workspace_root>/.substrate/policy.yaml`
- Authoritative merged inspection view for tuple-axis constraints: `substrate policy current show --explain`

Effective precedence for tuple-axis policy keys:
1. Workspace policy patch
2. Global policy patch
3. Built-in defaults

Existing broader config precedence remains unchanged from ADR-0027:
1. CLI flags for world-related config surfaces only
2. Workspace config patch
3. `SUBSTRATE_OVERRIDE_*` environment overrides when no enabled workspace is active
4. Global config patch
5. Built-in defaults

This feature adds no tuple-axis CLI flags and no tuple-axis environment variables. Full planning must keep the tuple-axis surface on the policy-file ladder above rather than introducing a parallel flag or env-var path.

## Failure posture + invariants

Failure posture:
- Backend allowlists remain deny-by-default.
- Tuple-axis constraints act as narrowing gates on top of backend allowlists.
- Empty tuple-axis lists mean unconstrained on that axis.
- Non-empty tuple-axis lists require an exact match on that axis.
- `llm.fail_closed.routing=true` preserves fail-closed routing when the required world boundary is unavailable.
- Schema-invalid tuple-axis values fail as user/config error rather than degrading silently.

Security and redaction invariants:
- No secrets enter policy YAML.
- `backend_id` remains an adapter/runtime selector and does not become a surrogate for `router`, `provider`, `protocol`, or `auth_authority`.
- `host_to_world_bridge` remains transport-only and does not become a second control plane.
- Host credential reads remain gated by the existing ADR-0027 policy surfaces.
- Allow and deny publication for tuple-aware policy evaluation must reuse the existing tuple vocabulary and keep secret-adjacent data redacted or omitted.

## Exit-code posture

Exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

This work requires no new exit codes in the alignment backbone.
- `0` remains success / no-op by contract.
- `2` covers tuple-axis schema or usage errors.
- `5` covers tuple-axis policy denials and protected-path or safety violations when applicable.

Full planning must keep any feature-local wording aligned to the canonical taxonomy unless an explicit override is recorded in the ADR and repeated in the feature contract.

## Cross-cutting seams / constraints

Shared constraints every downstream spec must align on:
- The tuple-axis keys stay under `llm.constraints.*`. No second config system, file family, or path root enters scope.
- The semantic owner for `router`, `provider`, `protocol`, `auth_authority`, `identity_tuple`, `placement_posture`, and `host_to_world_bridge` remains ADR-0042.
- The trace envelope and correlation vocabulary remain anchored to ADR-0028.
- The operator-facing merged view for tuple-axis constraints is `substrate policy current show --explain`. Full planning must resolve any remaining `config show --explain` references to this policy-view contract.
- Naming stays normalized:
  - `router`, `provider`, and `auth_authority` values use lowercase snake_case ids.
  - `protocol` values use lowercase dotted ids.
- `client` stays outside the standalone policy-key surface in v1.
- Ordering of tuple-axis evaluation remains stable:
  1. backend allowlist gate
  2. tuple-axis narrowing checks
  3. fail-closed routing posture
  4. host credential-read gates
  5. network and boundary enforcement gates
- Telemetry and operator messaging must reuse one tuple vocabulary across allow paths, deny paths, gateway status, and trace publication.

## Follow-ups for full planning

- Update `spec_manifest.md` so the authoritative merged inspection surface for `llm.constraints.*` is `substrate policy current show --explain`.
- Pin the exact deny-reason wording and explanation payload for router, provider, protocol, and auth-authority mismatches.
- Pin the exact telemetry field set for tuple-aware allow and deny records and confirm reuse of the existing `identity_tuple` and `placement_posture` family.
- Pin the exact schema-invalid examples and rejection grammar for each tuple-axis key.
- Pin the exact compatibility text for operators who do not set any `llm.constraints.*` entries.
- Reconcile ADR-0043 wording with the implemented ADR-0027 pack so the additive tuple-axis surface is described as an extension of the existing policy surface rather than a separate config root.

## Draft slice skeleton (pre-planning only)

Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `ITPS`

CI-checkpoint must prefer this slice list when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`.

Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` and must leave this draft unchanged.

### `ITPS0`
- `slice_id`: `ITPS0`
- `name`: Publish tuple-axis contract and schema surface
- `intent`: Stabilize the additive policy-key surface and operator-facing inspection contract for `llm.constraints.*`. Lock the authoritative file roots, precedence, naming grammar, and merged-view rules that downstream specs inherit.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS0/ITPS0-spec.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/CONFIGURATION.md`
  - `docs/USAGE.md`

### `ITPS1`
- `slice_id`: `ITPS1`
- `name`: Lock policy evaluation, telemetry, and validation closure
- `intent`: Stabilize ordered evaluation, deny posture, compatibility guardrails, and tuple-aware telemetry publication. Lock the validation surface that proves the tuple-axis policy contract without changing tuple semantics owned elsewhere.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md`
  - `docs/TRACE.md`
  - `crates/shell/src/execution/policy_cmd.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `crates/broker/src/policy/tests.rs`
  - `crates/broker/src/tests.rs`
  - `crates/trace/src/span.rs`
  - `crates/trace/src/tests.rs`
