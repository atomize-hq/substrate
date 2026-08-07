R5 CONTINUATION 0025 — ADD ONLY THE MINIMAL COMMON BOOTSTRAP-AUTHORITY BINDING

Continue in the same task and preserved worktree:

- task_thread_id: 019fddab-1dd2-7570-9bd3-7f3841ce0283
- task_host_id: local
- exact_worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original_dispatch_nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior_continuation_nonce: f426b9c13f0079bc1e3d28ad13db88c61ff7144ee00198e01f11c561b9705c63
- continuation_nonce: ee590b74571010efafdb5df8d8f7b69ed439425b6f73894c5f8601e282c34915
- expected base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- expected preserved tracked diff SHA-256: 2ca7279d89fd38b7591508c309e0a75100e3627c9784029336184a241e097829
- expected status entries: 11; staged paths: 0
- live remote must remain the expected base

This is a same-packet R5 correction. Preserve the current partial work without reset, clean, rebase,
bulk donor copy, or publication. Revalidate the exact binding before further edits.

## Minimal common-schema authority

R4 already landed MacPublisherControlAuthorityV1 and its canonical validation. R5 may now change:

1. crates/common/src/managed_artifact.rs only to:
   - add optional PublisherBootstrapAuthorizationV1.mac_control_authority;
   - require and validate it for mac_host_shared;
   - require its source commit/tree/ref and target/build binding to join the authorization;
   - reject it for every non-mac_host_shared authority;
   - add publisher_bootstrap_authorization_sha256_v1 only if needed for exact retry; and
   - add focused inline canonical/required/mismatch/non-MAC/unknown-field tests.
2. crates/common/src/lib.rs only for the exact new digest-helper re-export, if used.
3. crates/shell/src/execution/mod.rs and crates/shell/src/lib.rs only for the already-preserved exact
   R5 type/function re-exports needed by substrate-lifecycle-control and
   substrate-lifecycle-macos. No other public API or behavior is allowed there.

Do not add fields to LifecyclePublisherProtectedStateV1. Avoid all Linux/Windows constructor churn.
For bootstrap durability and exact retry, bind the canonical authorization digest through the
existing signed LifecyclePublisherAnchorV1.request_sha256 and bind the already-existing fixed global
System-Keychain control-admission account to authorization.mac_control_authority. Before enabling
XPC, prove the account authority source/build/digest agrees with the protected anchor and current
authorization. A mismatch is preserving-first terminal failure.

## Complete the FD3 path without a new carrier

Finish the current R5 direct bootstrap using only the existing FD3 channel and the newly bound
authorization field:

- Obtain the requester PID from the kernel-owned AF_UNIX peer credential mechanism, not JSON.
- Resolve and hash the retained requester executable and compare it to mac_control_authority.
- Resolve and hash the running executor image and compare it to executor_build_evidence.
- Join the controlling terminal/session using kernel process/terminal observations.
- Accept exactly one canonical request frame and one bounded response frame. EOF, second frame,
  replacement, mismatch, timeout, cancellation, or handle loss preserves state and fails.
- The direct control issuer may derive the authorization only from the exact retained lifecycle
  capsule, published implementation/build evidence, current control executable identity, and fixed
  executor identity. No ambient/default project, donor state, caller request, environment/argv/file
  carrier, generated output, or reusable authorization is allowed.

The donor is read-only evidence only. Do not import its LifecyclePublisherProtectedStateV1 fields,
pairing/session types, generic request wrapper, or broader implementation.

## Subject, proof, and review

The R5 implementation/test subject is now exactly thirteen paths: the nine paths authorized by
0024 plus crates/common/src/managed_artifact.rs, crates/common/src/lib.rs,
crates/shell/src/execution/mod.rs, and crates/shell/src/lib.rs. lib.rs may remain unchanged if no new
helper export is needed. No other product/test path is mutable.

Run focused common authorization tests, the two existing R5 test files, native arm64 and
x86_64-apple-darwin checks for the touched common/shell/control/MAC surface, formatting, exact
path/symbol checks, static MAC-only platform containment, secret scan and diff check. Do not run or
edit Linux/Windows targets.

No review cycle has begun. After deterministic proof is green, freeze the exact subject and run the
normal fresh discovery burst/review plus a different fresh closure review. Remediate valid P1/P2
only, with at most two causal supplemental cycles; record P3/P4 in 06 without remediation cycles.

All prior R5 role/action, descriptor, audit-before-decode, caller-migration, no-generic-selector,
MAC-only, no-native-action, donor, GitNexus-degraded/manual-fallback, and one-fast-forward
publication boundaries remain. On terminal CLEAN, publish exactly one R5 commit and return the
original dispatch-nonce LANDED_CLEAN receipt with next_increment
AUX-R3-MAC-EVIDENCE-RECOVERY-R6. Do not dispatch R6. Use gpt-5.6-terra at Extra High for every
subagent and reviewer.
