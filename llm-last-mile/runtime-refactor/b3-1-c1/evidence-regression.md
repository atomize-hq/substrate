**Kind:** evidence and regression record
**Stable ID:** `B3.1-C1-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** completed B3.1 and C1 evidence, differential, and regression records only
**Source span:** composite of the two preserved root compatibility spans listed in the extraction ledger

# B3.1/C1 family evidence and regression record

## B3.1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, B3.1 closes the retained worker-to-host
typed event prerequisite without changing `ExecuteStreamFrame`. Producer normalization now happens
only in `world-service` before `AgentEvent` construction, every retained post-acknowledgement
`Event` frame in scope carries the typed `worker_event` member or fails closed before emission, and
shell consumers validate the complete envelope instead of using JSON-pointer repair on
`AgentEvent.data`.

Broad proof remained monotonic through both `make shell-lib-wall` and `make shell-lib-wall-serial`
at `1324 discovered / 1276 passed / 48 failed / 0 ignored` with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly two new passing shell tests:
`execution::orchestrator_world_dispatch::tests::accepted_retained_typed_event_validation_precedes_generic_journaling`
and
`execution::orchestrator_world_dispatch::tests::typed_control_ack_projection_uses_top_level_worker_event_class`.
The historical normalized-signature SHA-256
`2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90` first moved to the earlier
independently authorized intermediate candidate SHA-256
`a1753d3d2dcd19f36a9350660ce88d9f2713871dd7734414c005ab8e52e70470`, then to the superseded
serial candidate SHA-256 `401fd5d38e059961de7fd4f773f7a817b9f2c1790ded0500228ed1168f7e811a`,
and finally to the accepted candidate SHA-256
`167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17`. The accepted final
differential against the superseded serial candidate is limited to 23 FILE:LINE-only movements in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`; test names, file paths, columns, and
normalized panic bodies remain unchanged, and the exact historical-to-final differential remains
recorded in
[`review-control/b3-1-differential-evidence.json`](../review-control/b3-1-differential-evidence.json).
No failure name, failure message, assertion, test identity, or production behavior was removed,
renamed, substituted, ignored, or weakened. B3.1 is complete, C1 is recorded below, and A1.2b is
separately recorded as complete on the same bound Tuesday candidate by the preserved root
[`A1.2b recorded result`](../05-debug-regression-ledger.md#a12b-recorded-result); no seam is promoted.

## C1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, C1 completes the event-to-obligation
materializer and semantic-cut packet without implementing A1.2b. The accepted retained path now
reconciles exact B1 acceptance plus B2.1 durable retained `Event` refs and validated B3.1
envelopes into `ObligationLedger` as each typed `Event` or terminal `Exit` is durably accepted.
Attention-driving retained events materialize exactly one canonical obligation before the exact B0
terminal event; typed non-attention events advance the classified set and
`materialized_through_event_sequence` watermark without hidden attention; byte-identical replay is
a no-op. `ObligationLedgerSnapshotReadV1` remains `Pending` before exact terminal-cut coverage and
returns one `Complete` snapshot only at the exact B2.1 terminal cut, including the exhaustive
ordered retained-event vector, closed attention disposition, and sorted unresolved
canonical-record commitments. The old terminal-coupled production writer is removed from the
accepted C1 path and retained only on the non-C1 `WorkerContinueForkCommand` compatibility branch
so no competing accepted-path writer remains.

Focused ledger, StateStore, retained-event, and accepted-dispatch tests are green. Both
`make shell-lib-wall` and `make shell-lib-wall-serial` retain
`1330 discovered / 1282 passed / 48 failed / 0 ignored` and failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly six new passing shell tests:
`execution::agent_runtime::obligation_ledger::tests::unresolved_attention_entries_sort_by_canonical_commitment_before_obligation_id`,
`execution::agent_runtime::state_store::tests::close_prepared_internal_continue_approval_response_obligation_updates_c1_materialized_canonical_record`,
`execution::agent_runtime::state_store::tests::obligation_ledger_plan_persists_to_nonlegacy_root_and_cursor_derives_from_state`,
`execution::agent_runtime::state_store::tests::obligation_ledger_plan_rebases_session_revision_after_unrelated_acceptance_advance`,
`execution::orchestrator_world_dispatch::tests::b21_retained_materializes_attention_obligation_before_terminal_cut_and_replay_is_noop`,
and
`execution::orchestrator_world_dispatch::tests::b21_retained_snapshot_is_pending_until_exact_terminal_cut_then_completes`.
The prior accepted normalized-signature SHA-256
`167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17` moved to the accepted C1
candidate SHA-256 `e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40`. The accepted
retained-failure differential is limited to 23 FILE:LINE-only movements in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`; failure names, file paths, columns,
and normalized panic bodies remain unchanged, and the exact baseline-to-final differential is
recorded in
[`review-control/c1-differential-evidence.json`](../review-control/c1-differential-evidence.json).
No failure name, failure message, assertion, test identity, or production behavior is removed,
renamed, substituted, ignored, or weakened. C1 is complete, and A1.2b is separately recorded as
complete on the same bound Tuesday candidate by the preserved root
[`A1.2b recorded result`](../05-debug-regression-ledger.md#a12b-recorded-result); no seam is promoted.
