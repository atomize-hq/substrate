# Decision Register — llm_and_agent_config_policy_surface

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:
- This decision register supports `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — File families for LLM + agent config/policy (existing YAML vs new formats)

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

**Problem / Context**
- Phase 4/5 ADRs require new config/policy keys. Introducing new “root” file families would multiply precedence rules and drift enforcement boundaries.

**Option A — Reuse existing YAML patch files (new keys only)**
- **Pros:**
  - Preserves a single precedence model across the repo.
  - Keeps `--explain` and provenance surfaces coherent.
  - Minimizes “two config systems” risk.
- **Cons:**
  - Requires adding new keys to existing schema types even before features are fully implemented.
- **Cascading implications:**
  - LLM/agent ADRs must defer to ADR-0027 for key paths + precedence.
- **Risks:**
  - Slightly larger default effective config/policy outputs as new sections appear.
- **Unlocks:**
  - Early fail-closed governance and operator controls before engine/hub code lands.
- **Quick wins / low-hanging fruit:**
  - Add keys + strict parsing first; wire behavior later.

**Option B — Introduce new config/policy file formats (e.g., `config.toml`)**
- **Pros:**
  - Can design a bespoke schema without legacy constraints.
- **Cons:**
  - Adds a second precedence system and new CLI surface requirements.
  - Increased risk of host/world drift and enforcement gaps.
- **Cascading implications:**
  - Requires additional migration/education and cross-ADR coordination.
- **Risks:**
  - High drift risk: different components consult different sources.
- **Unlocks:**
  - None required for Phase 3; mostly organizational.
- **Quick wins / low-hanging fruit:**
  - None compatible with “single precedence model”.

**Recommendation**
- **Selected:** Option A — Reuse existing YAML patch files (new keys only).
- **Rationale (crisp):** Substrate’s audit/enforcement story depends on one coherent config/policy layering model; adding a second file family is a drift multiplier.

**Follow-up tasks (explicit)**
- Add strict schema support for `llm.*` + `agents.*` in both config and policy.
- Update Phase 4/5 ADR drafts to reference ADR-0027 (no `config.toml` claims).

---

### DR-0002 — Backend id format for allowlisting + selection (`<kind>:<name>` vs structured ids)

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`

**Problem / Context**
- Policy allowlists and config selections need a stable string identifier that is:
  - easy to type (CLI dotted updates),
  - stable for trace attribution joins, and
  - future-compatible across CLI and API backends.

**Option A — `<kind>:<name>` string ids**
- **Pros:**
  - Human-typable and easy to represent in YAML + dotted updates.
  - Supports a clean “kind namespace” boundary for future extension.
  - Stable for event attribution and joins.
- **Cons:**
  - Requires documenting an id grammar and enforcing it in schema validation.
- **Cascading implications:**
  - ADRs that define new backends must assign stable ids in this format.
- **Risks:**
  - Potential naming collisions if not centrally documented; mitigated by ADR discipline.
- **Unlocks:**
  - Simple allowlist + selection semantics without maps/objects in v1.
- **Quick wins / low-hanging fruit:**
  - Start with `cli:*` backends and add `api:*` later.

**Option B — Structured ids (maps/objects; e.g., `{ kind, name, version }`)**
- **Pros:**
  - More explicit and can carry versioning inline.
- **Cons:**
  - Not representable via current dotted update CLI without adding a new value format.
  - Higher cognitive overhead in operator workflows.
- **Cascading implications:**
  - Requires additional schema tooling and CLI features.
- **Risks:**
  - Slower adoption; more places for inconsistencies.
- **Unlocks:**
  - Fine-grained versioning, but not needed for Phase 3.
- **Quick wins / low-hanging fruit:**
  - None without additional CLI work.

**Recommendation**
- **Selected:** Option A — `<kind>:<name>` string ids.
- **Rationale (crisp):** It fits the existing patch + dotted-update model and is sufficient for stable attribution and allowlisting.

**Follow-up tasks (explicit)**
- Enforce backend id format in schema validation for new keys.
- Document reserved ids as Phase 4/5 ADRs are finalized.

---

