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

On Linux, replay uses the world-api backend by default for secure, consistent isolation and fs-diff collection.

```
# Replay with default world isolation
substrate --replay <SPAN_ID>

# Verbose output shows isolation strategy and scopes used
substrate --replay-verbose --replay <SPAN_ID>
# Example lines when verbose:
# [replay] world strategy: overlay
# [replay] scopes: tcp:github.com:443,tcp:registry.npmjs.org:443

# Disable world isolation if needed (not recommended)
# Option 1: CLI flag (applies only to this invocation)
substrate --no-world --replay <SPAN_ID>

# Option 2: Environment variable
export SUBSTRATE_REPLAY_USE_WORLD=disabled
substrate --replay <SPAN_ID>
```

By default on Linux, replay will:
- Use the world-api backend (LinuxLocalBackend) for secure execution
- Configure the world with `always_isolate: true`, forcing isolation for ALL commands (even simple ones like `echo` that normally wouldn't be isolated)
- Return `fs_diff` (writes/mods/deletes) and `scopes_used` from the isolated execution
- Show isolation strategy in verbose mode (overlay/fuse/copy-diff)

To disable world isolation (not recommended for security reasons):
- Pass `--no-world` on the command line, or
- Set `SUBSTRATE_REPLAY_USE_WORLD=disabled` or `SUBSTRATE_REPLAY_USE_WORLD=0`

On macOS and Linux, replay uses the world backend (Lima on macOS, local namespaces on Linux) to collect `fs_diff` and scopes. Other platforms fall back to direct execution without isolation or `fs_diff`.

## Tips

- If the replayed command modifies files (e.g., `npm install`), run on a disposable project copy or in a clean world.
- Use `--trace <SPAN_ID>` first to confirm the command and working directory.
