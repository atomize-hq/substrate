# Decision Register — Full Isolation Landlock ↔ OverlayFS Compatibility

This decision register scopes decisions required to restore the `world_fs.write_allowlist` contract
in `world_fs.isolation=full` when Linux Landlock enforcement is enabled and the active filesystem
strategy uses overlayfs backing dirs (`upperdir` / `workdir`).

Related ADR:
- `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

---

### DR-0001 — Full Isolation: Landlock Runtime Allowlist for OverlayFS Backing Dirs

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

**Problem / Context**
- In full isolation on Linux, the command process runs under an applied Landlock policy.
- When using overlayfs, project writes that appear to target `/project/<prefix>` are serviced via
  overlayfs backing dirs (`upperdir` / `workdir`).
- If Landlock does not allow the overlay backing dirs, allowlisted project writes fail with `EPERM`
  even when mount-level allowlist remounting is correct.
- This runtime state must not be represented in the policy snapshot schema/hash (it is per-session).

**Option A — Derive overlay backing dirs at runtime via procfs mount inspection**
- **Pros:** No policy surface area expansion; works with policy snapshots; robust under version skew; keeps runtime-derived state out of snapshot hashes.
- **Cons:** Requires Linux-only parsing logic for `/proc/self/mountinfo` (preferred) or `/proc/self/mounts`; needs careful error messaging.
- **Cascading implications:** World-agent Landlock wrapper must read mount metadata and extend allowlists dynamically.
- **Risks:** Incorrect parsing could over-allow or under-allow; mitigate with fixture tests and fail-closed behavior.
- **Unlocks:** Generalizes to other filesystem strategies that have internal write roots (future).
- **Quick wins / low-hanging fruit:** Implement a helper in `crates/world` and reuse it from the Landlock exec wrapper.

**Option B — Plumb overlay backing dirs via explicit env vars from the world backend**
- **Pros:** Avoids procfs parsing; uses already-known overlay state paths from the backend.
- **Cons:** Requires new env var contracts and additional plumbing across execution paths (non-PTY + PTY); increases “ambient” runtime surface area.
- **Cascading implications:** Requires keeping env propagation consistent across platforms/backends; requires versioning if treated as a stable contract.
- **Risks:** Env propagation drift could silently break enforcement; needs strong invariants and tests.
- **Unlocks:** Enables backends to provide strategy-specific internal roots without procfs dependence.
- **Quick wins / low-hanging fruit:** Add env var(s) only as an internal implementation detail, not as a documented stable interface.

**Recommendation**
- **Selected:** Option A — Derive overlay backing dirs at runtime via procfs mount inspection
- **Rationale (crisp):** Overlay backing dirs are session-specific runtime state; procfs-based derivation keeps the policy snapshot deterministic while restoring allowlisted writability under Landlock.

**Follow-up tasks (explicit)**
- Implement Linux-only mount inspection helper (prefer `/proc/self/mountinfo`) in `crates/world`.
- Extend `crates/world-agent/src/internal_exec.rs` to include derived overlay backing dirs in the full isolation Landlock policy write allowlist.
- Add fixture-based unit tests for mount parsing and an integration test gated on Landlock support that asserts allowlisted writes succeed under full isolation.

---

### DR-0002 — Where overlay mountinfo parsing lives (world crate helper vs world-agent local)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-spec.md`, `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

**Problem / Context**
- The Landlock exec wrapper must derive overlayfs `upperdir` and `workdir` for the project mountpoint to extend the Landlock write allowlist.
- Multiple components can benefit from mountinfo parsing utilities; duplicating parsing logic increases drift risk.

**Option A — Add a Linux-only helper in `crates/world` (selected)**
- **Pros:** Single implementation reused by world-agent and future world components; enables fixture tests in one crate; keeps Linux-only parsing behind `#[cfg]`.
- **Cons:** Adds a helper API surface to `crates/world` that must remain minimal and well-scoped.
- **Cascading implications:** `crates/world-agent` depends on `crates/world` for mountinfo parsing logic used by the Landlock wrapper.
- **Risks:** If the helper API grows beyond mountinfo parsing, it can become a grab bag; mitigate by limiting scope to overlay backing dir derivation.
- **Unlocks:** A shared implementation point for future “strategy internal roots” derivations.
- **Quick wins / low-hanging fruit:** Implement mountinfo parsing as a pure function over text fixtures and reuse it in world-agent.