### DR-0003 — Enable gating posture (policy-only enable vs config+policy enable)

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-02-08  
**Status:** Superseded  
**Related docs:** `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

**Problem / Context**
- We need a fail-closed default posture without forcing operators to edit policy for non-enforcement configuration changes.

**Option A — Require BOTH config enable and policy enable**
- **Pros:**
  - Fail-closed by default even if only one surface is edited.
  - Operators can “allow but disable” (policy allows, config disables) without weakening enforcement.
  - Matches the mental model: config selects; policy allows.
- **Cons:**
  - Two flags to reason about during setup.
- **Cascading implications:**
  - Feature entrypoints must check both effective config and effective policy.
- **Risks:**
  - Misconfiguration confusion; mitigated by good error messages + `--explain`.
- **Unlocks:**
  - Workspace-specific enabling without touching global policy (when policy already permits).
- **Quick wins / low-hanging fruit:**
  - Provide one-line setup examples in the contract + playbook.

**Option B — Policy-only enable (config never disables)**
- **Pros:**
  - Single switch for enable posture.
- **Cons:**
  - Forces policy edits for “I want this disabled in this workspace” use cases.
  - Increases risk of accidental enablement if policy is set broadly.
- **Cascading implications:**
  - Feature entrypoints only consult policy, reducing config usefulness.
- **Risks:**
  - Operators may over-permit by policy in order to “get it working”.
- **Unlocks:**
  - Simpler setup at the cost of flexibility.
- **Quick wins / low-hanging fruit:**
  - None compatible with “config selects” goals.

**Recommendation**
- **Selected:** Option A — Require BOTH config enable and policy enable.
- **Rationale (crisp):** It is the most conservative posture and preserves config’s role as a safe “off switch” without weakening policy enforcement.

**Follow-up tasks (explicit)**
- Ensure documentation, playbook, and smoke scripts validate both-side gating expectations.

**Superseded by**
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md` (DR-0004)

---

### DR-0004 — Policy surface for LLM/agents: enable booleans vs requirements + allowlists

**Decision owner(s):** Shell + Broker maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`, `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

**Problem / Context**
- LLM + agents need fail-closed governance while keeping policy focused on enforcement requirements (as in ADR-0018) rather than acting as a second “feature enable” control plane.

**Option A — Keep `policy.llm.enabled` and `policy.agents.enabled`**
- **Pros:**
  - Simple mental model (“policy must also enable the feature”).
  - Explicit policy-owned global kill switch.
- **Cons:**
  - Duplicates enable controls across config and policy and increases operator confusion.
  - Encourages treating policy as “feature flags” rather than enforcement requirements.
- **Cascading implications:**
  - Feature entrypoints must check multiple enable booleans before consulting allowlists/requirements.
- **Risks:**
  - Misconfiguration drift where config/policy disagree about enablement intent.
- **Unlocks:**
  - None that cannot be achieved with allowlists + config enable.
- **Quick wins / low-hanging fruit:**
  - None; it adds keys and complexity without improving enforcement semantics.

**Option B — Policy has requirements/constraints only (no `*.enabled`); config enables/disables**
- **Pros:**
  - Matches the existing posture: config selects/enables; policy constrains/forces requirements (ADR-0018 pattern).
  - Fail-closed by default via deny-by-default allowlists (empty allowlists deny).
  - Keeps “why was this blocked?” explainability crisp: allowlist/requirement mismatches, not dueling enable flags.
- **Cons:**
  - Requires documenting the deny-by-default behavior explicitly (empty allowlist = deny).
- **Cascading implications:**
  - LLM operations require BOTH:
    - `config.llm.enabled=true`, AND
    - `policy.llm.allowed_backends` contains the selected backend, AND
    - `policy.llm.fail_closed.routing` / `policy.llm.require_approval` are satisfied.
  - Agent hub operations require BOTH:
    - `config.agents.enabled=true`, AND
    - `policy.agents.allowed_backends` contains the selected backend(s).
- **Risks:**
  - Operators may forget to populate allowlists; mitigated by actionable errors and `--explain`.
- **Unlocks:**
  - Clear separation of concerns across all future ADRs (gateway, engines, agent hub, agent toolbox (MCP protocol)).
- **Quick wins / low-hanging fruit:**
  - Remove redundant policy enable keys from the schema and keep only requirements + allowlists.

**Recommendation**
- **Selected:** Option B — Policy has requirements/constraints only; config enables/disables.
- **Rationale (crisp):** It matches the repo’s established model (policy expresses requirements, config selects) and keeps fail-closed behavior driven by deny-by-default allowlists rather than duplicated enable flags.

