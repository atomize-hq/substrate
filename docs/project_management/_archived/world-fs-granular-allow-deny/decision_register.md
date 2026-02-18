# Decision Register — world-fs-granular-allow-deny

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

Scope:
- This decision register supports `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — Breaking schema versioning (no compatibility)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/PROTOCOL.md`

**Problem / Context**
- The existing `world_fs.read_allowlist` / `world_fs.write_allowlist` model cannot express deny-overrides-allow safely, and the current system has a foot-gun where invalid patterns (notably `..`) can be accepted but silently ignored during allowlist resolution.
- The feature requires a contract where “configured but not enforced” is structurally impossible.

**Option A — V2 breaking schema (hard error legacy)**
- **Pros:**
  - Eliminates “accepted but ignored” states by rejecting legacy keys and invalid patterns at validation time.
  - Makes protocol/schema boundaries explicit (`PolicySnapshotV2` only), simplifying auditing.
- **Cons:**
  - Requires lockstep updates (shell + world-agent) and forces operator migration.
- **Cascading implications:**
  - Broker must reject legacy keys and invalid patterns with exit code `2`.
  - World-agent must reject `PolicySnapshotV1` and any unknown fields with HTTP `400` / WS fatal error frame.
- **Risks:**
  - Upgrade friction and temporary operator disruption if rollout is not coordinated.
- **Unlocks:**
  - Enables deny lists + strict enforcement as a security boundary without compatibility shims.
- **Quick wins / low-hanging fruit:**
  - Simplifies implementation by removing compatibility branches and dual-schema handling.

**Option B — Compatibility shims (preserve old keys; add new optional keys)**
- **Pros:**
  - Lower immediate rollout friction; older configs keep working.
- **Cons:**
  - Reintroduces ambiguous states (operators can configure policy that appears set but is not enforced across all paths).
  - Requires long-lived dual parsing/normalization logic across broker/shell/world-agent.
- **Cascading implications:**
  - Must define precedence between legacy and new keys and ensure every path enforces identically.
- **Risks:**
  - Security posture regression via silent downgrade/ignore paths.
- **Unlocks:**
  - Phased migration at the cost of complexity.
- **Quick wins / low-hanging fruit:**
  - None that preserve the ADR’s security posture.

**Recommendation**
- **Selected:** Option A — V2 breaking schema (no compatibility).
- **Rationale (crisp):** This feature’s safety requirements cannot tolerate “accepted but not enforced” policy; compatibility shims are structurally at odds with that.

**Follow-up tasks (explicit)**
- Implement/validate V2 policy schema (broker): `WFGAD0-code`, `WFGAD0-test`, `WFGAD0-integ`
- Implement/validate V2 protocol + snapshot model rejection behavior: `WFGAD1-code`, `WFGAD1-test`, `WFGAD1-integ-*`, `CP1-ci-checkpoint`, `WFGAD1-integ`

---

### DR-0002 — Deny-overrides-allow mechanism (mount masking vs LSM/interposition)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SECURITY.md`

**Problem / Context**
- Landlock is allowlist-only. Once broad allows exist (e.g., `.`), it cannot express “allow all except X”.

**Option A — Mount masking inside the per-command mount namespace**
- **Pros:**
  - Works with the existing full isolation architecture (per-command mount namespace chokepoint).
  - Can subtract visibility/access for specific paths even when broad allows exist.
- **Cons:**
  - Requires additional hardening (strict mode) to prevent the workload from undoing mounts.
  - Path coverage must be carefully applied to all nameable project views.
- **Cascading implications:**
  - Helper must apply deny masks after mounts exist and before user code executes.
  - Denied read/discover must surface as `EACCES`; denied writes must surface as `EROFS`.
- **Risks:**
  - Without strict mode, deny masks are bypassable by adversarial workloads.
- **Unlocks:**
  - Enables deny semantics immediately without requiring system-wide LSM policy.
- **Quick wins / low-hanging fruit:**
  - Leverages existing mount namespace boundary and helper chokepoint.

**Option B — LSM deny policy (AppArmor/SELinux) or syscall interposition**
- **Pros:**
  - Potentially stronger system-level enforcement if universally available and configured.
- **Cons:**
  - Not portable across environments; requires external policy distribution and privileged setup.
  - Syscall interposition is brittle and hard to make correct across all IO paths.
