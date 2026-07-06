# R6-3.6 PLAN — Semantic Goal Drift Positive Controls And Corpus Revalidation

Companion to `agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-spec.md`.
This is a validation packet, not a scorer-redesign packet. Execution order is proof-first:
baseline → fixtures → funnel reporting → rerun → docs → review.

## Live repo truth this plan assumes

- `R6-3.5` extraction hardening is already landed.
- `R6-3.X.2` containment first cut is already landed.
- The current acceptance corpus has only three semantic-goal-drift fixture cases, so recall is still
  under-proven.
- The next open scorer-design work after this packet remains the **remaining** graduated-distance family,
  not eligibility-bar loosening.

## Task sequence

1. **Docs lock + baseline capture**
   - Verify the packet scope against live `MAP`, `FINDINGS`, `R6-3` tasks, the scorer, the current
     acceptance corpus, and the batch tooling.
   - Record `git status`, recent commits, and the live semantic-goal-drift test targets.
   - Confirm the local Claude CLI review lane with `--model opus`.
   - Run the planning-doc Claude consult gate before fixture work begins.

2. **Positive-control fixture family**
   - Extend the acceptance corpus from three cases to a true-positive family of at least five unrelated
     pivots.
   - Update the current curated allowlist in
     `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs` in lockstep with the fixture
     directories; the bounded invariant becomes "curated allowlist" rather than "exactly 3 cases."
   - Keep all cases bundle-shaped and routed through the live analyzer checkpoint path.
   - Prefer target anchors that exercise repo, crate, file, work-item, and verification pivots.
   - Assert eligibility explicitly so a fixture that still carries `unknowns` fails loudly.

3. **Negative-control fixture family**
   - Add at least five legitimate non-pivots covering narrowing and progression.
   - Reuse already-landed containment behavior and delegation stratification where possible, but prove the
     still-missing pieces at the acceptance layer rather than rebuilding scorer logic.

4. **Containment boundary guardrails**
   - Add the Codex-discovered non-containment boundary cases.
   - Require at least one acceptance-level case so the live analyzer path cannot regress silently.

5. **Funnel reporting**
   - Inspect `scripts/dev/drift-batch-scan/` and add the smallest reporting deltas needed to make the
     eligibility/suppression funnel explicit.
   - Keep the reporting additive and non-private.
   - If a suppression bucket is not derivable from the current export without duplicating Rust logic or
     widening export surfaces, document that as the packet's honest limit.

6. **Focused validation + full analyzer wall**
   - Run the semantic-goal-drift-focused tests first.
   - Then run the full analyzer wall, workspace clippy, and workspace fmt.

7. **Corpus rerun**
   - Re-run the committed batch harness with the canonical command sequence.
   - Capture raw output, tabulated summary, target inspection summary, junk or weak-anchor summary if
     available, suppression funnel summary, and delegation split.

8. **Docs closeout**
   - Update `FINDINGS`, `MAP`, and the `R6-3` task ledger with the validation results and the next-step
     routing decision.

9. **Claude review gates**
   - Planning docs review: use Claude consult mode on the final `R6-3.6` spec/plan/tasks files with
     `--model opus` and read-only file access before fixture implementation starts.
   - Landed code review: after implementation is complete, use Claude review mode or a focused consult on
     the semantic-goal-drift diff, again with `--model opus`.

## Verification wall

Focused:

```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Full close:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Corpus harness:

```bash
cargo build -p agent-session-compactor -p agent-drift-analyzer
python3 scripts/dev/drift-batch-scan/sample_sessions.py --help
python3 scripts/dev/drift-batch-scan/run_batch.py --help
python3 scripts/dev/drift-batch-scan/tabulate.py --help
python3 scripts/dev/drift-batch-scan/inspect_targets.py --help
python3 scripts/dev/drift-batch-scan/filter_junk.py --help
```

## Risks

1. **SOW drift vs repo truth**
   - Risk: the original SOW predates the landed containment first cut and post-containment corpus re-check.
   - Mitigation: this packet validates the current analyzer state instead of replaying already-landed design
     moves.

2. **Acceptance-only optimism**
   - Risk: synthetic fixtures prove only unit-sized truth.
   - Mitigation: keep the real corpus rerun mandatory or document the exact blocker.

3. **Over-expanding into the graduated-distance remainder**
   - Risk: while adding controls, the packet drifts into new distance logic.
   - Mitigation: any scorer logic beyond the smallest direct bug exposed by a control is out of scope.

4. **Claude review confusion**
   - Risk: branch-wide review would include unrelated historical commits.
   - Mitigation: planning-doc review uses consult mode on the exact files; landed-code review happens later
     on the packet diff.
