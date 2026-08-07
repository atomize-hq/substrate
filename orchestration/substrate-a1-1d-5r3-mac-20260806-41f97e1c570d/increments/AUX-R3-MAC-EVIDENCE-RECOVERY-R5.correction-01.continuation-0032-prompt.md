R5 CORRECTIVE PACKET 01 / CONTINUATION 0032 — CLOSE THE POST-PM TRANSACTION

User authority is explicit for a new named corrective packet and fresh review budget.

Identity and preserved subject:
- increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R5
- corrective packet: AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01
- task: 019fddab-1dd2-7570-9bd3-7f3841ce0283 / local
- worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original dispatch nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior continuation nonce: 0e8e152431fa298d6477b4cf1320cec307034fd3397052ed9914d15ed1f39ab4
- correction continuation nonce: 4ebe6614bf0515a5100e3d1598ddc3d51480de18ae8bbda650fa12c8916e674f
- base/tree/remote: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40 / exact
- current subject: sha256:d1f4e28e5bd3ca3ccb66e981397698e527403a647ce2103021debdae5a67007d
- tracked diff: sha256:cf835ac3a2a619636cce2b4cfb2ece6c2b4766fa4a11f27ab023a35161c67648
- status entries/staged: 19 / 0
- exhausted prior review record: sha256:db01b433cde39ed5ec8bd6a8ffbc628105a60f6ff2315edbd38501f25847eced

Reverify all bindings before edits. Preserve the worktree; no reset, clean, rebase, archive, donor-copy, native action, publication, or R6 work. This prompt is the complete inline authority; the meta JSON is not expected in the product worktree.

The prior four review artifacts are immutable at these exact hashes:
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-allowlist-evidence.md: sha256:d816b0e02ee9881c7ed4bf630cea31e53b2d9a34b047d0142455ae3d7d0f9396
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-authority-security.md: sha256:648cd48bbfef59c7c72c1e0616f95bd06971f02d22d3b8e7b49ab35de88d0d52
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-lifecycle-convergence.md: sha256:65c947ae63db721ed02dd7dfe6e783103f9d38d94a555ea6b2a53b9d8e89310a
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-cycle-record.json: sha256:db01b433cde39ed5ec8bd6a8ffbc628105a60f6ff2315edbd38501f25847eced
Do not edit them. Create a new correction review record and three lens files using the correction-01 names stated below.

## Mandatory remediation inputs

Resolve all four before the fresh discovery review:
1. P1-R5-POST-PM-INSTANCE-RECEIPT-PLAN-GAP
2. P1-R5-POST-PM-PREPARED-CAS-UNRECOVERABLE
3. P1-R5-POST-PM-GUEST-EXECUTOR-CIRCULAR-BOOTSTRAP
4. P2-R5-POST-PM-EXECUTOR-TRANSACTION-PROOF-GAP

### A. Complete the instance receipt plan

Keep Stage-1 Create as the sole cross-generation instance receipt. Add deterministic signed generation-two plans for the existing mac.lima.instance Start, Stop, Remove and Restore actions. Each plan exact-joins scope, attempt, entry, action, manifest and request identity. Typed lima-stop must find the exact signed Stop plan before any effect. Add no role/action pair and no generic action selector.

### B. Make prepared-before-effect CAS exactly recoverable

Classify an admitted request before the normal current-counter gate as exactly one of: new, exact prepared retry, exact effect-started retry, exact completed retry, or conflict.

An exact prepared/effect retry requires request counter + 1 == protected-state counter, the same current source anchor, and a signed prepared record matching request/receipt/entry/action/attempt/manifest/executor. Never allocate or increment again.

Use a no-follow, absent-or-exact executor-owned attempt journal below the existing lifecycle state root, joined to request and prepared hashes, with Prepared, EffectStarted, EffectObserved and Completed states. Persist EffectStarted before spawn and EffectObserved before receipt publication. Prepared with no effect marker can start once. EffectStarted must first run the exact action-specific read-only observation: if intended after-state is exact, finish the receipt without replay; if exact before-state remains, resume only the same idempotent closed primitive under the same journal; partial/ambiguous/mismatched state preserves and blocks. Completed returns only the durable exact receipt.

