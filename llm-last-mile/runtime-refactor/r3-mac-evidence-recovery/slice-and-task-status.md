**Kind:** slice/task/status projection
**Stable ID:** `r3-mac-evidence-recovery-family`
**Canonical for:** MAC recovery slice/task status bundle from 03
**Status:** canonical
**Authority scope:** exact extracted 03 recovery-status source body only
**Source span:** [`../03-phase-slice-map.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../03-phase-slice-map.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06) lines 3143–3154; [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07) lines 3155–3165; [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07) lines 3166–3180; [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07) lines 3181–3184; [`../03-phase-slice-map.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](../03-phase-slice-map.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10) lines 3185–3215; [`../03-phase-slice-map.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only`](../03-phase-slice-map.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only) lines 3216–3237
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# R3 macOS evidence-recovery slice/task/status projection

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
**Source provenance:** extracted from [`../03-phase-slice-map.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](../03-phase-slice-map.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06), baseline lines 3143–3154
**Baseline span SHA-256:** `60fee5816a3b3372f02f14dfe172ac390e0a6eb807d4a07628645ed8c980e3b2`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN packet status (2026-08-07)

Before any renewed MAC implementation, the published recovery plan requires independently landable
R1 Bash descriptor, R2 macOS compile, R3 trusted evidence-project-ID, R4 protected MAC
Keychain/P-256/Stage-1, R5 typed mapped-submit, and R6 dual PM-bound data/TTY session packets.
They are exact-hunk recreation packets, not a donor merge. R6 is the first possible predecessor of
a separately authorized `EVIDENCE:R3-MAC-IMP-01`; each packet has its own predecessor, review,
remote publication gate, and explicit authority stop. `lima-stdio-v1`, raw `lima-action`, a direct
helper relay, an unbound channel, a Windows behavior change, and an ordinary Linux host pairing
route are excluded.
**Source provenance:** extracted from [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07), baseline lines 3155–3165
**Baseline span SHA-256:** `d465db00bd0505c16cf93b313e3209b112bd9bdb1b80ae45cd676deb030814a9`

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current evidence-command and successor rule (2026-08-07)

For every recovery-era provider/final evidence validation, the authoritative invocation is
`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid>
--expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id
<dispatch-bound-project-id> --expected-gated-successor <value>`. The expected project ID is supplied
by the fresh evidence dispatch and is never inferred from the artifact. The historical Linux
artifact uses its declared historical ID `2ccb802f-301c-4af4-9bd5-51d22808f0a2`.

This rule supersedes only the older present-tense R3 MAC succession for the bound recovery route:
historical attempt-4 and the 8,821-line donor cannot authorize evidence. A separately authorized
`EVIDENCE:R3-MAC-IMP-01` may begin only after the future R1–R6 recovery implementation has one
reviewed remote-equal receipt; no plan, donor, or partial R1–R5 receipt is an evidence predecessor.
**Source provenance:** extracted from [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07), baseline lines 3166–3180
**Baseline span SHA-256:** `e2423ef4a1da20856138d832f71c185e88c0b6da600424bd702e9bec8b745134`

## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`
**Source provenance:** extracted from [`../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](../03-phase-slice-map.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07), baseline lines 3181–3184
**Baseline span SHA-256:** `53c12c7d745bd67b8b15582f88beecdd4b8a4d6745c00cd2bf355c7744f1dba1`

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

- **Bound predecessor:** the remote-equal R3 recovery checkpoint plus reviewed root-LaunchDaemon
  probes showing `errSecNotAvailable` for default/Data-Protection Secure Enclave lookup,
  successful explicit legacy System-Keychain operations, software P-256 persistence/signing with
  private export capability, and rejection of Secure Enclave routing to the explicit legacy
  System Keychain.
- **Owned correction:** replace private `kSecUseSystemKeychain` and non-exportability assumptions
  with one public-API software P-256 signer in the explicitly opened and path-verified
  `/Library/Keychains/System.keychain`. Adds use `kSecUseKeychain`; searches and exact deletion use
  a one-element `kSecMatchSearchList`; every applicable item operation fails rather than allows
  authorization UI. Exact tag/service/type/size/permanence/signing/SPKI joins, duplicate and
  mismatch preserving stops, signed wrapper/anchor, monotonic CAS, canonical P1363 low-S
  signatures, receipts, and byte-identical retry remain mandatory.
- **Threat posture:** sufficiently privileged root with System-Keychain access may export the
  software private key. This matches the accepted Linux root-protected-key posture and must never
  be described as Secure-Enclave-backed, hardware-backed, or non-exportable.
- **Retirement:** a surviving protected wrapper blocks key deletion; otherwise only the exact tag
  in the exact opened System Keychain may be deleted through the validated item reference, followed
  by an exact final-absence check. Retirement and wrapper CAS share the exact current-anchor
  durable lock from pre-key validation through terminal readback, so wrapper/key ownership cannot
  cross between their checks. Ambiguity or mismatch preserves every remaining record.
- **Excluded:** Intel/T2, user/default/ambient/file/environment/caller-selected stores,
  software-file fallback, unsigned mode, product installation, Lima, pairing,
  `EVIDENCE:R3-MAC-IMP-01`, and MAC closeout. Secure Enclave/Data Protection Keychain and a user
  LaunchAgent signer are deferred hardening, not implied successor authority.
- **Proof/publication:** focused static and Rust regressions, locked/offline Apple-Silicon macOS
  build/tests, exact inventory/allowlist/secret/containment gates, fresh causal-cascading review,
  and one normal fast-forward commit only. A clean correction returns
  `next_increment=EVIDENCE:R3-MAC-IMP-01` with `successor_dispatched=false`.
**Source provenance:** extracted from [`../03-phase-slice-map.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](../03-phase-slice-map.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10), baseline lines 3185–3215
**Baseline span SHA-256:** `bcfead2af97850534844c4820746a793683a13b76046e72d81f07acc734ebfc5`

## R3 macOS retirement/recovery serial-gate amendment (2026-08-13; docs-only)

This amendment controls the next R3 macOS retirement/recovery sequence over any earlier wording that would move directly from the preserved failed attempt to prospective implementation or evidence. It records no implementation, experiment, recovery, cleanup, evidence, mirror refresh, or `MAC-CLOSEOUT` authority.

- **Prospective direction:** Candidate D remains conditionally sound as the structure to close: a pre-baseline external evidence finalizer may execute one frozen terminal host-removal suffix only after external receipt/acknowledgement durability, protected-CAS binding, and durable successor acceptance. Its exact noninteractive delete-only signer capability is a mandatory experiment gate.
- **Operational order:** Candidate C is mandatory. Exact preserved-orphan recovery and baseline parity precede final prospective contract closure and any prospective implementation.
- **Separation:** the prospective V2 protocol and the precommit-less orphan lane share no authority, schema owner/version, route, executable, journal, parser, signature domain, idempotency key, or target decoder. Neither is product uninstall or generalized recovery.

The serial gates are:

1. **`R3-MAC-RETIRE-G0-FREEZE` — historical stop for the archived protected lane.** Preserve the signer, failed-attempt roots/logs, prefix, publisher/Lima state, and mirror pin. The observed SecurityAgent contradiction remains unresolved. No live Keychain query or mutation is allowed. This stop remains evidence/quarantine posture, not the active developer-parity schedule.
2. **`R3-MAC-RETIRE-G1-PLANNING-ACCEPTANCE`.** Explicitly accept the corrected external-durability, successor-acceptance, receipt-chronology, guest/host state machines, finalizer boundary, experiment plan, destructive-edge matrix, and closed stop states. This docs-only amendment does not self-accept or authorize effects.
3. **`R3-MAC-RETIRE-G2-EXPERIMENT-AUTHORIZATION`.** A separate task must bind a rollback-safe disposable native-macOS environment, fresh surrogate identities, exact arms/repetitions, process/query UI controls, SecurityAgent observations, rollback, and receipt locations. It cannot touch the orphan or product state.
4. **`R3-MAC-RETIRE-G3-EXPERIMENT-CLOSURE`.** Independent receipts prove exact rollback/baseline restoration and record two separate conclusions: creator-route no-UI behavior for recovery, and exact prospective-finalizer delete-only capability. The experiment grants no live authority.
5. **`R3-MAC-RETIRE-G4-EXACT-RECOVERY-AUTHORIZATION`.** After G3, a new incident-specific authority binds exactly one old-attempt route, immutable target identity, executor/code identity, journal, UI posture, retry states, stop policy, and review wall. A one-prompt fallback requires an additional explicit exact authorization; it is never inferred.
6. **`R3-MAC-RETIRE-G5-ORPHAN-PARITY`.** The separately authorized recovery, if any, produces a durable receipt and independently verified exact target-scope parity proof. Mirror movement is later and separately authorized; it cannot backfill parity.
7. **`R3-MAC-RETIRE-G6-PROSPECTIVE-CONTRACT-CLOSURE`.** Only after G5, freeze the literal V2 owner/version values, canonical fields/signature domains, external-finalizer caller/route/endpoint/identities, capability evidence, exact component ledger/holdbacks, journal/CAS transfer, framing constants, and fault matrix in authoritative docs and golden vectors.
8. **`R3-MAC-RETIRE-G7-PROSPECTIVE-IMPLEMENTATION-AUTHORIZATION`.** A later top-level task must name exact source/test path and symbol fences, review checks, landing authority, and stop conditions. It cannot implement recovery or authorize evidence.
9. **`R3-MAC-RETIRE-G8-NATIVE-EVIDENCE`.** Only a later explicit operator authority may run one fresh evidence attempt. `MAC-CLOSEOUT` remains a distinct later gate.

No gate dispatches, accepts, or authorizes its successor. A stop, blocked recovery, failed capability gate, or parity failure preserves the current state and does not permit schedule compression, cross-lane substitution, broader cleanup, or retrospective authority.
**Source provenance:** extracted from [`../03-phase-slice-map.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only`](../03-phase-slice-map.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only), baseline lines 3216–3237
**Baseline span SHA-256:** `b8e848cd857f37e4a4863af57fac8cd1ac83164192f6912c8997913a9e18ee40`
