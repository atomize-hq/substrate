# World Deps Selection Layer — Decision Register

This is the single source of truth for **all** architectural decisions required to implement ADR-0002 (“install classes + provisioning-time system packages”) without ad-hoc patches and without regressing existing platforms.

## Non-negotiable rules
- Every decision below has **exactly two** viable options (Option A / Option B).
- Each decision ends with **one** selected option (no TBDs, no “optional” framing).
- Each option includes: pros, cons, cascading implications, risks, unlocks, and quick wins.

Related docs (authoritative inputs):
- `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md` (D1/D2)
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`
- `docs/project_management/next/world-sync/C0-spec.md` (workspace init/gating)
- `docs/project_management/next/world-sync/C2-spec.md` (filters/protected paths patterns)

Specs (this triad):
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

---

### DR-0001 — Selection config file locations + precedence

**Decision owner(s):** Shell/World maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0001 D1/D2, Y0-spec, C0-spec, `docs/WORLD.md`

**Problem / Context**
- World-deps must be selection-driven and no-op when unconfigured (ADR-0001 D1).
- We need a YAML-only selection surface that works for teams (workspace) and individuals (global) without confusing overlays.

**Option A — Workspace-only selection**
- **Pros:**
  - No ambiguity: the workspace fully defines what is managed.
  - Simplifies support: one file to inspect.
  - Aligns with world-sync’s `.substrate/` workspace model (C0).
- **Cons:**
  - Poor DX for users who want a default selection across many repos.
  - Forces copying selection files between projects.
- **Cascading implications:**
  - `substrate world deps` becomes tightly coupled to `substrate init` (workspace creation).
  - On ephemeral repos, users will repeatedly reconfigure selection.
- **Risks:**
  - Pushes users back to ad-hoc manual installs because config friction is too high.
- **Unlocks:**
  - Cleanest future integration with workspace templates and team policies.
- **Quick wins / low-hanging fruit:**
  - Minimal loader logic (single path lookup).

**Option B — Workspace + global selection, with workspace override**
- **Pros:**
  - Team workflows: repo can commit a workspace selection; users can also have a personal default.
  - Matches existing precedence mental model (dir settings override global settings).
  - Enables “configure once” (global) while still allowing workspace specificity.
- **Cons:**
  - Slightly more complex UI: must explain precedence clearly.
  - Must surface which file is active in `status --json` for debuggability.
- **Cascading implications:**
  - Requires spec-defined, deterministic precedence (no merging; no partial overlays).
  - Adds a new “configured” state: global configured but workspace not.
- **Risks:**
  - Users may be surprised when a workspace selection shadows their global selection; mitigated by explicit status output.
- **Unlocks:**
  - Smooth transition from personal experimentation to team-committed selections.
- **Quick wins / low-hanging fruit:**
  - `substrate world deps init --global` is immediately useful without requiring `substrate init`.

**Recommendation**
- **Selected:** Option B — Workspace + global selection, workspace overrides global
- **Rationale (crisp):** It preserves ADR-0001’s “selection is explicit” while keeping setup friction low across repos; override-only precedence avoids confusing merges while remaining fully deterministic.

**Follow-up tasks (explicit)**
- Implement lookup + precedence in `S0`.
- Ensure `status --json` includes `selection.active_path` and `selection.shadowed_paths`.

---

### DR-0002 — Selection config filename (avoid overlay confusion)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0001 D2

**Problem / Context**
- ADR-0001 D2 forbids confusing “same base name, one-character extension differences”.
- We already have overlay files: `scripts/substrate/world-deps.yaml` and `~/.substrate/world-deps.local.yaml`.

**Option A — `world-deps.yaml` for selection**
- **Pros:**
  - Short and memorable.
  - Works as a “primary” config name.
- **Cons:**
  - Too close to `world-deps.local.yaml`; high support burden.
  - Collides conceptually with the existing `world-deps.yaml` overlay naming convention.
- **Cascading implications:**
  - Must add additional UX scaffolding to explain selection vs overlays repeatedly.
- **Risks:**
  - Human error: editing the wrong file is likely.
- **Unlocks:**
  - None that a clearer name doesn’t also unlock.
- **Quick wins / low-hanging fruit:**
  - No meaningful benefit beyond brevity.

**Option B — `world-deps.selection.yaml` for selection**
- **Pros:**
  - Unambiguous: the filename encodes intent (selection, not overlay).
  - Avoids collisions with installed/user overlay conventions.
  - Eases onboarding (“create selection file” is literal).
- **Cons:**
  - Slightly longer filename.
- **Cascading implications:**
  - CLI/docs should consistently refer to this name.
- **Risks:**
  - Minimal.
- **Unlocks:**
  - Enables future additional surfaces (`world-deps.classes.yaml`, etc.) without confusion.
- **Quick wins / low-hanging fruit:**
  - Cleaner error messages and support instructions.

**Recommendation**
- **Selected:** Option B — `world-deps.selection.yaml`
- **Rationale (crisp):** Meets ADR-0001 D2’s “don’t confuse users” requirement and avoids future schema/name collisions.

**Follow-up tasks (explicit)**
- Use `.substrate/world-deps.selection.yaml` and `~/.substrate/world-deps.selection.yaml` (DR-0001).

---

### DR-0003 — Selection config schema shape (YAML-only, minimal, strict)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** Y0-spec (YAML-only), ADR-0001 D1 (selection gate)

**Problem / Context**
- Selection is a user/ops input. It must be easy to hand-edit, strict enough to validate, and stable under YAML-only constraints.

**Option A — Minimal strict allowlist**
- **Pros:**
  - Simple mental model: “selected tools are the only scope”.
  - Easy to validate and produce actionable errors.
  - Avoids policy-like complexity (no includes/excludes logic).
- **Cons:**
  - Cannot express advanced selection logic (wildcards, groups).
- **Cascading implications:**
  - Discovery is handled via `status --all` rather than schema features.
- **Risks:**
  - If inventory grows large, selecting many items becomes verbose (acceptable).
- **Unlocks:**
  - Fast, low-risk implementation.
- **Quick wins / low-hanging fruit:**
  - `world deps select` can be a tiny command that edits a list.

**Option B — Expressive schema (groups, wildcards, include/exclude)**
- **Pros:**
  - Powerful and compact for large inventories.
  - Better fit for future org policy controls.
- **Cons:**
  - Harder to validate and explain; increases support burden.
  - Risk of “ad-hoc patches” as edge cases appear.
- **Cascading implications:**
  - More complex CLI tooling required to manage the schema safely.
- **Risks:**
  - Ambiguous outcomes if rules conflict; violates “zero ambiguity” constraint.
- **Unlocks:**
  - Future policy-style selection at scale.
- **Quick wins / low-hanging fruit:**
  - None (implementation complexity is the opposite of a quick win).

**Recommendation**
- **Selected:** Option A — Minimal strict allowlist
- **Rationale (crisp):** We need correctness + clear UX first; expressiveness increases ambiguity and risk without being required by ADR-0002.

**Follow-up tasks (explicit)**
- Implement schema: `version: 1`, `selected: [<tool_name>...]` (case-insensitive compare; normalized output is lower-case).
- Enforce: unknown tools in selection are a configuration error (exit code per DR-0010).

---

### DR-0004 — Behavior when selection is missing (no-op requirement)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0001 D1, ADR-0002 constraints

**Problem / Context**
- World-deps must not surprise-install; selection missing must be treated as “not configured”.
- ADR-0001 D1 explicitly requires `status/install/sync` to print “not configured” and exit successfully.

**Option A — No-op + exit 0 for `status|sync|install|provision`**
- **Pros:**
  - Exactly matches ADR-0001 D1.
  - Safe default in agent-hub threat model (no unexpected changes).
  - “Greenfield” friendly: forces explicit selection as the first step.
- **Cons:**
  - Users who run `install <tool>` expecting action will get no-op; mitigated by explicit messaging.
- **Cascading implications:**
  - CLI must always show “how to configure selection” in this state.
  - JSON output must represent “not configured” explicitly (not as empty tool list only).
- **Risks:**
  - Users might miss that nothing happened if output is too subtle; mitigated via a single prominent line.
- **Unlocks:**
  - Enables safe auto-invocation by tools/agents without causing system changes.
- **Quick wins / low-hanging fruit:**
  - Unconfigured-state smoke tests are straightforward and stable.

**Option B — Exit non-zero for `sync/install` when selection missing**
- **Pros:**
  - More conventional: “I asked to install; error if config missing.”
- **Cons:**
  - Violates ADR-0001 D1.
  - Encourages people to add `--force` flags or ad-hoc bypasses.
- **Cascading implications:**
  - Harder to use in automated contexts; causes noisy CI failures.
- **Risks:**
  - Increased support burden; users view world-deps as brittle.
- **Unlocks:**
  - None that ADR-0001 allows.
- **Quick wins / low-hanging fruit:**
  - None (contradicts required behavior).

**Recommendation**
- **Selected:** Option A — No-op + exit 0 when selection is missing
- **Rationale (crisp):** ADR-0001 requires it, and it is the safest agent-hub posture.

**Follow-up tasks (explicit)**
- Define a single, consistent message and JSON field `selection.configured=false`.
- Ensure “no-op” includes “no world backend calls” (important for hardening/caging and for platform parity).

---

### DR-0005 — `--all` semantics (inventory vs ignore selection)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** `docs/WORLD.md` (current behavior), ADR-0001 D1 (selection gate)

**Problem / Context**
- Current `--all` means “include host-missing tools”; selection-layer needs a new definition.
- The UX must remain predictable and selection-driven, but still support discovery/debugging and “install everything” when explicitly requested.

**Option A — `--all` ignores selection and uses the full inventory as the operation scope**
- **Pros:**
  - Clear and powerful: “all tools in inventory” is unambiguous.
  - Enables discovery (`status --all`) and bulk install (`sync --all`) without editing selection.
  - Keeps selection as the default safety rail (no `--all`, no surprises).
- **Cons:**
  - A user can bypass selection intentionally; must be very explicit in output.
- **Cascading implications:**
  - `--all` becomes “dangerous but explicit”; must still respect install class enforcement (system packages never at runtime).
  - Unconfigured selection still no-ops (DR-0004); `--all` does not override “not configured”.
- **Risks:**
  - Users may run `sync --all` accidentally; mitigated by strong output + requiring `--all` to be spelled out (no default).
- **Unlocks:**
  - Simple discovery flow: `init` → `status --all` → `select`.
- **Quick wins / low-hanging fruit:**
  - Reuses existing flag name; minimal CLI churn.

**Option B — `--all` expands *display* only; install scope remains selection**
- **Pros:**
  - Preserves selection as the only install scope (strong safety).
  - `status --all` works for discovery, while `sync/install` remain guarded.
- **Cons:**
  - Breaks the existing mental model of `sync --all` meaning “install anyway”.
  - Requires either removing `--all` from `sync/install` or redefining it inconsistently across subcommands.
- **Cascading implications:**
  - Adds documentation burden (“`--all` means different things depending on subcommand”).
- **Risks:**
  - Confusing UX; increases support issues.
- **Unlocks:**
  - None that cannot be achieved with a separate flag later.
- **Quick wins / low-hanging fruit:**
  - Minimal runtime risk, but at the cost of clarity.

**Recommendation**
- **Selected:** Option A — `--all` ignores selection and uses inventory as the scope
- **Rationale (crisp):** It’s the only definition that stays consistent across `status|sync|install` and supports discovery + explicit bulk operations, while selection remains the default guardrail.

**Follow-up tasks (explicit)**
- Update help text and sample outputs to explicitly say: “`--all` ignores selection and uses inventory scope”.
- Ensure install-class enforcement still blocks `system_packages` at runtime even under `--all`.

---

### DR-0006 — Where install class metadata lives (single source of truth)

**Decision owner(s):** Shell/Manifest maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0002 (install classes), existing manager manifest layering

**Problem / Context**
- Install class must be explicit for each tool and support overrides via existing layering (inventory + overlays).
- We must avoid drift between “what recipes do” and “what the system believes they do”.

**Option A — Embed class metadata in the layered manager manifest**
- **Pros:**
  - Single source of truth: tool definition, detect, and install metadata stay together.
  - Overlays naturally override class metadata without new mechanisms.
  - Allows the shell to compute UX + enforcement using the same loaded manifest.
- **Cons:**
  - Requires a manifest schema bump and updating existing entries (breaking change).
- **Cascading implications:**
  - `crates/common/src/manager_manifest/schema.rs` and validation rules must change.
  - Tools that currently use `apt` recipes must be reclassified as `system_packages` (or redesigned as `user_space`).
- **Risks:**
  - Large manifest edit PRs if done all at once; mitigated by applying the change to the Substrate-owned manifest in one coordinated slice.
- **Unlocks:**
  - Enables automated “provision system packages” based on selection (no separate mapping).
- **Quick wins / low-hanging fruit:**
  - Immediate detection of unsafe recipes (e.g., `apt-get` in a `user_space` tool) via validation.

**Option B — Separate install-class mapping file (tool → class)**
- **Pros:**
  - Avoids touching the manager manifest schema immediately.
  - Class metadata can evolve independently.
- **Cons:**
  - Two sources of truth; drift is inevitable.
  - Requires a new overlay/precedence model, increasing complexity.
- **Cascading implications:**
  - More files to validate and explain; harder troubleshooting.
- **Risks:**
  - “Ad-hoc patch” pressure when mapping and recipes disagree.
- **Unlocks:**
  - Incremental adoption (but we are greenfield, so this is not required).
- **Quick wins / low-hanging fruit:**
  - Faster prototype only, not a maintainable architecture.

**Recommendation**
- **Selected:** Option A — Embed class metadata in the layered manager manifest
- **Rationale (crisp):** It eliminates drift by construction and uses existing overlay semantics instead of inventing a second configuration stack.

**Follow-up tasks (explicit)**
- Extend manifest schema to include `guest_install.class` and (for system packages) `guest_install.system_packages.*`.
- Update Substrate’s manifest entries to declare class explicitly.

---

### DR-0007 — How to represent system packages (script vs structured list)

**Decision owner(s):** Shell/Installer maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0002 (provisioning-time packages), hardening constraints

**Problem / Context**
- `system_packages` must be selection-driven and safe under agent-hub hardening. It must no-op when unconfigured and be auditable.

**Option A — Structured list of packages (per package manager)**
- **Pros:**
  - Safe: Substrate constructs the package install command; manifests can’t hide arbitrary privileged shell.
  - Auditable: easy to diff, review, and log.
  - Cross-platform extendable: can add `apt`, `apk`, `dnf`, etc without changing execution semantics.
- **Cons:**
  - Less flexible for exotic setup steps.
- **Cascading implications:**
  - Requires the shell to implement package-manager-specific installers (starting with apt only).
- **Risks:**
  - If guest distros change, package names may drift; mitigated by pinning guest distros in provisioning (Lima/WSL) and versioning manifests.
- **Unlocks:**
  - Deterministic “repair/upgrade” command (re-run is idempotent).
- **Quick wins / low-hanging fruit:**
  - Start with Ubuntu/Debian guests: `apt` only.

**Option B — Free-form privileged shell script**
- **Pros:**
  - Maximum flexibility.
  - Mirrors today’s `guest_install.apt` pattern.
- **Cons:**
  - Unsafe under agent-hub threat model: scripts can do arbitrary privileged actions.
  - Hard to reason about and review; encourages ad-hoc patches.
- **Cascading implications:**
  - Increases attack surface; conflicts with hardening trajectory (pivot_root/Landlock).
- **Risks:**
  - A “system packages” script becomes a backdoor for system mutation beyond packages.
- **Unlocks:**
  - None worth the risk (we can add explicit “manual steps” for exotic cases).
- **Quick wins / low-hanging fruit:**
  - Faster initial migration, but at unacceptable security cost.

**Recommendation**
- **Selected:** Option A — Structured lists (`system_packages.apt: [..]`, etc.)
- **Rationale (crisp):** It is the only safe, reviewable representation aligned with an agent-hub threat model and hardening.

**Follow-up tasks (explicit)**
- Implement `apt` installer in `S2` and treat other managers as unsupported with explicit errors.

---

### DR-0008 — Where user-space world-deps installs live (prefix + cage compatibility)

**Decision owner(s):** Shell/World maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** I2/I3 (full cage), `normalize_env_for_linux_guest` (macOS)

**Problem / Context**
  - User-space installs must be compatible with full caging (pivot_root) and Landlock (when enabled).
- Installs must not pollute the project tree nor depend on host-specific `$HOME` behavior.

**Option A — Fixed world-owned prefix: `/var/lib/substrate/world-deps`**
- **Pros:**
  - Stable and backend-agnostic (Linux host, Lima guest, WSL).
  - Works with cage: can be explicitly mounted RW as part of the “full cage” rootfs (I2/I3).
  - Avoids leaking host toolchains via `$HOME` or `/Users/...` mappings on macOS.
- **Cons:**
  - On Linux host backend, this is a host path and therefore “host mutation” (but in a dedicated, Substrate-owned directory).
- **Cascading implications:**
  - Full-cage rootfs must mount this path RW; Landlock allowlists must include it for world-deps operations.
  - Recipes must install into `${SUBSTRATE_WORLD_DEPS_ROOT}` and expose binaries via `${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR}`.
- **Risks:**
  - If the directory is not present/mounted, installs fail; mitigated by provisioning and doctor checks.
- **Unlocks:**
  - Deterministic, clean state management and uninstall/cleanup workflows later.
- **Quick wins / low-hanging fruit:**
  - Reuses existing macOS PATH normalization default (`/var/lib/substrate/world-deps/bin`).

**Option B — Per-user HOME-based prefix (`$HOME/.substrate/world-deps`)**
- **Pros:**
  - Avoids `/var/lib` writes on Linux host.
  - Familiar “user-space install” location.
- **Cons:**
  - `$HOME` is unstable/virtualized under future caging and on macOS guest normalization.
  - Encourages mixing Substrate-managed state with user-managed tool state.
- **Cascading implications:**
  - Cage must mount `$HOME` RW consistently, increasing complexity.
- **Risks:**
  - Drift/bugs when `$HOME` differs between host/guest (especially macOS).
- **Unlocks:**
  - Minimal.
- **Quick wins / low-hanging fruit:**
  - Smaller rootfs changes, but at the cost of long-term determinism.

**Recommendation**
- **Selected:** Option A — `/var/lib/substrate/world-deps`
- **Rationale (crisp):** It is deterministic across worlds and aligns with caging (explicit RW mount) rather than relying on unstable `$HOME` semantics.

**Follow-up tasks (explicit)**
- Define env vars: `SUBSTRATE_WORLD_DEPS_ROOT=/var/lib/substrate/world-deps` and `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=/var/lib/substrate/world-deps/bin`.
- Add hardening-track requirement: I2/I3 full cage must mount `/var/lib/substrate` RW (at least the world-deps subdir).

---

### DR-0009 — System packages install strategy (provisioning command surface)

**Decision owner(s):** Shell/Installer/World maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0002 (provisioning-time route), scripts (`scripts/wsl/provision.sh`)

**Problem / Context**
- `system_packages` cannot run implicitly during `world deps sync/install`.
- We need an explicit, idempotent “repair/upgrade” route that works cross-platform where technically possible.

**Option A — Dedicated CLI: `substrate world deps provision`**
- **Pros:**
  - One UX across platforms: same command path, same semantics.
  - Selection-driven: only installs packages required by selected tools.
  - Idempotent: safe to rerun; doubles as “repair”.
  - Compatible with future full cage: provisioning can be performed outside normal tool install flow.
- **Cons:**
  - Requires implementing a small provisioning runner and platform gating.
- **Cascading implications:**
  - Must add explicit failure messaging on Linux host backend (see DR-0010).
- **Risks:**
  - If run accidentally, it mutates guest packages; mitigated by it being explicit and selection-driven.
- **Unlocks:**
  - Clean path for “system packages needed for managed tooling” on Lima/WSL without runtime installs.
- **Quick wins / low-hanging fruit:**
  - Start with apt-only support for current Ubuntu guests.

**Option B — Provisioning only via external scripts (`lima-warm.sh`, `wsl-warm.ps1`)**
- **Pros:**
  - No new CLI surface.
  - Keeps OS mutation in scripts that already do provisioning.
- **Cons:**
  - Not selection-driven by default; scripts don’t know workspace selection.
  - Harder to run “repair/upgrade” from the CLI; inconsistent UX across platforms.
  - Increases coupling between installer scripts and runtime config.
- **Cascading implications:**
  - Requires script plumbing to read selection files and compute packages—messy and error-prone.
- **Risks:**
  - Encourages per-platform ad-hoc behavior; violates “worlds should feel the same”.
- **Unlocks:**
  - None that can’t be achieved with the CLI.
- **Quick wins / low-hanging fruit:**
  - Faster initially, but creates long-term maintenance debt.

**Recommendation**
- **Selected:** Option A — `substrate world deps provision`
- **Rationale (crisp):** It is the only approach that is selection-driven, consistent across platforms, and compatible with the hardening trajectory.

**Follow-up tasks (explicit)**
- Implement provisioning runner in `S2` and extend platform smoke scripts to cover it.

---

### DR-0010 — Linux host policy for `system_packages` (host mutation posture)

**Decision owner(s):** Shell/World/Installer maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0002 (explicit posture required), agent hub threat model

**Problem / Context**
- On Linux, the world-agent runs on the host. Installing “system packages” would mutate the host OS.
- Under an agent-hub threat model, Substrate must not silently escalate or mutate host state beyond explicit operator intent.

**Option A — Provide an opt-in “mutate host packages” command**
- **Pros:**
  - Delivers parity: same `provision` command works on Linux too.
  - Reduces manual steps for Linux users.
- **Cons:**
  - Extremely high blast radius: a single command would change the host OS package set.
  - Requires additional approval/policy gates (and careful audit logging).
- **Cascading implications:**
  - Must integrate with broker approval flows and potentially OS-specific privilege escalation behaviors.
- **Risks:**
  - Becomes an attractive escalation primitive for compromised tools/users.
- **Unlocks:**
  - Full automation parity on Linux, but at unacceptable security cost for P0.
- **Quick wins / low-hanging fruit:**
  - None; this is not a “quick win” safely.

**Option B — Forbid host package mutation; treat Linux `system_packages` as manual guidance**
- **Pros:**
  - Safest default: avoids OS-level mutation by Substrate on Linux hosts.
  - Compatible with hardening goals and reduces attack surface.
  - Still supports explicit, actionable UX: list packages and the user’s distro command snippet.
- **Cons:**
  - Less automation parity than Lima/WSL.
- **Cascading implications:**
  - `substrate world deps provision` must error on Linux with explicit guidance (not a silent no-op).
- **Risks:**
  - Some tools may remain blocked until user installs packages manually; mitigated by clear output and docs.
- **Unlocks:**
  - Keeps P0 implementation shippable without introducing an approval system expansion.
- **Quick wins / low-hanging fruit:**
  - Provide copy/paste commands for apt/dnf/pacman (best-effort) without executing them.

**Recommendation**
- **Selected:** Option B — Forbid host package mutation; manual guidance on Linux
- **Rationale (crisp):** It is the only posture aligned with an agent-hub threat model without adding a new privileged execution/approval subsystem.

**Follow-up tasks (explicit)**
- In `S2`, make `world deps provision` exit non-zero on Linux with a clear “unsupported (would mutate host)” message and package list.

---

### DR-0011 — Exit code taxonomy for world-deps (explicit, stable categories)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** ADR-0002 (explicit failure modes), JSON mode track (future structured I/O)

**Problem / Context**
- We need consistent failure handling across platforms and shells; errors must be scriptable.

**Option A — Keep “exit 1 for most errors”**
- **Pros:**
  - Minimal implementation effort.
  - Matches common Rust error patterns with `anyhow`.
- **Cons:**
  - Not actionable for automation (cannot distinguish config vs backend vs unsupported).
  - Harder to test/verify specific failure modes across platforms.
- **Cascading implications:**
  - Future JSON mode would still need an error taxonomy; pushing this later creates churn.
- **Risks:**
  - “Worlds should feel the same” degrades when automation can’t tell what failed.
- **Unlocks:**
  - None.
- **Quick wins / low-hanging fruit:**
  - None beyond “do nothing”.

**Option B — Define explicit, stable exit codes for world-deps commands**
- **Pros:**
  - Makes failures scriptable and testable.
  - Allows consistent cross-platform behavior even when implementations differ.
  - Provides a clean mapping for future JSON-mode structured errors.
- **Cons:**
  - Requires some plumbing to map errors to codes.
- **Cascading implications:**
  - Specs must define when we warn+continue vs fail-closed, and codes for each.
- **Risks:**
  - If poorly documented, could confuse; mitigated by including the mapping in `S0` and in `status --json`.
- **Unlocks:**
  - Automatable acceptance matrix (see `S2` automation section).
- **Quick wins / low-hanging fruit:**
  - Add a small `WorldDepsExit` enum and map common failures.

**Recommendation**
- **Selected:** Option B — Explicit, stable exit codes
- **Rationale (crisp):** The ADR requires explicit, actionable failures, and we need automation parity across platforms; explicit exit codes are the simplest stable contract.

**Follow-up tasks (explicit)**
- Implement exit codes as specified in `S0`/`S2` and add smoke tests that assert them.

---

### DR-0012 — Manifest schema version bump strategy (inventory/overlays → v2)

**Decision owner(s):** Common manifest + Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`

