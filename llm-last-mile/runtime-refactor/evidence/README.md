**Kind:** evidence index
**Stable ID:** `shared-evidence-navigation`
**Canonical for:** exact extracted `How to read this ledger` body; non-authoritative navigation for extracted evidence owners, the existing current-state projection, and receipt/review-control lookup
**Status:** canonical extracted body with non-authoritative navigation
**Authority scope:** exact extracted reading/interpretation source body only; follow the linked canonical owners and review-control records for everything else
**Source span:** [`../05-debug-regression-ledger.md#how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger), D11 Batch 4 baseline lines 3–15 (823 bytes)
**Baseline span SHA-256:** `80f4b753d6e574202b527589d33021c735a76e2c6563fc5dce890df524c93002`
**Supersedes:** canonical ownership of that exact source body only; navigation remains non-authoritative
**Superseded by:** none
**Projection consumers:** [`../index/README.md`](../index/README.md), [`../05-debug-regression-ledger.md#how-to-read-this-ledger`](../05-debug-regression-ledger.md#how-to-read-this-ledger)

# Runtime evidence index

> **Authority boundary:** Only the exact extracted body between the named markers below is canonical here. All navigation in this file remains non-authoritative. It does not authorize implementation, close a regression gate, satisfy a proof obligation, promote evidence into authority, or replace the linked canonical owners.
>
> **Receipt boundary:** `review-control/` remains the canonical receipt, review-record, and closeout artifact store. This file does not duplicate, normalize, or replace those records.

<!-- exact-extracted-body:how-to-read-this-ledger:start -->
## How to read this ledger

- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
- **Unresolved** means the target behavior lacks an implementation and proof gate.
- Historical diagnoses are retained only when they define a permanent negative or regression test.

Primary source memos:

- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)

<!-- exact-extracted-body:how-to-read-this-ledger:end -->

| Evidence component | Canonical owner | Current extracted scope |
|---|---|---|
| Batch 1 — canonical issue ledger | [`canonical-issue-ledger.md#canonical-issue-ledger`](canonical-issue-ledger.md#canonical-issue-ledger) | Full extracted `## Canonical issue ledger` table plus the preserved `### A1.2a-WB gate assignment` body; exact issue IDs, chronology, limitations, obligations, literals, hashes, commands, and negative requirements remain unchanged. |
| Batch 2 — closeout rule + review-process calibration | [`closeout-and-review-calibration.md`](closeout-and-review-calibration.md) | Exact extracted [`## Closeout rule`](closeout-and-review-calibration.md#closeout-rule) and [`## Review-process calibration`](closeout-and-review-calibration.md#review-process-calibration) bodies with separate noncontiguous source provenance; ordered bullets, literals, IDs, negative requirements, exceptions, review-governance reference-only boundaries, and the no-remediation/no-proof-result-change statement remain unchanged. |
| Batch 3 — baseline behaviors + cross-gate smoke scenarios | [`baseline-behaviors-and-smoke-scenarios.md`](baseline-behaviors-and-smoke-scenarios.md) | Exact extracted [`## Baseline behaviors that all tracks preserve`](baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve) and [`## Cross-gate smoke scenarios`](baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios) bodies with separate exact source provenance preserved between distinct boundary markers, plus the canonical bounded [`A3-scoped RG-BASE-03 disposition`](baseline-behaviors-and-smoke-scenarios.md#a3-scoped-rg-base-03-disposition) outside those bodies; Gate IDs, statuses, table order/cells, ordered scenario steps, deterministic parked-successor sequence, Covers lines, and fail-closed/exception boundaries remain unchanged. |
| Batch 4 — receipt/review-control navigation | [`receipts/README.md`](receipts/README.md) | Navigation only; follow the receipt index to the canonical `review-control/README.md` store. No receipt or review record is copied, reorganized into new evidence, or normalized here, and process/review evidence does not replace product proof, gate satisfaction, or packet/implementation/dispatch/promotion authority. |
| Batch 4 — current-state projection compatibility pointer | [`../macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`](../macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling) | The root `05` current cross-lane projection remains compatibility-only; canonical ownership and status stay with the existing D6 macOS-parity evidence owner, and no ownership or status transfer occurs here. |
