# Packet R5 Progress Acceptance Fixtures

This directory is the dedicated Packet `R5-7` semantic wall for `session_progress`.
It is separate from the frozen Packet `R2` acceptance wall under
`crates/agent-drift-analyzer/tests/fixtures/acceptance/**`.

Authority boundary:

- **Native annotated real-rollout cases** are the primary semantic acceptance authority here.
- **Synthetic bundle-shaped cases** are committed supporting coverage only.
- **Adapted external cases** belong to a bounded **secondary robustness** lane only. Packet
  `R5.75-5.3` admits them one by one under this root without promoting them to primary semantic
  authority.
- Objective-shaped adapted evidence does not belong here; it may only be considered in
  `../objective_acceptance/stretch-external/` if it clears the net-new-signal bar beyond the locked
  native objective corpus.

Included native primary cases:

- `019e899c-453f-71f2-a99d-155848c7b081` — annotated real-rollout troubleshooting advancement.
- `019e940c-a91b-7fe0-a967-b0bdd595b581` — annotated real-rollout planning meander/stall.
- `019e8b42-42bd-7b10-baae-3265edb65f4b` — annotated real-rollout verification closeout narrowing.
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` — annotated real-rollout delegated parent-visible positive proof that stays guardrail-only under limited child visibility.
- `real-implementation-advancing-019e894a-ord6` — annotated real-rollout implementation verification-wall advancement.
- `real-closeout-conservative-019e767c-ord3` — annotated real-rollout closeout/review checkpoint that stays `insufficient_evidence` instead of overclaiming narrowing.
- `real-reopen-regressing-019e894a-ord7` — annotated real-rollout reopen/re-verify checkpoint that regresses honestly after a previously clean verifier breaks again.

Included synthetic support cases:

- `synthetic-planning-advancing` — committed bundle-shaped planning narrowing proof.
- `synthetic-implementation-advancing` — committed bundle-shaped implementation verification-wall proof.
- `synthetic-parent-visible-opaque` — committed delegated guardrail proof that limits progress claims to `parent_visible_orchestration`.
- `synthetic-zero-verifier-anti-flap` — committed bundle-shaped zero-verifier anti-flap proof that keeps exploratory probe misses on `planning_convergence` / `insufficient_evidence` instead of escalating into troubleshooting overclaim.

Committed adapted secondary cases for Packet `R5.75-5`:

- `adapted-sparse-readable-f47b81f39f2495dd` — secondary robustness for the `R5.75-2`
  sparse-readable conservative outcome.
- `adapted-zero-verifier-097d97e914ca220f` — secondary robustness for the `R5.75-4`
  zero-verifier anti-flap conservative outcome.
- `adapted-parent-visible-da59436e63915185` — secondary robustness for the combined `R5.75-3` /
  `R5.75-4` delegated parent-visible guardrail behavior.

Excluded cases:

- `019e93f8-a5e9-7490-ac1a-955b74c92ad0` — delegated orchestration live fixture stayed outside this corpus because the current analyzer output did not deterministically surface `parent_visible_orchestration` on the copied real bundle.
- `019e9406-6736-79a2-946b-8a603e557422` — delegated orchestration bundle likewise stayed out for the same reason; R5 keeps delegated proof bounded to the explicit guardrail case instead of overclaiming child progress.
- `019e9401-9d69-7190-a43e-9ee3be08b369` — review-only bundle remained outside because it duplicated the planning/closeout shape already covered without adding a new core dimension.
- `05a56cc51632982b` — adapted objective-shaped evidence is excluded from this progress corpus; it
  may only be reconsidered in `../objective_acceptance/stretch-external/` if a later packet proves
  net-new objective signal.

Maintenance rules:

- Every case directory must contain exactly `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, `dedupe-audit.jsonl`, and `expected.json`.
- Every `annotated_real_rollout` case must record `source_rollout_id`, screening metadata, required and forbidden signals, decisive-evidence notes, counter-evidence notes, and `why_not_other_dimensions` in `expected.json`.
- Every future adapted external case must stay explicitly labeled as **secondary robustness** and
  must cite the earlier packet-owned expectation it preserves.
- Tests must fail closed if a committed case is missing; they must not read from `target/` or `~/.codex` at test time.
- The legacy R2 acceptance wall remains frozen and is verified separately by `acceptance_fixtures.rs`.
- Delegated cases are guardrail-only in R5: they may prove `parent_visible_orchestration` with limited confidence, but they must not claim positive opaque-child progress before R7.
- At least one case in this corpus must remain an `annotated_real_rollout` before R5 claims bounded semantic acceptance.
