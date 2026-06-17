# Plan: Agent Drift Analyzer Session Progress R5.75-1

Status: draft plan created on 2026-06-12 from `docs/specs/r5/R5_75/MAP.md` and
`docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`.

## Objective

Land a narrow analyzer-local objective-condensation packet so checkpoint objectives reliably anchor
to the concrete task ask instead of to giant pasted prompt/scaffold bodies.

## Planning Decisions Locked For This Packet

1. The packet stays local to checkpoint objective selection in
   `crates/agent-drift-analyzer/src/checkpoint/mod.rs` plus regressions in
   `crates/agent-drift-analyzer/tests/checkpoints.rs`.
2. Native rollout behavior is the primary correctness authority; the adapted external sample is a
   secondary smoke witness only.
3. The fix must address both:
   - candidate selection/ranking, and
   - final objective text condensation/extraction.
   Fixing only one side is not enough because the wrong long row can still win, and the right row
   can still serialize as an unhelpfully giant body.
4. Preserved boilerplate-target requests are part of the correctness wall, not optional niceties.
5. No new fixture family, schema bump, or compactor contract change belongs in this packet.
6. Packet prompts must treat earlier landed packets as prerequisite claims to verify, not
   assumptions to trust; if `R5.75-1.x` prerequisite work is missing in live code/tests, the later
   packet stops and reports that gap instead of silently repairing it.

## Scope Classification

- **In scope**
  - tighten candidate ranking so longer same-priority bodies do not win by length
  - extract or condense the shortest concrete ask from large user prompt bodies where possible
  - keep `/goal`, steer pivots, thread-goal text, and clear imperative task phrases preferred
  - add/update checkpoint regressions proving both condensation and preservation behavior
  - run the named native/adapted manual smoke checks
- **Out of scope**
  - sparse-session fail-open behavior (`R5.75-2`)
  - delegated parent-visible stabilization (`R5.75-3`)
  - zero-verifier anti-flap gating (`R5.75-4`)
  - adapted fixture-family expansion (`R5.75-5`)
  - replay/schema/compactor redesign

## Major Components And Dependencies

1. **Regression capture for concrete-task condensation**
   - dependency: packet scope from `R5.75/MAP.md`
   - outcome: failing or missing cases are written down before heuristic changes

2. **Objective-candidate ranking refinement**
   - dependency: current `objective_candidate(...)` and `narrowed_objective_summary(...)`
   - outcome: same-priority candidates stop favoring the longer pasted body when a shorter concrete
     ask is present

3. **Objective text condensation / target extraction**
   - dependency: ranking logic and a clear preservation rule set
   - outcome: when the chosen row still contains a giant pasted body, the stored objective text is
     reduced to the concrete ask rather than the entire body

4. **Boilerplate-target preservation wall**
   - dependency: condensation rules are defined
   - outcome: analyze/edit/compare/explain requests about instruction scaffolding remain intact and
     do not get shortened into misleading fragments

5. **Packet closeout validation**
   - dependency: code and regression updates complete
   - outcome: targeted/full tests plus the named smoke reviews prove the packet is genuinely ready
     for promotion

## Implementation Order

1. Add/update regressions for the giant-prompt failure modes and preserved boilerplate-target
   cases.
2. Refine candidate ordering so shorter concrete asks outrank longer same-priority pasted bodies.
3. Add objective-text condensation/extraction for large chosen prompt rows.
4. Re-run and adjust preservation regressions so deliberate boilerplate-target requests remain
   intact.
5. Run automated gates, then the named native/adapted smoke checks.

This order matters because the regression wall should define both the new condensation behavior and
the no-regression preservation behavior before the ranking and extraction logic is changed.

## Risks And Mitigations

### Risk 1: Over-condensing and losing the real target

- Risk: a heuristic trims too aggressively and keeps only a generic verb or partial phrase.
- Mitigation: favor explicit task lines (`/goal`, imperative ask, workspace/action-target phrase)
  and require regression coverage for preserved long-form targets.

### Risk 2: Preserved boilerplate-target requests regress

- Risk: shortening logic treats all instruction-heavy text as boilerplate and strips out the actual
  user ask to analyze or edit that surface.
- Mitigation: keep the existing preservation heuristics in the regression wall and ensure
  condensation bypasses or special-cases those deliberate target requests.

### Risk 3: Candidate ordering fix is incomplete without text normalization

- Risk: the correct row wins, but the resulting stored objective is still the whole pasted body.
- Mitigation: explicitly split the implementation into ranking refinement plus final text
  condensation/extraction.

