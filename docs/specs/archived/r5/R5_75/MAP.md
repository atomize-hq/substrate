# R5.75 Map: Sequential Pre-R6 Hardening And Validation

Status: draft map created on 2026-06-12 to turn the adopted post-`R5.5` fix list into a one-issue-at-a-time landing order with explicit promotion gates and manual smoke checks between landings.

## Objective

Finish the remaining analyzer-semantic hardening required before `R6` scorer work by landing one bounded issue at a time, validating between each landing, and refusing to advance to the next issue until the current issue is both test-green and smoke-proven against the known native and adapted repro sessions.

## Assumptions

1. The user’s requested new spec directory belongs under the existing `R5` authority stack, so this map lives at `docs/specs/r5/R5_75/` rather than a new top-level `docs/specs/R5_75/` tree.
2. Native `.codex/sessions/rollout-*.jsonl` sessions remain the primary behavior authority.
3. The adapted Hugging Face export corpus remains secondary robustness evidence only; it is useful for hardening but does not redefine native Codex rollout semantics.
4. `R5.5` landed meaningful improvements, but the validation handoff proved the family is not yet ready to declare “fully landed and R6-ready.”

## Required Fixes Adopted Into R5.75

This map includes only the fixes that were explicitly adopted as required for pre-`R6` closure:

1. `R5.75-0` — reconcile stale `R5.5` status/docs so the repo authority matches landed code plus remaining work
2. `R5.75-1` — objective condensation / target extraction for giant pasted user prompts
3. `R5.75-2` — sparse readable session fail-open instead of analyzer hard-abort
4. `R5.75-3` — delegated parent-visible stabilization
5. `R5.75-4` — zero-verifier anti-flap gating for long exploratory sessions
6. `R5.75-5` — adapted external robustness fixture family

Not adopted into this required set by default:

- parent-visible comparability fingerprint redesign beyond what live repros prove necessary
- adapted-trace mirrored-row compactor widening unless analyzer hardening still cannot hold without it

## Authority And Evidence Inputs

Primary native evidence bundle:

- `target/manual-r55-validation/`
- key session ids:
  - `019eb430-6f9a-7a03-9a63-cb451b654795`
  - `019eb47f-0118-7e90-8291-30a1fb93769e`
  - `019eb98e-3c16-7ba0-92f9-0085654b470c`
  - `019eb907-95c4-73e1-843e-e337d1e93cb9`
  - `019eb917-9531-74e0-897d-ad8d362138ec`
  - `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Secondary adapted-external evidence bundle:

- dataset root: `raw/RangaPrasath/`
- adapted home: `target/ranga-validation/codex-home/`
- run outputs: `target/ranga-validation/runs/`
- sample ids:
  - `05a56cc51632982b`
  - `f47b81f39f2495dd`
  - `9c1512861c25cef1`
  - `097d97e914ca220f`
  - `da59436e63915185`

## Shared Verification Ladder

Every implementation issue in this map must use the same broad verification ladder before promotion:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

If an issue directly changes replay/operator compatibility or committed checkpoint fixtures in ways that could affect downstream consumption, add the minimal sentinel spot-checks before promotion:

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

## Shared Manual Smoke Harness

Use the repo’s existing static Hybrid Drift smoke path for each named session.

Native-session smoke template:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="<native-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/<issue-id>/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

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

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Adapted-external smoke template:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="<adapted-session-id>"
export SMOKE_ROOT="target/r5_75-smoke/<issue-id>/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

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

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Manual smoke review rule:

- Do not treat green commands alone as promotion proof.
- Inspect `summary.md` plus the first checkpoints and confirm the issue-specific expectation below.
- `R5.75-0` is the docs-only exception: it may intentionally skip native control smoke when its
  packet closeout notes record the cross-doc audit plus baseline `agent-drift-analyzer` test run.
- Do not advance if the current issue appears fixed but any earlier landed issue regresses on its named smoke sessions.

## Sequential Landing Map

## R5.75-0: R5.5 Status Reconciliation And Authority Cleanup

### Problem

The current `R5.5` docs still read like most implementation packets are open follow-up work even though several packets landed and the remaining work has shifted to a narrower post-validation hardening family. That stale authority risks reopening already-landed work and makes later packet closure dishonest.

### Required Change

- add an explicit landed-status section to the `R5.5` authority docs
- mark already-landed `R5.5` work as landed/history rather than still-open implementation debt
- move the remaining pre-`R6` work into this new `R5.75` map/family
- keep root landing-order authority honest about `R5.75` being the next gate before `R6`

### Primary Files

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
- any matching root-order authority that names the next family

### Automated Gate

Manual review only for the doc changes themselves, plus:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-1

- Manual authority review across the edited docs must show:
  - landed `R5.5` work is not still presented as unchecked implementation work
  - remaining issues now point at `R5.75`, not back at stale `R5.5` wording
  - `R6` is not named as current/next until `R5.75` is complete
- Optional control smoke is intentionally skippable for this docs-only packet. If skipped, record
  the skip explicitly in the packet closeout notes together with the cross-doc audit and baseline
  `cargo test -p agent-drift-analyzer -- --nocapture` result.

### Promotion Gate

Do not begin `R5.75-1` until the docs are internally consistent and no stale `R5.5` checkbox language remains for already-landed work.

## R5.75-1: Objective Condensation And Target Extraction

### Problem

Objective selection is still too row-level. Giant pasted user prompts can still win as the checkpoint objective even when the concrete ask is embedded later in the same row or prompt body.

### Required Change

- condense long user prompts to the shortest concrete task ask when possible
- prefer `/goal`, short imperative asks, and clear workspace/action-target phrases over whole pasted bodies
- remove the current bias that can let longer same-priority candidates win just because they are longer
- preserve user-requested boilerplate targets when the actual task is to analyze or edit the boilerplate itself

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-2

Run native smoke on:

- `019eb430-6f9a-7a03-9a63-cb451b654795`
- `019eb47f-0118-7e90-8291-30a1fb93769e`
- `019eb98e-3c16-7ba0-92f9-0085654b470c`

Run adapted smoke on:

- `05a56cc51632982b`

Expected smoke outcome:

- first-checkpoint objective resolves to the concrete task, not the pasted skill/spec/profile body
- the adapted `05a56cc51632982b` session condenses to the workspace action request rather than the full pasted skill body
- explicit user requests to analyze or edit instruction/skill/AGENTS material remain preserved when that is the real target

### Promotion Gate

Do not begin `R5.75-2` until all four named objective repros show the true concrete task in smoke review and no preserved-boilerplate regression appears in targeted tests.

## R5.75-2: Sparse Readable Session Fail-Open

### Problem

The analyzer still hard-fails readable sessions when tool-call payloads or working-set inference are weak, even when literal objective rows remain available and a conservative checkpoint would be more honest than aborting.

### Required Change

- split structurally invalid bundle failures from semantically sparse-but-readable bundles
- keep hard-fail behavior for corrupt inputs only
- emit at least one conservative low-confidence checkpoint for sparse readable sessions
- use existing insufficient-evidence surfaces before widening public schema/contract

### Primary Files

- `crates/agent-drift-analyzer/src/input.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- any analyzer acceptance test that proves sparse readable fail-open behavior

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-3

Run adapted smoke on:

- `f47b81f39f2495dd`

Run native control smoke on:

- `019eb430-6f9a-7a03-9a63-cb451b654795`

Expected smoke outcome:

- `f47b81f39f2495dd` no longer aborts the analyzer pipeline
- analyzer emits at least one checkpoint for the sparse readable session
- resulting output stays conservative: low-confidence / insufficient-evidence rather than fabricated strong progress
- the native control session still produces normal output after the contract change

### Promotion Gate

Do not begin `R5.75-3` until the sparse adapted repro fail-opens cleanly and the native control session proves the relaxed contract did not break ordinary analyzer runs.

## R5.75-3: Delegated Parent-Visible Stabilization

### Problem

Delegated parent-visible progress remains unstable. Strong parent orchestration evidence can still collapse back into generic planning noise, especially when planning artifacts are edited or child visibility is partial/opaque.

### Required Change

- stop discarding parent-visible orchestration solely because planning/spec/handoff artifacts were edited
- preserve conservative parent-visible orchestration when delegation markers are strong but child visibility is limited
- only widen comparability/fingerprint logic if the named repros prove that reset behavior is still blocking stability after the earlier fix

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- any targeted acceptance fixture proving delegated-parent stability

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-4

Run native smoke on:

- `019eb907-95c4-73e1-843e-e337d1e93cb9`
- `019eb917-9531-74e0-897d-ad8d362138ec`
- `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Run adapted smoke on:

- `da59436e63915185`

Expected smoke outcome:

- delegated parent sessions remain in a stable parent-visible orchestration lane instead of dropping into generic planning-only noise
- limited child visibility remains visible as limiting evidence rather than being hidden
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` stays a positive proof that the parent-visible path still works
- if stability still fails only because comparability resets remain too broad, capture that as the bounded follow-up inside this issue before promoting

