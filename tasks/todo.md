# Active Tasks: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

- [x] Correct the claim that R6 is closed for sequencing.
- [x] Inventory every scoring module and classify context applicability.
- [x] Map existing behavioral tests and bounded replay evidence to the broad R6 claims.
- [x] Record `semantic_goal_drift` as cutover complete by design absent new failing evidence.
- [x] Preserve the R7 MAP/SPEC/PLAN/TASKS as design-ready drafts.
- [ ] Write the bounded `R6-C.1 — Scorer Context Applicability Acceptance Controls` packet.
- [ ] Add the exact `dead_end_thrash`, `truth_grounding_gap`, and `wrong_plan_branch` controls named
  in the closure finding.
- [ ] If and only if a control fails, open and land the smallest scorer-specific R6 implementation
  gap packet.
- [ ] Re-run focused scorer tests, full analyzer tests, and the bounded replay wall.
- [ ] Update the R6 finding and authority stack to `CLOSED` after proof is complete.
- [ ] Promote R7 from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready.
- [ ] Begin bounded direct-child delegated-session support only after promotion.

The R7 task ledger is intentionally preserved under `docs/specs/r7/`, but none of its implementation
items are active while this ledger remains partial.
