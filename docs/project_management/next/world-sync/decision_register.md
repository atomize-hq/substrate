# Decision Register — world-sync

Template standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

### DR-0001 — CLI namespace for world-sync commands

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/contract.md`, `docs/project_management/next/world-sync/WS0-spec.md`

**Problem / Context**
- The repo already has `substrate world deps sync`. A top-level `substrate sync` would collide in both user mental model and help output.

**Option A — Top-level commands (`substrate sync|checkpoint|rollback`)**
- **Pros:** shortest commands; matches legacy world-sync pack naming.
- **Cons:** collides with `world deps sync`; creates ambiguous “sync what?” UX; would require either renaming `world deps sync` or accepting a confusing contract.
- **Cascading implications:** docs, completions, and users must disambiguate two different “sync” concepts.
- **Risks:** long-term contract confusion; unstable help output/aliases.
- **Unlocks:** none that cannot be achieved under workspace namespace.
- **Quick wins / low-hanging fruit:** minimal.

**Option B — Workspace namespace (`substrate workspace sync|checkpoint|rollback`)**
- **Pros:** avoids collisions; aligns with existing `workspace init|enable|disable`; scopes semantics to workspace root.
- **Cons:** slightly longer commands.
- **Cascading implications:** requires adding new `WorkspaceAction` variants and handler plumbing.
- **Risks:** none material.
- **Unlocks:** coherent “workspace-scoped state” story consistent with ADR-0008.
- **Quick wins / low-hanging fruit:** easy to document and validate.

**Recommendation**
- **Selected:** Option B — Workspace namespace (`substrate workspace …`)
- **Rationale (crisp):** avoids existing collisions and matches the repo’s established command taxonomy.

**Follow-up tasks (explicit)**
- Implement CLI parsing + handler routing per `WS0-*` tasks.

### DR-0002 — Sync “source of truth” (what `workspace sync` applies)

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/filesystem-semantics-spec.md`, `docs/project_management/next/world-sync/WS1-spec.md`, `docs/project_management/next/world-sync/WS2-spec.md`

**Problem / Context**
- World execution produces `fs_diff` records (per-span) and may have a persistent world session. `workspace sync` must have a singular, operator-usable meaning.

**Option A — Span-addressed sync (`workspace sync --span SPAN_ID`)**
- **Pros:** precise targeting; naturally maps to trace spans.
- **Cons:** requires additional persistent state to discover “last span”; user-visible span ids are high-friction; span-local diffs do not model “pending” changes across a session.
- **Cascading implications:** adds a new user-facing identifier surface and new “span selection” UX.
- **Risks:** users sync the wrong span; repeated sync requires tracking many ids.
- **Unlocks:** post-fact sync of historical runs.
- **Quick wins / low-hanging fruit:** none; complexity is front-loaded.

**Option B — Session-addressed pending sync (apply the current session’s pending overlay changes)**
- **Pros:** matches user intent (“apply what changed in the world since I started”); no span-id UX; aligns with persistent world sessions.
- **Cons:** requires a deterministic definition of “pending changes” and a deterministic “clear after apply” rule.
- **Cascading implications:** world backend must expose “pending diff discovery” and (later) apply semantics.
- **Risks:** requires careful clearing/atomicity to avoid double-apply.
- **Unlocks:** auto-sync is naturally defined (apply pending changes on session close).
- **Quick wins / low-hanging fruit:** clean CLI UX from day 1.

**Recommendation**
- **Selected:** Option B — Session-addressed pending sync
- **Rationale (crisp):** minimizes user friction and defines a stable model for both manual and auto sync.

**Follow-up tasks (explicit)**
- Implement pending diff discovery (WS1) and apply+clear semantics (WS2/WS5).

