# Hybrid Drift Stack Smoke Guide

Use this guide to run a bounded manual smoke across the Hybrid Drift stack:

- `agent-session-compactor`
- `agent-drift-analyzer`
- `agent-drift-sentinel` replay mode
- `agent-drift-sentinel` real-session live mode

It is for targeted single-session validation, not whole-corpus runs.

## Scope

This guide covers:

- crate-local regression tests
- a bounded single-session compactor -> analyzer -> replay pipeline
- a bounded live smoke over one actually growing rollout file
- the key artifact checks after each stage

This guide does not cover:

- full `~/.codex` corpus sweeps
- `shell` / `world` / `shim` integration
- multi-session dashboards or fan-in
- broader host-runtime wiring beyond the current live slice

## Prerequisites

- Run from the repo root.
- Have a working Rust toolchain.
- Have a real `CODEX_HOME` with local Codex sessions.
- Have one actually active session whose `rollout-*.jsonl` file is still growing.

Setup:

```bash
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"

find "$CODEX_HOME/sessions" -name 'rollout-*.jsonl' -type f -print0 \
  | xargs -0 stat -f '%m %N' \
  | sort -n \
  | tail -n 10
```

Pick the session you want, then set:

```bash
export SESSION_ID="<active-session-id>"
export ROLLOUT_PATH="$(find "$CODEX_HOME/sessions" -name "rollout-*${SESSION_ID}*.jsonl" | head -n 1)"

export SMOKE_ROOT="target/hybrid-drift-smoke/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"
```

Confirm the rollout is genuinely moving before trusting any live proof:

```bash
stat -f '%z %N' "$ROLLOUT_PATH"
sleep 3
stat -f '%z %N' "$ROLLOUT_PATH"
```

If the size does not change, the live proof is not valid yet.

Optional cleanup:

```bash
rm -rf "$SMOKE_ROOT" "$LIVE_STATE_DIR"
```

## Quick Matrix

| Layer | Fast smoke | Bounded smoke |
| --- | --- | --- |
| Compactor | `cargo test -p agent-session-compactor end_to_end -- --nocapture` | `cargo run -p agent-session-compactor -- --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --output-dir "$COMPACTOR_OUT"` |
| Analyzer | `cargo test -p agent-drift-analyzer end_to_end -- --nocapture` | `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"` |
| Sentinel replay | `cargo test -p agent-drift-sentinel warning_policy -- --nocapture` | `cargo run -p agent-drift-sentinel -- --checkpoint-dir "$ANALYZER_OUT"` |
| Sentinel live | `cargo test -p agent-drift-sentinel real_session_live -- --nocapture` | `sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'` |

## Quick Start

Run this when you want one end-to-end bounded proof:

```bash
rm -rf "$SMOKE_ROOT" "$LIVE_STATE_DIR"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```

Success means:

- compactor emitted the five-file bundle
- analyzer emitted `checkpoints.jsonl` and `summary.md`
- replay rendered a coherent report over the analyzer bundle
- live mode attached to a truly growing rollout and emitted real checkpoint output or a legitimate sparse-startup `0`-checkpoint poll

## 1. Compactor

Use this to prove bounded export for one session.

Fast smoke:

```bash
cargo build -p agent-session-compactor
cargo test -p agent-session-compactor export_bundle -- --nocapture
cargo test -p agent-session-compactor end_to_end -- --nocapture
```

Optional broader ladder:

```bash
cargo test -p agent-session-compactor rollout_ingest -- --nocapture
cargo test -p agent-session-compactor normalization -- --nocapture
cargo test -p agent-session-compactor dedupe -- --nocapture
cargo test -p agent-session-compactor -- --nocapture
```

Bounded run:

```bash
cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"
```

Check:

```bash
ls -1 "$COMPACTOR_OUT"
sed -n '1,120p' "$COMPACTOR_OUT/manifest.json"
printf '\n---\n'
sed -n '1,80p' "$COMPACTOR_OUT/summary.md"
wc -l \
  "$COMPACTOR_OUT/rows.archival.jsonl" \
  "$COMPACTOR_OUT/rows.compact.jsonl" \
  "$COMPACTOR_OUT/dedupe-audit.jsonl"
find "$(dirname "$COMPACTOR_OUT")" -maxdepth 1 -type d \
  \( -name ".$(basename "$COMPACTOR_OUT").staging-*" -o -name ".$(basename "$COMPACTOR_OUT").backup-*" \) \
  | sort
```

Look for:

- exactly one session id in `manifest.json`
- non-empty `rows.archival.jsonl` and `rows.compact.jsonl`
- present `dedupe-audit.jsonl`
- no published staging or backup sibling directories

## 2. Analyzer

Use this to prove the compactor bundle is analyzable and exports progressive checkpoints.

Fast smoke:

```bash
cargo build -p agent-drift-analyzer
cargo test -p agent-drift-analyzer input_contract -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Optional broader ladder:

```bash
cargo test -p agent-drift-analyzer context_assembly -- --nocapture
cargo test -p agent-drift-analyzer task_frame -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Bounded run:

```bash
cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"
```

Check:

