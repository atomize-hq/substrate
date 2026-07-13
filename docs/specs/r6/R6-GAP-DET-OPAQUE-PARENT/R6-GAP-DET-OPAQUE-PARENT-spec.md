# R6-GAP-DET-OPAQUE-PARENT — Opaque-Parent Confidence Gap

Status: **ACTIVE — PACKET DOCS THEN LEDGER-RECONCILIATION GATES**. `R6-C.1-CONTROLS` is complete and this is the sole active named gap. No witness rerun, production edit, proof receipt, or successor work is authorized until these three packet docs are committed and fresh-review-clean **and** the separate post-docs ledger reconciliation below is committed and fresh-review-clean.

## Objective And Preserved Witness

Close only `CTX-R6-04`, preserved by commit `87409b39a`, without weakening its assertion or introducing R7 child-link behavior.

Exact control:

`dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`

Locked input and result:

- opaque delegating parent with visible `spawn_agent` / `wait_agent` orchestration but no attributable child command activity;
- no repeated-failure or repeated-verification history and both active repetition bits false;
- upstream `SessionProgress.dimension == ParentVisibleOrchestration`;
- required `dead_end_thrash` result: `0 / Low / Cleared`, unflagged, empty evidence.

The preserved red reached `ParentVisibleOrchestration` and returned `0 / Medium / Cleared`, unflagged, empty evidence. This is a confidence-only gap. Parent orchestration commands must not become evidence of child thrash.

## Owning Seam And Boundary

`score_dead_end_thrash` in `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs` owns the disposition. Its local confidence derivation currently observes command history. The directly relevant upstream context is `build_session_progress` and its `parent_visible_orchestration_progress` branch in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`; that branch already constructs the `ParentVisibleOrchestration` confidence seam and is read-only context for this packet.

Default production boundary: change only `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`. Do not edit progress construction, delegation inference, unrelated scorers, fixtures, exports, R7, or sentinel presentation. If the scorer-local boundary cannot satisfy the locked control without an upstream edit or a changed product meaning, stop under the escalation contract rather than widening silently.

Before any indexed-symbol edit, run:

```bash
npx gitnexus impact score_dead_end_thrash -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_confidence -r 97a0-substrate --direction upstream --depth 3
```

Run the second command only when editing that helper, and run the same exact impact form for any other symbol proposed for edit. `build_session_progress` and `parent_visible_orchestration_progress` remain read-only; if an upstream edit is proposed, impact that exact symbol and treat the packet boundary as invalid pending escalation. Warn and stop on HIGH or CRITICAL impact.

## Allowed Files By Atomic Batch

- Docs gate: exactly these three canonical packet docs.
- Post-docs ledger-reconciliation gate: exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`, in a separate orchestration batch after the packet docs are fresh-review-clean. Replace the active named-gap row's three `TO CREATE` markers with the actual packet paths, record the review-clean packet-docs commit, and reconcile only ledger-local current-status/next-action wording needed to make witness reconfirmation next without changing the preserved-red disposition. Commit this batch and obtain a fresh built-in `default` review; review fixes remain limited to that ledger. No witness or production work may share or precede this batch.
- Production-fix batch: `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`; `crates/agent-drift-analyzer/tests/dead_end_thrash.rs` only for an additional narrow regression that does not alter the preserved witness; this TASKS file; and `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md` for actual `CTX-R6-04` and named-gap evidence.
- No no-code receipt batch is allowed for this first named gap.
- Review fixes: only files already allowed for the batch under review.
- Transition: the active-phase mirrors in the landed R6-C.1 transition manifest, including this packet, with no production file and no successor packet file.

Unrelated dirty files stay unstaged.

## Acceptance

### Production-fix path

Use this path only while the exact witness remains red. The smallest scorer-local change must make the locked tuple pass while preserving `ParentVisibleOrchestration`, empty evidence, and no child misconduct claim. The existing `CTX-R6-03` regressing-frontier pattern must remain `30 / Medium / Active`, flagged, and the complete `dead_end_thrash` family plus affected checkpoint wall must pass. Do not weaken the witness or manufacture child activity.

### No-code path

No-code closure is available only to a **later** named gap when an already-landed fix commit from an earlier named gap in the prescribed sequence demonstrably made that later gap's preserved witness green. An unrelated earlier commit is never a valid receipt basis. Because this packet is the first named gap, it has no eligible earlier sequential named-gap commit and the no-code path is unavailable. If its exact witness is unexpectedly green before a packet production edit, preserve the exact output and stop under the escalation contract for authority reconciliation; do not close this gap from any unrelated already-landed commit.

## Exact Verification

Run focused proof before the family wall:

```bash
cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

These are the exact R6-C.1 focused, owning-family, and affected-checkpoint commands. Replay and broader scorer walls are not part of this gap.

## Commit, Review, And Exit

Before every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit each batch atomically. Dispatch a fresh built-in `default` reviewer after the packet docs, ledger reconciliation, production fix, every review-fix, and transition boundary; fixes use new commits and fresh reviewers until `REVIEW CLEAN`.

The gap exits only after its packet docs, post-docs ledger reconciliation, and production fix are separately committed, exact verification is recorded, and every required boundary is fresh-review-clean. No-code receipt cannot close this first named gap. A separate narrow transition commit then marks `R6-GAP-DET-OPAQUE-PARENT` complete and activates `R6-GAP-TGG-TRUTH-PATH-ACTION` at its docs-only gate. Record that successor's three canonical paths as `TO CREATE`; do not reference a successor TASKS file, create its docs, or execute it in this phase. Stop after the transition commit is independently review-clean.