**Option B — Implement mountinfo parsing only inside `crates/world-agent`**
- **Pros:** Keeps the helper local to the one caller; avoids adding any new helper surface to `crates/world`.
- **Cons:** Duplicates Linux parsing utilities if other crates need them later; increases drift risk across exec paths (PTY vs non-PTY).
- **Cascading implications:** Any future caller must either duplicate parsing or move it later under time pressure.
- **Risks:** Two independent parsers can diverge and produce different allowlists under the same mount topology.
- **Unlocks:** A smaller immediate change footprint for this slice.
- **Quick wins / low-hanging fruit:** Directly parse `/proc/self/mountinfo` in the wrapper.

**Recommendation**
- **Selected:** Option A — Add a Linux-only helper in `crates/world`
- **Rationale (crisp):** A single mountinfo parser reduces drift risk and enables fixture-based correctness tests in one place.

**Follow-up tasks (explicit)**
- `C0-code`: add the Linux-only mountinfo helper in `crates/world` and use it from the Landlock exec wrapper.
- `C0-test`: add fixture-based tests for the helper in `crates/world`.

---

### DR-0003 — Landlock allowlist scope for overlayfs backing dirs (exact dirs vs broader parent)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-spec.md`, `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

**Problem / Context**
- The overlayfs implementation requires writes to its `upperdir` and `workdir` for allowlisted project writes to succeed.
- The Landlock write allowlist extension must be as narrow as possible while reliably enabling overlayfs writes.

**Option A — Allow exactly `upperdir` and `workdir` (selected)**
- **Pros:** Minimal permission expansion; scopes the allowlist to session-derived internal roots; aligns with fail-closed posture.
- **Cons:** If overlayfs behavior changes to require additional internal paths, the allowlist will need an explicit update.
- **Cascading implications:** The mountinfo parser must return both `upperdir` and `workdir` and the wrapper must add both.
- **Risks:** Under-allowing causes false-deny failures; mitigate with the Linux smoke and full isolation integration tests.
- **Unlocks:** A general “derive exact internal roots” pattern for other filesystem strategies.
- **Quick wins / low-hanging fruit:** Add these two paths only and verify allowlisted writes succeed.

**Option B — Allow the parent overlay state directory (e.g., the enclosing `/var/lib/substrate/overlay/<WORLD_ID>/...`)**
- **Pros:** More resilient if overlayfs uses additional internal files under the same state dir.
- **Cons:** Broader permission than necessary; increases the blast radius of the Landlock allowlist in the event the path becomes nameable.
- **Cascading implications:** Requires careful derivation of the “correct” parent directory and guardrails to avoid accidentally allowing `/var/lib/substrate/overlay` broadly.
- **Risks:** Over-allowing weakens hardening and can mask unintended nameability of internal paths.
- **Unlocks:** Fewer future updates if overlayfs internal structure changes.
- **Quick wins / low-hanging fruit:** One allowlist entry instead of two.

**Recommendation**
- **Selected:** Option A — Allow exactly `upperdir` and `workdir`
- **Rationale (crisp):** Narrow, session-scoped allowlists restore correctness without expanding the Landlock surface beyond what overlayfs requires.

**Follow-up tasks (explicit)**
- `C0-code`: extend the Landlock write allowlist with `upperdir` and `workdir` only.
- `C0-test`: add fixture cases that verify both keys are required and errors are deterministic when either is missing.

---

### DR-0004 — Failure mode when Landlock is supported but overlay backing dirs cannot be derived (fail closed vs degrade)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-spec.md`, `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

**Problem / Context**
- If Landlock is supported but the wrapper cannot derive overlay backing dirs, allowlisted writes will fail with `EPERM`.
- A partial or silent fallback undermines the operator contract and can introduce security ambiguity.

