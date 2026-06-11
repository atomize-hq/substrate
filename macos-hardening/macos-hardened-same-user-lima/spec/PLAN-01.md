# PLAN-01: Supported Mode and Support Taxonomy

Source spec:
- [`SPEC-01-supported-mode-and-support-taxonomy.md`](./SPEC-01-supported-mode-and-support-taxonomy.md)

Source phase authority:
- [`../phase-0-security-contract-and-scope/README.md`](../phase-0-security-contract-and-scope/README.md)
- [`../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`](../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Primary design input:
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Plan type: first bounded contract slice for the macOS hardened same-user Lima
program  
Phase: `PLAN`  
Status: draft plan

## Plan summary

The next honest seam is not transport or version-floor work yet. It is the
feature’s first explicit support-contract slice:

1. freeze one supported-mode definition,
2. freeze one support taxonomy,
3. align the most visible feature-local docs to that taxonomy,
4. leave all version-sensitive and implementation-bearing semantics to later
   slices.

This plan should produce a narrow docs-and-contract landing that makes future
Slice `02` through Slice `12` work safer and more mechanically consistent.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice
2. `source-driven-development` is light/targeted only

Plan consequence:

1. use repo truth, phase authority, design docs, and the existing research note
   as the primary inputs
2. do **not** block this slice on new official Lima/systemd/Apple documentation
   fetches unless the implementation widens beyond support taxonomy or
   breakglass classification language

## Major components and dependencies

1. **feature-level contract alignment**
   - `../README.md`
   - should state the same supported-mode posture as the slice
2. **phase-0 contract alignment**
   - `../phase-0-security-contract-and-scope/README.md`
   - `../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`
   - should remain the milestone authority for the first phase
3. **feature execution scaffolding alignment**
   - `../ROADMAP.md`
   - `../EXECUTION-RUBRIC.md`
   - should use the same taxonomy and slice-language terms
4. **design-doc contract alignment**
   - `design/DESIGN-supported-mode-and-breakglass-taxonomy.md`
   - may need bounded refinement if the slice discovers wording drift

Dependency order:

1. terminology and support-mode freeze first
2. visible feature-local propagation second
3. validation and explicit deferral framing last

## Locked decisions

### What this slice changes

1. It freezes the first implementation-facing supported-mode definition.
2. It freezes one support taxonomy:
   - `supported`
   - `degraded-but-supported`
   - `breakglass`
3. It aligns the most visible feature-local authorities around that taxonomy.
4. It makes later deferred seams explicit.

### What this slice does not change

1. no Lima version-floor freeze
2. no transport contract freeze
3. no backend policy input propagation changes
4. no mount minimization or unit unification work
5. no operator CLI implementation or guest lifecycle implementation work

## Implementation order

### Packet 1: Freeze the supported-mode contract

Goal:

1. define the exact supported same-user Lima posture
2. define the exact non-goals relative to Linux host-side ownership
3. confirm which already-landed operator surfaces belong in the normal story

Primary touch surface:

1. `SPEC-01-supported-mode-and-support-taxonomy.md`
2. `../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`
3. `design/DESIGN-supported-mode-and-breakglass-taxonomy.md` only if bounded
   refinement is required

Why first:

1. the taxonomy depends on the supported-mode definition
2. later visible doc alignment must reuse one frozen posture

Verification checkpoint:

1. the same-user limitation is explicit
2. the normal operator path is explicit
3. Linux non-parity claims are explicit

### Packet 2: Freeze the support taxonomy and propagate it through visible feature-local authorities

Goal:

1. define `supported`, `degraded-but-supported`, and `breakglass` exactly
2. propagate those definitions to the feature-local documents that future
   sessions are most likely to read first

Primary touch surface:

1. `../README.md`
2. `../EXECUTION-RUBRIC.md`
3. `../ROADMAP.md`
4. `design/DESIGN-supported-mode-and-breakglass-taxonomy.md`
5. `../phase-0-security-contract-and-scope/README.md` or milestone `0.1` only
   where terminology drift exists

Why second:

1. visible feature-local docs should not disagree once the posture is frozen
2. the execution rubric should inherit the same terms as the feature contract

Verification checkpoint:

1. terminology is consistent across touched docs
2. support-class examples do not overlap or contradict each other

### Packet 3: Validation, explicit deferrals, and next-slice handoff clarity

Goal:

1. verify the slice did not absorb Slice `02` or later seams
2. make deferred follow-on seams explicit
3. leave the next-slice planning surface clean for a short prompt

Primary touch surface:

1. `SPEC-01-supported-mode-and-support-taxonomy.md`
2. `PLAN-01.md`
3. `TASKS-01.md`
4. touched feature-local docs only if final wording adjustments are required

Why third:

1. deferrals can only be stated cleanly after the supported-mode and taxonomy
   wording are stable
2. this packet protects Slice `02` from being muddied by leftover ambiguity

Verification checkpoint:

1. later seams are named explicitly
2. Slice `02` remains version-floor and breakglass-contract work rather than a
   rewrite of Slice `01`

## Risks and mitigations

### Risk 1: Slice `01` widens into version-sensitive Lima semantics

Mitigation:

1. treat any need for official-Lima semantic resolution as a stop sign for
   Slice `02`
2. keep Slice `01` at the contract-language layer

### Risk 2: Support-taxonomy language drifts across visible feature docs

Mitigation:

1. run terminology scans with `rg`
2. keep one terminology table or equivalent wording shared across the touched
   docs

### Risk 3: The slice stays too abstract and does not improve future execution

Mitigation:

1. require explicit deferred-seam language
2. require enough propagation that future short prompts can read the resulting
   documents and identify the next honest slice

## Parallelism guidance

This slice is mostly sequential.

Parallelizable work:

1. scanning existing feature-local docs for terminology drift
2. identifying candidate docs to align

Sequential work:

1. freezing the supported-mode posture
2. freezing the support taxonomy
3. validating that deferred seams are still cleanly separated

## Exit criteria

This plan is ready to hand off to `TASKS-01` only when:

1. the slice remains bounded to support-contract language and taxonomy
2. the expected touched docs are clear
3. later seams remain explicitly deferred
4. the next planning session could use the result without re-arguing the basic
   support posture
