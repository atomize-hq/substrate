---
seam_id: SEAM-2
seam_slug: status-schema-and-policy-evaluation-surface
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-status-schema-and-policy-evaluation-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - `status --json` top-level shape changes
    - `client_wiring.*` family or absence semantics change
    - ADR-0042 additive metadata boundary changes
    - fail-closed placement or secret-delivery trust-boundary rules change
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
# SEAM-2 - Status schema and policy evaluation surface

## Seam Brief (Restated)

- **Goal / value**:
  - Publish one authoritative inventory seam for machine-readable status and fail-closed policy evaluation so runtime and docs can consume a single truth surface.
- **Type**: integration
- **Scope**
  - In:
    - `substrate world gateway status --json` top-level object shape
    - `client_wiring.*` field family
    - non-secret output guarantees and absence semantics
    - gateway-integration decision flow over existing ADR-0027 keys
    - fail-closed no-host-fallback rule and host-to-world secret delivery boundary
    - the ban on trusting gateway-local config, admin, or persistence surfaces as policy inputs
  - Out:
    - command spelling and ownership-table wording
    - typed world-service lifecycle/status endpoint shape
    - provisioning changes
    - manual validation/playbook lock-in and final docs alignment
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/gateway-status-schema-spec.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/policy-spec.md`
  - `crates/transport-api-types/src/lib.rs`
  - `crates/transport-api-client/src/lib.rs`
  - `docs/CONFIGURATION.md`
  - `docs/USAGE.md`
- **Verification**:
  - This seam consumes upstream contract `C-01`; upstream closeout now publishes the command family, status authority rule, stable env semantics, exit taxonomy, and ownership split that this seam must inherit.
  - This seam produces owned contracts `C-02` and `C-03`. Pre-exec verification now passes because the seam-local contract-definition slice makes the status-schema boundary, absence semantics, fail-closed taxonomy, and non-trust rules concrete enough for execution.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads (inbound): `THR-01`
  - Stale triggers:
    - `status --json` top-level shape changes
    - `client_wiring.*` family or absence semantics change
    - ADR-0042 additive metadata boundary changes
    - fail-closed placement or secret-delivery trust-boundary rules change
- **Threading constraints**
  - Upstream blockers:
    - no seam-local upstream closeout blocker exists
    - `SEAM-1` closeout now publishes `THR-01` with the operator boundary this seam consumes
  - Downstream blocked seams:
    - `SEAM-3`
    - `SEAM-4`
  - Contracts produced:
    - `C-02`
    - `C-03`
  - Contracts consumed:
    - `C-01`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `SEAM-3` and `SEAM-4` can only consume schema and policy truth once the JSON envelope boundary, fail-closed rules, and non-trust posture are published in closeout-backed form.
- **Expected contracts to publish**:
  - `C-02`
  - `C-03`
- **Expected threads to publish / advance**:
  - `THR-02`: `defined` -> `published`
  - `THR-03`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - any change to top-level JSON shape or conditional presence rules
  - any change to the `client_wiring.*` family or omission semantics
  - any change to fail-closed placement, no-host-fallback posture, or trust-boundary rules
  - any change to the boundary against ADR-0042 additive metadata
- **Expected closeout evidence**:
  - landed schema-spec publication for `C-02`
  - landed policy-spec publication for `C-03`
  - explicit non-secret and absence-semantics evidence
  - explicit fail-closed and non-trust evidence

## Slice index

- `S00` -> `slice-00-status-schema-and-policy-contract-definition.md`
- `S1` -> `slice-1-status-json-envelope-and-wiring-boundary.md`
- `S2` -> `slice-2-policy-evaluation-and-trust-boundary.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
