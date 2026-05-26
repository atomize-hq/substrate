---
seam_id: SEAM-1
seam_slug: adapter-selection-boundary
status: exec-ready
execution_horizon: active
plan_version: v3
basis:
  currentness: current
  source_seam_brief: ../../seam-1-adapter-selection-boundary.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - stable backend-id grammar changes
    - `llm.allowed_backends` evaluation order or deny-by-default semantics change
    - backend inventory filename-to-id matching changes
    - the adapter-visible `status --json` owner line changes
    - ADR-0041 path cleanup changes the cited authority set
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations:
  - REM-005
---
# SEAM-1 - Adapter selection boundary

## Seam Brief (Restated)

- **Goal / value**:
  - Freeze one stable backend-id contract, one allowlist-first selection boundary, one failure taxonomy, and one explicit publication boundary for any additive adapter-visible gateway status data before downstream seams define protocol or parity behavior.
- **Type**:
  - integration
- **Scope**
  - In:
    - `contract.md`
    - `policy-spec.md`
    - the rule that one stable backend id maps to one adapter identity
    - ordered evaluation across config, policy, and inventory inputs before adapter dispatch
    - invalid-selection versus dependency-unavailable versus policy-denied classification
    - the owner line for any additive adapter-visible `status --json` subset
    - the ban on trusting gateway-local config, admin, persistence, or session state for authorization
  - Out:
    - request and response payload shape
    - capability and extension-key subset
    - session-handle facets
    - event and trace handoff details beyond the selection boundary
    - Linux/macOS/Windows parity proof
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- **Verification**:
  - This seam **produces** `C-01` and `C-02`.
  - Readiness means the owned-contract baseline is concrete enough that downstream seams can execute against one backend-id truth, one failure taxonomy, and one status-publication boundary without reopening ADR-0027, the gateway operator contract, or the status schema owner line.
  - The canonical `C-01` text now lives at `docs/contracts/gateway/backend-adapter-selection.md`.
  - `C-02` is narrowed to the current v1 publication boundary: no additive adapter-visible `status --json` field family is published beyond `status` and `client_wiring.*`, and any future additive family requires an explicit status-schema update first.
- **Basis posture**:
  - Currentness:
    - `current` because this seam has no inbound closeout dependency and the current ADR + pre-planning packet still agree on the selection-boundary shape.
  - Upstream closeouts assumed:
    - none
  - Required threads:
    - `THR-01`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none inside this pack; this is the first contract-definition seam
  - Downstream blocked seams:
    - `SEAM-2`
    - `SEAM-3`
  - Contracts produced:
    - `C-01`
    - `C-02`
  - Contracts consumed:
    - no pack-owned consumed contracts; ADR-0027 and the gateway contract docs are basis authorities only

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- The current pre-exec record supports `status: exec-ready`; only post-exec landing, closeout, and the non-blocking ADR authority cleanup remain outstanding.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-01` is the pack's prerequisite handoff. Downstream promotion needs closeout-backed proof that `C-01` and `C-02` were published from one owner line rather than inferred from ADR prose.
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - backend-id grammar changes
  - selection-order or failure-bucket changes
  - any widening or owner shift for additive adapter-visible `status --json` data
  - ADR-0041 authority references drifting again after cleanup
- **Expected closeout evidence**:
  - canonical `C-01` artifact path
  - the explicit `C-02` owner line and bounded field family
  - thread publication record for `THR-01`
  - planned-versus-landed delta for selection boundary and status publication wording

## Slice index

- `S00` -> `slice-00-c-01-c-02-contract-definition.md`
- `S1` -> `slice-1-selection-evaluation-and-failure-taxonomy.md`
- `S2` -> `slice-2-status-owner-line-and-adr-authority-cleanup.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Threading Alignment

- **Contracts produced (owned)**:
  - `C-01`: stable `<kind>:<name>` backend-id semantics, ordered config/policy/inventory evaluation, and pre-dispatch failure taxonomy
  - `C-02`: one explicit owner line and field-family boundary for any additive adapter-visible gateway status data
- **Contracts consumed**:
  - none from other seams; this seam treats ADR-0027 plus the gateway operator/status/policy contracts as upstream authorities, not consumed pack contracts
- **Dependency edges**:
  - `SEAM-1` -> `SEAM-2` via `THR-01` carrying `C-01` and `C-02`
  - `SEAM-1` -> `SEAM-3` via `THR-01` carrying `C-01` and `C-02`
- **Parallelization notes**:
  - `S00` starts immediately because no upstream publication is required.
  - `S1` can refine the selection decision tree in parallel with `S2`; the owner line is now recorded, and the remaining `S2` work is ADR authority-path cleanup plus keeping that boundary aligned during landing.
  - `S99` waits for landed contract evidence and the realized `THR-01` publication record.

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
