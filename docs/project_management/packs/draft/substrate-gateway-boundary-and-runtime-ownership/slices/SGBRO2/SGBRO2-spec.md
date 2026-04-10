# SGBRO2-spec - lock policy-evaluation planning wiring

## Behavior delta (single)
- Existing: the policy-evaluation lane is present in triage, but its execution-ready planning references are not yet normalized from a slice-local spec.
- New: `SGBRO2` becomes the authoritative planning slice for policy/trust-boundary wiring without changing policy semantics.
- Why: the planning pack needs one stable mid-chain policy slice that preserves ordering and boundary rules.

## Scope
- Lock the task and prompt wiring for the policy-evaluation slice.
- Keep the planning pack aligned to the accepted slice order and checkpoint boundary.
- Do not define new policy inputs or trust-boundary semantics here.

## Behavior (authoritative)

### Policy-evaluation planning lane
- `SGBRO2` stays ordered after `SGBRO1` and before `SGBRO3`.
- `SGBRO2` task entries point at the slice-local prompt directory and slice spec.
- `SGBRO2` does not widen the checkpoint boundary beyond `SGBRO4`.

## Acceptance criteria
- AC-SGBRO2-01: the SGBRO2 triad entries point at `slices/SGBRO2/kickoff_prompts/`.
- AC-SGBRO2-02: the SGBRO2 slice is ordered after SGBRO1 and before SGBRO3 in every planning surface.
- AC-SGBRO2-03: the policy slice does not widen the checkpoint boundary beyond `SGBRO4`.

## Out of scope
- Status-schema wording.
- Runtime-parity wording.
- Docs-validation closeout evidence.
