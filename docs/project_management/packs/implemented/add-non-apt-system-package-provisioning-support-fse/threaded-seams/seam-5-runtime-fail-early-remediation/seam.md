---
seam_id: SEAM-5
seam_slug: runtime-fail-early-remediation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-5-runtime-fail-early-remediation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
  - ../../governance/seam-1-closeout.md
  - ../../governance/seam-3-closeout.md
  - ../../governance/seam-4-closeout.md
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
# SEAM-5 - Runtime fail-early and remediation

## Seam Brief (Restated)

- **Goal / value**:
  - Keep runtime system-package handling read-only and deterministic while extending fail-early behavior and remediation to pacman-backed items alongside APT-backed items.
- **Type**: platform
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
- **Touch surface**:
  - Likely downstream code surfaces once execution begins:
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
    - `crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`
    - `crates/shell/tests/world_deps_apt_install_wdp5.rs`
    - `docs/reference/world/deps/README.md`
    - `docs/internals/world/deps.md`
- **Verification**:
  - Because this seam **produces** `C-05`, pre-exec verification must prove the runtime fail-early and remediation contract is concrete enough for downstream validation without waiting on post-exec publication.
  - Review should try to falsify:
    - whether any runtime path can still mutate the world package manager
    - whether explicit-item installs can still be poisoned by unrelated enabled system-package items
    - whether manager-aware missing-requirement rendering can drift across APT-backed and pacman-backed items
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-3-closeout.md`
    - `../../governance/seam-4-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-03`
    - `THR-04`
  - Stale triggers:
    - `C-03` changes pacman-backed schema/view semantics used for runtime derivation
    - `C-04` changes normalized requirement-set or manager-aware rendering assumptions
    - runtime docs or tests drift back toward mutation-at-runtime semantics
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-4` has now published `THR-04` with a passed seam-exit gate
  - Downstream blocked seams:
    - `SEAM-6`
  - Contracts produced:
    - `C-05`
  - Contracts consumed:
    - `C-01`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `C-05` / `THR-05` is the promotion input for the terminal validation and reconciliation seam. The seam-exit gate makes the runtime read-only and remediation posture a closeout-backed fact instead of an implicit reading of tests.
- **Expected contracts to publish**:
  - `C-05`
- **Expected threads to publish / advance**:
  - `THR-05`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - Any change to runtime in-scope rules, read-only probe families, remediation wording, or manager-aware ordering/rendering
- **Expected closeout evidence**:
  - Published `C-05` contract artifact location(s)
  - Runtime fail-early evidence across explicit-item, sync, and mixed-manager in-scope sets
  - Thread-state update record for `THR-05`
  - Recorded review-surface deltas for any runtime/doc drift discovered during landing

## Slice index

- `S1` -> `slice-1-c-05-runtime-fail-early-contract-definition.md`
- `S2` -> `slice-2-runtime-fail-early-and-remediation.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-5-closeout.md`
