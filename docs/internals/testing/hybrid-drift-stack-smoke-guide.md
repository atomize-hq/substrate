# Hybrid Drift Stack Smoke Guide

This guide covers manual validation for the Hybrid Drift Sentinel stack:

- `agent-session-compactor`
- `agent-drift-analyzer`
- `agent-drift-sentinel` replay mode
- `agent-drift-sentinel` bounded live-integration surfaces

Use it when you want to prove the current stack still works end to end, or when you need to isolate
which layer regressed.

## Scope

What this guide does cover:

- crate-local targeted test suites
- a bounded real-session pipeline using one known-good rollout session
- artifact inspection commands after each stage
- expected live-mode gate behavior

What this guide does not cover:

- full-corpus `~/.codex` runs across every session on this machine
- shell/world integration for the sentinel live path
- any autonomous runtime wiring beyond the current post-`L8` boundary

## Prerequisites

- Run from the repo root.
- Have a working Rust toolchain that can build the workspace.
- Have a populated local Codex corpus at `~/.codex`.
- For the bounded real-session example below, confirm this rollout file exists:

```bash
find ~/.codex/sessions -name '*019e767c-e64b-7b93-a540-7a33a90f780f*'
```

If you want to use a different session, replace `SESSION_ID` consistently in the commands below.

Recommended setup:

```bash
export SESSION_ID=019e767c-e64b-7b93-a540-7a33a90f780f
export SMOKE_ROOT=target/hybrid-drift-smoke/$SESSION_ID
export COMPACTOR_OUT=$SMOKE_ROOT/compactor
export ANALYZER_OUT=$SMOKE_ROOT/analyzer
```

Optional cleanup before a fresh chained smoke:

```bash
rm -rf "$SMOKE_ROOT"
```

## Quick Matrix

| Layer | Primary capability | Fast smoke | Bounded real/manual smoke |
| --- | --- | --- | --- |
| Compactor | Discover sessions, normalize rows, exact dedupe, export stable bundle | `cargo test -p agent-session-compactor end_to_end -- --nocapture` | `cargo run -p agent-session-compactor -- --codex-home ~/.codex --session-id "$SESSION_ID" --output-dir "$COMPACTOR_OUT"` |
| Analyzer | Load compactor bundle, infer task frame, score drift, emit checkpoints | `cargo test -p agent-drift-analyzer end_to_end -- --nocapture` | `cargo run -p agent-drift-analyzer -- --input-dir "$COMPACTOR_OUT" --output-dir "$ANALYZER_OUT"` |
| Sentinel replay | Render replay warnings over analyzer checkpoints | `cargo test -p agent-drift-sentinel warning_policy -- --nocapture` | `cargo run -p agent-drift-sentinel -- --checkpoint-dir "$ANALYZER_OUT"` |
| Sentinel live integration | Validate bounded live seam without runtime wiring | `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture` | `cargo run -p agent-drift-sentinel -- --checkpoint-dir "$ANALYZER_OUT" --mode live` should fail with the gate message |

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
  --codex-home ~/.codex \
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

Count the row/audit files:

```bash
wc -l \
  "$COMPACTOR_OUT/rows.archival.jsonl" \
  "$COMPACTOR_OUT/rows.compact.jsonl" \
  "$COMPACTOR_OUT/dedupe-audit.jsonl"
```

Known-good bounded example for `019e767c-e64b-7b93-a540-7a33a90f780f` on this machine:

- `rows.archival.jsonl`: `267`
- `rows.compact.jsonl`: `209`
- `dedupe-audit.jsonl`: `16`
- `summary.md` reports exactly one source file and one session id

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
- scores exactly three drift classes in v0.1
- emits a replay-facing checkpoint bundle for the sentinel

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

- `summary.md` should show `Sessions analyzed: 1` for the bounded single-session smoke
- `summary.md` should show `Checkpoints emitted: <n>` with `n >= 1`
- `checkpoints.jsonl` should contain at least one checkpoint object, and may contain multiple
  progressive checkpoints for a single session
- each checkpoint should include:
  - `schema_version`
  - `checkpoint_id`
  - `boundary.start` and `boundary.end`
  - `diagnostics`
  - `task_frame`
  - `drift_scores`
  - `expected_next_step`

Known-good bounded example for `019e767c-e64b-7b93-a540-7a33a90f780f`:

- analyzer summary reports `Sessions analyzed: 1`
- analyzer summary reports `Turns observed: 1`
- analyzer summary reports `User prompts observed: 1`
- analyzer summary reports `Checkpoints emitted: 16`
- analyzer summary reports `Checkpoints per turn: 16.00`
- analyzer summary reports `Checkpoints per user prompt: 16.00`
- analyzer summary reports `Avg rows between checkpoints: 17.33`
- analyzer summary reports `Avg seconds between checkpoints: 45.40`
- analyzer summary reports `Flagged checkpoints: 8`
- analyzer summary reports `Longest flagged streak: 7`
- analyzer summary reports `Flagged checkpoint rate: 0.50`
- analyzer summary reports `Drift-class flagged frequency: wrong_plan_branch=0.44, ignoring_repo_truth=0.06, dead_end_thrash=0.00`
- analyzer summary reports `Task-frame transition count: 14`
- analyzer summary reports `Task-frame confidence distribution: low=1, medium=15, high=0`
- analyzer summary reports `Working-set churn: 0.93`
- analyzer summary reports `Verification density: 0.02`
- analyzer summary reports `Average evidence items per checkpoint: 174.31`
- the session block reports `Distinct task frames: 15`
- the session block reports `Truth artifacts referenced: 4`
- the session block reports `Verification commands observed: 0`
- the emitted checkpoints use `schema_version: "v0.2"` and include a compact `diagnostics` object
- `checkpoints.jsonl` contains progressive checkpoint ids from
  `019e767c-e64b-7b93-a540-7a33a90f780f:0001` through
  `019e767c-e64b-7b93-a540-7a33a90f780f:0016`

