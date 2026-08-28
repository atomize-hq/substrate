**Kind:** evidence/regression/status projection
**Stable ID:** `r3-mac-evidence-recovery-family`
**Canonical for:** MAC recovery evidence/regression status bundle from 05
**Status:** canonical
**Authority scope:** exact extracted 05 recovery-status source body only
**Source span:** [`../05-debug-regression-ledger.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../05-debug-regression-ledger.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06) lines 3619–3630; [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07) lines 3631–3641; [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07) lines 3642–3652; [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07) lines 3653–3656; [`../05-debug-regression-ledger.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10`](../05-debug-regression-ledger.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10) lines 3657–3687; [`../05-debug-regression-ledger.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13`](../05-debug-regression-ledger.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13) lines 3688–3714
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# R3 macOS evidence-recovery evidence/regression projection

## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Under `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` amendment
`0003-fresh-mac-review-epoch.json`, this new nonce-bound epoch reconstructs the verified
19-path attempt-3 baseline solely to remediate the six mandatory P1/P2 findings. It binds the
actual XPC peer audit token before request decoding, repeats canonical carrier/mapping/role and
Stage-1 joins before any mapped mutation, removes the standalone retire operation, and refuses
pre-spawn SSH-UDS replacement by disabling SSH-side unlink. The added checks are non-native only:
no Lima, launchd, Keychain, code-signing, publisher installation, or evidence artifact is run or
created here. Fresh review is recorded only in the exact MAC review-control set; the sole
successor remains `EVIDENCE:R3-MAC-IMP-01`.
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../05-debug-regression-ledger.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), baseline lines 3619–3630
**Baseline span SHA-256:** `60fee5816a3b3372f02f14dfe172ac390e0a6eb807d4a07628645ed8c980e3b2`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression status (2026-08-07)

Recovery requires separate non-native proof for: Bash-3.2 caller-FD preservation; macOS cfg and
installer lifecycle-binary compile surface with Linux behavior unchanged; dispatch-supplied expected
project-ID validation including historic Linux evidence; audit-token-before-XPC-decode and
canonical P-256/Stage-1/ticket CAS negatives; raw `lima-action`/direct-helper rejection; and the
same-PM dual data/independent-TTY binding, no-confirmation-leak, retry/expiry/replay preservation
matrix. These checks do not substitute for native publisher, Keychain, XPC, Lima, session, or
restoration evidence. The latter remains `EVIDENCE:R3-MAC-IMP-01` and returns the platform-handoff
status if the supported host cannot prove the required two-session capability.
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07), baseline lines 3631–3641
**Baseline span SHA-256:** `78778c09bde63892d75000b390c4e817a4559d5c566e9246f055c6d213a6c6c3`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression-command and status correction (2026-08-07)

The recovery evidence validator command must include the fresh dispatch-bound
`--expected-product-project-id`; direct test coverage rejects missing/mismatched values and validates
the historical Linux artifact with `2ccb802f-301c-4af4-9bd5-51d22808f0a2`. For native evidence,
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` means the supported platform/privilege/capability was unavailable
before action; an available action, validator, or restoration-parity failure is
`BLOCKED_NATIVE_EVIDENCE`. Earlier MAC attempt-4 status is historical only and cannot bypass the
future recovery implementation receipt prerequisite.
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07), baseline lines 3642–3652
**Baseline span SHA-256:** `18a0968050eac1610755bdc920a8cb8581f6acf4697d0c5501b7f15a60b1ba07`

## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`

