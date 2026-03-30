---
seam_id: SEAM-5
seam_slug: runtime-fail-early-remediation
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
  - THR-03
  - THR-04
  stale_triggers:
  - C-03 changes pacman-backed schema/view semantics used for runtime derivation
  - C-04 changes normalized requirement-set or manager-aware rendering assumptions
  - runtime docs or tests drift back toward mutation-at-runtime semantics
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

# SEAM-5 - Runtime fail-early and remediation

- **Goal / value**:
  - Keep runtime system-package handling read-only and deterministic while extending fail-early behavior and remediation to pacman-backed items alongside APT-backed items.
- **Scope**
  - In:
    - runtime in-scope rules for `deps current sync`, `deps current sync --all`, and `deps current install <ITEM...>`
    - read-only `dpkg-query` and `pacman -Q` presence probes
    - explicit-item scope rule for `current install <ITEM...>`
    - manager-aware missing-requirement rendering and backend-specific guidance
    - dry-run and verbose behavior for runtime fail-early paths
    - error-path posture for invalid schema input, read-only probe connectivity failures, and unsatisfied system-package requirements
  - Out:
    - provisioning-time probe/support gate
    - pacman schema definition and inventory rendering
    - provisioning-time pacman mutation and mixed-manager execution behavior
    - smoke/manual evidence and shared-doc reconciliation landing
- **Primary interfaces**
  - Inputs:
    - `C-01` from `SEAM-1`
    - `C-03` from `SEAM-3`
    - `C-04` from `SEAM-4`
  - Outputs:
    - `C-05` runtime fail-early and remediation contract
    - runtime evidence consumed by the validation/conformance seam
- **Key invariants / rules**:
  - runtime `substrate world deps current sync|install` must not execute mutating `apt`, `apt-get`, `dpkg`, or `pacman`
  - runtime may use only read-only `dpkg-query` and `pacman -Q` probes
  - `deps current install <ITEM...>` scopes system-package fail-early only to the explicit expanded item set
  - a runtime in-scope set may contain both APT-backed and pacman-backed items; runtime exits `4` only when one or more derived requirements are unsatisfied
  - missing `dpkg-query` or `pacman -Q` counts as unsatisfied, not as a reason to fall back to mutation
  - remediation must include the exact command `substrate world enable --provision-deps`
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` publishing `C-01`
    - `SEAM-3` publishing `C-03`
    - `SEAM-4` publishing `C-04`
  - Transitive blockers:
    - older runtime docs and tests still encode APT-only or mutation-at-runtime assumptions that can stale this seam's basis
  - Direct consumers:
    - `SEAM-6`
  - Derived consumers:
    - runtime operators
    - support/docs maintainers
- **Touch surface**:
  - Primary planning surface:
    - `slices/NASP3/NASP3-spec.md`
  - Likely downstream code surfaces once seam-local planning begins:
    - `crates/shell/src/execution/cli.rs`
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
    - `crates/shell/tests/world_deps_apt_install_wdp5.rs`
    - `docs/reference/world/deps/README.md`
    - `docs/internals/world/deps.md`
- **Verification**:
  - Because this seam **consumes** upstream contracts, verification may depend on accepted upstream evidence for the pacman schema contract and provisioning normalization contract.
  - The first seam-local review should try to falsify:
    - whether any runtime path can still mutate the world package manager
    - whether explicit-item installs can still be poisoned by unrelated enabled system-package items
    - whether missing-requirement rendering can still become unstable across APT-backed and pacman-backed items
  - A passing pre-exec posture should leave the validation/conformance seam able to lock evidence against one accepted runtime story.
- **Risks / unknowns**:
  - Risk:
    - older tests and docs still encode the obsolete `apt first, script second` runtime plan and can drag the seam back toward mutation semantics.
  - De-risk plan:
    - make those stale assumptions explicit falsification targets during seam-local review and carry the drift into `SEAM-6` reconciliation if it still exists.
  - Risk:
    - changes in provisioning normalization or schema invalid-state handling can silently shift runtime fail-early behavior.
  - De-risk plan:
    - require revalidation of `THR-03` and `THR-04` before decomposition.
- **Rollout / safety**:
  - This seam is a safety seam as much as a runtime UX seam. Its main job is to keep system-package mutation out of runtime while still giving operators deterministic next steps.
  - Backend-specific guidance for Linux host-native and Windows is load-bearing because it prevents host-mutation misinterpretation.
- **Downstream decomposition context**:
  - This seam is `future` because it sits after provisioning routing in the accepted source order and the default horizon does not deep-plan beyond the next seam.
  - The most important threads are `THR-03`, `THR-04`, and `THR-05`.
  - The first seam-local review should focus on explicit-item scope, read-only probe families, manager-aware ordering, and the exact remediation wording path through shell/runtime surfaces.
  - Source-plan lineage: `NASP-PWS-runtime_fail_early` and `NASP3`.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-05`
  - Threads likely to advance:
    - `THR-05` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the runtime lane in the workflow diagram
    - the runtime-to-remediation edge in the service/data-flow diagram
  - Downstream seams most likely to require revalidation:
    - `SEAM-6`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
