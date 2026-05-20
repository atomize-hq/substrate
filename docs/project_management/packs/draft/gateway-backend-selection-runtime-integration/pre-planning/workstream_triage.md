# gateway-backend-selection-runtime-integration — workstream triage

## Scope

This triage proposes downstream FSE planning workstreams for the draft seam skeleton in `pre-planning/minimal_spec_draft.md`.

- Seam prefix source: `GBSRI`
- Draft seam order source:
  - `GBSRI-01` — `backend-selection-and-policy`
  - `GBSRI-02` — `runtime-realization-and-artifacts`
  - `GBSRI-03` — `parity-validation-and-rollout`
- Recommended workstream count: 3
- Planning pressure: high

<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->
```json
{
  "index_version": 1,
  "seam_prefix": "GBSRI",
  "recommended_workstream_order": [
    "GBSRI-FWS-contract_policy_surface",
    "GBSRI-FWS-runtime_realization_artifacts",
    "GBSRI-FWS-parity_validation_rollout"
  ],
  "draft_seam_order": [
    "GBSRI-01",
    "GBSRI-02",
    "GBSRI-03"
  ],
  "workstreams": [
    {
      "id": "GBSRI-FWS-contract_policy_surface",
      "role": "Freeze backend-selection, policy, exit-bucket, and env/auth boundary decisions that every later seam consumes.",
      "depends_on": [],
      "assumes": [
        "Existing operator command family and status schema owners remain unchanged.",
        "Config and policy authority stays with ADR-0027 surfaces."
      ],
      "owns": [
        "seam-planning/backend-selection-and-policy.md",
        "contract.md",
        "policy-spec.md",
        "env-vars-spec.md",
        "selection-policy decision table",
        "exit-bucket mapping",
        "auth-source precedence rule"
      ],
      "outcomes": [
        "One fixed realization order from config and policy into inventory validation.",
        "One fixed classification boundary for invalid integration, dependency unavailable, policy denial, and transient failure.",
        "One fixed trusted-input boundary that keeps gateway-local persistence and mutation outside authorization."
      ]
    },
    {
      "id": "GBSRI-FWS-runtime_realization_artifacts",
      "role": "Freeze the adapter-driven lifecycle after selection succeeds, including binding lookup, capability gating, auth validation, config render, launch, readiness, restart, and artifact semantics.",
      "depends_on": [
        "GBSRI-FWS-contract_policy_surface"
      ],
      "assumes": [
        "General adapter protocol and schema contracts remain external source-of-truth docs.",
        "One selected backend resolves to one integrated adapter binding."
      ],
      "owns": [
        "seam-planning/runtime-realization-and-artifacts.md",
        "gateway-runtime-adapter-protocol-spec.md",
        "gateway-runtime-adapter-schema-spec.md",
        "filesystem-semantics-spec.md",
        "adapter-binding metadata surface",
        "auth handoff payload surface",
        "runtime artifact path and permission surface"
      ],
      "outcomes": [
        "One fixed classification for missing integrated adapter bindings.",
        "One fixed classification for missing auth handoff material after policy permits the read path.",
        "One fixed ordering for capability gating, auth resolution, config render, launch, readiness, and restart.",
        "One fixed artifact-root, manifest, config, and managed-log rule set."
      ]
    },
    {
      "id": "GBSRI-FWS-parity_validation_rollout",
      "role": "Freeze cross-platform proof, compatibility posture, and validation evidence for the selected-backend lifecycle.",
      "depends_on": [
        "GBSRI-FWS-contract_policy_surface",
        "GBSRI-FWS-runtime_realization_artifacts"
      ],
      "assumes": [
        "Linux, macOS, and Windows stay inside one operator-facing command family.",
        "The `cli:codex` baseline remains the regression floor for compatibility."
      ],
      "owns": [
        "seam-planning/parity-validation-and-rollout.md",
        "platform-parity-spec.md",
        "compatibility-spec.md",
        "manual_testing_playbook.md",
        "smoke/linux-smoke.sh",
        "smoke/macos-smoke.sh",
        "smoke/windows-smoke.ps1",
        "cross-platform validation matrix"
      ],
      "outcomes": [
        "One fixed first additional integrated backend baseline for parity and fixtures.",
        "One explicit validation matrix covering supported, blocked, invalid, missing-inventory, missing-binding, and missing-auth cases.",
        "One explicit `cli:codex` regression promise and one explicit no-fallback rule for unsupported backends."
      ]
    }
  ]
}
```
<!-- PM_FSE_WORKSTREAM_INDEX:END -->

## Proposed Downstream Planning Workstreams

### GBSRI-FWS-contract_policy_surface — Backend Selection And Policy Surface
- Goal:
  - Freeze the selected-backend realization rules, exit-bucket mapping, policy-gate order, and env/auth sourcing boundaries.
- Owned surfaces:
  - `seam-planning/backend-selection-and-policy.md`
  - `contract.md`
  - `policy-spec.md`
  - `env-vars-spec.md`
  - selection-policy decision table
  - exit-bucket mapping
  - auth-source precedence rule
- Dependencies:
  - none
