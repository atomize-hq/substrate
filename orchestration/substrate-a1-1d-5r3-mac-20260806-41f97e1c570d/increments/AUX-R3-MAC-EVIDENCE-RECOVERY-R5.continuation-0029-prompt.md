R5 CONTINUATION 0029 — CORRECT STAGE-1 TEMPORAL MODEL AND COMPLETE THE SAME PACKET

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior_continuation_nonce: 272ee6e53079be7e14091241bc84dc22a15f2d0d2b04999e6158e3bb2e713c93
- continuation_nonce: bb14577c654e5fbdb30bb981d4f0d712acd04c960219939f9b26b789c0f6420a
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- expected tracked diff: sha256:ed4508ec3156589d8977679ce700dd6a2ec36105253ada38d28f6fcf26a6d453
- expected combined tracked+untracked subject: sha256:b6b7b8ce97fd9cdc6ecd023fde3aa816f1b187cccbff431870d75afdce4ccef9
- expected status entries: 17; staged paths: 0
- live remote must remain the expected base

First reverify every binding above, the exact 17-path inventory, diff-check, and the existing VALID bounded-stop discovery review record at `sha256:6eea8804d5c027ed5272f139afdef20ee752a32e86e132fdc117e5816738fb85`. Preserve the worktree: no reset, clean, rebase, archive, donor bulk-copy, native action, publication, or R6 work.

This inline prompt plus its nonce is the complete authority. The meta-only amendment file `authority-amendments/0029-r5-stage1-template-cross-generation-completion.json` is not expected in your task worktree; its absence is not a blocker.

## Correction to continuation 0028

Continuation 0028 was wrong only in requiring a complete post-PM `next_pm_manifest` to be signed before the absent Lima instance is created. Supersede that requirement. Retain every other 0028 remediation: root-owned bootstrap provenance, FD3 kernel-peer image binding and bounded framing, global control admission, closed ordinary requests, authoritative receipts, the exact path fence, MAC-only proof, and the seven discovery findings.

You may surgically revise or supersede the incomplete Stage-1 schema/code already present in the 17-path subject. Do not discard unrelated valid partial work.

## 1. Signed pre-effect successor template

Add canonical `MacLimaStageOneSuccessorTemplateV1` as a field of signed `LimaStageOneAuthorizationV1`.

Before create/start there is no `PlatformBootstrapMappingV1`. The Stage-1 authorization is the sole pre-PM mapping authority. It MUST NOT contain a fabricated PM commitment, placeholder machine ID, or complete post-PM manifest.

The signed template fixes:
- schema/derivation algorithm;
- current pre-PM manifest generation/digest and current anchor digest/counter (counter 0 is valid for the exact initial pre-PM anchor);
- next generation exactly current+1 and previous digest exactly current pre-PM digest;
- IH, scope, installation, intended principal, prefix/control root, instance/profile, source/executor/attempt/nonce/expiry;
- fixed manifest metadata/time policy, ordered non-machine-dependent entries, and exact planned receipt ID/path;
- exactly one post-effect observation slot: `guest_machine_id`.

The lifecycle signature covers every byte. Its signing public key must exact-match the current anchor/protected publisher key; a self-selected embedded key is not authority.

## 2. Deterministic post-effect derivation

After the exact selected create/start effect, the privileged macOS publisher/executor may observe only `guest_machine_id`. Using the existing canonical Lima mapping constructor, derive `PlatformBootstrapMappingV1`, its commitment, and the complete generation+1 `mac_host_shared/mac_lima` manifest. Every other byte comes from the signed template. Validate the canonical full manifest and all IH/scope/principal/control-root/instance/source/executor/attempt joins.

Use the existing System-Keychain publisher key for receipt and anchor signing. No new signer, transport, endpoint, session, FD3 frame, protected-state field, or R6 surface is authorized.

## 3. Specialized cross-generation completion

The generic same-manifest receipt validator is not suitable and must not be weakened. Add one R5 Stage-1-only specialized validator/completer (name may follow existing conventions, e.g. `validate/complete_mac_lima_stage_one_transition_v1`) with this exact transition:

1. source protected state/anchor is pre-PM generation N; signed-anchor counter 0 is valid initially;
2. durable prepared record is bound to generation N and allocates action counter current+1;
3. perform the selected absent-instance effect;
4. derive and validate full PM generation N+1 manifest from the signed template plus observed machine ID;
5. create/sign `ManagedActionReceiptV1` binding source anchor/prepared digest, exact PM observation, and generation N+1 manifest;
6. publish generation N+1 manifest, empty index/head, receipt file, receipt index, and receipt head using no-follow absent-or-exact semantics;
7. sign the generation N+1 anchor;
8. CAS protected state from the exact source revision to the N+1 anchor with `prepared_record=None`, counter advanced exactly once, and revision advanced exactly once;
9. CAS `MacPublisherControlAdmissionV1` to the exact authorization/manifest/anchor/state join;
10. return only the durable receipt observation.

This is the sole exception to generic same-manifest receipt validation. `LifecyclePublisherProtectedStateV1` remains unchanged. Exact partial state resumes; ambiguity or mismatch preserves and blocks. Never repeat an ambiguous effect, invent machine identity, overwrite a mismatched file, or treat `last-action-receipt.v1.json` as authoritative.

## 4. Finish the retained 0028 remediation

Complete the trusted fixed-path root provenance, no-TOFU FD3 peer image verification, timeouts/framing, global admission joins, and both ordinary request tags. Fix the existing `publisher_request_v1` versus `publisher_request` wrapper/decoder mismatch. Ordinary post-PM actions keep the generic same-generation receipt path; only Stage-1 uses the specialized transition.

The exact allowed implementation/test paths remain the prior fifteen paths; review metadata remains the four existing R5 files. No new path is authorized.

## Proof and review

Add focused negative and kill/retry coverage for the template signature/anchor join, valid counter zero, no placeholder PM, single observation slot, mapping/manifests, specialized N→N+1 ordering, generic-validator isolation, ordinary field-name/closed-shape behavior, and every retained 0028 finding. Run only MAC-native and x86_64-apple-darwin checks, installer regression, formatting, path/symbol fence, static MAC-only containment, changed-byte secret scan, diff check, and permitted change detection/manual fallback. No Linux/Windows target command and no native action.

`discovery-1` already exists. After remediation freeze the new subject and run `closure-1` with a different fresh read-only gpt-5.6-terra Extra High reviewer. At most two supplemental causal cycles may address P1/P2 directly caused or unmasked by immediately preceding remediation. Record P3/P4 only. CLEAN ends the loop.

On terminal CLEAN, publish exactly one normal fast-forward R5 commit and return the original dispatch nonce `LANDED_CLEAN` receipt with `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6`. Do not dispatch R6.
