# Replay Guide

This guide explains how to replay a previously traced command and compare results.

## Basics

Substrate writes command spans to a JSONL trace file (by default `~/.substrate/trace.jsonl`). Each span has a `span_id` that you can use to replay.

```
# Pretty-print a span
substrate --trace <SPAN_ID>

# Replay a span
substrate --replay <SPAN_ID>
```

## Linux Isolation

On Linux, replay is agent-first: when `/run/substrate.sock` responds, `--replay-verbose` prints `[replay] world strategy: agent (project_dir=...)`. If the agent is unavailable, replay emits a single `[replay] warn: agent replay unavailable (<cause>); falling back to local backend. Run `substrate world doctor --json` or set SUBSTRATE_WORLD_SOCKET to point at a healthy agent socket before switching to the local backend/copy-diff while still collecting `fs_diff`—even when the rest of the CLI is running with `SUBSTRATE_WORLD=disabled` (for example, when invoked via `scripts/dev/substrate_shell_driver`). Host-only runs use `--no-world` or `SUBSTRATE_REPLAY_USE_WORLD=disabled` and show `[replay] warn: running without world isolation (...)` in verbose mode so the warning sits alongside any `scopes: []` line. Replays manage their own world state so tests/harnesses do not need to touch global config files to flip modes.

```
# Replay with default world isolation (agent-first on Linux)
substrate --replay <SPAN_ID>

# Verbose output shows isolation strategy and scopes used
substrate --replay --replay-verbose <SPAN_ID>
# Example lines when verbose and the agent is healthy:
# [replay] span_id: <SPAN_ID>
# [replay] world strategy: agent (project_dir=/workspace)
# [replay] scopes: []
# [replay] world toggle: enabled (default)

# Agent unavailable (Linux only):
# [replay] warn: agent replay unavailable (connect failed: Connection refused (socket: /run/substrate.sock)); falling back to local backend. Run `substrate world doctor --json` or set SUBSTRATE_WORLD_SOCKET to point at a healthy agent socket.
# [replay] world strategy: overlay
# [replay] scopes: []

# Disable world isolation if needed (not recommended)
# Option 1: CLI flag (applies only to this invocation)
substrate --no-world --replay <SPAN_ID>

# Option 2: Environment variable
export SUBSTRATE_REPLAY_USE_WORLD=disabled
substrate --replay <SPAN_ID>

