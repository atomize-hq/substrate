# M4 Spec – World deps base manifest parity

## Goal
Ensure `substrate world deps` reads its base manifest from the installed Substrate layout by default, so behavior does not depend on the current working directory or being inside a git checkout.

This triad is about **manifest resolution and override semantics**, not expanding the tool list.

## Context / Problem
Today, `substrate world deps` may resolve `world-deps.yaml` from a repository-relative or CWD-relative fallback, which can lead to:
- Fresh installs behaving differently depending on where users run `substrate`.
- Confusion when the installer has staged `world-deps.yaml` under the versioned install config, but the CLI reads the repo copy instead.

The desired default behavior is consistent with `docs/CONFIGURATION.md`: release/dev installs should prefer the versioned manifest under `<prefix>/versions/<version>/config/world-deps.yaml`, with explicit overrides still supported.

## Scope
### Required behavior
1. **Default base manifest path (installed layouts)**
   - When running an installed Substrate binary (dev or release), `substrate world deps {status,install,sync}` uses:
     - Base manifest: `<prefix>/versions/<version>/config/world-deps.yaml`.
2. **Workspace/dev fallback**
   - When running a workspace build (e.g. `target/debug/substrate` inside a repo checkout), the CLI may fall back to the repo manifest:
     - Base manifest: `scripts/substrate/world-deps.yaml` (repo-root relative).
3. **Overrides (must remain supported)**
   - `SUBSTRATE_WORLD_DEPS_MANIFEST` overrides the base manifest path explicitly.
   - The user overlay path remains:
     - `~/.substrate/world-deps.local.yaml` (or `SUBSTRATE_HOME` equivalent).
4. **Observability**
   - `substrate world deps status` continues to surface the resolved base + overlay paths (human output + JSON) so support bundles can identify mis-resolution quickly.

### Non-goals / Out of scope
- Expanding the default tool list or changing install recipes.
- Changing guest PATH/HOME normalization.
- Changing the meaning of `world deps sync` beyond path resolution.

## Acceptance criteria
- Running `substrate world deps status` from arbitrary directories after install reports the base manifest under `<prefix>/versions/<version>/config/world-deps.yaml` (unless overridden).
- Workspace builds still work without needing installed manifests.
- JSON payload (`--json`) reports the same resolved `manifest.base` path used for execution.
- Documentation references remain accurate (no drift between `docs/CONFIGURATION.md` and behavior).

