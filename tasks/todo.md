# Active Tasks: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-REPLAY` (ACTIVE; active packet: `none`; authority transition landed; fresh transition review pending before replay execution)**

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
- [x] Complete the authority-only transition at `2937dbe5a` + `91f55f6bf`; the first review found
  one P2 stale R6-C.1 source-state issue, the fix reconciled it, and a fresh independent built-in
  `default` series reviewer returned `REVIEW CLEAN` with the original `CTX-R6-12` exact control `1 /
  1` green, diff check clean, and all three successor files absent/non-link.
- [x] Create and freshly review the active packet's canonical
  [`SPEC`](../docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md),
  [`PLAN`](../docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md), and
  [`TASKS`](../docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md);
  packet-doc series `8734f4dbe` + `334e7c6ac` received fresh independent built-in `default`
  `REVIEW CLEAN`.
- [x] Resolve `R6-GAP-WPB-EMPTY-AUTHORITY`; implementation/review-fix series `6b42e5476` +
  `e65df2561` + `cd4e24119` received fresh independent built-in `default` `REVIEW CLEAN` with exact,
  protected, family, checkpoint, full-analyzer, and static proof green.
- [ ] **Next eligible interaction:** fresh independent review of the landed authority transition.
  Only after that review is clean may Prompt 1 start `R6-REPLAY` with active packet `none`.
- [ ] Re-run focused scorer tests, full analyzer tests, and the bounded replay wall.
- [ ] Update the R6 finding and authority stack to `CLOSED` after proof is complete.
- [ ] Promote R7 from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready.
- [ ] Begin bounded direct-child delegated-session support only after promotion.

The R7 task ledger is intentionally preserved under `docs/specs/r7/`, but none of its implementation
items are active while this ledger remains partial.
