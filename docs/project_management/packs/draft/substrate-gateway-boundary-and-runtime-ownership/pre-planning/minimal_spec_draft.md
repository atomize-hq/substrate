**Pre-Planning Only: this draft is an alignment backbone for full planning and will be deleted or retired during full planning.**

# substrate-gateway-boundary-and-runtime-ownership minimal spec draft

## Scope and authority
- This draft defines pack-level defaults, precedence, ownership boundaries, failure posture, redaction posture, and the draft slice skeleton only.
- This draft does not define slice-specific behavior, detailed JSON schemas, implementation sequencing, runtime transport design, or task wiring.
- Full-planning source-of-truth ownership lands in these feature-local files:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
- External source-of-truth ownership remains with:
  - ADR-0027 and `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/*` for config and policy schema ownership
  - `docs/contracts/substrate-gateway-operator-contract.md` for the committed operator boundary wording
  - ADR-0017 for structured event routing and output-class separation
  - ADR-0028 for canonical trace vocabulary and correlation semantics
  - ADR-0041 for gateway runtime internals and backend-adapter identity
  - ADR-0042 for additive identity-tuple and placement-posture metadata outside `client_wiring.*`

## Defaults and precedence
- Command and output-mode selection starts at the CLI surface: `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json`.
- Gateway behavior and fail-closed posture continue to come from the existing ADR-0027 config and policy families. This feature adds no new config family and no new policy file family.
- File/path source of truth remains the existing config and policy contract surface:
  1. `$SUBSTRATE_HOME/config.yaml` and `<workspace_root>/.substrate/workspace.yaml`
  2. `$SUBSTRATE_HOME/policy.yaml` and `<workspace_root>/.substrate/policy.yaml`
- `status --json` is the authoritative Substrate-owned machine-readable wiring surface.
- `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL` remain stable non-secret wiring outputs. They do not override config or policy and they point to Substrate-managed gateway endpoints rather than upstream provider endpoints.

## Failure posture and invariants
- Fail closed when policy requires in-world execution. Do not fall back to a host-level gateway in that state.
- Keep Substrate as the owner of policy evaluation, world placement, lifecycle control, secret delivery, operator UX, and canonical tracing.
- Keep `substrate-gateway` as the owner of the in-world front door, provider normalization, planner and executor internals, and normalized event generation.
- Expose non-secret wiring only through the Substrate-owned status surface.
- Keep gateway-local config files, admin mutation surfaces, token persistence, and internal trace data outside the Substrate operator contract.
- Apply shared redaction posture to any operator-visible status or trace surface. Do not expose secret material in `status --json` or human-readable status output.

## Exit-code posture
- Exit code taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work requires no new exit codes.
- Gateway lifecycle and status behavior map onto the existing taxonomy:
  - `0`: success
  - `2`: invalid configuration, invalid policy, or invalid integration state
  - `3`: transient runtime failure
  - `4`: required gateway or world component unavailable
  - `5`: policy or safety failure

## Cross-cutting seams and constraints
- `contract.md` owns the operator-facing command family, the stable wiring env names, and the feature-local exit-code contract wording.
- `gateway-status-schema-spec.md` owns the `status --json` field list and the full `client_wiring.*` family.
- `policy-spec.md` owns evaluation rules over existing ADR-0027 inputs and the trust-boundary rules for secret delivery and fail-closed placement.
- `platform-parity-spec.md` owns Linux, macOS, and Windows parity guarantees for lifecycle visibility, placement, and status semantics.
- `manual_testing_playbook.md` owns the deterministic one-owner-per-surface review flow across ADR-0040, ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042.
- Do not restate external contract surfaces inside feature-local docs. Link back to the external owner when the feature consumes that surface.
- Keep `client_wiring.*` as the only status JSON field family locked by this pack for endpoint discovery. Record additive metadata outside that family under ADR-0042 ownership.
- Keep the single-slice baseline from `spec_manifest.md` unless full planning records a manifest update and the reason for the change.

## Follow-ups for full planning
- Confirm the final feature-local doc set against `pre-planning/impact_map.md` and keep the touch set aligned with the same ownership split.
- Pin the exact field list and absence semantics for `status --json`, including the hard boundary between `client_wiring.*` and ADR-0042 metadata.
- Pin the exact CLI text-output posture for `status`, `sync`, and `restart`, including the rule for abbreviated human-readable wiring versus full JSON wiring.
- Pin the exact world-agent/runtime transport decision for lifecycle and status operations and place the contract wording in the owning spec.
- ADR-0040 related-doc references now point at `packs/implemented/llm_and_agent_config_policy_surface/*`; keep them aligned if the external owner moves again.
- Decide whether provisioning evidence remains deferred to a later runtime pack or enters this pack as explicit planning scope.

## Draft slice skeleton (pre-planning only)
Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `SGBRO`

CI-checkpoint note: `pre-planning/ci_checkpoint_plan.md` must prefer this slice list when it populates the machine-readable slices list. Mechanical validation starts after slice tasks exist in `tasks.json`.

Workstream triage note: `pre-planning/workstream_triage.md` may propose edits to this slice skeleton as recommendations, but it does not edit this file.

### Slice list
- `slice_id`: `SGBRO0`
- `name`: Stabilize gateway boundary ownership authority set
- `intent`: Lock the feature-local authority set for the Substrate-owned command, status, policy-evaluation, parity, and validation seams. Preserve the external ownership boundaries defined by ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/slices/SGBRO0/SGBRO0-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`
