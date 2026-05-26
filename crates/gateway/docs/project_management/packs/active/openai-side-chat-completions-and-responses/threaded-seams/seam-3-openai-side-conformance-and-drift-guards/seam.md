---
seam_id: SEAM-3
seam_slug: openai-side-conformance-and-drift-guards
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-3-openai-side-conformance-and-drift-guards.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-1-closeout.md
    - crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-2-closeout.md
  required_threads:
    - THR-10
    - THR-11
    - THR-12
    - THR-13
  stale_triggers:
    - any change to `C-10` Chat Completions public subset (sync mapping, streaming chunk semantics, tool-loop rules, reject/ignore posture)
    - any change to `C-11` Responses public subset (sync mapping, streaming event minimum set, tool-loop rules, reject/ignore posture)
    - any change to `C-12` shared adapter invariants that affects fixture shaping or determinism (tool IDs, stream conversion boundary, chain-of-thought suppression)
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-3 - OpenAI-Side Conformance and Drift Guards

## Seam Brief (Restated)

- **Goal / value**: prevent silent drift in OpenAI-side ingress by adding deterministic regression coverage that enforces the contracted subset for both endpoints and shared behavior across them.
- **Type**: `conformance`
- **Scope**
  - **In**:
    - conformance + negative-case tests for `POST /v1/chat/completions` per `C-10`
    - conformance + negative-case tests for `POST /v1/responses` per `C-11`
    - cross-endpoint shared-behavior tests per `C-12`:
      - model echo behavior
      - `X-Provider` forcing behavior
      - error envelope and status-code mapping
      - chain-of-thought / reasoning suppression guarantees
    - deterministic fixture and stream-replay harness that keeps CI offline (no live upstream calls)
  - **Out**:
    - broad fuzzing or load/perf work (can be added later)
    - live upstream network dependency in CI (tests must run against fixtures and local fakes)
    - pack-wide feature expansion; this seam is only about drift guards for already-contracted behavior
- **Touch surface**:
  - `gateway/tests/*` (new conformance tests and fixtures)
  - optional small test-only helpers that enable deterministic stream replay into the public adapters
  - documentation anchors for `C-13` (the conformance contract) and pointers to evidence
- **Verification**:
  - tests fail on: streaming termination regressions, tool-loop mapping regressions, error-envelope drift, model echo drift, and built-in tool leakage
  - tests include at least one “happy path” and one “rejection path” per endpoint, plus a cross-endpoint parity check for shared behavior
  - pre-exec readiness now rests on landed `C-10`, `C-11`, and `C-12` evidence rather than provisional assumptions
- **Basis posture**:
  - **Currentness**: `current` (revalidated against landed `SEAM-1` and `SEAM-2` closeout-backed `C-10`, `C-11`, and `C-12` truth)
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-1-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-2-closeout.md`
  - **Required threads**: `THR-10`, `THR-11`, `THR-12`, `THR-13`
  - **Stale triggers**:
    - any change to `SEAM-1` or `SEAM-2` public response shapes, error envelope rules, or streaming semantics
    - any change to internal tool representation or usage accounting that affects adapter output mapping
- **Threading constraints**
  - **Upstream blockers**: none at pre-exec; `THR-10`, `THR-11`, and `THR-12` are now published/revalidated enough for deterministic conformance execution, but any later contract or adapter-boundary delta would stale this basis
  - **Downstream blocked seams**: none in-pack; downstream is future maintenance outside this pack
  - **Contracts produced**: `C-13` (drift-guard / conformance suite contract)
  - **Contracts consumed**: `C-10` (Chat Completions subset), `C-11` (Responses subset), `C-12` (shared adapter invariants)

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream promotion should not assume “we have tests” without closeout-backed truth that the suite is deterministic, covers the contracted subset (positive + negative), and actually fails on drift in the high-risk streaming and tool-loop surfaces.
- **Expected contracts to publish**: `C-13`
- **Expected threads to publish / advance**: `THR-13` from `identified` to `published`
- **Likely downstream stale triggers**:
  - any later expansion of OpenAI-side supported subset (new fields, new event types, new item types) must revalidate or extend `C-13` instead of silently widening behavior
  - any change to normalized stream or tool representation that invalidates existing fixtures must explicitly update the suite and its rationale
- **Expected closeout evidence**:
  - one canonical artifact location for `C-13` plus a clear mapping from contract clauses to concrete tests
  - fixtures and stream-replay evidence that runs offline in CI
  - explicit publication accounting for `THR-13`

## Slice index

- `S00` -> `slice-00-freeze-conformance-suite-contract-and-consumed-openai-side-contracts.md`
- `S1` -> `slice-1-build-deterministic-openai-side-conformance-harness-and-fixtures.md`
- `S2` -> `slice-2-lock-chat-completions-conformance-and-negative-cases.md`
- `S3` -> `slice-3-lock-responses-conformance-and-negative-cases.md`
- `S4` -> `slice-4-lock-cross-endpoint-shared-behavior-parity.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
