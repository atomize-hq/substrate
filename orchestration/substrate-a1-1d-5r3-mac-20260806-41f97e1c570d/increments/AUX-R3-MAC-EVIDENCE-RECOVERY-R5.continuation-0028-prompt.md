R5 CONTINUATION 0028 — REMEDIATE THE VALID DISCOVERY P1/P2 FINDINGS

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior_continuation_nonce: 614b644ffe0b074a92b0a1e3c08c8ecd0cd3979328e978c59a3c2b489438e693
- continuation_nonce: 272ee6e53079be7e14091241bc84dc22a15f2d0d2b04999e6158e3bb2e713c93
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- expected frozen implementation subject: sha256:a69dce04d313466f15f5c2284fe60fe4531a9f946ec53f9aca7f0b181e8c9147
- expected status entries: 17; staged paths: 0
- live remote must remain the expected base
- authority: authority-amendments/0028-r5-provenance-stage1-admission-receipt-remediation.json

Preserve the current thirteen implementation/test paths and four review artifacts. Do not reset, clean, rebase, bulk-copy the donor, publish, or begin R6. Reverify base/tree/remote/index/inventory and the review record before editing.

The seven discovery findings are valid and are now authorized for bounded R5 remediation. This remains MAC-only. It adds exactly two implementation/test paths: `scripts/substrate/dev-install-substrate.sh` and `tests/installers/dev_install_bash32_fd_regression.sh`. No other new path is allowed.

## 1. Root-owned retained bootstrap provenance

Add canonical `MacPublisherBootstrapProvenanceV1` with a fixed path:
`/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json`.

Only the MAC installer branch may create it, after both fixed MAC binaries and their managed-copy manifest are durable. It is root:wheel 0600 under a root-owned no-follow/fsynced parent and records source commit/tree/ref, reviewed R5 review-record digest, both image target triples/digests/no-follow physical identities/code requirements, the canonical pre-PM manifest identity, and IH commitment. Creation is absent-or-exact; mismatch or partial state preserves and blocks—never overwrite, repair, or adopt.

The hidden direct command treats stdin only as a requested seed. Before deriving authorization it must open the fixed provenance and both installed images no-follow, verify owner/mode/parent/file identities, recompute all hashes/code identities/source/review/manifest joins, and reject any seed value that does not exactly match retained provenance. No caller path, env/argv/file selector, project/CWD/Git/remote lookup, or ambient default is permitted.

## 2. FD3 peer admission and framing

Before decoding any FD3 authority bytes, use the kernel peer PID; resolve and open the peer executable with O_NOFOLLOW|O_CLOEXEC; fstat/hash via the descriptor; re-observe the peer path/identity; and compare digest, physical identity and code requirement to retained provenance/control authority. Decoded bytes only further constrain retained authority.

Apply a fixed bounded timeout to the initial FD3 frame and the response/EOF proof. Timeout, EOF, extra frame, peer exit, path/identity change or mismatch preserves state and fails.

## 3. Scope-bound global control admission

Replace the bare Keychain control authority with canonical `MacPublisherControlAdmissionV1` containing scope, control authority, bootstrap authorization digest, manifest generation/digest, current anchor digest and state revision. Listener startup and every XPC admission must exact-join this record to current protected state, signed anchor, measured peer and request context before request decode. Update it only with the same preserving anchor transition; mismatch blocks.

## 4. Signed Stage-1 successor manifest and authoritative lifecycle receipt

Extend signed `LimaStageOneAuthorizationV1` itself—not the generic request or wrapper—with the complete canonical `next_pm_manifest`, expected pre-PM anchor digest/counter, and exact planned Stage-1 receipt identity. Its existing lifecycle signature covers every byte.

The next manifest must be `mac_host_shared/mac_lima`, generation current+1, previous digest equal to the pre-PM manifest, and exact-join IH/scope/principal/source/executor/attempt. After the observed absent-instance create returns machine identity, derive/finalize PM and require the signed manifest's PM/control-root/instance/machine projection to match before publication.

Use the existing `ManagedActionPreparedRecordV1`, `ManagedActionReceiptV1`, receipt-index/head helpers and protected-state CAS. Remove the overwriteable `last-action-receipt.v1.json` shortcut. Ordering is:

`prepared allocation -> effect -> durable full PM manifest/empty index/head -> signed action receipt -> receipt index -> head -> next signed anchor -> protected-state/admission CAS -> bounded caller observation`.

Exact matching partial state resumes. Missing/mismatched/ambiguous state preserves and blocks. No post-PM or guest action is accepted before this transaction completes.

## 5. Ordinary request closure

Both ordinary tags reject every legacy manifest/protected-state/action-receipt/pairing/bootstrap field and every cross-tag/unknown field. Recompute and exact-join IH, PM, scope, principal, control root, instance/machine, source/build, current anchor, manifest and role/action before XPC/effect. Stage-1 remains absent-instance-create only; post-PM remains the closed role/action table only. Every ordinary action uses the authoritative prepared/receipt/index/head/anchor transaction.

## Proof and review

Add focused tests for all seven finding IDs, including provenance install/retry/tamper, peer-image TOCTOU/hash, FD3 timeout/framing, admission-anchor rollback/mismatch, Stage-1 next-manifest and each transaction kill/retry boundary, closed ordinary request shapes/joins, and authoritative receipt ordering. Preserve all earlier R5 tests.

Run only MAC-native and x86_64-apple-darwin checks, the MAC installer regression, formatting, exact fifteen implementation/test paths plus four review paths, static MAC-only containment, changed-byte secret scan and diff check. No native actions and no Linux/Windows target commands.

Discovery-1 already exists. After remediation, freeze the new subject and run `closure-1` using a different fresh gpt-5.6-terra Extra High read-only reviewer. At most two supplemental causal cycles may remediate newly caused/unmasked P1/P2; record P3/P4 only; CLEAN stops.

On terminal CLEAN, publish exactly one normal fast-forward R5 commit and return the original dispatch nonce `LANDED_CLEAN` receipt with `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6`. Do not dispatch R6.