**Problem / Context**
- Install classes (and provisioning-time system packages) must be expressed in the layered manager manifest so routing/enforcement is deterministic and reviewable (DR-0006/DR-0007).
- We are greenfield for this track: no backwards-compat layers or “dual schema” shims are allowed.

**Option A — Bump manifest `version` to `2` and require new keys**
- **Pros:**
  - Enforces correctness: every tool with `guest_install` must declare its class explicitly.
  - Prevents silent misclassification of legacy recipes (e.g., hidden `apt-get` mutation).
  - Simplifies implementation and validation (single schema; fail fast with actionable errors).
- **Cons:**
  - Requires updating Substrate-owned manifests in the same change window (breaking).
- **Cascading implications:**
  - `crates/common/src/manager_manifest/schema.rs` validation must reject `version: 1` manifests when used by the new world-deps codepath.
  - Installer/shipped manifests must be updated in lockstep.
- **Risks:**
  - If a user has an old overlay, world-deps will fail to load it; mitigated by explicit error messages and the greenfield policy (no migration).
- **Unlocks:**
  - Clean foundation for future classes and safe provisioning metadata without adding compat debt.
- **Quick wins / low-hanging fruit:**
  - Immediate “unsafe manifest” detection at load time instead of runtime surprises.

**Option B — Keep manifest `version: 1` and add optional keys**
- **Pros:**
  - Smaller apparent diff in the manifest files.
