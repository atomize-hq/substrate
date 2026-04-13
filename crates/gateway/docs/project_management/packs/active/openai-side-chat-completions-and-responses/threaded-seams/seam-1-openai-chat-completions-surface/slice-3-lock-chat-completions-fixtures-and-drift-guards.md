---
slice_id: S3
seam_id: SEAM-1
slice_kind: conformance
execution_horizon: future
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-10` or `C-12` changes after fixtures land"
    - provider-normalized output changes in a way that invalidates the locked sync or stream assertions
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-10
  - THR-11
contracts_produced:
  - C-10
  - C-12
contracts_consumed: []
open_remediations: []
---
### S3 - Lock Chat Completions Fixtures And Drift Guards

- **User/system value**: the compatibility surface gains deterministic regression coverage that proves the endpoint still matches the owned contracts and protects downstream seams from silent drift.
- **Scope (in/out)**:
  - In: add positive and negative contract tests, golden fixtures, and drift guards for sync and stream behavior on `/v1/chat/completions`.
  - Out: conformance coverage for `/v1/responses` or pack-wide drift-guard ownership that belongs to `SEAM-3`.
- **Acceptance criteria**:
  - positive fixtures cover sync text-only, tool-call-only, mixed output, streamed text deltas, streamed tool deltas, usage behavior, and `[DONE]`
  - negative fixtures cover known-but-unsupported fields, built-in tools or non-function tool types, and contracted error-envelope behavior
  - at least one regression surface proves the route remains a thin transform over normalized semantics rather than a provider-specific parser
  - the fixture set is stable enough that `SEAM-3` can later consume it as publication-backed evidence rather than rediscovering behavior from scratch
- **Dependencies**: `S00`, `S1`, `S2`, `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, `gateway/src/providers/openai.rs`, `gateway/tests/` or equivalent fixture locations, `THR-10`, `THR-11`
- **Verification**:
  - the seam can point to deterministic sync and stream tests plus fixture locations
  - failure conditions are explicit: chunk-shape drift, tool-call drift, unsupported-field drift, or provider-specific public parsing reappearing in the route
  - pass condition: the seam owns a durable test surface that supports publication of `THR-10` and `THR-11`
- **Rollout/safety**: prefer golden fixtures and deterministic in-repo verification; do not depend on live upstream network behavior for contract lock-in.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Falsification questions`, `Planned seam-exit gate focus`)

#### S3.T1 - Add Positive Contract Fixtures

- **Outcome**: the seam has deterministic sync and stream evidence for the supported subset.
- **Inputs/outputs**: inputs are landed sync and stream behavior from `S1` and `S2`; outputs are golden fixtures and positive-path regression tests.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: cover sync and stream with the same contract truths so tool calls, finish reasons, and usage behavior do not drift across modes.
- **Acceptance criteria**: the fixture set covers text-only, tool-call-only, mixed sync output, streamed text deltas, streamed tool-call deltas, and `[DONE]`.
- **Test notes**: store fixtures where downstream seams can cite them directly and keep assertion names anchored to contract behavior instead of implementation detail.
- **Risk/rollback notes**: if positive fixtures are ad hoc or provider-dependent, downstream conformance will not be able to trust them.

Checklist:
- Implement: add positive-path fixtures and regression tests
- Test: run the focused sync and stream contract cases
- Validate: confirm fixtures describe the public contract, not provider internals
- Cleanup: remove redundant one-off assertions that do not map back to `C-10` or `C-12`

#### S3.T2 - Add Negative Boundaries And Drift Guards

- **Outcome**: the seam has explicit coverage for rejected fields, tool-type boundaries, and architecture drift.
- **Inputs/outputs**: inputs are `C-10`, `C-12`, and the landed handler/adapter code; outputs are negative-path tests and drift-guard assertions.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: make it easy to detect regressions in reject/ignore posture, chain-of-thought suppression, model echo, and public stream conversion boundaries.
- **Acceptance criteria**: non-function tools and known-but-unsupported fields reject correctly, error envelopes stay contracted, and at least one guard fails if the route starts parsing provider-specific public stream framing.
- **Test notes**: include regression cases for unsupported fields, built-in tools, error classification, and thin-adapter invariants.
- **Risk/rollback notes**: without negative coverage, the compatibility route can drift outward silently while still passing happy-path tests.

Checklist:
- Implement: add negative-path and thin-adapter drift-guard tests
- Test: run focused rejection and invariant cases
- Validate: confirm failure messages point back to contract violations
- Cleanup: align test names and fixture labels with `C-10` and `C-12`
