# R3 macOS FD3 stream correction — allowlist and evidence review

- Packet: `AUX-R3-MAC-FD3-STREAM-CORRECTION`
- Discovery subject: `sha256:97b7d34d94437dcbf27db0b95647b9cdb1e2ae5f1522d51af79272fe7805fba5`
- Closure subject: `sha256:f92282462b5ab05935307a156387265ea9a5f4a9acb169630ae8500bbc0f24f3`
- Canonical fingerprint form: SHA-256 of lexically sorted, LF-terminated
  `<content SHA-256><two spaces><repository-relative path>` records; review metadata is excluded
  from the implementation subject.

## Discovery findings

- `FD3-REV-P2-001`: the discovery allowlist lens independently reported the same actual-Rust
  coverage gap as the lifecycle lens. It is one consolidated blocking finding.
- `FD3-REV-P3-003`: the supplemental broad
  `cargo test -p shell --lib managed_lifecycle` failure had not yet been bound to an exact base
  comparison or explicit waiver.

The exact command was subsequently run at the bound base and returned 101. Candidate and base
each have the same 54 count-encoded error headings, including the compiler summary; the complete
multisets are identical. The errors originate only in unchanged world-deps, world-gateway,
host-session-authority, orchestrator-dispatch, platform-macOS, and async-REPL test surfaces. This is
an inherited ambient failure, not a candidate regression.

## Closure and containment

The fresh closure reviewer independently recomputed the fingerprint and returned **CLEAN**, with
zero P1/P2 findings. It retained nonblocking `FD3-CLOSURE-P3-001` solely to record the inherited
ambient shell-library failure and its base-equivalence proof. Linux publisher sources,
`world-mac-lima`, VSock, Windows/WSL, Cargo manifests/lock, and installer artifact staging are
unchanged. Actual XPC channel and attestation functions are byte-identical to base. The changed-byte
secret scan is clean, and the repository contains no evidence or closeout action from this packet.

External nonpersistent proof artifacts are under
`AUX-R3-MAC-FD3-STREAM-CORRECTION` in the task visualization directory. Key digests are:

- remediated deterministic proof:
  `f9e2fad3eae9d72fcbb341bb25fc9a41cd6bc52f8d3e14e33212303e89fda296`;
- remediated allowlist/containment/secret scan:
  `8ff75d1b60d6a2d0db080f850077ce4fc4920cc62494ffded2c8d434991c5f07`;
- base shell-library comparison log:
  `07ee6385b423f90dfc99186f18b5b4f41aba16c7db7d5326bf221d25ba87bfe0`;
- base-equivalence report:
  `dbf6e6bf85289ae1693446bc511be4130369274733b68a8aacab96adcaeb4865`.

No installation, Keychain, launchd, Lima, codesigning mutation, publisher bootstrap, pairing,
`EVIDENCE:R3-MAC-IMP-01`, or `MAC-CLOSEOUT` action was run.
