# Plan: Agent Drift Analyzer Delegated Parent-Visible Stabilization (R5.75-3)

Status: draft plan created on 2026-06-22 from `docs/specs/r5/R5_75/MAP.md`, the delegated
`progress_acceptance` corpus contract, and the adjacent `R5.75` packet patterns.

## Objective

Land a narrow delegated-parent stabilization packet so parent-visible orchestration survives planning
artifact edits and limited child visibility, then convert that live behavior into one committed native
delegated real-rollout proof plus the named smoke closeout evidence.

## Planning Decisions Locked For This Packet

1. The packet stays local to delegated-parent logic in
   `crates/agent-drift-analyzer/src/checkpoint/progress.rs` plus regressions in
   `crates/agent-drift-analyzer/tests/checkpoints.rs` and
   `crates/agent-drift-analyzer/tests/progress_acceptance.rs`.
2. Native rollout behavior is the primary correctness authority. The adapted delegated session
   `da59436e63915185` is a required smoke witness for this packet, but adapted committed fixture-family
   work remains deferred to `R5.75-5`.
3. `019eb970-3543-7ab1-a5d6-2a62c00c7185` is the default first native delegated real-rollout fixture to
   promote into `progress_acceptance`, because the MAP already treats it as the positive proof that the
   parent-visible path works. `019eb907-95c4-73e1-843e-e337d1e93cb9` and
   `019eb917-9531-74e0-897d-ad8d362138ec` remain the instability witnesses the packet must stabilize in
   smoke.
4. Per-session acceptance should be explicit. The implementation plan therefore starts with a
   characterization step that locks exact status/confidence/evidence expectations for the four named
   sessions before closeout.
5. Planning/spec/handoff edits must stop acting as an automatic ejector seat from the parent-visible
   path. They should count as parent-visible synthesis when delegation markers are otherwise strong.
6. One narrow `parent_visible_comparability_fingerprint(...)` tweak is allowed inside this packet only
   if, after the planning-artifact stabilization lands, the remaining failures reduce to comparability
   resets being too broad on the named repros.
7. Packet-prompts are intentionally out of scope for this docs pass because the user asked for the
   spec/plan/tasks triplet only.

## Scope Classification

- **In scope**
  - characterize the four named delegated sessions and lock packet-local expectations
  - stop dropping parent-visible orchestration solely because the parent edited a planning artifact
  - preserve conservative parent-visible interpretation under opaque/partial child visibility
  - promote one native delegated real-rollout fixture into committed `progress_acceptance` coverage
  - add/update checkpoint regressions for delegated-parent stability
  - apply one narrow comparability-reset tweak only if the named repros prove it is still needed
  - run the named native/adapted manual smoke checks and the shared analyzer gates
- **Out of scope**
  - generalized parent-visible fingerprint redesign
  - structured-state comparability migration (`comparison_key` consumer wiring)
  - public schema or replay contract changes
  - compactor/export changes
  - adapted committed fixture-family expansion (`R5.75-5`)
  - zero-verifier anti-flap tuning beyond the explicit packet boundary (`R5.75-4`)
  - packet-prompts in this docs pass

## Major Components And Dependencies

1. **Characterization of named delegated repros**
   - dependency: packet scope from `R5.75/MAP.md`
   - outcome: exact acceptance expectations are written down for the three native sessions plus the
     adapted delegated witness

2. **Parent-visible stabilization for planning artifact edits**
   - dependency: characterization clarifies the current failure shape
   - outcome: `progress.rs` no longer returns to generic planning solely because a delegated parent
     refined a plan/spec/handoff artifact

3. **Committed delegated real-rollout fixture promotion**
   - dependency: stable behavior exists for the chosen native proof case
   - outcome: one native delegated real case enters the bounded `progress_acceptance` corpus, with
     matching README / expected-json / corpus-count updates

4. **Checkpoint regression wall**
   - dependency: the intended live behavior is defined
   - outcome: fast delegated-parent tests prove the planning-artifact stabilization and any narrow
     comparability logic stay bounded and conservative

5. **Conditional comparability follow-on**
   - dependency: post-stabilization smoke still fails only because resets are too broad
   - outcome: at most one narrow tweak lands in `parent_visible_comparability_fingerprint(...)`; if the
     problem is broader, the packet stops and opens a new follow-on

6. **Packet closeout validation**
   - dependency: code/tests/fixture updates complete
   - outcome: shared analyzer gates plus named native/adapted smoke prove the packet is genuinely ready
     for promotion to `R5.75-4`

## Implementation Order

1. Commit the docs triplet so the packet boundary is explicit before any code changes.
2. Characterize the named delegated sessions and record exact expected lane/status/confidence/evidence
   outcomes in the tasks ledger.
3. Land the narrow planning/spec/handoff stabilization in `progress.rs`.
4. Add/refresh fast delegated-parent regressions in `tests/checkpoints.rs`.
5. Promote the first native delegated real-rollout case into committed `progress_acceptance` coverage
   and update corpus-count/exclusion rules.
6. Rerun the named repros; only if the remaining failure is specifically over-broad parent-visible
   comparability reset behavior, apply one narrow comparability tweak.
7. Run the automated gates, then the named native/adapted smoke review, then update packet routing
   status only after the manual gate is satisfied.

This order matters because the packet should first prove what “correct” looks like, then make the
behavior deterministic enough to encode in both fast regressions and the committed semantic corpus.

## Risks And Mitigations

### Risk 1: planning-artifact stabilization overclaims child progress

- Risk: treating plan/spec/handoff edits as synthesis evidence could accidentally make opaque-child
  delegated sessions look like direct execution progress.
