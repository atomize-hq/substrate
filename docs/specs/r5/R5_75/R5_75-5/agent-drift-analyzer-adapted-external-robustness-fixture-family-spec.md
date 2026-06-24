# Spec: Agent Drift Analyzer Adapted External Robustness Fixture Family (R5.75-5)

Status: draft spec created on 2026-06-23 after `R5.75-4` was promoted in
`docs/specs/r5/R5_75/MAP.md`, using the live `R5.75` map, the current
`progress_acceptance` / `objective_acceptance` harnesses, the existing
`target/ranga-validation/` adapted sample corpus, and the current fixture README contracts.

## Assumptions I'm Making

1. `R5.75-4` is already promoted history, and `R5.75-5` is now the active packet exactly as routed in
   `docs/specs/r5/R5_75/MAP.md`.
2. This packet is primarily a fixture / harness / docs packet. It should not retune analyzer semantics
   that were already owned by `R5.75-1` through `R5.75-4`. If seeding a committed adapted fixture
   exposes a new semantic bug, stop and route that as a follow-on instead of quietly widening here.
3. Native Codex rollouts remain the primary authority. Adapted external cases are a bounded, explicitly
   secondary robustness wall only; they must never replace or dilute the native acceptance anchor.
4. Adapted *objective-shaped* evidence belongs only in the pre-existing
   `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/` family, and
   only when it adds signal beyond the already-landed locked objective corpus.
5. Adapted *progress-shaped* evidence belongs in the existing `progress_acceptance` harness as a
   separate secondary lane with explicit metadata and README contract, not as silent native-case
   inflation.
6. The user asked for the SPEC/PLAN/TASKS triplet only, so packet prompts are intentionally out of
   scope for this docs pass.

If any assumption above is wrong, correct it before implementation starts.

## Objective

Commit a bounded secondary robustness wall for the adapted external corpus so the adopted external
repro classes that already informed `R5.75-1` through `R5.75-4` become durable, reviewable, and
deterministically testable before `R6`.

Primary users:

1. engineers who need future regressions against adapted external shapes to fail in CI instead of
   reappearing only in manual smoke;
2. reviewers who need the acceptance wall to distinguish native primary authority from adapted
   secondary robustness;
3. future `R6` scorer work, which should inherit a bounded external robustness contract instead of
   relying on ad hoc reruns under `target/ranga-validation/`.

Success means:

1. the adopted adapted progress classes are committed in a secondary `progress_acceptance` lane with
   explicit expectations and provenance;
2. the objective harness either remains honestly placeholder-only for adapted external cases or adds
   exactly one bounded `stretch-external` case only when it contributes real new signal;
3. the fixture/docs contract makes primary-vs-secondary authority explicit everywhere it matters; and
4. the full named native + adapted smoke set still holds together after the committed fixture family
   lands.

## Scope Classification

- **In scope**
  - add a bounded adapted-external lane to `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    and its fixture contract
  - commit adapted progress fixtures for the adopted external repro classes:
    - sparse readable / no parseable tool-call payloads (`f47b81f39f2495dd`)
    - long exploratory zero-verifier session (`097d97e914ca220f`)
    - delegated opaque-parent / parent-visible session (`da59436e63915185`)
  - audit whether adapted objective-shaped case `05a56cc51632982b` adds net-new signal beyond the
    already-landed locked objective corpus, and either:
    - keep `stretch-external/` placeholder-only with an explicit no-op rationale, or
    - add one bounded `stretch-external` case if it truly contributes signal
  - restore or create the missing current fixture authority doc at
    `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
  - rerun the full named native + adapted smoke set before promotion
