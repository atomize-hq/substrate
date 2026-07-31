# R6-3.6 TASKS — Semantic Goal Drift Positive Controls And Corpus Revalidation

Status: CLOSED (created 2026-07-05 from the approved planning kickoff plus live `R6` repo truth; landed and
review-clean 2026-07-05). This packet
validates the **current** semantic-goal-drift scorer after `R6-3.5` and the `R6-3.X.2` containment first cut;
it does not reopen already-landed precision work.

## R6-3.6.0: Docs lock and baseline

- [x] Task R6-3.6.0.1: Lock the packet docs to live repo truth.
  - Acceptance: this `R6-3.6` spec/plan/tasks family states explicitly that `R6-3.5` and the `R6-3.X.2`
    containment first cut are already landed, and that this packet validates recall/precision on top of the
    shipping analyzer rather than restarting those packets.
  - Verify: manual review against `docs/specs/r6/MAP.md`, `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`,
    and `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`.
  - Files:
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-spec.md`
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-plan.md`
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-tasks.md`

- [x] Task R6-3.6.0.2: Capture the live baseline and confirm review tooling.
  - Acceptance: the run records the current git status, recent history, actual semantic-goal-drift test
    targets, and a working local Claude CLI lane using `--model opus` for later planning-doc and landed-code
    reviews.
  - Verify:
    - `git status --short`
    - `git log --oneline -n 20`
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - harmless Claude probe via `claude -p --model opus --output-format json --disable-slash-commands --tools ""`
  - Files:
    - none required unless a docs note is needed

- [x] Task R6-3.6.0.3: Run the Claude planning-doc review gate before fixture implementation.
  - Acceptance: the `R6-3.6` spec/plan/tasks files receive a Claude consult pass using `--model opus`, and
    any actionable findings are folded back into the docs before tasks `R6-3.6.1+` begin.
  - Verify: saved Claude review output or quoted findings in the run notes.
  - Files:
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-spec.md`
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-plan.md`
    - `docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-tasks.md`

## R6-3.6.1: Positive controls

- [x] Task R6-3.6.1.1: Add at least five true-positive semantic-goal-drift acceptance controls.
  - Acceptance: the committed acceptance corpus grows from the current three-case family to include at least
    five true-positive unrelated pivots that run through the live analyzer checkpoint path. Recommended
    families: repo pivot, crate pivot, file pivot, work-item pivot, and verification-target pivot. Update
    `SEMANTIC_GOAL_DRIFT_ACCEPTANCE_CASE_IDS` and the `...corpus_stays_bounded_and_bundle_shaped` root
    directory assertion in lockstep so "bounded" remains a curated allowlist, not a frozen count of three.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`

- [x] Task R6-3.6.1.2: Require stable-anchor proof for every positive control.
  - Acceptance: each positive control documents stable previous and current targets, and the assertions prove
    the fire is not coming from junk-only anchors, weak-only anchors, or accidental containment suppression.
    Each control must also assert that it clears the full current eligibility gate through the live extractor
    (`unknowns.is_empty()`), so a target-only but still-ineligible fixture fails loudly.
  - Verify: acceptance test assertions and, if needed, focused scorer tests.
  - Files:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (only if a tiny proof helper is needed)

## R6-3.6.2: Negative controls and guardrails

- [x] Task R6-3.6.2.1: Add at least five legitimate non-pivot controls.
  - Acceptance: the corpus covers path narrowing, symbol narrowing, plan→code→plan, review→fix→verify, and
    same-family doc progression, all proving `semantic_goal_drift` stays quiet on the current analyzer.
    Reuse already-landed coverage where it already exists
    (`synthetic-kickoff-narrowing-into-anchored-subtree`, scorer-level non-containment regressions, and the
    landed delegation split in `scripts/dev/drift-batch-scan/`) and scope this packet's delta to the
    still-missing acceptance-layer proof.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - Status note (2026-07-05): the acceptance corpus now includes five quiet containment / exact-target
    controls (`synthetic-kickoff-narrowing-into-anchored-subtree`,
    `synthetic-kickoff-line-suffix-same-file`, `synthetic-rolling-directory-to-file-narrowing`,
    `synthetic-rolling-file-to-containing-directory-broadening`,
    `synthetic-rolling-map-review-to-map-verify-progression`). The documented residual
    doc-progression / plan→code→plan classes remain open scorer debt and stay to be called out honestly in
    closeout docs rather than papered over as resolved.

- [x] Task R6-3.6.2.2: Pin the non-containment boundary cases.
  - Acceptance: these boundary cases stay explicitly covered:
    - `docs/specs/r6-map` → `docs/specs/r6/map.md`
    - `objective` → `objective.rs`
    - sibling files are not containment by shared stem alone
    At least one of these goes through the live acceptance path. If a boundary overlaps a documented
    residual over-fire rather than a fully closed bug, the docs must say so explicitly.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
    - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (only if a genuinely new scorer-level
      guard is required)

