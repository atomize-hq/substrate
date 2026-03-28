# World Configuration (Config Reference)

This page documents the user-facing configuration keys that control:

- how Substrate chooses the directory it treats as the “project” when running in a world, and
- whether the interactive REPL is allowed to `cd` outside that project root (“caging”).
- whether the host may request outbound netfilter enforcement from world backends.

If you want the implementation details (env plumbing, REPL drift restarts, Landlock allowlists), see:
`docs/internals/config/world_root_and_caging.md`.

## Keys

### `world.anchor_mode`

Controls how Substrate chooses the world’s project root.

Accepted values:

- `workspace` (default): anchor the world root to the detected workspace root (the nearest ancestor with
  `.substrate/workspace.yaml`). If no workspace exists, the root is the directory where `substrate` started.
- `follow-cwd`: treat the *current working directory* as the world root.
- `custom`: use the path specified by `world.anchor_path`.

Practical guidance:

- Use `workspace` for “stay within this workspace/project” workflows (stable root; policy allowlists stay meaningful while
  `cd`’ing into subdirectories).
- Use `follow-cwd` for “roaming shell” workflows where you intentionally `cd` between unrelated directories and want the
  world root to move with you.
- Use `custom` when you need the root pinned to a specific directory regardless of where you launch from or `cd` to.

### `world.anchor_path`

Only used when `world.anchor_mode=custom`.

- Must be a directory that exists.
- Relative paths are resolved relative to the directory where `substrate` is launched.

Example:

```yaml
world:
  anchor_mode: custom
  anchor_path: /home/spenser/__Active_code
```

### `world.caged`

Controls whether the REPL is allowed to leave the chosen root via `cd`.

- `true`: prevent leaving the resolved root; `cd ..` that escapes will be “bounced” back to the root.
- `false`: allow roaming outside the root.

Important interaction:

- `world.caged` has **no effect** when `world.anchor_mode=follow-cwd` because the root is defined to move with the cwd.
  (The system effectively behaves uncaged in that mode.)

### `world.net.filter`

Controls whether the host may request outbound egress enforcement from the world backend.

- `false` (default): the host never requests `isolate_network`, even if policy `net_allowed` is restrictive. This
  preserves existing behavior for current installs and policies.
- `true`: the host may request `isolate_network`, but only when policy `net_allowed` is restrictive after
  canonicalization (anything other than the allow-all singleton `["*"]`).

This key answers a different question than the runtime backend guard or the policy allowlist:

| Signal | Question it answers | Effect |
|----------|---------|---------|
| `world.net.filter` | May the host request enforcement? | Enables or disables host-side `isolate_network` requests. |
| `WORLD_NETFILTER_ENABLE=1` | May the world backend apply enforcement? | Allows the backend to honor a requested isolation run; without it, the backend fails the requested run instead of silently skipping enforcement. |
| policy `net_allowed` | What is the allowlist once enforcement is requested? | Defines the canonicalized allow-all, deny-all, or restrictive allowlist posture used by the requested isolation run. |

Practical examples:

- Allow-all: `world.net.filter=true` and canonicalized `net_allowed=["*"]` still keep the host in allow-all posture, so
  it does not request `isolate_network`.
- Deny-all: `world.net.filter=true` and canonicalized `net_allowed=[]` cause the host to request `isolate_network=true`
  with deny-all semantics; the backend still needs `WORLD_NETFILTER_ENABLE=1` to execute that request successfully.
- Restrictive allowlist: `world.net.filter=true` and canonicalized `net_allowed=["github.com","crates.io"]` cause the
  host to request `isolate_network=true` with that allowlist. The same restrictive policy does nothing when
  `world.net.filter=false`.

## Why this matters for policy

In `world_fs.isolation=full`, discover/read/write allow/deny lists are interpreted relative to the world root.
That means changing `world.anchor_mode` can change what a given relative allowlist entry means.

If you use restrictive allowlists (recommended), prefer `anchor_mode=workspace` or `custom` unless you explicitly want the
root to roam.

Use `world.net.filter=true` only when you want the host to turn restrictive `net_allowed` policy into an enforcement
request. Restrictive policy alone does not request filtering.

### `world.env.inherit_from_host`

Controls whether Substrate forwards a small allowlist of host environment variables into `--world` executions.

- `false` (default): do not forward host env vars beyond Substrate’s deterministic baseline.
- `true`: forward a small safe allowlist (terminal/locale/timezone) as defined by the world env contract for this release.

See:
- Full configuration reference (including env forwarding contract): `docs/CONFIGURATION.md`
- Planning contract (host-visible hardening): `docs/project_management/packs/implemented/world-deps-host-visible-hardening/WDH0-spec.md`

### Hardened worlds: `$HOME` is scratch (ephemeral)

By default, Substrate’s in-world env contract sets:

- `HOME=/root`
- `XDG_*` under `/root`

In hardened world executions (for example, when `world_fs.host_visible=false` / strict deny is enabled), the world backend
may make the base root filesystem effectively read-only. Substrate will still ensure `/root` is writable so tools work,
but you should treat it as **scratch space**:

- Content under `/root` (npm cache, `npx` temp installs, tool logs) may not persist across separate `substrate -c ...`
  invocations.
- For persistence/reproducibility, prefer:
  - Project-local installs (e.g. `npm i` + `npx <tool>` / `npm exec <tool>`), or
  - World-deps packages/bundles that install CLIs into `/var/lib/substrate/world-deps/bin`.

Implementation notes live in: `docs/internals/world/deps.md`.

## Related documentation

- Config file locations and precedence: `docs/reference/config/contract.md`
- Full configuration reference (including env overrides and exported state): `docs/CONFIGURATION.md`
- Workspace definition and precedence model:
  - `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- World filesystem isolation and policy model: `docs/WORLD.md`