### Risk 4: Synthetic tests pass but real smoke sessions still serialize the wrong objective

- Risk: the packet overfits unit regressions but misses live compaction patterns.
- Mitigation: require manual smoke on the three named native sessions plus the named adapted
  witness before promotion.

## Verification Checkpoints

### Checkpoint A: Regression wall captures the target failures

Pass when:

- `tests/checkpoints.rs` contains explicit giant-prompt condensation cases
- preservation cases for `AGENTS.md`, `<skill>`, `Available skills`, and tooling scaffolds remain
  represented

### Checkpoint B: Candidate ordering is corrected

Pass when:

- same-priority length bias no longer prefers the longer pasted body
- `/goal`, short imperative asks, and clear steer pivots remain preferred where expected

### Checkpoint C: Chosen-row text condenses correctly

Pass when:

- the stored checkpoint objective serializes as the concrete task ask
- preserved boilerplate-target requests remain full enough to describe the deliberate target

### Checkpoint D: Automated gates are green

Pass when:

- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
- `cargo test -p agent-drift-analyzer -- --nocapture` is green

Closeout note for Task `R5.75-1.5` on 2026-06-12:

- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- both commands passed on 2026-06-12
- no implementation/code changes were required for this closeout rerun
- docs-only packet-local closeout evidence was recorded in commit `47b3ab467`
  (`docs: record r5.75-1.5 closeout evidence`)

### Checkpoint E: Smoke review proves live correctness

Pass when:

- native sessions
  `019eb430-6f9a-7a03-9a63-cb451b654795`,
  `019eb47f-0118-7e90-8291-30a1fb93769e`,
  `019eb98e-3c16-7ba0-92f9-0085654b470c`
  show the concrete task as the first-checkpoint objective
- adapted session `05a56cc51632982b` condenses to the workspace action request rather than the
  full pasted skill body

## Parallelism And Sequencing

- **Must stay sequential**
  - regression definition before heuristic changes
  - candidate ordering before final smoke signoff
  - smoke review after automated tests are already green
- **Can happen together inside one edit cycle**
  - ranking refinement and text-condensation helper extraction in `checkpoint/mod.rs`
  - adding multiple related regression cases in `tests/checkpoints.rs`

## Packet Split

`R5.75-1` remains one packet with five internal work blocks:

1. `R5.75-1A` — giant-prompt and preservation regression capture
2. `R5.75-1B` — candidate-ordering refinement
3. `R5.75-1C` — chosen-row condensation / target extraction
4. `R5.75-1D` — automated regression validation
5. `R5.75-1E` — native/adapted smoke review and promotion audit

These are sequencing aids only, not separate promotion packets.

## Files Expected To Change

```text
docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md
docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-plan.md
docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-tasks.md
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

The docs are being added now; the code/test files are the expected implementation touch set for
the later packet work.

## Hard Gate For R5.75-1

Do not treat this packet as landed unless:

- the objective selector stores the concrete task ask for native sessions
  `019eb430-6f9a-7a03-9a63-cb451b654795`,
  `019eb47f-0118-7e90-8291-30a1fb93769e`,
  `019eb98e-3c16-7ba0-92f9-0085654b470c`,
  plus adapted session `05a56cc51632982b`,
- no preserved boilerplate-target regression appears in synthetic checkpoint coverage,
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green,
- `cargo test -p agent-drift-analyzer -- --nocapture` is green, and
- manual smoke review confirms the first checkpoint objective is the concrete ask rather than the
  pasted scaffold body.

This is the packet-specific closeout gate. Do not promote based on automated green status alone.

## Promotion Gate To R5.75-2

Do not promote to `R5.75-2` until:

- the objective selector stores the concrete task ask for all named repros
- preserved boilerplate-target requests still remain intact in regression coverage
- targeted and full analyzer tests are green
- smoke review confirms the expected first-checkpoint objective on the named native and adapted
  sessions

## Notes From Live Code Review

Current live code already explains the packet shape:

1. `narrowed_objective_summary(...)` currently uses `left.text.len().cmp(&right.text.len())` as
   the last tie-break, which can let the longer same-priority row win.
2. `normalized_objective_text(...)` currently only trims to the first double-newline paragraph,
   which is too weak for giant pasted user prompts whose concrete ask is embedded deeper in the
   same body.
3. Preservation helpers such as `preserves_user_requested_boilerplate_target(...)` and
   `mentions_preservable_boilerplate_target(...)` already form a correctness wall that this packet
   must keep intact rather than replace.
