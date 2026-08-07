R5 CONTINUATION 0031 — RECOVER EXACT BOOTSTRAP ATTEMPTS AND COMPLETE CLOSED POST-PM TRANSACTIONS

Continue in the same preserved task/worktree:
- thread/host: 019fddab-1dd2-7570-9bd3-7f3841ce0283 / local
- worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original dispatch nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior continuation nonce: e8b183d657cdfebb9890749939737d814461ff1b801d796d95c0987d5b479b5d
- continuation nonce: 0e8e152431fa298d6477b4cf1320cec307034fd3397052ed9914d15ed1f39ab4
- base/tree/remote: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40 / exact
- subject fingerprint: sha256:f3cd94a1c5e7de3dd531c30add61f8976d69fbacdad9db3b267364b201a4a5ab
- tracked diff: sha256:ea87ea0e647dbf14e1b3c4ce583f79692b2b8040f34d6945586415d12ef6c5a7
- status: 19 entries, zero staged paths
- review record: VALID bounded_stop sha256:f1db2ee8993b3ab7dad0bf645217ec4d13b14ed31251f5917223744cd54d601f
- completed cycles: discovery-1, closure-1, supplemental-causal-1

Reverify every binding before edits. Preserve the current 19-path subject. This inline prompt and nonce are complete authority; the meta-only amendment JSON is not expected in your worktree. Do not reset, clean, rebase, archive, donor-copy, run native actions, publish early, or begin R6.

Both blockers are valid R5-owned findings, not successor work:
- P1-R5-IH-ONLY-RANDOMIZED-DIRECT-BOOTSTRAP-RETRY
- P1-R5-POST-PM-NO-AUTHORITATIVE-TRANSACTION

## 1. Exact recoverable direct bootstrap

Keep the single canonical IH-only MacPublisherBootstrapRequestV1 FD3 request/response. Do not add scope, attempt, locator, authorization, manifest, time, path or retry fields to the request and do not widen ordinary stdin/argv/env/file carriers.

After the existing kernel-peer/image, terminal and install-provenance admission, derive a fixed attempt key from the authority domain, canonical IH commitment, admitted requester/control principal, selected prefix, install-provenance digest and source commit/tree/ref. Add a separate no-follow, CAS-updated System-Keychain attempt index/locator under the existing service/account conventions; do not add a LifecyclePublisherProtectedStateV1 field.

Persist the locator before authorization/key/capsule effects. It must retain enough exact original facts to reopen the original scope/attempt/times/nonces, request+authorization digests, capsule/protected-state identity, provenance/anchor joins, terminal state and canonical response. Absent allocates once. Exact incomplete/completed retries reopen and return/reconstruct the byte-identical response without a new UUID or time. Concurrent exact requests converge. Corrupt, mismatched, expired-incomplete, ambiguous or nonrecoverable state preserves and blocks. No second FD/frame, endpoint, transport, caller locator, generic retry broker, ambient discovery or complete-authorization serialization.

## 2. Complete the R5 closed post-PM transaction

Do not defer this to R6: the recovery PLAN/SPEC assigns ordinary post-PM mapped lifecycle effects and durable receipts to R5; R6 is pairing only.

Implement a closed execute_mac_post_pm_action_v1 path beneath execute_mac_managed_action_v1. Before effect, validate the exact current protected state, signed anchor, PM, manifest entry, request digest, closed role/action pair, install provenance, measured tool/artifact identities and no-follow predecessors. Persist the prepared record via protected-state CAS.

Use one exhaustive typed match over the existing R5/control-pack role/action table. Derive all instance, host, guest, unit, socket, binary, known-hosts, directory, ownership, mode and service targets only from the validated PM/manifest and fixed role rules. Use only fixed measured limactl/system tools, fixed installed/signed artifact bytes or embedded fixed unit bytes, and literal role-specific command families. Complete the existing warm/stop set for Lima instance state, guest layout/group/membership/private home/sentinel/binaries/units/publisher components/known-hosts and service/socket states. Internal PM-bound limactl shell/data use is permitted only as a fixed executor-owned lifecycle primitive; it is not caller-visible transport authority and must not implement pairing.

Never accept a caller command/path/payload/argv/env/tool, raw helper relay, arbitrary shell, generic operation string, generic selector, new endpoint or new transport. lima-warm.sh/lima-stop.sh must use only typed stage_one_create/post_pm_action requests.

After strict observation, publish the authoritative signed receipt file -> index -> head -> next signed anchor -> protected-state CAS -> global-admission CAS before success. Exact retry resumes; ambiguous/mismatched effect preserves and blocks without repeating/adopting. last-action-receipt.v1.json is diagnostic only.

## 3. Exact scope and proof

The authorized implementation/test paths remain exactly the existing fifteen:
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

Review metadata remains exactly the existing four R5 review paths. `06-review-finding-inventory.md` may change only if a valid P3/P4 must be recorded. No other path is authorized.

Add deterministic coverage for lost-response/completed/concurrent direct-bootstrap retries; corrupt/mismatched/expired locator; every locator/authorization/capsule kill boundary; request-carrier non-widening; exhaustive allowed and rejected post-PM pairs; prepared/effect/observation/receipt/index/head/anchor/state/admission kill and retry boundaries; raw warm/stop absence; and all earlier R5 findings. Run only the existing MAC-native/x86_64-apple-darwin deterministic checks, installer fixture, formatting, exact path/symbol fence, MAC-only containment, changed-byte secret scan and diff check. No Linux/Windows target commands and no native action.

## 4. Final causal review and publication

This is the final authorized causal remediation. Bind both P1s above as findings caused or unmasked by the immediately preceding remediation. Freeze the complete subject and run supplemental-causal-2 with one fresh different read-only gpt-5.6-terra Extra High reviewer. There is no further remediation cycle: CLEAN ends the loop and permits exactly one normal fast-forward R5 commit; any remaining/new P1/P2 must return validated bounded_stop/budget_exhausted. Record P3/P4 only.

On CLEAN, publish R5 and return next_increment AUX-R3-MAC-EVIDENCE-RECOVERY-R6. Otherwise preserve the worktree and return the exact blocker. In either case, send the receipt to the meta task as your final tool action and do not dispatch R6.
