# Plan: Agent Drift Analyzer Adapted External Robustness Fixture Family (R5.75-5)

Status: draft plan created on 2026-06-23 after re-reading the live `R5.75-5` packet in
`docs/specs/r5/R5_75/MAP.md`, the current `progress_acceptance` and `objective_acceptance`
contracts, and the available adapted corpus under `target/ranga-validation/`.

## Objective

Turn the already-used adapted external repros into a bounded, reviewable, explicitly secondary fixture
family without weakening the native acceptance anchor or widening into new analyzer semantics.

## Planning Decisions Locked For This Draft

1. The packet is a fixture-family packet first, not a semantic-retuning packet. If committed adapted
   fixtures reveal a fresh analyzer bug, stop and route follow-on work instead of quietly fixing logic
   here.
2. Native rollout cases remain the primary semantic authority. The adapted lane exists only as secondary
   robustness evidence and must be labeled that way in code, fixture metadata, docs, and promotion notes.
3. Progress-shaped adapted cases are required in this packet and should land as committed fixtures.
   Objective-shaped adapted cases are optional and must clear a "net-new signal" bar before they are
   admitted.
4. The packet should restore the missing current fixture authority doc at
   `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` instead of leaving the packet
   to rely on archived docs plus README fragments.
5. Packet prompts remain out of scope because the user asked for spec/plan/tasks only.

## Why This Packet Exists

`R5.75-1` through `R5.75-4` already relied on adapted external sessions as robustness witnesses:

- `05a56cc51632982b` helped validate giant-pasted-prompt objective behavior,
- `f47b81f39f2495dd` proved sparse-readable fail-open,
- `097d97e914ca220f` proved zero-verifier anti-flap conservatism,
- `da59436e63915185` preserved delegated parent-visible guardrails.

Those cases currently live as manual smoke evidence under `target/ranga-validation/` and
`target/r5_75-smoke/`, but they are not yet a committed bounded wall. Without this packet, future
regressions can silently reappear until someone reruns the adapted smoke manually.

## Dependency Graph

```text
docs lock (this SPEC / PLAN / TASKS)
  -> audit each adapted sample id by shape and decide its home
  -> restore/update the fixture authority docs and README contracts
  -> add the progress secondary lane + commit adapted progress fixtures
  -> decide whether objective case 05a56cc51632982b adds net-new signal
  -> run focused acceptance suites
  -> rerun full named native + adapted smoke set
  -> update MAP only after the packet is honestly promoted

explicitly deferred / out of scope:
  -> fresh analyzer semantic retuning
  -> widening the adapted corpus beyond the adopted repro classes
  -> `R5.75-6` structured-goal narrowing faithfulness work
  -> `R6` scorer redesign
```

## Major Components And Dependencies

1. **Adapted case routing audit**
   - dependency: live `MAP.md` packet scope and existing harness contracts
   - outcome: every adopted adapted session is classified by shape and assigned its correct home

2. **Fixture contract restoration**
   - dependency: the routing audit clarifies what the live corpus now owns
   - outcome: README/docs make native-vs-adapted authority and bounded scope explicit

3. **Progress secondary lane**
   - dependency: progress-shaped adopted cases are identified and sourced from deterministic compactor
     artifacts
   - outcome: committed adapted progress fixtures plus harness assertions

4. **Objective stretch-external audit**
   - dependency: `R5.75-1` locked objective corpus is re-read first
   - outcome: either one bounded adapted objective case lands, or the placeholder is intentionally kept
     empty with explicit rationale

5. **Packet closeout validation**
   - dependency: committed fixtures and docs are in place
   - outcome: focused harness tests and the full named smoke set prove the packet is ready for promotion

## Implementation Order

1. Commit the docs triplet so the packet boundary is explicit before any fixture edits begin.
2. Audit the adopted adapted sessions by shape:
   - `05a56cc51632982b` → objective-shaped candidate
   - `f47b81f39f2495dd` → progress-shaped sparse-readable candidate
   - `097d97e914ca220f` → progress-shaped zero-verifier candidate
   - `da59436e63915185` → progress-shaped delegated parent-visible candidate
3. Restore or create `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` and update
   the fixture README contract(s) so the live authority reflects the native primary wall plus adapted
   secondary lane.
4. Add the bounded adapted progress lane in `progress_acceptance.rs` and commit the three adopted
   progress fixtures with explicit secondary metadata.
