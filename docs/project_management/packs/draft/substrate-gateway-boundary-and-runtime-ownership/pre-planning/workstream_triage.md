# substrate-gateway-boundary-and-runtime-ownership — workstream triage

## Canonical inputs used

- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`

## Evidence links

- Stable sentinels:
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/CI-checkpoint/last_message.md`
- Lift evidence:
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/workstream-triage/pm_lift_pack.txt`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/logs/workstream-triage/pm_lift_pack.json`

## Lift and boundary decision

- Strict-pack status: `tasks.json` uses `meta.slice_spec_version = 2`, so pack-derived lift is the authoritative sizing signal for triage.
- Pack lift: `lift_score=85`, `estimated_slices=8`, `confidence=low`.
- Touch-set counts: `create=13`, `edit=15`, `crates_touched=4`.
- Split triggers: `split_required:estimated_slices>3`, `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`, `likely_split:crates_touched>2`.
- Accepted boundary decision: replace the draft one-slice skeleton with a 5-slice planning spine.
- Boundary rationale: isolate the highest-churn seams into separate planning lanes for contract, machine schema, policy, typed world-service transport plus parity, and final docs-validation alignment.
- Intake-lift note: `make pm-lift-intake` failed for `ADR-0040` because the embedded lift vector sets `risk.unknowns_high=false`; the intake schema requires `integer|null`. This triage uses pack-derived lift only.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "SGBRO",
  "accepted_slice_order": [
    "SGBRO0",
    "SGBRO1",
    "SGBRO2",
    "SGBRO3",
    "SGBRO4"
  ],
  "draft_slice_order": [
    "SGBRO0"
  ],
  "pws": [
    {
      "id": "SGBRO-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0040 remains the feature boundary authority and ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042 remain external owners for their existing surfaces."
      ],
      "owns": [
        "pre-planning/spec_manifest.md",
        "contract.md"
      ]
    },
    {
      "id": "SGBRO-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "SGBRO-PWS-contract"
      ],
      "assumes": [
        "ADR-0042 continues to own additive identity-tuple metadata outside the `client_wiring.*` family."
      ],
      "owns": [
        "gateway-status-schema-spec.md",
        "policy-spec.md"
      ]
    },
    {
      "id": "SGBRO-PWS-world_service_profile",
      "role": "implementation",
      "depends_on": [
        "SGBRO-PWS-contract",
        "SGBRO-PWS-schema_inventory"
      ],
      "assumes": [
        "Impact-map option A stays selected: Substrate exposes a typed world-service lifecycle and status surface instead of shell-side exec probing."
      ],
      "owns": [
        "platform-parity-spec.md"
      ]
    },
    {
      "id": "SGBRO-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "SGBRO-PWS-contract",
        "SGBRO-PWS-schema_inventory",
        "SGBRO-PWS-world_service_profile"
      ],
      "assumes": [
        "The manual playbook validates one-owner-per-surface coverage and external docs stay consumers of the feature-local authority set."
      ],
      "owns": [
        "manual_testing_playbook.md"
      ]
    },
    {
      "id": "SGBRO-PWS-implementation_seams",
      "role": "slice_spec",
      "depends_on": [
        "SGBRO-PWS-contract",
        "SGBRO-PWS-schema_inventory",
        "SGBRO-PWS-world_service_profile"
      ],
      "assumes": [
        "The accepted slice order keeps one dominant behavior delta per slice and reserves docs-only closeout work for the final slice."
      ],
      "owns": [
        "slices/SGBRO0/SGBRO0-spec.md",
        "slices/SGBRO1/SGBRO1-spec.md",
        "slices/SGBRO2/SGBRO2-spec.md",
        "slices/SGBRO3/SGBRO3-spec.md",
        "slices/SGBRO4/SGBRO4-spec.md"
      ]
    },
    {
      "id": "SGBRO-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "SGBRO-PWS-contract",
        "SGBRO-PWS-schema_inventory",
        "SGBRO-PWS-world_service_profile",
        "SGBRO-PWS-docs_validation",
        "SGBRO-PWS-implementation_seams"
      ],
      "assumes": [
        "Checkpoint cadence stays one end-of-feature CI checkpoint after the final accepted slice unless `pre-planning/ci_checkpoint_plan.md` changes first."
      ],
      "owns": [
        "pre-planning/ci_checkpoint_plan.md",
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/SGBRO0/kickoff_prompts/",
        "slices/SGBRO1/kickoff_prompts/",
        "slices/SGBRO2/kickoff_prompts/",
        "slices/SGBRO3/kickoff_prompts/",
        "slices/SGBRO4/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams

### SGBRO-PWS-contract — contract and boundary lock
- Goal: lock the operator-facing command family, ownership split, stable wiring env semantics, and exit-code boundaries before any slice spec or task graph mirrors them.
- Owned surfaces: `pre-planning/spec_manifest.md`, `contract.md`.
- Dependencies: none.
- Proposed slices/triads: `SGBRO0-{code,test,integ}` for command contract, ownership table, and exit-code acceptance criteria.

### SGBRO-PWS-schema_inventory — status schema and policy inventory
- Goal: pin the `status --json` envelope, `client_wiring.*` rules, fail-closed policy flow, and secret-delivery boundaries as singular planning inputs.
- Owned surfaces: `gateway-status-schema-spec.md`, `policy-spec.md`.
- Dependencies: `SGBRO-PWS-contract`.
- Proposed slices/triads: `SGBRO1-{code,test,integ}` for status schema and `SGBRO2-{code,test,integ}` for policy-evaluation and trust-boundary rules.

### SGBRO-PWS-world_service_profile — typed runtime and parity seam
- Goal: lock the typed world-service lifecycle/status path and the Linux/macOS/Windows guarantees that the CLI and status schema consume.
- Owned surfaces: `platform-parity-spec.md`.
- Dependencies: `SGBRO-PWS-contract`, `SGBRO-PWS-schema_inventory`.
- Proposed slices/triads: `SGBRO3-{code,test,integ}` for world-service transport, shell integration, and parity evidence.

### SGBRO-PWS-docs_validation — manual validation and external doc alignment
- Goal: build the deterministic review flow that proves one owner per surface and lines up the feature-local authority set with `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md`.
- Owned surfaces: `manual_testing_playbook.md`.
- Dependencies: `SGBRO-PWS-contract`, `SGBRO-PWS-schema_inventory`, `SGBRO-PWS-world_service_profile`.
- Proposed slices/triads: `SGBRO4-{code,test,integ}` acceptance coverage for manual validation, cross-doc assertions, and quality-gate evidence.

### SGBRO-PWS-implementation_seams — execution-ready slice specs
- Goal: convert the accepted 5-slice spine into execution-ready slice specs with disjoint AC ids and narrow behavior deltas.
- Owned surfaces: `slices/SGBRO0/SGBRO0-spec.md`, `slices/SGBRO1/SGBRO1-spec.md`, `slices/SGBRO2/SGBRO2-spec.md`, `slices/SGBRO3/SGBRO3-spec.md`, `slices/SGBRO4/SGBRO4-spec.md`.
- Dependencies: `SGBRO-PWS-contract`, `SGBRO-PWS-schema_inventory`, `SGBRO-PWS-world_service_profile`.
- Proposed slices/triads: `SGBRO0-{code,test,integ}`, `SGBRO1-{code,test,integ}`, `SGBRO2-{code,test,integ}`, `SGBRO3-{code,test,integ}`, `SGBRO4-{code,test,integ}`.

### SGBRO-PWS-tasks_checkpoints — single writer for tasks and gates
- Goal: translate the accepted slice order into `plan.md`, `tasks.json`, kickoff prompts, session logging, and the final checkpoint wiring.
- Owned surfaces: `pre-planning/ci_checkpoint_plan.md`, `plan.md`, `tasks.json`, `session_log.md`, `quality_gate_report.md`, `kickoff_prompts/`, `slices/SGBRO0/kickoff_prompts/`, `slices/SGBRO1/kickoff_prompts/`, `slices/SGBRO2/kickoff_prompts/`, `slices/SGBRO3/kickoff_prompts/`, `slices/SGBRO4/kickoff_prompts/`.
- Dependencies: `SGBRO-PWS-contract`, `SGBRO-PWS-schema_inventory`, `SGBRO-PWS-world_service_profile`, `SGBRO-PWS-docs_validation`, `SGBRO-PWS-implementation_seams`.
- Proposed slices/triads: add `SGBRO0` through `SGBRO4` triads plus `CP1-ci-checkpoint` after the final slice.

## Sequencing and gates

- Hard ordering: `SGBRO-PWS-contract` lands first.
- Hard ordering: `SGBRO-PWS-schema_inventory` lands after `SGBRO-PWS-contract`.
- Hard ordering: `SGBRO-PWS-world_service_profile` lands after `SGBRO-PWS-contract` and `SGBRO-PWS-schema_inventory`.
- Hard ordering: `SGBRO-PWS-docs_validation` and `SGBRO-PWS-implementation_seams` land after contract, schema, and typed-runtime seams are pinned.
- Hard ordering: `SGBRO-PWS-tasks_checkpoints` lands last.
- Gate A: lock the command contract, ownership table, and exit taxonomy before slice specs or checkpoint wiring.
- Gate B: lock the `status --json` envelope and policy decision flow before any world-service endpoint or shell status task references them.
- Gate C: lock typed world-service transport and parity guarantees before final docs-validation or task-graph wiring.
- CI checkpoint implication: replace the draft single-slice machine list with `["SGBRO0","SGBRO1","SGBRO2","SGBRO3","SGBRO4"]`.
- CI checkpoint implication: keep one end-of-feature checkpoint and move the boundary to `SGBRO4`.
- CI checkpoint implication: `tasks.json` must set `meta.checkpoint_boundaries=["SGBRO4"]` and wire `CP1-ci-checkpoint` after `SGBRO4-{code,test,integ}`.

## Risks and unknowns

- High-churn seam: `contract.md`, `crates/shell/src/builtins/world_gateway.rs`, `docs/USAGE.md`, and `docs/WORLD.md` all need the same command spelling and absent-state language.
- High-churn seam: `gateway-status-schema-spec.md`, `crates/transport-api-types/src/lib.rs`, `crates/transport-api-client/src/lib.rs`, and `crates/world-service/src/handlers.rs` all need the same status envelope and `client_wiring.*` semantics.
- High-churn seam: `policy-spec.md`, `contract.md`, and world-placement behavior in `crates/shell` and `crates/world-service` need one fail-closed decision taxonomy for exit codes `2`, `3`, `4`, and `5`.
- High-churn seam: `platform-parity-spec.md`, `docs/WORLD.md`, and later provisioning/runtime packs need one explicit boundary for what this pack owns versus what a later runtime pack owns.
- Unknown: ADR-0040 intake lift is invalid until `risk.unknowns_high` uses `integer|null`; intake-derived sizing stays unavailable until that field is fixed.

## Slice skeleton recommendations

- `SPLIT` draft `SGBRO0` into `SGBRO0`, `SGBRO1`, `SGBRO2`, `SGBRO3`, `SGBRO4`.
- `SGBRO0`: contract and boundary ownership lock for `sync`, `status`, `restart`, stable wiring env names, and exit-code wording.
- `SGBRO1`: `status --json` envelope and `client_wiring.*` field family.
- `SGBRO2`: policy-evaluation flow, fail-closed placement rules, and host-secret-delivery trust boundary.
- `SGBRO3`: typed world-service lifecycle/status transport, shell integration seam, and Linux/macOS/Windows parity guarantees.
- `SGBRO4`: manual validation playbook, external doc alignment, and final quality-gate evidence.
- `RETAIN` slice prefix `SGBRO`.
- `ACCEPTED SLICE ORDER`: `SGBRO0`, `SGBRO1`, `SGBRO2`, `SGBRO3`, `SGBRO4`.

## Follow-ups

- Update `pre-planning/spec_manifest.md` during full planning so the canonical slice-id section and slice-spec set match the accepted 5-slice order.
- Update `pre-planning/ci_checkpoint_plan.md` during full planning so the machine-readable slice list and checkpoint boundary point at `SGBRO4`.
- Fix the ADR-0040 lift vector field `risk.unknowns_high` so `make pm-lift-intake` returns valid intake evidence on the next pass.
- Map `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md` to `SGBRO3` and `SGBRO4` in full planning so the repo-level docs move with the same acceptance criteria.
