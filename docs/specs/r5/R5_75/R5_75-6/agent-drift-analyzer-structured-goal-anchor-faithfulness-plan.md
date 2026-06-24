# Plan: Agent Drift Analyzer Structured Goal Anchor Faithfulness (R5.75-6)

Status: Task `R5.75-6.0.1` docs lock finalized on 2026-06-24 after re-reading the live `R5.75-6` routing in
`docs/specs/r5/R5_75/MAP.md`, the Issue 7 diagnosis in
`docs/specs/r5/R5_75/structured-objective-bug-map.md`, the current checkpoint compatibility bridge,
and the current migration boundary that explicitly defers the broader structured-native consumer
cutover.

## Objective

Close the final pre-`R6` structured-objective gap by making the effective checkpoint objective
faithful to the grounded structured goal anchor, without widening into the later full consumer
migration.

## Planning Decisions Locked For This Draft

1. This packet is a **bridge-fidelity** packet, not a migration packet. The job is to stop the
   compatibility overlay from overwriting a correct structured goal with the wrong imperative line.
2. The structured goal anchor already produced by `context/objective.rs` is the semantic authority
   when it is grounded. Legacy narrowing remains fallback only.
3. The packet should prefer a local checkpoint-layer repair first. If a broader consumer migration
   seems required, stop and re-scope instead of smuggling it into `R5.75-6`.
4. Earlier `R5.75` packet behavior is part of the acceptance wall, not incidental context:
   `R5.75-1` anchored review semantics, `R5.75-3` delegated stability, `R5.75-4` anti-flap
   conservatism, and `R5.75-5` native/adapted smoke expectations all remain live guardrails.
5. Packet prompts remain out of scope because the user asked for SPEC/PLAN/TASKS only.

## Why This Packet Exists

`R5.75-1` fixed objective anchoring at the structured sidecar level, but the analyzer still exports
an effective compatibility objective through the legacy narrowing path:

- `checkpoint_analyses(...)` assembles structured context first,
- then `narrowed_objective_summary(...)` runs,
- and `infer_task_frame(...)` still reads `context.objective.text`.

That leaves one final pre-`R6` hole: a good structured goal can still be paired with a bad effective
objective string. The named bug-map repro `019eddaa-e8b2-74b2-9f45-e4ce17aaab55` proves the risk is
real today.

## Dependency Graph

```text
docs lock (this SPEC / PLAN / TASKS)
  -> inspect the live compatibility overlay and choose the bounded faithfulness rule
  -> implement the checkpoint-layer bridge guard
  -> add minimized wrong-imperative regression coverage
  -> rerun checkpoint/progress/analyzer/sentinel gates
  -> rerun native repro smoke + full carried-forward R5.75 smoke set
  -> update MAP only after R5.75-6 is honestly promoted

explicitly deferred / out of scope:
  -> TaskFrame schema/objective_key coexistence migration
  -> context/working_set.rs migration
  -> checkpoint/progress.rs comparison redesign
  -> R6 scorer retuning
```

## Major Components And Dependencies

1. **Bridge rule audit**
   - dependency: live `MAP.md`, bug-map Issue 7, and current checkpoint/inference code
   - outcome: a precise rule for when structured authority must beat legacy narrowing

2. **Checkpoint compatibility repair**
   - dependency: bridge rule audit
   - outcome: `context.objective.text` remains faithful when structured goal evidence is grounded,
     while legacy fallback still exists for absent/weak structure

3. **Regression harness expansion**
   - dependency: compatibility repair design is chosen
   - outcome: a minimized wrong-imperative regression and any required fallback guard in
     `tests/checkpoints.rs`

4. **Downstream non-regression wall**
   - dependency: code and tests are in place
   - outcome: progress/analyzer/sentinel suites prove that the packet does not regress earlier
     semantics or downstream consumer expectations

5. **Promotion smoke and closeout**
   - dependency: automated suites are green
   - outcome: named smoke witnesses prove `R5.75-6` is complete and `R5.75` can close honestly

## Implementation Order

1. Commit the docs triplet so the packet boundary is explicit before code changes begin.
2. Inspect the live bridge between `assemble_context(...)`, `narrowed_objective_summary(...)`, and
   `infer_task_frame(...)`, then lock the bounded faithfulness rule:
   - grounded structured goal wins,
   - optional reviewer nits do not become the effective objective,
   - legacy narrowing remains fallback when structure is absent or weak.
