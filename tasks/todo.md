# Active Tasks: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-C.1-CONTROLS` (ACTIVE; active packet: none)**

- [x] Correct the claim that R6 is closed for sequencing.
- [x] Inventory every scoring module and classify context applicability.
- [x] Map existing behavioral tests and bounded replay evidence to the broad R6 claims.
- [x] Record `semantic_goal_drift` as cutover complete by design absent new failing evidence.
- [x] Preserve the R7 MAP/SPEC/PLAN/TASKS as design-ready drafts.
- [x] Bootstrap the R6-R8 execution context pack without making it semantic authority.
- [x] Complete `R6-C.0A`: remediate all seven fresh closure-matrix findings in canonical authority
  at `d3dcda785` with fresh review clean.
- [x] Write and review the bounded `R6-C.1 — Scorer Context Applicability Acceptance Controls`
  SPEC/PLAN/TASKS; the artifact series through `ea19b39a7` received final fresh `REVIEW CLEAN`.
- [ ] Add the exact `dead_end_thrash`, `truth_grounding_gap`, and `wrong_plan_branch` controls named
  in the landed packet docs. The sole next action is the first row-atomic `dead_end_thrash` control;
  no production change is authorized first.
- [ ] If and only if a control fails, open and land the smallest scorer-specific R6 implementation
  gap packet.
- [ ] Re-run focused scorer tests, full analyzer tests, and the bounded replay wall.
- [ ] Update the R6 finding and authority stack to `CLOSED` after proof is complete.
- [ ] Promote R7 from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready.
- [ ] Begin bounded direct-child delegated-session support only after promotion.

The R7 task ledger is intentionally preserved under `docs/specs/r7/`, but none of its implementation
items are active while this ledger remains partial.
