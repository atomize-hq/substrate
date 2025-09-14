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

# Verbose output shows isolation strategy
substrate --replay-verbose --replay <SPAN_ID>

# Disable world isolation if needed (not recommended)
export SUBSTRATE_REPLAY_USE_WORLD=disabled
substrate --replay <SPAN_ID>
```

By default on Linux, replay will:
- Use the world-api backend (LinuxLocalBackend) for secure execution
- Configure the world with `always_isolate: true`, forcing isolation for ALL commands (even simple ones like `echo` that normally wouldn't be isolated)
- Return `fs_diff` (writes/mods/deletes) and `scopes_used` from the isolated execution
- Show isolation strategy in verbose mode (overlay/fuse/copy-diff)

To disable world isolation (not recommended for security reasons):
- Set `SUBSTRATE_REPLAY_USE_WORLD=disabled` or `SUBSTRATE_REPLAY_USE_WORLD=0`

On non-Linux platforms, replay falls back to direct execution without isolation or `fs_diff`.

## Tips

- If the replayed command modifies files (e.g., `npm install`), run on a disposable project copy or in a clean world.
- Use `--trace <SPAN_ID>` first to confirm the command and working directory.

