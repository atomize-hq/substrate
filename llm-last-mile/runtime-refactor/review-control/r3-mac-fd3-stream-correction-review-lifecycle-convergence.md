# R3 macOS FD3 stream correction — lifecycle and convergence review

- Packet: `AUX-R3-MAC-FD3-STREAM-CORRECTION`
- Discovery subject: `sha256:97b7d34d94437dcbf27db0b95647b9cdb1e2ae5f1522d51af79272fe7805fba5`
- Closure subject: `sha256:f92282462b5ab05935307a156387265ea9a5f4a9acb169630ae8500bbc0f24f3`
- Reviewers: fresh independent read-only `gpt-5.6-terra` Extra High discovery and
  closure reviews

## Discovery findings

- `FD3-REV-P2-001`: although the first Darwin regression covered the required behaviors, its
  framing implementation was a Python shadow and its kill/reap case did not execute the actual
  Rust client guard. The proof therefore did not close causality for the implementation helpers.
- `FD3-REV-P3-002`: the publisher-bootstrap branch comment in
  `src/bin/substrate-lifecycle-control.rs` still described every direct channel as one seqpacket
  frame.

The duplicate allowlist report of the proof gap is consolidated into `FD3-REV-P2-001`. The Rust
regression now compiles the exact product helper bodies and exercises fragmentation, coalescing,
canonical rejection, empty and missing EOF, exact 1 MiB and plus one, absolute timeout, and the
actual guarded-child kill/reap path. Executor unit tests exercise its actual `SOCK_STREAM`,
`FD_CLOEXEC`, `SO_NOSIGPIPE`, EOF, and bound helpers. The terminology now distinguishes the Linux
seqpacket frame from the Darwin EOF-delimited stream document.

## Closure

The fresh closure reviewer independently recomputed the nine-path fingerprint and returned
**CLEAN**, with zero unresolved P1/P2 findings. It confirmed partial-write and EINTR-safe loops,
one absolute deadline per exchange phase, the one-byte EOF probe, bidirectional `SHUT_WR`, no
automatic resend, exact-child cleanup, and intended-exec-only FD3 inheritance. Linux and guest
publisher seqpacket endpoints remain unchanged. No native evidence was run.
