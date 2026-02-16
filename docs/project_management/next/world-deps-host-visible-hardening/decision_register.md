# Decision Register — world-deps-host-visible-hardening

Template standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

### DR-0001 — `--world` environment model in host-visible worlds

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A), `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`

**Problem / Context**
- With `world_fs.host_visible=true`, host filesystem paths are nameable in-world. If we inherit host env (PATH/HOME/XDG), PATH-based resolution can find host toolchains (e.g., `~/.nvm/.../bin/npm`) even when no world-deps are enabled/applied.
- The desired contract is: toolchain availability is a **world-deps-controlled surface**, independent of host PATH.

**Option A — Construct a sanitized in-world env (default)**
- **Pros:** deterministic; blocks accidental host toolchain usage; consistent across PTY and non-PTY; makes “present” meaningful.
- **Cons:** may break workflows that implicitly depend on host env variables being forwarded into the world.
- **Cascading implications:** requires an explicit env forwarding model (see DR-0008) and a baseline PATH contract per platform/backend.
- **Risks:** unexpected behavior changes for users relying on inherited PATH; mitigated by explicit opt-in escape hatch (see DR-0007).
- **Unlocks:** host-visible behavior matches host-hidden behavior for PATH-based resolution.
- **Quick wins / low-hanging fruit:** implement in request/env builders first; add tests that assert no `$HOME/.nvm` etc in PATH.

**Option B — Inherit host env and only prepend `/var/lib/substrate/world-deps/bin`**
- **Pros:** minimal change; preserves legacy behavior for existing workflows.
- **Cons:** does not solve the problem; host toolchains remain discoverable and can satisfy probes/entrypoints.
- **Cascading implications:** forces complicated “present” logic to distinguish host vs world tools; brittle.
- **Risks:** security drift in host-visible; continued operator confusion.
- **Unlocks:** none for the hardening goal.
- **Quick wins / low-hanging fruit:** none (doesn’t meet desired behavior).

**Recommendation**
- **Selected:** Option A — Construct a sanitized in-world env (default)
- **Rationale (crisp):** host-visible must not imply host toolchain inheritance; deterministic env construction is the only stable basis for hardening.

**Follow-up tasks (explicit)**
- Implement sanitized env construction for all `--world` pathways (PTY + non-PTY): `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` (`WDH0-code`).
- Add unit + integration tests asserting sanitized PATH/HOME/XDG behavior: `docs/project_management/next/world-deps-host-visible-hardening/tasks.json` (`WDH0-test`, `WDH0-integ`).

### DR-0002 — Baseline `PATH` contract for `--world`

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`

**Problem / Context**
- Once we stop inheriting host PATH, we must define a deterministic baseline PATH that is stable across shells and does not accidentally reintroduce host toolchain segments.

**Option A — Fixed canonical baseline PATH per platform/backend**
- **Pros:** deterministic; simple; testable; does not depend on guest shell init or distro differences.
- **Cons:** may omit non-standard locations some distros rely on.
- **Cascading implications:** must include the standard system search dirs; must be documented and used everywhere.
- **Risks:** missing binaries if a backend places them outside canonical dirs; mitigated by keeping the baseline conservative and by allowing explicit config additions (see DR-0007).
- **Unlocks:** stable “command -v” semantics for wrappers/probes.
- **Quick wins / low-hanging fruit:** define once and reuse across request builders and probes.

**Option B — Discover baseline PATH dynamically inside the world (e.g., run a shell to print PATH)**
- **Pros:** adapts to the image/distro.
- **Cons:** depends on shell behavior and init files; can reintroduce non-determinism; becomes a circular dependency for early bootstrapping.
- **Cascading implications:** requires a “bootstrap PATH” anyway to run the discovery command; hard to test.
- **Risks:** PATH discovery differs between non-PTY and PTY evaluators; hard-to-debug parity issues.
- **Unlocks:** none that outweigh determinism costs.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Fixed canonical baseline PATH per platform/backend
- **Rationale (crisp):** hardening requires deterministic env construction; dynamic PATH discovery reintroduces drift.

**Follow-up tasks (explicit)**
- Define the baseline PATH strings and document them: `WDH0-spec.md` + `WDH0-code`.

### DR-0003 — Runnable `apt` packages must generate `/var/lib/substrate/world-deps/bin` entrypoints

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` (Inventory schema), `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`

