# persist-macos-host-os-install-state — workstream triage

## Scope and basis

- Phase: `phase_b`
- Candidate prefix: `PMHOS`
- Canonical inputs used:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
- Supporting log input used:
  - `logs/workstream-triage/planning_pressure_assessment.md`

## Workstream index

<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->
```json
{
  "index_version": 1,
  "candidate_prefix": "PMHOS",
  "recommended_candidate_order": [
    "PMHOS-FWS-contract_surface",
    "PMHOS-FWS-validation_doc_alignment"
  ],
  "draft_candidate_order": [
    "PMHOS-S1",
    "PMHOS-S2"
  ],
  "workstreams": [
    {
      "id": "PMHOS-FWS-contract_surface",
      "role": "Lock the additive install-state contract, schema, filesystem, and compatibility surfaces before validation planning finalizes.",
      "depends_on": [],
      "assumes": [
        "Hosted macOS install and hosted --no-world remain in scope.",
        "Runtime consumers for host_state.os.* stay outside this feature boundary.",
        "Linux and Windows behavior stays unchanged in this pack."
      ],
      "owns": [
        "contract.md",
        "install-state-schema-spec.md",
        "filesystem-semantics-spec.md",
        "compatibility-spec.md",
        "install_state.json writer vocabulary",
        "host_state.os schema surface"
      ],
      "outcomes": [
        "One canonical path and temp-path rule.",
        "One canonical field inventory for host_state.os.*.",
        "One canonical warning-only and rebuild posture.",
        "One canonical preservation rule for unknown keys and pre-existing Linux metadata."
      ]
    },
    {
      "id": "PMHOS-FWS-validation_doc_alignment",
      "role": "Lock parity wording, validation topology, manual testing, and operator-doc reconciliation after the contract surface locks.",
      "depends_on": [
        "PMHOS-FWS-contract_surface"
      ],
      "assumes": [
        "The hosted-only scope remains in force.",
        "docs/INSTALLATION.md is the only operator-facing document in the owned edit set.",
        "The existing harness split remains shared smoke plus macOS fixture coverage."
      ],
      "owns": [
        "platform-parity-spec.md",
        "manual_testing_playbook.md",
        "validation topology for tests/installers/install_state_smoke.sh",
        "validation topology for tests/mac/installer_parity_fixture.sh",
        "operator-doc reconciliation for docs/INSTALLATION.md"
      ],
      "outcomes": [
        "One parity matrix for macOS, Linux, and Windows.",
        "One validation split for shared writer assertions versus macOS branch assertions.",
        "One manual validation route for hosted install and hosted --no-world.",
        "One operator-doc update line for macOS write behavior versus Linux-only cleanup-state guidance."
      ]
    }
  ]
}
```
<!-- PM_FSE_WORKSTREAM_INDEX:END -->

## Proposed downstream planning workstreams

### PMHOS-FWS-contract_surface — Install-state surface lock

- Goal:
  - lock the vocabulary, contract rules, schema rules, filesystem semantics, and compatibility posture for the macOS `install_state.json` extension
- Owned surfaces:
  - `contract.md`
  - `install-state-schema-spec.md`
  - `filesystem-semantics-spec.md`
  - `compatibility-spec.md`
  - `install_state.json` writer vocabulary
  - `host_state.os` schema surface
- Dependencies:
  - none
- Expected downstream deliverables:
  - exact path and temp-path language
  - exact `host_state.os.family`, `product_version`, `build_version`, and `arch` field semantics
  - exact timestamp and rebuild posture
  - exact unknown-key and pre-existing Linux metadata preservation rules

### PMHOS-FWS-validation_doc_alignment — Platform parity, validation, and docs alignment

- Goal:
  - finalize validation topology, platform guarantees, manual testing evidence, and operator-doc reconciliation after the contract surface locks
- Owned surfaces:
  - `platform-parity-spec.md`
  - `manual_testing_playbook.md`
  - validation split for `tests/installers/install_state_smoke.sh`
  - validation split for `tests/mac/installer_parity_fixture.sh`
  - operator-doc reconciliation for `docs/INSTALLATION.md`
- Dependencies:
  - `PMHOS-FWS-contract_surface`
- Expected downstream deliverables:
  - exact macOS versus Linux versus Windows parity matrix
  - exact hosted install and hosted `--no-world` validation plan
  - exact split between shared writer-regression assertions and macOS branch assertions
  - exact docs update line for the macOS write delta and Linux-only cleanup-state guidance

## Sequencing and gates

### Hard ordering constraints

- `PMHOS-FWS-contract_surface` goes first.
- `PMHOS-FWS-validation_doc_alignment` starts after the contract surface locks the canonical metadata path, `host_state.os.*` field inventory, warning-only posture, and rebuild posture.
- Alignment reporting and final checkpoint proof consume both workstreams after the surface-lock pass.

### CI checkpoint implications

- `CP1` in `pre-planning/ci_checkpoint_plan.md` isolates the writer seam around draft candidate `PMHOS-S1`. That maps directly to `PMHOS-FWS-contract_surface`.
- `CP2` isolates the parity, no-change proof, and doc-alignment seam around draft candidate `PMHOS-S2`. That maps directly to `PMHOS-FWS-validation_doc_alignment`.
- The checkpoint structure supports the same two-workstream sequence with no extra split.

## Candidate skeleton recommendations

- `NO CHANGE`
- Keep the current draft candidate count at `2`.
- Keep the current draft candidate order:
  1. `PMHOS-S1`
  2. `PMHOS-S2`
- Keep the current draft candidate intent:
  - `PMHOS-S1` remains the install-state surface lock.
  - `PMHOS-S2` remains the macOS validation and doc-alignment seam.

## Risks and unknowns

- `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch` still need one exact partial-capture rule for successful installs with collection gaps.
- `created_at` versus `updated_at` still need one exact rewrite rule for first-write and rewrite paths.
- Unreadable prior files and unsupported schema versions still need one exact rebuild rule.
- `docs/INSTALLATION.md` still needs one exact wording line that separates macOS write behavior from Linux-only cleanup-state behavior.

## High-churn seams

- shared writer semantics in `scripts/substrate/install-substrate.sh`
- validation ownership split between `tests/installers/install_state_smoke.sh` and `tests/mac/installer_parity_fixture.sh`
- operator-doc wording for file creation versus Linux-only cleanup-state behavior

## Evidence links

### Stable sentinels

- `logs/spec-manifest/last_message.md`
- `logs/impact-map/last_message.md`
- `logs/min-spec-draft/last_message.md`
- `logs/CI-checkpoint/last_message.md`
- `logs/workstream-triage/planning_pressure_assessment.md`

### Canonical artifacts used

- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/ci_checkpoint_plan.md`

## Follow-ups

- Lock the exact `host_state.os.*` partial-capture semantics in downstream surface-lock planning.
- Lock the exact timestamp rewrite and rebuild rules in downstream surface-lock planning.
- Lock the final assertion split for the shared smoke harness and the macOS fixture harness in downstream validation planning.
