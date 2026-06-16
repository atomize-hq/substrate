# Plan: Agent Drift Analyzer Structured Objective Phase 1 Grounding Follow-On

Status: draft plan created on 2026-06-16 after reviewing the current crate snapshot, the narrow
harness-fix landing, and the structured-objective design stack. This plan assumes the harness fix
is correct and keeps the follow-on bounded to additive grounding restoration plus docs/acceptance
sequencing.

## Objective

Restore durable section/clause grounding identifiers to the structured-objective sidecar, harden
the focused checkpoint proof around ambiguous headings and duplicate excerpts, and reconcile the
phase-1 planning docs so the next implementation packets are reviewable and honest.

## Planning Decisions Locked For This Draft

1. The harness-fix commit stands. This follow-on is a new structured-objective packet family, not a
   reopening of the checkpoint fixture repair.
2. `section_index` / `clause_index` come back as additive optional schema fields first; true
   character offsets remain explicitly deferred unless later approved.
3. The code seam remains analyzer-local:
   `src/checkpoint/schema.rs`, `src/context/objective.rs`, and focused checkpoint tests.
4. The phase-1 plan/tasks docs must be reconciled as part of this family because future packet
   agents will otherwise inherit false “everything is still pending” assumptions.
5. `comparison_key` honesty is a planning gate: no downstream migration packet may treat it as the
   semantic comparison bridge until the dedicated derivation work lands.
6. The objective-acceptance harness should be the next meaningful acceptance seam after grounding
   restoration, but this family does not silently widen into a full acceptance implementation unless
   explicitly approved.

## Why This Follow-On Exists

The current sidecar is materially ahead of the old SO-2.1 stopgap: schema DTOs, optional
`structured`, section/clause decomposition, role candidates, and preliminary structured assembly are
already present. But current public evidence spans are weaker than the follow-on design direction:

- the restored sidecar still lacks public section/clause identifiers,
- excerpt-only grounding is too weak for ambiguous duplicate wording,
- `comparison_key` still mirrors display text,
- the phase-1 task ledger still presents landed work as if it were untouched backlog,
- and the dedicated objective-acceptance harness has not yet become the real promotion wall.

This plan isolates those gaps into a small, reviewable packet family.

## Dependency Graph

```text
follow-on docs lock
  -> additive section/clause grounding fields
  -> focused grounding + adversarial heading regressions
  -> phase-1 ledger reconciliation
  -> explicit comparison_key guardrail
  -> objective-acceptance harness sequencing
  -> full packet validation

explicitly deferred:
  -> real character offsets
  -> TaskFrame/objective_key coexistence
  -> working_set / checkpoint / progress migration
  -> downstream reliance on comparison_key
  -> analyzer-time classifier experiments
```

## Recommended Landing Sequence

## SO-G0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- add a bounded follow-on SPEC/PLAN/TASKS family for grounding restoration and docs reconciliation
- record the assumptions that keep the harness fix intact
- make the next packet ordering explicit before implementation begins

### Why First

The existing phase-1 docs no longer match landed code well enough to serve as a reliable packet
authority on their own. This follow-on needs its own honest implementation authority before more
changes land.

### Verification

Manual review only.

## SO-G1: Restore Additive Grounding Identifiers

### Scope

- add optional `section_index` / `clause_index` back to `ObjectiveEvidenceSpan`
- restore a local per-section clause index in the decomposition substrate
- populate the evidence spans from the current clause-backed assembly path
- keep `start_char` / `end_char` optional and conservatively unset unless real offsets already
  exist cheaply

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
crates/agent-drift-analyzer/src/context/objective.rs
```

### Why Before Anything Else

The grounding identifiers are the cheapest honest bridge between today’s coarse public evidence
spans and the future acceptance harness. They also avoid pretending that unimplemented character
offsets solve localization already.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## SO-G2: Focused Grounding And Heading Regressions

### Scope

- add checkpoint tests proving goal and verification spans carry section/clause identifiers
- add an ambiguous duplicate-ish scope vs checklist case where excerpt-only grounding would be too
  weak
- add adversarial heading controls for:
  - `Task constraints`
  - `Verification task`
  - `Output request`
  - `Questions to ask`
  - `Implementation steps`
  - `What I need`

### Primary Files

```text
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why After SO-G1