---

### DR-0009 — Per-agent `policy_overlay` eligibility for host credential read gate

**Decision owner(s):** Broker + Engine + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`, `docs/project_management/next/llm_cli_backend_engine/decision_register.md` (DR-0008)

**Problem / Context**
- We introduced `agents.host_credentials.read.allowed_backends` as an explicit policy gate for host credential reads by backend adapters.
- Agent files may include a `policy_overlay`, but it must be restriction-only. We need to decide whether this new gate is eligible for per-agent tightening.

**Option A — Allow `agents.host_credentials.read.allowed_backends` in `policy_overlay` (subset-only)**
- **Pros:** Lets an individual backend tighten the host-credential-read permission even if globally allowed; aligns with “policy overlays only tighten” posture; supports least-privilege per backend.
- **Cons:** Requires defining and enforcing “tighten-only” semantics for list keys (overlay list must be a subset of effective policy list).
- **Cascading implications:** Update the overlay allowlist and implement subset validation (fail closed if overlay attempts to broaden).
- **Risks:** A buggy subset check can become a broadening path; must be tested and must fail closed.
- **Unlocks:** Per-backend hardening without forcing workspace/global policy edits.
- **Quick wins / low-hanging fruit:** `cli:codex` can ship with an overlay that defaults this permission off unless explicitly enabled for that backend.

**Option B — Disallow this key in `policy_overlay` (policy.yaml only)**
- **Pros:** Simpler; fewer overlay interactions; avoids subset validation for this key.
- **Cons:** Coarser control; cannot tighten per backend without editing workspace/global policy.
- **Cascading implications:** Document that host credential read gating is global/workspace only.
- **Risks:** Over-permission at the workspace/global layer becomes harder to contain.
- **Unlocks:** Less schema/validation work now.
- **Quick wins / low-hanging fruit:** None beyond keeping scope smaller.

**Recommendation**
- **Selected:** Option A — Allow `agents.host_credentials.read.allowed_backends` in `policy_overlay` (subset-only)
- **Rationale (crisp):** Host credential reads are security-sensitive and per-backend tightening is exactly what `policy_overlay` is for; subset-only semantics keep it restriction-only.

---

### DR-0010 — Per-agent `policy_overlay` eligibility for secret env injection allowlist

**Decision owner(s):** Broker + Gateway + Security  
**Date:** 2026-02-09  
**Status:** Accepted  
**Related docs:** ADR-0027, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`, `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0014)

**Problem / Context**
- We introduced `llm.secrets.env_allowed` as a policy allowlist of secret env var *names* that Substrate may read on the host and inject into in-world gateway/engine processes.
- Phase 8 additive clarification: this key gates **host env reads** for host→world secret delivery, regardless of whether the in-world gateway/engine receives those secret values via legacy env injection (v1) or via an FD/pipe auth bundle (v1.1).
- Agent files may include a `policy_overlay`, but it must be restriction-only. We need to decide whether this key is eligible for per-agent tightening.

**Option A — Allow `llm.secrets.env_allowed` in `policy_overlay` (subset-only)**
- **Pros:** Allows per-backend least privilege: an individual agent/backend can narrow which secret env var names are injectable for that backend even if the workspace/global policy is broader; matches the established “overlays only tighten” posture.
- **Cons:** Requires subset-only validation for list keys (overlay list must be a subset of the effective policy list).
- **Cascading implications:** Add `llm.secrets.env_allowed` to the overlay allowlist and implement subset checks that fail closed on attempted broadening.
- **Risks:** A bug in subset validation can become a broadening path; must be tested; must fail closed.
- **Unlocks:** Safe multi-backend environments where only some backends may receive certain secrets.
- **Quick wins / low-hanging fruit:** `api:openai` can tighten to only `OPENAI_API_KEY`, while other backends receive none by default.

**Option B — Disallow this key in `policy_overlay` (policy.yaml only)**
- **Pros:** Simpler; fewer overlay interactions; avoids subset validation for this key.
- **Cons:** Coarser control; cannot tighten per-backend without editing workspace/global policy.
- **Cascading implications:** Document that secret env injection allowlists are global/workspace only.
- **Risks:** Over-permission at the workspace/global layer becomes harder to contain.
- **Unlocks:** Less schema/validation work now.
- **Quick wins / low-hanging fruit:** None beyond keeping scope smaller.

**Recommendation**
- **Selected:** Option A — Allow `llm.secrets.env_allowed` in `policy_overlay` (subset-only)
- **Rationale (crisp):** This gate is security-sensitive; per-backend tightening is exactly what overlays are for, and subset-only semantics preserve restriction-only guarantees.

**Follow-up tasks (explicit)**
- Update ADR-0027, `SCHEMA.md`, and `contract.md` to remove `policy.llm.enabled` / `policy.agents.enabled` and to document deny-by-default allowlists.
- Update Phase 4/5 ADR drafts to stop referencing policy enable booleans.

---

### DR-0008 — Agent world fallback control: config defaults vs policy `fail_closed.routing`

**Decision owner(s):** Shell + Broker + Agent Hub maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

**Problem / Context**
- Agents can run either on the host or inside a world boundary. When an agent is configured/routed for in-world execution and a world boundary is not available, the “host fallback vs fail closed” behavior is a security boundary decision (analogous to ADR-0018’s `fail_closed.routing` posture).

**Option A — Config-owned `agents.defaults.execution.world_required` (and/or agent `config.execution.world_required`)**
- **Pros:**
  - Keeps all agent runtime behavior knobs in config.
- **Cons:**
  - Makes a security boundary decision config-owned rather than policy-owned.
  - Reintroduces the misleading `require_world` phrasing, which ADR-0018 is explicitly moving away from.
- **Cascading implications:**
  - Easy to accidentally broaden behavior by switching config defaults (e.g., allow silent host fallback).
- **Risks:**
  - Privilege boundary drift: “routing to host” becomes a config preference rather than an enforced constraint.
- **Unlocks:**
  - None; this option weakens the enforcement boundary posture for host fallback decisions.
- **Quick wins / low-hanging fruit:**
  - No new policy keys required; all configuration remains in config surfaces.

**Option B — Policy-owned `agents.fail_closed.routing` (default true); config selects scope only**
- **Pros:**
  - Aligns naming and semantics with ADR-0018 (`fail_closed.routing`).
  - Keeps the security boundary decision (host fallback) policy-owned and auditable.
  - Still allows agent config to select `execution.scope` (host vs world) without allowing silent broadening when the world is unavailable.
- **Cons:**
  - Requires documenting one additional policy key.
- **Cascading implications:**
  - Per-agent variance is supported by allowing an embedded, restriction-only `policy_overlay` to set `agents.fail_closed.routing=true` (tighten only).
- **Risks:**
  - Operators may over-restrict; mitigated by `--explain` and clear error messages.
- **Unlocks:**
  - Consistent, policy-owned enforcement posture for host fallback decisions across LLM and agents (`fail_closed.routing`).
- **Quick wins / low-hanging fruit:**
  - Reuse an existing policy key shape and semantics already established by ADR-0018.

**Recommendation**
- **Selected:** Option B — Policy-owned `agents.fail_closed.routing`.
- **Rationale (crisp):** World-boundary fallback is a security boundary; it must be policy-owned and expressed with ADR-0018-aligned `fail_closed.routing` semantics rather than config-owned “require world” phrasing.

**Follow-up tasks (explicit)**
- Ensure `SCHEMA.md`, `contract.md`, and ADR-0027 use `agents.fail_closed.routing` and do not introduce `agents.*require_world*` keys.

---

### DR-0005 — Agent definitions: inline config keys vs inventory directory (`~/.substrate/agents/`)

**Decision owner(s):** Shell + Agent Hub maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md`

