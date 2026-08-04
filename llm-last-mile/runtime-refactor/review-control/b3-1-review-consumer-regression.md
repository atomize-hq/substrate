# B3.1 consumer regression and differential review

Terminal subject fingerprint:
`sha256:57a1926c85c3069a6d7a42b5476f51edb5f7ef0032c31a68dee2c30d19294288`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed checked the
terminal consumer path, compatibility projection, and broad-wall differential without authoring
subject bytes.

## Recorded causal lineage

The discovery subject was
`sha256:f9b151fbdfcdf8a28846ef36ebe4f67d6ba6290cb193bc7f44ceca24db7c84f6`.
Its accepted consumer-side finding was `B3-1-P2-002`: compatibility projection on the retained
continue-world-worker path stripped required `event_class`, `event_id`, and `message_id`
information.

The closure subject was
`sha256:06536a430f05715d5358a6cecb9394c729a68df5d671569ec3c692158835cbac`.
That changed subject left only the producer-side `B3-1-P1-002` open. The consumer/journal path was
otherwise clean.

## Terminal consumer state

The terminal subject preserves the canonical consumer contract:

- `classify_continue_world_worker_event_with_frame_identity`,
  `validate_typed_continue_world_worker_event`, and
  `typed_continue_world_worker_event_projection` in
  `crates/shell/src/execution/orchestrator_world_dispatch.rs:5574-5917` require `event.worker_event`
  on accepted retained events, enforce exact B0/B1 equality, and derive any remaining compatibility
  presentation from the validated typed member rather than from raw `AgentEvent.data` repair.
- The accepted retained path validates before durable generic journaling at
  `crates/shell/src/execution/orchestrator_world_dispatch.rs:1890-1952`, preserving the existing
  B2.1 canonical commitment handoff.
- The new focused shell tests
  `accepted_retained_typed_event_validation_precedes_generic_journaling` and
  `typed_control_ack_projection_uses_top_level_worker_event_class` lock the retained consumer
  ordering and control-ack projection boundary.
- `llm-last-mile/runtime-refactor/review-control/b3-1-differential-evidence.json` records the
  accepted August 4, 2026 serial and parallel walls: `1324/1276/48/0`, failure-name SHA-256
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`, normalized-signature
  SHA-256 `167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17`, byte-identical
  serial/parallel failure-name inventories, and byte-identical serial/parallel normalized
  signatures.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