## R6-3.6.3: Funnel reporting

- [x] Task R6-3.6.3.1: Add or verify the eligibility/suppression funnel metrics.
  - Acceptance: the batch tooling reports enough counts to explain why the corpus emitted zero or non-zero
    fires, including sessions, checkpoints, structured-objective coverage, stable-target coverage,
    current-bar eligibility, target-resolved eligibility when available, candidate adjacent pairs,
    exact-match or same-target suppressions, structural-containment suppressions, target-hygiene suppressions,
    remaining disjoint pairs, emitted fires, and delegation stratification when surfaced by current tooling.
    If any suppression bucket is not derivable from the current checkpoint export without duplicating Rust
    scorer logic in Python or widening analyzer export/schema surfaces, the task records that blocker
    instead of papering over it.
  - Verify:
    - `python3 scripts/dev/drift-batch-scan/tabulate.py --help`
    - dry-run or sample invocation on existing batch output
  - Files:
    - `scripts/dev/drift-batch-scan/README.md`
    - `scripts/dev/drift-batch-scan/tabulate.py`
    - `scripts/dev/drift-batch-scan/inspect_targets.py`
    - `scripts/dev/drift-batch-scan/filter_junk.py`
    - `scripts/dev/drift-batch-scan/run_batch.py` (only if the smallest reporting tag must be added)
  - Result note (2026-07-05): `tabulate.py` now prints the explicit export-derivable funnel and
    labels the current honest limits. Structural-containment suppressions, scorer-true stable-target
    hygiene suppressions, and `sanctioned_replan` suppressions remain non-derivable from the current
    checkpoint export without duplicating Rust scorer logic or widening the export surface, so the
    packet records them as blockers instead of inventing synthetic counts.

## R6-3.6.4: Verification and corpus rerun

- [x] Task R6-3.6.4.1: Run the focused semantic-goal-drift wall and the full analyzer wall.
  - Acceptance: the focused tests, full analyzer wall, workspace clippy, and workspace fmt all pass after
    the validation packet changes.
  - Verify:
    - `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo fmt --all -- --check`
  - Files:
    - any touched packet files

- [x] Task R6-3.6.4.2: Re-run the 110-session corpus harness or record the blocker honestly.
  - Acceptance: the committed `scripts/dev/drift-batch-scan/` flow is rerun with the canonical live command
    sequence, or the exact blocker is recorded with an actionable rerun command. No private generated batch
    artifacts are committed.
  - Verify:
    - `cargo build -p agent-session-compactor -p agent-drift-analyzer`
    - `python3 scripts/dev/drift-batch-scan/sample_sessions.py ...`
    - `python3 scripts/dev/drift-batch-scan/run_batch.py ...`
    - `python3 scripts/dev/drift-batch-scan/tabulate.py ...`
    - `python3 scripts/dev/drift-batch-scan/inspect_targets.py ...`
    - `python3 scripts/dev/drift-batch-scan/filter_junk.py ...`
  - Files:
    - docs only, unless a tiny reporting improvement was required
  - Result note (2026-07-05): reran the committed harness on a fresh seed-42 draw in scratch space:
    `110/110` sessions analyzed clean, `44` repos, `938` checkpoints, `0` fires, `221` adjacent
    target-eligible pairs, `213` same-target exact-match suppressions, `8` changed-target candidates,
    `7` remaining disjoint pairs, `0` genuine pivots.

## R6-3.6.5: Docs closeout and Claude review

- [x] Task R6-3.6.5.1: Update findings and routing docs with the validation outcome.
  - Acceptance: `FINDINGS`, `MAP`, and the `R6-3` task ledger record:
    - positive controls passed/failed
    - negative controls passed/failed
    - corpus totals and funnel metrics
    - delegation split
    - remaining residue
    - explicit yes/no decision on whether to proceed to the remaining graduated-distance work next
  - Verify: manual doc audit against the final test and corpus outputs.
  - Files:
    - `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`

- [x] Task R6-3.6.5.2: Reserve the Claude landed-code review gate.
  - Acceptance: the packet closeout notes state that after implementation lands, the code diff must receive a
    second Claude pass with `--model opus` before claiming review-clean.
  - Verify: final packet closeout notes.
  - Files:
    - this tasks ledger or the future closeout notes
  - Result note (2026-07-05): completed via local Claude CLI read-only review session
    `9a42a4fb-980c-4cfd-ac32-3bda7b0eafee` (`--model opus` → `claude-opus-4-8`), verdict `READY`
    with no actionable findings.
