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

## Linux Isolation (optional)

On Linux, you can ask replay to use the world backend for isolation and fs-diff collection.

```
export SUBSTRATE_REPLAY_USE_WORLD=1
substrate --replay <SPAN_ID>
```

When enabled, replay will:
- Execute in a session world
- Return `fs_diff` (writes/mods/deletes) and `scopes_used` if available

On non-Linux platforms or when isolation is disabled, replay falls back to direct execution without `fs_diff`.

## Tips

- If the replayed command modifies files (e.g., `npm install`), run on a disposable project copy or in a clean world.
- Use `--trace <SPAN_ID>` first to confirm the command and working directory.

