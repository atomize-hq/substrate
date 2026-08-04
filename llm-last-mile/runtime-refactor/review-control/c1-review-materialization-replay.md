# C1 materialization, replay, and single-writer review

Terminal subject fingerprint:
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed checked the
terminal C1 materialization bytes without authoring subject bytes. The terminal focus was
deterministic obligation identity/revision, materialization equivalence, replay/no-op behavior,
same-session revision monotonicity, and exclusion of competing accepted-path writers.

## Recorded causal lineage

The discovery subject was
`sha256:edbbd444cb2281a6378fcd503ef4b6f5b6a9e6ffaa46bb0dac59973e37ae55bb`.
Its accepted materialization findings were:

- `C1-P2-002`: stored obligations were treated as equivalent on identity alone, so immutable
  materialization-derived field drift could survive duplicate/replay paths and exact-snapshot reads
  without failing closed.
- `C1-P3-002`: the state-store exact-noop branch allowed stale plans with extra obligations to
  bypass the exact expected-state comparison, reopening a replay/repair loophole.

The closure subject is
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`.
That changed subject closes both findings:

- `OrchestrationObligationRecord::matches_c1_materialization_projection` now compares the immutable
  materialization-derived fields rather than obligation identity alone.
- `apply_obligation_ledger_materialization_plan` no longer accepts the stale exact-noop shortcut; it
  requires exact expected-state equality, rebases only the same-session revision cursor when that is
  the sole legitimate drift, and otherwise fails closed.

## Terminal materialization state

The terminal subject preserves the required C1 replay and single-writer behavior:

- Byte-identical retained replay remains a no-op for canonical obligations, ledger revision, and
  materialized-event coverage.
- The accepted retained path materializes obligations as each durable typed `Event` or terminal
  `Exit` is observed; it does not wait for foreground return.
- Immutable obligation drift is rejected on both exact-snapshot reconstruction and the compatibility
  host-inbox projection fast path.
- The old terminal-coupled writer remains removed from the accepted retained C1 path, so the
  accepted path has one C1 writer.
- The targeted rebase, snapshot, obligation-ledger, StateStore, and retained-dispatch tests that
  cover this corridor are green in the recorded proof set.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
