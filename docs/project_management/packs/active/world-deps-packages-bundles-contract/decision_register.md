# Decision Register — world-deps-packages-bundles-contract

Template standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

### DR-0001 — Replace legacy world-deps selection CLI surface

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`, `docs/project_management/next/world_deps_packages_bundles_contract.md`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP0-spec.md`

**Problem / Context**
- The repo currently ships a legacy `world deps` selection-file model (`status|init|select|provision`) that conflicts with ADR-0011’s inventory + enabled-patch contract.

**Option A — Keep the legacy CLI surface and add a parallel “v2” namespace**
- **Pros:** smaller immediate diff; preserves legacy docs.
- **Cons:** two competing operator contracts; adds long-lived ambiguity; increases support cost.
- **Cascading implications:** duplicated docs and smoke; unclear deprecation policy.
- **Risks:** users enable the wrong model; tests cover only one contract.
- **Unlocks:** none required by ADR-0011.

**Option B — Replace the CLI surface in place**
- **Pros:** single operator contract; aligns to ADR-0011 and `ADR-0008` scope model.
- **Cons:** breaking change for any users relying on legacy commands.
- **Cascading implications:** requires tests enforcing replacement completeness (legacy paths not read).
- **Risks:** short-term churn; mitigated by explicit help text and smoke updates.
- **Unlocks:** coherent inventory/enabled/applied model.

**Recommendation**
- **Selected:** Option B — Replace the CLI surface in place
- **Rationale (crisp):** ADR-0011 is approved and explicitly requires legacy paths removed from plumbing; parallel namespaces preserve ambiguity.

**Follow-up tasks (explicit)**
- Implement the new `current|global|workspace` CLI contract.
- Remove legacy selection file reads from `world deps` plumbing and enforce via tests.

### DR-0002 — World “applied” view computation model

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world_deps_packages_bundles_contract.md`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-spec.md`

**Problem / Context**
- The contract defines `present|missing|blocked` as world-backed status and requires a deterministic `current list applied`.

**Option A — Persist an “applied set” state file inside the world**
- **Pros:** fast `applied` view; avoids probes when unchanged.
- **Cons:** introduces a new persisted schema and migration surface; risks drift between state and reality.
- **Cascading implications:** new file location, versioning, and corruption semantics.
- **Risks:** false positives/negatives if state diverges.
- **Unlocks:** stable `applied` output independent of probe runtime.
- **Quick wins / low-hanging fruit:** incremental implementation that starts with a small, fixed schema and a single read path.

**Option B — Derive applied status from probes on demand**
- **Pros:** no new persisted schema; status reflects reality at query time; matches contract’s probe-centric semantics.
- **Cons:** `applied` may be slower for large inventories.
- **Cascading implications:** probes must be stable and observable; concurrency must be controlled.
- **Risks:** performance; mitigated by scoping default `applied` to the enabled set and requiring `--all` for full inventory.
- **Unlocks:** no migration surface; probe results remain the single source of truth.
- **Quick wins / low-hanging fruit:** aligns directly with WDP2 scope without introducing a new on-disk format.

**Recommendation**
- **Selected:** Option B — Derive from probes on demand
- **Rationale (crisp):** avoids introducing a new state schema surface and keeps “applied” grounded in reality.

**Follow-up tasks (explicit)**
- Implement the WDP2 applied derivation contract (tasks: `WDP2-code`, `WDP2-test`, `WDP2-integ-core`, `WDP2-integ-linux`, `WDP2-integ-macos`, `WDP2-integ`).

### DR-0003 — Script installer execution shell inside the world

**Decision owner(s):** World maintainers  
**Date:** 2026-02-13  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world_deps_packages_bundles_contract.md`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`

**Problem / Context**
- Script installers in common ecosystems frequently assume bash features (`set -euo pipefail`, arrays, `source`).

**Option A — Execute script installers under POSIX `sh -c`**
- **Pros:** minimal dependency footprint; aligned to non-interactive runtime shell.
- **Cons:** incompatible with common installer recipes; increases wrapper/script complexity.
- **Cascading implications:** installer docs and examples avoid bash-only constructs; more escape/quoting rules appear in wrapper generation.
- **Risks:** installer failure rates increase for common ecosystems; failures surface late during mutation slices.
- **Unlocks:** no additional world image dependency for installs.
- **Quick wins / low-hanging fruit:** reuse existing `sh -c` execution plumbing without introducing a bash requirement.

**Option B — Execute script installers under `bash -lc`**
- **Pros:** high compatibility with ecosystem installers; matches contract’s install-time note.
- **Cons:** requires bash to exist in the world image.
- **Cascading implications:** world images guarantee bash availability for install execution; wrapper generation and remediation messaging reference `bash -lc` explicitly.
- **Risks:** installs fail-closed when bash is absent; failure is deterministic and actionable.
- **Unlocks:** supports `source`-based installers and bash functions without additional wrapper shims.
- **Quick wins / low-hanging fruit:** aligns with common installer recipes without requiring rewrite.

**Recommendation**
- **Selected:** Option B — Execute under `bash -lc`
- **Rationale (crisp):** maximizes compatibility while preserving the runtime “no rcfiles required” contract via generated wrappers.

**Follow-up tasks (explicit)**
- Implement the WDP4 script installer execution shell contract and wrapper generation (tasks: `WDP4-code`, `WDP4-test`, `WDP4-integ`).
