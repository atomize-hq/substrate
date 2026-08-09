# R3 macOS FD3 stream correction — authority and security review

- Packet: `AUX-R3-MAC-FD3-STREAM-CORRECTION`
- Base: `cc2bf70ecf337f6af4b7369da0d79ff62c2c6a86` /
  `de3c995e4d35a9cba547181323a5335e10a46d69`
- Discovery subject: `sha256:97b7d34d94437dcbf27db0b95647b9cdb1e2ae5f1522d51af79272fe7805fba5`
- Closure subject: `sha256:f92282462b5ab05935307a156387265ea9a5f4a9acb169630ae8500bbc0f24f3`
- Reviewers: fresh independent read-only `gpt-5.6-terra` Extra High discovery and
  closure reviews

## Mandatory predecessor P1 corrections

This packet closes both independently supplied P1 defects without treating them as new discovery
findings. First, the executor re-arms `FD_CLOEXEC` immediately after confirming descriptor 3 and
before socket type, peer, terminal, image, codesign, provenance, or JSON work. Second, the direct
FD3 installer parser accepts only the canonical direct-response field set and
`bootstrap_channel_bound: true`; it no longer requires or fabricates XPC attestation. Actual XPC
responses still require `xpc_attestation` and `audit_token_bound`.

The fixed `/usr/bin/sudo -C 4 --` and privileged-helper paths, scrubbed environment, immediate
RAII ownership, intended FD3-only inheritance, kernel `getpeereid`/`LOCAL_PEERPID` admission,
controlling-terminal join, peer and running-image identity, CDHash/code requirement, and retained
provenance checks all remain before authority use. Every incomplete client exchange retains a guard
that kills and reaps the exact child, and no ambiguous partial exchange is automatically resent.

## Discovery and remediation

The authority/security discovery lens was `CLEAN`. The other lenses reported the consolidated
`FD3-REV-P2-001`: the first live test modeled framing and kill/reap in Python instead of executing
the actual Rust helper and guard bodies. Remediation now extracts the exact bound client helper,
canonical-parser, and `RetainedBootstrapChildGuardV1` source bytes into a nonpersistent offline
Rust harness. The macOS executor binary tests execute its real stream, CLOEXEC, EOF, and bound
helpers. No privileged path is entered.

## Closure

The fresh closure reviewer independently recomputed the closure fingerprint and returned
**CLEAN**, with zero P1/P2 findings. It confirmed the two predecessor P1 corrections, fixed
authority path, predecode admission order, bounded canonical EOF protocol, absolute deadlines,
and exact child cleanup. The remediated deterministic proof digest is
`f9e2fad3eae9d72fcbb341bb25fc9a41cd6bc52f8d3e14e33212303e89fda296`.
No native evidence was run by either reviewer.
