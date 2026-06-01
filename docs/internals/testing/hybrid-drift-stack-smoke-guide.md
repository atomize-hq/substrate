# Hybrid Drift Stack Smoke Guide

This guide covers manual validation for the Hybrid Drift Sentinel stack:

- `agent-session-compactor`
- `agent-drift-analyzer`
- `agent-drift-sentinel` replay mode
- `agent-drift-sentinel` real-session live mode over one active Codex session

Use it when you want to prove the current stack still works end to end, or when you need to isolate
which layer regressed.

## Scope

What this guide does cover:

- crate-local targeted test suites
- a bounded single-session pipeline through compactor, analyzer, and sentinel replay
- a bounded real-session live smoke over one actually active rollout file under `CODEX_HOME`
- artifact inspection commands after each stage
- expected live-session startup, delta-delivery, and bounded-proof behavior

What this guide does not cover:

- full-corpus `~/.codex` runs across every session on this machine
- shell/world integration for the sentinel live path
- multi-session dashboards or fan-in
- any broader host-runtime wiring beyond the current post-`L8` boundary

## Prerequisites

- Run from the repo root.
- Have a working Rust toolchain that can build the workspace.
- Have a populated local Codex corpus at `~/.codex` or another real `CODEX_HOME`.
- Have one actually active Codex session whose `rollout-*.jsonl` file is still growing while you
  run the live smoke.

Recommended setup:

```bash
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"

find "$CODEX_HOME/sessions" -name 'rollout-*.jsonl' -type f -print0 \
  | xargs -0 stat -f '%m %N' \
  | sort -n \
  | tail -n 10
```

Pick the active session you want to monitor, then set:

```bash
export SESSION_ID="<active-session-id>"
export ROLLOUT_PATH="$(find "$CODEX_HOME/sessions" -name "rollout-*${SESSION_ID}*.jsonl" | head -n 1)"

export SMOKE_ROOT="target/hybrid-drift-smoke/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"
```

Before trusting the live proof, confirm the rollout file is genuinely moving:

```bash
stat -f '%z %N' "$ROLLOUT_PATH"
sleep 3
stat -f '%z %N' "$ROLLOUT_PATH"
```

If the size does not change, keep the source Codex session active and retry until it does.

Optional cleanup before a fresh chained smoke:

```bash
rm -rf "$SMOKE_ROOT" "$LIVE_STATE_DIR"
```

## Quick Matrix

| Layer | Primary capability | Fast smoke | Bounded real/manual smoke |
| --- | --- | --- | --- |
| Compactor | Discover sessions, normalize rows, exact dedupe, export stable bundle | `cargo test -p agent-session-compactor end_to_end -- --nocapture` | `cargo run -p agent-session-compactor -- --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --output-dir "$COMPACTOR_OUT"` |
| Analyzer | Load compactor bundle, infer task frame, score drift, emit checkpoints | `cargo test -p agent-drift-analyzer end_to_end -- --nocapture` | `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"` |
| Sentinel replay | Render replay warnings over analyzer checkpoints | `cargo test -p agent-drift-sentinel warning_policy -- --nocapture` | `cargo run -p agent-drift-sentinel -- --checkpoint-dir "$ANALYZER_OUT"` |
| Sentinel live | Monitor one active session through compactor/analyzer libraries and emit only new checkpoints | `cargo test -p agent-drift-sentinel real_session_live -- --nocapture` | `sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'` |

## 1. Agent Session Compactor

### Capability

The compactor:

- discovers rollout JSONL artifacts under a Codex home
- ingests current rollout records
- emits deterministic archival rows and compact rows
- performs exact dedupe with audit output
- exports a five-file analyzer-facing bundle
- publishes through a staging directory so incomplete runs do not look complete

Expected final bundle contract:

- `manifest.json`
- `rows.archival.jsonl`
- `rows.compact.jsonl`
- `dedupe-audit.jsonl`
- `summary.md`

### Fast crate smoke

Run the narrow contract and end-to-end tests first:

```bash
cargo build -p agent-session-compactor
cargo test -p agent-session-compactor export_bundle -- --nocapture
cargo test -p agent-session-compactor end_to_end -- --nocapture
```

If you want the broader compactor ladder:

```bash
cargo test -p agent-session-compactor rollout_ingest -- --nocapture
cargo test -p agent-session-compactor normalization -- --nocapture
cargo test -p agent-session-compactor dedupe -- --nocapture
cargo test -p agent-session-compactor -- --nocapture
```

### Bounded real-session smoke

```bash
cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"
```

### What success looks like

List the exported files:

```bash
ls -1 "$COMPACTOR_OUT"
```

Inspect the manifest and summary:

