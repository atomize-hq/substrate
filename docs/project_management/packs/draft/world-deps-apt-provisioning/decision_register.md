# Decision Register — world-deps-apt-provisioning

Template standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

## DR-0001 — APT requirement derivation conflict policy (de-dup, ordering, version pins)

**Decision owner(s):** Shell / world-deps maintainers  
**Date (UTC):** 2026-03-05  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`

**Problem / context**

Provisioning derives an APT package list from the effective enabled world-deps set. Multiple enabled items can reference the same APT package name with different `version` pins. The derivation must be deterministic and must not silently pick an arbitrary version.

**Option A — Strict conflict failure (deterministic; fail closed)**

- Derivation inputs: the in-scope package set after bundle expansion and filtering to `install.method=apt`.
- Normalization:
  - De-duplicate by APT package `name`.
  - Stable ordering: sort by `name` (ascending, byte/ASCII order).
  - Version selection:
    - If all entries for a `name` have `version` unset, the normalized entry has `version` unset.
    - If exactly one distinct non-empty `version` exists for a `name`, the normalized entry uses that `version` (pins win over unpinned).
    - If two or more distinct non-empty `version` values exist for a `name`, the command fails with exit `2` and prints the conflicting versions.
- No partial execution: if any conflict exists, no APT provisioning action is executed.

**Option B — Precedence-based resolution (deterministic, but allows implicit selection)**

- De-duplicate by APT package `name`.
- Stable ordering as in Option A.
- Resolve conflicting version pins by precedence (for example, “closest inventory layer wins” or “first in effective enabled order wins”).

**Recommendation**

- **Selected:** Option A — Strict conflict failure (deterministic; fail closed)
- **Rationale (crisp):** a silent version selection is a hidden OS-mutation policy; conflicts must be surfaced and fixed explicitly.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` (derivation algorithm + error messaging)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (exit-code mapping and operator-visible conflict posture for provisioning)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` (runtime preflight uses the same normalized requirement set)

## DR-0002 — Provisioned-state tracking posture (probe-only vs state file)

**Decision owner(s):** Shell / world-deps maintainers  
**Date (UTC):** 2026-03-05  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

**Problem / context**

Runtime `substrate world deps current sync|install` MUST NOT invoke APT/dpkg. The runtime path still needs a deterministic way to decide whether APT-backed items are already satisfied (no-op) or must be provisioned (fail early with remediation).

**Option A — Probe-only (no persisted state)**

- Provisioning and runtime both use a read-only presence probe for the normalized APT requirement set.
- Runtime behavior:
  - If all required APT packages are present, runtime proceeds with non-APT installs.
  - If any required APT package is missing, runtime exits `4` and prints remediation that includes `substrate world enable --provision-deps`.
- No persisted “provisioned” state file is written by this feature.

**Option B — Persisted provisioned-state file (Substrate-managed)**

- Provisioning writes a deterministic state file recording the normalized APT requirement set and a success marker.
- Runtime reads the state file:
  - If the state file is present and matches the current normalized requirement set, runtime proceeds with non-APT installs.
  - Otherwise runtime exits `4` and prints remediation.

**Recommendation**

- **Selected:** Option A — Probe-only (no persisted state)
- **Rationale (crisp):** a state file can drift from reality; a probe is the only correctness-preserving no-op detector.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` (provisioning no-op and “missing packages” messaging)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` (runtime preflight probe contract + exit codes)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (runtime no-op vs fail-early semantics)

## DR-0003 — Provisioning execution isolation model (request profile, guard rails, env-var relationship)

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-05  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`

**Problem / context**

Provisioning-time APT must be able to mutate guest OS packages without weakening hardened runtime execution. The isolation model must be explicit, must fail closed on Linux host-native, and must not require operators to set `SUBSTRATE_WORLD_REQUEST_PROFILE`.

**Option A — Dedicated provisioning request profile (`world-deps-provision`)**

- Provisioning executions (and only provisioning executions) use the Agent API request `profile` value `world-deps-provision`.
- Guard rails:
  - Linux host-native provisioning is blocked before any OS-mutation attempt and exits `4` with explicit “no host OS mutation” messaging.
  - Runtime `deps current sync|install` never uses this provisioning profile and never invokes APT/dpkg.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` relationship:
  - Provisioning does not depend on `SUBSTRATE_WORLD_REQUEST_PROFILE` and MUST NOT require operators to set it.
  - Provisioning ignores `SUBSTRATE_WORLD_REQUEST_PROFILE` for its own executions.

**Option B — No dedicated profile (widen the hardened runtime posture)**

- Provisioning uses the default request profile and relies on widening write allowances or weakening isolation so APT/dpkg can run.
- Guard rails rely solely on command-level checks.

**Recommendation**

- **Selected:** Option A — Dedicated provisioning request profile (`world-deps-provision`)
- **Rationale (crisp):** provisioning is a separate, explicit OS-mutation workflow; it must be isolated from hardened runtime by construction.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (platform/backends support matrix + explicit profile/env relationship)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` (world-agent request construction + guard rails)
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` (runtime prohibition + remediation invariants)