- **Cascading implications:**
  - Would require new provisioning surfaces, new docs, and platform-specific enablement/guards.
- **Risks:**
  - Inconsistent operator experience across machines; difficult to support in a repo that targets multiple OSes.
- **Unlocks:**
  - A future “enterprise hardening” path, but not within this ADR’s scope.
- **Quick wins / low-hanging fruit:**
  - None aligned with current architecture.

**Recommendation**
- **Selected:** Option A — mount masking inside the per-command mount namespace.
- **Rationale (crisp):** It is the only viable mechanism consistent with the current architecture that can express deny exceptions under broad allows.

**Follow-up tasks (explicit)**
- Deny masking implementation + wildcard snapshot semantics: `WFGAD3-code`, `WFGAD3-test`, `WFGAD3-integ-*`, `CP2-ci-checkpoint`, `WFGAD3-integ`

---

### DR-0003 — Deny enforcement posture (strict vs best_effort via explicit lever)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (authoritative decision detail), `docs/project_management/_archived/world-fs-granular-allow-deny/SECURITY.md`

**Problem / Context**
- Deny masks are only a security boundary if the workload cannot undo them after startup, but some workloads require mount operations inside the world.

**Option A — Strict-only denies**
- **Pros:**
  - Strong security by default; denies are always a hard boundary.
  - Fewer configuration states.
- **Cons:**
  - Breaks mount-dependent workflows even when operators accept the risk.
- **Cascading implications:**
  - Must reject any attempt to request best-effort behavior.
- **Risks:**
  - Operators avoid deny lists entirely due to compatibility needs.
- **Unlocks:**
  - Simplified enforcement implementation.
- **Quick wins / low-hanging fruit:**
  - Fewer tests for posture branching.

**Option B — Policy lever (`world_fs.enforcement=strict|best_effort`)**
- **Pros:**
  - Preserves strict as a security boundary while still allowing compatibility workflows.
  - Forces explicit operator choice (no silent downgrade).
- **Cons:**
  - Adds an operator foot-gun: best-effort denies are not a security boundary under adversarial workloads.
- **Cascading implications:**
  - Schema must require `enforcement` iff any deny_list is non-empty, and enforce fail-closed strict prerequisites when `require_world=true`.
- **Risks:**
  - Misconfiguration to best-effort when strict security is expected.
- **Unlocks:**
  - Incremental adoption path without blocking mount-dependent tools.
- **Quick wins / low-hanging fruit:**
  - None; requires clear docs and validation.

**Recommendation**
- **Selected:** Option B — policy lever (intentionally supports both `strict` and `best_effort` as an operator choice).
- **Rationale (crisp):** Strict security and workflow compatibility are both required; the explicit lever makes the tradeoff intentional and enforceable.

**Follow-up tasks (explicit)**
- Schema/validation wiring: `WFGAD0-code`, `WFGAD0-test`, `WFGAD0-integ`
- Strict lockdown security boundary: `WFGAD5-code`, `WFGAD5-test`, `WFGAD5-integ-*`, `CP3-ci-checkpoint`, `WFGAD5-integ`

---

### DR-0004 — Deny support scope (full isolation only)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/world-fs-granular-allow-deny/contract.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`

**Problem / Context**
- `world_fs.isolation=workspace` is not a full pivot-root view and does not provide a reliable chokepoint for deny semantics; ignoring keys would create false security expectations.

**Option A — Deny lists only in `world_fs.isolation=full`**
- **Pros:**
  - Keeps the deny semantics within the architecture that can actually enforce them.
  - Enables hard errors for unsupported modes, preventing silent ignore.
- **Cons:**
  - Deny semantics are unavailable for workspace isolation users.
- **Cascading implications:**
  - Schema must hard error if deny/enforcement/read/discover/write keys are present under workspace isolation.
- **Risks:**
  - Users may need to migrate to full isolation for this feature.
- **Unlocks:**
  - Clear, enforceable contract with no “best effort” ambiguity for workspace mode.
- **Quick wins / low-hanging fruit:**
  - Avoids implementation effort for an unreliable mode.

**Option B — Deny lists in both `full` and `workspace`**
- **Pros:**
  - Broader availability.
