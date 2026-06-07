# Hybrid Drift Stack Smoke Guide

Use this guide when you want to validate the Hybrid Drift path for one real Codex session:

`rollout-*.jsonl -> agent-session-compactor -> agent-drift-analyzer -> agent-drift-sentinel`

This is for single-session smoke validation. It is not a corpus-wide run.

## Recommended Path

Use the helper script:

```bash
scripts/dev/hybrid-drift-live.sh <session-id>
```

Example:

```bash
scripts/dev/hybrid-drift-live.sh 019e93f8-a5e9-7490-ac1a-955b74c92ad0
```

What the script does:

- resolves the rollout file for the session id under `CODEX_HOME`
- derives `target/hybrid-drift-smoke/<session-id>` and `target/hybrid-drift-live/<session-id>`
- runs compactor once
- runs analyzer once
- runs sentinel replay once
- starts sentinel live and leaves it running until you stop it with `Ctrl-C`

Useful options:

```bash
scripts/dev/hybrid-drift-live.sh --fresh-state <session-id>
scripts/dev/hybrid-drift-live.sh --skip-static <session-id>
scripts/dev/hybrid-drift-live.sh --no-growth-check <session-id>
scripts/dev/hybrid-drift-live.sh --codex-home /path/to/.codex <session-id>
```

Use `--fresh-state` when you want to discard prior live dedupe state and replay the session from the start.

## What Live Mode Actually Does

`agent-drift-sentinel --mode live` is a long-running poll loop. It does not exit on its own after the first checkpoint burst.

It will usually behave like this:

1. print the session header and resolved rollout path
2. rerun the pipeline when the rollout file grows
3. emit only checkpoints newer than the last delivered cursor in `LIVE_STATE_DIR`
4. stay quiet when the rollout has not changed
5. print `emitted 0 new checkpoint(s)` when the rollout grew but nothing new passed dedupe

If you wrap it in `timeout 8 ...` or a `sleep 8; kill "$pid"` shell wrapper, you are terminating it yourself. That is useful for bounded proof, not for watching it live.

## Manual Path

Run these commands from the repo root when you want the exact steps instead of the helper script.

### 1. Pick a real active session

```bash
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"

stat_mtime_path() {
  if stat -f '%m %N' "$1" >/dev/null 2>&1; then
    stat -f '%m %N' "$1"
  else
    stat -c '%Y %n' "$1"
  fi
}

find "$CODEX_HOME/sessions" -name 'rollout-*.jsonl' -type f -print0 |
  while IFS= read -r -d '' path; do
    stat_mtime_path "$path"
  done \
  | sort -n \
  | tail -n 10
```

Choose a session id from an actually active Codex session.

### 2. Set derived paths

```bash
export SESSION_ID="<active-session-id>"
mapfile -t ROLLOUT_MATCHES < <(
  find "$CODEX_HOME/sessions" -name "rollout-*${SESSION_ID}*.jsonl" -type f | sort
)

if [ "${#ROLLOUT_MATCHES[@]}" -eq 0 ]; then
  echo "no rollout artifact found for $SESSION_ID" >&2
  exit 1
fi

if [ "${#ROLLOUT_MATCHES[@]}" -gt 1 ]; then
  printf 'multiple rollout artifacts matched %s:\n' "$SESSION_ID" >&2
  printf '  %s\n' "${ROLLOUT_MATCHES[@]}" >&2
  exit 1
fi

export ROLLOUT_PATH="${ROLLOUT_MATCHES[0]}"

export SMOKE_ROOT="target/hybrid-drift-smoke/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"
```

This matters: `LIVE_STATE_DIR` must match the current `SESSION_ID`. If it still points at another session, sentinel will fail closed on persisted state mismatch.

### 3. Check whether the rollout is moving

```bash
stat_size_path() {
  if stat -f '%z %N' "$1" >/dev/null 2>&1; then
    stat -f '%z %N' "$1"
  else
    stat -c '%s %n' "$1"
  fi
}

stat_size_path "$ROLLOUT_PATH"
sleep 3
stat_size_path "$ROLLOUT_PATH"
```

If the size changes, you have a real live source. If it does not change, sentinel can still attach, but you should not treat quiet output as live proof yet.

### 4. Optional static preflight

This proves each stage once before you start the long-running live process.

```bash
rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"
```

### 5. Watch it live

This is the command to keep running in the foreground:

```bash
cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

Stop it with `Ctrl-C`.

## Bounded Proof Commands

These are for short proof runs. They are expected to terminate the process.

Portable bounded wrapper:

```bash
sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```

GNU `timeout` equivalent:

```bash
timeout 8 cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

Expected outcome:

- `Terminated: 15` from the portable wrapper means the shell sent `SIGTERM`
- exit `124` from `timeout` means `timeout` ended the process

Neither outcome means live sentinel crashed.

## Fast Test Ladder

Use this when you want targeted crate-local proof before the manual smoke.

```bash
cargo test -p agent-session-compactor end_to_end -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
```

## Expected Artifacts

Static preflight should leave:

- `target/hybrid-drift-smoke/<session-id>/compactor/`
- `target/hybrid-drift-smoke/<session-id>/analyzer/`

Live mode should leave:

- `target/hybrid-drift-live/<session-id>/live-session-state.json`
- `target/hybrid-drift-live/<session-id>/compactor/`
- `target/hybrid-drift-live/<session-id>/analyzer/`

Useful inspection commands:

```bash
find "$SMOKE_ROOT" -maxdepth 2 -type f | sort
find "$LIVE_STATE_DIR" -maxdepth 2 -type f | sort
sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
sed -n '1,80p' "$LIVE_STATE_DIR/analyzer/summary.md"
sed -n '1,5p' "$LIVE_STATE_DIR/analyzer/checkpoints.jsonl"
```

## Common Failure Modes

### Persisted state belongs to another session

You reused `LIVE_STATE_DIR` from an older session.

Fix:

```bash
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"
```

If you want a clean restart:

```bash
rm -rf "$LIVE_STATE_DIR"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"
```

### Live run prints `emitted 0 new checkpoint(s)`

That usually means one of two things:

- the rollout grew, but no new checkpoints were fresh after dedupe
- you restarted against existing `LIVE_STATE_DIR`, so already-delivered checkpoints were suppressed

That is expected unless you wanted a fresh replay from the beginning. Use `--fresh-state` with the helper script or remove `LIVE_STATE_DIR` manually.

### Live run “just ends”

If you used `timeout` or the `sh -c ... kill ...` wrapper, you ended it yourself. Run the foreground live command with no wrapper if you want to watch it.

### `MissingRolloutArtifact`

The session id is wrong, the session has no rollout file yet, or `CODEX_HOME` is pointed at the wrong tree.

### `AmbiguousRolloutArtifacts`

The session id filter matched more than one rollout file. Use the full session id, not a prefix.

### `RolloutShrank`

The source file rotated or truncated, or you are watching the wrong session.

## Current Command Set

```bash
scripts/dev/hybrid-drift-live.sh <session-id>

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```