**Source provenance:** extracted from [`../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../05-debug-regression-ledger.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07), baseline lines 3653–3656
**Baseline span SHA-256:** `53c12c7d745bd67b8b15582f88beecdd4b8a4d6745c00cd2bf355c7744f1dba1`

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION regression ledger (2026-08-10)

| Observation | Reproduced result | R3 contract consequence |
|---|---|---|
| Root LaunchDaemon exact-tag lookup through default, Data Protection Keychain, and Data Protection plus `SessionCreate` routes | all stopped before creation with OSStatus `-25291` (`errSecNotAvailable`); no key or UI was created | Secure Enclave/Data Protection is not an R3 product route and cannot justify a root-LaunchDaemon non-exportability claim |
| Public explicit `/Library/Keychains/System.keychain` generic add/read/delete | root, scrubbed-environment probe opened and exact-path-verified the Keychain, round-tripped the exact service/account/data, deleted it, and proved `-25300` final absence with no UI | public `SecKeychainOpen`, add-only `kSecUseKeychain`, search-list scoping, UI-fail, and final-absence verification are the required routing pattern |
| Public software P-256 creation in the explicit System Keychain | `SecKeyCreateRandomKey` created a permanent tagged P-256 key and signing worked, but private external representation succeeded even where the extractable attribute reported false | R3 uses a software P-256 key and explicitly accepts privileged-root export capability; code/tests/docs must not assert hardware backing or non-exportability |
| Secure Enclave plus explicit legacy System-Keychain selection | public attempt failed with OSStatus `-50`, created nothing, and restored exact absence | no private API, implicit store, or Secure Enclave fallback is permitted |
| Nonce-scoped public-route root-LaunchDaemon regression | `/Users/spensermcconnell/.codex/evidence/system-keychain-software-signer-correction/c708627e-system-keychain-20260810T151614Z-7c81ad4e/receipt.md` (`fef2ca75b601cf828653ecef584c4fbdb926386ba5f4e6b678dcff6ea03398ec`) records create, exact persisted-attribute reopen, restart/same-public-point sign/verify, expected 97-byte private export, exact deletion/`-25300` absence, and launchd/plist/process restoration; artifact manifest SHA-256 `1bc70bf1436427a07e577cdbcbc8a1430852c562d400eaffcba8d6efcc1a0032` | the final public route is headless and retry-safe on Apple Silicon; this bounded product-independent proof does not run or authorize product lifecycle, Lima, pairing, `EVIDENCE:R3-MAC-IMP-01`, or MAC closeout |

The corrective regression first failed on private `kSecUseSystemKeychain`. It now requires the
exact System-Keychain path constant and returned-path check; distinct add/search routing; UI-fail;
all-match duplicate detection; exact service/tag/private-class/P-256-size/permanent/sign-capable
validation; creation followed by exact reopen and SPKI comparison; no product private export; and
retirement that refuses deletion while the protected wrapper survives, deletes only the exact tag,
and verifies final absence. Missing, mismatched, duplicate, ambiguous, orphaned, or surviving-
wrapper state is preserving-first. The signed wrapper/anchor, monotonic CAS, receipt/retry,
audit-token/designated-requirement admission, canonical SPKI, P1363 low-S, and Apple-Silicon-only
support boundary remain unchanged.

Fresh causal discovery found that the first correction checked wrapper absence and then deleted the
tag without sharing the wrapper CAS lock, allowing wrapper creation to cross retirement. The
remediation moves key/SPKI validation under the exact current-anchor CAS lock for every protected
wrapper write, holds that same lock across retirement's wrapper check/delete/final-absence
sequence, and deletes through the validated key reference with public `kSecMatchItemList` rather
than a second tag-wide query. The regression fixes the required lock order and item selector.

Secure Enclave/Data Protection Keychain and a user LaunchAgent signer are deferred hardening only.
They are not part of this increment, do not authorize `EVIDENCE:R3-MAC-IMP-01`, and do not imply an
automatic successor dispatch. Intel/T2 remains out of scope.
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10`](../05-debug-regression-ledger.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10), baseline lines 3657–3687
**Baseline span SHA-256:** `a3fd63fdf019e248480da69ae308442cf6cdb72fa299a633d8e0a0fa4d03856a`

## R3 macOS retirement/orphan planning and G2/G3 stop ledger (2026-08-13)

This entry records bounded planning observations plus one independently reviewed G2 pre-effect stop. It does not resolve the SecurityAgent cause, run a native experiment arm, query or mutate the live signer, authorize recovery, implement V2 retirement, perform cleanup, refresh the evidence mirror, create native evidence, or authorize `MAC-CLOSEOUT`.

