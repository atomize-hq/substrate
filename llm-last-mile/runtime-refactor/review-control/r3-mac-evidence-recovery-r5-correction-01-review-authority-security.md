# R5 Correction-01 discovery — authority/security

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01`
Subject: `sha256:fed39d93a3fd411bf186373e17d4f3a00c3d7a231814f815fac060492c737cf1`

Fresh read-only discovery found the following blocking issues before remediation:

- `P1-R5-C01-INSTALL-PROVENANCE-TOCTOU`: mutable prefix control/executor paths could be reopened by the privileged installer before provenance measurement.
- `P1-R5-C01-POST-PM-ARTIFACT-COPY-TOCTOU`: a descriptor-measured artifact path was later handed back to `limactl copy` by pathname.
- `P1-R5-C01-POST-PM-AFTER-OBSERVATION`: receipt completion accepted existence/raw command output rather than exact bytes/owner/mode or semantic post-state.
- `P1-R5-C01-ADMISSION-TRANSITION-RECOVERY`: a crash after protected-state CAS and before global admission CAS made the pre-decode admission gate reject the only recovery route.
- `P2-R5-C01-LIMA-EFFECT-TIMEOUT`: the five-second FD3 framing limit incorrectly applied to fixed Lima lifecycle effects.
- `P2-R5-C01-STAGE1-MAPPING-RETURN`: a verified Stage-1 successor mapping was derived but omitted from the durable response.

The review performed no edits and no native/XPC/Lima/Keychain action. Remediation must retain the packet's fixed transport, typed request map, and MAC-only fence.

## Closure-1 (fresh independent review)

Subject: `sha256:eb3eeafd1f74382067ece8f04cd0f1df93f03c8bd2ab921c42f570f68f3ecfd6`

The fresh closure reviewer reported `P1-R5-C01-GENERIC-TARGET-AFTER-STATE`: generic `test -e` probes could complete directory/private-home and fixed publisher targets without exact type/owner/group/mode validation. Remediate only with fixed executor-owned target-state checks.

## Supplemental-causal-1 (fresh independent review)

Subject: `sha256:bf8607b7b9ee027be8badda0fb9deaa04918fbb026bfcbd4b26f6ecc061c8a22`

The fresh reviewer confirmed that the closure P1 target-state remediation is clean: filesystem
completion now requires exact type/owner/group/mode and, for byte projections, digest. It found
no new authority/security P1.

## Supplemental-causal-2 (fresh independent review)

Subject: `sha256:a7a1d181d2baa53cb0a914be3a999681e39588d56dce627c963ce686f61d94db`

`CLEAN`. The executor emits the ordinary canonical post-PM request set only after the durable
Stage-1 successor state, anchor, manifest, admission, and capsule transition. The issuer derives
mapping, anchor, manifest, principal, attempt, and executor identity from that retained successor
state, not from the Stage-1 authorization. The wrapper consumes exactly one attested returned
request and cannot synthesize any authority field. No new authority carrier, selector, endpoint,
or transport was found.
