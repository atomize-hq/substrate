R5 CONTINUATION 0026 — BIND THE DIRECT SEED AND CREATE THE FIRST SIGNED ANCHOR

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior_continuation_nonce: ee590b74571010efafdb5df8d8f7b69ed439425b6f73894c5f8601e282c34915
- continuation_nonce: 7541b1c6ca6fa045a27e2f28fc160dd853ba7cda1692e9927194d83bcd077202
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- expected preserved tracked diff SHA-256: 1980e47680d0d775a43de622da8622bbd8c7d8c76273b9693e4d061b674b0463
- expected status entries: 13; staged paths: 0
- live remote must remain the expected base

Preserve the exact current 13-path subject. Do not reset, clean, rebase, bulk-copy the donor, publish, or begin R6. Reverify identity/base/tree/remote/index/diff before editing.

## Correction to the direct-bootstrap source rule

Amendment 0025 over-banned every seed request. The nonserialized object is the complete `PublisherBootstrapAuthorizationV1`, not the canonical evidence seed from which the hidden direct command independently derives it.

Only `substrate-lifecycle-control publisher-bootstrap` may read exactly one canonical `ManagedLifecycleControlRequestV1` seed from stdin. That seed is untrusted derivation input and may contain only the exact MAC host scope/IH/optional-PM/manifest/build fields named in amendment 0026. It must have no mapped tag, action receipt, protected state, publisher request, Stage-1 authorization, pairing ticket, complete bootstrap authorization, caller target, caller code requirement, or caller authorization digest. The wrapper and ordinary mapped-submit route remain unable to invoke this command.

Decode and recompute every join before opening the FD3 effect path. Derive the complete authorization in memory from the exact IH carrier, optional finalized PM carrier, canonical manifest/bootstrap seed, published source/build evidence, current control image/code identity, fixed executor image/evidence, stable manifest attempt/time data, exhaustive component map, and the successful literal controlling-TTY confirmation. Send that authorization only once over FD3; never write it to a file, environment, argv, normal request, or output. Exact retry requires the same canonical seed and authorization digest.

For the pre-PM host-publisher posture, PM fields are both absent. For a finalized-PM posture, both carriers/commitments are present and exact-join. Never discover or select a default/ambient VM or mapping.

## Correct the image binding

`mac_control_authority.artifact_sha256` is the measured `substrate-lifecycle-control` image. `executor_build_evidence.artifact_sha256` is the measured `substrate-lifecycle-macos` image. They are expected to differ. Remove the current erroneous equality requirement. Exact-join source commit/tree/ref and target triple, but independently measure and compare each image, physical identity, and code identity. A MAC-only reviewed-implementation-receipt digest field on `MacPublisherControlAuthorityV1` is authorized if needed for the control-pack provenance join; no Linux/Windows constructor or behavior change is authorized.

## Greenfield bootstrap does not require an old anchor

`publisher_expected_absent=true` means the initial publisher protected state is absent. It does not require an impossible pre-existing signed anchor. After validating the FD3 kernel peer, images, build/source evidence, TTY, time, component set and authorization, create the first anchor through this exact System-Keychain transition:

1. preflight the fixed global control authority;
2. absent-create or exact-join a scope-bound canonical `Prepared` bootstrap-intent record binding authorization digest, control authority, executor evidence and attempt;
3. create or exact-reopen only the intent-bound non-exportable P-256 key and verify its SPKI;
4. build/sign the initial `LifecyclePublisherAnchorV1` with `request_sha256` equal to the canonical authorization digest;
5. CAS the existing `LifecyclePublisherProtectedStateV1` shape with no new fields;
6. persist or exact-join the fixed global control-admission account; and
7. mark the intent Completed and return one bounded response.

Exact retries may resume only from a matching Prepared/Completed intent, key, state and admission record. An orphan key without a matching intent, any mismatched digest/source/image/control account, or any ambiguous state preserves everything and blocks. Do not delete, replace, adopt, rotate, or synthesize on mismatch.

## Scope, proof, review, publication

The mutable implementation/test subject remains exactly the current thirteen paths listed in amendment 0026. Do not add a path. Do not expand `LifecyclePublisherProtectedStateV1`. Do not touch Linux/Windows code, targets, or behavior. Do not perform native install, signing, Keychain, XPC, Lima, lifecycle, or evidence actions in this implementation task.

Add focused tests for exact seed admission/rejection; independent control/executor measurements; pre-PM versus finalized-PM rules; first-anchor creation; every intent/key/state/admission kill/retry boundary; orphan/mismatch preservation; and the existing FD3/TTY/timeout/replay negatives. Run only MAC-native and x86_64-apple-darwin deterministic proof plus formatting, exact allowlist/symbol/static containment, changed-byte secret scan and diff check.

No review cycle has begun. After deterministic proof is green, freeze the exact subject and run the repository causal review sequence: one fresh same-subject discovery burst/review, then one different fresh closure reviewer; at most two causal supplemental cycles for newly caused/unmasked P1/P2; P3/P4 record-only; CLEAN stops. Use gpt-5.6-terra at Extra High for all subagents/reviewers.

On terminal CLEAN, publish exactly one normal fast-forward R5 commit and return the original-dispatch-nonce `LANDED_CLEAN` receipt with `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6`. Do not dispatch R6.
