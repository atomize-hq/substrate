---
slice_id: S1
seam_id: SEAM-3
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-12` changes the normalized stream boundary in a way that requires different stream fixtures or replay adapters"
    - "`SEAM-2` lands `/v1/responses` with a different public streaming event surface than the planned `C-11`"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-10
  - THR-11
  - THR-12
  - THR-13
contracts_produced:
  - C-13
contracts_consumed:
  - C-10
  - C-11
  - C-12
open_remediations: []
---
### S1 - Build Deterministic OpenAI-Side Conformance Harness And Fixtures

- **User/system value**: conformance tests can exercise both endpoints offline and deterministically, enabling drift guards that fail for real contract regressions without flaky infrastructure.
- **Scope (in/out)**:
  - In: create the test harness primitives for:
    - building an in-process gateway app/router with deterministic configuration
    - injecting stub providers and deterministic stream payloads
    - loading fixtures from disk and asserting against public OpenAI-shaped outputs
  - Out: writing the full suite for each endpoint (handled in later slices).
- **Acceptance criteria**:
  - harness supports: sync requests, streaming requests, tool-loop continuation requests, and negative-case requests
  - harness forbids live upstream network by default (stub provider + local fixtures only)
  - harness can assert:
    - JSON bodies (sync)
    - SSE lines/payloads (stream) including termination semantics
  - harness has one minimal “smoke conformance” test proving offline determinism end-to-end
- **Dependencies**:
  - `S00`
  - `gateway/src/server/mod.rs` (existing route wiring + test patterns for stub providers)
  - `gateway/src/server/openai_compat.rs` (Chat Completions transforms and existing unit-test surfaces)
  - `gateway/tests/fixtures/*` (existing fixture conventions)
  - `THR-10` / `C-12` (shared invariants), `THR-11` / `C-10` (Chat Completions)
- **Verification**:
  - at least one test proves the harness can exercise `/v1/chat/completions` sync and stream without any network calls
  - fixtures are validated (parse + replay deterministically) and produce stable assertions
  - pass condition: later endpoint-specific conformance slices can add cases without inventing new harness wiring
- **Rollout/safety**: keep harness APIs small; prefer reusing existing stub-provider patterns already used in server tests over introducing new testing frameworks.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md` (`R1`, `R2`)

#### S1.T1 - Create A Test App Builder With Stub Provider Injection

- **Outcome**: a reusable helper constructs an in-process server app with deterministic configuration and injected stub providers.
- **Inputs/outputs**: inputs are existing server test patterns (stub providers + `ProviderRegistry` injection); outputs are a small test-only helper module used by `gateway/tests/*`.
- **Thread/contract refs**: `THR-10`, `THR-13`, `C-12`, `C-13`
- **Implementation notes**: reuse the pattern from `gateway/src/server/mod.rs` tests (stub providers capturing `GatewayRequest` and returning deterministic responses/streams).
- **Acceptance criteria**: conformance tests can select a provider deterministically and can capture the normalized request boundary for assertions when needed.
- **Test notes**: include a sanity test that asserts the captured `GatewayRequest` matches `C-12` invariants (thin adapter behavior).
- **Risk/rollback notes**: if provider injection is ad hoc per test, conformance becomes duplicated and inconsistent.

Checklist:
- Implement: add a test app builder helper for conformance tests
- Test: validate provider injection and request capture behavior
- Validate: confirm no real provider network calls are possible via the helper
- Cleanup: keep helper API stable and narrow

#### S1.T2 - Define Fixture Loading And Stream Replay Utilities

- **Outcome**: fixtures and stream payloads can be loaded and replayed deterministically into sync and stream paths.
- **Inputs/outputs**: inputs are `gateway/tests/fixtures/*` conventions and existing stream adapter expectations; outputs are fixture-loader and stream-replay utilities.
- **Thread/contract refs**: `THR-13`, `C-13`
- **Implementation notes**: keep stream fixtures source-of-truth in repo files, not inline strings; ensure replay preserves exact ordering and termination boundaries.
- **Acceptance criteria**: fixtures support both “happy path” and negative-case payloads, and streaming fixtures can be asserted line-by-line.
- **Test notes**: add a fixture validation test that fails if fixture schema changes unexpectedly.
- **Risk/rollback notes**: without stream replay utilities, each streaming test will re-encode framing rules and drift independently.

Checklist:
- Implement: add fixture loader + stream replay utilities
- Test: add a fixture schema validation test
- Validate: confirm ordering/termination assertions are stable
- Cleanup: document fixture format choices in `C-13`