The new tests should validate the restored public grounding surface rather than driving ad hoc
local scaffolding first.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## SO-G3: Reconcile The Phase-1 Ledger To Landed Reality

### Scope

- update the existing phase-1 plan/tasks docs so already-landed schema bridge, sidecar exposure,
  section/clause decomposition, and preliminary assembly are not still presented as wholly pending
- identify the remaining work in honest terms:
  - grounding hardening still needed,
  - `comparison_key` derivation still pending,
  - objective-acceptance harness still pending,
  - downstream migration still deferred
- preserve auditability rather than pretending the old ledger never existed

### Primary Files

```text
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
```

### Why Separate From SO-G1/SO-G2

This keeps the code patch focused while still ensuring future agents do not inherit stale packet
assumptions immediately after the grounding work lands.

### Verification

Manual review against:

- the live crate state,
- `docs/specs/r5/DESIGN-r5-structured-objective-architecture.md`,
- `docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md`.

## SO-G4: Lock The Comparison-Key Guardrail And Acceptance Ordering

### Scope

- document explicitly that `comparison_key == display text` is temporary and not yet the approved
  semantic bridge
- block downstream migration from treating current `comparison_key` as ready
- make the objective-acceptance harness the next planned acceptance seam after grounding restoration

### Primary Files

```text
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-spec.md
```

### Why Before Acceptance Work Starts

Without this guardrail, a later packet could migrate internal comparisons onto a stopgap key and
recreate raw-string truth under a more official name.

### Verification

Manual review of the reconciled docs set.

## SO-G5: Bootstrap The Next Objective-Acceptance Packet Boundary

### Scope

- define the next packet entry criteria for the objective-acceptance harness
- require the harness to consume structured fields, role spans, grounding refs, forbidden-promotion
  expectations, compatibility rendering, and unknown-field correctness
- keep this family from silently widening into full fixture expansion unless separately approved
- name the next packet entry point explicitly as SO-4.1 / SO-4.2 under the reconciled phase-1
  ledger rather than leaving the handoff implied by packet numbering alone

### Primary Files

```text
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-grounding-follow-on-tasks.md
```

### Explicit Handoff After This Packet

- the next packet to start after the grounding family closes is SO-4.1 / SO-4.2 in the reconciled
  phase-1 docs
- that packet's acceptance wall must cover structured fields, role spans, grounding refs,
  forbidden promotions, compatibility rendering, and unknown-field correctness
- SO-5 fixture growth remains blocked until the SO-4 harness boundary exists

### Why Separate From The Grounding Patch

Grounding restoration is a small code patch. The acceptance harness is larger and deserves a clean
handoff boundary instead of becoming opportunistic scope creep.

### Verification

Manual review only.

## Risks And Mitigations

### Risk: additive identifiers drift from the actual decomposition units

Mitigation:

- derive `clause_index` directly from the existing section-local clause splitting path,
- cover both goal and verification evidence in focused regressions,
- add a duplicate-ish excerpt case that fails if the wrong clause is surfaced.

### Risk: doc reconciliation accidentally rewrites history instead of clarifying it

Mitigation:

- keep the existing phase-1 docs, but mark landed work and remaining work honestly,
- preserve explicit follow-on packet names instead of deleting earlier packet structure.

### Risk: future agents treat current `comparison_key` as done

Mitigation:

- make the guardrail explicit in both the existing phase-1 docs and this follow-on family,
- keep downstream migration tasks deferred and ask-first.

## Verification Checkpoints

1. **After SO-G1**
   - schema compiles,
   - evidence spans serialize additively,
   - focused checkpoints still pass.

2. **After SO-G2**
   - new regressions prove the identifiers matter,
   - adversarial heading cases are green.

3. **After SO-G3 / SO-G4**
   - docs match live crate reality,
   - no future packet reader could reasonably conclude that `comparison_key` is already finished.

4. **Packet closeout**
   - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
   - `cargo test -p agent-drift-analyzer -- --nocapture`

## Out Of Scope

- implementing real `start_char` / `end_char` offsets,
- migrating `TaskFrame` or progress/comparability logic,
- touching `src/input.rs` or the checkpoint harness fix path,
- regenerating orchestration prompts unless the reconciled ledger explicitly requires it later,
- introducing any classifier runtime or model-assisted extraction path.
