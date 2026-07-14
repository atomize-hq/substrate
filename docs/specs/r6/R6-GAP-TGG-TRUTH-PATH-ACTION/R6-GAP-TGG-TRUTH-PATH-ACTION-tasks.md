# Tasks: R6-GAP-TGG-TRUTH-PATH-ACTION

Status: **COMPLETE — IMPLEMENTATION/PROOF AND TRANSITION REVIEW CLEAN**. The operator decisions `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A` and `R6-TGG-CLIPPY-SCOPE-01 = A` are fully exercised: boundary series `2067149ce` + `023417c03`, Option-A implementation series `4ba9f2647` + `1f6e863bf` + `9565fb805`, earlier proof/blocker receipt series `a341066a3` + `0da262979`, the one-expression source commit `5622ddb73`, and final proof-receipt series `fee9c2b16` + `6674a8316` all received fresh independent built-in `default` `REVIEW CLEAN`. Authority-only transition series `2937dbe5a` + `91f55f6bf` also received fresh independent built-in `default` `REVIEW CLEAN`, completing Tasks 3A, 4, and 5. Only `R6-GAP-WPB-EMPTY-AUTHORITY` is active at its docs-only packet-creation gate with active packet `none`; `R6-REPLAY` remains blocked, and no successor packet was created or executed.

## Required Gates

Staged gate for every later commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Exact indexed-symbol gates after Tasks 2A and 2B are review-clean and before source edits:

```bash
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_session -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact analyze_loaded_bundle -r 97a0-substrate --direction upstream --depth 3
```

If any current scorer helper is edited, run its own exact command first:

```bash
npx gitnexus impact historical_truth_grounding_gap_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact Function:crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs:dedupe_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact first_event_index -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact is_historical_truth_grounding_gap_reason -r 97a0-substrate --direction upstream --depth 3
```

Record a separate literal pre-edit command/result for every other existing helper proposed for edit. Record genuinely new private helpers as absent from the pre-edit index; do not fabricate impact output. Stop and warn on HIGH/CRITICAL.

