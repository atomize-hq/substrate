---
seam_id: SEAM-2
seam_slug: shim-doctor-disabled-aware-reporting
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-shim-doctor-disabled-aware-reporting.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - C-01 or THR-01 changes after SEAM-1 closeout
    - hidden world backend or world-deps probe paths reappear before landing
    - exact disabled-mode copy or omission rules change without synchronized test/doc updates
    - json-mode or attribution work reshapes shim payloads or field paths before closeout
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
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-2 - Shim doctor disabled-aware reporting

## Seam Brief (Restated)

- **Goal / value**: make `substrate shim doctor` truthful for host-only installs by short-circuiting disabled-mode probes and publishing one canonical disabled-aware JSON/text contract.
- **Type**: capability
- **Scope**
  - In:
    - gate shim-doctor world and world-deps collection on the published `effective_world_enabled` handoff from `SEAM-1`
    - short-circuit all world backend and world-deps probes when disabled
    - publish `.world.status` and `.world_deps.status` plus the disabled-mode omission rules
    - render the exact disabled-mode text lines and suppress `Error:` for disabled/skipped states
  - Out:
    - health summary derivation and docs alignment
    - cross-platform smoke orchestration beyond seam-local proof
    - why-disabled attribution beyond the base status contract
- **Touch surface**: `crates/shell/src/builtins/shim_doctor/report.rs`, `crates/shell/src/builtins/shim_doctor/output.rs`, `crates/shell/tests/shim_doctor.rs`.
- **Verification**:
  - disabled mode must not spawn `substrate world doctor --json`
  - disabled mode must not compute applied world-deps state
  - disabled JSON must publish `C-02` / `C-03` enum values and omit forbidden legacy fields
  - disabled text must publish the exact `C-04` operator lines while enabled-mode failures remain visible
- **Basis posture**:
  - Currentness: current (revalidated against `governance/seam-1-closeout.md` and the landed `THR-01` helper path)
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads: `THR-01`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers resolved: `THR-01` is published and revalidated for this seam
  - Downstream blocked seams: `SEAM-3`, `SEAM-4`
  - Contracts produced: `C-02`, `C-03`, `C-04`
  - Contracts consumed: `C-01`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it publishes the disabled shim status contracts that every downstream health and conformance seam must consume without local reinvention.
- **Expected contracts to publish**: `C-02`, `C-03`, `C-04`
- **Expected threads to publish / advance**: `THR-02`, `THR-03`, `THR-04`
- **Likely downstream stale triggers**: shim payload field-path drift, disabled-mode omission rule drift, copy-line drift, or any reintroduced disabled-mode probe path.
- **Expected closeout evidence**: landed disabled gating in `shim_doctor/report.rs`, exact text/JSON tests, and a recorded no-probe statement for the disabled path.

## Slice index

- `S1` -> `slice-1-contract-definition-c-02-c-03-c-04.md`
- `S2` -> `slice-2-disabled-path-rendering-and-tests.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
