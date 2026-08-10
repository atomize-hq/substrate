# R3 macOS System-Keychain software signer correction — authority and security review

- Packet: `AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION`
- Base: `c6641a03d71979c76b75fbb88f8276231417d35f` / `312f0284a111f6bfc693199a4f12b74f463989cf`
- Discovery subject: `sha256:41befc0353221867731b20e078ce87d202f670752fbefc97d359e44ee2372625`
- Closure subject: `sha256:66bfcff9a7360a574ef96e05871ef9ed4827ce7ff936dfc280a3f565e5f3fe4f`
- Review mode: fresh independent read-only discovery and different fresh read-only closure

## Discovery finding

`R3-MAC-SKC-P2-001` (`AS-P2-01`, P2): signer retirement checked wrapper absence and
then deleted by a rebuilt tag-wide query without holding the protected-state CAS guard. A wrapper
CAS could therefore validate key A, cross retirement, and persist a wrapper whose key had been
deleted. A raced same-tag item could also be selected by the second query.

The remediation acquires the exact `<scope-id>:current-anchor` durable guard before key/SPKI
validation in protected-state CAS and holds it through the write. Retirement takes the same guard
across wrapper absence, validated key lookup, deletion, and final absence. Deletion now uses the
documented `kSecMatchItemList` selector with the already validated reference, exact tag, exact
one-keychain search list, and UI-fail policy. The causal regression fixes both lock order and item
selector.

The authority lens found no other P1/P2: the implementation uses only public explicit-System-
Keychain routing, admits no authorization UI or alternate store/fallback, preserves SPKI,
canonical-signature, wrapper/CAS, audit-token and designated-requirement joins, and states the
privileged-root export threat truthfully.

## Closure bounded stop

The different closure reviewer verified the runtime remediation but found
`R3-MAC-SKC-P2-002` (P2): active recovery specification
`llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md` lines 31-32 and 131-136 still
mandate a non-exportable macOS signer. That file is byte-identical to the bound base and outside
this packet's path fence. The smallest required expansion is that one specification path plus its
stale-claim regression. Per the bound authority, closure is `BLOCKED_SCOPE_EXPANSION`, not CLEAN.

## Continuation-01 supplemental causal review and closure

Continuation nonce
`cbbd50a3d151df0ef71699c2ae412998cf34f53a2ec6913ac1394079e4732589` authorized exactly
the active recovery specification and its stale-claim regression. The specification now states the
fixed root LaunchDaemon, software P-256 signer in the explicitly opened legacy System Keychain,
privileged-root export capability, preserved admission/SPKI/signature/wrapper/CAS/preserving-first
controls, no alternate store or unsigned fallback, Apple Silicon-only support, and deferred Secure
Enclave, Data Protection Keychain, and user-LaunchAgent hardening.

A fresh independent supplemental causal reviewer and a different fresh closure reviewer each
recomputed subject `sha256:a52b409b7cb151353c167709d49e16a528d61acf1213b0e3586646451dbaad42`
and returned CLEAN with zero P1/P2 findings. No authority, fallback, product-evidence, closeout, or
successor action was added.
