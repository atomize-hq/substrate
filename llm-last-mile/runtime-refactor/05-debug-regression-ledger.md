# Debug Regression Ledger

## How to read this ledger

- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
- **Unresolved** means the target behavior lacks an implementation and proof gate.
- Historical diagnoses are retained only when they define a permanent negative or regression test.

Primary source memos:

- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)

## Canonical issue ledger

Canonical content: [`evidence/canonical-issue-ledger.md#canonical-issue-ledger`](evidence/canonical-issue-ledger.md#canonical-issue-ledger).

### A1.2a-WB gate assignment

Canonical content: [`evidence/canonical-issue-ledger.md#a12a-wb-gate-assignment`](evidence/canonical-issue-ledger.md#a12a-wb-gate-assignment).

## A0 closeout evidence

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence`](a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence).

## A1.1d mandatory-review state

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state`](a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state).

## A1.1d-5I installer/bootstrap audit record

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record`](a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record).

## A1.1e explicit-home policy proof requirement

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement`](a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement).

## A1.2a, A1.2a-WB, and A1.2a-S recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result).

## B0 closeout evidence

Canonical content: [`b1-b2-1/evidence-regression.md#b0-closeout-evidence`](b1-b2-1/evidence-regression.md#b0-closeout-evidence).

## B1 production-path sequencing blocker and disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition`](b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition).

## B1/B2.1 `RegressionMasked` stop and ownership disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition`](b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition).

## B1/B2.1-R0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result).

## B3.2a and B3.2a-WA recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result`](b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result).

## B1/B2.1 core recovery and B1/B2.1-0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result).

## B3.1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#b31-recorded-result`](b3-1-c1/evidence-regression.md#b31-recorded-result).

## C1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#c1-recorded-result`](b3-1-c1/evidence-regression.md#c1-recorded-result).

## A1.2b recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result).

## Baseline behaviors that all tracks preserve

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve`](evidence/baseline-behaviors-and-smoke-scenarios.md#baseline-behaviors-that-all-tracks-preserve).

## Cross-gate smoke scenarios

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios`](evidence/baseline-behaviors-and-smoke-scenarios.md#cross-gate-smoke-scenarios).

### S1 — Authority survives episode loss

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s1--authority-survives-episode-loss`](evidence/baseline-behaviors-and-smoke-scenarios.md#s1--authority-survives-episode-loss).

### S1A — Revision-bound host transitions survive helper loss and replay

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s1a--revision-bound-host-transitions-survive-helper-loss-and-replay`](evidence/baseline-behaviors-and-smoke-scenarios.md#s1a--revision-bound-host-transitions-survive-helper-loss-and-replay).

### S2 — Receipt, supervisor, obligation, cancel

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s2--receipt-supervisor-obligation-cancel`](evidence/baseline-behaviors-and-smoke-scenarios.md#s2--receipt-supervisor-obligation-cancel).

### S3 — World-UAA mediation under narrowed policy

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s3--world-uaa-mediation-under-narrowed-policy`](evidence/baseline-behaviors-and-smoke-scenarios.md#s3--world-uaa-mediation-under-narrowed-policy).

### S4 — Host visibility and failure truth

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s4--host-visibility-and-failure-truth`](evidence/baseline-behaviors-and-smoke-scenarios.md#s4--host-visibility-and-failure-truth).

### S5 — Existing gateway carrier preservation and direct Codex adoption

Compatibility anchor only; canonical content: [`evidence/baseline-behaviors-and-smoke-scenarios.md#s5--existing-gateway-carrier-preservation-and-direct-codex-adoption`](evidence/baseline-behaviors-and-smoke-scenarios.md#s5--existing-gateway-carrier-preservation-and-direct-codex-adoption).

## Closeout rule

Compatibility anchor only; canonical content: [`evidence/closeout-and-review-calibration.md#closeout-rule`](evidence/closeout-and-review-calibration.md#closeout-rule).

### R2-2 historical failed integration closeout and remaining-seam correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction`](a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction).

## A1.1d-5R2-2F0-HC empirical closure record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record).

### Environment-inventory correction evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence`](a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence).

### Preflight and preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation`](a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation).

### Forced overlap matrix

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix`](a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix).

### Bounded clean-code broad evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence`](a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence).

## A1.1d-5R2-2F0 historical parallel artifact recovery and authority correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction).

### Bounded recovery result

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result`](a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result).

### Corrected semantic and concurrency authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority`](a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority).

### Canonical candidate preservation, proof, review, and stop boundary

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary`](a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary).

## A1.1d-5R2-2F readiness-boundary evidence ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger).

### Reviewer finding and correction disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition`](a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition).

## A1.1d-5R2-2F5-PD evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger).

### Verified starting state and retained evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence`](a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence).

### Exact blocked-candidate preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation`](a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation).

### Control-pack contradiction and source decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision`](a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision).

### Security decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#security-decision`](a1.1d-5r2-2f/evidence-regression.md#security-decision).

## A1.1d-5R2-2F final evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger).

### Verified source and preservation identities

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities`](a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities).

### Blocked-donor hunk disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition`](a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition).

### Final runtime behavior and proof

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof`](a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof).

### Final decision and next node

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node`](a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node).
## Renewed R2-2 publication decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger).

## A1.1d-5R2-2-B1 broad-wall invocation evidence correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction).

### Verified B0 evidence and causal reconstruction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction).

### Baseline and future classification authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority).

## Closeout-remediation planning ledger (historical RP0 checkpoint)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint).

### Immutable planning checkpoint

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint).

### Renewed-closeout blocker record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record).

### R1 source-closure evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence).

### P1 failed-harness evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence).

### P1 source-closure and decision record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record).

## RP3/RP4/RP5 closeout ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger).
## Review-process calibration

Compatibility anchor only; canonical content: [`evidence/closeout-and-review-calibration.md#review-process-calibration`](evidence/closeout-and-review-calibration.md#review-process-calibration).


## A1.1d-5R2-4 terminal evidence ledger

Canonical content: [`a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger`](a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger).

## A1.1d-5R3 planned proof and regression ledger (archived for active scheduling)

Canonical content: [`a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling`](a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling).

### Source-closure and risk ledger

Canonical content: [`a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger`](a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger).

### Candidate and manifest matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices`](a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices).

### Lifecycle convergence and preservation matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices`](a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices).

### Baseline and product behavior wall

Canonical content: [`a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall`](a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall).

### `R3-NATIVE-LINUX-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-linux-01`](a1.1d-5r3/evidence-regression.md#r3-native-linux-01).

### Packet-local provider evidence gates

Canonical content: [`a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates`](a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates).

### `R3-NATIVE-MAC-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-mac-01`](a1.1d-5r3/evidence-regression.md#r3-native-mac-01).

### `R3-NATIVE-WIN-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-win-01`](a1.1d-5r3/evidence-regression.md#r3-native-win-01).

### Final decision rule

Canonical content: [`a1.1d-5r3/evidence-regression.md#final-decision-rule`](a1.1d-5r3/evidence-regression.md#final-decision-rule).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/evidence-status.md#r3-implementation-status-append`](a1.1d-5r3/evidence-status.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-manifest`](a1.1d-5r3/evidence-status.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-mac`](a1.1d-5r3/evidence-status.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression-command and status correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07).

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION regression ledger (2026-08-10)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10).
## R3 macOS retirement/orphan planning and G2/G3 stop ledger (2026-08-13)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13`](r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13).
## Current cross-lane regression ledger (2026-08-20; controlling)

Canonical content: [`macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`](macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling).
