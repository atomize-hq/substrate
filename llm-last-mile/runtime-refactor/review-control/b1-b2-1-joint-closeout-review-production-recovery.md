# B1/B2.1 joint closeout production and recovery review

## Discovery cycle

Subject fingerprint:
`sha256:e5893da84231eb69f394a4bcbb11fe27cb50a234ba08b9a4d73bc6d687bd5fac`

The fresh independent read-only reviewer found two blocking documentation overclaims in the
production/recovery lens.

- `B1-B2-1-P1-001` (`P1`): `b1-b2-1-joint-closeout-linux-evidence.md` described the supported Linux
  smoke as if it were source-bound proof for the bound commit/tree, but the artifact only proved
  the existing installed product boundary around `/home/spenser/.substrate/bin/substrate`.
- `B1-B2-1-P2-001` (`P2`): the closeout docs described retained `ContinueWorldWorker`
  caller/waiter-drop survival as if it were proved on the full production entrypoint, while the
  actual retained drop proof is compositional at
  `execute_accepted_continue_world_worker_stream_for_turn_kind(...)`.

Discovery verdict: `findings`.

## Closure cycle

Subject fingerprint:
`sha256:353093b37763ebe05d7f2eaff6fec44994d8067861b791a3d5a7392370fd9124`

The fresh delta-focused closure review confirmed that `B1-B2-1-P1-001` was resolved, but it found
one remaining blocking stale claim in the phase-map row.

- `B1-B2-1-P2-002` (`P2`): `03-phase-slice-map.md` still said the joint closeout proved “both
  accepted families survive caller drop/restart,” which reintroduced the retained full-entrypoint
  overclaim that the rest of the remediated packet had already narrowed to accepted-stream-boundary
  proof.

Closure verdict: `findings`.

## Supplemental causal cycle

Subject fingerprint:
`sha256:95bb6b041f64a4c13adfa34966526aaa9bdfca85c99c2656a891de0a3e4bb601`

The fresh supplemental causal reviewer verified that the last stale phase-map row now matches the
rest of the packet: actual caller/waiter/guard-drop proof is limited to `RunWorldTask` plus the
ephemeral accepted-task production routes, while retained `ContinueWorldWorker` keeps only the
full-dispatch handoff proof plus accepted-stream-boundary foreground-waiter-drop proof. The cited
docs now align with the preserved source/test split in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Supplemental verdict: `CLEAN`.