- **Cons:**
  - Likely unenforceable in adversarial model; high risk of partial enforcement and drift.
- **Cascading implications:**
  - Requires defining what “deny” means in workspace mode and proving it is enforceable.
- **Risks:**
  - Creates a false sense of protection if enforcement is incomplete.
- **Unlocks:**
  - None aligned with the ADR’s fail-closed posture.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — deny lists only in full isolation.
- **Rationale (crisp):** Workspace isolation cannot reliably enforce the deny semantics; hard errors are required to prevent false security assumptions.

**Follow-up tasks (explicit)**
- Schema validation and hard-error behavior: `WFGAD0-code`, `WFGAD0-test`, `WFGAD0-integ`

---

### DR-0005 — Add `discover` dimension (directory visibility)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/manual_testing_playbook.md`

**Problem / Context**
- Operators need “visible but not readable” and “can traverse known paths without listing” behaviors without conflating directory listing and file reads.

**Option A — Add `discover` dimension (defaults to mirror `read`)**
- **Pros:**
  - Expresses directory visibility as a separate knob from read access.
  - Provides explicit defaulting rule (mirror read), avoiding ambiguous behavior.
- **Cons:**
  - Requires additional enforcement plumbing and tests (Landlock and/or mount semantics).
- **Cascading implications:**
  - Schema must define defaulting: if `discover` omitted, it equals `read`.
  - ENV/protocol must carry discover allowlist if distinct.
- **Risks:**
  - Increased implementation complexity in low-level enforcement.
- **Unlocks:**
  - Enables a stable operator contract for directory visibility.
- **Quick wins / low-hanging fruit:**
  - Clear manual test case (“visible but not readable”) becomes expressible and automatable.

**Option B — Keep `read` as a single bundled dimension**
- **Pros:**
  - Fewer knobs; simpler surface.
- **Cons:**
  - Cannot express “visible but not readable” deterministically.
- **Cascading implications:**
  - Operators must choose between over-exposing content or breaking tooling relying on listing/traversal.
- **Risks:**
  - Forces policy workarounds and inconsistent behavior between listing and read needs.
- **Unlocks:**
  - None for the requested visibility semantics.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — add `discover` dimension with default mirror of `read`.
- **Rationale (crisp):** Directory visibility must be expressible independently from file reads to satisfy the operator contract.

**Follow-up tasks (explicit)**
- Implement/validate discover/read split: `WFGAD4-code`, `WFGAD4-test`, `WFGAD4-integ`

---