| Observation | Current bounded record | Consequence |
|---|---|---|
| SecurityAgent contradiction | SecurityAgent appeared during two prior cleanup attempts despite query-level `kSecUseAuthenticationUIFail`. No credential or approval was supplied. The exact Security-framework suboperation requesting interaction is unresolved. | Query-level failure alone is not accepted as proof of a no-UI route. No prompt is expected or authorized by this ledger. |
| Preserved precommit-less orphan | Scope `019ff983-39ca-7182-9db8-b86aa66fa443` remains bound to the failed attempt. External cleanup records report the publisher registration absent and fourteen exact lifecycle generic-password deletions with final absence. The P-256 signer was last observed as one exact item and was not re-queried by this docs task. | Do not recreate state, insert a retroactive V2 precommit, or use the prospective finalizer. Preserve signer and evidence pending separate authority. |
| Disposable experiment | Task `R3-MAC-RETIRE-G2-G3-DISPOSABLE-EXPERIMENT` attempted to freeze the authorized packet and stopped before effects. The accepted contract does not yet bind the exact prospective-finalizer executable/code/privilege identity or the canonical delete-only `SecAccess` posture; inventing either is prohibited. The creator matrix also retains two executable-detail stops: prior-interaction-setting handling for the first-call process-deny arm and precommitted wrong-identity status classification. No experiment executable or surrogate was created and no native arm ran. | Creator-recovery and prospective-finalizer conclusions remain separate and are both `NOT_RUN`. Independent review accepted `RESTORED_BY_NO_EFFECT`; G3 is blocked and G4 remains unauthorized. See `docs/guidance/2026-08-13-r3-mac-g2-g3-disposable-experiment-pre-effect-stop.md`. |

Future evidence/hash slots remain empty except for the bounded G2/G3 stop receipts:

| Slot | Status |
|---|---|
| Disposable experiment authorization ID/digest | `R3-MAC-RETIRE-G2-G3-DISPOSABLE-EXPERIMENT`; packet `a8a59811472e6c285ce12515559ca70848485dbbd9a6c1cfb1788d0571024d9d`; `BLOCKED PRE-EFFECT` |
| Disposable experiment receipt and baseline-restoration manifest SHA-256 | stop receipt `08c26343e49c3698e689dcb81c4072085e730bd51595117f96cad33283647c0a`; restoration manifest `eb5e03fa9579247047101f6d75cfe05dc5360ea0e4999836855505757c584a8c`; independent review `3bcd7e87dfbad729c5eab27f8e3f7fa46fa79be5e7fd022c615fea22a18f24c6` (`PASS_RESTORED_BY_NO_EFFECT`) |
| Creator-recovery no-UI conclusion receipt SHA-256 | `15134c8472fd032853f43fa816c96f43192a1b08250cdfe10a14a1769863b28c`; `NOT_RUN` |
| Prospective-finalizer delete-only capability receipt SHA-256 | `a730696c4e24210fac9e3111e74379e5d3d3a10e0c91cb0239af363a19fd11a0`; `NOT_RUN` |
| Exact orphan-recovery authorization ID/digest | `PENDING / NOT AUTHORIZED` |
| Exact orphan-recovery receipt SHA-256 | `PENDING / NOT RUN` |
| Exact orphan baseline-parity proof/acknowledgement SHA-256 | `PENDING / NOT RUN` |
| Prospective V2 contract-closure digest | `PENDING / NOT CLOSED` |
| Separate prospective implementation authority ID/digest | `PENDING / NOT AUTHORIZED` |
| Native macOS evidence artifact/receipt SHA-256 | `PENDING / NOT RUN` |
| Evidence-mirror refresh receipt SHA-256 | `PENDING / NOT AUTHORIZED` |
| `MAC-CLOSEOUT` receipt SHA-256 | `PENDING / NOT AUTHORIZED` |
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13`](../05-debug-regression-ledger.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13), baseline lines 3688–3714
**Baseline span SHA-256:** `f8ec5ed60e6fe060a50cc316636c8a1e20c6bfe9a4b6b2690e687747e79415cd`
