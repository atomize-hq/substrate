# Decision Register — stabilize-dev-install-helper-discovery

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:

- This decision register publishes the SEAM-1 staging and discovery decisions absorbed from the source pack planning materials.
- The decisions here are documentary publication decisions, not runtime implementation tasks.

---

### DR-0001 — Exact durable bundle surface under `$SUBSTRATE_HOME`

**Decision owner(s):** SEAM-1 maintainers  
**Date:** 2026-03-30  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threading.md`

**Problem / Context**

- SEAM-2 and SEAM-3 need one exact staged bundle inventory rather than an implied set of helper and support files spread across installer internals.
- The SEAM-1 contract must publish the durable surface without broadening scope into cleanup or parity concerns.

**Option A — Publish only the helper scripts**

- **Pros:**
  - Smallest possible contract surface.
  - Lowest documentation burden.
- **Cons:**
  - Omits the macOS support files and Linux guest-binary staging surface that downstream seams need as basis.
  - Leaves the bundle boundary under-specified.
- **Cascading implications:**
  - Downstream seams would have to infer path membership from script behavior.
  - `C-02` would not be concrete enough for later cleanup or conformance planning.
- **Risks:**
  - Contract drift between the staged tree and the published docs.
- **Unlocks:**
  - None that satisfy the source pack boundary.

**Option B — Publish one exact durable bundle surface**

- **Pros:**
  - Gives downstream seams one explicit bundle inventory to consume.
  - Keeps staged-path membership stable and reviewable.
  - Matches the seam map’s claim that SEAM-1 publishes the first exact staged surface.
- **Cons:**
  - Freezes a larger path list in the publication docs.
- **Cascading implications:**
  - `contract.md` must name the fixed script, YAML, macOS support, and best-effort Linux guest-binary paths.
  - Later seams must revalidate if the fixed path list changes.
- **Risks:**
  - Any future staging change becomes a stale trigger for the SEAM-1 contract.
- **Unlocks:**
  - Clean downstream contract consumption for cleanup and conformance seams.

**Recommendation**

- **Selected:** Option B — Publish one exact durable bundle surface.
- **Rationale (crisp):** SEAM-1 is the first publishable contract boundary, and downstream seams require a single explicit staged inventory rather than script-derived assumptions.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`

---

### DR-0002 — Helper-discovery precedence and CLI posture

**Decision owner(s):** SEAM-1 maintainers  
**Date:** 2026-03-30  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threading.md`

**Problem / Context**

- The helper-resolution contract must remain deterministic, preserve the current override behavior, and keep the dev-install posture narrow.
- The publication artifact must freeze the exact order without introducing new runtime wording or flag behavior.

**Option A — Allow discovery rules to stay implicit**

- **Pros:**
  - Avoids restating existing behavior in the docs.
- **Cons:**
  - Leaves downstream seams to infer ordering from code and tests.
  - Does not publish the first contract boundary cleanly.
- **Cascading implications:**
  - `C-01` would be under-specified.
  - SEAM-3 would have to reconstruct the contract from implementation detail.
- **Risks:**
  - Hidden helper-order drift becomes easy to miss.

**Option B — Publish the exact helper-order and CLI posture**

- **Pros:**
  - Freezes the helper order as a contract boundary.
  - Keeps `--home` valid and `--prefix` invalid in one explicit place.
  - Makes the fail-closed condition part of the published seam surface.
- **Cons:**
  - The order and CLI posture become explicit compatibility obligations.
- **Cascading implications:**
  - Any helper-order or flag-surface change becomes a stale trigger for the seam.
  - Downstream seam planning can cite one canonical discovery contract.
- **Risks:**
  - None beyond the normal contract-freeze maintenance burden.

**Recommendation**

- **Selected:** Option B — Publish the exact helper-order and CLI posture.
- **Rationale (crisp):** SEAM-1 owns helper discovery, so the exact order and posture belong in the published contract rather than in code-only assumptions.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`

---

### DR-0003 — Managed-asset eligibility and manifest boundary

**Decision owner(s):** SEAM-1 maintainers  
**Date:** 2026-03-30  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threading.md`

**Problem / Context**

- The managed-asset boundary must distinguish repo-managed symlinks from copied Linux guest binaries without broadening cleanup inference or collapsing all staged files into one class.

**Option A — Treat all staged assets as equally dev-managed**

- **Pros:**
  - Simplifies the concept of a managed asset.
- **Cons:**
  - Erases the distinction downstream cleanup needs between symlink provenance and copied binaries.
  - Makes the contract too broad for the cleanup seam.
- **Cascading implications:**
  - `C-03` would not provide a usable ownership boundary.
  - Later cleanup logic would have to infer ownership more aggressively.
- **Risks:**
  - Ambiguous cleanup eligibility and accidental overreach.

**Option B — Publish the symlink-versus-manifest split**

- **Pros:**
  - Preserves a narrow managed-asset boundary.
  - Gives downstream seams a concrete rule for repo-managed versus copied Linux guest binaries.
  - Matches the staging shape already reflected in the seam map.
- **Cons:**
  - Requires the contract to name the manifest path explicitly.
- **Cascading implications:**
  - Changes to the manifest location or symlink eligibility stale the seam.
  - Downstream seams can reason about cleanup without inventing new ownership rules.
- **Risks:**
  - Future staging changes must be revalidated against the published boundary.

**Recommendation**

- **Selected:** Option B — Publish the symlink-versus-manifest split.
- **Rationale (crisp):** SEAM-1 needs one explicit ownership boundary for later cleanup and conformance seams, and the split between repo-managed symlinks and manifest-tracked copied binaries is the narrowest useful rule.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/contract.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`