### C. Remove the circular guest-executor dependency

R5 must not invoke `/usr/libexec/substrate/substrate-lifecycle-linux r5-post-pm`; remove every such R5 call. Do not edit `src/bin/substrate-lifecycle-linux.rs`; it remains R6-owned.

The fixed macOS privileged executor must execute the existing closed R5 guest transitions itself using only the measured limactl image and an exhaustive role/action table of fixed guest executable paths and literal argv. It may use fixed `limactl copy` plus fixed guest install/chown/chmod/fsync/digest/rename commands for manifest/build-evidence-bound byte artifacts. Source and destination are derived only from the validated manifest, PM and retained build evidence—never caller input, repo/CWD/env or arbitrary payload. Non-byte effects use only fixed system executable/argv plans. Known-hosts stays descriptor/no-follow on the host.

The `mac.lima.publisher-executor` action installs and attests the fixed guest artifact for later use; it is not needed to execute R5. Its receipt completes only after exact path/digest/owner/mode observation. This is not pairing/session authority and adds no endpoint or transport.

### D. Real deterministic proof

Factor receipt planning, state classification, effect planning, artifact joins and retry decisions into pure helpers. Add Rust tests that execute those helpers—not lexical-only assertions—for:
- all four instance plans and Create/unlisted rejection;
- new/prepared/effect-started/completed/conflict counter/CAS states and kill boundaries;
- exhaustive accepted/rejected role/action effect plans and fixed argv;
- artifact source/target/digest/owner/mode joins and absence of r5-post-pm;
- exact after-state completion, exact before-state safe resume and ambiguous preservation.

Retain all prior MAC-only checks, installer fixture, formatting, diff/secret/path/cfg containment. No Linux/Windows target command and no native action.

## Exact file and review fences

Implementation/test paths remain exactly:
- crates/common/src/lib.rs
- crates/common/src/managed_artifact.rs
- crates/shell/src/execution/managed_lifecycle.rs
- crates/shell/src/execution/managed_lifecycle/macos_client.rs
- crates/shell/src/execution/mod.rs
- crates/shell/src/lib.rs
- crates/shell/tests/managed_lifecycle_v1.rs
- scripts/mac/lima-lifecycle.sh
- scripts/mac/lima-stop.sh
- scripts/mac/lima-warm.sh
- scripts/substrate/dev-install-substrate.sh
- src/bin/substrate-lifecycle-control.rs
- src/bin/substrate-lifecycle-macos.rs
- tests/installers/dev_install_bash32_fd_regression.sh
- tests/mac/lifecycle_r3.sh

The only new review paths are:
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-correction-01-review-authority-security.md
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-correction-01-review-lifecycle-convergence.md
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-correction-01-review-allowlist-evidence.md
- llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-correction-01-review-cycle-record.json

`06-review-finding-inventory.md` may change only to record a valid P3/P4. No other path is authorized.

## Fresh causal review budget

The old budget remains exhausted and immutable. This correction packet has a new budget:
1. after mandatory remediation, freeze the subject and run one fresh three-lens discovery burst using read-only gpt-5.6-terra Extra High reviewers;
2. remediate only valid P1/P2;
3. run one different fresh closure review;
4. allow at most two supplemental causal cycles only for P1/P2 directly caused/unmasked by the immediately preceding remediation;
5. record P3/P4 only; CLEAN ends the loop.

Use packet_id `AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01` and the four new correction review paths. If terminal CLEAN, validate the new record, run final MAC-only/change-detection/publication gates, publish the complete accumulated R5 subject as one normal fast-forward commit, and return increment AUX-R3-MAC-EVIDENCE-RECOVERY-R5, packet_id AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01, next_increment AUX-R3-MAC-EVIDENCE-RECOVERY-R6. If blocked, preserve everything and return the exact blocker. Send the receipt to meta as the final tool action; do not dispatch R6.
