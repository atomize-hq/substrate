# R5 Correction-01 discovery — allowlist/evidence

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5-CORRECTION-01`
Subject: `sha256:fed39d93a3fd411bf186373e17d4f3a00c3d7a231814f815fac060492c737cf1`

Fresh read-only discovery confirmed the listed R5 correction paths are the required bounded closure. It independently reported:

- `P1-R5-C01-POST-PM-ARTIFACT-COPY-TOCTOU` because measurement and copy did not share retained bytes.
- `P1-R5-C01-POST-PM-AFTER-OBSERVATION` because `test -e` could complete an old/wrong artifact target.
- `P1-R5-C01-POST-PM-OBSERVATION-AMBIGUITY` because unexpected nonzero probe failures could replay a possibly completed effect.

The required remediation is confined to the existing executor, installer, MAC fixture, and existing tests. No new role/action, selector, endpoint, transport, forwarding implementation, guest pairing/session surface, Linux/Windows target, or unlisted product path is authorized. No P3/P4 findings were reported.

## Closure-1 (fresh independent review)

Subject: `sha256:eb3eeafd1f74382067ece8f04cd0f1df93f03c8bd2ab921c42f570f68f3ecfd6`

The independent closure found no path-fence expansion. The two valid findings remain in the existing executor/warm/test closure and require no forwarding, pairing, generic selector, endpoint, or transport change.

## Supplemental-causal-1 (fresh independent review)

Subject: `sha256:bf8607b7b9ee027be8badda0fb9deaa04918fbb026bfcbd4b26f6ecc061c8a22`

The first-run post-PM request binding defect is confined to the existing executor/warm/test
closure. It requires no role/action expansion, forwarding, pairing/session surface, endpoint,
or transport change.

## Supplemental-causal-2 (fresh independent review)

Subject: `sha256:a7a1d181d2baa53cb0a914be3a999681e39588d56dce627c963ce686f61d94db`

`CLEAN`. The final remediation remains within the existing macOS executor, mapped wrapper, warm
caller, and MAC fixture paths. It adds no role/action pair, tag, generic selector, endpoint,
transport, forwarding path, pairing/session surface, or non-MAC target. The static fixture proves
the bounded opaque response selection and rejection cases.