### Promotion Gate

Do not begin `R5.75-4` until the three native delegated repros and the one adapted delegated repro hold a conservative but stable parent-visible interpretation.

## R5.75-4: Zero-Verifier Anti-Flap Gate

### Problem

Long browse/read/tool-output-heavy sessions with zero verifier density can still escalate into troubleshooting or dead-end-ish output without decisive evidence.

### Required Change

- cap or suppress troubleshooting/implementation escalation when verifier attempts, concrete source-edit progress, and explicit failure evidence are absent
- prefer planning-convergence / insufficient-evidence for long exploratory sessions unless decisive signals appear
- keep this analyzer-local; do not widen into `R6` scorer retuning yet

### Primary Files

- `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

### Automated Gate

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Manual Smoke Check Before Promoting To R5.75-5

Run adapted smoke on:

- `097d97e914ca220f`
- `da59436e63915185`

Expected smoke outcome:

- zero-verifier exploratory sessions stay boring and conservative
- no troubleshooting-frontier or equivalent strong failure posture appears unless verifier/failure evidence truly exists
- low-confidence planning / insufficient-evidence remains the default outcome when the session mostly reads, browses, or emits tool output without proof work

### Promotion Gate

Do not begin `R5.75-5` until the named exploratory adapted sessions stop flapping into overclaiming progress/failure lanes.

## R5.75-5: Adapted External Robustness Fixture Family

### Problem

The adapted external corpus is currently useful evidence but not yet a committed, bounded, secondary robustness wall. Without a committed robustness family, later regressions can quietly reappear before `R6`.

### Required Change

- add a separate adapted-external acceptance family or fixture lane
- keep it explicitly secondary to the native rollout corpus
- cover at least the adopted external repro classes:
  - giant pasted prompt body
  - sparse readable / no parseable tool-call payloads
  - long exploratory zero-verifier session
  - delegated opaque-parent session

### Primary Files

- `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
- any separate adapted-external acceptance test/module chosen during implementation
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- any narrow design note needed to record the secondary-fixture contract

### Automated Gate

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Add touched sentinel spot-checks only if the committed fixtures alter downstream replay expectations.

### Manual Smoke Check Before Declaring R5.75 Complete

Rerun the full named smoke set:

Native:

- `019eb430-6f9a-7a03-9a63-cb451b654795`
- `019eb47f-0118-7e90-8291-30a1fb93769e`
- `019eb98e-3c16-7ba0-92f9-0085654b470c`
- `019eb907-95c4-73e1-843e-e337d1e93cb9`
- `019eb917-9531-74e0-897d-ad8d362138ec`
- `019eb970-3543-7ab1-a5d6-2a62c00c7185`

Adapted:

- `05a56cc51632982b`
- `f47b81f39f2495dd`
- `097d97e914ca220f`
- `da59436e63915185`

Expected smoke outcome:

- all earlier issue expectations still hold together
- adapted sessions are clearly recorded as secondary robustness proof, not native authority replacement
- no current adopted repro class is left unguarded by either native fixtures, adapted fixtures, or manual smoke evidence

### Promotion Gate

Do not declare `R5.75` complete until the full smoke set above is rerun after the fixture-family landing and all prior issue expectations still hold.

## R6 Readiness Gate

Do not open `R6` until all of the following are true:

- `R5.75-0` authority docs are honest about landed `R5.5` work and remaining `R5.75` scope
- giant prompt objective repros condense to the concrete task instead of pasted scaffold bodies
- sparse readable sessions fail open conservatively instead of hard-aborting the analyzer
- delegated parent-visible sessions stay stable and conservative under limited child visibility
- zero-verifier exploratory sessions no longer flap into troubleshooting/dead-end overclaim
- the adapted external robustness family is committed as a secondary acceptance wall
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- any touched sentinel spot-checks are green
- root landing-order authority names `R6` as next only after `R5.75`

## Out Of Scope For This Map

These can be considered only if the required issues above prove insufficient:

- scorer retuning or `dead_end_thrash` scoring redesign (`R6` scope)
- broad compactor normalization changes for native traces
- opportunistic refactors not required to land the named issue
- generalized parent-visible fingerprint redesign unless the named delegated repros prove it is still necessary after the narrower stabilization work
