---
seam_id: SEAM-2
seam_slug: world-manager-probe-support-gate
type: platform
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
  - THR-01
  stale_triggers:
  - C-01 changes manager-selection semantics, supported families, or unsupported-backend
    wording
  - world_enable or world-agent shared-file changes alter where the in-world probe
    runs
  - platform parity assumptions change and require different support-gate outcomes
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
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-2 - World-manager probe and support gate

- **Goal / value**:
  - Turn the accepted manager-selection contract into one deterministic in-world probe and one fail-closed support gate that every provisioning path can trust.
- **Scope**
  - In:
    - exact probe inputs: `/etc/os-release` `ID`, `/etc/os-release` `ID_LIKE`, and in-world `command -v pacman`
    - normalization of Debian-family and Arch-family identifiers
    - contradiction handling between world identity and package-manager executable presence
    - supported vs unsupported provisioning outcomes across Linux host-native, macOS Lima guest, and Windows WSL
    - invariant that routing decisions happen inside the world, not on the host
  - Out:
    - schema validation for pacman-backed packages
    - requirement-set normalization and mixed-manager rejection
    - exact pacman execution command
    - runtime read-only presence probes and remediation wording
    - validation evidence and shared-doc reconciliation
- **Primary interfaces**
  - Inputs:
    - `C-01` from `SEAM-1`
    - accepted probe-precedence decision DR-0002
    - existing world-enable / dispatch / world-agent touch surfaces
  - Outputs:
    - `C-02` deterministic world-manager probe and support-gate contract
    - closeout-backed support-gate evidence for provisioning and parity validation
- **Key invariants / rules**:
  - `/etc/os-release` is authoritative for family selection; package-manager presence only confirms support
  - host PATH, host installer detection, host package-manager state, and host-side `PKG_MANAGER` vocabulary are not routing inputs
  - if `/etc/os-release` is unreadable, unmapped, or contradicted by package-manager presence, the seam fails closed with exit `4`
  - Linux host-native and Windows backends remain unsupported for provisioning in v1
  - dry-run still performs the in-world probe while performing no mutation
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` publishing `C-01`
  - Transitive blockers:
    - overlapping host-installer detection work can stale `/etc/os-release` vocabulary assumptions if guest routing starts reusing host logic
  - Direct consumers:
    - `SEAM-4`
    - `SEAM-6`
  - Derived consumers:
    - operators reading backend diagnostics
    - future backend-capability reporting work
- **Touch surface**:
  - Primary planning surface:
    - `slices/NASP0/NASP0-spec.md`
  - Likely downstream code surfaces once seam-local planning begins:
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `crates/world-agent/src/service.rs`
    - `crates/shell/tests/world_enable.rs`
- **Verification**:
  - Because this seam **consumes** `C-01`, verification should prove the producer contract still matches the probe boundary before implementation begins.
  - The first seam-local review should try to falsify:
    - whether any host-side manager detection still leaks into world provisioning
    - whether contradiction handling can still silently fall back to the wrong manager
    - whether unsupported backend messaging and exit `4` posture drift across shell, dispatch, and world-agent layers
  - A passing pre-exec posture should leave `SEAM-4` able to plan against one stable support-gate outcome model.
- **Risks / unknowns**:
  - Risk:
    - adjacent work on `world_enable` or `world-agent` can move helper/staging boundaries and stale this seam's touch surface before decomposition.
  - De-risk plan:
    - revalidate the shared files at seam-local review time and keep probe changes orthogonal to any staging/tracing work.
  - Risk:
    - host-installer detection vocabulary from ADR-0031 could be mistaken for guest-provisioning routing vocabulary.
  - De-risk plan:
    - make the in-world-only rule explicit in the first seam-local review bundle and reject any host-derived routing inputs.
- **Rollout / safety**:
  - This seam should only make supported-vs-unsupported manager selection and gating explicit; it must not add new mutation paths by itself.
  - Every failure mode here should remain fail-closed and non-mutating.
- **Downstream decomposition context**:
  - This seam is `next` because the source accepted delivery order starts with `NASP0`, and provisioning routing cannot safely decompose until the support gate is fixed.
  - The most important threads are `THR-01` and `THR-02`.
  - The first seam-local review should focus on `/etc/os-release` normalization, contradiction handling, shared-file touch boundaries, and exact unsupported-backend outcomes.
  - Source-plan lineage: `NASP-PWS-os_probe` and `NASP0`.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-02`
  - Threads likely to advance:
    - `THR-02` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the in-world probe node and support-gate branches in the pack workflow diagram
    - the platform posture diagram if unsupported wording becomes more concrete
  - Downstream seams most likely to require revalidation:
    - `SEAM-4`
    - `SEAM-6`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