**Option A — Fail closed (selected)**
- **Pros:** Preserves the operator contract; avoids unsafe “best-effort” behavior under full isolation; produces a high-signal diagnostic.
- **Cons:** Introduces a new hard failure mode for environments with unexpected mount topologies.
- **Cascading implications:** The wrapper must surface a deterministic, actionable error and block exec.
- **Risks:** Operators in unusual environments hit failures; mitigated by diagnostics and by limiting to the “Landlock supported + full isolation + writable + overlay” case.
- **Unlocks:** Prevents silent “allowlist says writable but runtime denies” drift.
- **Quick wins / low-hanging fruit:** Treat missing mountinfo data as a missing prerequisite and return early.

**Option B — Degrade by skipping the Landlock extension and proceeding**
- **Pros:** Avoids new hard failures.
- **Cons:** Leaves allowlisted writes broken (still `EPERM`), making the system appear inconsistent; can mask the real cause.
- **Cascading implications:** The wrapper must decide between “apply Landlock” and “skip Landlock”, which complicates security reasoning.
- **Risks:** Silent drift between mount allowlist semantics and Landlock enforcement persists.
- **Unlocks:** None for this feature’s contract.
- **Quick wins / low-hanging fruit:** Keep existing behavior.

**Recommendation**
- **Selected:** Option A — Fail closed
- **Rationale (crisp):** Full isolation requires a deterministic enforcement story; failing closed prevents unsafe drift and forces actionable diagnostics.

**Follow-up tasks (explicit)**
- `C0-code`: return an actionable “missing prerequisites” error when derivation fails under the defined conditions.
- `C0-test`: add a test case that asserts a deterministic error path for missing `upperdir`/`workdir` in fixtures.

---

### DR-0005 — Cross-platform task scope (Linux behavior-only vs full behavior parity)

**Decision owner(s):** Substrate core team  
**Date:** 2026-01-20  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/full-isolation-landlock-overlayfs-compat/tasks.json`, `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/*`

**Problem / Context**
- This feature changes Linux full-isolation behavior because it depends on Linux Landlock and overlayfs mount semantics.
- The repository still requires compile parity on macOS and Windows.

**Option A — Linux is the only behavior platform; macOS/Windows are CI parity only (selected)**
- **Pros:** Matches the feature’s Linux-only behavior contract; avoids requiring macOS/Windows runners to validate Linux kernel behavior.
- **Cons:** Does not run a meaningful behavioral smoke on macOS/Windows for this feature.
- **Cascading implications:** `tasks.json` uses `behavior_platforms_required=["linux"]` and `ci_parity_platforms_required=["linux","macos","windows"]`.
- **Risks:** None for this feature’s defined contract; compile parity still guards cross-platform builds.
- **Unlocks:** Keeps smoke focused on the platform where the behavior exists.
- **Quick wins / low-hanging fruit:** Use a single Linux smoke script that reproduces the allowlisted-write failure and validates the fix.

**Option B — Treat Linux/macOS/Windows as behavior platforms**
- **Pros:** Forces smoke runs on all host OSes.
- **Cons:** The relevant Landlock+overlayfs behavior runs in a Linux kernel; macOS and Windows jobs cannot validate the Linux-only behavior without additional backend/provisioning scope.
- **Cascading implications:** Requires meaningful smoke scripts on all three platforms, which is not aligned with the feature’s scope.
- **Risks:** Adds noise and false confidence if non-Linux smoke scripts do not exercise the behavior.
- **Unlocks:** None for this feature.
- **Quick wins / low-hanging fruit:** None.

**Recommendation**
- **Selected:** Option A — Linux is the only behavior platform; macOS/Windows are CI parity only
- **Rationale (crisp):** The behavior change is Linux-kernel-specific; cross-platform validation is compile parity only.

**Follow-up tasks (explicit)**
- `docs/project_management/next/full-isolation-landlock-overlayfs-compat/tasks.json`: keep `meta.behavior_platforms_required=["linux"]` and `meta.ci_parity_platforms_required=["linux","macos","windows"]`.
- `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh`: exit `0` as a defined no-op.
- `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/windows-smoke.ps1`: exit `0` as a defined no-op.
