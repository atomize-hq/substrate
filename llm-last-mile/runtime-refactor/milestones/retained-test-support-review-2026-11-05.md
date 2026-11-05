# Retained Codex test support — review and provenance

Reviewed November 5, 2026 (America/Indianapolis), against Milestone 1 commit `91cd0f53e68e840fe30e7cf6b620b13d0c5233cf`.

## Decision

Retain and publish the three pre-existing diffs: 905 gateway-fixture lines, 613 service-fixture lines and 22 backlog lines. A focused read-only review found no material correctness, forwarding, privacy or cancellation blocker. No production behavior changed during this review. The test additions are preserved in commit `a0a8decb78e24f870fbd44db3618604bc03ca9f0`; the earlier M1 record remains an accurate record of its original publication boundary.

The gateway diagnostics preserve all provider methods, headers, lazy stream polling, errors and drop behavior. Persisted diagnostics use closed categories/status/flags rather than raw request/error strings; a failed diagnostic sink does not replace the provider result. Service tests cover retained history and custody through completion, explicit cancellation, dropped completion, and blocked-stdin startup cancellation. The image-support backlog entry records a bounded unresolved capability gap and acceptance criteria.

## Why the changes remained uncommitted

The short investigation supports a scope/ownership carry-over rather than a documented rejection of the code:

1. The original October 26 live-text report ended after a failed live attempt. It explicitly retained two uncommitted test-support additions and the existing image backlog entry, with an empty index and no commit. That failed operation did not establish provider/model qualification.
2. The offline follow-through expanded the gateway diagnostic fixture to 905 lines and the service live diagnostic fixture to 244 lines. Its report recorded passing offline tests and an offline-ready endpoint, again explicitly leaving the support uncommitted. Preparing a diagnostic did not authorize another live attempt.
3. The October 28 handoff labeled the three paths **“Protected live path — never stage/revert.”** The two additional ordinary SDK regressions were present in the later November 2 incoming service-file snapshot. This brief investigation does not establish their exact authorship or a separate reason for leaving them uncommitted.
4. The November 4 handoff repeated the protected-file requirement. Consequently, the November 5 M1 publication preserved their working bytes and excluded the additions, except for the existing service fixture's required `ordinary_effect: None` field adaptation.

The examined records contain no recorded source-review rejection or instruction to delete these additions. They do not prove that forgetting alone caused the missing commits. The immediate reason they stayed out of M1 was the inherited ownership/scope instruction. The subsequent user request to review and dispose of them resolves that publication hold for this bounded follow-up.

Provenance is retained in the ignored Task03 `live0156-text/final-report.md`, `live0156-text/offline-diagnostic/report.md`, the October 28 and November 4 handoffs, and Task20's November 2 `initial-source-inventory.json`. Their identities and this review's raw results are retained under Task20 `single-session-20261102/retained-support-review-20261105`; private runtime evidence is not added to Git.

## Verification

From the assigned integration worktree, Rust 1.89, offline/locked:

- `cargo +1.89.0 test --offline --locked -p substrate-gateway --lib live_diagnostics::tests:: -- --nocapture --test-threads=1`: **6 passed, 0 failed, 0 ignored**.
- `cargo +1.89.0 test --offline --locked -p world-service --lib -- test_live_text_ test_ordinary_native_sdk_cancellation_retains_history_until_reaped test_ordinary_codex_sdk_startup_cancel_observes_reaping --nocapture --test-threads=1`: **5 passed, 0 failed, 0 ignored**.
- Both commands completed with actual child waits and unchanged source. The tested files retain the same bytes as the earlier M1 workspace. Existing formatting/strict checks retain their original scope and identities.

Both live workers remain explicitly ignored. This review ran no live worker, accessed no real account and sent no provider request. The local request gate limits admitted local POSTs, not all upstream HTTP attempts or charges; process observations are not SDK acknowledgment. The original failed live result, unresolved image eligibility, R15 combined-workspace gate and remaining M2/M3 qualification obligations stay unchanged. Publication is preservation of reviewed test support and documentation, not live-provider or installed qualification.