**Problem / Context**
- Substrate needs to manage multiple agents/backends with stable identities. Encoding arbitrary agent ids into a strict YAML schema via fixed key paths is brittle and does not match the established inventory-directory mental model (ADR-0011).

**Option A — Define all agents inline under `config.yaml` keys**
- **Pros:**
  - Single file to inspect/edit.
- **Cons:**
  - Hard to keep strict schema with dynamic agent ids (either becomes non-strict or requires a fixed registry list).
  - Poor parity with the deps inventory pattern that already exists.
- **Cascading implications:**
  - Encourages “one giant file” and increases merge/conflict risk.
- **Risks:**
  - Schema drift and operational brittleness as the agent set grows.
- **Unlocks:**
  - None; inventory directories solve the same problem more cleanly.
- **Quick wins / low-hanging fruit:**
  - None compatible with strict schema + dynamic ids.

**Option B — Inventory directory: one file per agent under `agents/`**
- **Pros:**
  - Matches the deps model: inventory directory + sparse patch defaults.
  - Natural support for dynamic ids while preserving strict per-file schema.
  - Safer to collaborate: smaller diffs and less merge contention.
- **Cons:**
  - Adds a new inventory directory to document and implement.
- **Cascading implications:**
  - Agent files must follow the same safety rule as deps: filename-derived id must match `id:` inside the YAML.
  - Inventory precedence must mirror existing scope rules:
    - global inventory: `$SUBSTRATE_HOME/agents/` (default `~/.substrate/agents/`)
    - workspace inventory: `<workspace_root>/.substrate/agents/`
