# Decision Register — Policy + Config Mental Model Simplification (ADR-0003)

This decision register records ADR-0003 architectural decisions as exactly two options (A/B) with explicit tradeoffs and a single selected option. Every decision maps to concrete follow-up triad tasks in:
- `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`

### DR-0001 — Workspace root marker

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`

**Problem / Context**
- Workspace scoping must be explicit and testable without relying on `.git` presence or heuristics.

**Option A — Marker file at `.substrate/workspace.yaml`**
- **Pros:** explicit, unambiguous, testable; supports walk-up discovery and strict “in workspace” gating.
- **Cons:** requires an explicit init step; directories without the marker are not workspaces.
- **Cascading implications:** `substrate workspace init` is mandatory for workspace-scoped commands; nested workspaces require an explicit refusal rule.
- **Risks:** users with existing repos need to initialize workspaces explicitly.
- **Unlocks:** stable workspace root for anchor resolution and future world-sync config.
- **Quick wins / low-hanging fruit:** deterministic tests for discovery and refusal.

**Option B — Infer from `.substrate/` directory or `.git/`**
- **Pros:** less explicit setup for some repositories.
- **Cons:** ambiguous; multiple `.substrate/` uses exist; `.git/` is not universal; complicates gating and precedence semantics.
- **Cascading implications:** discovery logic accumulates special cases and drift.
- **Risks:** incorrect root selection causes policy/config application in the wrong directory.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none without reintroducing ambiguity.

**Recommendation**
- **Selected:** Option A — Marker file at `.substrate/workspace.yaml`
- **Rationale (crisp):** ADR-0003 requires a single, strict mental model with deterministic discovery.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM0-test`, `PCM0-integ`

### DR-0002 — Canonical file inventory

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`

**Problem / Context**
- Configuration and policy inputs must have a finite, unambiguous inventory with strict loader behavior.

**Option A — Strict global + workspace inventory**
- **Pros:** predictable and testable; one mental model; eliminates collisions between config/settings/profile/policy.
- **Cons:** breaking change; requires removing legacy discovery paths.
- **Cascading implications:** strict parsing and legacy rejection tests are mandatory.
- **Risks:** existing installs relying on legacy paths break immediately.
- **Unlocks:** stable contracts for future features (world-sync, json-mode).
- **Quick wins / low-hanging fruit:** one loader path per artifact and one set of tests.

**Option B — Preserve legacy filenames and discovery fallbacks**
- **Pros:** fewer immediate breakages.
- **Cons:** preserves ambiguity and drift; increases maintenance surface; violates ADR-0003 hard rules.
- **Cascading implications:** requires ongoing support and compatibility policy not allowed by ADR-0003.
- **Risks:** split-brain behavior and non-deterministic policy/config selection.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none without violating greenfield constraints.

**Recommendation**
- **Selected:** Option A — Strict global + workspace inventory
- **Rationale (crisp):** ADR-0003 forbids compatibility and requires a single source of truth.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM1-code`, `PCM3-code` and corresponding test/integration tasks

### DR-0003 — Backwards compatibility policy

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/sequencing.json`

**Problem / Context**
- The mental model simplification must not carry legacy aliases, migrations, or discovery fallbacks.

**Option A — No compatibility; remove all legacy artifacts**
- **Pros:** simplest and most deterministic mental model; lowest long-term maintenance; aligns with sequencing policy.
- **Cons:** immediate breaking changes for existing installs.
- **Cascading implications:** explicit rejection tests for legacy files/flags/env vars are required.
- **Risks:** users need to update workflows immediately.
- **Unlocks:** clean contracts for policy evaluation and world enable state.
- **Quick wins / low-hanging fruit:** fewer code paths and fewer schema variants.

**Option B — Compatibility layer (aliases and migrations)**
- **Pros:** smoother upgrade.
- **Cons:** violates ADR-0003 hard rules; preserves ambiguity; increases complexity and test surface.
- **Cascading implications:** requires a deprecation and removal policy not defined in sequencing.
- **Risks:** compatibility layer becomes permanent.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — No compatibility; remove all legacy artifacts
- **Rationale (crisp):** ADR-0003 explicitly forbids backward compatibility.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM1-code`, `PCM3-code` and corresponding test/integration tasks

