# Integration Map — Policy + Config Mental Model Simplification (ADR-0003)

## Scope
- Implement ADR-0003 contracts for:
  - workspace discovery and initialization,
  - config and policy inventory (filenames, schema, strict parsing, precedence),
  - policy mode and routing semantics,
  - env script split and world enable home semantics,
  - hard removals of legacy artifacts and naming collisions.

## Non-scope
- Implementing world-sync functionality itself.
- Any backwards compatibility, migrations, aliases, or legacy discovery fallbacks.

## End-to-end data flow (inputs → derived state → actions → outputs)

Inputs:
- CLI flags:
  - world selection: `--world` / `--no-world`
  - anchor selection: `--anchor-mode` / `--anchor-path`
  - roaming guard: `--caged` / `--uncaged`
  - config/policy subcommands per ADR-0003
- Env vars (exhaustive for this track):
  - `SUBSTRATE_HOME`
  - `SUBSTRATE_WORLD`
  - `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`
  - `SUBSTRATE_CAGED`
  - `SUBSTRATE_POLICY_MODE`
  - sync env vars listed in ADR-0003 (world-sync contract alignment)
- On-disk inputs:
  - global: `$SUBSTRATE_HOME/config.yaml`, `$SUBSTRATE_HOME/policy.yaml`
  - workspace: `<workspace_root>/.substrate/workspace.yaml`, `<workspace_root>/.substrate/policy.yaml`

Derived state:
- Resolved `$SUBSTRATE_HOME`.
- Resolved `<workspace_root>` (or “no workspace”).
- Effective config (merged per precedence).
- Effective policy (selected per precedence).
- Policy-derived “requires world” constraints (observe/enforce only).
- Anchor root (resolved per anchor_mode).

Actions:
- Read/write config and policy files per CLI.
- Evaluate policy decisions in observe/enforce mode.
- Select host vs world execution per ADR-0003 routing rules.
- Enforce roaming guard in the interactive shell based on resolved anchor root.
- Generate and maintain `$SUBSTRATE_HOME/env.sh` and `$SUBSTRATE_HOME/manager_env.sh` per ADR-0003 ownership rules.

Outputs:
- Trace/telemetry records including policy decisions (observe/enforce).
- Updated YAML files and env scripts when explicitly requested.

## Component map (what changes where)

- `crates/common`
  - Canonical paths for YAML config/policy and env scripts.
  - Shared helpers for strict YAML parsing and error surfacing.

- `crates/shell`
  - Workspace discovery (`.substrate/workspace.yaml`) and nested init refusal.
  - `substrate workspace init`.
  - `substrate config` and `substrate config global` commands and update parsing.
  - Env var precedence integration (`SUBSTRATE_*`).
  - World enable home semantics (`--home`) and env script behavior.
  - Roaming guard anchor semantics (`--caged/--uncaged` uses anchor root only).

- `crates/broker`
  - Policy parsing, invariants, and evaluation semantics for `disabled|observe|enforce`.
  - Approval persistence targeting rules (“save to policy”).

- `crates/world` / `crates/world-agent` / `crates/shim`
  - Routing behavior when world is required or selected and backend is unavailable.
  - Env plumbing for filesystem isolation naming (`SUBSTRATE_WORLD_FS_ISOLATION`).
  - Runtime manager wiring and manager env script generation responsibilities.

- Installers (`scripts/substrate/*`)
  - Write `$SUBSTRATE_HOME/env.sh` with stable exports only.

## Composition with adjacent tracks (explicit dependencies)
- `world_sync` planning and implementation depends on ADR-0003:
  - `world_sync` config/env contracts for sync keys and protected excludes must match ADR-0003.
  - Update `docs/project_management/next/world-sync/C0-spec.md` and `docs/project_management/next/world-sync/C1-spec.md` before any `world_sync` execution triads begin.

## Sequencing alignment (final)
- `docs/project_management/next/sequencing.json` order places this sprint before `world_sync`.
- Internal triad order is linear:
  - `PCM0-integ` before `PCM1-code`
  - `PCM1-integ` before `PCM2-code`
  - `PCM2-integ` before `PCM3-code`