- **Risks:**
  - Slightly more discovery logic required; mitigated by `agents list` UX (ADR-0025).
- **Unlocks:**
  - Clean expansion path for “one file per agent backend” across CLI and API agents.
- **Quick wins / low-hanging fruit:**
  - Start with a small reserved set (`codex`, `claude_code`, `gemini_cli`) while keeping the inventory mechanism generic.

**Recommendation**
- **Selected:** Option B — Inventory directory (`agents/`) with one file per agent.
- **Rationale (crisp):** It preserves strict schema while supporting dynamic agent ids and matches an already-accepted inventory pattern (deps).

**Follow-up tasks (explicit)**
- Update ADR-0027 `SCHEMA.md` to include agent file locations and the filename/id match requirement.
- Add an ADR-0025 discussion item to define how agent hub loads and reconciles inventory + defaults.

---

### DR-0006 — Per-agent policy variance: embedded `policy_overlay` vs separate per-agent policy files

**Decision owner(s):** Shell + Broker + Agent Hub maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

**Problem / Context**
- Agents need different restrictions (world FS, commands, egress). We need a per-agent mechanism without creating “policy scattered everywhere” that undermines effective-policy explainability.

**Option A — Separate per-agent policy overlay files**
- **Pros:**
  - Keeps agent config and policy physically separated.
- **Cons:**
  - Introduces another file family and extra indirection.
  - Harder to reason about and to show in `agents list/status` UX without bespoke tooling.
- **Cascading implications:**
  - Requires additional discovery and explain surfaces for overlay resolution.
- **Risks:**
  - Operators may miss overlays and misdiagnose restrictions.
- **Unlocks:**
  - None required; embedded overlays can remain disciplined.
- **Quick wins / low-hanging fruit:**
  - None; more moving parts.

**Option B — Embed a restriction-only `policy_overlay` in each agent file**
- **Pros:**
  - One file per agent contains the complete “what it is + what it is allowed to do” picture.
  - Keeps strict schema (agent file schema is versioned and validated).
  - Easier to audit and render in `agents status` and in trace explain paths.
- **Cons:**
  - Requires clear rules so it doesn’t become a second full policy system.
- **Cascading implications:**
  - The overlay MUST be restriction-only (see DR-0007) and MUST NOT contain secrets.
- **Risks:**
  - Without strict rules, can drift into “policy per agent”; mitigated by restriction-only composition.
- **Unlocks:**
  - Per-agent tightening for `world_fs.*`, `cmd_*`, and `net_allowed` without affecting base policy.
- **Quick wins / low-hanging fruit:**
  - Start by supporting a small subset of policy keys in the overlay and expand only via ADR updates.

**Recommendation**
- **Selected:** Option B — Embedded `policy_overlay` inside each agent file.
- **Rationale (crisp):** It provides per-agent restriction capability while keeping operator auditability and minimizing file-family complexity.

**Follow-up tasks (explicit)**
- Define the allowed key subset and restriction-only composition rules in `SCHEMA.md` (DR-0007).

---

### DR-0007 — Policy overlay semantics: restriction-only AND vs replace/override

**Decision owner(s):** Broker + Agent Hub maintainers  
**Date:** 2026-02-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

**Problem / Context**
- If per-agent policy overlays can broaden permissions, we lose the ability to reason about base policy as the enforcement truth. We need a composition rule that is safe by construction.

**Option A — Replace/override semantics (agent overlay can broaden)**
- **Pros:**
  - Flexible.
