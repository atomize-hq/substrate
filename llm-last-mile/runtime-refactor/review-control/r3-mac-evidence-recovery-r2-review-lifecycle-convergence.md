# Lifecycle/convergence discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R2

## Review boundary

Fresh read-only lifecycle/convergence review by a `gpt-5.6-terra` Extra High reviewer of the
ten-path R2 implementation subject bound to
`sha256:03bb9c919cd142e1b65079eddd188602410e1d36a3fce740df2979ec41d89b74`.
The reviewer did not mutate, test, install, stage, or publish.

## Valid blocking finding

| ID | Priority | Disposition | Evidence |
|---|---|---|---|
| `P2-R2-LIFECYCLE-001` | P2 | unresolved; bounded stop | `stage_managed_mac_control_binary_copy` removes an already-managed destination before the replacement is durable, publishes the copied binary before the manifest entry, and leaves no rollback/recovery path if the manifest update fails. A retry can reject the now-unmanifested file as unmanaged. The sequential two-binary loop can also retain only one copied member after a later failure. The existing regression fixture proves the happy path but not copy/manifest failure and retry convergence. |

The finding is inside the original lifecycle-review concern, but Amendment 0016 authorizes only
the three `linux_client.rs` cfg substitutions and prohibits every other product or test edit.
The required helper/fixture remediation therefore cannot be made under this continuation.
Linux selected socket behavior remains equivalent: its branch keeps
`SOCK_SEQPACKET | SOCK_CLOEXEC`; the manual close-on-exec fallback is MAC-only.

## Closure review after P2-R2-LIFECYCLE-001 remediation

Fresh read-only closure review by a different `gpt-5.6-terra` Extra High reviewer of the
remediated ten-path R2 subject bound to
`sha256:3277175c543c1769c6121e70bab52c2f94c35cda04ac3a0031d758a7b6467095`.
The reviewer did not mutate, test, install, stage, or publish. The new MAC static proof remained
strict: it verifies separate binary/manifest temporaries, binary-temp copy/chmod, manifest-temp
entry construction, cleanup of both temporaries, and manifest publication before destination
replacement.

| ID | Priority | Disposition | Evidence |
|---|---|---|---|
| `P2-R2-LIFECYCLE-002` | P2 | unresolved; bounded stop | `stage_managed_mac_control_binary_copy` uses `grep -Fxv -- "${dest}" "${manifest_path}" > "${manifest_tmp}" || true`. This conflates `grep` status 1 (no retained entries) with a read/I/O/permission failure. A manifest read failure can publish a replacement containing only the current binary. If the other fixed pair member still exists, its later pass rejects that regular file as unmanaged, so retries remain blocked. The new fixture covers copy and manifest-move failures but does not inject the manifest-read failure. |

The issue is directly unmasked by the P2-R2-LIFECYCLE-001 manifest transaction remediation. Its
correct repair must distinguish `grep` status 1 from a real read failure, clean both temporary
paths, and refuse publication on error. Continuation 0018 authorizes no production or additional
test edit, so the issue cannot be remediated in this task state.

## Supplemental causal review after P2-R2-LIFECYCLE-002 remediation

Fresh independent read-only supplemental-causal review by a `gpt-5.6-terra` Extra High reviewer
of the corrected ten-path R2 subject bound to
`sha256:58eb0087e72627a045912f72adfd4e386469a40e093db0ecde00d0e949ef7668`.
The reviewer did not mutate, test, install, stage, or publish.

The causal evidence is direct: P2-R2-LIFECYCLE-001 introduced the per-binary
`manifest_tmp` transaction path, and closure-1 found P2-R2-LIFECYCLE-002 in that path's
`grep -Fxv` status suppression. The correction captures the conditional `grep` status, permits
only status 0 or normal no-match status 1, and on a status greater than 1 removes both temporary
paths before returning without publishing either the manifest or destination. The existing Bash
fixture injects exit 83 only for the exact existing MAC manifest filtering invocation, proves the
pre-existing two-binary pair and manifest remain exact and temporary-free after failure, then
proves retry convergence.

**CLEAN.** No P1/P2 finding directly caused or unmasked by this remediation was reported. The
fixed pair remains gated by the positive `IS_MAC` branch; Linux keeps
`SOCK_SEQPACKET | SOCK_CLOEXEC`, and out-of-corridor Windows and R4 lifecycle-binary debt were
not treated as findings.
