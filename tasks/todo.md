# Active Tasks: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-REPLAY` (ACTIVE; active packet: `R6-GAP-DET-REPLAY-STALL`; trusted `CTX-R6-02` behavioral-RED witness `60cde3dd7`; Task `.0` series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` fresh independent `REVIEW CLEAN`; decision pending)**

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
- [x] Authority transition series `56bb9966f` + `07a3b1fe5` received fresh independent built-in
  `default` `REVIEW CLEAN`.
- [x] Complete `CTX-R6-01`; implementation/fix series `a0089c8de` + `968a4377f` received fresh
  independent built-in `default` `REVIEW CLEAN`.
- [x] Select trusted depth-1 built-in `default` subagent rollout
  `019eb311-c7ce-7f50-ae13-b51a5b5461c3` under the canonical selected-checkpoint contract and
  preserve `CTX-R6-02` behavioral RED at witness commit `60cde3dd7`.
- [x] Land packet docs-gate commit `200725001` for active packet `R6-GAP-DET-REPLAY-STALL`; its
  fresh review returned `REVIEW FINDINGS`, so this is not a review-clean gate receipt.
- [x] Land first authority/status correction `08fa86e94`; this final status correction completes the
  current review-fix candidate without claiming the series review-clean.
- [x] Complete packet Task `.0`; full docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`.
- [ ] **Current decision gate:** DECISION REQUIRED `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE`; do not edit Rust before explicit operator acceptance.
- [ ] Resolve `R6-GAP-DET-REPLAY-STALL` only after that decision, then run `CTX-R6-06`, focused
  scorer proof, full analyzer proof, and the bounded replay family wall.
- [ ] Update the R6 finding and authority stack to `CLOSED` after proof is complete.
- [ ] Promote R7 from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready.
- [ ] Begin bounded direct-child delegated-session support only after promotion.

The R7 task ledger is intentionally preserved under `docs/specs/r7/`, but none of its implementation
items are active while this ledger remains partial.