- **Out of scope**
  - new analyzer semantic retuning for objective extraction or session progress
  - replacing native acceptance cases with adapted ones
  - widening the adapted family beyond the adopted repro classes in `MAP.md`
  - packet prompts in this docs pass
  - schema/version/replay/export changes unless fixture loading cannot proceed without a narrowly
    justified harness-only adjustment

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Primary acceptance harness seams:
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- Primary fixture roots:
  - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
  - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/**`
- Primary fixture contract docs:
  - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
  - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`
  - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md`
  - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- Source adapted evidence roots:
  - `target/ranga-validation/runs/<session-id>/compactor/`
  - `target/r5_75-smoke/R5.75-2/<session-id>/`
  - `target/r5_75-smoke/R5.75-4/<session-id>/`

No new runtime dependency, no analyzer-scoring redesign, and no native-authority downgrade belong in
this packet.

## Commands

Focused harness and acceptance gates:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Full analyzer wall:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Useful fixture-contract inspection:

```bash
rg -n "PROGRESS_ACCEPTANCE_CASE_IDS|FixtureKind|stretch-external|annotated_real_rollout|synthetic_bundle_shaped" \
  crates/agent-drift-analyzer/tests/progress_acceptance.rs \
  crates/agent-drift-analyzer/tests/objective_acceptance.rs \
  crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md \
  crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md
```

Adapted source-artifact inspection before committing any fixture:

```bash
for SESSION_ID in 05a56cc51632982b f47b81f39f2495dd 097d97e914ca220f da59436e63915185; do
  echo "===== $SESSION_ID ====="
  find "target/ranga-validation/runs/$SESSION_ID" -maxdepth 2 -type f | sort
done
```

Full native smoke rerun for promotion:

```bash
export CODEX_HOME="$HOME/.codex"
for SESSION_ID in \
  019eb430-6f9a-7a03-9a63-cb451b654795 \
  019eb47f-0118-7e90-8291-30a1fb93769e \
  019eb98e-3c16-7ba0-92f9-0085654b470c \
  019eb907-95c4-73e1-843e-e337d1e93cb9 \
  019eb917-9531-74e0-897d-ad8d362138ec \
  019eb970-3543-7ab1-a5d6-2a62c00c7185
do
  export SMOKE_ROOT="target/r5_75-smoke/R5.75-5/$SESSION_ID"
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
done
```

Full adapted smoke rerun for promotion:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
for SESSION_ID in \
  05a56cc51632982b \
  f47b81f39f2495dd \
  097d97e914ca220f \
  da59436e63915185
do
  export SMOKE_ROOT="target/r5_75-smoke/R5.75-5/$SESSION_ID"
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
done
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Landing-order authority. The R5.75-5 packet here defines the adopted repro classes and promotion gate.

docs/specs/r5/R5_75/structured-objective-bug-map.md
  Root diagnosis context. Useful for provenance, but not the packet's live implementation authority.

docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md
docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md
docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md
  This packet's implementation authority.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Dedicated semantic wall for session_progress. This packet adds the bounded adapted secondary lane.

crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**
  Committed progress fixtures plus README contract. Adapted cases must stay explicitly secondary here.

crates/agent-drift-analyzer/tests/objective_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/**
  Existing objective acceptance harness and placeholder robustness lane for adapted objective-shaped cases.

docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
  Missing current fixture-manifest authority that this packet should restore/update so native vs adapted
  fixture ownership is reviewable in one place.

target/ranga-validation/runs/<session-id>/compactor/
  Source bundle artifacts to snapshot into committed adapted fixtures when the packet proves the case
  belongs in the bounded corpus.

target/r5_75-smoke/R5.75-5/<session-id>/
  Promotion smoke reruns after the fixture family lands.
```

## Code Style

Keep the primary-vs-secondary boundary explicit in code and fixture metadata. Adapted robustness should
be readable as a distinct lane, not inferred from comments.

```rust
#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FixtureKind {
    AnnotatedRealRollout,
    AnnotatedAdaptedExternal,
    SyntheticBundleShaped,
}

const ADAPTED_EXTERNAL_PROGRESS_CASE_IDS: [&str; 3] = [
    "adapted-sparse-readable-f47b81f39f2495dd",
    "adapted-zero-verifier-097d97e914ca220f",
    "adapted-parent-visible-da59436e63915185",
];
```

Conventions to preserve:

- keep native and adapted case lists deterministic and separately asserted
- preserve at least one native `annotated_real_rollout` as the primary semantic anchor
- name adapted cases by behavior plus source session id so provenance stays obvious
- prefer explicit README / manifest / expected metadata over ad hoc chat-only rationale

## Testing Strategy

- **Progress acceptance harness**
  - preserve the bounded native corpus and its existing expectations
  - add a separate adapted-external lane with explicit metadata and expected secondary status
  - verify each adapted progress case carries clear `source_artifact`, expectation notes, signal
    requirements, and why the case remains secondary
- **Objective acceptance harness**
  - audit `05a56cc51632982b` against the already-landed locked cases
  - only add a `stretch-external` case if it proves something the locked native corpus does not
  - otherwise keep `stretch-external/` placeholder-only and record the no-op rationale in docs/tasks
- **Full analyzer wall**
  - `cargo test -p agent-drift-analyzer -- --nocapture`
- **Promotion smoke gate**
  - rerun the full named native set and full named adapted set from `MAP.md`
  - inspect `summary.md` and `checkpoints.jsonl`, not just command exit status
- **Conditional sentinel spot-checks**
  - only if the final packet changes downstream checkpoint contract assumptions rather than fixture/docs
    shape alone

## Boundaries

- **Always**
  - keep native rollout cases as the primary authority
  - keep adapted cases explicitly secondary in harness metadata, README wording, and tasks closeout notes
  - route adapted cases by shape into the correct existing home instead of duplicating coverage blindly
  - rerun the full named smoke set before promotion
- **Ask first**
  - changing analyzer semantics to force an adapted case green when the committed native wall already says
    behavior is correct
  - widening the adapted corpus beyond the adopted repro classes in `MAP.md`
  - adding a second adapted objective case after `05a56cc51632982b`
  - changing replay/export/schema contracts
- **Never**
  - present adapted external fixtures as equal authority with native real rollouts
  - duplicate the giant-pasted-prompt objective class if `R5.75-1` locked cases already cover it
  - rewrite existing native expectations just to align them with adapted outputs
  - mark the packet promoted from green tests alone without rerunning the full named smoke set

## Success Criteria

1. `progress_acceptance` has an explicit adapted-external secondary lane with committed fixtures for:
   - `f47b81f39f2495dd`
   - `097d97e914ca220f`
   - `da59436e63915185`
2. Each committed adapted progress case records deterministic expectations that match the already-landed
   packet boundaries:
   - sparse readable stays conservative (`R5.75-2`)
   - zero-verifier exploration stays conservative (`R5.75-4`)
   - delegated parent-visible evidence stays guardrail-only and limited (`R5.75-3`)
3. The objective harness either:
   - remains placeholder-only for `stretch-external/` with an explicit no-op rationale because
     `05a56cc51632982b` adds no net signal, or
   - commits exactly one bounded `stretch-external` case with explicit rationale for why it adds signal
4. `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` exists again and documents the
   native primary corpus, synthetic support cases, and adapted secondary robustness lane honestly.
5. `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`,
   `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`, and
   `cargo test -p agent-drift-analyzer -- --nocapture` are green before promotion.
6. The full named native + adapted smoke set reruns green, and no previously adopted repro class is left
   guarded only by memory or ad hoc `target/` artifacts.

## Open Questions

1. Does `05a56cc51632982b` add net-new objective-shape signal after `R5.75-1`'s locked cases, or should
   this packet keep `stretch-external/` placeholder-only and only document the cross-reference?
2. Is a new `FixtureKind::AnnotatedAdaptedExternal` enough to make the secondary lane explicit, or does
   the progress harness also need a separate root-level README/constant split to keep reviews honest?
3. Do any of the adapted source bundles remain too unstable to commit as deterministic fixtures even
   after `R5.75-2` through `R5.75-4` landed, and if so should the packet stop rather than forcing them
   into the corpus?
