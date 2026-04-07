---
slice_id: S3
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S3 - Contract publication surfaces

- **User/system value**: once runtime and validation behavior land, operators and downstream planners can consume one documented contract instead of reading code or stale planning artifacts.
- **Scope (in/out)**:
  - In:
    - update operator-facing and planning-facing docs that must publish `C-01` and `C-02`
    - align `docs/REPLAY.md`, `docs/TRACE.md`, `docs/internals/env/inventory.md`, and the active WPEP pack contract surfaces to the landed truth
    - record the minimal governance evidence needed for downstream planning
  - Out:
    - net-new runtime changes
    - downstream conformance/drift-guard ownership that belongs to `SEAM-3`
- **Acceptance criteria**:
  - all touched docs describe the same four-case routing matrix and the same behavior matrix
  - documentation preserves the canonical trace omission rule
  - the pack's planning surfaces stop carrying stale proxy wording
- **Dependencies**:
  - `C-01`, `C-02`
  - `THR-01`
  - landed outcomes from `S1` and `S2`
  - `docs/REPLAY.md`
  - `docs/TRACE.md`
  - `docs/internals/env/inventory.md`
  - active WPEP pack contract/playbook surfaces
- **Verification**:
  - readback diff across all touched publication surfaces
  - targeted smoke/manual checks cited by the updated docs
- **Rollout/safety**: documentation must never imply raw builtin/preexec command bodies are present in canonical trace.
- **Review surface refs**: `../../review_surfaces.md` R2 and R3

#### S3.T1 - Publish the replay routing contract in operator-facing docs

- **Outcome**: replay docs stop describing replay as a separate routing model.
- **Inputs/outputs**:
  - Inputs: landed `C-01` behavior, `docs/REPLAY.md`, any nearby replay contract references
  - Outputs: one explicit four-case routing description and verification guidance
- **Thread/contract refs**: `THR-01`, `C-01`
- **Acceptance criteria**:
  - docs explain how replay determines `world_network` without introducing replay-only heuristics
- **Test notes**:
  - cross-check examples and wording against runtime/test evidence from `S1`

Checklist:
- Implement: docs updates only
- Test: N/A
- Validate: compare wording to `C-01`
- Cleanup: remove stale replay-specific routing language

#### S3.T2 - Publish the tracing behavior matrix and omission posture in docs/planning surfaces

- **Outcome**: operator docs, inventory, and the active WPEP pack all cite one tracing contract.
- **Inputs/outputs**:
  - Inputs: landed `C-02` behavior, `docs/TRACE.md`, `docs/internals/env/inventory.md`, WPEP contract/playbook surfaces
  - Outputs: one matrix description and one safe trace omission story
- **Thread/contract refs**: `THR-01`, `C-02`
- **Acceptance criteria**:
  - no touched doc implies canonical trace includes raw builtin command bodies
  - Case B documentation and trace docs use the same matrix terminology
- **Test notes**:
  - verify docs point at the same validation assets updated in `S2`

Checklist:
- Implement: docs/planning updates only
- Test: N/A
- Validate: compare wording to `C-02`
- Cleanup: remove stale proxy wording and duplicated matrix text where possible
