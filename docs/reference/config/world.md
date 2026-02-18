# World Root and Caging (Config Reference)

This page documents the user-facing configuration keys that control:

- how Substrate chooses the directory it treats as the “project” when running in a world, and
- whether the interactive REPL is allowed to `cd` outside that project root (“caging”).

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

## Why this matters for policy

In `world_fs.isolation=full`, discover/read/write allow/deny lists are interpreted relative to the world root.
That means changing `world.anchor_mode` can change what a given relative allowlist entry means.

If you use restrictive allowlists (recommended), prefer `anchor_mode=workspace` or `custom` unless you explicitly want the
root to roam.

### `world.env.inherit_from_host`

Controls whether Substrate forwards a small allowlist of host environment variables into `--world` executions.

- `false` (default): do not forward host env vars beyond Substrate’s deterministic baseline.
- `true`: forward a small safe allowlist (terminal/locale/timezone) as defined by the world env contract for this release.

See:
- Full configuration reference (including env forwarding contract): `docs/CONFIGURATION.md`
- Planning contract (host-visible hardening): `docs/project_management/packs/active/world-deps-host-visible-hardening/WDH0-spec.md`

## Related documentation

- Config file locations and precedence: `docs/reference/config/contract.md`
- Full configuration reference (including env overrides and exported state): `docs/CONFIGURATION.md`
- Workspace definition and precedence model:
  - `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- World filesystem isolation and policy model: `docs/WORLD.md`
