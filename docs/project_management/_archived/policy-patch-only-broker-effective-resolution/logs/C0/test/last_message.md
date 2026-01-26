- Added C0 acceptance tests in `crates/broker/src/tests.rs` (module `c0_policy_patch_only_broker_effective_resolution`) covering: patch-only sparse merge, `.substrate/workspace.disabled`, CLI `policy current show --explain` stderr JSON-only, and broker↔CLI effective-policy equality.
- Added test-only dep `serial_test` in `crates/broker/Cargo.toml` (lockfile updated in `Cargo.lock`).

**Commands run (in worktree)**
- `cargo fmt`
- `cargo test -p substrate-broker c0_ -- --nocapture` (expected red)
  - Fails deterministically for spec reasons:
    - Broker rejects sparse patch (`missing field id`)
    - Broker ignores `.substrate/workspace.disabled` (uses workspace policy anyway)
    - CLI `--explain` writes a note line before the JSON object on stderr
- `make triad-task-finish TASK_ID="C0-test"` (succeeded)

**Commit**
- `af0441875a59dd58ed6d05fceaac538270e1f418` on branch `policy-patch-only-broker-effective-resolution-c0-test`