# R3 macOS System-Keychain software signer correction — lifecycle and convergence review

- Packet: `AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION`
- Discovery subject: `sha256:41befc0353221867731b20e078ce87d202f670752fbefc97d359e44ee2372625`
- Closure subject: `sha256:66bfcff9a7360a574ef96e05871ef9ed4827ce7ff936dfc280a3f565e5f3fe4f`
- Review mode: fresh independent read-only lifecycle discovery and different fresh closure

## Discovery finding and remediation

`R3-MAC-SKC-P2-001` (`LC-P2-001`, P2) identified the same causal race: wrapper CAS validated the
key before acquiring the current-anchor guard, while retirement performed its wrapper absence read
and key deletion without that guard. The possible terminal state was a surviving wrapper bound to
a missing key, making retries permanently preserving-blocked.

The remediated ordering is one scope-bound critical section. Wrapper CAS acquires the exact account
guard before key/SPKI validation and retains it through observed-state comparison and the write.
Retirement holds the same guard across wrapper absence, exact key validation, exact-item deletion,
and final absence. Sequential zero/one/duplicate behavior, create/reopen SPKI equality, canonical
signature joins, wrapper comparison, and retry behavior remain unchanged. The focused source-order
regression and all locked/offline lifecycle tests pass.

## Closure bounded stop

The fresh closure reviewer found no remaining runtime/convergence P1/P2, but closure cannot be
CLEAN while the active recovery specification still requires non-exportable host signing. The
contradiction is `R3-MAC-SKC-P2-002` at
`llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md` lines 31-32 and 131-136. Because
that path is outside the authorized fence, the correct terminal state is
`BLOCKED_SCOPE_EXPANSION`; no commit or publication is authorized.

## Continuation-01 supplemental causal review and closure

The Rust source remains
`sha256:960a15cfe8992d89dcee4aeb4c7317cd15aeea83e8895b68ae03aa8aa2f3ded2`, while the
complete seven-path implementation subject is
`sha256:a52b409b7cb151353c167709d49e16a528d61acf1213b0e3586646451dbaad42`.
The authorized specification and regression remediation resolves `R3-MAC-SKC-P2-002` without
changing runtime bytes or lifecycle ordering. A fresh independent supplemental causal reviewer and
a different fresh closure reviewer both returned CLEAN with zero P1/P2 or lower-priority findings.