## Task Ledger

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.0 — Land and review the original packet-docs gate.**
  - Result: `03754a2de` (`docs: lock truth path action gap packet`) received fresh built-in `default` `REVIEW CLEAN`.
  - Scope note: this approved the earlier scorer-only packet, not the Option-A seam selected later.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.1 — Reconcile the original packet into the canonical ledger.**
  - Result: `a5380c04e` (`docs: record truth path packet gate`) was ledger-only and received fresh built-in `default` `REVIEW CLEAN`.
  - Scope note: this historical gate predates and cannot substitute for the Option-A decision reconciliation in Task 2B.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.2 — Reconfirm the original witness and preserve the cross-checkpoint review finding.**
  - Original pre-edit result: `CTX-R6-12` was red at `0 / Medium / Cleared`, unflagged, versus required `80 / High / Active`, flagged; the analogous archetype control passed and the owning family was `9 passed; 1 failed`.
  - Incomplete production series: `52c9ab296` made the original control pass and `73132aead` recorded its then-green wall, but fresh review returned `REVIEW FINDINGS` P1 because score posture was being used as grounding and clean `Cleared` grounding was dropped.
  - Review-clean witness: `6409ae072` (`test: preserve cross-checkpoint grounding gaps`) changes only `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` and independently received fresh built-in `default` `REVIEW CLEAN`.
  - Preserved failures at that witness: `truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery` was actual `20 / Medium / HistoricalOnly`, unflagged, versus required `80 / High / Active`, flagged; `truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action` was actual `80 / High / Active`, flagged, versus required `0 / Medium / Cleared`, unflagged.
  - Receipt: current commit `48f259d25` (`docs: record grounding provenance review gap`) records the P1, the review-clean witness, the `10 passed; 2 failed` historical family result, and the need for an operator decision. It is a decision receipt, not proof or closure.
  - Decision: operator selected `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A` on 2026-07-13.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.2A — Commit and freshly review this Option-A packet amendment.**
  - Review-clean series: packet authority `49b2bbd7f` (`docs: authorize typed grounding provenance`) + `70c0ca9f4` (`docs: tighten grounding provenance gates`) and gate-state corrections `baa0983ac` (`docs: reconcile provenance packet review`) + `d6cbc9d2c` (`docs: correct provenance task blocker`) received fresh built-in `default` `REVIEW CLEAN`.
  - Reviewed scope: exactly this SPEC, PLAN, and TASKS; no other file.
  - Required amendment: replace docs-only/scorer-only implementation authority with the bounded typed, session-local, path-scoped seam across `score_truth_grounding_gap`, `score_session`, and `analyze_loaded_bundle` plus necessary private helpers, focused tests, TASKS, and the canonical ledger.
  - Verify: inspect only the three-file diff and run `git diff --check`; do not run scorer tests or edit implementation.
  - Commit/review: atomic packet-only commit; fresh built-in `default`; docs-only fixes in new commits and fresh reviewers until `REVIEW CLEAN`.
  - Completion record: complete at review-clean series `49b2bbd7f` plus `70c0ca9f4`; this is packet authority, not implementation proof.
  - Stop rule: Task 3A remains unauthorized until Task 2B is committed and fresh-review-clean.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.2B — Reconcile the Option-A decision in the canonical ledger.**
  - Prerequisite: Task 2A committed and fresh-review-clean.
  - Separate allowed batch: exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.
  - Review-clean series: `dba992383` (`docs: record grounding provenance decision`) plus ledger-only fix `70955db2a` (`docs: fix grounding decision ledger gate`) received fresh built-in `default` `REVIEW CLEAN` after the packet gate-state series `baa0983ac` + `d6cbc9d2c`.
  - Resolved review findings: the ledger now records the actual Task 2A gate state and keeps helper visibility private, allowing only the provenance type itself to become `pub(crate)` when cross-module signature threading requires it.
  - Record the explicit Option-A decision, review-clean witness `6409ae072`, decision receipt `48f259d25`, Task 2A's actual review-clean commit, the bounded internal seam, and Task 3A as next.
  - Preserve the gap as ACTIVE, Task 3A unchecked, Task 4 open, and transition/successor blocked. Make no future proof claim.
  - Commit/review: staged gate; ledger-only commit; fresh built-in `default`; ledger-only fixes and fresh reviewers until `REVIEW CLEAN`.
  - Completion record: complete at review-clean ledger series `dba992383` + `70955db2a`; `dba992383` alone was not completion.
  - Stop rule satisfied: no source or test edit preceded this separate review-clean gate.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.2C — Record clippy-scope decision A and freshly review the expanded boundary.**
  - Operator decision: `R6-TGG-CLIPPY-SCOPE-01 = A`, selected on 2026-07-14.
  - Allowed docs-only batch: exactly this TASKS and `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`; do not edit the packet SPEC/PLAN, source, or tests in this batch.
  - Expanded production boundary after this batch is fresh-review-clean: exactly one additional file, `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`, limited to the semantics-preserving `clippy::nonminimal_bool` simplification in `score_confidence`.
  - Forbidden: no lint allowance; no scoring-behavior, threshold, or evidence change; and no adjacent dead-end refactor.
  - Pre-edit GitNexus impact: exact `score_confidence` is LOW, with one direct caller (`score_dead_end_thrash`) and one affected process. No HIGH/CRITICAL risk was reported.
  - Commit/review: staged `detect-changes` gate; atomic TASKS/ledger commit; fresh independent built-in `default`; fixes only in a new TASKS/ledger commit followed by another fresh review until `REVIEW CLEAN`.
  - Stop rule: do not edit `dead_end_thrash.rs` before this docs series is committed and fresh-review-clean. After that gate, land the authorized one-expression source fix in a separate atomic commit containing only `dead_end_thrash.rs`.
  - Completion record: complete at boundary series `2067149ce` (`docs: authorize clippy scope fix`) + `023417c03` (`docs: reconcile clippy scope amendment`), which received fresh built-in `default` `REVIEW CLEAN`. This authorizes only the separate one-expression source fix and does not itself prove or close Task 3A.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.3A — Implement and prove Option-A typed provenance.**
  - Prerequisites: Tasks 2A, 2B, and 2C committed and fresh-review-clean; all required GitNexus impacts below HIGH; no unisolatable unrelated work.
  - Allowed files only:
    - `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs` — `score_truth_grounding_gap`, the provenance type (private unless cross-module signature threading requires `pub(crate)`), and necessary private helpers;
    - `crates/agent-drift-analyzer/src/scoring/mod.rs` — `score_session` and necessary private threading helpers;
    - `crates/agent-drift-analyzer/src/lib.rs` — `analyze_loaded_bundle` and necessary private per-session initialization/threading helpers;
    - `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs` — only the separately authorized semantics-preserving `clippy::nonminimal_bool` simplification in `score_confidence`; no lint allowance, behavior/threshold/evidence change, or adjacent refactor;
    - `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` — focused provenance behavior only;
    - this TASKS — actual commands/results/review state;
    - `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md` — actual named-gap evidence only.
  - Provenance source: qualifying typed read observations plus event order only. Never `DriftScore`, `raw_score`, `state`, `flagged`, historical posture, evidence presence, or evidence reason strings.
  - Path rule: carry only a matching truth path still declared in the current task frame; path A read never grounds path B action; dropping a path from the frame permanently drops that carried read, so later re-declaration requires a new qualifying read.
  - Session rule: initialize/reset per bundle session; never cross sessions or move from parent to child. Prove parent-to-child non-inheritance only through existing analyzer bundle/session fixtures; do not add or validate R7 topology.
  - Temporal rule: same-path read must carry across the next checkpoint, across multiple checkpoints, and after a prior same-path action while the path remains declared. This is proof against packet-authorized TTL/expiry/maximum-checkpoint/read-consumption behavior, not authority to introduce any such semantics.
  - Visibility rule: every new helper is private. The provenance type is private unless cross-module signature threading requires `pub(crate)`; no helper and no other new item may use `pub(crate)`.
  - Public boundary: no public API/schema/export/replay/sentinel/presentation change and no other scorer/context/classifier change.
  - Preserve focused controls:
    - original `truth_grounding_gap_flags_truth_path_action_before_read` remains exact `80 / High / Active`, flagged, with authority and action evidence;
    - historical-only recovery does not ground the later action;
    - real same-path read grounds the next-checkpoint action;
    - `truth_grounding_gap_does_not_ground_path_b_from_path_a_read` asserts the path-B action at `80 / High / Active`, flagged;
    - `truth_grounding_gap_does_not_resurrect_read_after_path_redeclaration` asserts the re-declared-path action at `80 / High / Active`, flagged after an intervening frame removed the path;
    - `truth_grounding_gap_does_not_carry_read_across_sessions` asserts the later session's same-path action at `80 / High / Active`, flagged;
    - `truth_grounding_gap_does_not_inherit_parent_read_in_child_session` asserts the child session's same-path action at `80 / High / Active`, flagged;
    - `truth_grounding_gap_carries_clean_read_across_multiple_checkpoints` asserts the later same-path action at `0 / Medium / Cleared`, unflagged;
    - `truth_grounding_gap_retains_clean_read_after_prior_same_path_action` asserts the later same-path action at `0 / Medium / Cleared`, unflagged.
  - Run focused before family/checkpoint/static wall:

    ```bash
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_ground_path_b_from_path_a_read -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_resurrect_read_after_path_redeclaration -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_carry_read_across_sessions -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_does_not_inherit_parent_read_in_child_session -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_carries_clean_read_across_multiple_checkpoints -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_retains_clean_read_after_prior_same_path_action -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_retains_medium_confidence_for_partial_parent_visible_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test dead_end_thrash -- --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture
    cargo test -p agent-drift-analyzer checkpoints -- --nocapture
    cargo fmt --all -- --check
    cargo check -p agent-drift-analyzer
    cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
    git diff --check
    ```

  - Acceptance: all three focused `dead_end_thrash` regressions, the full `dead_end_thrash` target, every focused truth-grounding control, full `truth_grounding_gap` target, all matching checkpoint tests, format, check, and literal clippy green; exact counts and dispositions recorded only after execution.
  - Source-fix commit/review: after Task 2C is fresh-review-clean, run the required proof, staged `detect-changes`, and land only the one-expression `dead_end_thrash.rs` change as a separate atomic commit; obtain fresh independent built-in `default` review, fixing only within the authorized expression scope in a new commit and repeating fresh review until clean.
  - Final proof receipt: only after the source series and required proof are fresh-review-clean, land a separate TASKS/ledger-only receipt with actual commit hashes, exact proof results, and actual review verdict. Obtain fresh independent review of that receipt and fix/re-review until clean. This receipt, not the boundary amendment or earlier receipt series, must reconcile final Task 3A/Task 4/status wording.
  - Landed implementation series: `4ba9f2647` (`fix: track truth grounding provenance`) + `1f6e863bf` (`fix: preserve grounded path identity and order`) + `9565fb805` (`fix: keep grounding provenance path-scoped`) received fresh independent built-in `default` `REVIEW CLEAN` after two review/fix rounds.
  - Resolved review findings: component path prefixes now retain their identity and use canonical order; declaration pruning is exact; and pathless actions use the all-declared-path grounding rule rather than being grounded by an unrelated single path.
  - Final green behavior/static proof after `5622ddb73`:
    - the three exact `dead_end_thrash` controls passed (`3 / 3`), and the full `dead_end_thrash` target passed (`18 / 18`);
    - the complete `truth_grounding_gap` target passed (`22 / 22`), preserving all nine packet-locked controls;
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` passed `35` matching unit tests and `131` matching integration tests, plus its matching export/provenance tests;
    - `cargo fmt --all -- --check`, `cargo check -p agent-drift-analyzer`, exact `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and `git diff --check` all passed.
  - Clippy-scope impact/detect evidence: pre-edit GitNexus impact for exact `score_confidence` was LOW with one direct caller (`score_dead_end_thrash`) and one affected process; staged detect-changes was LOW and reported exactly that one changed symbol.
  - One-expression source fix: `5622ddb73` (`fix: simplify dead-end confidence gate`) changed exactly `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs` and only the De Morgan-equivalent Boolean expression in `score_confidence`. Fresh independent built-in `default` review returned `REVIEW CLEAN`, confirming no lint allowance, scoring-behavior, threshold, evidence, or adjacent dead-end change.
  - Earlier review-clean proof/blocker receipt series: `a341066a3` (`docs: record grounding proof blocker`) + `0da262979` (`docs: reconcile grounding proof receipt`) received fresh built-in `default` `REVIEW CLEAN` after correcting the P2 landed-state wording. Its former review-state discrepancy remains resolved; the later `5622ddb73` source fix and green literal clippy gate supersede its historical blocker state, while this final receipt supplies the post-fix completion evidence.
  - **EXECUTED — `R6-TGG-CLIPPY-SCOPE-01 = A`:** review-clean boundary series `2067149ce` + `023417c03` authorized exactly one additional production file; separate atomic source commit `5622ddb73` landed the semantics-preserving `clippy::nonminimal_bool` simplification in `score_confidence`, passed the locked proof wall, and received fresh independent `REVIEW CLEAN`.
  - Final proof-receipt review: `fee9c2b16` (`docs: record final grounding proof`) + `6674a8316` (`docs: reconcile final proof receipt`) received fresh independent built-in `default` `REVIEW CLEAN`. That reviewer independently reran the nine packet-locked grounding controls (`9 / 9`), three focused dead-end controls (`3 / 3`), full `dead_end_thrash` (`18 / 18`), full `truth_grounding_gap` (`22 / 22`), checkpoint matches (`35` unit + `131` integration plus matching export/provenance tests), format, check, literal all-target clippy with `-D warnings`, and diff check; all passed.
  - Historical non-acceptance: `52c9ab296` + `73132aead` remains review-findings evidence, not completion. `6409ae072` remains a review-clean witness, not a production closure.
  - Completion record: **COMPLETE** at final proof-receipt series `fee9c2b16` + `6674a8316`, fresh independent built-in `default` `REVIEW CLEAN`. No implementation, proof, source-review, or receipt-review blocker remains. At the Task 3A boundary this completed only Task 3A; the later review-clean transition series applies the phase-status change without altering that historical gate.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.3B — Attributed no-code proof receipt — ineligible and not selected.**
  - Task 2 selected the production path; the operator has now selected internal provenance Option A.
  - No no-code attribution or receipt is authorized. Leave unchecked as the preserved unselected branch.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.4 — Obtain a review-clean closure path.**
  - Require Task 3A committed with exact focused/family/checkpoint/static proof and fresh built-in `default` `REVIEW CLEAN`.
  - Record actual commit hashes, impact results, command counts/results, review findings/dispositions, and final verdict here.
  - Completion state: **COMPLETE** at final proof-receipt series `fee9c2b16` + `6674a8316`, fresh independent built-in `default` `REVIEW CLEAN`. Boundary series `2067149ce` + `023417c03`, implementation series `4ba9f2647` + `1f6e863bf` + `9565fb805`, earlier receipt series `a341066a3` + `0da262979`, source commit `5622ddb73`, the required proof wall, fresh source review, and final receipt review are all clean. At that boundary Task 5 became the sole next gate; the review-clean transition series below later completed that phase-status change.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.5 — Land and independently review the narrow transition.**
  - Prerequisite: Tasks 2A, 2B, 3A, and 4 committed and review-clean.
  - Current state: **COMPLETE / FRESH TRANSITION REVIEW CLEAN**. Authority-only transition series `2937dbe5a` + `91f55f6bf` marks this gap complete, activates only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only packet-creation gate with active packet `none`, and keeps `R6-REPLAY` blocked.
  - Exact authority-only manifest:
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `SPEC.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/01-authority-and-status-map.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/02-phase-and-gate-map.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/06-operator-prompt-library.md`
    - `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-plan.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-tasks.md`
    - this packet's SPEC, PLAN, and TASKS
    - `tasks/plan.md`
    - `tasks/todo.md`
  - Apply actual hashes/results only. Mark this gap complete; activate only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only gate; keep `R6-REPLAY` blocked.
  - Successor boundary: record only these non-link paths:
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md`
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md`
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md`
  - Do not create, link, cite as existing, or execute the successor packet.
  - Commit/review: separate authority-only commit; fresh independent built-in `default`; transition-only fix commits and fresh reviewers until clean.
  - Stop: transition review-clean; no successor work.
  - Transition result (2026-07-14): `2937dbe5a` applied the exact authority-only manifest and
    successor boundary without creating or linking any successor file. Its first fresh review found
    one P2 stale R6-C.1 source-state issue; `91f55f6bf` fixed it, and a fresh independent built-in
    `default` reviewer returned `REVIEW CLEAN` for the series with no findings. The reviewer verified
    the original `CTX-R6-12` exact control `1 / 1` green, diff check clean, and all three exact
    successor paths absent and represented only as non-link `TO CREATE` strings. Task 5 is complete;
    Prompt 1 packet creation is now eligible, but no successor work starts in this phase.

## Explicit Exclusions

No public API, schema, export, replay, sentinel, presentation, R7, wrong-plan-branch implementation, context extraction, command classification, successor packet, TTL/freshness, or read-consumption work belongs in Task 3A. No unrelated scorer change is allowed: the sole expansion is the one-expression, semantics-preserving `score_confidence` boolean simplification in `dead_end_thrash.rs`. No status checkbox advances on intent, a prior witness, or unreviewed future evidence.
