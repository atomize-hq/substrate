---
seam_id: SEAM-1
seam_slug: backend-selection-and-policy-surface
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-backend-selection-and-policy-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - canonical `C-01` or `C-02` rules change outside this seam
    - shell selection or auth-resolution logic changes outside the planned slice order
    - failure-bucket wording drifts between shell docs, shell tests, and runtime markers
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
  - REM-001
---
# SEAM-1 - Backend selection and policy surface

## Seam Brief (Restated)

- **Goal / value**:
  - Adopt already-published `C-01` and `C-02` truth at the shell boundary so downstream runtime work consumes executable behavior instead of relying on Codex-only special cases.
  - Make `THR-01` publishable through landed shell logic, deterministic tests, and supporting-doc drift guards.
- **Type**:
  - integration
- **Scope**
  - In:
    - shell validation and request construction in `crates/shell/src/builtins/world_gateway.rs`
    - shell lifecycle coverage in `crates/shell/tests/world_gateway.rs`
    - shell adoption of published backend selection, inventory, allowlist, and precedence rules from `C-01` / `C-02`
    - minimum supporting ADR-0046 alignment so implementation docs do not compete with canonical `docs/contracts/` ownership
    - closeout evidence needed to publish `THR-01`
  - Out:
    - integrated adapter binding metadata and capability gates
    - adapter payload schemas or managed runtime artifact semantics
    - status-schema widening, tuple metadata, or tuple-policy surface changes
    - Linux/macOS/Windows parity proof and rollout governance
- **Touch surface**:
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
- **Verification**:
  - This seam **consumes** published `C-01` and `C-02` and turns them into shell behavior plus evidence.
  - Readiness means `SEAM-2` can consume `THR-01` without inferring missing semantics from the current `cli:codex` implementation.
  - Verification for this seam centers on:
    - `validate_gateway_lifecycle_config` rejects empty or disallowed lifecycle posture before dispatch
    - `build_gateway_request` keeps selection on existing config/policy roots and passes only an allowed backend id to the runtime boundary
    - `resolve_integrated_auth_payload` plus `resolve_cli_codex_integrated_auth` enforce env-primary/file-fallback/no-mixed-source auth precedence
    - shell-side tests prove the distinction between invalid integration, policy denial, transient runtime failure, and component unavailable where the shell owns that classification
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
- **Basis posture**:
  - Currentness:
    - `current` because `SEAM-1` has no inbound closeout dependency and the seam is being decomposed against the latest extracted pack state plus the current shell implementation evidence.
  - Upstream closeouts assumed:
    - none
  - Required threads:
    - `THR-01`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none inside this pack; this is the first publishable seam on the critical path
  - Downstream blocked seams:
    - `SEAM-2`
    - `SEAM-3`
  - Contracts produced:
    - `C-01`
    - `C-02`
  - Contracts consumed:
    - no pack-owned consumed contracts; ADR-0040, ADR-0041, and existing config-policy docs are basis authorities only
  - Canonical contract refs:
    - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
    - `docs/contracts/substrate-gateway-policy-evaluation.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` remains orientation only and does not satisfy the seam-local review gate by itself.

## Seam-exit gate plan

- **Planned location**:
  - `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-01` is the controlling upstream handoff for both downstream seams. They must consume closeout-backed publication truth, not inferred shell behavior.
- **Expected contracts to consume**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01`: `identified` -> `published`
- **Likely downstream stale triggers**:
  - selection or inventory validation behavior changes at the shell boundary
  - auth precedence or no-host-fallback rules change
  - failure taxonomy shifts between invalid integration, policy denial, component unavailable, and dependency unavailable
  - supporting ADR-0046 docs drift away from canonical `docs/contracts/` truth
- **Expected closeout evidence**:
  - landed shell updates in `crates/shell/src/builtins/world_gateway.rs`
  - landed shell tests in `crates/shell/tests/world_gateway.rs`
  - supporting ADR-0046 doc alignment in `contract.md`, `policy-spec.md`, and `env-vars-spec.md`
  - recorded remediation disposition for any remaining landing-only follow-through, including `REM-001`

## Slice index

- `S00` -> `slice-00-c-01-c-02-contract-definition.md`
- `S1` -> `slice-1-selection-order-and-inventory-truth.md`
- `S2` -> `slice-2-policy-precedence-and-fail-closed-boundary.md`
- `S3` -> `slice-3-shell-adoption-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Threading alignment

- **Dependency edges**:
  - `SEAM-1` -> `SEAM-2` via `THR-01` carrying `C-01` and `C-02`
  - `SEAM-1` -> `SEAM-3` via `THR-01` carrying `C-01` and `C-02`
- **Execution posture**:
  - The seam remains `status: decomposed`: the seam-local review, contract, and revalidation gates all pass. Remaining work is implementation, conformance, and seam-exit evidence, with `REM-001` carried only as landing/closeout alignment.
- **Slicing strategy**:
  - baseline check, then shell selection implementation, then shell auth implementation, then conformance, then explicit seam exit

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