### DR-0006 — Wildcard deny semantics (`**/*.pem`) guarantee

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SECURITY.md`

**Problem / Context**
- Operators want wildcard denies, but the system must not overpromise “always denied” behavior within a long-running process.

**Option A — Snapshot-at-exec-start semantics**
- **Pros:**
  - Deterministic, simple contract: enumerate matches at exec boundary and mask them before user code runs.
  - Avoids reliance on kernel deep features (fanotify/inotify) for correctness.
- **Cons:**
  - Does not guarantee blocking of within-process create/rename+read after exec.
- **Cascading implications:**
  - Schema/docs must state the non-guarantee explicitly.
  - Helper must not follow symlinks during scan for determinism.
- **Risks:**
  - Some operators may expect stronger semantics than provided.
- **Unlocks:**
  - Wildcard support without a new daemon or kernel requirements.
- **Quick wins / low-hanging fruit:**
  - Enables simple deny patterns like `**/*.pem`.

**Option B — “Always denied” via watchers or deep kernel features**
- **Pros:**
  - Stronger guarantee in principle.
- **Cons:**
  - Complex, brittle, and platform/kernel dependent.
  - Hard to make deterministic and performant.
- **Cascading implications:**
  - Requires new runtime components and substantially more testing and threat modeling.
- **Risks:**
  - Overly complex system with subtle bypasses and performance regressions.
- **Unlocks:**
  - Stronger semantics, but out of scope for this ADR.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — snapshot-at-exec-start semantics.
- **Rationale (crisp):** Determinism and honest guarantees are required; watcher-based approaches are too complex and non-portable for this feature scope.

**Follow-up tasks (explicit)**
- Implement/validate wildcard snapshot semantics: `WFGAD3-code`, `WFGAD3-test`, `WFGAD3-integ-*`, `CP2-ci-checkpoint`, `WFGAD3-integ`

---

### DR-0007 — Wildcard resolution location (helper vs host/world-agent)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/world-fs-granular-allow-deny/ENV.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`

**Problem / Context**
- Wildcard resolution must happen against the authoritative in-namespace filesystem view that will be executed, after mounts exist, and must fail closed.

**Option A — Resolve inside the helper (post-mount, per exec)**
- **Pros:**
  - Helper sees the final in-namespace filesystem state.
  - Aligns with “deny masks applied before user code” and fail-closed posture.
- **Cons:**
  - Requires helper to implement scanning logic and validation.
- **Cascading implications:**
  - Enforcement plan env contract must carry deny patterns to the helper.
  - Helper must validate pattern grammar and fail closed on parse/validation errors.
- **Risks:**
  - Helper complexity increases; must be carefully tested.
- **Unlocks:**
  - Deterministic enforcement at the final chokepoint.
- **Quick wins / low-hanging fruit:**
  - Avoids host-side path translation and mount timing issues.

**Option B — Resolve on the host or in world-agent service (pre-mount)**
- **Pros:**
  - Keeps helper smaller.
- **Cons:**
  - Cannot reliably see the final mount namespace view; risks mismatch between resolved paths and executed view.
  - Higher risk of “accepted but not enforced” due to timing differences.
- **Cascading implications:**
  - Would require additional protocols to communicate resolved match sets and their mapping to in-world paths.
- **Risks:**
  - Enforcement drift and incorrect masking.
- **Unlocks:**
  - None aligned with the “final chokepoint” principle.
- **Quick wins / low-hanging fruit:**
  - None.

**Recommendation**
- **Selected:** Option A — resolve inside the helper.
- **Rationale (crisp):** Enforcement must occur at the final chokepoint with the authoritative in-namespace view; pre-mount resolution cannot guarantee correctness.

**Follow-up tasks (explicit)**
- Helper enforcement plan + deny scanning: `WFGAD2-code`, `WFGAD2-test`, `WFGAD2-integ`, `WFGAD3-code`, `WFGAD3-test`, `WFGAD3-integ-*`

---

### DR-0008 — Strict-mode bypass prevention mechanism (cap drop + seccomp)

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-01-29  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/world-fs-granular-allow-deny/ENV.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/SECURITY.md`

**Problem / Context**
- Mount-based deny masks are bypassable if the workload retains the ability to call mount/umount/remount APIs after startup.

**Option A — Drop mount authority and deny mount-family syscalls before exec**
- **Pros:**
  - Makes denies a hard security boundary under the stated threat model.
  - Covers both legacy and newer mount syscalls when present.
- **Cons:**
  - Requires careful implementation to avoid killing processes (`EPERM`, not `SIGSYS`).
  - Can break workloads that require mount operations (mitigated by best_effort mode).
- **Cascading implications:**
  - Must run after deny masks are applied and before `exec`.
  - Must apply to the entire workload process tree.
- **Risks:**
  - Kernel compatibility and syscall availability differences require careful gating.
- **Unlocks:**
  - True strict-mode security boundary for deny lists.
- **Quick wins / low-hanging fruit:**
  - Aligns with existing helper chokepoint.

**Option B — Convention-only (document “don’t mount/umount”) or Landlock-only**
- **Pros:**
  - Minimal implementation work.
- **Cons:**
  - Not a security boundary; adversarial workloads can bypass.
  - Landlock cannot express deny exceptions under broad allows.
- **Cascading implications:**
  - Would require weakening the security posture of the feature.
- **Risks:**
  - Violates ADR contract (“strict deny” becomes misleading).
- **Unlocks:**
  - None aligned with strict deny semantics.
- **Quick wins / low-hanging fruit:**
  - None that preserve the security guarantee.

**Recommendation**
- **Selected:** Option A — cap drop + seccomp deny of mount-family syscalls.
- **Rationale (crisp):** Without explicit bypass prevention, strict deny cannot be a security boundary.

**Follow-up tasks (explicit)**
- Strict lockdown implementation and validation: `WFGAD5-code`, `WFGAD5-test`, `WFGAD5-integ-*`, `CP3-ci-checkpoint`, `WFGAD5-integ`
