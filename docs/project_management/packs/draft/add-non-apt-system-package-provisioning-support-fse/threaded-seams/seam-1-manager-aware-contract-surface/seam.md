---
seam_id: SEAM-1
seam_slug: manager-aware-contract-surface
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-1-manager-aware-contract-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
  - shared CLI/runtime wording changes in ADR-0033 or overlapping pack contracts
  - exit-code or request-profile posture changes before seam-local review
  - v1 pacman scope or authority-handoff targets change and make the extracted contract basis stale
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
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-1 - Manager-aware contract surface

## Seam Brief (Restated)

- **Goal / value**:
  - Freeze one authoritative manager-aware operator contract and accepted decision set so downstream seams can plan against a single truth (instead of inheriting APT-only drift or overlapping pack ambiguity).
- **Type**: integration
- **Scope**
  - In:
    - Shared manager-aware semantics for `substrate world enable --provision-deps`
    - Shared runtime invariant that `substrate world deps current sync|install` must not mutate system package managers
    - Authority handoff from the older APT-only pack and the upstream bundles contract
    - Exit-code posture, request-profile posture, platform/backend guarantees, and mixed-manager failure rule
    - Accepted decisions DR-0001 through DR-0004 for schema posture, probe precedence, pacman execution shape, and v1 pacman runnable scope
  - Out:
    - Implementing the in-world probe and support gate
    - Implementing `install.method=pacman` schema validation or inventory views
    - Implementing provisioning-time requirement derivation or pacman dispatch
    - Implementing runtime read-only probes or remediation rendering
    - Smoke/manual evidence and shared-doc reconciliation landing work
- **Touch surface**:
  - Primary contract inputs:
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
    - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
  - Known “second truth” reconciliation targets (owned by `SEAM-6`):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (compat shim)
    - `docs/reference/world/deps/README.md`
- **Verification**:
  - Because this seam **produces** `C-01`, verification is: `C-01` is concrete enough (rules + boundaries + exit codes + defer/authority map) for downstream seams to begin planning and later pass their own pre-exec gates without waiting for post-exec publication/reconciliation.
- **Basis posture**:
  - Currentness: `provisional` (until seam-local review confirms no contradictory “second truth” is being relied on for pre-exec planning)
  - Upstream closeouts assumed: none
  - Required threads (inbound): none
  - Stale triggers:
    - shared CLI/runtime wording changes in ADR-0033 or overlapping pack contracts
    - exit-code or request-profile posture changes before seam-local review
    - v1 pacman scope or authority-handoff targets change and make the extracted contract basis stale
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - Contracts produced:
    - `C-01` (owned by `SEAM-1`, carried on `THR-01`)
  - Contracts consumed:
    - none (within this pack’s contract registry); upstream docs are basis references only and must be explicitly deferred-to in `C-01`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `THR-01` is load-bearing and unblocks every downstream seam; the seam-exit gate makes “contract and authority handoff are real” a closeout-backed promotion input instead of an implicit assumption.
- **Expected contracts to publish**:
  - `C-01`
- **Expected threads to publish / advance**:
  - `THR-01`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - Any change to:
    - provisioning-vs-runtime workflow wording
    - exit-code mapping for unsupported backends/managers
    - request-profile / execution isolation posture for provisioning
    - v1 pacman “provisioning-only, non-runnable” constraint
- **Expected closeout evidence**:
  - Published `C-01` contract artifact location(s) and a recorded authority/defer map
  - A recorded “review surface delta” list for reconciliation targets owned by `SEAM-6`
  - Thread-state update record for `THR-01`

## Slice index

- `S1` -> `slice-1-c-01-contract-definition.md`
- `S2` -> `slice-2-authority-handoff-and-decisions.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
