# R6-3.6 SPEC — Semantic Goal Drift Positive Controls And Corpus Revalidation

Status: OPEN (planning lock 2026-07-05). Packet-scoped to `crates/agent-drift-analyzer` validation,
acceptance fixtures, and corpus-report tooling.

Authority / cross-references:
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md` — live validation record through the
  `R6-3.5` extraction hardening and the `R6-3.X.2` containment first cut.
- `docs/specs/r6/MAP.md` — `R6-3.5` and `R6-3.X.2` landed truth plus the still-open graduated-distance
  remainder and deferred `R6-3.X.3` eligibility revisit.
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md` — open
  `R6-3.X.2` / `R6-3.X.3` debt, including the already-landed containment first cut.
- Live code and tests:
  - `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
  - `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
  - `scripts/dev/drift-batch-scan/**`

## Problem

The original SOW correctly identified the missing proof wall: low or zero real-corpus fires can show good
precision, but they do not prove recall. Live repo truth has advanced since that SOW was written:

1. `R6-3.5` extraction hardening already landed and removed the observed junk-target over-fire.
2. `R6-3.X.2` already landed a bounded containment first cut, so canonical path and symbol narrowing no
   longer flags as drift.
3. The remaining open `R6-3.X.2` work is the genuinely graduated or weighted distance remainder.

What is still missing is an explicit validation packet that proves the **shipping analyzer can still fire on
known true abandoned-goal pivots** while staying quiet on legitimate narrowing and progression after those
precision fixes. Today the committed acceptance corpus is still too small and too asymmetric for that claim:
it has only three semantic-goal-drift fixture cases, and the real-corpus reports emphasize precision and
residue but do not yet pin a reusable positive-control family. The live acceptance harness also hardcodes
that three-case allowlist, so growing the corpus requires updating the curated case list and its
directory-equality assertion in lockstep.

## Objective

Add a bounded validation wall for the current `semantic_goal_drift` scorer state, after `R6-3.5` and the
`R6-3.X.2` containment first cut, before more of the graduated-distance remainder is trusted or expanded.

This packet must answer:

- Can the current scorer still fire on known true unrelated pivots?
- Can it stay quiet on narrowing/progression cases that should remain non-drift after containment?
- Can the batch tooling explain **why** a candidate did or did not fire, instead of only reporting top-line
  fire counts?

## Scope Reconciliation With Live Repo Truth

This packet is **validation-first**, not a restart of extraction hardening or a duplicate of the already
landed containment first cut.

In scope:
- adding positive-control and negative-control fixture families
- adding containment false-negative guardrails where the current proof wall is still too thin
- adding or tightening batch-tooling funnel metrics and delegation-stratified summaries
- rerunning the focused analyzer wall and the corpus harness
- updating `FINDINGS`, `MAP`, and the relevant task ledger with the validation outcome and the next-step
  decision

Out of scope:
- new extraction-hardening work unless a fixture exposes a direct current-input bug
- new eligibility-bar loosening
- sentinel behavior changes
- compactor normalization changes
- full graduated or weighted distance implementation beyond already-landed containment

## Primary Files

