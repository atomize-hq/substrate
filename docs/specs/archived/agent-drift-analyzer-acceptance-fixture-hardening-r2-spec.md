# Spec: Agent Drift Analyzer Acceptance Fixture Hardening R2

## Assumptions I'm Making

1. Live repo truth on `2026-06-06` is the authority: the `R1` family is landed through bounded
   replay re-proof, and `R2` is the next analyzer-owned packet rather than a reopening of
   outcome-evidence or verification-loop semantics.
2. `R2` exists to preserve the `R1E` outcome, not to discover new semantics. It should harden a
   durable analyzer-local acceptance seam against regression.
3. The acceptance corpus should be frozen from the already-screened non-subagent `R1E` sessions:
   - cleared controls:
     - `019e93fa-60d4-73d1-9092-014130b60e14`
     - `019e940c-a91b-7fe0-a967-b0bdd595b581`
     - `019e943c-668e-7a03-992b-6a98cf3055da`
   - representative recovered sticky case:
     - `019e894a-86c9-71e3-b57b-e3d3285f0988`
4. Delegated sessions and the non-success-tail session remain outside the `R2` corpus:
   - delegated exclusions:
     - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
     - `019e9406-6736-79a2-946b-8a603e557422`
   - non-success-tail exclusion:
     - `019e9401-9d69-7190-a43e-9ee3be08b369`
5. The durable fixture seam should be analyzer-input-level, not sentinel-output-level. Tests
   should run `agent-drift-analyzer` against checked-in compactor-style bundle inputs and assert
   final checkpoint posture directly.
6. `target/hybrid-drift-evals/*-r1e/` is the extraction source for this packet, not a runtime
   dependency of the final tests. `R2` tests should not read from `target/` or `~/.codex`.
7. The checked-in corpus should be reduced enough to stay reviewable, but it must preserve the row
   identities and bundle semantics required to reproduce the final analyzer result honestly.
8. `R1E` remains the sole owner of bounded replay proof claims and continuity-note refresh. `R2`
   may cite that proof corpus, but it must not relitigate or overclaim it.

## Objective

Add analyzer-owned acceptance fixture hardening that preserves the landed `R1E` honesty result
inside `crates/agent-drift-analyzer`.

Primary users:

- the maintainer changing analyzer heuristics and needing a fast regression wall that proves the
  final checkpoint does not drift back to falsely active `dead_end_thrash`
- the reviewer checking later `R*` packets and needing one stable acceptance seam that is smaller
  than rerunning the full bounded replay corpus each time

Success means:

- analyzer tests own a checked-in acceptance corpus derived from the screened `R1E` sessions
- each acceptance case asserts the final `dead_end_thrash` state, `flagged` bit, and `raw_score`
  directly from analyzer output
- the three cleared controls stay cleared
- the representative formerly sticky session stays `recovered` and unflagged
- excluded delegated and non-success-tail sessions stay out of the checked-in success-tail corpus
- `R2` does not reopen sentinel proof ownership, turn context, archetype, progress, or broader
  scorer redesign

## Packet Boundary

This packet is **analyzer acceptance-fixture hardening only**.

In scope:

- define a checked-in analyzer acceptance fixture contract
- freeze the `R1E` success-tail corpus into analyzer-owned test fixtures
- add analyzer test helpers that materialize the frozen cases into temporary bundle inputs
- add analyzer acceptance tests that assert final checkpoint posture on the frozen corpus
- document fixture inclusion, exclusion, and update rules so later packets do not silently widen
  the corpus

Out of scope:

- changing analyzer semantics again
- changing sentinel scheduler, runtime, live input, or operator presentation
- adding new proof sessions beyond the screened `R1E` corpus
- changing compactor bundle schema
- adding turn-context, `session_archetype`, or `session_progress`
- claiming that `R2` itself constitutes new bounded replay or live proof

## Tech Stack

- Language: Rust 2021
- Target crate:
  - `crates/agent-drift-analyzer`
- Upstream fixture source:
  - `target/hybrid-drift-evals/*-r1e/compactor`
- Analyzer input contract:
  - compactor bundle `v0.2`
- Analyzer output contract:
  - checkpoint schema `v0.3`
- No new crate or external dependency is required

## Commands

Build:

```bash
cargo build -p agent-drift-analyzer
```

Focused acceptance wall:

```bash
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
```

Related regression wall:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
```

Full analyzer wall:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Formatting gate:

```bash
cargo fmt --all -- --check
```

Fixture extraction reference shape during implementation:

```bash
export SESSION_ID="<screened-session-id>"
export SOURCE_DIR="target/hybrid-drift-evals/${SESSION_ID}-r1e/compactor"
export CASE_DIR="crates/agent-drift-analyzer/tests/fixtures/acceptance/$SESSION_ID"
```

The checked-in tests must not depend on those environment variables after fixture creation.

## Project Structure

```text
crates/agent-drift-analyzer/tests/support/mod.rs
  Shared analyzer test helpers. R2 should add loader helpers for checked-in acceptance cases and
  temporary output directories.

crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
  New analyzer-owned acceptance wall for final checkpoint posture on the frozen R1E corpus.

crates/agent-drift-analyzer/tests/fixtures/acceptance/<case>/
  Checked-in analyzer-input fixture directory for one acceptance case. Each case should carry:
  - manifest.json
  - rows.archival.jsonl
  - rows.compact.jsonl
  - dedupe-audit.jsonl
  - expected.json
  The four bundle artifacts are copied from the landed `R1E` compactor output. `expected.json`
  is the only Packet `R2` addition inside each committed case.

target/hybrid-drift-evals/*-r1e/
  Extraction source only. Not referenced by committed tests.

docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-*.md
  Packet authority for the R2 acceptance hardening slice.
```

## Code Style

Make acceptance expectations explicit and analyzer-local. Assert the final checkpoint posture
directly instead of inferring success from summary prose or sentinel labels.

```rust
#[test]
fn representative_sticky_success_tail_stays_recovered() {
    let case = load_acceptance_case("019e894a-86c9-71e3-b57b-e3d3285f0988");
    let result = analyze_bundle(&case.request()).expect("analyze acceptance case");
    let final_score = case.final_dead_end_thrash(&result);

    assert_eq!(final_score.state, DriftState::Recovered);
    assert!(!final_score.flagged);
    assert_eq!(final_score.raw_score, 20);
}
```

Conventions:

- keep fixtures keyed to real screened session ids
- store expected posture in structured per-case data, not hard-coded prose comments
- assert final analyzer checkpoint posture directly
- keep acceptance helpers inside analyzer tests; do not route these checks through sentinel
- prefer reduced fixtures, but never at the expense of losing the row identity needed to preserve
  the analyzer result honestly
- fail closed when a checked-in case is missing; never fall back to `target/` or `~/.codex`

## Testing Strategy

Required test layers:

1. Fixture loading coverage
   - prove checked-in acceptance cases can be materialized into temporary analyzer input/output
     directories without relying on `target/` or `~/.codex`
2. Acceptance posture coverage
   - assert final `dead_end_thrash` posture on each frozen case:
     - three cleared controls
     - one recovered sticky case
3. Existing scorer regression coverage
   - keep `dead_end_thrash` and export tests green so the new acceptance seam supplements rather
     than replaces current synthetic coverage
4. Full analyzer wall
   - prove the new acceptance seam composes cleanly with the rest of the analyzer crate

## Boundaries

- Always:
  - keep `R1E` as the sole owner of bounded replay proof claims
  - keep the checked-in corpus limited to the screened `R1E` sessions listed above
  - assert final analyzer checkpoint posture directly from analyzer output
  - preserve stable session ids and expected final posture in fixture metadata
- Ask first:
  - adding more sessions to the acceptance corpus
  - checking in full unreduced replay artifacts if reduced fixtures are sufficient
  - moving the acceptance seam into sentinel or a cross-crate harness
  - changing expected final states away from the landed `R1E` results
- Never:
  - read acceptance fixtures from `target/` or `~/.codex` at test time
  - treat delegated sessions as valid `R2` success-tail fixtures
  - treat `019e9401-9d69-7190-a43e-9ee3be08b369` as a success-tail acceptance case
  - claim `R2` re-proves bounded replay or live behavior by itself

## Success Criteria

1. `crates/agent-drift-analyzer` gains a checked-in acceptance corpus derived from the screened
   `R1E` sessions.
2. Analyzer tests assert the final `dead_end_thrash` state, `flagged` bit, and `raw_score` for
   every frozen case.
3. The cleared control sessions still end:
   - `state=Cleared`
   - `flagged=false`
   - `raw_score=0`
4. The representative formerly sticky session still ends:
   - `state=Recovered`
   - `flagged=false`
   - `raw_score=20`
5. The acceptance wall runs entirely from checked-in fixtures and analyzer code, without reading
   mutable live artifacts.
6. The packet does not widen into new analyzer semantics, new proof claims, or `R3+` context work.

## Open Questions

No blocking questions remain for Packet `R2`. Later corpus changes, expected-posture changes, or
fixture-reduction work require new packet authority rather than silent acceptance-seam drift.

Future follow-up questions outside Packet `R2`:

1. Should the excluded non-success-tail session `019e9401-9d69-7190-a43e-9ee3be08b369` become a
   later negative acceptance fixture for a different packet, or remain cited-only until a packet
   explicitly needs mixed success/failure tails?
2. If a reduced per-case bundle cannot reproduce the landed final posture honestly, should a
   later packet allow a full per-case compactor bundle fixture for that session instead of forcing
   further reduction?
