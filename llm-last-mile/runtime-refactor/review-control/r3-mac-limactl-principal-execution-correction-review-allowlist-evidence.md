# R3 MAC limactl principal-execution correction — allowlist and evidence review

- Packet: `AUX-R3-MAC-LIMACTL-PRINCIPAL-EXECUTION-CORRECTION`
- Terminal subject: `sha256:de7e47a4445345653f6506a28cb51242f304a2e39faff21bede048748cbc894f`
- Final verdict: **CLEAN** — P0/P1/P2/P3 are zero

The product/test subject is limited to:

- `src/bin/substrate-lifecycle-macos.rs`
- `tests/mac/lifecycle_r3.sh`
- `tests/mac/limactl_principal_runner_r3.sh`

The correction changes only the macOS lifecycle `limactl` principal runner, Stage-1 profile FD3,
post-PM copy-source FD4, directly related observations/cleanup, and focused fixtures. It does not
change Windows/WSL, ordinary Linux hosts, transport, selectors, pairing protocol, VSock, or an
ownership-separated Lima architecture.

The bounded live Darwin proof uses a compiled fake `limactl` for credential, environment, FD,
ownership, drift, substitution, retry, nonzero, timeout, and disconnect cases. Its only real Lima
operation is `limactl validate /dev/fd/3` against an isolated temporary home. It creates, starts,
stops, deletes, or removes no instance. No official evidence packet, canonical installer,
Keychain, launchd, sudoers, known_hosts, prefix, or manual-attempt mutation was performed.

Final checks on the terminal subject:

- `cargo test --locked --offline --bin substrate-lifecycle-macos`: 23 passed, 1 ignored;
- the ignored root-only bounded live proof: 1 passed;
- `bash tests/mac/limactl_principal_runner_r3.sh`: passed;
- `bash tests/mac/lifecycle_r3.sh`: passed;
- `cargo check --locked --offline --target aarch64-apple-darwin --bin substrate-lifecycle-macos`: passed;
- `cargo fmt --all -- --check`: passed; and
- `git diff --check`: passed.

The terminal review fingerprint remained unchanged after these checks.
