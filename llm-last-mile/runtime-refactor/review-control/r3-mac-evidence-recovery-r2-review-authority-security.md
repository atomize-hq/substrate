# Authority/security discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R2

## Review boundary

Fresh read-only authority/security review by a `gpt-5.6-terra` Extra High reviewer of the
ten-path R2 implementation subject bound to
`sha256:03bb9c919cd142e1b65079eddd188602410e1d36a3fce740df2979ec41d89b74`.
The fingerprint is the SHA-256 of the lexically sorted per-file `shasum -a 256` manifest for
the exact nine tracked R2 paths and
`tests/mac/dev_install_compile_surface_r3.sh`. The reviewer did not mutate, test, install,
stage, or publish.

## Finding adjudication

| Reported item | Reviewer priority | R2 disposition | Evidence |
|---|---|---|---|
| The exact macOS `BUILD_FLAGS` pair reaches the currently non-compiling `substrate-lifecycle-macos` binary before the fixed-copy loop. | P1 | deferred R4 gate; not an R2 blocking finding | The R2 compile-surface test records the existing `sha2`, `AuditTokenV1`, and pointer-cast failures without compiling the lifecycle-macos binary. Amendment 0016 expressly defines R2 proof as the named shell/ordinary-host compile surface plus installer source shape, and preserves the lifecycle binary compile closure as a mandatory R4 gate. The reviewer found no caller-controlled authority, secret-bearing diff, or additional authority/security P1–P4 issue. |

The deferred gate is not a claim that a real installer action succeeded: no installation or
native action was run. The fixed-copy source shape remains confined to the two named binaries,
the managed manifest, and the MAC branch.
