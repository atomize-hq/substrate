# Plan: R6-GAP-TGG-TRUTH-PATH-ACTION

Status: **ACTIVE — OPTION-A PACKET-AMENDMENT GATE**. `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A` is operator-decided. Current HEAD/receipt is `48f259d25`; review-clean witness `6409ae072` preserves the two opposite cross-checkpoint failures. Do not implement from this working tree amendment. First commit and freshly review the three packet docs, then land and freshly review the separate ledger-only decision reconciliation.

## Locked Decisions

1. Replace the stale scorer-only authority with the narrow internal call-chain boundary `analyze_loaded_bundle` → `score_session` → `score_truth_grounding_gap`.
2. Carry typed, path-keyed provenance only from qualifying read observations and event order.
3. Never infer provenance from `DriftScore`, `raw_score`, `state`, `flagged`, evidence presence, historical posture, or evidence reason strings.
4. Carry only matching paths still declared in the current task frame; a read of path A never grounds an action against path B.
5. Initialize provenance per session and never cross parent/child or sibling trajectories.
6. Preserve the original `CTX-R6-12`, make historical-only context non-grounding, and let a real same-path read carry across the tested next checkpoint boundary.
7. Add no freshness, TTL, or read-consumption semantics without separate evidence and a separate decision.
8. Make no public API, schema, export, replay, sentinel, or presentation change.
9. Keep Task 3A, Task 4, the transition, and successor work incomplete until their actual proof and review gates pass.

## Ordered Execution

### 0. Land This Packet Amendment

Current allowed files are exactly the three packet docs. Inspect only that diff, run `git diff --check`, commit atomically, and dispatch a fresh built-in `default` reviewer. Apply docs-only findings in new docs-only commits and use a fresh built-in `default` reviewer each time until `REVIEW CLEAN`.

No ledger, source, test, staging beyond the intended packet docs, or implementation is authorized by the working-tree amendment itself.

### 1. Reconcile The Operator Decision In The Canonical Ledger

After Step 0 is committed and review-clean, use a separate ledger-only batch touching exactly:

`docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`

Record:

- `R6-TGG-CROSS-CHECKPOINT-PROVENANCE-01 = A`;
- review-clean witness `6409ae072` and current decision receipt `48f259d25`;
- the review-clean packet-amendment commit;
- the bounded three-source-file internal provenance seam and focused test authority;
- Task 3A as next while the gap stays active and unproven.

Run the staged gate, commit separately, and obtain fresh built-in `default` `REVIEW CLEAN`. Use ledger-only fix commits and fresh reviewers until clean. This separate reconciliation is required before implementation.

### 2. Run Every Pre-Edit Impact Gate

Only after Steps 0 and 1 are each committed and review-clean, run and record:

```bash
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_session -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact analyze_loaded_bundle -r 97a0-substrate --direction upstream --depth 3
```

If an existing helper will change, run its own literal command before editing it. For the currently bounded scorer helpers, the conditional gates are:

```bash
npx gitnexus impact historical_truth_grounding_gap_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact dedupe_evidence -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact first_event_index -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact is_historical_truth_grounding_gap_reason -r 97a0-substrate --direction upstream --depth 3
```

Add a separate exact command/result for every other existing helper proposed for edit. A new private helper cannot have pre-edit indexed impact; record that honestly and inspect it through staged `detect-changes`. Stop and warn on HIGH or CRITICAL before editing.

### 3. Implement The Internal Provenance Thread

Within only the allowed Option-A files:

1. In `lib.rs`, initialize a new non-exported provenance value inside each `for session in &bundle.sessions` iteration. Pass it through checkpoints of that session only. Do not change the public `analyze_loaded_bundle` signature or result schema.
2. In `scoring/mod.rs`, thread that internal value through `score_session` only as necessary for `truth_grounding_gap`; do not change scorer ordering or another scorer's contract.
3. In `scoring/truth_grounding_gap.rs`, make `score_truth_grounding_gap` consult/update typed path-scoped provenance. Derive it only from qualifying read-like `CommandObservation`s and event order. Retain entries only for matching currently declared truth paths.
4. Keep historical evidence generation separate from provenance. Historical-only state may inform disposition/evidence but cannot ground an action.
5. In `tests/truth_grounding_gap.rs`, preserve the original and review-clean witnesses and add the smallest focused controls needed for path A/path B isolation and per-session/parent-child reset.

Necessary private or `pub(crate)` non-exported types/helpers may live only in `truth_grounding_gap.rs`, `scoring/mod.rs`, or `lib.rs`. No other file is an implementation escape hatch.

### 4. Prove Focused Before Family

Run the exact focused controls first:

```bash
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action -- --exact --nocapture
```

Run every newly added path-scope or session-isolation test individually with `--exact` before continuing. Then run:

```bash
cargo test -p agent-drift-analyzer --test truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
git diff --check
```

The original `CTX-R6-12`, historical-only non-grounding, one-boundary same-path carry, path isolation, session isolation, the full owning target, and checkpoints must all be green. Record actual counts/results in TASKS and the canonical ledger; do not infer them from earlier receipts.

### 5. Commit And Fresh-Review The Closure Candidate

Allowed implementation/proof files are only:

- `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`;
- `crates/agent-drift-analyzer/src/scoring/mod.rs`;
- `crates/agent-drift-analyzer/src/lib.rs`;
- `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs`;
- this TASKS;
- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.

Before each commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically and dispatch a fresh built-in `default` reviewer. Apply each actionable finding in a new bounded commit, rerun affected focused proof before the family/checkpoint/static wall, and dispatch another fresh reviewer. Continue until `REVIEW CLEAN`. Existing review-clean witness `6409ae072` is not a substitute for review of the Option-A implementation series.

### 6. Transition And Stop

Do not check Task 4 or begin the authority transition until the Option-A closure series has exact green proof and fresh `REVIEW CLEAN`. Then use the already listed authority-only transition manifest in TASKS in a separate commit/review series. Keep `R6-REPLAY` blocked, activate only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only gate, create no successor packet, and stop after the transition is review-clean.

## Escalation Boundary

Stop for HIGH/CRITICAL impact, a required edit outside the bounded files, inability to keep provenance typed/path-scoped/session-local, evidence that freshness or consumption is required, any public/schema/export/replay/sentinel/presentation consequence, or failure of the focused-to-family/checkpoint wall. Do not silently widen Option A.
