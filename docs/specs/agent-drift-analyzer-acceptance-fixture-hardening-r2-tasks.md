# Tasks: Agent Drift Analyzer Acceptance Fixture Hardening R2

This task list implements:

- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
- `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`

## Task List

- [ ] Task: Lock the `R2` packet boundary and acceptance-fixture contract in repo docs
  - Acceptance:
    - the docs define `R2` as analyzer-local acceptance hardening only
    - the docs name the exact included screened `R1E` sessions:
      - `019e93fa-60d4-73d1-9092-014130b60e14`
      - `019e940c-a91b-7fe0-a967-b0bdd595b581`
      - `019e943c-668e-7a03-992b-6a98cf3055da`
      - `019e894a-86c9-71e3-b57b-e3d3285f0988`
    - the docs name the excluded sessions and why they stay out
    - the docs define one stable per-case fixture layout and expected posture contract
    - the docs say the committed bundle files come from landed `R1E` compactor artifacts and that
      `expected.json` is the Packet `R2`-local addition
    - the docs explicitly keep `R1E` as the sole owner of bounded replay proof claims
  - Verify: doc review against the `R2` spec, plan, and
    `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md`
    - `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-plan.md`
    - `docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-tasks.md`

- [ ] Task: Add analyzer test support for checked-in acceptance cases
  - Acceptance:
    - analyzer test helpers can load one named acceptance case from disk
    - helpers materialize a temp output directory without depending on `target/` or `~/.codex`
    - helpers expose the final `dead_end_thrash` score from analyzer output cleanly
    - the helper contract stays analyzer-local and does not call sentinel
  - Verify:
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`

- [ ] Task: Freeze cleared control fixture `019e93fa-60d4-73d1-9092-014130b60e14`
  - Acceptance:
    - one checked-in case exists for session `019e93fa-60d4-73d1-9092-014130b60e14`
    - the case contains analyzer input files plus `expected.json`
    - the expected final posture is `state=Cleared`, `flagged=false`, `raw_score=0`
    - the committed fixture is derived from the landed `R1E` source artifact, not hand-invented
  - Verify:
    - manual diff against the corresponding `R1E` source case
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/019e93fa-60d4-73d1-9092-014130b60e14/`

- [ ] Task: Freeze cleared control fixture `019e940c-a91b-7fe0-a967-b0bdd595b581`
  - Acceptance:
    - one checked-in case exists for session `019e940c-a91b-7fe0-a967-b0bdd595b581`
    - the case contains analyzer input files plus `expected.json`
    - the expected final posture is `state=Cleared`, `flagged=false`, `raw_score=0`
    - the committed fixture is derived from the landed `R1E` source artifact, not hand-invented
  - Verify:
    - manual diff against the corresponding `R1E` source case
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/019e940c-a91b-7fe0-a967-b0bdd595b581/`

- [ ] Task: Freeze cleared control fixture `019e943c-668e-7a03-992b-6a98cf3055da`
  - Acceptance:
    - one checked-in case exists for session `019e943c-668e-7a03-992b-6a98cf3055da`
    - the case contains analyzer input files plus `expected.json`
    - the expected final posture is `state=Cleared`, `flagged=false`, `raw_score=0`
    - the committed fixture is derived from the landed `R1E` source artifact, not hand-invented
  - Verify:
    - manual diff against the corresponding `R1E` source case
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/019e943c-668e-7a03-992b-6a98cf3055da/`

- [ ] Task: Freeze recovered sticky fixture `019e894a-86c9-71e3-b57b-e3d3285f0988`
  - Acceptance:
    - one checked-in case exists for session `019e894a-86c9-71e3-b57b-e3d3285f0988`
    - the case contains analyzer input files plus `expected.json`
    - the expected final posture is `state=Recovered`, `flagged=false`, `raw_score=20`
    - the committed fixture is derived from the landed `R1E` source artifact, not hand-invented
  - Verify:
    - manual diff against the corresponding `R1E` source case
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/019e894a-86c9-71e3-b57b-e3d3285f0988/`

- [ ] Task: Add the analyzer acceptance wall over the frozen corpus
  - Acceptance:
    - analyzer acceptance tests assert the final `dead_end_thrash` state, `flagged` bit, and
      `raw_score` for each frozen case
    - the three cleared controls all end `Cleared / false / 0`
    - the representative sticky case ends `Recovered / false / 20`
    - tests read only checked-in analyzer fixtures and analyzer output
    - tests do not call sentinel or treat summary prose as the source of truth
  - Verify:
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/`

- [ ] Task: Lock fixture maintenance and exclusion rules near the acceptance seam
  - Acceptance:
    - the acceptance seam documents that delegated sessions remain excluded
    - the acceptance seam documents that `019e9401-9d69-7190-a43e-9ee3be08b369` remains outside
      the success-tail corpus
    - later corpus changes require explicit approval and packet authority
    - tests do not silently fall back to live artifacts when a fixture is missing
    - fixture metadata or nearby seam docs record that the copied bundle inputs are frozen from
      landed `R1E` artifacts rather than re-derived at test time
  - Verify:
    - doc review in nearby test comments or fixture metadata
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/`

Packet `R2` exit condition:

- analyzer owns a checked-in acceptance corpus derived from the screened `R1E` sessions
- the final checkpoint posture is asserted directly for the three cleared controls and the one
  recovered sticky case
- delegated and non-success-tail sessions stay explicitly excluded from the success-tail corpus
- the acceptance wall runs without mutable live artifacts and without reopening bounded replay
  proof ownership