# Host-only replay with verbose output (warning included)
substrate --replay --replay-verbose --no-world <SPAN_ID>
# [replay] world toggle: disabled (--no-world flag)
# [replay] warn: running without world isolation (--no-world flag)
# [replay] scopes: []
```

Shell-side fallbacks emit `substrate: warn: shell world-agent path (<endpoint>) ...` so you can
distinguish them from `[replay] warn: ...` messages that come from the replay runtime.


By default on Linux, replay will:
- Prefer the world-agent path when `/run/substrate.sock` responds; fall back once to the local world backend/copy-diff with a single warning when the agent is unavailable
- Configure the world with `always_isolate: true`, forcing isolation for ALL commands (even simple ones like `echo` that normally wouldn't be isolated)
- Carry world-root/caging env into the replay so cwd/path alignment matches the original span when isolation is active
- Return `fs_diff` (writes/mods/deletes) and `scopes_used` from the isolated execution
- Show isolation strategy in verbose mode (overlay/fuse/copy-diff)
- Print the active world toggle in verbose mode (`[replay] world toggle: enabled (default)`), followed by `[replay] warn: running without world isolation (...)` whenever `--no-world` or `SUBSTRATE_REPLAY_USE_WORLD=disabled` opt-outs are in effect. This keeps the new `scopes: [...]` line aligned with the toggle that produced it.

### Replay world toggles

Use these toggles to exercise different replay modes without mutating global config:

- **Default (world-on)** – omit the flag/env overrides. The CLI forces `SUBSTRATE_WORLD=enabled` for the replay process so even host-only harnesses run in world isolation.
- **Per-invocation flag** – append `--no-world` when launching `substrate --replay` to keep the run entirely on the host. The verbose output includes `[replay] world toggle: disabled (--no-world flag)` followed by a warning about missing isolation.
- **Environment override** – export `SUBSTRATE_REPLAY_USE_WORLD=disabled` (or `0`/`false`) to keep the run host-only. This is useful for scripts/tests that need to toggle modes programmatically.
- **Force world** – pass `--world` to override an accidental `SUBSTRATE_REPLAY_USE_WORLD=disabled` export. This is mostly relevant when re-running commands captured from CI logs.

Both toggles work on Linux and macOS (Lima). On Windows/WSL, replay still respects the flag/env but the backend falls back to direct execution because world isolation is experimental there. Verbose output continues to show the world toggle summary even when the platform ultimately degrades to host execution.

To disable world isolation (not recommended for security reasons):
- Pass `--no-world` on the command line, or
- Set `SUBSTRATE_REPLAY_USE_WORLD=disabled` or `SUBSTRATE_REPLAY_USE_WORLD=0`

On macOS and Linux, replay uses the world backend (Lima agent on macOS, agent-first on Linux with a local backend fallback) to collect `fs_diff` and scopes. Other platforms fall back to direct execution without isolation or `fs_diff`.

### Isolation fallback & cleanup

- Linux replays attempt to install nftables policy inside a per-replay netns. When `ip netns add` fails (missing CAP_NET_ADMIN, stale namespaces, etc.), `--replay-verbose` prints a warning and falls back to socket cgroup matching. The fallback scopes nft rules to `/sys/fs/cgroup/substrate/<WORLD_ID>`, keeping host traffic untouched even though rules run in the host namespace.
- If both netns and writable cgroups are unavailable, replay disables nft scoping entirely and emits `[replay] warn: nft fallback unavailable (no netns/cgroup)` so you can remediate before rerunning.
- Use the cleanup helper to diagnose and purge leftover resources:
  ```
  # Inspect namespaces/cgroups/nft tables (no deletion)
  substrate world cleanup

  # Delete idle namespaces/cgroups/tables (Linux host)
  sudo substrate world cleanup --purge
  ```
- macOS + Lima: run the cleanup command inside the guest (`limactl shell substrate sudo substrate world cleanup --purge`). WSL users can do the same via `wsl -d substrate-wsl -- sudo substrate world cleanup --purge`.
- When cleanup needs to happen manually, follow the instructions printed by the helper (e.g., `sudo ip netns delete substrate-<WORLD_ID>`, `sudo nft delete table inet substrate_<WORLD_ID>`, `sudo rm -rf /sys/fs/cgroup/substrate/<WORLD_ID>`).

## Tips

- If the replayed command modifies files (e.g., `npm install`), run on a disposable project copy or in a clean world.
- Use `--trace <SPAN_ID>` first to confirm the command and working directory.

### Copy-diff scratch space

- Copy-diff uses a scratch root to stage the project and diff changes. The default order is: env override (`SUBSTRATE_COPYDIFF_ROOT`) → `/run` (`XDG_RUNTIME_DIR` or `/run/user/<uid>/substrate/copydiff`, `/run/substrate/copydiff` for root) → `/tmp/substrate-<uid>-copydiff` → `/var/tmp/substrate-<uid>-copydiff`. Hosts without `/run/user/<uid>` automatically skip that entry.
- ENOSPC or other copy-diff errors print a single replay warning per attempt (for example, `[replay] warn: copy-diff storage /tmp/substrate-1000-copydiff (/tmp) ran out of space; retrying fallback location`) and keep retrying the next root.
- Set `SUBSTRATE_COPYDIFF_ROOT=/path/with/space` to pin the scratch root; the warning will mention the override when it fails and the verbose output prints the root actually used (`[replay] copy-diff root: ... (env:SUBSTRATE_COPYDIFF_ROOT)`).
- Manual cleanup: remove any leftover `substrate-*-copydiff` directories under `/run`, `/tmp`, `/var/tmp`, or your override path if a replay was interrupted. These roots only hold temporary copies of the project/work trees created during replay.