```bash
sed -n '1,120p' "$COMPACTOR_OUT/manifest.json"
printf '\n---\n'
sed -n '1,80p' "$COMPACTOR_OUT/summary.md"
```

Count the row and audit files:

```bash
wc -l \
  "$COMPACTOR_OUT/rows.archival.jsonl" \
  "$COMPACTOR_OUT/rows.compact.jsonl" \
  "$COMPACTOR_OUT/dedupe-audit.jsonl"
```

Expected success signatures:

- `manifest.json` reports exactly one session id for the bounded run
- `rows.archival.jsonl` and `rows.compact.jsonl` both exist and are non-empty once the source
  session has meaningful activity
- `dedupe-audit.jsonl` exists even if the bounded run had few or no duplicates
- `summary.md` reports the expected bundle counts for one scoped session

Verify the hardened publish behavior did not leave a published staging or backup sibling:

```bash
find "$(dirname "$COMPACTOR_OUT")" -maxdepth 1 -type d \
  \( -name ".$(basename "$COMPACTOR_OUT").staging-*" -o -name ".$(basename "$COMPACTOR_OUT").backup-*" \) \
  | sort
```

For a successful bounded run, that command should print nothing.

## 2. Agent Drift Analyzer

### Capability

The analyzer:

- loads the compactor bundle as its only input surface
- assembles a deterministic per-session context pack
- infers a task frame from the compacted history
- scores drift and emits replay-facing checkpoints
- exports `checkpoints.jsonl` plus `summary.md`

Expected analyzer output contract:

- `checkpoints.jsonl`
- `summary.md`

### Fast crate smoke

```bash
cargo build -p agent-drift-analyzer
cargo test -p agent-drift-analyzer input_contract -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

If you want the broader analyzer ladder:

```bash
cargo test -p agent-drift-analyzer context_assembly -- --nocapture
cargo test -p agent-drift-analyzer task_frame -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer ignoring_repo_truth -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Bounded manual smoke from compactor output

```bash
cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"
```

### What success looks like

List the analyzer artifacts:

```bash
ls -1 "$ANALYZER_OUT"
```

Inspect the summary and the first checkpoint:

```bash
sed -n '1,120p' "$ANALYZER_OUT/summary.md"
printf '\n---\n'
sed -n '1,120p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Expected success signatures:

- `summary.md` shows `Sessions analyzed: 1` for the bounded single-session smoke
- `summary.md` shows `Checkpoints emitted: <n>` with `n >= 1` once the source session is rich
  enough for analysis
- `checkpoints.jsonl` contains one or more progressive checkpoint objects for that session
- each checkpoint includes `schema_version`, `checkpoint_id`, `boundary`, `diagnostics`,
  `task_frame`, `drift_scores`, and `expected_next_step`

## 3. Agent Drift Sentinel Replay

### Capability

The replay sentinel:

- loads analyzer checkpoint bundles
- applies scheduler cooldown, heartbeat, debounce, and repeated-failure rules
- separates visible warnings from silent checkpoints
- renders a console-oriented replay report
- optionally shapes bounded adjudication requests, disabled by default

Replay mode remains the manual static-bundle surface. Live mode is a separate real-session path.

### Fast crate smoke

```bash
cargo build -p agent-drift-sentinel
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel adjudication -- --nocapture
```

If you want the broader replay ladder:

```bash
cargo test -p agent-drift-sentinel scheduler -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel adjudication_fallback -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

### Bounded replay smoke from analyzer output

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

That cursor run should process nothing after the first bounded single-session checkpoint.

### What success looks like

The replay command prints a console report. Inspect these fields first:

- `Processed checkpoints`
- `Visible warnings`
- `Silent checkpoints`
- `Next cursor`
- the warning block with `Objective`, `Drift`, `Expected next step`, and `Evidence`

Success means the replay report is internally consistent with the analyzer bundle you just built.

### Optional adjudication-shaping smoke

The CLI can shape adjudication requests, but the safe default is still off. To prove the shaping
path without changing the replay contract:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT" \
  --enable-model-adjudication \
  --model gpt-5.4-mini \
  --reasoning-effort medium
```

Success means the replay report still renders and a `Prepared model adjudication requests:` section
appears after it.

## 4. Agent Drift Sentinel Real-Session Live Mode

### Capability

The current live slice is real-session, bounded, and library-first:

- it discovers one active `rollout-*.jsonl` artifact for a target `session_id`
- it reruns the compactor and analyzer crates through their library APIs into a bounded state dir
- it keeps polling when the session is still in a legitimate sparse startup state
- it surfaces real compactor/analyzer contract failures instead of broadly hiding them
- it emits only checkpoints strictly after the last delivered cursor within one live process
- it prints live console blocks using the shared scheduler and presentation surfaces

It does **not** integrate with `shell`, `world`, `shim`, or any broader host-runtime wiring.

### Fast live-surface smoke

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

### Bounded real live-session smoke

Use the portable stock-shell form by default:

```bash
sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```

If you have GNU `timeout`, this is equivalent:

```bash
timeout 8 cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

