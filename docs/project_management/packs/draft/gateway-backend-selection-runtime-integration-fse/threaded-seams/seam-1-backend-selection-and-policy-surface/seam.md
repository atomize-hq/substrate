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
    - backend inventory roots or filename rules change outside this seam
    - auth precedence changes in canonical contract docs or shell gateway handling
    - selection order, failure taxonomy, or trusted-input boundaries drift between canonical docs, supporting ADR-0046 docs, and `crates/shell/src/builtins/world_gateway.rs`
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations:
  - REM-001
  - REM-002
---
# SEAM-1 - Backend selection and policy surface

## Seam Brief (Restated)

- **Goal / value**:
  - Publish one authoritative integrated-lifecycle truth for backend selection and policy evaluation before runtime realization or parity proof consumes it.
  - Replace the current Codex-only implied behavior with explicit contract language for selection order, inventory discoverability, allowlist timing, trusted-input boundaries, and auth-source precedence.
- **Type**:
  - integration
- **Scope**
  - In:
    - canonical publication of `C-01` in `docs/contracts/substrate-gateway-backend-adapter-selection.md`
    - canonical publication of `C-02` in `docs/contracts/substrate-gateway-policy-evaluation.md`
    - supporting ADR-0046 alignment in `contract.md`, `policy-spec.md`, and `env-vars-spec.md`
    - shell gateway request construction and pre-dispatch validation in `crates/shell/src/builtins/world_gateway.rs`
    - backend inventory roots, filename/id consistency, deny-by-default allowlisting, trusted-input boundaries, and auth precedence
    - invalid integration, dependency unavailable, and policy denial classification at the selection boundary
  - Out:
    - integrated adapter binding metadata and capability gates
    - adapter payload schemas or managed runtime artifact semantics
    - status-schema widening, tuple metadata, or tuple-policy surface changes
    - Linux/macOS/Windows parity proof and rollout governance
- **Touch surface**:
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/contract.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md`
  - `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md`
  - `crates/shell/src/builtins/world_gateway.rs`
- **Verification**:
  - This seam **produces** `C-01` and `C-02`.
  - Producer-seam readiness means the canonical docs and seam-local plan make selection order, inventory roots and filename rules, deny-by-default allowlisting, trusted-input boundaries, and auth precedence concrete enough that `SEAM-2` can consume `THR-01` without inferring missing semantics from the current `cli:codex` implementation.
  - Verification for this seam centers on:
    - one explicit ordered selection path from config/policy/inventory inputs to allowed backend id
    - one explicit precedence rule between allowlisted env material and allowlisted host credential files
    - distinct invalid-integration, dependency-unavailable, and policy-denial outcomes
    - shell-side tests and docs remaining aligned with the canonical `docs/contracts/` refs
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
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01`: `identified` -> `published`
- **Likely downstream stale triggers**:
  - backend inventory roots or filename/id invariants change
  - auth precedence or no-host-fallback rules change
  - failure taxonomy shifts between invalid integration, dependency unavailable, and policy denial
  - supporting ADR-0046 docs drift away from canonical `docs/contracts/` truth
- **Expected closeout evidence**:
  - landed canonical contract updates in `docs/contracts/substrate-gateway-backend-adapter-selection.md` and `docs/contracts/substrate-gateway-policy-evaluation.md`
  - supporting ADR-0046 doc alignment in `contract.md`, `policy-spec.md`, and `env-vars-spec.md`
  - landed shell evidence for the selection/policy gate in `crates/shell/src/builtins/world_gateway.rs`
  - recorded remediation disposition for `REM-001` and `REM-002`

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
  - The seam stays `status: decomposed` until the seam-local review is signed off, the owned contracts are concrete enough to clear `REM-001` and `REM-002`, and revalidation confirms the shell/code basis still matches the plan.
- **Slicing strategy**:
  - contract-first, then dependency-first implementation, then conformance, then explicit seam exit

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
