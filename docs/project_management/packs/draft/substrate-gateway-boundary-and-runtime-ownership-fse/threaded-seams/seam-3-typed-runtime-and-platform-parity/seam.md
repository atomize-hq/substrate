---
seam_id: SEAM-3
seam_slug: typed-runtime-and-platform-parity
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-typed-runtime-and-platform-parity.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - typed world-agent endpoint ownership changes
    - published `status --json` envelope or `client_wiring.*` semantics change
    - published fail-closed policy or trust-boundary rules change
    - shell-side exec probing is reintroduced
    - Linux/macOS/Windows parity guarantees or allowed divergence list changes
    - provisioning is pulled back into this pack
gates:
  pre_exec:
    review: passed
    contract: failed
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
# SEAM-3 - Typed runtime and platform parity

## Seam Brief (Restated)

- **Goal / value**:
  - Define the typed world-agent lifecycle/status path and platform guarantees that keep gateway lifecycle behavior stable across Linux, macOS, and Windows.
  - Prevent raw exec probing, backend-private quirks, or provisioning assumptions from becoming the operator contract.
- **Type**: platform
- **Scope**
  - In:
    - typed world-agent lifecycle/status ownership
    - shell builtin consumption path
    - shared API type/client alignment
    - Linux/macOS/Windows parity guarantees for placement, lifecycle visibility, and status semantics
    - allowed divergence list and required validation evidence
  - Out:
    - the operator command family definition
    - the owned `status --json` field list and policy decision tables
    - provisioning-script changes or backend warm-flow edits
    - cross-doc/manual-playbook lock-in and checkpoint wiring
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `crates/world-agent/src/handlers.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/agent-api-client/src/lib.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `docs/WORLD.md`
- **Verification**:
  - This seam consumes upstream contracts `C-01`, `C-02`, and `C-03`; those inputs are now published by `../../governance/seam-1-closeout.md` and `../../governance/seam-2-closeout.md`.
  - This seam produces owned contract `C-04`. The current blocker is that the feature-local parity spec and durable runtime/parity contract baseline do not yet exist, so the seam remains `decomposed` rather than `exec-ready`.
  - Later seam-local verification should prove:
    - the typed world-agent path is authoritative for lifecycle/status operations
    - shell and shared clients consume the same runtime contract
    - Linux, macOS, and Windows guarantees are explicit and testable
    - provisioning remains correctly deferred outside this pack
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - none after `SEAM-2` closeout published the schema and policy threads
  - Downstream blocked seams:
    - `SEAM-4`
  - Contracts produced:
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `SEAM-4` can only lock docs and quality-gate conformance once the typed runtime/parity contract is published and backed by explicit divergence and validation evidence.
- **Expected contracts to publish**:
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-04`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - typed lifecycle/status contract shape changes
  - shell/client consumption path changes
  - allowed divergence list changes
  - Linux/macOS/Windows parity evidence requirement changes
- **Expected closeout evidence**:
  - landed feature-local parity-spec publication
  - landed durable runtime/parity contract publication
  - landed world-agent and shared client path evidence
  - explicit parity expectations and allowed-divergence accounting

## Slice index

- `S00` -> `slice-00-runtime-parity-contract-definition.md`
- `S1` -> `slice-1-typed-lifecycle-status-api-boundary.md`
- `S2` -> `slice-2-shell-consumption-and-platform-parity-evidence.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
