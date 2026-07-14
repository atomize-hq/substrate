# Active Tasks: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-GAP-WPB-EMPTY-AUTHORITY` (ACTIVE; active packet: none; docs-only packet-creation gate; transition review pending)**

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
- [x] Add, atomically commit, and freshly review all thirteen `dead_end_thrash`,
  `truth_grounding_gap`, and `wrong_plan_branch` controls; the wall receipt `5618f7864` records
  `10 PASS / 3 preserved RED` with no production change.
- [x] Resolve `R6-GAP-DET-OPAQUE-PARENT`; production series `bcd94bf4f` + `931e50c85` +
  `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN` with its focused, family,
  checkpoint, and format/check proof green.
- [x] Resolve `R6-GAP-TGG-TRUTH-PATH-ACTION`; final proof-receipt series `fee9c2b16` + `6674a8316`
  received fresh independent built-in `default` `REVIEW CLEAN` with exact focused, family,
  checkpoint, format, check, literal-clippy, and diff proof green.
- [ ] Obtain fresh independent review of the authority-only transition candidate; no successor work
  begins until it is `REVIEW CLEAN`.
- [ ] **Next eligible action after transition review clean:** atomically create and freshly review
  exactly these three non-link `TO CREATE` paths; they do not yet exist, and no successor execution
  begins first:
  - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md`
  - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md`
  - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md`
- [ ] Resolve active `R6-GAP-WPB-EMPTY-AUTHORITY`; do not activate replay early.
- [ ] Re-run focused scorer tests, full analyzer tests, and the bounded replay wall.
- [ ] Update the R6 finding and authority stack to `CLOSED` after proof is complete.
- [ ] Promote R7 from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready.
- [ ] Begin bounded direct-child delegated-session support only after promotion.

The R7 task ledger is intentionally preserved under `docs/specs/r7/`, but none of its implementation
items are active while this ledger remains partial.
