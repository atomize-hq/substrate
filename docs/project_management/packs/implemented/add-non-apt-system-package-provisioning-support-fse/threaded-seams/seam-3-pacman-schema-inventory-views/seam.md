---
seam_id: SEAM-3
seam_slug: pacman-schema-inventory-views
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-3-pacman-schema-inventory-views.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
  - ../../governance/seam-1-closeout.md
  - ../../governance/seam-2-closeout.md
  required_threads:
  - THR-01
  stale_triggers:
  - C-01 changes pacman v1 scope or manager-aware inventory authority boundaries
  - upstream bundles contract changes merge or view semantics in ways that affect pacman-backed items
  - inventory examples or validation rules drift toward translation-layer or runnable pacman behavior
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
# SEAM-3 - Pacman schema and inventory views

## Seam Brief (Restated)

- **Goal / value**:
  - Extend world-deps authoring and inventory views with additive pacman schema support while keeping inventory compatibility, validation determinism, and non-runnable v1 scope explicit.
- **Type**: integration
- **Scope**
  - In:
    - `install.method` enum extension to include `pacman`
    - `install.pacman` field shape, author-order preservation, and invalid-state rules
    - v1 pacman constraints: non-runnable packages, no wrappers, no widened present semantics
    - mutual exclusion with `install.apt`, `install.script*`, and `install.manual_instructions`
    - inventory list/show JSON and YAML rendering obligations for pacman-backed packages
  - Out:
    - world-manager probe and support gating
    - provisioning-time requirement normalization or pacman execution
    - runtime read-only probes and remediation wording
    - smoke/manual evidence and shared-doc reconciliation landing
- **Touch surface**:
  - Primary planning surfaces:
    - `world-deps-pacman-schema-spec.md`
    - `slices/NASP1/NASP1-spec.md`
  - Likely downstream code surfaces once execution begins:
    - `crates/shell/src/builtins/world_deps/inventory.rs`
    - `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
    - `crates/shell/tests/world_deps_inventory_views.rs`
    - shared bundles-contract wording that must defer on pacman-specific schema truth
- **Verification**:
  - Because this seam **produces** `C-03`, pre-exec verification must prove the additive pacman schema and inventory-view contract is concrete enough for downstream implementation without waiting on post-exec publication.
  - Review should try to falsify:
    - whether pacman support still implies a translation layer
    - whether any view surface can still erase or rewrite the `pacman` method
    - whether runnable pacman behavior can still sneak in through wrappers, entrypoints, or probe semantics
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`
  - Required threads (inbound): `THR-01` (published and revalidated for downstream consumers)
  - Stale triggers:
    - `C-01` changes pacman v1 scope or manager-aware inventory authority boundaries
    - upstream bundles contract changes merge or view semantics in ways that affect pacman-backed items
    - inventory examples or validation rules drift toward translation-layer or runnable pacman behavior
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1` published `C-01`, and `SEAM-2` closed without changing schema ownership
  - Downstream blocked seams:
    - `SEAM-4` (consumes `C-03` / `THR-03`)
    - `SEAM-5` (consumes `C-03` / `THR-03`)
    - `SEAM-6` (consumes `C-03` / `THR-03`)
  - Contracts produced:
    - `C-03` (owned by `SEAM-3`, carried on `THR-03`)
  - Contracts consumed:
    - `C-01` (owned by `SEAM-1`, carried on `THR-01`)

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `C-03` / `THR-03` is the promotion input for provisioning routing (`SEAM-4`), runtime fail-early handling (`SEAM-5`), and validation reconciliation (`SEAM-6`). The seam-exit gate makes the additive pacman schema and inventory-view rules a closeout-backed fact rather than an implicit reading of tests.
- **Expected contracts to publish**:
  - `C-03`
- **Expected threads to publish / advance**:
  - `THR-03`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - Any change to:
    - `install.method` vocabulary or mutual-exclusion rules
    - `install.pacman` shape or author-order preservation
    - non-runnable v1 pacman constraints
    - inventory list/show rendering for pacman-backed items
- **Expected closeout evidence**:
  - Published `C-03` schema/view artifact location(s)
  - Inventory validation and view evidence for pacman-backed items
  - Thread-state update record for `THR-03`
  - Recorded review-surface deltas for any touch-surface drift discovered during landing

## Slice index

- `S1` -> `slice-1-c-03-schema-contract-definition.md`
- `S2` -> `slice-2-inventory-validation-and-view-rendering.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
