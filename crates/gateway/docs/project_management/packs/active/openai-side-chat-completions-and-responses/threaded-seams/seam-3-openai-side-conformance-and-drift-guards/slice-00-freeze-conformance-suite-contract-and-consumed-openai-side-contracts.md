---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-10` or `C-12` drift in a way that invalidates fixture shapes or the conformance assertions"
    - "`C-11` publishes with materially different Responses event or item rules than the provisional assumptions in this slice"
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
### S00 - Freeze Conformance Suite Contract And Consumed OpenAI-Side Contracts

- **User/system value**: execution starts from one concrete conformance contract (`C-13`) and explicit consumption of `C-10`/`C-11`/`C-12`, instead of inventing drift-guard behavior while writing tests.
- **Scope (in/out)**:
  - In: define the owned `C-13` contract (what the suite must assert, what variance is allowed), define the fixture schema and stream-replay boundary, and name canonical artifact locations.
  - Out: implementing tests, expanding public API behavior, or publishing new OpenAI-side feature scope.
- **Acceptance criteria**:
  - one canonical landing artifact path is named for `C-13`
  - `C-13` explicitly maps required behaviors to test categories for:
    - `C-10` Chat Completions (sync + stream + tool loop + reject/ignore + error envelope)
    - `C-11` Responses (sync + stream + tool loop + reject/ignore + error envelope)
    - `C-12` shared invariants (model echo, `X-Provider` forcing, reasoning suppression)
  - the contract states the determinism requirements: no live upstream calls, stable ordering assertions, and tolerance rules for optional fields
  - the contract defines the minimal fixture set and file layout under `gateway/tests/fixtures/` for both endpoints
  - the verification checklist names exact test entrypoints and pass/fail conditions needed for `gates.pre_exec.contract`
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-3-openai-side-conformance-and-drift-guards.md`, `docs/foundation/openai-side-chat-completions-c10-contract.md`, `docs/foundation/openai-side-responses-c11-contract.md`, `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- **Verification**:
  - a reviewer can answer: what drift is caught, what variance is allowed, and how the suite stays offline and deterministic without inspecting runtime diffs
  - pass condition: `SEAM-3` can later satisfy `gates.pre_exec.contract` without waiting for closeout-backed publication to exist already
- **Rollout/safety**: keep `C-13` narrow and contract-focused; avoid snapshotting incidental internal shapes that are not part of the public subset contracts.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- `C-13` (Conformance suite contract): `docs/foundation/openai-side-conformance-suite-c13-contract.md`
- Consumed contract anchors:
  - `C-10`: `docs/foundation/openai-side-chat-completions-c10-contract.md`
  - `C-11`: `docs/foundation/openai-side-responses-c11-contract.md`
  - `C-12`: `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- Normative policy baseline: `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`

#### S00.T1 - Define The Suite Scope And Assertion Strategy

- **Outcome**: `C-13` defines required assertions and tolerated variance per endpoint and shared behavior.
- **Inputs/outputs**: inputs are `threading.md`, `C-10`, `C-12`, and the planned `C-11`; outputs are the `C-13` artifact with a clause-to-test mapping table.
- **Thread/contract refs**: `THR-13`, `C-13` (consumes `THR-11` / `C-10`, `THR-12` / `C-11`, `THR-10` / `C-12`)
- **Implementation notes**: assert only what the contracts require (required fields and ordering guarantees) and explicitly tolerate what the contracts allow to vary (optional fields, implementation-private ordering where not promised).
- **Acceptance criteria**: each contracted behavior is mapped to at least one positive and one negative test category where applicable.
- **Test notes**: define which cases must be golden (exact-match) versus contract-match (partial + ordering constraints).
- **Risk/rollback notes**: without variance rules, the suite will either be brittle (false positives) or too loose (miss drift).

Checklist:
- Implement: write `C-13` with explicit assertion rules and a clause-to-test mapping table
- Test: enumerate the minimal test cases required to cover each clause
- Validate: verify every “high risk” hotspot has an explicit drift assertion strategy
- Cleanup: remove ambiguity about what is tolerated versus forbidden

#### S00.T2 - Freeze Fixture Schema And Offline Execution Rules

- **Outcome**: `C-13` defines the fixture file layout and stream-replay boundary that keeps CI deterministic.
- **Inputs/outputs**: inputs are existing `gateway/tests/fixtures/*` patterns and current adapter boundaries; outputs are fixture schema rules (JSON files, line-based SSE fixtures, and any normalization helpers).
- **Thread/contract refs**: `THR-13`, `C-13`
- **Implementation notes**: prefer deterministic stubs and in-process adapters; forbid live network usage by default.
- **Acceptance criteria**: one reviewer can see exactly where fixtures live, how streams are replayed, and how the gateway is exercised (HTTP vs direct transform) without reading the test code.
- **Test notes**: require at least one “fixture sanity” test that validates fixture parsing and replay determinism.
- **Risk/rollback notes**: if fixture schema is implicit, later slices will encode brittle ad hoc harness behavior.

Checklist:
- Implement: define fixture directories, file naming, and schema
- Test: include a fixture validation test plan
- Validate: confirm suite can run without network dependencies
- Cleanup: explicitly state the stream-replay boundary to avoid provider-shaped public fixtures
