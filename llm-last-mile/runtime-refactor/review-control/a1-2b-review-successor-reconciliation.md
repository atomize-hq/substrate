# A1.2b successor issuance and startup reconciliation review

Terminal subject fingerprint:
`sha256:72ac97adbbadd1692409782789daea7f1c64cdd34996563865b4d5959afc4b0b`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed checked the
final successor issuance and startup/post-turn reconciliation corridor without authoring subject
bytes. The terminal focus was preserved-Start versus V3 successor identity separation, exact retry
and join behavior, current-authority ancestry preservation, and released-transport join safety.

## Recorded causal lineage

The discovery subject was
`sha256:ecc3fe2f83745ec07507f4ef21d1758a020068e2755c624eeaed3e7a01c58df3`.
That reviewer found no blocking successor-reconciliation issue on the initial remediated
candidate.

The closure subject was
`sha256:be2b7f4352153c6bfd206fd223eab81c2b13510e824ccd1aa5087e7bf75564c8`.
It produced one accepted blocking finding:

- `A1-2B-P1-001`: successor issuance could reuse preserved Start `intent_id` or
  `issuer_request_id`, allowing Attach application to commit and then misrouting startup ownership
  resolution into the legacy Start branch.

## Supplemental causal evidence

The terminal subject closes `A1-2B-P1-001`:

- `crates/shell/src/execution/agent_runtime/host_session_authority/transition.rs` now rejects any
  successor issuance or exact-join attempt whose `intent_id` or `issuer_request_id` collides with
  preserved Start maps before mutation or retry join.
- `resolve_startup_ownership_inner` now enters the preserved Start path only for an exact preserved
  Start identity pair, not one-sided legacy-map presence.
- Focused tests
  `successor_attach_issue_rejects_preserved_start_identity_collisions`,
  `successor_attach_issue_claim_apply_retries_to_one_initial_application`,
  `successor_attach_startup_acceptance_preserves_current_authority_and_retries`, and
  `successor_resume_complete_cut_join_and_current_resolution_survive_released_transport_payload`
  pass on the terminal subject.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
