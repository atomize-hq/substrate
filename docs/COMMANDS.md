# Command Matrix

This document mirrors the mental model of the CLI parser in `crates/shell/src/execution/cli.rs`. The root command accepts options (dashed flags) and positional data until you introduce a subcommand. After a subcommand token, parsing switches entirely to that subcommand’s own flags and positionals.

## Root Command: Execution Modes (before subcommands)

| Scenario | Required Flags/Positionals | Compatible Root Add-ons | Mutual Exclusions | Notes |
| --- | --- | --- | --- | --- |
| Interactive async REPL | default invocation or `--async-repl` | `--ci`, `--no-exit-on-error`, `--pty`, `--shell`, `--shim-skip`, `--world`/`--no-world`, `--caged`/`--uncaged`, `--anchor-mode`, `--anchor-path` | `-c`, `-f`, `--legacy-repl`, any subcommand | Default workflow (`docs/USAGE.md`) backed by `cli.rs:86`. |
| Legacy REPL (sync) | `--legacy-repl` | Same as interactive add-ons except `--async-repl` | `-c`, `-f`, `--async-repl`, subcommands | Regression fallback defined at `cli.rs:90`. |
| Single command | `-c/--command <CMD>` | `--ci`, `--no-exit-on-error`, `--pty`, `--shell`, `--shim-skip`, `--world`/`--no-world`, anchor/caging flags | `-f`, `--version-json`, shim mgmt, trace/replay | Primary non-interactive path at `cli.rs:27`. |
| Script execution | `-f/--file <SCRIPT>` | Same add-ons as `-c` | `-c`, `--version-json`, shim mgmt, trace/replay | Invokes REPL state preservation per `cli.rs:37`. |
| Version metadata | `--version-json` | `--world`, `--no-world`, `--shim-skip` | `-c`, `-f`, shim mgmt, trace/replay | Information-only request (`cli.rs:62`). |
| Trace inspect | `--trace <SPAN>` | `--world`, `--no-world`, anchor/caging flags | `-c`, `-f`, shim mgmt, `--replay` | Pull span metadata (`cli.rs:94`). |
| Replay | `--replay <SPAN>` | `--replay-verbose`, world + anchor toggles | `-c`, `-f`, `--trace`, shim mgmt | Deterministic replay (`cli.rs:98`). Defaults to world isolation even when the rest of the CLI is host-only; use `--no-world` or `SUBSTRATE_REPLAY_USE_WORLD=disabled` for host pass-through. |
| Replay verbose | `--replay <SPAN> --replay-verbose` | Same as replay | Requires `--replay` | Adds command/cwd/mode diagnostics, the active world toggle summary, capability warnings, and the `scopes: [...]` line. Shell warnings continue to use the `shell world-agent path` prefix so they can be distinguished from `[replay] warn:` entries. |

## Root Command: Order-Independent Flags

These flags apply before any subcommand and can appear anywhere before the first positional/subcommand token.

| Flag | Applies To | Pairings To Cover | Incompatible Paths | Notes |
| --- | --- | --- | --- | --- |
| `--ci` | Interactive, `-c`, `-f` | Combine with/without `--no-exit-on-error`, with world toggles | Info-only commands | Strict exit behavior (`cli.rs:45`). |
| `--no-exit-on-error` | Same as `--ci` | Pair with `--ci` toggled both ways | Info-only commands | Overrides CI default (`cli.rs:49`). |
| `--pty` (Unix) | Interactive + `-c` | With CI, with shell override, with anchor toggles | Hidden on Windows | Manual PTY control (`cli.rs:53`). |
| `--shell <PATH>` | Interactive, `-c`, `-f` | Use with PTY on/off, CI on/off | Info-only commands | Forces specific shell (`cli.rs:58`). |
| `--shim-skip` | All execution/replay paths | Paired with CI, world toggles | None | Bypass deployment guard (`cli.rs:74`). |
| `--world` | All primary modes + subcommands | With `--caged`, anchor flags, replay | `--no-world` | Forces isolation even if disabled (`cli.rs:130`). |
| `--no-world` | Same coverage as `--world` | With `--caged`, anchor flags, replay | `--world` | Host pass-through (`cli.rs:139`). |
| `--caged` | Execution & replay | All anchor modes | `--uncaged` | Lock cwd to anchor (`cli.rs:106`). |
| `--uncaged` | Execution & replay | Combine with follow-cwd/custom anchors | `--caged` | Free movement (`cli.rs:110`). |
| `--anchor-mode <project|follow-cwd|custom>` | Execution & replay | Test with each enum, plus `--anchor-path` for custom | None | Sets anchor selection (`cli.rs:114`). |
| `--anchor-path <PATH>` | Requires `--anchor-mode custom` | Validate with path + caged combos | Non-custom anchor modes | Explicit root override (`cli.rs:122`). |

