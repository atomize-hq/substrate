# persist-macos-host-os-install-state — CI checkpoint plan

This file defines where later multi-platform verification is expected to happen for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint intent during pre-planning.
- If you discover a mismatch between the plan and reality, update this plan first.
- This document remains advisory until downstream FSE planning or decomposition turns the checkpoint cadence into concrete execution behavior.

## Applicability and platform scope

Checkpoint planning applies to this pack because ADR-0039 changes shared `install_state.json` behavior across macOS producer flows while preserving Linux and Windows compatibility boundaries. The highest-cost verification runs after schema, writer-flow, validation, and doc-reconciliation surfaces align.

Likely platform scope for later verification:
- macOS is the behavior platform for hosted install and hosted `--no-world` producer flows.
- Linux is a CI-parity and no-change platform for `host_state.platform.*` preservation, additive merge behavior, and Linux-only cleanup semantics.
- Windows is a CI-parity and no-change platform for shared-file compatibility and the no-write boundary.
- WSL is outside this checkpoint plan because the touch set does not enter WSL-specific scripts, backends, or runner surfaces.

## Machine-readable plan

```json
{
  "version": 1,
  "defaults": {
    "min_candidates_per_checkpoint": 2,
    "max_candidates_per_checkpoint": 6
  },
  "platform_scope": {
    "behavior_platforms": ["macos"],
    "ci_parity_platforms": ["linux", "macos", "windows"],
    "excluded_platforms": ["wsl"]
  },
  "checkpoints": [
    {
      "checkpoint_id": "CP1",
      "candidate_ids": ["PMHOS-01", "PMHOS-02", "PMHOS-03"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted"
      },
      "rationale": "Run one checkpoint after the schema and merge contract, the macOS writer flow, and the validation and documentation reconciliation land together. The feature changes a shared persisted file, and the highest-value confirmation arrives after producer, compatibility, and evidence surfaces align in one bounded seam."
    }
  ]
}
```

## Human-readable rationale

### CP1

Boundary summary:
- CP1 covers `PMHOS-01`, `PMHOS-02`, and `PMHOS-03` together.
- One checkpoint fits the pack because the draft candidate count is three and the risk seam sits at the shared-file boundary, not at an earlier isolated subsystem seam.

Why this boundary is code-grounded:
- `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` share the install-state writer pattern and both need the macOS producer contract plus warning-only recovery semantics.
- `tests/mac/installer_parity_fixture.sh` and `tests/installers/install_state_smoke.sh` validate different parts of the same shared-file contract, so the verification boundary lands after both harness roles are reconciled.
- `docs/INSTALLATION.md` and the pack-local parity/manual-validation docs need the final producer and no-change story before later platform confirmation is worth the cost.

What surfaces are stabilized at this checkpoint:
- macOS creation or update of `<effective_prefix>/install_state.json`
- additive merge of `host_state.os.*` with existing Linux cleanup and platform metadata
- warning-only degradation for collection, parse, temp-file write, and replace failures
- Linux no-change guarantees for `host_state.platform.*` and cleanup-reader behavior
- Windows no-write and no-change compatibility boundaries
- operator guidance and validation evidence that describe the same producer matrix

Intended verification gates at this checkpoint:
- Compile parity on Linux, macOS, and Windows to catch shared-script, test-harness, and documentation-invocation regressions across the repo-supported CI-parity platforms.
- Feature smoke centered on macOS producer behavior, with the existing harnesses asserting file creation, expected `host_state.os.*` content, and warning-only degradation on partial collection failures.
- Targeted CI testing for installer and harness surfaces that exercises additive merge preservation, Linux cleanup no-change behavior, and Windows compatibility parity without expanding into unrelated platform subsystems.

Risk reduced by running verification here:
- Confirms the shared persisted-file contract after all producer, compatibility, and validation surfaces agree on one shape.
- Catches regressions where macOS writes break Linux cleanup semantics or shared install-state merge behavior.
- Avoids earlier heavy multi-platform runs that would validate only partial producer or parity intent.

Downstream confirmation still required:
- Final downstream planning needs to map the checkpoint gates to the exact harness and runner wiring owned by the execution subsystem.
- Final downstream planning needs to confirm whether targeted CI testing stays scoped to installer and harness surfaces or expands to a broader release-safe pass.

## Follow-ups

- Replace draft candidate IDs with the final downstream identifiers once decomposition finalizes them.
- Confirm the final gate-to-harness mapping once downstream planning locks the exact touched surfaces and validation cadence.
- Reconcile ADR-0039 validation wording with the selected hosted-plus-dev producer scope before execution wiring begins.
- Convert this checkpoint intent into concrete execution wiring only in the downstream subsystem that owns execution.
