# ADR Intake Sheet

## 1. Codename + Created + Status

- Codename: `staging_beaver`
- Created: 2026-02-20T01:32:05Z
- Status: brainstorming
- Dependencies: []

## 2. Working Title (tentative)

Stabilize dev-install helper discovery under `$SUBSTRATE_HOME` (reduce coupling to `<repo>/target/*`).

## 3. Problem / Motivation

- Production installs stage a complete bundle into `~/.substrate/versions/<version>/` (with `bin/`, `scripts/`, `config/`) and then link `~/.substrate/bin/*` into that version dir.
- Dev installs (`scripts/substrate/dev-install-substrate.sh`) currently link `~/.substrate/bin/substrate` directly to `<repo>/target/<profile>/substrate`, which makes the inferred “version dir” become `<repo>/target/` (via canonicalization).
- This couples runtime helper discovery (e.g., `substrate world enable` helper path) and artifact expectations (e.g., `bin/linux/world-agent`) to `<repo>/target/*` conventions rather than Substrate home.
- The coupling creates sharp edges for “install with `--no-world`, enable later”, for multi-repo setups, and for users who clean `target/` frequently.

## 4. Proposed Outcome

- After `dev-install-substrate.sh`, `substrate world enable` can reliably locate its helper scripts via `$SUBSTRATE_HOME` paths (even after `cargo clean` + rebuild), without requiring a production-like `~/.substrate/versions/dev/` layout.
- Optionally (follow-up), dev installs can move toward full “version dir” parity with production installs.

## 5. Non-Goals

- Changing how binaries are built (still `cargo build …` from the repo).
- Changing the meaning of `--no-world` beyond provisioning/build/staging decisions already discussed elsewhere.
- Changing production installer layout.

## 6. Constraints / Invariants

- UX: dev-install must remain quick for inner-loop iteration.
- Correctness: links should always point at the current repo build outputs (debug/release).
- Safety: avoid surprising system-level provisioning; this ADR is about layout, not enabling worlds.

## 7. Interfaces / Contracts (concrete changes)

ADR Candidate A (this one / Option 2):
- `scripts/substrate/dev-install-substrate.sh` ensures helper scripts exist under:
  - `~/.substrate/scripts/substrate/world-enable.sh`
  - `~/.substrate/scripts/substrate/install-substrate.sh`
  so `substrate world enable` can use its existing `$SUBSTRATE_HOME/scripts/...` fallback.
- `scripts/substrate/dev-uninstall-substrate.sh` removes those staged helper scripts when uninstalling the matching dev install.

Candidate B (follow-up / Option 1):
- Adopt a production-like `~/.substrate/versions/<label>/` layout (bin/scripts/config) and make `~/.substrate/bin/*` link into it.

## 8. Options

### Option 1 — Full parity: stage a “bundle-like” version dir and link `~/.substrate/bin` to it (prod-like)

**Description**
Treat `~/.substrate/versions/<label>/` as the canonical dev “bundle root” and link everything through it (binaries + helper scripts + config assets), matching the production install mental model.

Concrete on-disk expectations (Linux/macOS; Windows analogous):
- `~/.substrate/versions/<label>/bin/` contains the host binaries needed by the CLI (`substrate`, `substrate-shim`, etc).
- `~/.substrate/versions/<label>/scripts/substrate/` contains helper scripts (`world-enable.sh`, `install-substrate.sh`, …).
- `~/.substrate/bin/*` are symlinks into `~/.substrate/versions/<label>/bin/*`.

Important nuance: the `substrate world enable` CLI currently infers the “version dir” by `canonicalize()`-ing `~/.substrate/bin/substrate`. That means **`~/.substrate/versions/<label>/bin/substrate` must be a real file (copy or hardlink), not a symlink to `<repo>/target/...`**, or we must change the CLI’s version-dir inference logic. Otherwise the inferred version dir stays `<repo>/target/` and this option doesn’t achieve stable helper discovery.

**Pros**
- Eliminates reliance on `<repo>/target/` as a pseudo-version dir.
- Makes `substrate world enable`/helper discovery consistent with production.
- Simplifies support/debugging (“version dir always lives under SUBSTRATE_HOME”).

**Cons**
- More symlinks/files to manage; slight complexity in dev-install script.
- If we choose “copy/hardlink binaries into version dir”, the installed CLI won’t automatically track `cargo build` outputs unless dev-install is rerun.