## 3. Agent Drift Sentinel Replay

### Capability

The replay sentinel:

- loads analyzer checkpoint bundles
- applies scheduler cooldown, heartbeat, debounce, and repeated-failure rules
- separates visible warnings from silent checkpoints
- renders a console-oriented replay report
- optionally shapes bounded adjudication requests, disabled by default

Replay mode is the only CLI-enabled mode today.

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

Known-good bounded example for `019e767c-e64b-7b93-a540-7a33a90f780f`:

- `Processed checkpoints: 1`
- `Visible warnings: 1`
- `Silent checkpoints: 0`
- `Next cursor: 019e767c-e64b-7b93-a540-7a33a90f780f:1`

The current sample warning is a high-severity `wrong_plan_branch` visible warning rendered through
the `repeated_failure` trigger path.

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

## 4. Agent Drift Sentinel Live Integration

### Capability

The current live-integration slice is intentionally bounded and library-first:

- it defines an incremental live checkpoint event contract
- it validates fixture-backed append-only event intake
- it reuses shared scheduler and presentation logic in a `LiveRuntime`
- it emits structured operator events through a sink surface
- it proves the seam with a fixture-driven end-to-end test

It does **not** provide a CLI-enabled live runtime. The CLI gate must still hold.

### Fast live-seam smoke

```bash
cargo test -p agent-drift-sentinel live_input -- --nocapture
cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

### What the bounded live proof covers

The fixture-backed end-to-end proof uses:

- `crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl`

The expected event sequence is:

1. visible warning from `checkpoint_ready`
2. heartbeat event
3. silent checkpoint event
4. manual-review status event

If `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture` passes, the bounded `L7`
proof is intact.

### Manual gate smoke: live CLI must still fail

This is the correct negative smoke today:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT" \
  --mode live
```

Expected result:

```text
Error: live mode remains gated by S10; replay usefulness review must pass first
```

That failure is not a regression. It proves the post-`L8` runtime gate is still enforced.

## 5. Full Chained Smoke

Use this when you want one bounded proof across all four layers.

```bash
export SESSION_ID=019e767c-e64b-7b93-a540-7a33a90f780f
export SMOKE_ROOT=target/hybrid-drift-smoke/$SESSION_ID
export COMPACTOR_OUT=$SMOKE_ROOT/compactor
export ANALYZER_OUT=$SMOKE_ROOT/analyzer

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home ~/.codex \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Success means:

- compactor emitted the five-file bundle
- analyzer emitted `checkpoints.jsonl` and `summary.md`
- sentinel replay rendered a visible warning report
- live integration passed the fixture-driven bounded proof

## 6. Failure Triage

### Compactor fails

Check these first:

- does `~/.codex` exist and contain rollout JSONL files?
- did the output directory already contain stale test artifacts you meant to remove?
- does `manifest.json` exist without the other four files? If so, that is a bug.

Useful commands:

```bash
find ~/.codex/sessions -name "rollout-*.jsonl" | head
find "$(dirname "$COMPACTOR_OUT")" -maxdepth 1 -type d | sort
```

### Analyzer fails

Check these first:

- is the compactor bundle missing one of the five required files?
- does `rows.compact.jsonl` exist and contain JSONL rows?
- did you accidentally point `--input-dir` at the analyzer output directory instead of the compactor output directory?

Useful commands:

```bash
ls -1 "$COMPACTOR_OUT"
sed -n '1,5p' "$COMPACTOR_OUT/rows.compact.jsonl"
```

### Sentinel replay fails

Check these first:

- does `"$ANALYZER_OUT/checkpoints.jsonl"` exist?
- does the checkpoint JSON include the fields shown above?
- are you accidentally passing `--mode live`?

Useful commands:

```bash
ls -1 "$ANALYZER_OUT"
sed -n '1,3p' "$ANALYZER_OUT/checkpoints.jsonl"
```

### Live integration fails

Interpret the failure by path:

- `live_input*` failures usually mean fixture ordering/cursor rules regressed
- `live_checkpoint_compatibility` failures usually mean the analyzer checkpoint contract changed
- `live_runtime` or `operator_sink` failures usually mean scheduler/presentation reuse drifted
- `live_end_to_end` failures usually mean the bounded operator-event sequence changed
- `cargo run ... --mode live` succeeding would be a scope regression today

## 7. Current Known-Good Commands

These commands were re-validated in this worktree while writing this guide:

```bash
cargo test -p agent-session-compactor end_to_end -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture

cargo run -p agent-session-compactor -- \
  --codex-home ~/.codex \
  --session-id 019e767c-e64b-7b93-a540-7a33a90f780f \
  --output-dir target/hybrid-drift-smoke/019e767c-e64b-7b93-a540-7a33a90f780f/compactor

cargo run -p agent-drift-analyzer -- \
  --input-dir target/hybrid-drift-smoke/019e767c-e64b-7b93-a540-7a33a90f780f/compactor \
  --output-dir target/hybrid-drift-smoke/019e767c-e64b-7b93-a540-7a33a90f780f/analyzer

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir target/hybrid-drift-smoke/019e767c-e64b-7b93-a540-7a33a90f780f/analyzer

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir target/hybrid-drift-smoke/019e767c-e64b-7b93-a540-7a33a90f780f/analyzer \
  --mode live
```
