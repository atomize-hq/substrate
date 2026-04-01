# world-disabled-reason-attribution — workstream triage

## Lift and boundary decision

- Strict-pack status:
  - `tasks.json` uses `meta.slice_spec_version = 2`, so pack-derived lift is the authoritative sizing input once the touch set is locked.
- Boundary decision:
  - accept a **3-slice** planning skeleton for full planning
  - keep the helper seam, replay-surface seam, and validation-doc seam separate so the final checkpoint lands only after docs and telemetry are aligned

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "WDRA",
  "accepted_slice_order": [
    "WDRA0",
    "WDRA1",
    "WDRA2"
  ],
  "draft_slice_order": [
    "WDRA0",
    "WDRA1",
    "WDRA2"
  ],
  "pws": [
    {
      "id": "WDRA-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0037 remains the source of truth for effective world-disable attribution semantics."
      ],
      "owns": [
        "pre-planning/spec_manifest.md",
        "contract.md",
        "decision_register.md",
        "telemetry-spec.md",
        "platform-parity-spec.md"
      ]
    },
    {
      "id": "WDRA-PWS-implementation_seams",
      "role": "slice_spec",
      "depends_on": [
        "WDRA-PWS-contract"
      ],
      "assumes": [
        "The accepted split keeps helper placement separate from replay-surface wording and from final validation-doc updates."
      ],
      "owns": [
        "slices/WDRA0/WDRA0-spec.md",
        "slices/WDRA1/WDRA1-spec.md",
        "slices/WDRA2/WDRA2-spec.md"
      ]
    },
    {
      "id": "WDRA-PWS-tests_ci",
      "role": "docs_validation",
      "depends_on": [
        "WDRA-PWS-contract",
        "WDRA-PWS-implementation_seams"
      ],
      "assumes": [
        "Replay validation remains centered on crates/shell/tests/replay_world.rs and the feature-local smoke wrappers."
      ],
      "owns": [
        "manual_testing_playbook.md",
        "smoke/linux-smoke.sh",
        "smoke/macos-smoke.sh",
        "smoke/windows-smoke.ps1"
      ]
    },
    {
      "id": "WDRA-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "WDRA-PWS-contract",
        "WDRA-PWS-implementation_seams",
        "WDRA-PWS-tests_ci"
      ],
      "assumes": [
        "CI remains a single end-of-feature checkpoint after WDRA2 unless the checkpoint plan changes first."
      ],
      "owns": [
        "pre-planning/ci_checkpoint_plan.md",
        "pre-planning/alignment_report.md",
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/WDRA0/kickoff_prompts/",
        "slices/WDRA1/kickoff_prompts/",
        "slices/WDRA2/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams

### `WDRA-PWS-contract` — contract and telemetry lock
- Goal:
  - lock the replay-visible copy contract, structured telemetry fields, and platform guarantees before slice specs and tasks mirror them
- Owned surfaces:
  - `pre-planning/spec_manifest.md`
  - `contract.md`
  - `decision_register.md`
  - `telemetry-spec.md`
  - `platform-parity-spec.md`
- Dependencies:
  - none

### `WDRA-PWS-implementation_seams` — execution-ready slice specs
- Goal:
  - author one behavior delta per slice with deterministic AC ids
- Owned surfaces:
  - `slices/WDRA0/WDRA0-spec.md`
  - `slices/WDRA1/WDRA1-spec.md`
  - `slices/WDRA2/WDRA2-spec.md`
- Dependencies:
  - `WDRA-PWS-contract`

### `WDRA-PWS-tests_ci` — manual and smoke evidence lane
- Goal:
  - keep replay validation artifacts aligned with the same contract and AC ids
- Owned surfaces:
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
- Dependencies:
  - `WDRA-PWS-contract`
  - `WDRA-PWS-implementation_seams`

### `WDRA-PWS-tasks_checkpoints` — single writer for task graph and gates
- Goal:
  - translate the accepted slice order into execution-ready tasks, checkpoint wiring, prompts, and planning-gate artifacts
- Owned surfaces:
  - `pre-planning/ci_checkpoint_plan.md`
  - `pre-planning/alignment_report.md`
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - kickoff prompt directories
- Dependencies:
  - `WDRA-PWS-contract`
  - `WDRA-PWS-implementation_seams`
  - `WDRA-PWS-tests_ci`

## Sequencing and gates

- Hard ordering:
  - `WDRA-PWS-contract` lands first
  - `WDRA-PWS-implementation_seams` and `WDRA-PWS-tests_ci` land after the contract lock
  - `WDRA-PWS-tasks_checkpoints` lands last
- CI-checkpoint implication:
  - `pre-planning/ci_checkpoint_plan.md` defines one checkpoint after `WDRA2`
  - `tasks.json` must keep `meta.checkpoint_boundaries = ["WDRA2"]`

## Risks and unknowns

- High-churn seam:
  - `crates/shell/src/execution/routing/replay.rs` owns both human copy and replay selection bookkeeping; the helper seam must stay narrow.
- High-churn seam:
  - `crates/replay/src/replay/executor.rs` owns `replay_strategy`; additive fields must remain stable.
- High-churn seam:
  - `crates/shell/tests/replay_world.rs`, `docs/REPLAY.md`, and `docs/TRACE.md` all change in the final slice and must stay aligned.

## Slice skeleton recommendations

- `NO CHANGE` `WDRA0`
  - keep helper placement, redaction contract, and winning-layer mapping together
- `NO CHANGE` `WDRA1`
  - keep replay stderr copy and replay_strategy field wiring together
- `NO CHANGE` `WDRA2`
  - keep regression coverage and docs alignment as the checkpoint-boundary seam

## Evidence links
- `logs/spec-manifest/last_message.md`
- `logs/impact-map/last_message.md`
- `logs/min-spec-draft/last_message.md`
- `logs/CI-checkpoint/last_message.md`
