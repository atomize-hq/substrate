# R6 Correction-01 allowlist and evidence review — CLEAN

- Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R6-CORRECTION-01`
- Base: `a9626bc10a1737416c1d8f7f48630850118f3e6a`
- Final candidate subject: `sha256:6333a3f395b164ad484f31ad64786cb81a3ca47a0394be16bc6ee904cfdb89ce`
- Result: **CLEAN** for the amended R6 product/test fence and combined audit.

## Exact R6 changed product/test paths

1. `crates/common/src/lib.rs`
2. `crates/common/src/managed_artifact.rs`
3. `crates/shell/src/execution/managed_lifecycle.rs`
4. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`
5. `crates/shell/tests/managed_lifecycle_v1.rs`
6. `src/bin/substrate-lifecycle-control.rs`
7. `src/bin/substrate-lifecycle-linux.rs`
8. `src/bin/substrate-lifecycle-macos.rs`
9. `tests/mac/lifecycle_r3.sh`

The only review metadata for this correction is this report, its authority/lifecycle siblings, and `r3-mac-evidence-recovery-r6-correction-01-review-cycle-record.json`. The four original R6 review artifacts are immutable lineage inputs; their SHA-256 values remain `778a733a90d60541d3c1ea4388eddaf097b89063b321e33a59c0b9174209565a`, `5bf75f90792603dd6e27ccbce982185ae3a814b88e890a71a40d590d80144a70`, `b21b507548cd95ff01e37d829f353d618494adf382338d92bac8f0811464c51b`, and `12f9659045852167a8d0a6f89471f88b20237a691474c881f110cdde3d1f4485`.

## Mechanical containment

The product/test diff is exactly the nine paths above. Changed-byte credential scanning found no private key or known token pattern. Frozen Windows/WSL paths are unchanged, and static R6 guest-entrypoint analysis confirms no caller-selected `--state-root`, `--tty-path`, `--transport`, `--ticket-file`, or `--transcript-file` is accepted before ordinary Linux-host decoding. The public shell mapped tag has no operator-TTY discriminant; the private control-only tag is fixed. No direct SSH, VSock, TCP, generic broker, third session, public terminal FD API, XPC terminal descriptor, or PTY byte relay is present.

`cargo fmt --all -- --check`, `git diff --check`, and the focused checks listed by the lifecycle review passed. The Linux/Windows target compilation gates were not run under MAC-only authority amendment 0016; the allowed focused Lima-guest source tests/build and static containment proof were run. No native execution occurred.

## Combined R1-R6 audit

`git diff --name-only d4bd729de8d344dfc567aaa62203838f68c2c895..candidate` has 56 changed paths. They were attributed to the prior R1 installer, R2 shell/lifecycle, R3 planning/evidence, R4 common/macOS, R5 bridge/correction, and current R6 packets using the committed three-lens reports and cycle records. The final correction reviewer found no new valid P1/P2 outside the R6 fence. A historical P4 trailing-whitespace condition exists in three already-committed R5 review Markdown files; it is outside this fence, recorded only, and was not changed.
