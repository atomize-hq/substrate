# SGBRO1-spec - lock status-schema planning wiring

## Behavior delta (single)
- Existing: the status-schema planning slice is identified in triage, but its execution-ready task/prompt wiring is not yet pinned from a slice-local spec.
- New: `SGBRO1` becomes the authoritative planning slice for the status-schema lane and keeps its triad paths aligned without widening into runtime or policy work.
- Why: later planning artifacts need a stable, non-boundary schema slice that can be referenced without ambiguity.

## Scope
- Lock the task and prompt wiring for the status-schema slice.
- Keep `tasks.json`, `plan.md`, and the checkpoint plan consistent with the status-schema slice id.
- Do not redefine the JSON schema itself in this slice.

## Behavior (authoritative)

### Status-schema planning lane
- `SGBRO1` stays ordered after `SGBRO0` and before `SGBRO2`.
- `SGBRO1` task entries point at the slice-local prompt directory and slice spec.
- `SGBRO1` does not introduce a checkpoint boundary.

## Acceptance criteria
- AC-SGBRO1-01: the SGBRO1 triad entries point at `slices/SGBRO1/kickoff_prompts/`.
- AC-SGBRO1-02: the SGBRO1 slice references the accepted five-slice order in the same terms as the manifest.
- AC-SGBRO1-03: the SGBRO1 slice does not introduce any new checkpoint boundary.

## Out of scope
- Contract wording.
- Policy wording.
- Runtime-parity wording.
