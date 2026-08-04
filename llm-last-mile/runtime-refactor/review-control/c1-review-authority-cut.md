# C1 authority, semantic cut, and ownership review

Terminal subject fingerprint:
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed checked the
final C1 authority surface without authoring subject bytes. The terminal focus was the exact B1
acceptance join, the supervisor-owned terminal cut, the closed `ObligationLedgerSnapshotReadV1`
contract, and the preserved StateStore/HostSessionAuthority ownership boundaries.

## Recorded causal lineage

The discovery subject was
`sha256:edbbd444cb2281a6378fcd503ef4b6f5b6a9e6ffaa46bb0dac59973e37ae55bb`.
Its accepted authority-cut findings were:

- `C1-P2-001`: session-wide ledger cursor equality treated unrelated acceptance advancement within
  the same orchestration session as a stale conflict, blocking valid later C1 materialization.
- `C1-P3-001`: `read_obligation_ledger_snapshot` returned `Pending` when the caller requested a
  terminal cut that mismatched the already-observed exact supervisor terminal event, instead of
  failing closed on the contradictory cut request.

The closure subject is
`sha256:e0a6dc627760a38ec77367bd25cf4b1dc7f431cbe972e7696445bea3fa1d34e8`.
That changed subject closes both findings: the state-store materialization path now rebases
same-session revision cursors across unrelated acceptance advancement, and the snapshot read path
now rejects any requested terminal event ID or sequence that differs from the exact supervisor
observation.

## Terminal authority state

The terminal subject preserves the declared C1 ownership split:

- `crates/shell/src/execution/agent_runtime/state_store.rs` remains a physical persistence surface;
  it supports exact C1 CAS/rebase behavior but does not classify retained events, infer
  completeness, or assume HostSessionAuthority semantics.
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs` still owns the semantic cut:
  completeness remains gated by the exact B2.1 terminal event and the exhaustive ordered retained
  event set through that cut.
- `crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs` remains the source
  of durable retained journal refs and the exact terminal observation; C1 consumes that truth but
  does not reinterpret or recreate it.
- `crates/shell/src/execution/orchestrator_world_dispatch.rs` still routes only already-validated
  typed retained events into the ledger owner; HostSessionAuthority stays consume-only and
  unmodified.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