- `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
- `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
- `scripts/dev/drift-batch-scan/**`
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md`
- `docs/specs/r6/MAP.md`
- `docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md`

Secondary scope only if the smallest proof move needs it:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

## Required Validation Work

### 1. Re-establish the live baseline

Before edits, capture:

- `git status --short`
- `git log --oneline -n 20`
- focused semantic-goal-drift tests
- full analyzer wall

Also confirm the live corpus tooling can report the funnel fields this packet depends on, or identify the
smallest missing metric that must be added.

### 2. Add a positive-control family

Add at least five true-positive controls that go through the same live analyzer/scorer path as the existing
acceptance fixtures. They must be current-bar eligible, use stable target anchors, and represent genuine
unrelated pivots rather than weak-anchor churn.

Recommended families:
- repo-target pivot
- crate/package pivot
- file-target pivot
- feature/work-item pivot
- verification-target pivot

Expected for each:
- `semantic_goal_drift` fires
- evidence names both sides clearly enough to audit the pivot
- the fixture clears the full current `unknowns.is_empty()` eligibility bar through the live extractor, not
  just a resolved target
- the fire does not depend on junk or weak anchors
- containment does not suppress it accidentally

### 3. Add a negative-control family

Add at least five legitimate non-pivots proving the recent precision fixes stay intact:

- path narrowing
- symbol narrowing
- plan → code → same-plan update progression
- review → fix → verify progression
- same-family doc progression

Expected for each:
- `semantic_goal_drift` does not fire
- the quiet result remains auditable as suppression or relatedness, not silent disappearance

### 4. Pin containment false-negative guardrails

At minimum pin these boundary cases:

- `docs/specs/r6-map` → `docs/specs/r6/map.md` is **not** containment
- `objective` → `objective.rs` is **not** containment
- sibling files are not containment by shared stem alone

At least one of these guardrails must run through the acceptance path, not only scorer-local unit tests.
Where a boundary overlaps a documented residual over-fire rather than a fully closed bug, the doc update must
say so explicitly instead of pretending this packet settles it.

### 5. Expose the eligibility and suppression funnel

The batch output must explain why there were zero fires or non-zero fires. At minimum it should report:

- total sessions
- total checkpoints
- checkpoints with structured objective
- checkpoints with stable target anchors
- current-bar eligible checkpoints
- target-resolved eligible checkpoints, if available
- candidate adjacent pairs considered
- same-target or exact-match suppressions
- structural-containment suppressions
- stable-target-hygiene suppressions
- remaining disjoint pairs
- emitted `semantic_goal_drift` fires

If feasible, also stratify by:
- delegation topology
- weak or junk-only suppression

If a suppression category is not derivable from the current checkpoint export without re-implementing Rust
scorer logic in Python or widening the analyzer export surface, record that honestly as the blocker. Do not
add export or schema changes in this packet unless the user explicitly widens scope.

### 6. Rerun the validation wall

Run:

- `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture`
- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`

### 7. Rerun the corpus harness

Use the committed `scripts/dev/drift-batch-scan/` flow with the canonical live command or document the exact
reason it could not be run. Do not commit private generated batch data.

### 8. Update routing docs and next-step decision

Update the findings and packet routing docs with:

- positive-control pass/fail count
- negative-control pass/fail count
- corpus rerun totals and funnel metrics
- delegation stratification
- remaining disjoint residue
- clear yes/no decision on whether the repo should proceed to the **remaining** graduated-distance work
  next

## Acceptance

This packet is complete only when all are true:

1. At least five known true semantic-goal-drift positive controls fire.
2. At least five legitimate narrowing/progression negative controls stay quiet.
3. The containment false-negative guardrails are pinned.
4. The eligibility/suppression funnel explains the fire outcome, or the docs state exactly which suppression
   categories are not derivable from the current export and why the packet left them that way.
5. The corpus rerun completes, or its blocker is explicit and actionable.
6. The corpus report includes delegation stratification when available from current tooling.
7. No production input contracts are weakened.
8. No eligibility-bar loosening is introduced.
9. No new weighted/graduated-distance implementation is introduced in this packet.
10. The full analyzer wall, workspace clippy, and workspace fmt are green.
11. Docs say clearly whether to proceed to the remaining `R6-3.X.2` graduated-distance work next.

## Non-Goals

- Reopening `R6-3.5` extraction-hardening design
- Reopening the already-landed containment first cut except to validate it
- Loosening the shared eligibility bar (`R6-3.X.3`)
- Full delegated/subagent semantics (`R7`)
- `progress.rs` reset/comparability migration (`R6-4`)
- Sentinel posture changes

## Review Gate

Planning docs and landed code for this packet both require a Claude pass through the local CLI using
`--model opus`. A live probe on 2026-07-05 confirmed that this resolves to `claude-opus-4-8`. No separate
callable `ultracode` flag is exposed by the local CLI help, so the review gate uses the confirmed `opus`
model selector.
