---
seam_id: SEAM-1
seam_slug: persisted-platform-metadata-contract
type: integration
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - upstream detection contract changes selected-manager or pkg_manager.source vocabulary after pre-exec revalidation
    - os_release sentinel or field-path rules change before closeout publishes C-01 and C-02
    - ADR-0032 or related docs reintroduce a competing feature-directory authority before closeout
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
  planned_location: reserved_final_slice
  status: passed
open_remediations: []
---

# SEAM-1 - Persisted platform metadata contract

- **Goal / value**:
  - Freeze the exact persisted `install_state.json` platform contract so downstream work can implement and validate one canonical file shape instead of inventing alternate field layouts or path semantics.
- **Scope**
  - In:
    - Exact persisted fields: `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`
    - `schema_version = 1` invariants
    - Additive compatibility and preservation of `host_state.group`, `host_state.linger`, and unknown keys
    - Canonical file-path rule and operator-facing alias relationship
    - Verbatim-copy ownership boundary to the upstream detection contract
    - Missing os-release sentinel handling as persisted contract input
  - Out:
    - Recomputing distro or package-manager selection locally
    - Runtime branch selection for hosted or dev installer write/no-write behavior
    - Temp-file replace mechanics beyond naming the contract requirement
    - Smoke-harness assertions, checkpoint evidence, and documentation hardening
    - Uninstaller cleanup alignment
- **Primary interfaces**
  - Inputs:
    - Normalized distro and package-manager outputs produced by the external detection contract
    - Existing `install_state.json` content when present
    - Source-pack decisions DR-0001 through DR-0005
  - Outputs:
    - `C-01` persisted platform schema contract
    - `C-02` canonical path and authority-boundary contract
    - Closeout-backed inputs for `SEAM-2` and `SEAM-3`
- **Key invariants / rules**:
  - `schema_version` remains integer `1`
  - `install_state.json` remains the only persisted metadata file touched by this feature
  - `pkg_manager.selected` and `pkg_manager.source` are copied verbatim from the external detection contract
  - Missing or unreadable os-release input persists the literal `unknown` sentinel rather than inventing fallback distro values
  - Existing `host_state.group`, `host_state.linger`, and unknown keys survive rewrite unchanged
  - Linux-only behavior change; non-Linux platform metadata writes stay out of scope
- **Dependencies**
  - Direct blockers:
    - External detection contract remains authoritative for selected-manager vocabulary, source vocabulary, and normalized distro outputs
    - The accepted `contract.md` authority override and `DR-0005` now govern the canonical feature-directory path; any new competing source-path reference before closeout is a stale trigger rather than a live blocker
  - Transitive blockers:
    - Adjacent installer and documentation packs that touch the same shared files can make current assumptions stale
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - Future guidance consumers that prefer persisted metadata when available
    - Support and operator documentation that must describe the same field/path contract
- **Touch surface**:
  - Primary planning surfaces:
    - `contract.md`
    - `decision_register.md`
    - `install-state-schema-spec.md`
  - Likely downstream code surfaces once seam-local planning begins:
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/dev-install-substrate.sh`
- **Verification**:
  - Because this seam **produces** owned contracts, verification should prove the contract is concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - The current pre-exec review revalidated:
    - whether any alternate field nesting or alias is still possible
    - whether any local re-derivation of selected manager or source remains hidden in downstream code paths
    - whether path semantics still imply more than one canonical metadata location
  - The resulting pre-exec posture leaves `SEAM-2` able to plan against one stable schema and one stable path rule.
- **Risks / unknowns**:
  - Risk:
    - ADR-0032 still contains stale feature-directory links even though `contract.md` and `DR-0005` now establish the single authoritative override for planning.
  - De-risk plan:
    - Keep the override explicit in seam-exit and closeout evidence; treat any new competing path reference as a downstream stale trigger.
  - Risk:
    - Upstream detection vocabulary or sentinel semantics could change after extraction.
  - De-risk plan:
    - Revalidate this seam against the latest external detection contract before closeout-backed downstream promotion.
  - Risk:
    - Existing runtime code may still assume event-only writes or alternate field handling.
  - De-risk plan:
    - Keep runtime branch semantics out of this seam and force `SEAM-2` to consume only the closeout-backed contract.
- **Rollout / safety**:
  - This seam should land as additive, Linux-only, and non-failing for metadata persistence concerns.
  - It must not broaden scope to new CLI/env/log surfaces while clarifying the contract.
- **Downstream decomposition context**:
  - This seam is `active` because the source pack made field ownership and schema authority the first critical-path slice, and every other seam depends on it.
  - The most important threads are `THR-01` and `THR-03`.
  - The first seam-local review should focus on source-of-truth boundaries, field naming, path equivalence, additive compatibility, and sentinel semantics.
  - Source-plan lineage: primarily `PDLDPM0` plus the contract, schema, and decision surfaces that made the old pack execution-ready.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`
    - `C-02`
  - Threads likely to advance:
    - `THR-01` from `defined` to `published`
    - `THR-03` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the authority-boundary map between upstream detection, writer code, and operator docs
    - the compatibility/rewrite diagram once real implementation evidence exists
    - whether ADR-0032 still carries stale path links even though the accepted pack contract remains authoritative
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
