---
seam_id: SEAM-1
seam_slug: operator-boundary-and-command-contract
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-operator-boundary-and-command-contract.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - command spelling or command ordering changes for `substrate world gateway sync|status|restart`
    - human-readable `status` or stable env export wording diverges from `status --json` as the authoritative wiring surface
    - absent-state or exit-code wording changes collapse invalid integration, dependency-unavailable, and policy-denial outcomes
    - the Substrate versus `substrate-gateway` ownership split drifts in ADR-0040, operator docs, or archived gateway-planning references
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
# SEAM-1 - Operator boundary and command contract

## Seam Brief (Restated)

- **Goal / value**: publish one authoritative operator contract for gateway lifecycle, wiring discovery, absent-state behavior, exit taxonomy, and the durable ownership split between Substrate and `substrate-gateway`.
- **Type**: integration
- **Scope**
  - In:
    - define `C-01` tightly enough that `SEAM-2`, `SEAM-3`, and `SEAM-4` can consume one operator boundary without re-reading archived drafts
    - align the `sync|status|restart` command family, the `status --json` authority rule, and the stable non-secret wiring env names
    - make the `0|2|3|4|5` gateway lifecycle/status boundaries explicit
    - publish the operator-facing ownership split and the rule that gateway-local config, admin, and persistence are not required Substrate contract surfaces
  - Out:
    - `status --json` field-by-field schema and `client_wiring.*` shape (`SEAM-2`)
    - fail-closed policy-evaluation decision tables and trust-boundary rules (`SEAM-2`)
    - typed world-service lifecycle/status transport and parity guarantees (`SEAM-3`)
    - final docs-validation and quality-gate lock-in (`SEAM-4`)
- **Touch surface**:
  - Primary planning surfaces:
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/contract.md`
    - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
    - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/workstream_triage.md`
  - Likely downstream code and operator-doc surfaces once execution begins:
    - `crates/shell/src/execution/cli.rs`
    - `crates/shell/src/builtins/mod.rs`
    - `crates/shell/src/builtins/world_gateway.rs`
    - `crates/shell/tests/world_gateway.rs`
    - `docs/USAGE.md`
- **Verification**:
  - `C-01` is concrete enough for execution because `S00` names the exact command family, status entrypoint rule, stable env semantics, exit taxonomy, ownership table, and the verification surfaces that later slices must update together.
  - The producer seam does not require its own final accepted contract artifact as a pre-exec input; publication and accepted evidence are handled through `S99` and closeout.
- **Canonical contract refs**:
  - `docs/contracts/substrate-gateway-operator-contract.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers: none; this is the first active seam and the producer for `C-01`
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - Contracts produced: `C-01`
  - Contracts consumed: none
  - Canonical contract refs: `docs/contracts/substrate-gateway-operator-contract.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `SEAM-1` is the producer seam for the first critical-path handoff. `SEAM-2` cannot promote on inferred command wording, inferred exit semantics, or archived gateway docs.
- **Expected contracts to publish**: `C-01`
- **Expected threads to publish / advance**: `THR-01` from `defined` to `published`
- **Likely downstream stale triggers**:
  - any command-family spelling or ordering change
  - any change to the rule that `status --json` is the authoritative machine-readable wiring surface
  - any change to stable env semantics, absent-state wording, or exit-code boundaries
  - any change to the Substrate versus `substrate-gateway` ownership table
- **Expected closeout evidence**:
  - landed feature-local operator contract publication
  - landed CLI/parser/builtin/test surfaces proving the command family and absent-state behavior
  - landed operator-doc publication for stable env semantics and exit taxonomy
  - explicit `THR-01` publication accounting

## Slice index

- `S00` -> `slice-00-operator-contract-definition.md`
- `S1` -> `slice-1-command-family-and-status-entrypoint.md`
- `S2` -> `slice-2-exit-taxonomy-and-wiring-semantics.md`
- `S3` -> `slice-3-ownership-and-archived-drift-normalization.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