## Root Command: Utility Flags (no subcommand)

| Flag | Description | Compatible Options | Conflicts | References |
| --- | --- | --- | --- | --- |
| `--shim-status` | Text shim deployment summary | `--world`, `--no-world`, `--shim-skip` | `-c`, `-f`, `--shim-deploy`, `--shim-remove` | `cli.rs:66`, `docs/USAGE.md:94`. |
| `--shim-status-json` | JSON shim status | Same as above | Same as above | `cli.rs:70`. |
| `--shim-deploy` | Force redeploy | `--world`, `--no-world`, `--shim-skip` | `-c`, `-f`, status/remove | `cli.rs:78`. |
| `--shim-remove` | Delete shim tree | Same as deploy | `-c`, `-f`, status/deploy | `cli.rs:82`. |
| `--trace`, `--replay` | See execution modes | See table above | See table above | Included here to highlight they remain root-level options. |

## Subcommands (change parsing scope)

Once you type `graph`, `world`, `shim`, or `health`, the parser stops accepting additional root flags. Everything that follows must be ordered according to the subcommand’s own definition (subcommand-level flags can be interleaved before any positional data).

### `shim` Subcommand

| Invocation | Required Positionals | Subcommand Flags | Notes |
| --- | --- | --- | --- |
| `substrate shim doctor` | — | `--json` | Manifest/PATH report honoring env overrides (`cli.rs:279`, `docs/USAGE.md:127`). |
| `substrate shim repair --manager <NAME>` | `manager` (positional value to `--manager`) | `--yes` | Appends manager snippet; no other shim actions allowed concurrently (`cli.rs:279`, `docs/USAGE.md:112`). |

### `health` Subcommand

| Invocation | Required Positionals | Subcommand Flags | Notes |
| --- | --- | --- | --- |
| `substrate health` | — | `--json` | Aggregated host/guest readiness (`cli.rs:272`, `docs/USAGE.md:140`). |

### `graph` Subcommand

| Invocation | Positional Arguments | Subcommand Flags | Key Combinations |
| --- | --- | --- | --- |
| `substrate graph ingest <FILE>` | `file` path | — | Execute with various world/anchor states. |
| `substrate graph status` | — | — | Baseline mock backend status. |
| `substrate graph what-changed <SPAN_ID>` | `span_id` (string) | `--limit <N>` | Exercise different limits plus anchor/caging toggles. |

### `world` Subcommand

| Invocation | Positional Arguments | Subcommand Flags | Notes |
| --- | --- | --- | --- |
| `substrate world doctor` | — | `--json` | Host readiness report (`cli.rs:187`). |
| `substrate world enable` | — | `--prefix`, `--profile`, `--dry-run`, `--verbose`, `--force`, `--timeout` | Provisioning control per `cli.rs:197`. |
| `substrate world deps status [TOOL...]` | Optional `tools` list (order matters) | `--json` | No tools vs explicit names, JSON toggle. |
| `substrate world deps install <TOOL...>` | One or more tool names | `--dry-run`, `--verbose` | Ensure installers respect permutations. |
| `substrate world deps sync` | — | `--all`, `--verbose` | Cover all flag combinations (up to 4 permutations). |

## Testing Checklist

- Interactive-only builtins (`cd`, `pwd`, `export`, `unset`, `exit`, `quit`) must be validated under both async and legacy loops (`docs/USAGE.md:48`).
- Replay/trace commands should be executed under all anchor modes with and without world isolation to catch regression hits in `world_root_mode` handling.
- Shim doctor, shim repair, and health commands depend on environment overrides (`HOME`, `SUBSTRATE_MANAGER_MANIFEST`, `SHIM_TRACE_LOG`); stage tests with synthetic directories to ensure they respect those variables.
- Always validate CI combinations (`--ci`, `--no-exit-on-error`) across `-c`, `-f`, and interactive sessions to ensure termination semantics remain consistent.
- Pair `--shim-skip` with every execution mode whenever shim deployment logic changes.