```bash
ls -1 "$ANALYZER_OUT"
sed -n '1,120p' "$ANALYZER_OUT/summary.md"
printf '\n---\n'
sed -n '1,120p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Look for:

- `Sessions analyzed: 1`
- `Checkpoints emitted: <n>` with `n >= 1` once the session is rich enough
- `checkpoints.jsonl` entries with `schema_version`, `checkpoint_id`, `boundary`, `diagnostics`, `task_frame`, `drift_scores`, and `expected_next_step`
- `v0.3` checkpoints carrying analyzer-owned `DriftState` on every `DriftScore`

## 3. Sentinel Replay

Use this to validate the static analyzer bundle surface.

Replay mode:

- applies scheduler cooldown, heartbeat, debounce, and repeated-failure rules
- separates visibility from posture
- prefers analyzer-exported `DriftState` on `v0.3`
- uses the older previous-checkpoint plus historical-reason-prefix logic only for `v0.2`

Fast smoke:

```bash
cargo build -p agent-drift-sentinel
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel adjudication -- --nocapture
```

Optional broader replay ladder:

```bash
cargo test -p agent-drift-sentinel scheduler -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel adjudication_fallback -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Bounded run:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"
```

Optional cursor smoke:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT" \
  --cursor-session-id "$SESSION_ID" \
  --cursor-ordinal 1
```

Check:

- `Processed checkpoints`
- `Visible warnings`
- `Silent checkpoints`
- `Next cursor`
- whether `Posture` is reported independently from `Visible` versus `Silent`

Optional adjudication-shaping smoke:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT" \
  --enable-model-adjudication \
  --model gpt-5.4-mini \
  --reasoning-effort medium
```

## 4. Sentinel Live

Use this to prove the real-session live seam over one active session.

Live mode:

- discovers one rollout artifact for the target `session_id`
- reruns compactor and analyzer through library APIs
- emits only checkpoints strictly after the last delivered cursor within one live process
- prints live console blocks using the shared replay/live presentation surface

Fast smoke:

```bash
cargo build -p agent-drift-sentinel
cargo test -p agent-drift-sentinel live_input -- --nocapture
cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Portable bounded run:

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

Check:

- header lines for session, rollout artifact, and state dir
- sparse startup only while the rollout is still too early to analyze
- once the rollout grows and is analyzable, one or more emitted checkpoints
- later polls may emit `0` new checkpoints, which is expected dedupe within the same process

Expected bounded exit:

- `143` for the portable `sh -c ... kill ... wait` form
- `124` for the GNU `timeout` form

Neither is a failure here.

Inspect the live state dir:

```bash
find "$LIVE_STATE_DIR" -maxdepth 2 -type f | sort
printf '\n--- analyzer summary ---\n'
sed -n '1,80p' "$LIVE_STATE_DIR/analyzer/summary.md"
printf '\n--- first checkpoints ---\n'
sed -n '1,5p' "$LIVE_STATE_DIR/analyzer/checkpoints.jsonl"
```

Look for:

- both `compactor/` and `analyzer/` artifacts
- analyzer output scoped to the target session
- live console output driven by real rollout growth
- posture that is not conflated with visible versus silent presentation

## Failure Triage

### Compactor fails

Check:

- does `"$CODEX_HOME"` exist with rollout files?
- did you leave stale test artifacts in the output dir?
- does `manifest.json` exist without the other four files?

Useful commands:

```bash
find "$CODEX_HOME/sessions" -name "rollout-*.jsonl" | head
find "$(dirname "$COMPACTOR_OUT")" -maxdepth 1 -type d | sort
```

### Analyzer fails

Check:

- is one of the five required compactor files missing?
- does `rows.compact.jsonl` exist and contain JSONL?
- did you point `--input-dir` at the wrong directory?

Useful commands:

```bash
ls -1 "$COMPACTOR_OUT"
sed -n '1,5p' "$COMPACTOR_OUT/rows.compact.jsonl"
```

### Sentinel replay fails

Check:

- does `"$ANALYZER_OUT/checkpoints.jsonl"` exist?
- does the checkpoint JSON include the expected fields?
- are you passing live-only flags by mistake?

Useful commands:

```bash
ls -1 "$ANALYZER_OUT"
sed -n '1,3p' "$ANALYZER_OUT/checkpoints.jsonl"
```

### Sentinel live fails

Interpret by path:

- `real_session_live` failures usually mean session discovery, startup readiness, or delta delivery regressed
- `live_input*` failures usually mean fixture ordering or cursor rules regressed
- `live_checkpoint_compatibility` failures usually mean the analyzer checkpoint contract changed
- `live_runtime` or `operator_sink` failures usually mean scheduler or presentation reuse drifted
- `MissingRolloutArtifact` usually means `SESSION_ID` is wrong or the file is gone
- `AmbiguousRolloutArtifacts` usually means your session filter matched more than one rollout file
- `RolloutShrank` usually means the source file rotated, truncated, or the wrong session was picked
- `NoSessions` is only a legitimate first-poll state when the rollout still has no real session activity beyond `session_meta`
- `InsufficientContract` is only a legitimate sparse-startup state when the rollout genuinely lacks directive text, path hints, or parseable tool-call arguments

## Current Command Set

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```