5. Re-read the `R5.75-1` locked objective cases and decide whether `05a56cc51632982b` adds genuine new
   signal. Only then either:
   - add one `stretch-external` case, or
   - record the explicit no-op rationale and keep the placeholder-only contract intact.
6. Run the focused objective/progress acceptance suites and the full analyzer wall.
7. Rerun the full named native + adapted smoke set from `MAP.md`, inspecting the outputs rather than
   relying on command success alone.
8. Update `docs/specs/r5/R5_75/MAP.md` only after all earlier issue expectations still hold together.

This order matters because the packet first decides what belongs in the corpus, then makes the contract
explicit, then commits the bounded fixtures, then proves the packet still preserves all earlier issue
expectations under full smoke.

## Risks And Mitigations

### Risk 1: the adapted lane silently dilutes native authority

- Risk: new fixture kinds or case lists could make adapted outputs look equivalent to native primary
  acceptance.
- Mitigation: keep explicit native/adapted separation in constants, `FixtureKind`, README language, and
  fixture-manifest docs; preserve at least one native annotated real-rollout anchor and keep adapted
  cases labeled secondary everywhere.

### Risk 2: objective stretch-external duplicates existing coverage

- Risk: `05a56cc51632982b` may only restate `R5.75-1`'s locked cases and add corpus bulk without new
  value.
- Mitigation: require a "net-new signal" audit before adding any adapted objective case; default to a
  documented no-op if that bar is not met.

### Risk 3: fixture committal exposes a fresh semantic gap and the packet widens into implementation

- Risk: one adapted case might not be deterministic or might still disagree with the live analyzer.
- Mitigation: treat that as a stop condition; do not silently retune analyzer code inside this fixture
  packet.

### Risk 4: the missing fixture authority doc remains unresolved

- Risk: the packet could update READMEs/tests but still leave the live docs authority broken because the
  currently referenced fixture manifest only exists in `docs/specs/archived/`.
- Mitigation: make restoring/updating
  `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` a first-class packet task.

## Verification Checkpoints

### Checkpoint A: Adapted routing is explicit

Pass when:

- each adopted adapted sample id is classified by shape and assigned a correct home
- the packet explicitly records why `05a56cc51632982b` is or is not admitted

### Checkpoint B: Fixture authority is honest

Pass when:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` exists and documents native,
  synthetic, and adapted ownership honestly
- README contracts clearly mark adapted cases as secondary robustness, not primary authority

### Checkpoint C: Progress secondary lane is committed

Pass when:

- `progress_acceptance` contains deterministic adapted fixtures for the three adopted progress classes
- the harness explicitly distinguishes native annotated real rollouts, adapted external cases, and
  synthetic support cases

### Checkpoint D: Objective stretch-external outcome is honest

Pass when either:

- one bounded adapted objective case lands with explicit net-new-signal rationale, or
- `stretch-external/` stays placeholder-only and the packet records the explicit no-op reason

### Checkpoint E: Acceptance suites are green

Pass when:

- `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture` is green
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
- `cargo test -p agent-drift-analyzer -- --nocapture` is green

### Checkpoint F: Full smoke rerun still holds earlier issue expectations

Pass when:

- every named native smoke session still holds the packet expectations from `R5.75-1` through `R5.75-4`
- every named adapted smoke session still holds the adopted expectations the new fixture family claims
- no adopted repro class is left guarded only by ad hoc `target/` artifacts

## Parallel vs Sequential Work

- **Can happen in parallel**
  - progress fixture metadata drafting for the three adopted progress-shaped sessions
  - objective `05a56cc51632982b` audit, once the locked objective corpus is re-read
- **Must stay sequential**
  - the docs lock before implementation
  - routing audit before fixture committal
  - fixture authority contract update before promotion
  - full named smoke rerun after all fixture/harness changes are complete

## Promotion Readiness Summary

`R5.75-5` is ready to promote only when:

1. the adapted progress classes are committed as a bounded secondary wall,
2. objective `stretch-external/` is either honestly still empty or bounded to one real-signal case,
3. the fixture authority docs are current in the live docs tree,
4. objective/progress acceptance suites and the full analyzer wall are green, and
5. the full native + adapted smoke rerun proves `R5.75-1` through `R5.75-4` still hold together after
   the fixture-family landing.
