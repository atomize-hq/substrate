# Plan: Agent Drift Analyzer Acceptance Fixture Hardening R2

## Scope

This plan implements:

- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`

`R2` is the analyzer-owned durability packet after the landed `R1E` bounded replay re-proof.
Its job is to preserve the known-good `R1E` final checkpoint outcomes inside
`crates/agent-drift-analyzer` without rerunning the broader replay workflow every time later
packets touch analyzer semantics.

This packet should:

- define one checked-in acceptance fixture contract for analyzer-local tests
- freeze the screened `R1E` non-subagent corpus into analyzer-owned fixtures
- assert final `dead_end_thrash` posture directly from analyzer output
- keep the frozen corpus small, explicit, and reviewable
- document why delegated and non-success-tail sessions stay excluded

This packet should not:

- change analyzer semantics
- widen the acceptance corpus beyond the screened `R1E` sessions
- move proof ownership back into sentinel or into `target/hybrid-drift-evals`
- claim new bounded replay or live proof
- broaden into `R3` turn-context, `R4` archetype, or `R5` progress work

## Why This Packet Comes Next

`R1E` proved the analyzer fix on a screened non-subagent replay corpus, but that proof currently
lives in replay artifacts and continuity notes. The analyzer crate still lacks a committed,
crate-local acceptance wall that protects the same final checkpoint outcomes from regression.

The current analyzer coverage is strong on synthetic scoring semantics, but it is still weak on
frozen real-session-shaped acceptance:

- `crates/agent-drift-analyzer/tests/dead_end_thrash.rs` covers targeted scorer behavior
- `crates/agent-drift-analyzer/tests/end_to_end.rs` covers rerun stability
- `crates/agent-drift-analyzer/tests/support/mod.rs` builds synthetic bundles in temp dirs

What is missing is one durable acceptance seam grounded in the landed `R1E` corpus:

- three cleared success-tail controls
- one representative recovered sticky success tail
- explicit exclusion of delegated and non-success-tail sessions

That is why `R2` comes before any new context-aware scoring work.

## Implementation Strategy

### Workstream 1: Freeze The Fixture Contract

Define one analyzer-owned fixture shape under `crates/agent-drift-analyzer/tests/fixtures/acceptance/`.

Each case should include:

- `manifest.json`
- `rows.archival.jsonl`
- `rows.compact.jsonl`
- `dedupe-audit.jsonl`
- `expected.json`

`expected.json` should hold only the analyzer-owned assertions needed by `R2`, especially the
final `dead_end_thrash` state, `flagged` bit, and `raw_score`.
The bundle files themselves should be copied from the landed `R1E` compactor artifacts so the
acceptance seam stays provenance-preserving rather than partially re-synthesized.

Why first:

- later test helpers and acceptance assertions depend on a stable per-case layout
- this is the clean boundary between extraction-time artifacts and committed test inputs

### Workstream 2: Curate The Frozen Corpus

Freeze the screened `R1E` corpus into committed analyzer fixtures:

- cleared controls:
  - `019e93fa-60d4-73d1-9092-014130b60e14`
  - `019e940c-a91b-7fe0-a967-b0bdd595b581`
  - `019e943c-668e-7a03-992b-6a98cf3055da`
- recovered sticky case:
  - `019e894a-86c9-71e3-b57b-e3d3285f0988`

Keep the following sessions out of the committed success-tail corpus:

- delegated runs:
  - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
  - `019e9406-6736-79a2-946b-8a603e557422`
- non-success-tail run:
  - `019e9401-9d69-7190-a43e-9ee3be08b369`

Why second:

- the acceptance wall should protect exactly the landed proof surface, not a silently expanded
  approximation of it

### Workstream 3: Add Analyzer Acceptance Helpers

Extend analyzer test support so acceptance cases can be loaded from disk, materialized into a temp
output directory, and inspected through analyzer-owned helpers.

The helpers should:

- locate a case by session id
- create a temporary output directory
- run `analyze_bundle(...)` against the checked-in input bundle
- read checkpoints and expose the final `dead_end_thrash` score cleanly

Why third:

- the case-loading seam should be reusable by every acceptance fixture without each test
  re-implementing file-system plumbing

### Workstream 4: Add The Acceptance Wall

Add one analyzer test target dedicated to frozen acceptance fixtures.

That wall should assert:

- each cleared control ends `state=Cleared`, `flagged=false`, `raw_score=0`
- the representative sticky case ends `state=Recovered`, `flagged=false`, `raw_score=20`

It may also assert stable final checkpoint counts or case metadata if that improves clarity, but
the primary contract is final checkpoint honesty, not summary prose.

Why fourth:

- once the contract, corpus, and helpers are stable, the acceptance wall becomes small and direct

### Workstream 5: Lock Maintenance Rules

Document how the corpus changes in future packets:

- adding sessions requires explicit approval
- changing expected outcomes requires an updated packet authority
- extraction sources stay external to committed tests
- missing checked-in fixtures fail closed instead of falling back to `target/` or `~/.codex`

Why last:

- the maintenance rules should describe the final implemented acceptance seam, not a draft version

## Sequencing

Sequential work:

1. lock the `R2` fixture contract in docs
2. decide and document the per-case committed file layout
3. freeze the four allowed `R1E` cases into committed fixtures
4. add analyzer test helpers for case loading and final-score inspection
5. add the analyzer acceptance wall over the frozen corpus
6. run focused and full analyzer verification

Parallel-safe work after the fixture contract is locked:

- curate individual cleared-control cases in parallel
- draft `expected.json` files while bundle reduction is happening
- sketch acceptance test names while fixtures are being copied or reduced

Not parallel-safe:

- changing the fixture layout after cases have already been curated
- broadening the corpus while acceptance assertions are being written
- coupling the acceptance wall to sentinel output instead of analyzer output

## Major Risks And Mitigations

### Risk 1: Fixture Bloat Makes The Corpus Hard To Review

Mitigation:

- prefer reduced per-case bundles
- keep only the analyzer input files plus `expected.json`
- allow full per-case bundle input only when reduction would break honest reproduction

### Risk 2: The Acceptance Wall Quietly Reopens Proof Ownership

Mitigation:

- keep `R1E` as the only source of bounded replay proof claims
- phrase `R2` tests as committed acceptance regressions derived from landed proof, not fresh proof
- do not read from `target/` or `~/.codex` at test time

### Risk 3: Delegated Or Non-Success-Tail Sessions Leak Into The Corpus

Mitigation:

- encode the allowed session ids explicitly in docs
- keep excluded sessions cited in the spec and tasks
- require explicit approval before expanding the corpus

### Risk 4: The Harness Asserts The Wrong Thing

Mitigation:

- assert final analyzer checkpoint posture directly
- avoid using sentinel labels or summary prose as the source of truth
- keep the expected contract to structured fields: `state`, `flagged`, `raw_score`

### Risk 5: Later Packets Treat Fixture Updates As Routine Cleanup

Mitigation:

- state clearly that expected outcome changes require packet-level authority
- keep maintenance rules inside the packet docs and nearby test helpers
- preserve case naming by session id so reviews can trace provenance easily

## Verification Checkpoints

### Checkpoint 1: Doc Boundary Locked

Confirm the spec, plan, and tasks all agree that:

- `R2` is analyzer-local acceptance hardening only
- the corpus is exactly the screened `R1E` success-tail set
- delegated and non-success-tail sessions stay excluded

### Checkpoint 2: Fixture Contract Locked

Confirm each committed case follows one stable layout and carries the structured expected final
posture contract.

### Checkpoint 3: Acceptance Wall Passes

Run:

```bash
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
```

The wall should prove the frozen corpus still ends with the landed final analyzer posture.

### Checkpoint 4: Analyzer Wall Still Passes

Run:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

The new acceptance seam should complement existing analyzer tests rather than destabilize them.
