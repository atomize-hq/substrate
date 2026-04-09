# SGBRO3-spec - lock typed runtime and parity planning wiring

## Behavior delta (single)
- Existing: the runtime/parity lane is accepted in triage, but its execution-ready planning references are not yet formalized from a slice-local spec.
- New: `SGBRO3` becomes the authoritative planning slice for runtime/parity task wiring while preserving the later handoff to the checkpoint-boundary slice.
- Why: the final docs-validation slice needs one stable predecessor for runtime/parity planning evidence.

## Scope
- Lock the task and prompt wiring for the typed runtime and parity slice.
- Keep the planning pack aligned to the accepted slice order and checkpoint plan.
- Do not add provisioning or runtime implementation detail here.

## Behavior (authoritative)

### Runtime/parity planning lane
- `SGBRO3` stays the last non-boundary slice before `SGBRO4`.
- `SGBRO3` task entries point at the slice-local prompt directory and slice spec.
- `SGBRO3` does not alter the CP1 boundary.

## Acceptance criteria
- AC-SGBRO3-01: the SGBRO3 triad entries point at `slices/SGBRO3/kickoff_prompts/`.
- AC-SGBRO3-02: the SGBRO3 slice remains the penultimate implementation slice before docs-validation and checkpoint lock-in.
- AC-SGBRO3-03: the parity slice does not alter the CP1 boundary.

## Out of scope
- Contract wording.
- Status-schema wording.
- Policy-evaluation wording.
