# Tasks: R6-GAP-TGG-TRUTH-PATH-ACTION

Status: **ACTIVE — OPTION-A IMPLEMENTATION REVIEW-CLEAN / CLIPPY DECISION + RECEIPT-SERIES REVIEW REQUIRED**. The operator explicitly decided `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A`. Packet authority and gate-state series `49b2bbd7f` + `70c0ca9f4` + `baa0983ac` + `d6cbc9d2c`, followed by ledger series `dba992383` + `70955db2a`, received fresh built-in `default` `REVIEW CLEAN`, completing Tasks 2A and 2B. Option-A implementation series `4ba9f2647` + `1f6e863bf` + `9565fb805` also received fresh independent `REVIEW CLEAN` after two review/fix rounds, and its behavior wall is green. Task 3A nevertheless remains incomplete because the packet's literal `cargo clippy ... -D warnings` gate exits `101` solely on one unchanged, out-of-scope `clippy::nonminimal_bool` warning in `dead_end_thrash.rs`, while the landed proof/blocker receipt `a341066a3` received a P2 bookkeeping finding and its receipt series remains pending fresh review. Stable decision `R6-TGG-CLIPPY-SCOPE-01` now gates either a narrowly authorized one-file simplification (recommended) or continued strict-boundary blocking. Task 3A stays blocked on both literal clippy and receipt-series review; Task 4, the transition, and successor work remain blocked.

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

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.3A — Implement and prove Option-A typed provenance.**
  - Prerequisites: Tasks 2A and 2B committed and fresh-review-clean; all required GitNexus impacts below HIGH; no unisolatable unrelated work.
  - Allowed files only:
    - `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs` — `score_truth_grounding_gap`, the provenance type (private unless cross-module signature threading requires `pub(crate)`), and necessary private helpers;
    - `crates/agent-drift-analyzer/src/scoring/mod.rs` — `score_session` and necessary private threading helpers;
    - `crates/agent-drift-analyzer/src/lib.rs` — `analyze_loaded_bundle` and necessary private per-session initialization/threading helpers;
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
    cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture
    cargo test -p agent-drift-analyzer checkpoints -- --nocapture
    cargo fmt --all -- --check
    cargo check -p agent-drift-analyzer
    cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
    git diff --check
    ```

  - Acceptance: every focused control, full `truth_grounding_gap` target, all matching checkpoint tests, format, check, and clippy green; exact counts and dispositions recorded only after execution.
  - Commit/review: staged `detect-changes` gate; atomic bounded commit; fresh built-in `default`; fix in new bounded commits, rerun affected focused proof before the full wall, and use fresh reviewers until clean.
  - Landed implementation series: `4ba9f2647` (`fix: track truth grounding provenance`) + `1f6e863bf` (`fix: preserve grounded path identity and order`) + `9565fb805` (`fix: keep grounding provenance path-scoped`) received fresh independent built-in `default` `REVIEW CLEAN` after two review/fix rounds.
  - Resolved review findings: component path prefixes now retain their identity and use canonical order; declaration pruning is exact; and pathless actions use the all-declared-path grounding rule rather than being grounded by an unrelated single path.
  - Green behavior/static proof:
    - all nine packet-locked exact controls passed (`9 / 9`);
    - reviewer regressions expanded the complete `truth_grounding_gap` target to `22 / 22`, all passing;
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` passed `35` matching unit tests and `131` matching integration tests, plus its matching export/provenance tests;
    - `cargo fmt --all -- --check`, `cargo check -p agent-drift-analyzer`, and `git diff --check` passed;
    - `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings -A clippy::nonminimal_bool` passed all targets.
  - Preserved literal gate failure: exact required `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings` exits `101` solely at unchanged `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs:90-92` (`clippy::nonminimal_bool`), last changed by the earlier review-clean dead-end series `bcd94bf4f` + `931e50c85`. GitNexus impact for exact `score_confidence` is LOW: one direct caller (`score_dead_end_thrash`) and one process.
  - Landed proof/blocker receipt: `a341066a3` (`docs: record grounding proof blocker`) records the implementation series, green behavior proof, literal-clippy blocker, and stable decision. Fresh review returned one P2 because the receipt still described itself as uncommitted. The receipt series beginning at `a341066a3` remains pending fresh review; neither that receipt nor this follow-on correction has a review-clean verdict.
  - **DECISION REQUIRED — `R6-TGG-CLIPPY-SCOPE-01`:**
    - **A (recommended):** authorize one semantics-preserving boolean simplification in `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`, the necessary focused `dead_end_thrash` regression rerun, exact clippy, and the existing TGG wall.
    - **B:** keep the strict Option-A allowed-file boundary; leave Task 3A and the phase blocked on the literal clippy gate.
  - Historical non-acceptance: `52c9ab296` + `73132aead` remains review-findings evidence, not completion. `6409ae072` remains a review-clean witness, not a production closure.
  - Completion record: **INCOMPLETE**. The implementation series is freshly review-clean and its behavior proof is green, but the literal clippy gate remains red and the receipt series beginning at landed proof/blocker receipt `a341066a3` remains pending fresh review after its P2. Do not check Task 3A or begin Task 4 until both gates are resolved.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.3B — Attributed no-code proof receipt — ineligible and not selected.**
  - Task 2 selected the production path; the operator has now selected internal provenance Option A.
  - No no-code attribution or receipt is authorized. Leave unchecked as the preserved unselected branch.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.4 — Obtain a review-clean closure path.**
  - Require Task 3A committed with exact focused/family/checkpoint/static proof and fresh built-in `default` `REVIEW CLEAN`.
  - Record actual commit hashes, impact results, command counts/results, review findings/dispositions, and final verdict here.
  - Current state: **OPEN / BLOCKED ON TASK 3A, `R6-TGG-CLIPPY-SCOPE-01`, AND RECEIPT-SERIES REVIEW**. The review-clean implementation series does not satisfy closure while the literal clippy gate and fresh review of the receipt series beginning at landed proof/blocker receipt `a341066a3` remain open; neither `a341066a3` nor this follow-on correction is claimed review-clean.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.5 — Land and independently review the narrow transition.**
  - Prerequisite: Tasks 2A, 2B, 3A, and 4 committed and review-clean.
  - Current state: **BLOCKED**. No transition or successor work is authorized.
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

## Explicit Exclusions

No public API, schema, export, replay, sentinel, presentation, R7, wrong-plan-branch implementation, context extraction, command classification, unrelated scorer, successor packet, TTL/freshness, or read-consumption work belongs in Task 3A. No status checkbox advances on intent, a prior witness, or unreviewed future evidence.
