# R3 MAC limactl HOME provenance correction — allowlist and evidence review

- Packet: `AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION`
- Discovery reviewer: fresh independent read-only `gpt-5.6-terra` Extra High
- Discovery subject: `sha256:0b1c7f989cfd3dbb7b1864262413c6df01df10bf7affbb60c716ce08aa27bcc8`

## Discovery result

**CLEAN — no P1/P2/P3/P4 finding.**

The product/test candidate is limited to the authorized installer and new focused macOS fixture.
The installer hunk is limited to the privileged provenance version environment and its required
`/var/empty` validation; the fixed no-follow image, fixed PATH, scrubbed environment, and
provenance joins remain present. No Windows, WSL, ordinary Linux, macOS host runtime/lifecycle
body, Cargo manifest, generated context, secret, or protected-checkout path changed.

The fixture reads the exact base source for RED and uses only a temporary fake executable for
GREEN. It verifies the Lima 2.1.1-style missing-HOME panic, fixed PATH, explicit
`HOME=/var/empty`, and absence of caller-HOME and ambient-sentinel leakage. It does not invoke
the real installer or mutate `/Library`, Keychain, launchd, Lima, sudoers, known_hosts, or
official evidence.

## Closure

The different fresh closure reviewer independently recomputed the declared manifest fingerprint
`sha256:e74ede1d8c3a795591c5c3b3f7567441c8add1e4dfa035979e5dbd0e31946a7b`
and returned **CLEAN**: P1/P2/P3/P4 are zero. It found no allowlist expansion, secret, or
external-build/install boundary violation.