### DR-0003 — Protected path set for sync/checkpoint/rollback

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/filesystem-semantics-spec.md`, `crates/shell/src/execution/config_model.rs`

**Problem / Context**
- Protected excludes are injected into `sync.exclude` and represent the minimum safety boundary for all filesystem mutation workflows.

**Option A — Keep legacy `.substrate-git/**` in the protected set**
- **Pros:** matches older docs and the legacy world-sync pack.
- **Cons:** conflicts with ADR-0008 (workspace state is unified under `.substrate/` and `.substrate-git` is legacy); introduces an unnecessary third protected root.
- **Cascading implications:** inconsistent with current `PROTECTED_EXCLUDES`.
- **Risks:** drift between docs and implementation.
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** none.

**Option B — Protected excludes are exactly `[".git/**", ".substrate/**"]`**
- **Pros:** matches current implementation (`PROTECTED_EXCLUDES`) and ADR-0008; covers internal git because it lives under `.substrate/`.
- **Cons:** none.
- **Cascading implications:** legacy `.substrate-git/` is ignored and receives no special treatment beyond “not referenced”.
- **Risks:** none.
- **Unlocks:** fewer path edge cases.
- **Quick wins / low-hanging fruit:** simplest deterministic contract.

**Recommendation**
- **Selected:** Option B — exactly `[".git/**", ".substrate/**"]`
- **Rationale (crisp):** aligns the world-sync contract with the repo’s current safety boundary and directory layout.

**Follow-up tasks (explicit)**
- Ensure all world-sync filtering logic uses `PROTECTED_EXCLUDES` semantics (WS2/WS5/WS6/WS7).

### DR-0004 — Conflict detection model for sync apply

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/filesystem-semantics-spec.md`, `docs/project_management/next/world-sync/WS2-spec.md`, `docs/project_management/next/world-sync/WS5-spec.md`

**Problem / Context**
- Sync apply needs a deterministic notion of “conflict” that does not require scanning or hashing the entire workspace.

**Option A — Content-hash snapshot at session start**
- **Pros:** most correct; detects true “both sides changed” conflicts.
- **Cons:** expensive (hashing) or complex (sparse snapshot metadata); requires persistent storage of snapshot metadata; complicates cross-platform backend implementations.
- **Cascading implications:** introduces a new state file or internal DB surface that must be specified and versioned.
- **Risks:** performance regressions; subtle snapshot drift bugs.
- **Unlocks:** future higher-fidelity merges.
- **Quick wins / low-hanging fruit:** none.

**Option B — Session-start mtime heuristic**
- **Pros:** cheap; no extra persisted state; deterministic rule: “host changed after session start” is a conflict.
- **Cons:** conservative and imperfect; relies on filesystem mtimes.
- **Cascading implications:** world backend must expose a stable “session start time” for the pending-diff set.
- **Risks:** false positives on coarse mtime filesystems; false negatives when mtimes are not updated.
- **Unlocks:** simple conflict-policy behavior now; can be replaced later by a new feature if needed.
- **Quick wins / low-hanging fruit:** implementable in early slices.

**Recommendation**
- **Selected:** Option B — Session-start mtime heuristic
- **Rationale (crisp):** provides a deterministic, implementable conflict model without introducing a new persisted schema surface.

**Follow-up tasks (explicit)**
- Encode the conflict rule and its edge cases in `filesystem-semantics-spec.md` and enforce in WS2/WS5 implementations.

### DR-0005 — Internal git repository contract (dir style + branch)

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/internal-git-spec.md`, `docs/project_management/next/world-sync/WS6-spec.md`, `docs/project_management/next/world-sync/WS7-spec.md`

**Problem / Context**
- Substrate’s internal git must be deterministic, host-only, and must never interfere with the user repo’s `.git/`.

**Option A — Bare repo (`git init --bare`)**
- **Pros:** conventional for “repo.git” naming.
- **Cons:** awkward with a fixed external work-tree (`--work-tree`); easy to misconfigure; increases cognitive burden.
- **Cascading implications:** more brittle command invocations; higher chance of “detached work-tree” foot-guns.
- **Risks:** operator support burden.
- **Unlocks:** none required for this feature.
- **Quick wins / low-hanging fruit:** none.

**Option B — Separate git-dir with explicit work-tree (`git --git-dir=… --work-tree=… init --initial-branch=main`)**
- **Pros:** matches the “git-dir + work-tree” model used by the PRD; deterministic branch name; works regardless of whether the workspace is a git repo.
- **Cons:** slightly non-standard naming; but contract is explicit.
- **Cascading implications:** internal commands always provide both `--git-dir` and `--work-tree`.
- **Risks:** low.
- **Unlocks:** consistent checkpoint/rollback workflows.
- **Quick wins / low-hanging fruit:** simplest deterministic contract for Substrate-owned history.

**Recommendation**
- **Selected:** Option B — Separate git-dir + explicit work-tree, initial branch `main`
- **Rationale (crisp):** reduces foot-guns and keeps the contract explicit and deterministic.

**Follow-up tasks (explicit)**
- Implement internal git init + checkpoint/rollback per WS6/WS7.

### DR-0006 — Windows behavior for `workspace sync` apply

**Decision owner(s):** World / Shell maintainers  
**Date:** 2026-02-10  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-sync/platform-parity-spec.md`, `docs/project_management/next/world-sync/WS2-spec.md`, `docs/project_management/next/world-sync/WS5-spec.md`

**Problem / Context**
- The Windows/WSL backend currently does not expose reliable `fs_diff` retrieval (`crates/shell/src/execution/platform_world/windows.rs` returns empty diffs in stubs), so sync apply cannot be made correct without additional backend work.

**Option A — “Best-effort” Windows sync apply**
- **Pros:** claims parity early.
- **Cons:** risks false guarantees and silent data loss; hard to validate; violates “avoid false guarantees” posture.
- **Cascading implications:** forces ad-hoc fallbacks and hidden behavior differences.
- **Risks:** high (safety, trust).
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** none.

**Option B — Explicit unsupported on Windows for apply paths (exit `4`)**
- **Pros:** honest; deterministic; avoids false guarantees; unblocks Linux/macOS progress.
- **Cons:** Windows users do not get sync apply in this feature.
- **Cascading implications:** smoke scripts and docs must assert exit `4` + actionable message.
- **Risks:** low.
- **Unlocks:** Windows support can be planned as a separate feature once backend capabilities exist.
- **Quick wins / low-hanging fruit:** clear contract now.

**Recommendation**
- **Selected:** Option B — Explicit unsupported on Windows for sync apply
- **Rationale (crisp):** safety and truthfulness outweigh premature parity claims.

**Follow-up tasks (explicit)**
- Ensure Windows smoke for WS2/WS5 asserts “unsupported” behavior deterministically.