- Expected downstream deliverables:
  - one fixed selection and policy evaluation order
  - one fixed exit-bucket mapping for invalid integration, dependency unavailable, policy denial, and transient failure
  - one fixed precedence rule between env-based auth material and host credential file reads
  - one fixed trusted-input boundary that keeps gateway-local persistence and mutation outside authorization

### GBSRI-FWS-runtime_realization_artifacts — Runtime Realization And Artifacts
- Goal:
  - Freeze the adapter-driven lifecycle after selection succeeds, including binding lookup, capability gating, auth handoff validation, runtime config rendering, artifact management, launch, readiness, and restart ordering.
- Owned surfaces:
  - `seam-planning/runtime-realization-and-artifacts.md`
  - `gateway-runtime-adapter-protocol-spec.md`
  - `gateway-runtime-adapter-schema-spec.md`
  - `filesystem-semantics-spec.md`
  - adapter-binding metadata surface
  - auth handoff payload surface
  - runtime artifact path and permission surface
- Dependencies:
  - `GBSRI-FWS-contract_policy_surface`
- Expected downstream deliverables:
  - one fixed classification for missing integrated adapter bindings
  - one fixed classification for missing auth handoff material after policy permits the read path
  - one fixed ordering for capability gating, auth resolution, config render, launch, readiness, and restart
  - one fixed artifact-root, manifest, config, and managed-log rule set

### GBSRI-FWS-parity_validation_rollout — Parity Validation And Rollout
- Goal:
  - Freeze cross-platform proof, compatibility posture, and validation evidence for the selected-backend lifecycle.
- Owned surfaces:
  - `seam-planning/parity-validation-and-rollout.md`
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
  - cross-platform validation matrix
- Dependencies:
  - `GBSRI-FWS-contract_policy_surface`
  - `GBSRI-FWS-runtime_realization_artifacts`
- Expected downstream deliverables:
  - one fixed first additional integrated backend baseline for parity and fixtures
  - one explicit validation matrix covering supported, blocked, invalid, missing-inventory, missing-binding, and missing-auth cases
  - one explicit `cli:codex` regression promise
  - one explicit no-fallback rule for unsupported backends

## Sequencing And Gates

### Hard Ordering Constraints
1. Start `GBSRI-FWS-contract_policy_surface` first.
2. Start `GBSRI-FWS-runtime_realization_artifacts` after `GBSRI-FWS-contract_policy_surface` freezes:
   - exit-bucket mapping
   - auth-source precedence
   - trusted-input boundaries
3. Start `GBSRI-FWS-parity_validation_rollout` after `GBSRI-FWS-runtime_realization_artifacts` freezes:
   - first additional integrated backend baseline
   - runtime ordering through readiness and restart
   - artifact-path and permission semantics

### CI Checkpoint Implications
- `CP1` groups `GBSRI-01` and `GBSRI-02`.
- `CP1` closes when the contract-policy seam and runtime-realization seam cite one aligned selection truth, one aligned failure taxonomy, and one aligned runtime lifecycle order.
- `CP2` isolates `GBSRI-03`.
- `CP2` closes when Linux, macOS, and Windows parity proof plus compatibility evidence line up under one validation matrix.

## Draft Seam Skeleton Recommendations

No draft seam skeleton change is recommended.

Rationale:
- The three-seam backbone in `minimal_spec_draft.md` matches the downstream seam-planning files declared in `spec_manifest.md`.
- `ci_checkpoint_plan.md` already groups `GBSRI-01` and `GBSRI-02` under `CP1` and isolates `GBSRI-03` under `CP2`.
- The critical path is driven by unresolved shared decisions rather than seam-count mismatch.

## Risks And Unknowns

- Pin the first non-`cli:codex` integrated backend id before runtime and parity planning freeze examples, fixtures, and smoke assertions.
- Pin the missing integrated adapter binding classification before runtime planning freezes failure-shape vocabulary.
- Pin the missing auth handoff material classification before policy and runtime planning freeze denial versus dependency boundaries.
- Pin auth precedence between env material and host credential files before env-var and protocol planning freeze examples.
- Pin backend inventory roots and filename rules before filesystem semantics and parity validation freeze discoverability wording.

## High-Churn Boundaries

- Selection and policy versus runtime realization:
  - exit-bucket ownership
  - trusted-input boundary
  - auth-source precedence
- Runtime realization versus parity validation:
  - additional backend baseline
  - readiness proof
  - unsupported-backend classification
- Filesystem semantics versus parity validation:
  - inventory discoverability paths
  - generated artifact paths
  - inspectability evidence

## Evidence Links

- Stable sentinels:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/logs/impact-map/last_message.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/logs/CI-checkpoint/last_message.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/logs/workstream-triage/planning_pressure_assessment.md`
- Canonical artifacts relied on:
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/ci_checkpoint_plan.md`

## Follow-ups

- Align downstream seam-planning drafts to the workstream ids in this file.
- Confirm the first additional integrated backend id before parity fixtures or smoke assertions are authored.
- Confirm the missing-binding and missing-auth classifications before protocol or schema drafting starts.
- Confirm backend inventory roots and filename rules before filesystem semantics or operator discoverability wording is authored.