**Problem / Context**
- Script packages already place entrypoints in `/var/lib/substrate/world-deps/bin`. Runnable `apt` packages currently rely on system PATH (`/usr/bin/...`), which is fragile in host-visible worlds (and makes “which”/presence ambiguous).

**Option A — Mandatory wrappers/symlinks for runnable `apt` entrypoints**
- **Pros:** deterministic `command -v`/`which`; enables wrapper-based “present”; consistent model across install methods.
- **Cons:** requires extra mutation step after apt install; requires wrapper collision handling.
- **Cascading implications:** `sync/install` must reconcile wrappers (create/update/remove) as part of apply.
- **Risks:** wrappers can drift if the real binary moves; mitigated by using stable system paths and by updating wrappers on sync.
- **Unlocks:** host-visible behaves like host-hidden for PATH lookup.
- **Quick wins / low-hanging fruit:** implement wrappers for the built-in node/npm/bun packages as first coverage.

**Option B — No wrappers; rely on baseline PATH to find `/usr/bin/<tool>`**
- **Pros:** less mutation; closer to “normal” system behavior.
- **Cons:** presence becomes ambiguous; PATH ordering still matters; cannot ensure toolchain comes from enabled deps.
- **Cascading implications:** “present” needs ad-hoc probe rules; operator confusion continues.
- **Risks:** reintroduces accidental host toolchain usage if PATH ever includes host segments.
- **Unlocks:** none for hardening.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Mandatory wrappers/symlinks for runnable `apt` entrypoints
- **Rationale (crisp):** wrappers are the only robust anchor for deterministic entrypoint resolution in host-visible worlds.

**Follow-up tasks (explicit)**
- Implement wrapper creation/update/removal for runnable `apt` packages during apply: `WDH1-code`.
- Add integration tests for `command -v <entrypoint>` returning `/var/lib/substrate/world-deps/bin/<entrypoint>` after sync: `WDH1-test`, `WDH1-integ-*`.

### DR-0004 — Runnable package “present” semantics must be wrapper/probe based (not PATH based)

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`

**Problem / Context**
- In host-visible worlds, “present” cannot be derived from an inherited PATH because host toolchains can satisfy `command -v`.

**Option A — Define “present” as wrapper-exists + wrapper/probe succeeds under sanitized env**
- **Pros:** independent of host PATH; deterministic; aligns to “world-deps controlled surface”.
- **Cons:** adds wrapper maintenance responsibility; requires clear failure messaging for wrapper exec failures.
- **Cascading implications:** probes must execute under the sanitized env; wrapper generation becomes part of apply.
- **Risks:** wrapper present but broken; mitigated by “present requires probe success” and remediation messaging.
- **Unlocks:** accurate `present/missing/blocked` in host-visible.
- **Quick wins / low-hanging fruit:** use `probe.command` when provided; otherwise default to wrapper execution check for runnable packages.

**Option B — Define “present” as “command exists somewhere on PATH”**
- **Pros:** simple.
- **Cons:** fails the contract; host toolchains can satisfy presence; makes “applied” meaningless.
- **Cascading implications:** forces special cases for every toolchain.
- **Risks:** incorrect applied views; security posture drift.
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Wrapper/probe semantics under sanitized env
- **Rationale (crisp):** “present” must be host-path-independent to be meaningful in host-visible worlds.

**Follow-up tasks (explicit)**
- Update applied/probe logic to use wrapper/probe semantics under sanitized env: `WDH1-code`.
- Add smoke + integration assertions for “present” independence from host PATH: `WDH1-integ-*`.

### DR-0005 — Deny explicit execution of host-mounted toolchain binaries in host-visible worlds

**Decision owner(s):** World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md`

**Problem / Context**
- Sanitizing PATH prevents accidental host toolchain usage, but a user can still explicitly invoke a host-visible binary path (e.g., `/home/<user>/.config/nvm/.../npm`).
- The desired posture is hardened: “nothing from the host should be available” as a toolchain inside the world.

