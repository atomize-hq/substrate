# R6-GAP-TGG-TRUTH-PATH-ACTION — Truth-Path Action-Before-Read Gap

Status: **ACTIVE — PACKET DOCS GATE ONLY**. `CTX-R6-12`, preserved by witness commit `e67d8b214`, is the only active named gap. The sole current action is to commit and freshly review this SPEC/PLAN/TASKS family. Witness reconfirmation, production/no-code proof, and successor work are not authorized until that gate and the separate canonical-ledger reconciliation are each committed and fresh-review-clean.

## Objective And Preserved Witness

Close only `CTX-R6-12` without weakening or rewriting its committed control:

`truth_grounding_gap_flags_truth_path_action_before_read`

Locked input and result:

- one declared truth artifact, `docs/specs/agent-drift-analyzer-v0.4-spec.md`;
- the first write/verification action is an `apply_patch` touching that same path;
- there is no earlier truth read;
- required result: `80 / High / Active`, flagged;
- evidence includes both `truth artifact hint: docs/specs/agent-drift-analyzer-v0.4-spec.md` and `command family: apply_patch`.

The preserved witness instead produced `0 / Medium / Cleared`, unflagged. At the controls wall it was the sole red among the ten matching `truth_grounding_gap` tests. The directly analogous passing regression is `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes`: the same pre-read write obligation is correctly `80 / High / Active`, flagged, for both planning and implementation when the action is outside the truth path. This packet must make truth-path-touching action obey that same provenance rule without making archetype load-bearing.

## Owning Seam And Boundary

`score_truth_grounding_gap` in `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs` owns the disposition. Live source currently collects a write/verification as `ungrounded_actions` only when it does **not** touch declared truth; a write/verification that touches truth before any read therefore contributes neither a grounded read nor an ungrounded action. The repair is scorer-local: pre-read write/verification against declared truth is still action and must retain its action evidence.

Default production boundary: change only `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`, and within it only `score_truth_grounding_gap`. `first_event_index`, `historical_truth_grounding_gap_evidence`, `dedupe_evidence`, and `is_historical_truth_grounding_gap_reason` are read-only unless the scorer-local repair proves impossible; impact any proposed helper edit and stop/escalate rather than widening silently. Do not edit context extraction, command classification, dispatcher code, another scorer, R7, replay, exports, or presentation.

Before any indexed-symbol edit, run:

```bash
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
```

If any additional symbol is proposed, first run the identical command with its exact symbol name. Warn and stop on HIGH or CRITICAL impact.

## Allowed Files By Atomic Batch

- Packet-docs gate: exactly this SPEC, PLAN, and TASKS.
- Post-docs ledger gate: exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`, in a separate batch after the packet-docs gate is fresh-review-clean. Replace the active row's three `TO CREATE` markers with these actual paths, record the review-clean packet-docs commit, and make witness reconfirmation the next action without changing the preserved-red disposition.
- Red production-fix batch: `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`; this TASKS; and the canonical ledger for actual `CTX-R6-12` / named-gap evidence. `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` is allowed only if a distinct narrow regression is necessary; the preserved witness itself must not change.
- Eligible no-code receipt batch: this TASKS and the canonical ledger only. No production or test file may change.
- Review fixes: only files already allowed for the atomic batch under review.
- Final transition: exactly the authority manifest listed in this TASKS, with this packet and no successor packet file.

Unrelated dirty files remain unstaged.

## Red Versus No-Code Closure

After both entry gates are review-clean, run the focused witness, then the analogous regression, then the owning family.

- **Red:** take the scorer-local production path. Make the smallest change that treats pre-read write/verification as ungrounded action whether or not its path overlaps declared truth. Preserve event ordering, history recovery, read-before-action behavior, evidence deduplication, and the archetype-equivalence result.
- **Unexpectedly green:** no-code closure is eligible only if an already-landed production commit from the earlier sequential gap `R6-GAP-DET-OPAQUE-PARENT` can be shown to change this exact witness from red to green. Prove that attribution at the candidate commit boundary in clean checkouts, with focused proof before family proof, and record the exact hashes/results. If the predecessor boundary cannot demonstrate red-before/green-after, stop for authority reconciliation; an unrelated landed commit or current-HEAD green is not a receipt.

## Exact Verification

Focused proof precedes family proof, both before and after a red-path repair:

```bash
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
```

For a red production path, also run after those commands:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer
```

Replay and broader scorer walls are not part of this gap.

## Commit, Review, And Exit

Before every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit each batch atomically. After packet docs, ledger reconciliation, production fix or no-code receipt, every review fix, and the final transition, dispatch a fresh built-in `default` reviewer. Apply actionable findings in a new bounded commit and repeat with a fresh reviewer until `REVIEW CLEAN`.

The gap exits only when entry gates, selected closure path, exact proof, and fresh reviews are complete. A separate authority-only transition then marks this gap complete and activates only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only gate. Record its three canonical paths as non-link `TO CREATE` entries; do not create, link, cite as existing, or execute that successor. Stop after the transition series is independently review-clean.
