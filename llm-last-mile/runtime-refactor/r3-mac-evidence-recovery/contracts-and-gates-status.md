**Kind:** contract/gate/status projection
**Stable ID:** `r3-mac-evidence-recovery-family`
**Canonical for:** MAC recovery contract/gate status bundle from 04
**Status:** canonical
**Authority scope:** exact extracted 04 recovery-status source body only
**Source span:** [`../04-contracts-and-gates.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../04-contracts-and-gates.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06) lines 10044–10055; [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07) lines 10056–10069; [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07) lines 10070–10091; [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07) lines 10092–10095; [`../04-contracts-and-gates.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](../04-contracts-and-gates.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10) lines 10096–10126; [`../04-contracts-and-gates.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only`](../04-contracts-and-gates.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only) lines 10127–10234
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# R3 macOS evidence-recovery contract/gate/status projection

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
**Source provenance:** extracted from [`../04-contracts-and-gates.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../04-contracts-and-gates.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), baseline lines 10044–10055
**Baseline span SHA-256:** `60fee5816a3b3372f02f14dfe172ac390e0a6eb807d4a07628645ed8c980e3b2`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN recovery decision (2026-08-07)

For MAC recovery, a signed guest pairing uses two distinct logical sessions bound to the same
accepted PM mapping/SSH transport identity and a single fresh pairing-session nonce: a typed data
session carries ticket/hello/transcript/anchor frames, while an independently controlling guest TTY
session displays scope and receives the human-entered fingerprint/challenge/literal. Transport
observations are evidence only; P-256 SPKI/anchor/ticket/record joins remain authority. Confirmation
must not pass through the data session, argv, environment, file, pipe, or automation. Absence,
non-TTY, mismatched PM/machine/source/artifact/session, expiry, replay, or ambiguous generation-CAS
preserves state and blocks. A raw `lima-action` is replaced by a fixed control-binary typed
`submit-mapped-lifecycle-v1` request whose carrier/mapping/role/Stage-1 joins are repeated by the
admitted XPC executor before effect. This is source-planning authority only; native proof remains
exclusively `EVIDENCE:R3-MAC-IMP-01`.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07), baseline lines 10056–10069
**Baseline span SHA-256:** `831a3866aecda5bed2fa14fa5b00d0c38df05aa067314bdabb2a3355615f4d89`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current contract correction (2026-08-07)

For this planning packet only, the authoritative planning subject is the exact eight-file list in
`r3-mac-evidence-recovery/TASKS.md` (five current-status/control documents plus `SPEC.md`, `PLAN.md`,
and `TASKS.md`), not any earlier six-file planning-subject shorthand. Its hash is computed only by
`for p in <eight literal paths>; do shasum -a 256 -- "$p"; done | LC_ALL=C sort | shasum -a 256`;
review-control files are excluded and all discovery/closure entries bind the resulting value.

The recovery-era evidence command additionally requires
`--expected-product-project-id <dispatch-bound-project-id>` between source-ref and gated-successor;
a missing/mismatched argument invalidates evidence. The historical Linux validation keeps
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` as its explicitly passed historical binding.

`LimaStageOneAuthorizationV1` is limited to the absent fixed `mac.lima.instance/Create` branch
before PM finalization. It never substitutes for `PublisherBootstrapAuthorizationV1`, never authorizes
forwarding/post-PM roles, and never makes a script bootstrap-capable. The attested host control alone
prints full fingerprint/challenge values on its retained terminal; a separate guest controlling TTY
requires manual values from that terminal and binds only a confirmation commitment to the protected
record. The recovery route supersedes older present-tense attempt-4-to-evidence wording: only a
future reviewed remote-equal R1–R6 receipt may precede a fresh evidence dispatch.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07), baseline lines 10070–10091
**Baseline span SHA-256:** `97294b9aba6317d0c18dfb5ec52894c1922329f0b6aeeee44a0c3d7cb67dfae4`

## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`
**Source provenance:** extracted from [`../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../04-contracts-and-gates.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07), baseline lines 10092–10095
**Baseline span SHA-256:** `53c12c7d745bd67b8b15582f88beecdd4b8a4d6745c00cd2bf355c7744f1dba1`

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

The R3 macOS signer store is exactly the legacy System Keychain opened by public
`SecKeychainOpen("/Library/Keychains/System.keychain")` and identity-checked with
`SecKeychainGetPath`. `kSecUseKeychain` is add destination only. Every read, update, and deletion
uses a one-element `kSecMatchSearchList`; every applicable item operation sets the public
noninteractive UI-fail policy. Private `kSecUseSystemKeychain`, a default or ambient Keychain,
user/file/environment/caller-selected stores, Data Protection Keychain, software-file fallback,
and unsigned fallback are contract violations.

The key is one permanent, sign-capable software P-256 private key with service label
`com.substrate.lifecycle.v1` and exact application tag `<scope-id>:signing-key`. Zero exact-tag
matches may create. Exactly one must validate its tag, label, private class, EC P-256 type and size,
permanence, signing capability, canonical public point/SPKI, and equality with the SPKI recorded in
the signed wrapper. More than one match, a wrong attribute, a missing key under an existing wrapper,
an orphan under a new scope, create/reopen drift, or SPKI mismatch stops without regeneration.
Product signing never exports private bytes, but the control pack explicitly accepts that
sufficiently privileged root may export this software key.

Retirement first requires the protected wrapper to be absent. It then reopens and revalidates the
one exact key, deletes only the validated item reference through documented `kSecMatchItemList`
within the exact one-keychain search list, and re-queries for final absence. Protected-wrapper CAS
acquires the scope's current-anchor durable lock before key/SPKI validation and holds it through
readback; retirement holds the same lock across wrapper absence, key validation, exact deletion,
and final absence. Any lookup, identity, delete, or final-absence uncertainty preserves remaining
state and stops. The fixed code-designated root LaunchDaemon, audit-token/peer admission, canonical P1363
low-S signatures, signed protected wrapper, monotonic CAS, receipt/retry, and preserving-first
joins are unchanged. Secure Enclave/Data Protection Keychain and a session-capable user
LaunchAgent signer are explicitly deferred hardening and are not evidence or successor authority.
Intel/T2 is not an R3 support target.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](../04-contracts-and-gates.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10), baseline lines 10096–10126
**Baseline span SHA-256:** `97c4ef774300541a5b29ea496b6ffe89620a43b8fd89755731d4ae7ff4ffc79a`

## AUX-R3-MAC-EVIDENCE-RETIREMENT-V2 planning contract amendment (2026-08-13; docs-only)

This section is the authoritative R3 planning constraint for prospective macOS evidence retirement and the one preserved precommit-less orphan. It supersedes earlier retirement wording only where that wording permits destructive work before external durability/successor acceptance, conflates a pre-removal receipt with post-removal parity, allows the publisher to strand its own retry authority, or allows prospective authority to reach the old orphan. It authorizes no source, test, configuration, build, install, launchd, Keychain experiment/query/mutation, recovery, cleanup, evidence, mirror refresh, or `MAC-CLOSEOUT` effect.

### Normative authority and chronology

Candidate D is conditionally sound only for a newly precommitted prospective V2 lane. Candidate C is the mandatory serial order: preserve and experimentally characterize the old state, separately authorize exact recovery, prove exact parity, then close the prospective contract before any later implementation authorization.

The prospective and recovery lanes are permanently disjoint in authority, schemas, owner/version values, routes, executables, journals, parsers, signature domains, idempotency keys, and target decoders. No conversion or fallback is permitted in either direction. The old attempt's null/precommit-less state can never be upgraded into V2 authority.

The prospective V2 lane obeys all of the following:

1. **External-durability invariant:** no protected state, evidence record, registration, Keychain record, signer, file, component parent, or retry/handoff state is deleted until the still-authorized publisher has signed the exact canonical pre-removal receipt; the harness has durably persisted, reopened no-follow, byte-compared, and hashed that receipt; the harness has signed and independently made its acknowledgement durable; and surviving protected CAS has verified and bound both hashes.
2. **Successor-acceptance invariant:** a current owner cannot delete any state required for its own signing, retry, journaling, or successor invocation until the next owner independently verifies and durably accepts the complete authority capsule, current CAS head, frozen effect plan, and retry state. Guest removal requires surviving-host acceptance. Host removal capable of stranding the publisher requires finalizer acceptance.
3. **Receipt-chronology invariant:** publisher-signed pre-removal receipts describe only authorization, identities, counters, before/quiesced observations, and frozen plans. They never assert future absence or parity. Post-removal absence is recorded only in separately typed harness parity proofs. A parity proof cannot retroactively authorize an earlier effect.
4. **Contiguous terminal ownership invariant:** after finalizer acceptance, the publisher makes no mutation and the finalizer exclusively owns one frozen host terminal-removal suffix. Harness-owned residual cleanup is a later separate phase and never enters the finalizer API.
5. **No-originating-authority invariant:** the finalizer can verify and execute authority but cannot create, countersign, repair, widen, or substitute it.

### Closed prospective transition order

The guest transition is:

`Prepared -> QuiescePrepared -> Quiesced -> PreRemovalReceiptSigned -> ReceiptExternallyDurable -> AcknowledgementExternallyDurable -> AcknowledgementCASBound -> HandoffHostBound -> Removing -> Removed -> ParityExternallyDurable -> ParityHostBound`.

`Quiesced` stops ordinary service/dispatch while preserving the exact retirement signer, protected journal/CAS, handoff channel, and retry executor. It removes no component. `HandoffHostBound` requires the surviving host to independently verify and durably copy the complete canonical guest capsule, guest-CAS head, target/effect plan, and retry state. Only that surviving host owner may then execute removal. `Removed` is an effect observation, not parity. Host retirement cannot begin until `ParityHostBound`.

An unused reservation follows the same authority order: publisher-signed exact pre-effect proof; external proof durability; independent external acknowledgement durability; surviving protected-CAS binding; then exact pairing-record restoration/removal. Timeout, process death, pathname state, or memory-only acknowledgement is insufficient.

The host transition is:

`Prepared -> QuiescePrepared -> Quiesced -> PreRemovalReceiptSigned -> ReceiptExternallyDurable -> AcknowledgementExternallyDurable -> AcknowledgementCASBound -> FinalizationRequestFrozen -> FinalizerAccepted -> Removing -> EffectsComplete -> HarnessResidualRemoving -> ParityExternallyDurable -> TerminalAcknowledgementBound -> Complete`.

`Quiesced` preserves the publisher's signing, protected CAS/lock/journal, latch, finalizer handoff, and retry capabilities and performs no removal. `FinalizationRequestFrozen` binds one canonical request digest, the current live-CAS precondition, exact authority bytes, identities, targets, and effect order while publisher authority survives. `FinalizerAccepted` exists only after the finalizer has mutually attested its peer, independently verified live CAS and every authority join, claimed the request digest, and durably copied the complete capsule and predecessor head into its surviving root journal. It is the irreversible commit point. No host removal occurs before it.

`Removing` is exclusively finalizer-owned and uses a prepared/invoked/observed substate for each effect. `EffectsComplete` is an immutable finalizer response, not parity. The harness may then remove only its separately precommitted attempt-created artifacts under its own durable journal. It subsequently produces the externally durable parity proof and terminal acknowledgement. `Complete` exists only after the finalizer journal binds those artifacts to the exact response/journal head.

### Receipt and parity artifact wall

Prospective V2 must define separate closed canonical types and signature domains for:

- guest and host publisher-signed pre-removal receipts;
- harness receipt-durability acknowledgements;
- the finalization request, which transports rather than originates authority;
- guest and host harness parity proofs;
- the host terminal acknowledgement;
- finalizer journal generations and immutable `EffectsComplete` response.

Each type requires strict owner/version checking, unknown/duplicate-field rejection, canonical bytes, named hash inputs, exact predecessor joins, and golden vectors. No pre-removal type may contain a claimed after-state. No post-removal type may authorize an already-performed deletion. V1 signed bytes remain unchanged and historical-only; product V1/V2 state keeps the retirement commitment null and cannot enter these routes.

### Fixed external-finalizer boundary

The prospective finalizer is root-owned, pre-baseline external evidence infrastructure and not an installed product component. It is a non-authoritative, dual-authorized executor of exactly one frozen contiguous host suffix. Before G6 contract closure, the accepted contract must bind literal values for its executable, root launchd service/route, local endpoint, immutable configuration if any, root journal, exact parent/file physical and code identities, source/build identity, capability digest, endpoint ownership/mode, intended-principal coordinator identity, framing limit, target roles, and fixed effect implementations. This amendment creates no placeholder executable or ambient route.

The only privilege transition is the precommitted launchd route starting the precommitted executable as root. No sudo, setuid, caller-selected executable, mutable configuration, direct publisher call, product install/update/remove route, recovery route, or network route is allowed. The only caller is the exact precommitted external coordinator as the intended principal.

Before parsing authority-bearing bytes, finalizer and coordinator mutually verify the fixed endpoint and exact audit/peer credentials, effective UID and canonical account, PID plus process-start identity, executable physical identity, code identity/designated requirement, and image digest. They repeat peer verification after complete framing and request EOF and before acceptance. Socket ownership/mode alone is never authority.

The accepted connection is normalized to FD3. It contains one bounded length-prefixed canonical document and then request EOF; the response has the same one-frame-plus-EOF property. The later G6 contract freezes the exact maximum and encoding. Missing/trailing bytes, duplicate/unknown fields, a second frame, absent EOF, wrong owner/version, peer drift, extra argv/stdin, ambient environment, cwd input, or alternate framing stops before acceptance with no effect.

Authority-bearing records are carried inline. No caller-supplied path is followed. The finalizer accepts no arbitrary command, action, path, launchd label, Keychain service/account/tag/predicate, environment-derived target, dynamic/plugin load, product uninstall, publisher upgrade, or recovery input. It recomputes exact target identities and allowed operations solely from closed component roles, the precommitted exhaustive evidence ledger, and its compiled allowlist.

Before `FinalizerAccepted`, the finalizer independently verifies the publisher signature, exact durable harness acknowledgement, receipt/ack file and parent identity claims, live signed protected CAS and generation/counter, CAS-bound request digest, terminal latch and current lock identity, guest parity binding, authorization window at first acceptance, caller/finalizer/code/build/evidence/scope identities, exact target membership, and absence of a prior different request claim. It acquires the exact current-anchor lock before reading the live CAS and holds it through the acceptance generation.

The finalizer durably claims one request/idempotency digest and transfers successor CAS ownership into one fixed root-owned, no-follow, per-scope locked, hash-chained, generation-CAS journal. Each generation and its directory are fsynced and reopened/verified; the journal survives all publisher deletions. After a crash, the exact lock must be reacquired and the journal/live target state revalidated before continuing.

For each effect, `EffectPrepared` is durable before invocation, `EffectInvoked` is durable immediately before the system call, and `EffectObserved` is durable only after exact final-state observation. A restart after `Prepared` may invoke. A restart after `Invoked` must observe before any reinvocation. Mismatch or ambiguity preserves state. Caller disconnect before acceptance has no effect; disconnect after acceptance leaves the finalizer responsible for convergence. A byte-identical committed caller may rejoin the same digest; an alternate digest is rejected. Expiry applies at first acceptance only. Same-digest terminal replay returns the immutable response without inspecting new targets.

The frozen suffix may contain only precommitted host-publisher roles: fixed publisher service/endpoint quiesce or bootout; exact pairing/lifecycle records; protected wrapper/current anchor; the one exact P-256 signer; the exact durable lock if attempt-created and no longer needed; fixed helper/plist/provenance/bootstrap artifacts; created-empty publisher parents; and the latch last. Detailed order is determined by the exhaustive resource DAG and separate `required_until` holdbacks. Harness-owned selected-prefix/build/coupled artifacts are prohibited finalizer targets.

The component DAG and holdbacks may refine but cannot invert these terminal edges: host quiescence and `FinalizerAccepted` precede all host-publisher removal; protected-wrapper/current-anchor final absence precedes signer deletion; the exact current-anchor lock and latch remain reacquirable through signer final-absence observation; helper/plist/provenance removal follows signer absence; the latch is the last publisher target; and the external finalizer, journal, coordinator, and receipt infrastructure survive through terminal acknowledgement and `Complete` as measured pre-baseline evidence infrastructure.

The finalizer has **no private signing key** and cannot sign receipts or originate authority. A finalizer key is prohibited unless a later larger amendment defines its necessity, provisioning/access control, every signature domain, and its own retirement protocol. The finalizer's immutable response is authenticated by the committed journal/CAS chain and then bound by the harness-signed parity proof and terminal acknowledgement.

The protocol accepts only the same-digest finalization request/rejoin and, after `EffectsComplete`, the exact harness terminal acknowledgement for that request. There is no caller-selected action. Before acceptance the only response is a no-effect stop. After acceptance, responses are limited to same-digest in-progress/rejoin or immutable `EffectsComplete` with the request digest and journal head. Only after the journal binds exact parity and terminal acknowledgement may it return immutable `Complete`; effects completion alone is never terminal success.

### Keychain capability and experiment gate

The exact prospective finalizer identity's noninteractive delete-only System-Keychain capability is mandatory evidence before G6. In a separately authorized rollback-safe disposable native-macOS experiment, the exact identity and proposed access-control posture must prove that it can delete the one exact prospective signer and deterministically re-observe absence, while sign, export, ACL/trusted-application mutation, other-key deletion, and target broadening all fail without UI or mutation. Root is not assumed sufficient. SecurityAgent activation/window/prompt, ambiguous status, identity/access-control drift, or residual surrogate fails the gate.

A failed gate pivots only the terminal signer-deletion mechanism. It cannot broaden the finalizer, merge recovery, authorize a prompt, or create product-uninstall/general-recovery power.

The same separately authorized experiment packet may include creator-route arms for the old orphan only when conclusions remain separate. It must compare query-level UI failure with a fresh process whose first Security-framework call successfully disables process interaction before the same query-level UI-fail operation, plus wrong-identity and already-absent retries. Every effectful arm uses a fresh surrogate, precommitted repetition count, exact logs/settle window, and closed rollback in a rollback-capable native-macOS environment. It uses no live orphan or product identity. The experiment grants no live authority.

No password, credential, Touch ID, `Allow`, `Always Allow`, ACL/trusted-application edit, or persistent authorization change is permitted. Any UI, duplicate/mismatch, unaccounted SecurityAgent state, suppression failure before query, rollback failure, or baseline drift is an experiment failure and no conclusion. A later one-prompt live recovery fallback requires separate exact user authorization binding the one old key, executor/code identity, machine/build/session, operation, invocation, expected prompt text and SecurityAgent/code identity, and exactly one prompt. Cancellation, mismatch, a second prompt, `Always Allow`, persistent ACL/trust change, or UI on a no-UI route terminates that authorization.

### Closed stops, review matrix, and authority posture

Before `FinalizerAccepted`, any authority, identity, peer, code, framing, durability, CAS, target, lock, capability, or baseline mismatch is a no-effect preserving stop. After acceptance, the same-request journal is the only rejoin authority. Alternate input, target drift, unexplained state, ambiguous effect, UI, lock loss, journal corruption, or parity failure stops without widened cleanup or substituted authority.

The only stop/rejoin classifications are `SafePreAcceptanceStop`, `RejoinAcceptedRequest`, `IdentityOrAuthorityStop`, `InteractionStop`, `AmbiguousEffectStop`, and `TerminalParityFailure`. They never grant new authority or success.

| Injected edge | Mandatory result |
|---|---|
| Before guest receipt/acknowledgement durability or CAS binding | No guest deletion; same-state retry or preserving stop. |
| After guest CAS binding but before surviving-host acceptance | No guest deletion; signing/handoff remains callable. |
| Before host receipt/acknowledgement durability, CAS binding, or finalizer acceptance | No host deletion; same-state retry or `SafePreAcceptanceStop`. |
| After finalizer acceptance and before effect invocation | Surviving finalizer journal retains ownership and resumes the same digest. |
| After `EffectInvoked` and before `EffectObserved` | Observe before any reinvocation. |
| Caller disconnect after acceptance | No rollback or authority return; finalizer continues or same-digest rejoin occurs. |
| Key deletion/absence UI or ambiguity | `InteractionStop` or `AmbiguousEffectStop`; no approval or widened query. |
| `EffectsComplete` before parity/terminal acknowledgement | Immutable effects response only; never terminal success. |
| Parity/terminal acknowledgement mismatch | `TerminalParityFailure`; no evidence, mirror movement, or closeout. |
| Same-digest retry after `Complete` | Byte-identical terminal response; no new target inspection. |

A later implementation review must inject failure before and after every receipt/acknowledgement write, file fsync, parent fsync, reopen/hash, signature, CAS, peer re-attestation, EOF, finalizer claim, journal generation, lock acquisition/reacquisition, effect prepared/invoked/return/observed boundary, caller disconnect, service bootout/absence, Keychain delete/final absence, unlink/parent-fsync, response, harness residual effect, parity proof, and terminal acknowledgement. It must prove no destructive-before-ack path; no host deletion before finalizer acceptance; observe-before-reinvoke after ambiguous returns; same-digest deterministic replay; peer/path/identity/target substitution rejection; unknown-field and alternate-digest rejection; cross-lane decoder rejection; and every SecurityAgent/UI stop.

This amendment remains planning authority only. Literal V2 schema/domain/route constants, implementation file/symbol fences, experiment execution, live recovery, landing, evidence, mirror refresh, and `MAC-CLOSEOUT` each require their later serial authorities from the phase map.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only`](../04-contracts-and-gates.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only), baseline lines 10127–10234
**Baseline span SHA-256:** `cfdde21d828409a1359659135ee1c30d57ce502e0fe7b0f44316f08cccf50e05`