**Risk notes**
- Need to ensure profile switches (debug/release) update the version dir consistently.
- Hardlinks may fail across filesystems; copies always work but can drift from the current build output.

### Option 2 — Minimal parity: keep current binary links; only stabilize helper discovery

**Description**
Keep `~/.substrate/bin/substrate` pointing directly at `<repo>/target/<profile>/substrate` (so the “version dir” remains `<repo>/target/`), but ensure the world-enable helper scripts are discoverable even when `target/scripts/…` is missing (e.g., after `cargo clean`).

Concrete on-disk expectations:
- Keep the current `~/.substrate/bin/* -> <repo>/target/...` symlink pattern unchanged.
- Also stage helper scripts under `~/.substrate/scripts/substrate/` so `locate_helper_script()` can fall back to `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`.

**Pros**
- Smaller change; reduces breakage risk.
- Preserves the “always run latest build output” dev workflow.

**Cons**
- Still leaves version-dir inference pointing into `<repo>/target/`.
- Doesn’t solve other `<repo>/target/` coupling (e.g., expecting `bin/linux/world-agent` under `<repo>/target/` for provisioning).

**Risk notes**
- Might not fully solve downstream tooling that assumes “version dir” semantics.

## 9. Recommendation (tentative) + “Choose Option X when…”

Tentative: **Option 2** (smallest vertical slice; fixes the most brittle behavior without changing the dev inner loop).

Choose **Option 2** when we want to keep `~/.substrate/bin/substrate -> <repo>/target/...` (always-current build outputs) but make `substrate world enable` robust across `cargo clean` cycles.

Choose **Option 1** when we want dev + prod installs to share the same mental model and are willing to also address the version-dir inference nuance (or accept that staged binaries can drift from current `cargo build` output).

## 10. Slice Decomposition (required)

- ADR Candidate A (this one / Option 2): stabilize helper discovery under `$SUBSTRATE_HOME`.
  - Slice 1: dev-install stages helper scripts under `~/.substrate/scripts/substrate/…`.
  - Slice 2: dev-uninstall removes `~/.substrate/scripts/substrate/…` entries created by dev-install.
- ADR Candidate B (follow-up / Option 1): production-like version dir layout parity (`~/.substrate/versions/<label>/{bin,scripts,config}`) + updated dev-uninstall semantics.
- ADR Candidate C (follow-up): profile switching semantics (`--profile debug|release`) and how it interacts with `--version-label` (overwrite-by-default; use different label for side-by-side).

## 11. Acceptance Criteria Draft (observable outcomes)

- After dev-install, `~/.substrate/scripts/substrate/world-enable.sh` exists.
- After dev-install, `~/.substrate/scripts/substrate/install-substrate.sh` exists (world-enable helper dependency).
- After `cargo clean` + rebuilding `target/<profile>/substrate` (without re-running dev-install), `substrate world enable --dry-run` succeeds by resolving the helper under `$SUBSTRATE_HOME/scripts/...`.
- After dev-uninstall, the staged helper scripts under `~/.substrate/scripts/substrate/…` are removed (only for the matching dev install prefix).

## 12. Open Questions / Unknowns (with priority)

- P0: Should `version-label` default remain `dev`, and should multiple labels be supported concurrently?
  - Proposed answer: yes (keep current behavior; avoid changing defaults).
- P0: Profile switching semantics: default to overwriting the selected label’s staged scripts; use a different `--version-label` when you want debug+release available simultaneously.
  - Proposed answer: yes (overwrite-by-default; label for side-by-side).
- P1: Should dev-install stage helpers by **symlink** (preferred) or **copy** into `~/.substrate/scripts/substrate/`?
  - Proposed answer: symlink (keeps helper scripts aligned with the repo checkout).
- P2: Should we add a tiny guard so dev-uninstall only removes helpers if they are symlinks pointing into the current repo (avoid deleting user customizations)?
  - Proposed answer: yes (safety belt; reduces foot-guns).

## 13. “Ready to Draft ADR?” checklist (yes/no with reasons)

- [ ] One behavior delta locked (Option 2: stage helpers under `$SUBSTRATE_HOME/scripts/...`).
- [ ] P0/P1/P2 proposals confirmed (or adjusted).
- [ ] Acceptance criteria reflect `cargo clean` + rebuild scenario.
