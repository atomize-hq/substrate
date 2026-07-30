# SFR-RB-100 P7 Validation Receipt

Status: **REMEDIATED PRIVATE LANE PASS — BOUNDED FOLLOW-UP REVIEW PENDING**

## Authority

- P6 baseline: `133b55249f88492e16f80f98a63368911d733c7e`
- Private-run source checkpoint: `154465e38e4cfe94ac4046144d581d84060be858`
- Inventory route: repository `CurrentNativeV2`
- Inventory cutoff: `2026-07-30T04:19:50Z`
- Raw rollout, selected-session, checkpoint, path, repository, message, and identifier data remained
  outside Git.

## Release Gate Ledger

The following commands exited zero after all four accepted bounded-review findings were remediated:

| Gate | Result |
|---|---|
| `cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture` | PASS, 26 tests |
| `cargo test -p agent-session-compactor --test current_native_adapter -- --nocapture` | PASS, 7 tests |
| `cargo test -p agent-session-compactor --test bounded_closure -- --nocapture` | PASS, 10 tests |
| `cargo test -p agent-session-compactor --test export_bundle -- --nocapture` | PASS, 5 tests |
| `cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture` | PASS, 2 tests |
| `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` | PASS, 4 tests |
| `cargo test -p agent-drift-analyzer --test delegated_acceptance -- --nocapture` | PASS, 2 tests |
| `cargo test -p agent-drift-analyzer --test input_contract -- --nocapture` | PASS, 17 tests |
| `cargo test -p agent-drift-sentinel --test live_event_shape -- --nocapture` | PASS, 12 tests |
| `cargo test -p agent-drift-sentinel --test real_session_live -- --nocapture` | PASS, 38 tests |
| `cargo test -p agent-session-compactor -- --nocapture` | PASS |
| `cargo test -p agent-drift-analyzer -- --nocapture` | PASS |
| `cargo test -p agent-drift-sentinel -- --nocapture` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace -- --nocapture` | PASS |
| `python3 scripts/dev/drift-batch-scan/test_sample_sessions.py` | PASS, 12 tests |
| `python3 scripts/dev/drift-batch-scan/test_tabulate.py` | PASS, 14 tests |
| `git diff --check` | PASS |

## Frozen Inventory And Selection

| Field | Value |
|---|---|
| Candidate count | 706 |
| Inventory digest | `ac292bfb5eeff13fb489bc39d77f7d65620d5283858f4a62d58604acacfd4d87` |
| Quota schema | 1 |
| Seed | 42, tie-break only |
| Required families | `language_repo`, `workflow`, `tooling`, `delegation` |
| Quota-config digest | `dab39998472995a51c8f2ba3f62d4556bc05c79c4f2e026ced8abbff06b8a703` |
| Selected count | 6 |
| Selected-set digest | `b0df284f3bbb3a69acabbcefcf59aba8e04a267251154e5b6d512149a0780c57` |
| Selection-receipt file digest | `98ad10a7ae164fff6d2fb622aa469ae74756bdc893de37e689f450faeb1e2801` |

Ordinary buckets use quota and mandatory population 3. High-risk delegation buckets use quota and
mandatory population 5.

| Family | Label | Eligible | Selected | Underfill | Result |
|---|---|---:|---:|---:|---|
| delegation | `delegated_child_visible` | 706 | 6 | 0 | filled |
| delegation | `delegated_parent_opaque` | 0 | 0 | 5 | permitted inventory scarcity |
| delegation | `single_agent` | 0 | 0 | 3 | permitted inventory scarcity |
| language/repository | `docs_only` | 8 | 3 | 0 | filled |
| language/repository | `js_ts` | 690 | 3 | 0 | filled |
| language/repository | `mixed` | 688 | 3 | 0 | filled |
| language/repository | `python` | 525 | 3 | 0 | filled |
| language/repository | `rust` | 411 | 3 | 0 | filled |
| tooling | `cargo_rust` | 348 | 3 | 0 | filled |
| tooling | `generic_filesystem_doc` | 674 | 3 | 0 | filled |
| tooling | `node_npm` | 509 | 3 | 0 | filled |
| tooling | `python_pytest` | 524 | 3 | 0 | filled |
| workflow | `docs_planning` | 699 | 6 | 0 | filled |
| workflow | `implementation` | 698 | 6 | 0 | filled |
| workflow | `mixed` | 699 | 6 | 0 | filled |
| workflow | `review_fix` | 702 | 6 | 0 | filled |
| workflow | `verification` | 698 | 6 | 0 | filled |

Every sufficiently populated bucket filled its quota. Each underfilled bucket had zero eligible
candidates, and every eligible candidate in those buckets was therefore selected. The selected set
was non-empty.

## Private Batch Result

| Field | Value |
|---|---:|
| Attempted sessions | 6 |
| Succeeded sessions | 6 |
| Failed sessions | 0 |
| Checkpoint files | 6 |
| Total checkpoints | 13 |
| Checkpoints with structured objective | 13 |
| Checkpoints with structured target | 0 |
| Current-bar eligible checkpoints | 0 |
| Adjacent checkpoint pairs | 7 |
| Target-resolved adjacent pairs | 0 |
| Semantic-goal-drift firings | 0 |
| Distinct anonymized repository ranks | 3 |
| Checkpoint-set digest | `5bda9ce249f49b59f2d6680bb125e884aba126b03663e8e7c735eff093ee60d1` |
| Batch-receipt file digest | `17fb9bbff1b0c6bb3c5b920f3313e16bf04a725ab1feefe70644667a2ed6efe1` |
| Aggregate-receipt file digest | `43660be5630ea0189c70dfcc284d4ccbdaa592f2dd27ac51b357048c8e3485ca` |

The zero eligible/firing counts are distribution observations, not a contradiction: the selected
private checkpoints did not contain analyzer-grounded structured targets. The committed P7 wall,
not this heuristic private lane, owns exact positive, negative, suppression, and error outcomes.
Heuristic strata make no claim about structural containment, scorer-true stable-target hygiene, or
sanctioned-replan behavior.

## Tool Identity

| Tool | Version or digest |
|---|---|
| Rust compiler | `rustc 1.89.0 (29483883e 2025-08-04)` |
| Cargo | `cargo 1.89.0 (c24e10642 2025-06-23)` |
| Python | `3.11.9` |
| Compactor | version `0.1.0`; binary digest `b728f2a8e77f78b98abe200f89e35460d3d54de63f14baeab0f63eb463f033eb` |
| Analyzer | binary digest `3c3944bb2471bbf83d9b68e63ff52852a76f42f5828000c5e75aa4dbb119f0fc` |
| Sampler | `52cbd7c409b9de8f856601ff20c5e47a258d57c47ce9c6ba6cb830ad02662aa5` |
| Batch runner | `13ff4b94909eb6bb1d5b62da38efe3835a9a2a79fb9c7ee8a672a4b733105f87` |
| Tabulator | `75a7f7b5fca66f3e7787df339ffceda4745d4c8738a17581aa40d9c27c19f965` |

## Bounded Review Remediation

The initial bounded implementation review reported one High and three Medium P7-owned
fixture/harness defects. All four were accepted and repaired without production-Rust changes:

1. `d62d8fc0a` binds the selected manifest to its receipt digest, requires a fresh batch directory,
   writes checkpoints atomically, exits nonzero on any batch failure, and binds aggregation to the
   exact successful checkpoint set.
2. `232c6d1e3` makes the matrix expected projection exactly match each tracked `expected.json` and
   requires a diagnostic owner compatible with the declared terminal boundary.
3. `1e8c811b4` makes aligned, narrowing, and sanctioned-replan controls causal. The replan case now
   proves the same eligible pivot becomes an active score of 80 when only the sanction marker is
   removed.
4. `154465e38` carries whole-wall P7-01 and P7-09 execution through the public-live coordinator and
   includes public events, scheduler decisions, and runtime state in the canonical determinism
   projection.

The frozen six-session selection was then rerun through the strengthened chain. The selected-set
digest matched before execution; the fresh checkpoint set contained exactly six files and thirteen
checkpoints; the batch recorded six successes and zero failures; and the tabulator recomputed the
same checkpoint-set digest before producing the aggregate receipt.

## Three-Way Triage

1. **Fixture/harness errors — fixed.** The first inventory pass encountered a valid non-object JSON
   row; the sampler now ignores non-object rows. The later bounded review found four additional
   P7-owned proof defects, all repaired and covered by mutation or counterfactual tests as recorded
   above. The private lane was rerun after those repairs rather than reusing its earlier receipt.
2. **Production contract defect — none surfaced.** All six selected sessions completed compaction
   and analysis. The run produced no concrete evidence contradicting a stated P7 or inherited
   P1-P6 contract.
3. **New behavior request — none opened.** Zero target-grounded eligibility and two zero-population
   delegation strata are recorded limitations of this frozen inventory, not requests to change
   production behavior or thresholds.

## Privacy And Conclusion

- Generated receipts declare `raw_private_fields_included: false`.
- The committed receipt contains only aggregate counts, anonymous labels, digests, versions, and
  classifications.
- Raw inventory, selection, per-session output, and tabulation artifacts remain untracked.
- `current_native_recall_validation_receipt_is_privacy_safe` recursively applies the committed P7
  forbidden-marker scan to this document.
- Structural receipt validation proved the selected-set digest before execution, a fresh exact
  checkpoint set, a non-empty run, zero batch failures, filled sufficiently populated quotas, and
  only exact permitted-scarcity underfill.

**Conclusion:** the remediated P7 release wall and strengthened private-batch obligations are
satisfied. A bounded follow-up review of only the accepted-finding remediation delta remains
required before P7 closeout.
