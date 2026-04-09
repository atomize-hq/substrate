**Warning: Pre-Planning Only. This draft exists only to align full planning and must be deleted or retired during full planning.**

# Minimal spec draft

## Scope + authority
- This draft defines pack-level defaults, precedence, delegated owners, failure posture, security invariants, cross-slice alignment rules, and a draft slice skeleton only.
- This draft does not define slice-specific behavior, detailed JSON field tables, implementation tasks, protocol mechanics, policy-schema redesign, telemetry-schema redesign, or backend-adapter internals.
- Authoritative inputs for this draft are:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`

## Defaults + precedence
- The pack-level command family is `substrate world gateway sync`, `substrate world gateway status`, and `substrate world gateway restart`.
- `substrate world gateway status --json` is the only stable JSON discovery surface frozen by this pre-planning draft.
- Precedence for surfaces owned by this feature is:
  1. CLI command selection defines whether the operator is invoking `sync`, `status`, or `restart`.
  2. Existing Substrate config and policy surfaces remain authoritative for enablement, backend selection, allowlists, and fail-closed gating.
  3. Stable non-secret wiring env vars are output/discovery values only and do not override CLI, config, or policy.
- This feature adds no new config file family, no new policy file family, and no gateway-local admin/config input.
- Source-of-truth paths for full planning are:
  - `pre-planning/spec_manifest.md` for selected feature-local docs and slice IDs.
  - `pre-planning/impact_map.md` for touch set and delegated dependencies.
  - ADR-0040 for the boundary ownership baseline.
  - The future feature-local owner docs selected by `spec_manifest.md`: `contract.md`, `runtime-boundary-spec.md`, `gateway-status-schema-spec.md`, `platform-parity-spec.md`, `manual_testing_playbook.md`, and `slices/SGBRO0/SGBRO0-spec.md`.

## Failure posture + invariants
- The boundary posture is fail-closed when policy requires in-world execution. Substrate does not fall back to a host-side gateway in that state.
- Substrate owns policy gating, world placement, lifecycle, host secret sourcing, host-to-world secret delivery ownership, operator UX, and canonical tracing.
- `substrate-gateway` owns the in-world front door, backend adapter/runtime internals, provider/planner/executor routing, and normalized event generation.
- Gateway-local config files, local persistence, and admin mutation surfaces are not trusted inputs for Substrate-managed operation.
- `client_wiring.*` values and the stable env vars `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` are non-secret discovery values only.
- This draft does not create a second trace vocabulary. Structured-event routing remains delegated to ADR-0017 and canonical trace vocabulary remains delegated to ADR-0028.
- This draft does not freeze `sync --json` or any gateway-local admin/status API.

## Exit-code posture
- Exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work does not require new exit codes.
- Full planning must keep the ADR-0040 mapping aligned to the canonical taxonomy:
  - `0`: success
  - `2`: invalid configuration, invalid policy, or invalid integration state
  - `3`: transient runtime failure
  - `4`: required gateway or world component unavailable
  - `5`: policy or safety failure

## Cross-cutting seams / constraints
- Command grammar remains `substrate world gateway ...`. Archived alternate grammar is historical-only.
- `status --json` is the only stable JSON schema surface in this pack baseline. Full planning must not promote `sync --json` into a second stable schema unless `spec_manifest.md` changes first.
- Config/policy file paths, precedence, defaults, and allowlist semantics remain delegated to the ADR-0027 owner docs referenced by ADR-0040 and `spec_manifest.md`. Feature-local docs must link to that owner surface instead of restating it.
- Backend-adapter/runtime mechanics remain delegated to ADR-0041. Feature-local docs must not invent a substitute mechanics spec.
- Linux, macOS, and Windows share one ownership boundary contract. Full planning must state any divergence explicitly in `platform-parity-spec.md`.
- Full planning must keep the pack docs-only. `impact_map.md` lists runtime code/doc surfaces as evidence-only downstream touch points, not execution scope for this pack.

## Follow-ups for full planning
- Update stale delegated ADR-0027 links from `docs/project_management/packs/active/llm_and_agent_config_policy_surface/...` to the current authoritative `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/...` paths.
- Decide whether `gateway-status-schema-spec.md` is versionless with additive-only growth or carries an explicit schema version field.
- Replace the provisional ADR-0041 delegation link with the exact downstream spec path after ADR-0041 completes spec determination.
- Decide whether human-readable `substrate world gateway status` output needs explicit contract text beyond the stable `status --json` surface.
- If full planning splits or merges `SGBRO0`, update `pre-planning/spec_manifest.md` before wiring `tasks.json`.

## Draft slice skeleton (pre-planning only)
Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

Baseline slice count from `pre-planning/spec_manifest.md`: 1

Slice prefix (draft): `SGBRO`

### `SGBRO0`
- `slice_id`: `SGBRO0`
- `name`: `Freeze boundary contract docs`
- `intent`: Stabilize the command grammar, delegated ownership boundaries, failure posture, and the minimum status-surface baseline for full planning. Keep runtime implementation work and detailed field schemas out of this slice.
- `likely touch surfaces`: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`, `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/runtime-boundary-spec.md`, `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`, `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`, `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`, `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`

CI-checkpoint should prefer this slice list when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`. Mechanical validation stays disabled until slice tasks exist in `tasks.json`.

Workstream triage may propose edits to this slice skeleton in `pre-planning/workstream_triage.md`, but it must not edit this draft directly.