### What success looks like

The live command should print a header like:

- `# Agent Drift Sentinel Live`
- `Session: ...`
- `Rollout artifact: ...`
- `State dir: ...`

Then interpret progress by phase:

1. Sparse startup is allowed.
   - If the rollout is still too early to analyze, polls may rerun the pipeline and emit `0` new
     checkpoints.
   - This is acceptable only while the rollout still lacks analyzer-usable session activity,
     directive text, path hints, or parseable tool-call arguments.
2. Once the session becomes analyzable and the rollout grows, the live command should emit one or
   more checkpoints.
3. Later growth polls may rerun the pipeline and emit `0` new checkpoints, which is the expected
   proof that already delivered checkpoints are not replayed within the same live process.

Expected bounded-exit behavior:

- the portable `sh -c ... kill ... wait` form usually exits `143`
- the GNU `timeout` form usually exits `124`

Neither exit code is a sentinel failure here; both mean the bounded proof intentionally stopped an
otherwise infinite live monitor.

Inspect the bounded live state directory after the run:

```bash
find "$LIVE_STATE_DIR" -maxdepth 2 -type f | sort
printf '\n--- analyzer summary ---\n'
sed -n '1,80p' "$LIVE_STATE_DIR/analyzer/summary.md"
printf '\n--- first checkpoints ---\n'
sed -n '1,5p' "$LIVE_STATE_DIR/analyzer/checkpoints.jsonl"
```

Success means:

- the state dir contains `compactor/` and `analyzer/` artifacts
- the analyzer artifacts are scoped to the target session
- the live console output reflects real rollout growth rather than static replay only

## 5. Full Chained Smoke

Use this when you want one bounded proof across all four layers.

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
- sentinel replay rendered a replay report over the bounded bundle
- the live proof attached to the real rollout artifact and emitted real checkpoint output or a
  legitimate sparse-startup `0`-checkpoint poll before later growth

## 6. Failure Triage

### Compactor fails

Check these first:

- does `"$CODEX_HOME"` exist and contain rollout JSONL files?
- did the output directory already contain stale test artifacts you meant to remove?
- does `manifest.json` exist without the other four files? If so, that is a bug.

Useful commands:

```bash
find "$CODEX_HOME/sessions" -name "rollout-*.jsonl" | head
find "$(dirname "$COMPACTOR_OUT")" -maxdepth 1 -type d | sort
```

### Analyzer fails

Check these first:

- is the compactor bundle missing one of the five required files?
- does `rows.compact.jsonl` exist and contain JSONL rows?
- did you accidentally point `--input-dir` at the analyzer output directory instead of the
  compactor output directory?

Useful commands:

```bash
ls -1 "$COMPACTOR_OUT"
sed -n '1,5p' "$COMPACTOR_OUT/rows.compact.jsonl"
```

### Sentinel replay fails

Check these first:

- does `"$ANALYZER_OUT/checkpoints.jsonl"` exist?
- does the checkpoint JSON include the expected fields?
- are you accidentally passing live-only flags such as `--session-id` or `--codex-home`?

Useful commands:

```bash
ls -1 "$ANALYZER_OUT"
sed -n '1,3p' "$ANALYZER_OUT/checkpoints.jsonl"
```

### Sentinel live fails

Interpret the failure by path:

- `real_session_live` test failures usually mean session discovery, startup readiness, or
  checkpoint-delta delivery regressed
- `live_input*` failures usually mean fixture ordering or cursor rules regressed
- `live_checkpoint_compatibility` failures usually mean the analyzer checkpoint contract changed
- `live_runtime` or `operator_sink` failures usually mean scheduler or presentation reuse drifted
- `MissingRolloutArtifact` usually means `SESSION_ID` is wrong or the target file is gone
- `AmbiguousRolloutArtifacts` usually means your session filter matched more than one rollout file
- `RolloutShrank` usually means the source file rotated, truncated, or the wrong session was picked
- `NoSessions` can be a legitimate first-poll startup state only when the rollout still has no
  real session activity beyond `session_meta`
- `InsufficientContract` is only a legitimate sparse-startup state when the rollout genuinely lacks
  directive text, path hints, or parseable tool-call arguments; after real activity exists, treat
  it as an upstream contract problem rather than a sentinel retry case

## 7. Current Command Set

These commands reflect the current Packet 18 surfaces:

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
