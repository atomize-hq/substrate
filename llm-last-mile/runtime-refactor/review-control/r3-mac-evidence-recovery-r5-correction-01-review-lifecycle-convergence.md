# R5 Correction-01 discovery — lifecycle/convergence

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01`
Subject: `sha256:fed39d93a3fd411bf186373e17d4f3a00c3d7a231814f815fac060492c737cf1`

Fresh read-only discovery found these blocking lifecycle defects before remediation:

- `P1-R5-C01-POST-PM-RECEIPT-RETRY-SIGNATURE`: an existing signed receipt retained a public key while retry comparison cleared only its signature bytes, stranding receipt-to-index/head recovery.
- `P1-R5-C01-POST-PM-OBSERVATION-AMBIGUITY`: every nonzero Lima probe was collapsed to a generic error and could be classified as a safe before/after state.
- `P1-R5-C01-KNOWN-HOSTS-REMOVE-STRANDING`: an admitted known-hosts Remove persisted EffectStarted, then had neither a descriptor-preserving completion nor retry observation.
- `P2-R5-C01-COMPLETION-JOURNAL-CONVERGENCE`: a crash after state/admission CAS but before the immutable Completed journal marker left exact completed retry without journal convergence.

No P3/P4 findings were reported. This review is discovery evidence only; it made no native actions or repository changes.

## Closure-1 (fresh independent review)

Subject: `sha256:eb3eeafd1f74382067ece8f04cd0f1df93f03c8bd2ab921c42f570f68f3ecfd6`

The fresh closure reviewer reported `P2-R5-C01-STAGE1-MAPPING-ADOPTION`: the executor returned the canonical Stage-1 mapping but normal `lima-warm.sh` did not capture/adopt it before constructing the next typed post-PM request. Remediation must use the returned fixed response only; no new selector/carrier may be introduced.

## Supplemental-causal-1 (fresh independent review)

Subject: `sha256:bf8607b7b9ee027be8badda0fb9deaa04918fbb026bfcbd4b26f6ecc061c8a22`

The reviewer reported `P2-R5-C01-FIRST-RUN-POST-PM-REQUEST-BINDING`: the normal warm flow
adopts the executor-returned final mapping but retains its pre-Stage-1 caller request. That
request cannot bind its mapping commitment/current anchor/manifest to an as-yet unobserved guest
machine, so the immediate post-PM branch rejects. The bounded remediation must make the trusted
Stage-1 completion emit or bind an existing canonical post-PM request and make the shell consume
that opaque result without a new tag, selector, carrier, endpoint, or transport.

## Supplemental-causal-2 (fresh independent review)

Subject: `sha256:a7a1d181d2baa53cb0a914be3a999681e39588d56dce627c963ce686f61d94db`

`CLEAN`. The canonical successor request set is emitted only after Stage-1 has durably advanced
to the successor manifest and anchor. The warm wrapper validates the returned mapping/manifest
joins, selects exactly one matching opaque request, and replaces the stale pre-Stage-1 request.
The fixture executes canonical first-run adoption and rejects both a response tamper and duplicate
candidate selection. No P1/P2 convergence defect was found.