3. Implement the local checkpoint-layer repair in the smallest viable seam.
4. Add checkpoint regressions for the real failure shape and any necessary fallback guard.
5. Run the focused checkpoint suite first, then the downstream progress/analyzer/sentinel walls.
6. Rerun manual smoke for `019eddaa-...` and `019eb47f-...`, then rerun the full named `R5.75-5`
   smoke set under `target/r5_75-smoke/R5.75-6/`.
7. Update `docs/specs/r5/R5_75/MAP.md` only after the packet is honestly promoted and `R5.75`
   truly closes.

This order keeps the packet bounded: first repair the bridge, then lock the repro, then prove no
other `R5.75` behavior regressed.

## Risks And Mitigations

### Risk 1: the packet silently widens into full consumer migration

- Risk: once the team touches `task_frame.objective`, it is easy to start pulling in
  `objective_key`, `structured_objective`, or progress comparability redesign.
- Mitigation: treat any schema or broader consumer migration requirement as a stop condition and
  re-scope rather than widening this packet.

### Risk 2: fixing the wrong-imperative repro breaks fallback behavior

- Risk: if legacy narrowing is disabled too broadly, sessions without useful structured grounding may
  lose their current compatibility behavior.
- Mitigation: keep fallback explicit and add a focused fallback guard in `tests/checkpoints.rs` if
  the implementation touches shared normalization logic.

### Risk 3: progress or sentinel surfaces regress because objective text changes

- Risk: even a bounded objective-string change can alter continuity logic or downstream replay
  expectations.
- Mitigation: make `progress_acceptance`, the full analyzer wall, and sentinel spot-checks required
  promotion gates rather than optional validation.

### Risk 4: the packet fixes only a synthetic test and not the real repro

- Risk: a minimized regression could miss the actual optional-nit session shape.
- Mitigation: require the named real-session smoke rerun for `019eddaa-...` in addition to unit
  regressions, and keep the earlier `019eb47f-...` anchored-review witness in the smoke wall.

## Verification Checkpoints

### Checkpoint A: Bridge rule is explicit

Pass when:

- the implementation documents or clearly encodes when a structured goal anchor outranks legacy
  narrowing, and
- that rule stays within the bounded `R5.75-6` scope.

### Checkpoint B: Wrong-imperative repro is locked

Pass when:

- `tests/checkpoints.rs` contains a durable regression for the `019eddaa`-shape failure, and
- the test asserts the effective objective stays aligned to the real validate/readiness ask.

### Checkpoint C: Fallback behavior stays honest

Pass when either:

- existing tests already prove non-structured fallback remains correct after the change, or
- a new focused guard is added to prove the packet did not over-disable narrowing.

### Checkpoint D: Downstream automated walls are green

Pass when:

- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- sentinel `warning_policy` and `live_end_to_end` are green

### Checkpoint E: Full smoke rerun preserves prior packets

Pass when:

- `019eddaa-...` no longer narrows to the optional reviewer nit,
- `019eb47f-...` still anchors to the correct evaluate/review ask,
- the full named native + adapted smoke set from `R5.75-5` still holds, and
- no earlier `R5.75` issue regresses under the new effective-objective bridge.

## Parallel vs Sequential Work

- **Can happen in parallel**
  - drafting the minimized checkpoint regression once the bridge rule is known
  - preparing the native smoke rerun commands and expected assertions
- **Must stay sequential**
  - docs lock before implementation
  - bridge rule before code repair
  - code repair before final regression assertions
  - automated gates before manual promotion smoke
  - honest smoke review before any `MAP.md` promotion update

## Promotion Readiness Summary

`R5.75-6` is ready to promote only when:

1. the checkpoint compatibility bridge no longer lets optional reviewer nits overwrite a grounded
   structured goal,
2. the bounded fix stays analyzer-local and does not widen into deferred migration work,
3. the named wrong-imperative repro and anchored-review witness both pass,
4. checkpoint/progress/analyzer/sentinel gates are green, and
5. the full carried-forward smoke rerun proves `R5.75-1` through `R5.75-5` still hold, allowing
   `R5.75` to close and `R6` to begin honestly.
