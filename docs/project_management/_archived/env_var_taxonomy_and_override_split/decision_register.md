# Decision Register — Env Var Taxonomy + Override Split

This decision register captures the A/B decisions referenced by `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`.

### DR-0001 — Naming scheme for override inputs

**Decision owner(s):** shell/config  
**Date:** 2026-01-04  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`, `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`

**Problem / Context**
- `SUBSTRATE_*` variables are written by Substrate-owned scripts/runtime as stable exports (`$SUBSTRATE_HOME/env.sh`), and some of the same variable names are also consulted as override inputs by effective-config resolution.
- Dual-use makes exported state indistinguishable from intentional operator overrides.

**Option A — `SUBSTRATE_OVERRIDE_*` for override inputs; keep `SUBSTRATE_*` as state**
- **Pros:** Minimal churn for existing state consumers; clear intent for overrides; preserves existing stable export scripts.
- **Cons:** Two namespaces exist; docs must clearly explain the split.
- **Cascading implications:** Config resolver must be updated to read `SUBSTRATE_OVERRIDE_*` and stop reading config-shaped `SUBSTRATE_*`.
- **Risks:** Operators accustomed to `SUBSTRATE_*` overrides must update workflows.
- **Unlocks:** Exported state can be safely present in the environment without silently overriding config.
- **Quick wins / low-hanging fruit:** Change is localized to the config resolver and docs.

**Option B — `SUBSTRATE_STATE_*` for state; keep `SUBSTRATE_*` as override inputs**
- **Pros:** “State” is explicit; override inputs remain the shorter names.
- **Cons:** Large blast radius across crates/scripts/docs that read `SUBSTRATE_*` as state.
- **Cascading implications:** Requires renaming stable exports and any internal consumers.
- **Risks:** High migration cost; breaks installer/dev workflows.
- **Unlocks:** Explicit state namespace in the environment.
- **Quick wins / low-hanging fruit:** None; requires broad refactors.

**Recommendation**
- **Selected:** Option A — `SUBSTRATE_OVERRIDE_*` for override inputs
- **Rationale (crisp):** It eliminates dual-use while minimizing cross-crate churn.

**Follow-up tasks (explicit)**
- Implement in `EV0-code`: read `SUBSTRATE_OVERRIDE_*` env vars as override inputs; ignore config-shaped `SUBSTRATE_*` (task: `EV0-code`).
- Implement in `EV0-code`: perform an explicit repo-wide grep/audit to ensure no commands bypass effective config by reading config-shaped `SUBSTRATE_*` values as operator inputs; fix any hits found (task: `EV0-code`).
- Add tests for the split (task: `EV0-test`).
- Update operator docs and the canonical env-var catalog references (task: `EV0-integ-core` / `EV0-integ`).

### DR-0002 — Scope of override variables

**Decision owner(s):** shell/config  
**Date:** 2026-01-04  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`, `docs/ENVIRONMENT_VARIABLES.md`

**Problem / Context**
- The repo contains both config-shaped inputs (keys that correspond to config schema) and non-config override-only controls (socket paths, test toggles, transport overrides).
- The override split is targeted at eliminating dual-use for config-shaped keys exported in `env.sh`.

**Option A — Override vars exist only for config-shaped keys**
- **Pros:** Smallest surface area; maps 1:1 to config schema keys; avoids renaming non-config controls.
- **Cons:** Non-config override knobs remain under existing names and conventions.
- **Cascading implications:** The override catalog must explicitly list which keys are config-shaped override inputs.
- **Risks:** Some operators may expect every `SUBSTRATE_*` knob to have an override variant.
- **Unlocks:** Clear rule: `SUBSTRATE_OVERRIDE_*` is only for effective-config resolution.
- **Quick wins / low-hanging fruit:** Implement by updating only the config resolver and docs.

**Option B — Provide override vars for all operator-intended overrides (including override-only knobs)**
- **Pros:** Consistent naming for all operator controls; reduces guesswork.
- **Cons:** Larger migration; more names to document; higher risk of churn.
- **Cascading implications:** Installer/scripts/backends may need updates to accept renamed env vars.
- **Risks:** Breaks existing workflows relying on current override-only env vars.
- **Unlocks:** Uniform override interface.
- **Quick wins / low-hanging fruit:** None; requires large catalog and broad changes.

**Recommendation**
- **Selected:** Option A — override vars only for config-shaped keys
- **Rationale (crisp):** It solves the dual-use footgun with minimal surface area and minimal churn.

**Follow-up tasks (explicit)**
- Ensure `docs/ENVIRONMENT_VARIABLES.md` lists the reserved `SUBSTRATE_OVERRIDE_*` keys for config-shaped overrides (task: `EV0-integ`).

### DR-0003 — Canonical environment variable catalog location

**Decision owner(s):** docs/config  
**Date:** 2026-01-04  
**Status:** Accepted  
**Related docs:** `docs/ENVIRONMENT_VARIABLES.md`, `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`

**Problem / Context**
- Env var references are scattered across docs and code; drift makes operator guidance unreliable.

**Option A — Canonical catalog under `docs/ENVIRONMENT_VARIABLES.md`**
- **Pros:** Single source of truth; easy discovery; can be referenced from `docs/CONFIGURATION.md` and ADRs.
- **Cons:** Requires careful curation so internal-only variables are labeled and not treated as stable operator interface.
- **Cascading implications:** Changes to env vars must update this file.
- **Risks:** If not maintained, it becomes stale and misleading.
- **Unlocks:** Clear taxonomy and authoritative inventory.
- **Quick wins / low-hanging fruit:** Reference the catalog from configuration docs.

**Option B — Catalog under `docs/project_management/**` only**
- **Pros:** Keeps catalog “internal”; avoids implying stability.
- **Cons:** Lower discoverability; increases drift risk vs operator docs.
- **Cascading implications:** Requires duplication or cross-linking to operator docs.
- **Risks:** Operators do not find it.
- **Unlocks:** Internal-only planning artifact.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — `docs/ENVIRONMENT_VARIABLES.md`
- **Rationale (crisp):** It creates a discoverable single source of truth while still allowing explicit “internal/test” labeling.

**Follow-up tasks (explicit)**
- Ensure `docs/CONFIGURATION.md` references `docs/ENVIRONMENT_VARIABLES.md` and the override split (task: `EV0-integ`).