- **Cons:**
  - Unsafe: agent configs can silently weaken enforcement.
  - Undercuts policy explainability and the “fail-closed by default” posture.
- **Cascading implications:**
  - Requires complex governance and likely additional policy gates to control policy widening.
- **Risks:**
  - High risk of accidental privilege escalation.
- **Unlocks:**
  - None compatible with safety-first posture.
- **Quick wins / low-hanging fruit:**
  - None; would require extensive guardrails.

**Option B — Restriction-only AND semantics (agent overlay can only tighten)**
- **Pros:**
  - Safe by construction: overlays cannot broaden beyond base policy.
  - Matches ADR-0018’s “deny is a boundary” posture (agent can add deny, not remove).
  - Enables per-agent differences without undermining the broker’s base policy model.
- **Cons:**
  - Requires explicit composition rules per key family.
- **Cascading implications:**
  - Composition is “most restrictive wins”, e.g.:
    - `net_allowed`: effective host allowed iff allowed by BOTH base and overlay.
    - `cmd_denied`: union (deny more).
    - `cmd_allowed`: additional filter (must match BOTH if overlay list is non-empty).
    - `world_fs.*`: overlay may only narrow allow and/or add deny; never expand.
    - `llm.fail_closed.routing`: OR (overlay can require fail-closed routing).
    - `agents.fail_closed.routing`: OR (overlay can require fail-closed routing).
    - `require_approval`: OR (overlay can require approval).
- **Risks:**
  - Operators may over-restrict; mitigated by clear errors and explain surfaces.
- **Unlocks:**
  - Safe differentiation of agents (executor vs orchestrator vs specialist) without a second policy system.
- **Quick wins / low-hanging fruit:**
  - Implement with “evaluate both policies” rather than attempting to compute strict set intersections for complex patterns.

**Recommendation**
- **Selected:** Option B — Restriction-only AND semantics.
- **Rationale (crisp):** It preserves the enforcement story (base policy remains the floor) while allowing safe, per-agent specialization.

**Follow-up tasks (explicit)**
- Document the allowed overlay key subset and composition rules in `SCHEMA.md` and ADR-0027.
- Ensure `--explain` surfaces include whether an allow/deny came from base vs overlay.

---

### DR-0011 — Planning Pack platform scope (Linux+macOS now vs Linux+macOS+Windows now)

**Decision owner(s):** Planning Pack owner(s)  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/active/llm_and_agent_config_policy_surface/plan.md`, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/tasks.json`

**Problem / Context**
- ADR-0027 surfaces are cross-platform by design, but this Planning Pack does not require Windows execution at this time.

**Option A — Require Linux + macOS only for Phase 3 execution**
- **Pros:**
  - Keeps the Phase 3 critical path bounded while still validating the primary behavior platforms.
  - Avoids scheduling Windows runner provisioning and platform-fix loops.
- **Cons:**
  - Windows parity is deferred and must be planned explicitly when it becomes required.
- **Cascading implications:**
  - `tasks.json` sets `meta.behavior_platforms_required=["linux","macos"]` and `meta.ci_parity_platforms_required=["linux","macos"]`.
  - CI checkpoint dispatches smoke for Linux and macOS only.
- **Risks:**
  - Windows-specific issues can surface later; mitigated by keeping schema and CLI surfaces strict and deterministic.
- **Unlocks:**
  - Unblocks downstream LLM/agent ADRs that depend on the surfaces.
- **Quick wins / low-hanging fruit:**
  - Smoke scripts remain deterministic on Linux and macOS via `make feature-smoke`.

**Option B — Require Linux + macOS + Windows for Phase 3 execution**
- **Pros:**
  - Immediate parity validation across all primary desktop platforms.
- **Cons:**
  - Adds coordination cost and runner availability constraints to Phase 3.
  - Requires platform-fix loops and smoke gates for Windows.
- **Cascading implications:**
  - `tasks.json` must include Windows in behavior and CI parity platforms and include a Windows platform-fix task at the checkpoint boundary.
- **Risks:**
  - Execution stalls when Windows runners are unavailable.
- **Unlocks:**
  - Earlier Windows signal.
- **Quick wins / low-hanging fruit:**
  - None; it expands the critical path.

**Recommendation**
- **Selected:** Option A — Require Linux + macOS only for Phase 3 execution.
- **Rationale (crisp):** The Phase 3 objective is to land strict schema + inventory surfaces; Linux+macOS validation is sufficient for current execution needs.

