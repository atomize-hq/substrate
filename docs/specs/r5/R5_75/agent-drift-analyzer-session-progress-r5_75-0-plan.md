# Plan: Agent Drift Analyzer Session Progress R5.75-0

Status: landed/closed on 2026-06-12. Original draft created from
`docs/specs/r5/R5_75/MAP.md` and
`docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`.

## Objective

Land a bounded docs/authority reconciliation packet that makes the `R5.5` and root landing-order
docs honest about what already landed, what remains, and why `R5.75` must complete before `R6`.

## Planning Decisions Locked For This Packet

1. `R5.75-0` is docs-only. It does not include analyzer-semantic edits, scorer retuning, fixture
   growth, or compactor changes.
2. The packet must preserve historical `R5.5` intent while adding present-tense landed-status and
   remaining-scope truth.
3. `R5.75-0` is the gate for all later `R5.75-*` implementation packets because stale authority
   would otherwise let future sessions reopen already-landed work.
4. Root landing-order docs must name `R5.75` as the current active family and keep `R6` closed.
5. If a secondary root-order doc is still actively naming the next family, it should be kept in
   sync in this packet rather than deferred.

## Scope Classification

- **In scope**
  - add landed-status / remaining-scope reconciliation language to the `R5.5` docs
  - update root landing-order authority to route remaining pre-`R6` work through `R5.75`
  - align `R6` gate wording with the new packet family
  - run manual authority review plus baseline analyzer validation
- **Out of scope**
  - changing analyzer logic
  - adding new fixture families
  - retuning sentinel/scorer behavior
  - broad refactors or new design seams

## Major Components And Dependencies

1. **`R5.5` plan reconciliation**
   - dependency: the new `R5.75` map already exists
   - outcome: current status language stops implying that all original `R5.5` packets are still
     open
2. **`R5.5` task-ledger reconciliation**
   - dependency: the packet’s landed vs remaining truth is settled in the plan language
   - outcome: tasks distinguish landed/history from remaining `R5.75` follow-up work
3. **Root landing-order synchronization**
   - dependency: `R5.5` docs now name `R5.75` as the remaining family
   - outcome: top-level authority agrees that `R6` is still gated by `R5.75`
4. **Packet closeout validation**
   - dependency: all doc edits are complete
   - outcome: manual authority audit and baseline test proof before promotion

## Implementation Order

1. Reconcile `R5.5` plan status language first.
2. Reconcile `R5.5` task-ledger language second.
3. Update root landing-order authority third.
4. Perform cross-doc audit and baseline analyzer validation last.

This order matters because the root landing-order doc should be updated only after the `R5.5` docs
already state the same remaining-family truth.

## Risks And Mitigations

### Risk 1: Overwriting history instead of reconciling it

- Risk: the edits could make it look like `R5.5` never contained implementation packets.
- Mitigation: add explicit “landed status” language instead of deleting all historical packet
  references.

### Risk 2: Conflicting next-family wording remains in multiple docs

- Risk: one doc says `R5.75` is next while another still implies `R5.5` or `R6`.
- Mitigation: run a final `rg` audit across all touched authority docs before closing.

### Risk 3: Scope creep into analyzer semantics

- Risk: the packet starts trying to fix code while reconciling the docs.
- Mitigation: keep the files list doc-only and treat any analyzer change as a separate later
  packet.

## Verification Checkpoints

### Checkpoint A: R5.5 plan reconciliation complete

Pass when:

- the `R5.5` plan has honest landed-status / remaining-scope language
- no new contradiction is introduced about whether `R6` is open

### Checkpoint B: R5.5 task ledger reconciled

Pass when:

- landed packets are not still presented as open unchecked implementation work
- remaining work is routed into the `R5.75` family

### Checkpoint C: Root landing-order synchronized

Pass when:

- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` names `R5.75` as the active pre-`R6` family
- any touched companion root-order doc says the same thing

### Checkpoint D: Promotion validation complete

Pass when:

- cross-doc `rg` audit is clean
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- optional native control smoke is either green or explicitly recorded as intentionally skipped

## Packet Split

`R5.75-0` stays a single packet, but it has four sequential work blocks:

1. `R5.75-0A` — reconcile `R5.5` plan status
2. `R5.75-0B` — reconcile `R5.5` task ledger
3. `R5.75-0C` — synchronize root landing-order authority
4. `R5.75-0D` — perform cross-doc validation and promotion audit

These are sequencing aids inside one packet, not separate promotion-worthy packets.

## Files Expected To Change

```text
docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md
docs/specs/hybrid-drift-sentinel-implementation-order.md   # only if it still names the active next family
```

## Promotion Gate To R5.75-1

Do not promote to `R5.75-1` until:

- touched authority docs agree on landed `R5.5` work vs remaining `R5.75` work
- no touched authority doc presents `R6` as currently open
- baseline analyzer validation is green
- any optional skipped smoke is explicitly recorded rather than silently omitted

## Packet Closeout Note (2026-06-12)

`R5.75-0` landed as a docs-only packet once the following closeout evidence was recorded:

- cross-doc authority audit run with
  `rg -n "R5\\.5|R5\\.75|R6" docs/specs/r5/R5_75/MAP.md docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- optional native control smoke intentionally skipped because `R5.75-0` is docs-only and did not
  change analyzer/runtime behavior
