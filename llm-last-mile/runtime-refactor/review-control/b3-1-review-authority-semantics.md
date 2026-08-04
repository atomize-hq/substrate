# B3.1 authority and envelope semantics review

Terminal subject fingerprint:
`sha256:57a1926c85c3069a6d7a42b5476f51edb5f7ef0032c31a68dee2c30d19294288`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed checked the
final B3.1 authority surface without authoring subject bytes. The terminal subject preserves the
declared ownership split: shared schema and strict decoding live in
`crates/common/src/agent_events.rs:218-554`, producer normalization and join happen only in
`crates/world-service/src/member_runtime.rs:1370-1883`, and retained consumer validation stays in
`crates/shell/src/execution/orchestrator_world_dispatch.rs:5574-5917` before the generic B2.1
journal handoff.

## Recorded causal lineage

The discovery subject was
`sha256:f9b151fbdfcdf8a28846ef36ebe4f67d6ba6290cb193bc7f44ceca24db7c84f6`.
Its accepted blocking findings were producer/consumer boundary issues rather than authority drift:
`B3-1-P1-001`, `B3-1-P2-001`, and `B3-1-P2-002`.

The closure subject was
`sha256:06536a430f05715d5358a6cecb9394c729a68df5d671569ec3c692158835cbac`.
That changed subject closed the discovery set except `B3-1-P1-002`, a producer-side compatibility
message leak in `member_runtime.rs`. No shell-side authority or B0/B1 join drift remained open.

## Terminal authority state

The terminal subject still requires typed retained events to carry exact B0 frame identity and the
complete B1/B3.1 join. `validate_typed_continue_world_worker_event` in
`crates/shell/src/execution/orchestrator_world_dispatch.rs:5732-5849` equality-checks stream,
frame, acceptance record, request/message causation, backend, world, active run, participants, and
opaque correlation before projecting compatibility fields. The shell then persists the already
validated canonical worker event through the existing generic journal path at
`crates/shell/src/execution/orchestrator_world_dispatch.rs:1890-1952`; it does not recover missing
meaning from JSON-pointer inference.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