**Option A — Enforce an execution-time guard with a deterministic denylist of toolchain path patterns (default deny)**
- **Pros:** closes the explicit-path escape hatch for common host toolchains without blocking workspace executables; deterministic and testable.
- **Cons:** not a perfect “all host mounts” classifier; requires maintaining the denylist over time.
- **Cascading implications:** must define default deny patterns and override knobs; must be enforced in the world execution path (not only shell).
- **Risks:** false positives if a workspace intentionally contains `/.nvm/`-like paths; mitigated by allow overrides and clear error messages.
- **Unlocks:** host-visible behavior matches host-hidden for the practical toolchain leak vectors observed (nvm/pyenv/cargo/bun).
- **Quick wins / low-hanging fruit:** ship a conservative default denylist (`/.nvm/`, `/.config/nvm/`, `/.pyenv/`, `/.cargo/bin/`, `/.local/bin/`, `/.bun/bin/`).

**Option B — Do not guard explicit paths; only harden PATH-based resolution**
- **Pros:** less complexity; fewer surprises.
- **Cons:** does not meet the “no host deps” posture; explicit host toolchains remain runnable in-world.
- **Cascading implications:** policy claims are weaker than observed behavior.
- **Risks:** users (or agents) bypass world-deps by calling host binaries directly.
- **Unlocks:** none for the hardened posture.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
-- **Selected:** Option A — Execution-time guard with deterministic toolchain-path denylist (default deny)
- **Rationale (crisp):** closes the explicit-path bypass while avoiding collateral damage to normal workspace execution.

**Follow-up tasks (explicit)**
- Specify default deny patterns and override knobs, plus exit code (`5`) in `WDH2-spec.md`: `WDH2-code`.
- Implement enforcement in the world execution path (world-agent/backend): `WDH2-code`.
- Add integration test and smoke case: explicit host binary path invocation fails with exit `5`: `WDH2-test`, `WDH3-integ-*` (final checkpoint).
- Track persistent policy/config configurability for the denylist in `docs/BACKLOG.md` (P1 “world exec-guard denylist” item).

### DR-0006 — `$SUBSTRATE_HOME/deps/` scaffolding is created on install (with examples)

**Decision owner(s):** Shell / Installer maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A), `docs/project_management/next/world-deps-host-visible-hardening/WDH3-spec.md`

**Problem / Context**
- Operators see “available” deps even when `$SUBSTRATE_HOME/deps/` doesn’t exist (built-ins). Without scaffolding, it’s unclear where custom global deps live and whether host deps are “leaking”.

**Option A — Scaffold `$SUBSTRATE_HOME/deps/` during install/first-run init (non-enabling)**
- **Pros:** discoverable; reduces confusion; provides canonical examples; supports docs-first learning.
- **Cons:** adds files to `$SUBSTRATE_HOME`; may surprise users who expect a minimal home dir.
- **Cascading implications:** installer must ensure idempotency and never auto-enable examples.
- **Risks:** examples become stale; mitigated by keeping examples shape-only and referencing the contract docs.
- **Unlocks:** aligns operator mental model with inventory/patch model.
- **Quick wins / low-hanging fruit:** create README + example YAML stubs + example script stub.

**Option B — Do not scaffold; require users to create dirs manually**
- **Pros:** minimal footprint.
- **Cons:** discoverability is poor; increases confusion during troubleshooting; slows adoption.
- **Cascading implications:** support burden increases.
- **Risks:** repeated “where are available deps coming from?” confusion.
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Scaffold on install/first-run init (non-enabling)
- **Rationale (crisp):** scaffolding is low cost and eliminates the main operator confusion observed in host-visible debugging.

**Follow-up tasks (explicit)**
- Specify exact layout + example file contents in `WDH3-spec.md`: `WDH3-code`.
- Implement installer/init scaffolding + idempotency: `WDH3-code`.
- Add a smoke case asserting the scaffold exists after install: `WDH3-integ-*`.

### DR-0007 — Provide an explicit, audited opt-in to inherit host env PATH segments