- **Cons:**
  - Violates greenfield constraint in practice: optional keys imply implicit defaults and ambiguous behavior.
  - Makes it hard to guarantee “no OS mutation at runtime” because legacy keys remain valid.
- **Cascading implications:**
  - Requires complex fallback rules and continuous support burden.
- **Risks:**
  - “Ad-hoc patch” pressure when legacy + new fields combine in unexpected ways.
- **Unlocks:**
  - None consistent with the stated constraints.
- **Quick wins / low-hanging fruit:**
  - None worth the ambiguity introduced.

**Recommendation**
- **Selected:** Option A — Bump manifest to `version: 2` and require new keys
- **Rationale (crisp):** Greenfield means we prefer a clean, strict schema that fails fast over a permissive schema that accumulates ambiguity and security risk.

**Follow-up tasks (explicit)**
- Update `S1` to reference this decision explicitly and to define the v2 validation rules.
- Update Substrate-owned manifests (`config/manager_hooks.yaml`, `scripts/substrate/world-deps.yaml`) to `version: 2` during WDL1.

---

### DR-0013 — Semantics for “configured but empty selection” (post `world deps init`)

**Decision owner(s):** Shell maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`

**Problem / Context**
- `substrate world deps init` creates a valid selection file with an empty `selected: []`.
- We must define whether this state triggers any world/backend work and what user-visible behavior is.

**Option A — Treat “configured + empty” as an error**
- **Pros:**
  - Forces users to immediately select tools; prevents “I ran sync and nothing happened” confusion.
- **Cons:**
  - Makes `init` immediately create a failing state (bad UX).
  - Adds noise in automation where repos intentionally keep selection empty by default.
- **Cascading implications:**
  - `status/sync/provision` would return non-zero even though configuration is valid.
- **Risks:**
  - Encourages bypass flags or ad-hoc file edits to “fix” the error.
- **Unlocks:**
  - None needed for ADR-0002.
- **Quick wins / low-hanging fruit:**
  - None (it creates friction, not value).

**Option B — Treat “configured + empty” as a no-op with explicit message**
- **Pros:**
  - `init` is immediately safe and useful (creates a valid scaffold).
  - Avoids unnecessary world-agent calls when there is no work to do.
  - Matches the “selection-driven/no surprises” posture.
- **Cons:**
  - Users may wonder why nothing happened; mitigated by a clear message and next steps (`select`, `status --all`).
- **Cascading implications:**
  - `sync/provision` can succeed (`exit 0`) without requiring backend readiness in this state.
- **Risks:**
  - Minimal; messaging is sufficient.
- **Unlocks:**
  - Enables teams to commit an empty selection scaffold and let individuals opt in.
- **Quick wins / low-hanging fruit:**
  - Simplest implementation and easiest to test (no backend calls).

**Recommendation**
- **Selected:** Option B — No-op + explicit message
- **Rationale (crisp):** An empty selection is valid configuration and should not force backend work; “do nothing but explain” is the lowest-risk, most predictable contract.

**Follow-up tasks (explicit)**
- Update `S0` to explicitly state that `sync/provision` make **no world-agent calls** when selection is empty (unless `--all` is used).
- Add tests + smoke checks that assert no backend calls / exit `0` in this state.

---

### DR-0014 — `system_packages` “present” detection strategy (package-aware vs probe-based)

**Decision owner(s):** Shell + manifest maintainers  
**Date:** 2025-12-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`, `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

**Problem / Context**
- We need one unambiguous, testable contract for when a `system_packages` tool is considered “present” in the guest.
- This contract drives `world deps status` output and determines whether `world deps sync` is blocked by missing system prerequisites.

**Option A — Package-aware detection (dpkg-query/dpkg -s for apt guests)**
- **Pros:**
  - Direct: verifies the exact package set is installed.
  - Independent of whether any specific binary is on PATH.
- **Cons:**
  - Couples “present” semantics to a specific package manager and guest OS family.
  - Requires implementing and maintaining multiple query strategies for different distros/package managers.
- **Cascading implications:**
  - `system_packages` class would need per-provider detection logic and explicit behavior on unsupported guests.
- **Risks:**
  - Drifts toward ad-hoc per-platform behavior, reducing “worlds should feel the same”.
- **Unlocks:**
  - Precise “package parity” checks in the future (if we ever need them).
- **Quick wins / low-hanging fruit:**
  - Works quickly for Ubuntu guests only, but does not generalize cleanly.

**Option B — Probe-based detection (guest_detect is authoritative and deterministic)**
- **Pros:**
  - Cross-platform contract: “present” means the probe command succeeds.
  - Simple to implement and reason about; no package-manager coupling.
  - Makes `system_packages` satisfaction align with real-world needs (binaries needed by recipes).
- **Cons:**
  - Indirect: a package could be installed but the chosen probe might not reflect it if misconfigured.
- **Cascading implications:**
  - `system_packages` tools must define `guest_detect.command` explicitly (no default).
  - Manifest review must ensure the probe covers the packages’ intended effect.
- **Risks:**
  - If the probe is too weak, `sync` may proceed even though some packages are missing; mitigated by requiring probes that cover the actual prerequisites.
- **Unlocks:**
  - Keeps provisioning logic (S2) and detection logic (S1) decoupled and stable as we add more platforms.
- **Quick wins / low-hanging fruit:**
  - Immediate: for apt guests, use probes like `command -v gcc && command -v make` to validate toolchain prerequisites.

**Recommendation**
- **Selected:** Option B — Probe-based detection (`guest_detect.command` is authoritative)
- **Rationale (crisp):** It provides one portable contract across worlds without entangling `present` semantics with distro/package-manager specifics, and it is directly automatable via `status --json` assertions.

**Follow-up tasks (explicit)**
- Update `S1` to require `guest_detect.command` for `system_packages` tools and to define sync satisfaction based on it.
- Update `S2` to state that provisioning does not define “present”; detection remains probe-based.
- Update the manual testing playbook to assert `system_packages` becomes `present` after `provision` by checking `status --json` fields.
