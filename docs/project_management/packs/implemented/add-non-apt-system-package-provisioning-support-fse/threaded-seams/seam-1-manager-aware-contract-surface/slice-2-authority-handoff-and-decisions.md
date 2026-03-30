---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
  - any “second truth” doc asserts runtime mutation
  - request-profile posture is changed into an operator surface
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced: []
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S2 - Authority handoff + accepted decision register (DR-0001..DR-0004)

#### Goal

Make the “who is authoritative for what” story deterministic and capture the accepted decisions that downstream seams must treat as non-negotiable inputs.

#### Authority / defer map (expanded)

This seam produces a manager-aware contract that must not fork existing world-deps semantics. Treat this map as binding for downstream planning until `SEAM-6` reconciliation lands.

- **Authoritative (this seam / `C-01`)**
  - One operator contract for:
    - `substrate world enable --provision-deps` (provisioning-time only)
    - runtime `substrate world deps current sync|install` no-mutation posture + remediation pointer back to provisioning
  - Fail-closed rules:
    - unsupported backend/world manager -> exit `4`
    - mixed-manager requirement sets -> exit `4` before any package-manager command runs
    - mismatch between detected manager and required install methods -> exit `4`
  - v1 pacman scope constraints and “no AUR helper / no translation” posture

- **Defers to (upstream authority remains binding)**
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - inventory/enabled merge semantics, wrapper semantics, schema v1 baseline, and “enabled set drives derived requirements”
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - base meaning of exit codes; this seam only binds feature-specific interpretations
  - `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
    - APT provisioning baseline semantics (as long as it does not contradict `C-01`)

- **Orientation / rationale (must not be treated as the operator contract if it conflicts)**
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

- **Reconciliation targets (owned by `SEAM-6`, tracked as `REM-001`)**
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (compat shim)
  - `docs/reference/world/deps/README.md`
  - Any shared “world deps” internal docs that still imply runtime mutation or APT-only posture

#### Accepted decisions (DR-0001..DR-0004)

These decisions are the “contract basis” that downstream seams must treat as inputs, not as re-opened options.

- **DR-0001 — Schema posture (explicit method, additive on v1)**
  - Decision: represent pacman as an explicit system-package method (`install.method=pacman` + `install.pacman`) alongside `install.method=apt`.
  - Rationale: avoids abstract translation layers and keeps inventory intent readable.
  - Downstream constraints:
    - remain additive on `version: 1` (no schema version bump)
    - pacman is provisioning-only + non-runnable in v1

- **DR-0002 — Probe strategy (in-world only; `/etc/os-release` + manager presence)**
  - Decision: derive the world OS family/manager via an in-world probe using `/etc/os-release` plus in-world package-manager availability.
  - Rationale: avoids host-derived routing and makes behavior stable across host platforms.
  - Downstream constraints:
    - host PATH must never be consulted for manager selection
    - unsupported/ambiguous results fail closed (exit `4`)

- **DR-0003 — Pacman invocation and idempotency (exact command shape)**
  - Decision: provisioning pacman execution is exactly:
    - `pacman -Sy --noconfirm --needed <packages...>` in normalized order
  - Rationale: deterministic, non-interactive, and idempotent enough for v1 without introducing retries/lock intervention.
  - Downstream constraints:
    - no AUR helper widening
    - no retries / lock-file intervention in v1
    - stable normalization and ordering is required for reproducible dry-run output and logs

- **DR-0004 — Mismatch policy (fail closed; no partial provisioning)**
  - Decision: when the enabled set’s required manager(s) do not match the detected manager, or when multiple managers are required, fail closed.
  - Rationale: avoids partial provisioning and avoids making the tool guess which manager to mutate.
  - Downstream constraints:
    - exit `4` for mismatch and mixed-manager cases
    - failure occurs before any mutating manager command runs

#### Verification checklist

- Downstream seams can cite an explicit authority map for all overlapping documents.
- Downstream seams treat DR-0001..DR-0004 as fixed inputs.
- `REM-001` is recognized as a downstream reconciliation obligation (not a pre-exec blocker for `SEAM-1` execution).

