# R6-3.X.2C PLAN — Relation Routing Contract And Suppression Guardrails For Semantic Goal Drift

Companion to
`agent-drift-analyzer-semantic-goal-drift-relation-routing-contract-and-suppression-guardrails-spec.md`.
This remains an analyzer-local, precision-first follow-up packet. Execution order:
docs reopen → routing contract → suppressive-family guardrails → coverage expansion → rerun → routing-doc
reconciliation.

## Live repo truth this plan assumes

- `R6-3.5` target hygiene is already landed and remains authoritative.
- `R6-3.X.2` containment first cut is already landed and must remain intact.
- `R6-3.X.2B` landed useful relation machinery, but did not settle the final routing contract.
- The strongest confirmed issue is the relation-only routing mismatch.
- `R6-3.X.3` stays deferred unless this packet's rerun proves a new blocker.

## Dependency graph

1. **Authority reconciliation**
   - `FINDINGS`, `MAP`, `R6-3` ledger, packet `R6-3.X.2B`, live scorer, acceptance harness.
2. **Routing contract**
   - explicit decision surface replacing `claims_drift()`;
   - relation-authoritative candidate routing, confidence/evidence-gated suppressions, explanatory-only score.
3. **Suppressive-family guardrails**
   - tighten `SameArtifactFamily`, `SameWorkItemFamily`, doc-bundle handling, and role classification order.
4. **Proof wall**
   - scorer-local false-negative guards, acceptance fixtures, prior-witness preservation.
5. **Rerun and routing truth**
   - focused tests, full analyzer wall, default seed-42 corpus rerun, findings/map/ledger reconciliation.

## Vertical task sequence

1. **Docs reopen + packet framing**
   - Create `R6-3.X.2C` packet docs.
   - Record that `R6-3.X.2B` landed but did not settle the final routing contract.
   - Lock the analyzer-local boundary and explicit defers.

2. **Routing contract slice**
   - Define the decision contract (`Suppress` / `Fire` / `NoClaim`).
   - Replace `claims_drift()` callers with the explicit routing helper.
   - Keep the relation taxonomy authoritative and score explanatory only.

3. **Guardrail slice**
   - Tighten suppressive-family semantics where the external review found legitimate risk.
   - Preserve the landed doc-bundle fix but expand it into a clearly specified relation rule and coverage set.
   - Fix role classification ordering so path/extension semantics beat substring hints.

4. **Proof-wall slice**
   - Add scorer-local false-negative guards for same-artifact, same-work-item, doc-bundle, and role-ordering.
   - Add at least one live acceptance case for doc-bundle member behavior and any newly covered guard case
     that is user-visible at analyzer output level.
   - Preserve prior witnesses without rebaseline.
   - `R6-3.X.2C.2.1` through `R6-3.X.2C.2.4` may be implemented as independent sub-slices after the routing
     contract lands; the listed order is the preferred review order, not a semantic dependency unless a
     later task explicitly consumes an earlier helper.

5. **Rerun + closeout slice**
   - Run the focused wall, the full analyzer wall, and the seed-42 corpus rerun by default unless explicitly
     waived with an honest confidence-loss note.
   - Update `FINDINGS`, `MAP`, and the `R6-3` ledger with the honest packet status and defer decisions.
   - Soften unknown-strata wording if the tooling still supports only a coarse gate.

## Checkpoints

### Checkpoint A — reopen framing locked
- packet docs exist
- analyzer-local boundary is explicit
- `R6-3.X.2B` vs `R6-3.X.2C` status is explicit
- `R6-3.X.3` defer remains explicit

### Checkpoint B — routing contract settled
- `claims_drift()` is replaced in the plan/design
- decision outcomes are explicit
- relation remains authoritative
- score is explanatory only

### Checkpoint C — suppressive-family guardrails settled
- `SameArtifactFamily` and `SameWorkItemFamily` gating rules are explicit
- doc-bundle-member coverage requirements are explicit
- role classification ordering is explicit

### Checkpoint D — proof wall defined
- every touched suppressive family has a paired false-negative guard
- acceptance-level doc-bundle coverage is explicit
- prior-witness non-regression is explicit

### Checkpoint E — closeout criteria locked
- focused tests and full analyzer wall are explicit
- default corpus rerun requirement and waiver rule are explicit
- findings/map/ledger reopen is explicit
- unknown-strata wording follow-up is explicit

## Verification wall

Focused scorer/analyzer wall:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full analyzer closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Default corpus rerun:

```bash
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --seed 42 --out /tmp/r6_3_x_2c_selected.jsonl
python3 scripts/dev/drift-batch-scan/run_batch.py --repo "$PWD" --selected /tmp/r6_3_x_2c_selected.jsonl --batch-dir /tmp/r6_3_x_2c_batch
python3 scripts/dev/drift-batch-scan/tabulate.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
python3 scripts/dev/drift-batch-scan/inspect_targets.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
python3 scripts/dev/drift-batch-scan/filter_junk.py --checkpoints-dir /tmp/r6_3_x_2c_batch/checkpoints
```

## Risks and mitigations

1. **Routing contract fixes drift into score-threshold fuzziness**
   - Risk: the patch "uses evidence" by making score bands primary.
   - Mitigation: require relation-authoritative routing and explanatory-only score in code/tests/docs.

2. **Guardrails over-tighten and silently reintroduce under-fire**
   - Risk: suppressive families become too strict and regress legitimate progression handling.
   - Mitigation: keep existing negative controls and rerun the acceptance wall plus the default corpus proof.

3. **Coverage expands only at scorer-local layer**
   - Risk: doc-bundle and family behavior look good in unit tests but not through the live analyzer path.
   - Mitigation: require at least one acceptance fixture for doc-bundle behavior and preserve bundle-shaped
     analyzer proof.

4. **Docs stay inconsistent after code lands**
   - Risk: `FINDINGS` / `MAP` / ledger continue to say `R6-3.X.2B` fully closed the routing story.
   - Mitigation: make routing-doc reconciliation a packet-complete checkpoint.

5. **2C accidentally widens into `R6-3.X.3`**
   - Risk: once touching shared helpers, the packet drifts into eligibility changes.
   - Mitigation: keep `eligible_current_goal(...)` frozen and repeat the defer rule in spec, plan, and tasks.

## Locked decisions

- The seed-42 corpus rerun remains default-required for this packet. It may be waived only with an explicit
  confidence-loss note in `FINDINGS`, `MAP`, and the `R6-3` ledger.
- No suppressive family besides `Exact` and `StructuralContainment` may suppress with Low confidence.
  Low-confidence `SameArtifactFamily`, `SameWorkItemFamily`, `SameDocFamily`, `PlanCodeRoleShift`, and
  `ReviewFixVerifyRoleShift` must resolve to `NoClaim` or be reclassified as `Unrelated`,
  `SharedConstraintOnly`, or `WeakOrGenericOnly` when evidence is inadequate.
