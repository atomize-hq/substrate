# SFR-RB-100 P7 Validation Receipt

Status: **PRIVATE LANE PASS — BOUNDED IMPLEMENTATION REVIEW PENDING**

## Authority

- P6 baseline: `133b55249f88492e16f80f98a63368911d733c7e`
- Private-run source checkpoint: `52b15b8dad1509ad85addbfc2d56c08aa5861c1d`
- Inventory route: repository `CurrentNativeV2`
- Inventory cutoff: `2026-07-30T04:19:50Z`
- Raw rollout, selected-session, checkpoint, path, repository, message, and identifier data remained
  outside Git.

## Release Gate Ledger

The following commands exited zero after the P7-19 canonical typed-output correction:

| Gate | Result |
|---|---|
| `cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture` | PASS |
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
| `python3 scripts/dev/drift-batch-scan/test_tabulate.py` | PASS, 9 tests |
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
| Total checkpoints | 13 |
| Checkpoints with structured objective | 13 |
| Checkpoints with structured target | 0 |
| Current-bar eligible checkpoints | 0 |
| Adjacent checkpoint pairs | 7 |
| Target-resolved adjacent pairs | 0 |
| Semantic-goal-drift firings | 0 |
| Distinct anonymized repository ranks | 3 |
| Batch-receipt file digest | `f9a8b14820e7195286440021eca5097274a37baad4f5c2892bd5563cef089d1b` |
| Aggregate-receipt file digest | `273aef0e7e1e46b9d077f437a33585ff2c48e377e5f5f67f5368339f0ac05d87` |

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
| Compactor | version `0.1.0`; binary digest `2033589c6c1b20e8963c7bd97aa40106bab6a69a4927d7ef819bf86ca369491d` |
| Analyzer | binary digest `d3da8947e9f122be607c4f102c955116ab2018c28247ee7626ddb6406448f48b` |
| Sampler | `52cbd7c409b9de8f856601ff20c5e47a258d57c47ce9c6ba6cb830ad02662aa5` |
| Batch runner | `d97c76fdc9cc983065e9ab1ab3f668e94c8ee637148a3b0b1505d9fd6c2de4ed` |
| Tabulator | `cf9a56ba557275e1003ce378fc928902ee70e22adb6c786af3a66e4cd9689157` |

## Three-Way Triage

1. **Fixture/harness error — fixed.** The first inventory pass encountered a valid non-object JSON
   row. The sampler incorrectly attempted object lookup on the decoded scalar. The parser now
   ignores non-object rows, and a focused regression test plus the complete Python sampler suite
   pass at commit `52b15b8dad1509ad85addbfc2d56c08aa5861c1d`.
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
- Structural receipt validation proved a non-empty run, zero batch failures, filled sufficiently
  populated quotas, and only exact permitted-scarcity underfill.

**Conclusion:** the P7 release wall and private-batch obligations are satisfied. Independent bounded
implementation review remains required before P7 closeout.
