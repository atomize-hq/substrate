# A1.2b ledger and authority-boundary review

Terminal subject fingerprint:
`sha256:72ac97adbbadd1692409782789daea7f1c64cdd34996563865b4d5959afc4b0b`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed reviewed
the final A1.2b ledger-cut and boundary surface without authoring subject bytes. The terminal
focus was the unchanged C1 cut contract, successor/post-turn boundary discipline, review-pack
claim accuracy, and the distinction between `ReleaseEligible` state and any later `Released`
cleanup.

## Recorded causal lineage

The discovery subject was
`sha256:ecc3fe2f83745ec07507f4ef21d1758a020068e2755c624eeaed3e7a01c58df3`.
It produced one accepted boundary finding:

- `A1-2B-P2-001`: complete obligation snapshots still accepted a
  `materialized_through_event_sequence` beyond the exact terminal event, so the cut check was not
  fail-closed on overrun watermarks.

The closure subject was
`sha256:be2b7f4352153c6bfd206fd223eab81c2b13510e824ccd1aa5087e7bf75564c8`.
That changed subject closed `A1-2B-P2-001` but still exposed one accepted review-pack finding:

- `A1-2B-P2-003`: A1.2b docs and evidence still claimed a shipped destructive release step even
  though production only committed `ReleaseEligible` and the staged tests covered reopened
  `Released` join tolerance through a manually validated root mutation.

## Supplemental causal evidence

The terminal subject closes `A1-2B-P2-003` while preserving the earlier cut fix:

- `consume_obligation_snapshot` now requires the complete-snapshot materialization watermark to
  equal the exact terminal event sequence.
- Focused tests
  `successor_resume_complete_cut_rejects_materialization_watermark_beyond_terminal_event`,
  `successor_resume_complete_cut_with_unresolved_attention_advances_to_awaiting_attention`, and
  `successor_resume_complete_cut_join_and_current_resolution_survive_released_transport_payload`
  pass on the terminal subject.
- `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`,
  `03-phase-slice-map.md`, and `05-debug-regression-ledger.md` now describe A1.2b as
  `ReleaseEligible` handoff plus exact join tolerance for a separately validated reopened
  `Released` state; they no longer claim that this packet ships the destructive
  payload-deletion/root-advance step.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