### DR-0004 — Anchor mode naming

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`

**Problem / Context**
- Anchor naming must align with workspace scoping and remove “root” naming from all surfaces.

**Option A — `anchor_mode=workspace|follow-cwd|custom`**
- **Pros:** matches workspace scoping model; removes “root” naming; aligns with roaming guard semantics.
- **Cons:** breaking rename for existing users.
- **Cascading implications:** CLI/env/config surfaces require consistent renames and explicit removal tests.
- **Risks:** legacy names present in code create silent partial behavior.
- **Unlocks:** a single anchor root for roaming guard and project isolation.
- **Quick wins / low-hanging fruit:** one set of config keys and one CLI surface.

**Option B — Keep legacy “root” naming**
- **Pros:** fewer renames.
- **Cons:** violates ADR-0003 removal rules; preserves collisions with filesystem isolation concepts.
- **Cascading implications:** requires mapping code and compatibility layers.
- **Risks:** ambiguous “root vs anchor” semantics remain.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — `anchor_mode=workspace|follow-cwd|custom`
- **Rationale (crisp):** ADR-0003 requires anchor-only naming and removal of root naming.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM3-code` and corresponding test/integration tasks

### DR-0005 — Environment script responsibilities

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`

**Problem / Context**
- Cached state must not drift because runtime overwrites installer state.

**Option A — Split `env.sh` (stable exports) from `manager_env.sh` (runtime wiring)**
- **Pros:** stable cached exports; runtime can regenerate glue without clobbering state; deterministic sourcing chain.
- **Cons:** introduces one more file and ownership rules that require tests.
- **Cascading implications:** runtime must always source `env.sh` when present; installers and CLI must never write exports into `manager_env.sh`.
- **Risks:** partial updates create inconsistent exports if ownership rules are violated.
- **Unlocks:** reliable, repeatable shell initialization and traceable state.
- **Quick wins / low-hanging fruit:** clear test boundaries for rewrite behavior.

**Option B — Store stable exports in `manager_env.sh`**
- **Pros:** fewer files.
- **Cons:** runtime rewrite clobbers stable exports; drift is unavoidable; violates ADR-0003 drift prevention goal.
- **Cascading implications:** persistent split-brain behaviors across code paths.
- **Risks:** cached state becomes unreliable.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Split `env.sh` from `manager_env.sh`
- **Rationale (crisp):** stable cached state requires stable ownership boundaries.

**Follow-up tasks (explicit)**
- `PCM3-code`, `PCM3-test`, `PCM3-integ`

### DR-0006 — World enable home semantics

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`

**Problem / Context**
- World enable state writes must not be split between an install root and a different `$SUBSTRATE_HOME`.

**Option A — `substrate world enable --home <PATH>` controls all state writes**
- **Pros:** unambiguous; single root for logs/config/env scripts; consistent helper invocation contract.
- **Cons:** breaking CLI change; requires removing `--prefix`.
- **Cascading implications:** tests must verify `SUBSTRATE_PREFIX` has no effect and `--prefix` is rejected.
- **Risks:** existing scripts invoking `--prefix` break.
- **Unlocks:** consistent enable and world-deps manager wiring.
- **Quick wins / low-hanging fruit:** simpler internal path resolution.

**Option B — Keep `--prefix` while metadata still writes to `$SUBSTRATE_HOME`**
- **Pros:** fewer CLI changes.
- **Cons:** split-brain; ambiguous; violates ADR-0003 hard rules.
- **Cascading implications:** two roots must be reconciled in every subsystem.
- **Risks:** provisioning one install and updating a different home becomes common.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — `--home` controls all state writes
- **Rationale (crisp):** a single root for state is mandatory for a simplified mental model.

**Follow-up tasks (explicit)**
- `PCM3-code`, `PCM3-test`, `PCM3-integ`

### DR-0007 — Policy mode taxonomy

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`

**Problem / Context**
- Runtime must represent “no evaluation” distinctly from “evaluate but do not block”.

**Option A — `disabled|observe|enforce`**
- **Pros:** explicit disabled mode; observe enables auditable decisions without blocking; enforce gates execution.
- **Cons:** requires plumbing and tests for three modes.
- **Cascading implications:** routing logic must treat observe as non-blocking and disabled as no evaluation.
- **Risks:** incorrect mode mapping causes silent policy bypass.
- **Unlocks:** deterministic audit and future agent planning based on computed decisions.
- **Quick wins / low-hanging fruit:** explicit config key and env export.

**Option B — `observe|enforce` only**
- **Pros:** fewer modes.
- **Cons:** cannot represent “no evaluation” state; forces evaluation even for trace-only use cases.
- **Cascading implications:** increases cost and complexity for trace-only operation.
- **Risks:** accidental evaluation becomes unavoidable.
- **Unlocks:** none aligned with ADR-0003 goals.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — `disabled|observe|enforce`
- **Rationale (crisp):** ADR-0003 requires an explicit disabled mode.

**Follow-up tasks (explicit)**
- `PCM2-code`, `PCM2-test`, `PCM2-integ`

### DR-0008 — Command pattern semantics

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`