---

### DR-0012 — Cross-platform execution model for this Planning Pack (schema v4 boundary-only + bounded CI checkpoints)

**Decision owner(s):** Planning Pack owner(s)  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`

**Problem / Context**
- The execution plan must enforce deterministic triad structure, bounded cross-platform validation cadence, and a platform-fix model with a bounded task footprint.

**Option A — Schema v4 boundary-only platform-fix + bounded CI checkpoints**
- **Pros:**
  - Cross-platform smoke dispatch is bounded and auditable via `ci_checkpoint_plan.md`.
  - Platform-fix tasks exist only at checkpoint boundaries, minimizing task count.
  - Mechanical validation enforces boundary wiring (`meta.checkpoint_boundaries`).
- **Cons:**
  - Platform-specific issues can be discovered at the checkpoint boundary instead of immediately after every slice.
- **Cascading implications:**
  - `tasks.json` uses `meta.schema_version=4`, `meta.cross_platform=true`, and `meta.checkpoint_boundaries=[...]`.
- **Risks:**
  - Larger checkpoint groups can accumulate drift; mitigated by keeping checkpoint sizes within bounds.
- **Unlocks:**
  - Fast iteration with preserved safety gates.
- **Quick wins / low-hanging fruit:**
  - Single checkpoint at Phase 3 completion.

**Option B — Schema v3 per-slice platform-fix + per-slice cross-platform dispatch**
- **Pros:**
  - Platform issues surface earlier (slice-by-slice).
- **Cons:**
  - High dispatch cost and high coordination overhead for a schema-heavy feature.
  - Task counts grow quickly (core + per-platform + aggregator per slice).
- **Cascading implications:**
  - `tasks.json` becomes larger and harder to execute deterministically.
- **Risks:**
  - Validation becomes the dominant cost and slows the feature’s critical path.
- **Unlocks:**
  - None required for Phase 3.
- **Quick wins / low-hanging fruit:**
  - None; it expands coordination immediately.

**Recommendation**
- **Selected:** Option A — Schema v4 boundary-only platform-fix + bounded CI checkpoints.
- **Rationale (crisp):** It preserves safety with bounded validation cadence and bounded task footprint.

---

### DR-0013 — Agent inventory validation surface (`substrate agents validate` vs implicit-only validation)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`, `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`

**Problem / Context**
- Phase 3 introduces a new operator-visible file surface: the agent inventory directory (`$SUBSTRATE_HOME/agents/` and `<workspace_root>/.substrate/agents/`).
- The feature requires deterministic validation of strict parsing and restriction-only `policy_overlay` semantics.

**Option A — Add `substrate agents validate` (explicit validation command)**
- **Pros:**
  - Deterministic, operator-invokable validation for agent inventory strictness and overlay broadening rejection.
  - Enables smoke scripts to validate the new surface without requiring downstream gateway/hub behavior.
  - Produces actionable errors that point at the invalid file path.
- **Cons:**
  - Adds a new CLI surface that must remain stable.
- **Cascading implications:**
  - Define the command’s exit codes and error posture in `contract.md` and `LACP1-spec.md`.
- **Risks:**
  - None beyond maintaining a small, stable validation command.
- **Unlocks:**
  - Safe iterative development of gateway/hub features that depend on the inventory directory.
- **Quick wins / low-hanging fruit:**
  - Smoke scripts validate inventory/overlay without additional runtime features.

**Option B — Validate agent inventory only when agent routing/execution occurs**
- **Pros:**
  - No new CLI surface.
- **Cons:**
  - Validation is not reachable until later Phase 4/5 features exist, which blocks deterministic Phase 3 smoke.
  - Failures are coupled to unrelated runtime behavior.
- **Cascading implications:**
  - Phase 3 acceptance criteria cannot be validated via smoke scripts.
- **Risks:**
  - Higher risk of landing invalid inventory files that only fail at runtime.
- **Unlocks:**
  - None required for Phase 3.
- **Quick wins / low-hanging fruit:**
  - None; it defers validation.

**Recommendation**
- **Selected:** Option A — Add `substrate agents validate`.
- **Rationale (crisp):** It provides deterministic validation and smoke coverage for a new operator-visible file surface without requiring downstream gateway/hub behavior.
