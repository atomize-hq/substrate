---
seam_id: SEAM-2
seam_slug: adapter-protocol-and-schema
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-adapter-protocol-and-schema.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - backend-id selection semantics or the published status subset change upstream
    - adopted capability ids or extension keys change
    - request, response, error, or session-handle fields change
    - ADR-0017 event-envelope owner wording changes
    - ADR-0028 trace owner wording changes
gates:
  pre_exec:
    review: passed
    contract: pending
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations:
  - REM-002
  - REM-003
---
# SEAM-2 - Adapter protocol and schema

## Seam Brief (Restated)

- **Goal / value**:
  - Define one deterministic adapter lifecycle and one bounded schema inventory so gateway adapters can execute behind the published selection boundary without widening ADR-0017 or ADR-0028 ownership.
- **Type**:
  - integration
- **Scope**
  - In:
    - `gateway-backend-adapter-protocol-spec.md`
    - `gateway-backend-adapter-schema-spec.md`
    - adapter registry lookup and dispatch order
    - capability-validation order and fail-closed extension-key handling
    - request normalization and response emission ordering
    - adapter error object shape
    - backend-defined session-handle facet schema
    - the exact owner line between local adapter translation and ADR-0017 / ADR-0028
  - Out:
    - stable backend-id selection rules already owned by `SEAM-1`
    - cross-platform guarantee proof
    - ADR-0024 supersession proof and ADR-0040 alignment decisions
    - operator command-family semantics
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/alignment_report.md`
  - likely downstream code surfaces once execution begins:
    - `crates/agent-api-types/src/lib.rs`
    - `crates/agent-api-client/src/lib.rs`
    - `crates/world-agent/src/handlers.rs`
    - `crates/shell/src/builtins/world_gateway.rs`
- **Verification**:
  - This seam **consumes** `C-01` and `C-02` from `../../governance/seam-1-closeout.md`.
  - This seam **produces** `C-03` and `C-04`.
  - Readiness remains blocked until the exact adopted Universal Agent API subset and the local-to-external ADR-0017 / ADR-0028 owner line are concrete enough to implement without post-exec invention.
- **Basis posture**:
  - Currentness:
    - `current` because `SEAM-1` closeout is landed, `THR-01` is published, and the upstream handoff is now authoritative for this seam.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
  - Required inbound threads:
    - `THR-01`
  - Outbound publication thread:
    - `THR-02`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none for activation; `THR-01` is already published
  - Current readiness blockers:
    - `REM-002`
    - `REM-003`
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-03`
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- The current pre-exec record supports `execution_horizon: active` and `basis.currentness: current`, but readiness remains blocked on the open schema-subset and owner-line remediations.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-02` is the downstream handoff that `SEAM-3` must consume. Promotion into parity and validation work needs closeout-backed proof that `C-03` and `C-04` were published from one deterministic lifecycle and one bounded schema inventory.
- **Expected contracts to publish**:
  - `C-03`
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-02`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - capability or extension-key subset changes
  - request/response/error or session-handle schema changes
  - ADR-0017 or ADR-0028 owner-line wording changes
  - any widening of the adapter protocol boundary after publication
- **Expected closeout evidence**:
  - canonical `C-03` artifact path
  - canonical `C-04` artifact path
  - thread publication record for `THR-02`
  - planned-versus-landed delta for dispatch lifecycle, schema inventory, and owner-line wording

## Slice index

- `S00` -> `slice-00-c-03-c-04-contract-definition.md`
- `S1` -> `slice-1-dispatch-lifecycle-and-owner-line.md`
- `S2` -> `slice-2-schema-subset-and-fail-closed-capability-rules.md`
- `S3` -> `slice-3-adoption-surfaces-and-verification.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Threading Alignment

- **Contracts produced (owned)**:
  - `C-03`: deterministic adapter dispatch lifecycle and local-to-external owner line
  - `C-04`: bounded adopted Universal Agent API subset for capability advertisement, extension keys, request/response payloads, adapter errors, and session-handle facets
- **Contracts consumed**:
  - `C-01`: stable backend-id selection contract
  - `C-02`: published adapter-visible status boundary
- **Dependency edges**:
  - `SEAM-1` -> `SEAM-2` via `THR-01` carrying `C-01` and `C-02`
  - `SEAM-2` -> `SEAM-3` via `THR-02` carrying `C-03` and `C-04`
- **Parallelization notes**:
  - `S00` starts first because the owned protocol/schema baseline is still blocking readiness.
  - `S1` and `S2` may refine in parallel later, but execution readiness depends on both the lifecycle owner line and the adopted schema subset becoming concrete.
  - `S3` and `S99` wait for landed contract evidence and the realized `THR-02` publication record.

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeout consumed here: `../../governance/seam-1-closeout.md`
- Future seam closeout target: `../../governance/seam-2-closeout.md`
