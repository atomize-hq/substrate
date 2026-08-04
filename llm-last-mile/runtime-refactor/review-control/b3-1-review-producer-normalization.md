# B3.1 producer normalization and secret-safety review

Terminal subject fingerprint:
`sha256:57a1926c85c3069a6d7a42b5476f51edb5f7ef0032c31a68dee2c30d19294288`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed inspected
the final producer-normalization bytes and authored no subject byte. The terminal focus was the
fail-closed retained producer path in `crates/world-service/src/member_runtime.rs`.

## Recorded causal lineage

The discovery subject was
`sha256:f9b151fbdfcdf8a28846ef36ebe4f67d6ba6290cb193bc7f44ceca24db7c84f6`.
It produced three accepted findings:

- `B3-1-P1-001`: producer compatibility data preserved raw wrapper content, including secret-bearing
  prompt/command fields, instead of a sanitized retained payload.
- `B3-1-P2-001`: the typed payload was rebuilt from a narrow subset rather than preserving the safe
  producer payload object and its exact retained identifiers.
- `B3-1-P2-002`: the consumer compatibility projection lost required `event_class` / `event_id` /
  `message_id` semantics.

The closure subject was
`sha256:06536a430f05715d5358a6cecb9394c729a68df5d671569ec3c692158835cbac`.
That changed subject closed the discovery set except `B3-1-P1-002`: typed retained events still
allowed raw wrapper `message` / `text` to surface as the top-level compatibility message instead of
the sanitized retained payload message.

## Supplemental causal evidence

The terminal subject closes `B3-1-P1-002` in the producer-only normalization boundary:

- `agent_event_from_wrapper_event` now derives retained compatibility message text through
  `wrapper_event_compatibility_message` before `AgentEvent` construction at
  `crates/world-service/src/member_runtime.rs:1370-1457`.
- `normalized_world_worker_event_facet_v1`, `normalized_world_worker_event_class_v1`, and
  `typed_retained_thread_id_from_data` at
  `crates/world-service/src/member_runtime.rs:1570-1669` and `1966-1977` require explicit typed
  retained shape, explicit thread identity, and fail closed on unsupported or ambiguous wrapper
  taxonomies.
- `normalized_world_worker_payload_object`, `normalized_world_worker_payload_uaa_event`, and the
  recursive secret filter at `crates/world-service/src/member_runtime.rs:1759-1894` preserve the
  safe producer payload object, retain `event_id` / `message_id`, and strip secret-bearing
  `prompt` / `command` keys before typed or compatibility projection.
- The focused producer tests at `crates/world-service/src/member_runtime.rs:3656-3890` now prove
  that missing explicit thread IDs, unsupported payloads, and ambiguous wrapper fallback fail
  closed, while sanitized retained payloads keep required message and identity fields without
  leaking raw wrapper text.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
