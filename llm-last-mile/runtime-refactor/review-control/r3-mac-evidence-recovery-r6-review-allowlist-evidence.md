# R6 allowlist and deterministic-evidence review — bounded stop

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6`
- Candidate subject fingerprint: `sha256:018791e14303b553d10286933aba6f911b41b3ad3247cf7bb3aa76092ff51462`
- Combined audit range: `d4bd729de8d344dfc567aaa62203838f68c2c895..candidate`
- Result: **FINDINGS — MAC-only containment is present, but the candidate is not clean.**

## Changed-file union and containment

The implementation subject changes exactly these authorized R6 production/test paths:

1. `crates/common/src/lib.rs`
2. `crates/common/src/managed_artifact.rs`
3. `crates/shell/src/execution/managed_lifecycle.rs`
4. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`
5. `crates/shell/tests/managed_lifecycle_v1.rs`
6. `src/bin/substrate-lifecycle-control.rs`
7. `src/bin/substrate-lifecycle-linux.rs`
8. `src/bin/substrate-lifecycle-macos.rs`
9. `tests/mac/lifecycle_r3.sh`

The only permitted review metadata is this report, its two sibling R6 reports, and `r3-mac-evidence-recovery-r6-review-cycle-record.json`. No implementation diff appears under Windows/WSL paths or frozen scripts. Retired `GuestPublisherPairingGuestChannelV1`, `GuestPublisherPairingStdioPhaseV1`, `GuestPublisherPairingStdioFrameV1`, and `guest-pairing-stdio` are absent from the candidate source shape. This is containment evidence, not a clean verdict.

## Findings

1. **`P1-R6-OPERATOR-TTY-PIPE-RELAY`** — The only observed connection from retained host terminal input to the guest confirmation reader is the local child stdin pipe (`src/bin/substrate-lifecycle-macos.rs:8292-8384`), which the continuation explicitly excludes. The Linux comment and the static test require the same forbidden behavior (`src/bin/substrate-lifecycle-linux.rs:522-530`; `tests/mac/lifecycle_r3.sh`).
2. **`P2-R6-UNAUTHORIZED-PUBLIC-TERMINAL-API`** — The public `*_with_terminal_v1` and `open_mac_xpc_channel_with_terminal_v1` surface expands the named amendment fence instead of keeping the terminal wiring private and fixed.
3. **`P2-R6-PROOF-GAPS`** — Static string tests do not constitute the contract's executable negative proof for no pipe confirmation, terminal identity, no third session, or every loss/replay/CAS/expiry/Drop row.

## Proof observations

Focused common, shell integration, macOS binary, Linux guest-entrypoint source-shape, cargo checks (including the authorized macOS target check), `bash tests/mac/lifecycle_r3.sh`, formatting, and diff-whitespace checks were run during the candidate attempt. The full Linux binary suite is not a gate on this macOS host and has two pre-existing socketpair `ENOTSUP` failures; the focused R6 guest-entrypoint test passed. No installation, Keychain, XPC service, Lima, signing, native pairing, or evidence action was run.

The combined R1–R6 read-only audit found no independently valid P1/P2 outside the amended R6 fence. It records R1–R5's three-lens reports/cycle records and R5 Correction-01 as lineage; R6 has no terminal landing receipt because this packet stops before publication.
