# WDH0 — `--world` env normalization (PATH/HOME/XDG)

## Goal
Make `--world` execution deterministic in host-visible worlds by constructing a sanitized in-world environment by default, rather than inheriting host user toolchain env.

## Inputs (authoritative)
- `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A)
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` (DR-0001, DR-0002, DR-0007, DR-0008)

## Contract

### Baseline PATH (default)
For all `--world` execution (PTY + non-PTY), the world environment MUST set:

- `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=/var/lib/substrate/world-deps/bin`
- `PATH=/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`

The baseline PATH MUST NOT include host-user toolchain paths (e.g. `$HOME/.config/nvm`, `~/.pyenv`, `~/.cargo/bin`, `~/.local/bin`).

### HOME/XDG (default)
For all `--world` execution (PTY + non-PTY), the world environment MUST NOT forward host user home state by default.
At minimum, it MUST ensure `XDG_*` variables do not point at host locations.

Exact behavior:
- `HOME=/root`
- `XDG_CONFIG_HOME=/root/.config`
- `XDG_DATA_HOME=/root/.local/share`
- `XDG_CACHE_HOME=/root/.cache`

### Host env forwarding model (default: strict allowlist)
By default, `--world` MUST NOT forward arbitrary host environment variables into the world.

In addition to the baseline PATH/HOME/XDG values above, `--world` MUST set:
- `TERM=xterm-256color`

### Config lever: `world.env.inherit_from_host`
Introduce a configuration key:
- `world.env.inherit_from_host: true|false`
- Default: `false`
- Merge semantics: replace (workspace override wins over global)
- Config locations:
  - Global: `$SUBSTRATE_HOME/config.yaml`
  - Workspace: `<workspace_root>/.substrate/workspace.yaml`

Behavior:
- When `world.env.inherit_from_host=false` (default):
  - No host env vars are forwarded (beyond the deterministic baseline variables specified in this spec).
- When `world.env.inherit_from_host=true`:
  - Only the following host env vars may be forwarded, when set on the host:
    - `LANG`
    - `LC_*` (all locale category variables)
    - `TZ`
    - `NO_COLOR`
  - Reserved keys MUST still be overwritten to their deterministic values:
    - `PATH` (baseline, prefixed with `/var/lib/substrate/world-deps/bin`)
    - `HOME` and `XDG_*` (as above)
    - `TERM` (as above)
  - A warning MUST be emitted once per command indicating that host env forwarding is enabled and can reintroduce coupling:
    - Example: `substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)`

Backlog:
- Add `world.env.allow_list` (configurable extension) so operators can extend the forwarded set explicitly. Track in `docs/BACKLOG.md`.

## Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 0: env constructed and execution succeeded
- 2: invalid `world.env.inherit_from_host` value

## Acceptance criteria
- With `world_fs.host_visible=true` and default env mode, `substrate --world -c 'echo "$PATH"'`:
  - begins with `/var/lib/substrate/world-deps/bin:`
  - does not contain `/.config/nvm/`, `/.pyenv/`, `/.cargo/bin`, `/.local/bin`
- PTY (`substrate` REPL) and non-PTY (`substrate --world -c ...`) match on PATH construction.
- When `world.env.inherit_from_host=true`, only the allowlisted env vars above are forwarded.

## Out of scope
- Wrapper generation and presence semantics (WDH1)
- Exec-time host binary guard (WDH2)
- Installer scaffolding (WDH3)
