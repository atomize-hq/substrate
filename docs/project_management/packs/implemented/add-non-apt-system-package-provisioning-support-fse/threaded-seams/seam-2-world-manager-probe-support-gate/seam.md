---
seam_id: SEAM-2
seam_slug: world-manager-probe-support-gate
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-world-manager-probe-support-gate.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
  - ../../governance/seam-1-closeout.md
  required_threads:
  - THR-01
  stale_triggers:
  - C-01 changes manager-selection semantics, supported families, or unsupported-backend
    wording
  - world_enable or world-service shared-file changes alter where the in-world probe
    runs
  - platform parity assumptions change and require different support-gate outcomes
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
# SEAM-2 - World-manager probe and support gate

## Seam Brief (Restated)

- **Goal / value**:
  - Turn the accepted manager-selection contract (`C-01`) into one deterministic in-world probe plus one fail-closed support gate that every provisioning path can trust.
- **Type**: platform
- **Scope**
  - In:
    - probe inputs: `/etc/os-release` (`ID`, `ID_LIKE`) and in-world `command -v pacman`
    - normalization of Debian-family and Arch-family identifiers
    - contradiction handling between world identity and package-manager executable presence
    - supported vs unsupported provisioning outcomes across Linux host-native, macOS Lima guest, and Windows WSL
    - invariant that routing decisions happen inside the world, not on the host
  - Out:
    - schema validation for pacman-backed packages (`C-03`, `SEAM-3`)
    - requirement-set normalization and mixed-manager rejection (`C-04`, `SEAM-4`)
    - exact pacman execution command (`SEAM-4`)
    - runtime read-only presence probes and remediation wording (`C-05`, `SEAM-5`)
    - validation evidence and shared-doc reconciliation (`SEAM-6`)
- **Touch surface**:
  - Primary planning surface:
    - `slices/NASP0/NASP0-spec.md`
  - Likely downstream code surfaces once execution begins:
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `crates/world-service/src/service.rs`
    - `crates/shell/tests/world_enable.rs`
- **Verification**:
  - Because this seam **consumes** `C-01` (`THR-01`), pre-exec verification must revalidate that:
    - no host-side manager detection leaks into the provisioning flow
    - contradiction handling cannot silently fall back to the wrong manager
    - unsupported backend posture and exit `4` remain fail-closed and consistent across shell, dispatch, and world-service layers
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads (inbound): `THR-01` (published and revalidated)
  - Stale triggers:
    - `C-01` changes manager-selection semantics, supported families, or unsupported-backend wording
    - `world_enable` or `world-service` shared-file changes alter where the in-world probe runs
    - platform parity assumptions change and require different support-gate outcomes
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1` published `C-01` and `SEAM-2` revalidated `THR-01` against the pack-root contract and decision register
  - Downstream blocked seams:
    - `SEAM-4` (consumes `C-02` / `THR-02`)
    - `SEAM-6` (consumes `C-02` / `THR-02`)
  - Contracts produced:
    - `C-02` (owned by `SEAM-2`, carried on `THR-02`)
  - Contracts consumed:
    - `C-01` (owned by `SEAM-1`, carried on `THR-01`)

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `C-02` / `THR-02` is the promotion input for provisioning routing (`SEAM-4`) and parity validation (`SEAM-6`). The seam-exit gate makes “probe + support gate is real, evidenced, and fail-closed” a closeout-backed fact instead of an assumption.
- **Expected contracts to publish**:
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-02`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - Any change to:
    - `/etc/os-release` family mapping and normalization rules
    - contradiction rules and fail-closed posture
    - platform/backend supported-vs-unsupported posture (Linux host-native, macOS Lima, Windows WSL)
    - exit `4` wording and diagnostic labeling that downstream seams quote
- **Expected closeout evidence**:
  - Published `C-02` contract artifact location(s)
  - Probe/support-gate evidence across supported and unsupported paths (including the manual-only Arch lane on macOS, if still required)
  - Thread-state update record for `THR-02`
  - Recorded “review surface delta” list for any touch-surface drift discovered during landing

## Slice index

- `S1` -> `slice-1-c-02-contract-definition.md`
- `S2` -> `slice-2-in-world-probe-and-support-gate-integration.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
