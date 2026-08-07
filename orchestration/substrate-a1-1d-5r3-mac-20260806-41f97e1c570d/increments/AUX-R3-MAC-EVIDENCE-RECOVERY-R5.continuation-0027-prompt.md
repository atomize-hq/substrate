R5 CONTINUATION 0027 — BIND MANIFEST IDENTITY INTO THE FIRST ANCHOR

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior_continuation_nonce: 7541b1c6ca6fa045a27e2f28fc160dd853ba7cda1692e9927194d83bcd077202
- continuation_nonce: 614b644ffe0b074a92b0a1e3c08c8ecd0cd3979328e978c59a3c2b489438e693
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- expected preserved tracked diff SHA-256: e4a3efba520eaf074904803ded3914a8857319b3b1c0d76933e6870664c79577
- expected status entries: 13; staged paths: 0
- live remote must remain the expected base
- binding authority: authority-amendments/0027-r5-manifest-anchor-pre-pm-binding.json

Preserve the exact current thirteen-path subject. Do not reset, clean, rebase, bulk-copy the donor, publish, or begin R6. Reverify identity/base/tree/remote/index/diff before editing.

## Authoritative manifest-to-anchor correction

Add mandatory canonical `manifest_generation: u64` and `manifest_sha256: String` members to `PublisherBootstrapAuthorizationV1`. For the MAC R5 path, require a positive generation and a canonical lower-hex SHA-256. The canonical authorization digest must bind both values. FD3 still carries exactly one `PublisherBootstrapAuthorizationV1`; never send the seed, manifest bytes, wrapper, protected state, or a second frame.

The executor must copy those exact values into the first `LifecyclePublisherAnchorV1`; `request_sha256` remains the canonical authorization digest. The Prepared bootstrap intent, key, state, control-admission record and exact retries all join the same digest and manifest identity. Do not extend `LifecyclePublisherProtectedStateV1`.

Make this change only in the existing common type/validator and MAC R5 constructors/tests inside the preserved thirteen paths. Do not touch Linux or Windows source, constructors, targets, or behavior.

## Closed pre-PM MAC bootstrap-manifest exception

For greenfield direct publisher bootstrap only, permit `mac_host_shared` authorization and its bound canonical manifest to have both PM commitment fields absent. This is not a general relaxation. The exception is valid only when all are true:

- `publisher_expected_absent=true`;
- authority domain is `mac_host_shared` and platform kind is `mac_lima`;
- manifest generation is exactly 1;
- `previous_manifest_sha256` is absent;
- lifecycle state is `ManifestDurable`;
- entries and planned action receipts are empty;
- scope/installation, IH commitment, requester/intended principal, attempt, generation and digest exact-join the validated hidden direct seed; and
- the retained controlling-TTY confirmation succeeds.

Both PM fields must be absent together. A manifest PM commitment with absent authorization PM, or the inverse, rejects. This pre-PM manifest is bound in the authorization, Prepared intent and first signed anchor only; it authorizes no mapped action, ticket/pairing action, guest projection, forwarding, PM-dependent locator, or generic request.

After the sole Stage-1 absent-instance create finalizes PM, publish the strictly next full `mac_host_shared` manifest with both PM carrier/commitment present and equal, `previous_manifest_sha256` equal to the pre-PM digest, and CAS-advance the protected anchor to it before any post-PM or guest action. Outside this exact pre-PM shape, the existing `mac_host_shared` PM-required rule remains.

## Proof, review, and publication

Add focused tests for canonical manifest-field binding; missing/zero/bad/mismatched values; first-anchor exact copy; the exact pre-PM acceptance shape; every one-sided PM/non-first/nonempty/prior/wrong-state/IH/scope/principal/attempt negative; finalized-PM exact joins; and Stage-1 next-manifest/anchor ordering. Preserve all earlier FD3 peer/TTY/image/intent/key/state/admission kill/retry and replay negatives.

Run only MAC-native and x86_64-apple-darwin deterministic proof, formatting, exact thirteen-path/symbol/static containment, changed-byte secret scan and diff check. No native actions.

No review cycle has begun. After deterministic proof is green, freeze the exact subject and run one fresh same-subject discovery burst/review, then one different fresh closure review. At most two supplemental causal cycles may remediate newly caused/unmasked P1/P2; record P3/P4 only; CLEAN stops. Every subagent/reviewer uses gpt-5.6-terra at Extra High.

On terminal CLEAN, publish exactly one normal fast-forward R5 commit and return the original-dispatch-nonce `LANDED_CLEAN` receipt with `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6`. Do not dispatch R6.
