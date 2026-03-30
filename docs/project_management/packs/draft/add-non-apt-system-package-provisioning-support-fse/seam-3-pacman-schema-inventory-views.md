---
seam_id: SEAM-3
seam_slug: pacman-schema-inventory-views
type: integration
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
  - governance/seam-1-closeout.md
  - governance/seam-2-closeout.md
  required_threads:
  - THR-01
  stale_triggers:
  - C-01 changes pacman v1 scope or manager-aware inventory authority boundaries
  - upstream bundles contract changes merge/view semantics in ways that affect pacman-backed
    items
  - inventory examples or validation rules drift toward translation-layer or runnable
    pacman behavior
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
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-3 - Pacman schema and inventory views

- **Goal / value**:
  - Extend world-deps authoring and view surfaces with additive pacman schema support while keeping inventory compatibility, validation determinism, and non-runnable v1 scope explicit.
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
- **Primary interfaces**
  - Inputs:
    - `C-01` from `SEAM-1`
    - upstream bundles contract for unchanged merge and enabled-set semantics
    - accepted schema/runnable-scope decisions DR-0001 and DR-0004
  - Outputs:
    - `C-03` additive pacman schema and inventory-view contract
    - inventory-authoring basis for provisioning and runtime seams
- **Key invariants / rules**:
  - schema change is additive-only; package files remain on `version: 1`
  - authored `install.pacman` list order is preserved in stored definitions and resolved views
  - the schema layer does not de-duplicate or sort `install.pacman`; later provisioning normalization owns that work
  - pacman-backed packages remain non-runnable prerequisites in v1
  - inventory views must show pacman-backed items as `pacman`, not collapse them into `apt`, `script`, or `manual`
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` publishing `C-01`
  - Transitive blockers:
    - upstream bundles contract wording can stale the schema authority story if it keeps hard-coding `apt | script | manual`
  - Direct consumers:
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - Derived consumers:
    - inventory authors
    - inventory validation and list/show tests
- **Touch surface**:
  - Primary planning surfaces:
    - `world-deps-pacman-schema-spec.md`
    - `slices/NASP1/NASP1-spec.md`
  - Likely downstream code surfaces once seam-local planning begins:
    - `crates/shell/src/builtins/world_deps/inventory.rs`
    - `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
    - `crates/shell/tests/world_deps_inventory_views.rs`
    - upstream bundles-contract wording that must defer on pacman-specific schema truth
- **Verification**:
  - Because this seam **produces** `C-03`, verification should prove the pacman schema and view contract is concrete enough for downstream planning rather than requiring final implementation evidence already to exist.
  - The first seam-local review should try to falsify:
    - whether pacman support still implies a translation layer
    - whether any view surface can still erase or rewrite the `pacman` method
    - whether runnable pacman behavior can still sneak in through wrappers, entrypoints, or probe semantics
  - A passing pre-exec posture should leave provisioning and runtime seams able to consume one stable pacman schema contract.
- **Risks / unknowns**:
  - Risk:
    - older upstream docs and contracts can still hard-code the older method enum and create second-truth authoring guidance.
  - De-risk plan:
    - keep that drift visible for `SEAM-6` reconciliation while forcing schema authority into `C-03`.
  - Risk:
    - the absence of built-in pacman catalog entries could be mistaken for unsupported authored pacman inventory.
  - De-risk plan:
    - keep the authored-vs-built-in boundary explicit in seam-local review and in the eventual reconciliation surfaces.
- **Rollout / safety**:
  - This seam must remain additive and validation-driven; it should not widen into executable provisioning behavior or runtime mutation semantics.
  - Invalid pacman schema shapes should stay taxonomy-aligned as exit `2` behavior in downstream consumers.
- **Downstream decomposition context**:
  - This seam is now `active` because `SEAM-2` closed with a passed seam-exit gate and `THR-01` remains current for additive schema work.
  - The most important threads are `THR-01` and `THR-03`.
  - The first seam-local review should focus on schema authority, invalid-state coverage, list/show rendering, and the exact boundary between schema order and provisioning normalization.
  - Source-plan lineage: `NASP-PWS-schema_inventory`, `world-deps-pacman-schema-spec.md`, and `NASP1`.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-03`
  - Threads likely to advance:
    - `THR-03` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the inventory-to-provisioning edge in the service/data-flow diagram
    - the touch-surface map around validation/tests vs execution paths
  - Downstream seams most likely to require revalidation:
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
