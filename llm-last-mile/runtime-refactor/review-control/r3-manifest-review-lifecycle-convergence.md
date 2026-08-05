# R3 MANIFEST lifecycle and convergence review

Terminal subject fingerprint:
`sha256:8b9a601620f69303c60d458977f48509f48c0ba8fd977afa713a39b57fb6e394`

A fresh independent read-only `gpt-5.4` reviewer using Extra High reasoning and standard/default
speed returned `NO BLOCKING FINDINGS` for the bounded MANIFEST evidence bundle and authored no
subject byte.

## Subject and convergence scope

The reviewed subject is the packet-local managed-artifact core only:

- canonical manifest/action-receipt/protected-state encoders and validators in
  `crates/common/src/managed_artifact.rs`;
- manifest publication, receipt-index/head CAS helpers, and preserving provider stubs in
  `crates/shell/src/execution/managed_lifecycle.rs` plus platform stub clients;
- the hidden authenticated direct-interactive control binary in
  `src/bin/substrate-lifecycle-control.rs`;
- the new shell integration coverage in `crates/shell/tests/managed_lifecycle_v1.rs`; and
- the native evidence validator and focused tests under `scripts/ci/`.

No ordinary `Cli` or `run_shell_with_cli` path was changed.

## Crash, retry, and idempotency

The subject preserves the required non-destructive state-transition discipline:

- receipt publication is exact-byte idempotent and only advances receipt-index/head state when the
  selected manifest/head still match;
- manifest publication and receipt resumption reject mismatched scope, manifest generation, digest,
  signer, prepared-record digest, and planned-action joins;
- temp residue now forces manual recovery instead of silently merging stale bytes into a new
  transition; and
- provider-specific execution remains preserving `provider_unavailable` behavior until later
  LINUX/MAC/WIN packets replace the stubs.

## Proof coverage

The reviewed clean gate run passed:

- `cargo test -p substrate-common -- --nocapture`
- `cargo test -p shell managed_lifecycle --lib -- --nocapture`
- `cargo test -p shell --test managed_lifecycle_v1 -- --nocapture`
- `cargo build -p substrate --bin substrate-lifecycle-control`
- `make r3-native-evidence-validator-test`
- `cargo fmt --all`
- `cargo clippy -p substrate-common -p shell -p substrate --all-targets -- -D warnings`
- `git diff --check`

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