**Problem / Context**
- Policy command matching must be predictable and safe under strict parsing.

**Option A — Glob when `*` exists, otherwise substring match**
- **Pros:** simple and predictable; low accidental broad match risk; easy to test.
- **Cons:** limited expressiveness versus regex.
- **Cascading implications:** shared matcher used by `cmd_allowed`, `cmd_denied`, `cmd_isolated`.
- **Risks:** users expect regex behavior and configure incorrect patterns.
- **Unlocks:** deterministic matching for audit and replay.
- **Quick wins / low-hanging fruit:** minimal matcher implementation and tests.

**Option B — Regex patterns**
- **Pros:** expressive.
- **Cons:** harder to reason about; increases accidental matches; expands attack surface for poorly bounded patterns.
- **Cascading implications:** regex compilation errors become a config failure mode that needs new UX rules.
- **Risks:** policy patterns become non-auditable at a glance.
- **Unlocks:** none required by ADR-0003.
- **Quick wins / low-hanging fruit:** none under strict contract constraints.

**Recommendation**
- **Selected:** Option A — Glob when `*` exists, otherwise substring match
- **Rationale (crisp):** ADR-0003 prioritizes predictability and testability over expressiveness.

**Follow-up tasks (explicit)**
- `PCM1-code` (matcher exposure for schema), `PCM2-code` (runtime evaluation), and corresponding tests

### DR-0009 — Protected sync excludes

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`

**Problem / Context**
- Sync must not mutate `.git`, `.substrate`, or `.substrate-git` under any configuration layer.

**Option A — Always-on protected excludes**
- **Pros:** safe by default; prevents repo corruption and internal state corruption; simple invariant to test.
- **Cons:** less configurability.
- **Cascading implications:** protected patterns are prepended and cannot be removed.
- **Risks:** users attempt to remove protected patterns and are surprised by the invariant.
- **Unlocks:** stable foundation for world-sync.
- **Quick wins / low-hanging fruit:** deterministic tests for protected patterns.

**Option B — User-controlled excludes**
- **Pros:** maximum control.
- **Cons:** enables unsafe configurations; violates ADR-0003 safety posture; complicates future sync UX.
- **Cascading implications:** requires additional safety policy semantics and warnings.
- **Risks:** data loss and corruption.
- **Unlocks:** none aligned with ADR-0003 safety posture.
- **Quick wins / low-hanging fruit:** none without violating safety constraints.

**Recommendation**
- **Selected:** Option A — Always-on protected excludes
- **Rationale (crisp):** protected internal and repo paths are non-negotiable under ADR-0003 safety posture.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM0-test`, `PCM0-integ`

### DR-0010 — Nested workspace behavior

**Decision owner(s):** spenser  
**Date:** 2025-12-27  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`

**Problem / Context**
- Nested workspaces introduce ambiguity in discovery and in policy/config layering.

**Option A — Refuse nested workspaces**
- **Pros:** no ambiguity about workspace root; simplest discovery and gating semantics; deterministic tests.
- **Cons:** restricts some monorepo layouts.
- **Cascading implications:** init must detect parent marker and perform a no-write early exit with status 2.
- **Risks:** users with nested repos cannot use multiple workspace roots under ADR-0003.
- **Unlocks:** stable workspace boundary and stable anchor selection.
- **Quick wins / low-hanging fruit:** straightforward implementation and tests.

**Option B — Allow nested workspaces**
- **Pros:** supports nested repo layouts.
- **Cons:** introduces ambiguous layering and selection; requires new precedence rules not allowed by ADR-0003.
- **Cascading implications:** complex discovery and conflicts between policy/config files.
- **Risks:** wrong policy/config selection during execution.
- **Unlocks:** none aligned with ADR-0003 constraints.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Refuse nested workspaces
- **Rationale (crisp):** ADR-0003 prioritizes unambiguous discovery over nested repo flexibility.

**Follow-up tasks (explicit)**
- `PCM0-code`, `PCM0-test`, `PCM0-integ`

