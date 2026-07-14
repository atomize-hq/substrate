# Plan: R6-GAP-TGG-TRUTH-PATH-ACTION

Status: **ACTIVE — PACKET DOCS GATE ONLY**. The preserved `CTX-R6-12` witness is `e67d8b214`. Complete and freshly review this docs gate, then reconcile the canonical ledger in a separate review-clean commit before any witness, production, or no-code work.

## Decisions

1. Preserve `truth_grounding_gap_flags_truth_path_action_before_read` and its locked `80 / High / Active`, flagged result with authority plus action evidence.
2. Keep the repair inside `score_truth_grounding_gap`; a truth-path-touching write/verification before a truth read is action, not grounding.
3. Preserve the directly analogous passing `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes` control.
4. Select between red production repair and attributed no-code receipt only after both entry gates are review-clean.
5. Run focused witness and focused regression before the owning family; keep production/no-code proof separate from the final authority transition.

## Ordered Execution

### 0. Lock The Packet

Stage only the three canonical packet docs, run the staged gate, commit atomically, and dispatch a fresh built-in `default` reviewer. Use docs-only fix commits and fresh reviewers until clean. Do not run the witness first.

### 1. Reconcile The Review-Clean Packet Into The Ledger

In a separate batch, touch only `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`. Replace the active named-gap row's three `TO CREATE` markers with the actual packet paths, record the review-clean packet commit, and change only ledger-local status/next-action wording required to make witness reconfirmation next while preserving `PRESERVED RED / GAP ACTIVE`. Commit and obtain a fresh built-in `default` review; use ledger-only fixes and fresh reviewers until clean.

No witness rerun, production edit, or no-code receipt is authorized until Tasks 0 and 1 are each committed and fresh-review-clean.

### 2. Reconfirm Focused Before Family

Run, in order:

```bash
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
```

If the witness remains red, use Step 3A. If it is green, use Step 3B only when the earlier sequential named-gap production boundary can prove red-before/green-after attribution; otherwise stop/escalate.

### 3A. Red Production Path

1. Run:

   ```bash
   npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
   ```

   Impact every additional proposed symbol before editing; stop on HIGH/CRITICAL.
2. Make the smallest change inside `score_truth_grounding_gap` in `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`. Do not change the witness. Add a distinct test only if necessary.
3. Repeat the three Step 2 commands in the same order, then run:

   ```bash
   cargo test -p agent-drift-analyzer checkpoints -- --nocapture
   cargo fmt --all -- --check
   cargo check -p agent-drift-analyzer
   ```
4. Record exact results in TASKS and only changed `CTX-R6-12` / named-gap ledger evidence.

### 3B. Attributed No-Code Path

This second named gap may close without production change only if an already-landed production commit from `R6-GAP-DET-OPAQUE-PARENT` demonstrably made `CTX-R6-12` green. In clean checkouts, prove the exact witness red at the candidate commit's first parent and green at the candidate commit, then run the analogous focused regression and owning family at the green boundary. Record exact commits, commands, and results in a receipt-only TASKS/ledger commit. If the boundary is not red-before/green-after, or attribution points to an unrelated commit, stop/escalate. Never manufacture a no-code receipt from current-HEAD green alone.

### 4. Commit And Fresh Review

For the red fix, eligible no-code receipt, and every review fix:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically and dispatch a fresh built-in `default` reviewer. Apply findings in a new bounded commit and repeat with a fresh reviewer until clean.

### 5. Transition And Stop

Only after the selected closure path is committed and fresh-review-clean, make a separate authority-only transition using the exact manifest in TASKS. It must mark this gap complete; activate only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only gate; keep `R6-REPLAY` blocked; replace no successor `TO CREATE` path with a link or existence claim; and include no production file or successor packet file.

Run the staged gate, commit, and obtain fresh independent review. Use transition-only fix commits and fresh reviewers until clean. Stop at the review-clean transition; do not start the successor.

## Escalation Boundary

Escalate only for HIGH/CRITICAL impact, inability to prove no-code attribution, scorer-local repair failure, unavailable preserved evidence, unisolatable unrelated work, or review showing the packet's product meaning is invalid. Ordinary red proof, LOW/MEDIUM impact, bounded edits, tests, commits, and review fixes remain autonomous.