**Decision owner(s):** Shell / Config maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`, `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md`

**Problem / Context**
- Sanitizing env by default is a breaking change for some workflows. We need a deterministic, auditable escape hatch that does not silently reintroduce host toolchain leakage.

**Option A — Add a config lever (`world.env.inherit_from_host=true|false`)**
- **Pros:** persistent and auditable in config; matches Substrate’s `world.*` naming; supports workspace overrides; avoids per-invocation env hacks.
- **Cons:** requires extending config schema/editor/explain plumbing.
- **Cascading implications:** must define strict semantics (exact forwarded set) and precedence (workspace over global).
- **Risks:** users enable it and expect “full inherit” when the contract is allowlist-only; mitigated by explicit naming (`inherit_from_host`) + docs + warning output.
- **Unlocks:** enables later install-time UX (“pick deps upfront”) without relying on ambient host env.
- **Quick wins / low-hanging fruit:** add a single boolean key merged by replace.

**Option B — Add an env var override only (no config lever)**
- **Pros:** simplest to implement; no schema changes.
- **Cons:** non-discoverable; not persistent; hard to audit; invites “works on my machine” drift.
- **Cascading implications:** encourages ad-hoc wrappers/scripts; weakens reproducibility across teams.
- **Risks:** env var can leak into shells/CI unintentionally; support burden increases.
- **Unlocks:** none beyond short-term convenience.
- **Quick wins / low-hanging fruit:** minimal code changes in env builder.

**Recommendation**
- **Selected:** Option A — Config lever `world.env.inherit_from_host`
- **Rationale (crisp):** we want a stable, team-auditable posture switch; config is the right surface for a behavior lever.

**Follow-up tasks (explicit)**
- Specify key name, precedence, and exact forwarded allowlist semantics: `WDH0-spec.md` + `WDH0-code`.
- Add config editor + `config explain` provenance for the new key: `WDH0-code` + `WDH0-test`.
- Backlog: add configurable allowlist key `world.env.allow_list` (see `docs/BACKLOG.md`): follow-up work.

### DR-0008 — Host env forwarding model (beyond PATH/HOME/XDG)

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`

**Problem / Context**
- Even with PATH sanitization, forwarding arbitrary host env vars can recreate toolchain coupling (e.g., `NVM_DIR`, `PYENV_ROOT`) and can create confusing “why does the world behave like my host?” moments.
- At the same time, fully allowlisting env vars risks breaking common tools (`TERM`, locale, color settings).

**Option A — “Inherit most” (copy host env, then override reserved keys)**
- **Pros:** high compatibility for legacy workflows that depend on host env.
- **Cons:** forwards ambient state (including secrets) into the world; recreates host coupling; undermines the “world is its own environment” mental model.
- **Cascading implications:** needs extensive redaction/observability considerations; must enumerate a growing denylist of toolchain/secrets vars.
- **Risks:** accidental leakage of host credentials into the world; “host deps” drift via env-coupled tooling.
- **Unlocks:** fewer immediate breakages.
- **Quick wins / low-hanging fruit:** implement by cloning env then overriding PATH/HOME/XDG.

**Option B — Strict allowlist (copy only a small safe set from host, otherwise baseline-only)**
- **Pros:** strongest isolation posture; minimizes ambient coupling and secret drift; aligns with the expectation that tooling/deps are provisioned via world-deps rather than host env.
- **Cons:** some workflows will need explicit additions over time.
- **Cascading implications:** requires a user-facing extension story: add a configurable allowlist later (`world.env.allow_list`) without changing the default posture.
- **Risks:** support burden if the default allowlist is too small; mitigated by keeping the default set OS/terminal-only and adding a backlog item for configurability.
- **Unlocks:** deterministic worlds even when host is visible; easier reasoning about “no host deps”.
- **Quick wins / low-hanging fruit:** start with `LANG`, `LC_*`, `TZ`, `NO_COLOR` (and set `TERM` deterministically).

**Recommendation**
- **Selected:** Option B — Strict allowlist
- **Rationale (crisp):** matches the desired posture: worlds should be operational with minimal OS-level env and should not inherit developer toolchains or secrets from the host.

**Follow-up tasks (explicit)**
- Specify the default allowlist and baseline-only behavior in `WDH0-spec.md`: `WDH0-code`.
- Add tests asserting non-allowlisted host vars are absent in-world by default: `WDH0-test`.
- Backlog: add `world.env.allow_list` (configurable extension) in `docs/BACKLOG.md`.