- Mitigation: keep child-visibility limits explicit and conservative; the packet may preserve
  `ParentVisibleOrchestration`, but it must not fabricate positive opaque-child progress. Fast
  regressions and the committed fixture contract should assert that delegated cases remain
  guardrail-only in `R5`.

### Risk 2: the first promoted delegated real fixture is too easy and leaves the unstable repros
unresolved

- Risk: promoting only the already-positive `019eb970-3543-7ab1-a5d6-2a62c00c7185` could make the
  semantic wall look better while `019eb907-95c4-73e1-843e-e337d1e93cb9` and
  `019eb917-9531-74e0-897d-ad8d362138ec` still collapse in smoke.
- Mitigation: keep the unstable native repros as named smoke gates, and require explicit
  session-by-session expectations for them in the characterization step before closing the packet.

### Risk 3: the packet silently widens into a fingerprint redesign project

- Risk: once comparability-reset behavior appears, the fix could balloon into a broader redesign.
- Mitigation: require the narrow planning-artifact stabilization first; allow a comparability tweak only
  if the remaining failure is narrowly isolated to reset behavior on the named repros. Anything broader
  opens a new packet.

### Risk 4: the delegated corpus contract drifts

- Risk: adding a real delegated case could accidentally loosen the bounded semantic wall or remove the
  synthetic guardrail proof without a clear replacement.
- Mitigation: update the README, `progress_acceptance.rs`, and case directories together; keep exact
  file-count and explicit-exclusion assertions intact; retain the synthetic delegated guardrail case
  unless there is a documented reason to replace it.

## Verification Checkpoints

### Checkpoint A: Characterization is explicit

Pass when:

- the tasks ledger records exact expected lane/status/confidence/evidence outcomes for
  `019eb907-95c4-73e1-843e-e337d1e93cb9`,
  `019eb917-9531-74e0-897d-ad8d362138ec`,
  `019eb970-3543-7ab1-a5d6-2a62c00c7185`,
  and `da59436e63915185`
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` is confirmed as the first native delegated real-rollout case
  to promote unless the characterization proves otherwise

### Checkpoint B: Planning-artifact stabilization lands

Pass when:

- parent-visible classification no longer disappears merely because a delegated parent edited a
  plan/spec/handoff artifact
- child-visibility-limited cases still surface their limiting evidence

### Checkpoint C: Committed delegated proof exists

Pass when:

- the bounded `progress_acceptance` corpus includes one native delegated real-rollout case
- README / corpus-count / exclusion rules are updated coherently
- delegated cases remain guardrail-only in the corpus contract

### Checkpoint D: Conditional comparability branch stays narrow

Pass when:

- either no comparability tweak was needed, or
- one additive tweak in `parent_visible_comparability_fingerprint(...)` was enough, and it stayed on
  the legacy objective surface with no broader redesign

### Checkpoint E: Automated gates are green

Pass when:

- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
- `cargo test -p agent-drift-analyzer -- --nocapture` is green

### Checkpoint F: Named smoke proves live correctness

Pass when:

- native sessions
  `019eb907-95c4-73e1-843e-e337d1e93cb9`,
  `019eb917-9531-74e0-897d-ad8d362138ec`,
  `019eb970-3543-7ab1-a5d6-2a62c00c7185`
  all hold the packet-local expectations recorded in Checkpoint A
- adapted session `da59436e63915185` holds the same packet-owned parent-visible stability bar without
  overclaiming child progress

## Parallelism And Sequencing

- **Must stay sequential**
  - characterization before final acceptance wording
  - planning-artifact stabilization before any comparability fallback
  - committed delegated-fixture promotion after stable behavior exists for the chosen case
  - smoke review after automated gates are already green
- **Can happen together inside one edit cycle**
  - `progress.rs` stabilization and synthetic checkpoint regression updates
  - delegated fixture directory work, README updates, and `progress_acceptance.rs` corpus-count changes

## Packet Split

`R5.75-3` remains one packet with six internal work blocks:

1. `R5.75-3A` — docs lock + characterization
2. `R5.75-3B` — planning/spec/handoff stabilization in `progress.rs`
3. `R5.75-3C` — checkpoint regression updates
4. `R5.75-3D` — committed native delegated real-rollout fixture promotion
5. `R5.75-3E` — conditional comparability follow-on (only if needed)
6. `R5.75-3F` — automated gates + native/adapted smoke review

These are sequencing aids only, not separate promotion packets.

## Files Expected To Change

```text
docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md
docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md
docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
crates/agent-drift-analyzer/tests/progress_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<delegated-case-id>/
```

## Hard Gate For R5.75-3

Do not treat this packet as landed unless:

- the parent-visible path survives planning/spec/handoff edits when delegation evidence is strong,
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` becomes a committed delegated real-rollout proof in
  `progress_acceptance`,
- the unstable native repros
  `019eb907-95c4-73e1-843e-e337d1e93cb9` and `019eb917-9531-74e0-897d-ad8d362138ec`
  no longer collapse into generic planning-only noise,
- adapted session `da59436e63915185` holds the packet-local delegated-parent bar in manual smoke,
- all three analyzer gates are green, and
- the packet stayed narrow: no structured-state comparability migration, no generalized fingerprint
  redesign, no adapted committed fixture-family expansion.

Automated green status alone does not satisfy this packet.

## Promotion Gate To R5.75-4

Do not promote to `R5.75-4` until every task in the companion tasks ledger is complete and the named
native/adapted smoke sessions all hold a conservative but stable parent-visible interpretation that
matches the recorded characterization for this packet.
