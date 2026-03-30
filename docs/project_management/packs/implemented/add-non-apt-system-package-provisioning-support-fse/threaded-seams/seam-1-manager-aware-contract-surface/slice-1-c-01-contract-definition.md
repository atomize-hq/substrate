---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
  - exit-code taxonomy changes
  - ADR-0030 or ADR-0033 changes provisioning-vs-runtime contract language
  - upstream bundles contract changes inventory/enabled semantics relied on by this contract
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
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-01` manager-aware operator contract

#### Goal

Make `C-01` explicit enough that every downstream seam can plan and implement without inventing a second truth for:

- provisioning-time system-package mutation (APT + pacman) via `substrate world enable --provision-deps`
- runtime no-mutation posture for `substrate world deps current sync|install`
- fail-closed behavior (unsupported backends/world OS families; mixed-manager requirements; mismatched manager)

#### `C-01` — Shared manager-aware operator contract (authoritative pre-exec text)

##### 1) Single operator entrypoint for system-package provisioning

- The only operator-facing surface that may mutate world OS system-package state is:
  - `substrate world enable --provision-deps`
- Runtime flows under `substrate world deps current sync|install`:
  - MUST NOT invoke mutating `apt`, `apt-get`, `dpkg`, or `pacman`
  - MAY perform read-only presence checks (`dpkg-query`, `pacman -Q`) and MUST fail early with remediation when unmet system-package requirements exist

##### 2) In-world manager selection (no host routing)

- Manager selection MUST be derived in-world only and MUST NOT route from:
  - host PATH
  - host package-manager presence
  - host installer detection
  - host OS package-manager state
- Allowed manager outcomes are:
  - `apt`
  - `pacman`
  - unsupported (fail closed)

##### 3) v1 pacman scope is provisioning-only and non-runnable

- `install.method=pacman` remains:
  - provisioning-only in v1
  - non-runnable (no runnable-wrapper generation; no `world deps current install` mutation)
  - official-repo-only (no AUR helpers; no `yay`/`paru`/`pamac`)
- v1 pacman does not include:
  - version pinning
  - retries
  - lock-file intervention
  - distro translation / abstract mapping layer

##### 4) Provisioning behavior (fail-closed, no partial mixed-manager)

- Provisioning derives requirement sets from the effective enabled view (per the upstream bundles contract).
- If provisioning is unsupported on the active backend:
  - `substrate world enable --provision-deps` MUST fail closed with exit `4` and remediation text that does not imply host OS mutation.
- If the effective enabled set contains any system-package requirements:
  - provisioning MUST execute only the detected manager path, and MUST NOT partially provision:
    - If both normalized manager requirement sets are non-empty, provisioning MUST exit `4` before any OS package-manager command runs.
    - If the enabled set requires `install.method=pacman` but the detected manager is `apt` (or vice-versa), provisioning MUST exit `4` with explicit mismatch remediation.
- Accepted pacman execution decision (carried forward into `SEAM-4`):
  - pacman execution is exactly `pacman -Sy --noconfirm --needed <packages...>` in normalized order.
- Request-profile posture:
  - provisioning may require a stricter internal request profile / guard rails, but `SUBSTRATE_WORLD_REQUEST_PROFILE` is not an operator control surface for this feature.

##### 5) Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Feature-specific mapping:
  - `0`: success; or “no system-package requirements” no-op by contract
  - `2`: invalid inventory/config/schema (including invalid `install.method`, invalid `install.pacman` shape, or enabling an unknown item)
  - `3`: world backend unavailable when required (cannot reach world-agent to probe or provision)
  - `4`: unsupported operation / missing prerequisites (includes: provisioning unsupported on this backend; detected world manager unsupported; mixed-manager requirement set; detected manager mismatches required methods; runtime system-package requirements missing)
  - `5`: safety/policy violation (reserved; this seam does not introduce new safety surfaces)

##### 6) Authority / defer map (minimal; expanded in `S2`)

- Defers to `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` for:
  - inventory and enabled resolution semantics
  - wrapper/runnable semantics
  - schema versioning baseline (`version: 1`)
- Defers to `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` for:
  - provisioning-time-only OS mutation posture (APT baseline) and runtime fail-early posture
- Defers to `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` for:
  - rationale for extending provisioning-time system packages to pacman, without broadening to other managers

#### Verification checklist (contract gate input)

- `C-01` explicitly forbids runtime OS package-manager mutation.
- `C-01` explicitly forbids host-derived manager selection.
- `C-01` pins v1 pacman scope as provisioning-only and non-runnable.
- `C-01` states fail-closed, no-partial-provisioning posture for mixed-manager and mismatch cases.
- Exit-code mapping is explicit and references the canonical taxonomy.
- Downstream seams can cite `C-01` without inventing new configuration/protocol/log surfaces.