### DR-0009 — Apt wrapper form and target resolution for runnable entrypoints

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`

**Problem / Context**
- For runnable `apt` packages we must generate deterministic `/var/lib/substrate/world-deps/bin/<entrypoint>` entrypoints.
- The wrapper implementation must avoid recursion (wrapping `npm` must not resolve back to `/var/lib/substrate/world-deps/bin/npm`) and must be stable under the sanitized PATH contract.

**Option A — Generate a small POSIX `sh` wrapper that execs `/usr/bin/<entrypoint>`**
- **Pros:** deterministic; avoids PATH recursion; simple; works under `/bin/sh -c`; easy to test.
- **Cons:** assumes the real binary lives at `/usr/bin/<entrypoint>`; not true for every distro/package.
- **Cascading implications:** inventory must ensure entrypoints for runnable apt packages correspond to stable `/usr/bin` locations; otherwise those deps must be modeled as script wrappers instead.
- **Risks:** wrapper breaks if binary not at `/usr/bin`; mitigated by clear error + remediation (“use script wrapper or adjust inventory”) and by limiting built-in runnable apt entrypoints to those with stable paths.
- **Unlocks:** deterministic entrypoint resolution in host-visible worlds without needing new inventory schema fields.
- **Quick wins / low-hanging fruit:** implement for `node`, `npm`, `npx`, `bun` (where applicable) and any other built-in runnable apt entrypoints.

**Option B — Generate a wrapper that resolves the target with `command -v` at runtime**
- **Pros:** adapts to non-standard binary locations.
- **Cons:** easy to accidentally recurse into the wrapper; requires careful PATH manipulation inside the wrapper; harder to reason about and test.
- **Cascading implications:** wrapper must explicitly remove `/var/lib/substrate/world-deps/bin` from PATH before resolution; introduces more moving parts.
- **Risks:** non-determinism if PATH differs by evaluator; hard-to-debug.
- **Unlocks:** supports unusual layouts without inventory changes.
- **Quick wins / low-hanging fruit:** none (complexity is front-loaded).

**Recommendation**
- **Selected:** Option A — POSIX `sh` wrapper that execs `/usr/bin/<entrypoint>`
- **Rationale (crisp):** deterministic and recursion-free under the sanitized env contract; avoids adding new schema surfaces.

**Follow-up tasks (explicit)**
- Update `WDH1-spec.md` to require wrapper scripts (not symlinks) and to define `/usr/bin/<entrypoint>` resolution: `WDH1-code`.
- Add a test covering a multi-entrypoint package (e.g. `npm,npx`) and asserting wrappers exec the intended target: `WDH1-test`.

### DR-0010 — When and where `$SUBSTRATE_HOME/deps/` scaffolding is performed

**Decision owner(s):** Shell / Installer maintainers  
**Date:** 2026-02-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/world-deps-host-visible-hardening/WDH3-spec.md`

**Problem / Context**
- The user expectation is “scaffold on install”, but Substrate may be installed via multiple mechanisms (package manager, script, dev build). We need a single deterministic point where the scaffold is created.

**Option A — Scaffold during `$SUBSTRATE_HOME` initialization (first-run bootstrap)**
- **Pros:** deterministic and universal; does not depend on a specific installer; works for package-manager installs and dev builds.
- **Cons:** scaffolding happens on first run (not necessarily during external install).
- **Cascading implications:** bootstrap path must be idempotent and must not overwrite user files; must be safe under concurrent starts.
- **Risks:** very old installations might not get the scaffold until a later run; acceptable.
- **Unlocks:** consistent behavior across all install paths.
- **Quick wins / low-hanging fruit:** implement alongside existing `$SUBSTRATE_HOME` directory/config creation.

**Option B — Scaffold only in the installer scripts (outside the binary)**
- **Pros:** matches “on install” literally for script-based installs.
- **Cons:** misses package-manager installs and dev builds; multiple installers would need duplicate logic; harder to keep in sync.
- **Cascading implications:** every installer must be updated and tested.
- **Risks:** drift and inconsistent UX depending on install method.
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** none.

**Recommendation**
- **Selected:** Option A — Scaffold during `$SUBSTRATE_HOME` initialization
- **Rationale (crisp):** provides one universal, deterministic point that covers all installation methods.

**Follow-up tasks (explicit)**
- Update `WDH3-spec.md` to state scaffolding occurs during `$SUBSTRATE_HOME` bootstrap/init: `WDH3-code`.
- Add a test that a missing `$SUBSTRATE_HOME/deps/` is created during init and that user-edited files are not overwritten: `WDH3-test`.
