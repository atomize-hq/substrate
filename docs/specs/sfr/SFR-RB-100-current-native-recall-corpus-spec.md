# SFR-RB-100 SPEC — Current-Native Recall Corpus

Status: **APPROVED 2026-07-29 — P7 IMPLEMENTATION AUTHORIZED**

Implementation authority: **P7 ONLY**. The user approved autonomous execution of the exact
review-reconciled specification, plan, and task ledger. The test/fixture/script/doc-only default
fence and all production-defect stop rules remain binding.

Baseline: post-P6 commit `133b55249f88492e16f80f98a63368911d733c7e` on
`feat/sfr-p4-path-semantics`.

Advisory input:
`docs/guidance/2026-07-29-sfr-p7-chatgpt-pro-consultation.md`.
That consultation is non-authoritative; this specification reconciles it with live repository
contracts.

## Assumptions To Confirm

1. P7 is a validation and recall-proof packet, not a production-behavior packet.
2. The committed corpus is the reproducible release gate. Private real-session evidence is a local
   signoff lane and never an ordinary CI dependency.
3. Existing R6, R7, and R8 acceptance walls and receipts remain frozen historical proof.
4. The target committed corpus is approximately 18–20 cases, with a 15–25 envelope and 22-case soft
   cap. Contract-matrix completeness takes precedence over the count.
5. The private sampling lane is required and non-vacuous: its exact quota configuration and required
   named strata are frozen before selection, its selected set is non-empty, sufficiently populated
   buckets meet quota, and inventory-scarcity underfill follows the explicit limitation rule below.
6. If a structurally valid required case exposes a production defect, P7 stops. The defect is
   repaired in a separately authorized, narrowly bounded remediation before P7 is regenerated from
   a new clean baseline.
7. P8 remains a separate additive external review and receipt after P7 is implemented, validated,
   independently reviewed, and committed.

## Terminology Contract

The two version names describe different layers and must never be used interchangeably:

| Term | Exact meaning | Repository evidence |
|---|---|---|
| **Repository `CurrentNativeV2` rollout class** | A raw Codex JSONL stream routed to the repository’s explicit current-native adapter because `session_meta.payload.multi_agent_version == "v2"`. `v2` is the value of the multi-agent marker and the repository adapter name; this specification does **not** claim that the complete Codex rollout schema is globally versioned “v2.” | `current_native_marker` in `crates/agent-session-compactor/src/ingest/current_native.rs`; `RolloutFormat::CurrentNativeV2` in `crates/agent-session-compactor/src/ingest/codex_rollout.rs` |
| **Compactor bundle schema v0.2** | Analyzer-facing normalized/exported bundle contract. It is downstream of either supported raw rollout format. | `SCHEMA_VERSION: "v0.2"` in `crates/agent-session-compactor/src/export/files.rs`; exact validation in `crates/agent-drift-analyzer/src/input.rs` |
| **Legacy rollout** | Supported pre-current-native raw rollout shape retained as a bounded compatibility floor. | `RolloutFormat::Legacy` in the compactor ingest adapter |
| **Canonical semantic projection** | Stable, path/time-independent semantic fields used for parity and determinism comparisons. | Defined by the P7 matrix and expected-result files; it excludes temporary paths, generated timestamps, and other intentionally variable bytes. |

Naming rules:

- Use `CurrentNativeV2` only for raw Codex rollout inputs that the repository actually routes to
  `RolloutFormat::CurrentNativeV2`, and for cases that retain that raw shape to the production
  parser.
- Use `bundle-v0.2` for the compactor-to-analyzer contract.
- Do not call an analyzer-only fixture a “v2 fixture.”
- A cross-format parity case may be described as
  `Legacy/CurrentNativeV2 -> bundle-v0.2`, never `v1/v2 bundle parity`.

### Verification record

Verified on 2026-07-29:

- `current_native_marker` returns the current-native route only for the exact string
  `multi_agent_version: "v2"`; missing, `disabled`, and `"v1"` markers take the legacy route.
- `select_rollout_format` maps that route to the repository enum
  `RolloutFormat::CurrentNativeV2`.
- Sanitized metadata inspection of the 25 most recent local rollout files found 14 with the `"v2"`
  marker and 11 with no `multi_agent_version` marker. Therefore, it would be incorrect to call all
  current Codex rollouts “rollout schema v2.”
- Compactor export writes exact manifest schema `"v0.2"`, and analyzer input rejects any other
  bundle schema version.
- Focused executable verification passed:
  `cargo test -p agent-session-compactor --test current_native_adapter -- --nocapture`
  (`7 passed`) and
  `cargo test -p agent-session-compactor --test export_bundle -- --nocapture`
  (`5 passed`).

## Objective

Build a sanitized, deterministic, structurally faithful P7 recall corpus that proves the restored
P1–P6 current-native contracts compose correctly across:

```text
raw Codex JSONL routed as repository `CurrentNativeV2`
    -> compactor format selection and typed ingest
    -> direct bounded linked-live closure
    -> compactor bundle schema v0.2 export
    -> analyzer input and semantic-goal-drift scoring
    -> Sentinel public live-event validation and state behavior
```

The primary users are Substrate maintainers and release reviewers. They need reproducible evidence
that:

- at least one real current-native structural witness still produces a true
  `semantic_goal_drift` result;
- conservative controls stay quiet or fail closed for the correct reason;
- current-native typed identities and linkage survive end-to-end composition;
- malformed and incomplete inputs fail at the owning boundary with exact outcomes;
- historical behavior remains green without duplicating or rewriting historical corpora; and
- no private rollout content enters Git.

Success is a reviewable, CI-runnable proof wall plus a separately reproducible private sampling
receipt—not a larger framework, scorer retuning, or a migration away from the legacy compatibility
floor.

## User Stories

1. **As a maintainer**, I can run one committed P7 wall and prove that raw Codex input routed
   through the repository’s `CurrentNativeV2` adapter reaches the shipping semantic analyzer and
   public-live boundary.
2. **As a reviewer**, I can trace every P7 case from its raw event variants through selected closure,
   expected bundle projection, analyzer result or error, and public-live result.
3. **As a privacy reviewer**, I can run a recursive machine check proving the committed corpus uses
   only synthetic identifiers, repositories, messages, and paths.
4. **As a release owner**, I can reproduce the private sample selection from a frozen candidate
   inventory and distinguish honest underfill from coverage.
5. **As a future maintainer**, I can add current-native coverage without cloning every historical
   R6/R7/R8 group or confusing the `multi_agent_version: "v2"` adapter class with bundle schema
   v0.2.

## Scope

### In scope

- A P7-owned contract-family × pipeline-boundary matrix.
- Approximately 18–20 sanitized committed cases, subject to the approved envelope.
- At least one complete raw repository-`CurrentNativeV2` positive witness.
- Conservative semantic, verification, path, session-isolation, delegation, malformed-input,
  incomplete-bundle, and public-live controls.
- Two or three explicit `Legacy`/`CurrentNativeV2` semantic-parity pairs.
- Exact per-case expected compactor projection, analyzer result/error, Sentinel result, and
  mutation behavior.
- Recursive privacy and provenance enforcement.
- Determinism checks across varied temporary roots, discovery order, cache state, and fixed
  timestamps/identifiers.
- Deterministic private real-session inventory and overlapping-quota selection.
- Existing focused and full-suite regression gates.
- Documentation of coverage, underfill, provenance, sanitization, and final signoff.

### Out of scope

- Scorer threshold, eligibility, weighting, or suppression changes.
- Production parser, closure, export, analyzer, Sentinel, scheduler, or presentation behavior.
- Fixture-specific production branches.
- Transitive delegation-graph discovery; the authorized closure remains direct and bounded.
- Renaming or versioning the v0.2 bundle as v2.
- Rewriting, relocating, or expanding frozen R6/R7/R8 receipts and fixture walls.
- Removing the legacy rollout compatibility floor.
- Uploading or committing raw private sessions or derived private text.
- Making a developer’s private `~/.codex/sessions` store a CI prerequisite.
- Broad property testing, fuzzing, continuous private-corpus automation, performance benchmarking,
  or unbounded new event-variant discovery.
- P8 external review work.

## Required Coverage Matrix

The committed corpus must be selected against this matrix. Historical cases are referenced as
inherited proof; they are not cloned merely to make each old grouping look current-native.

| Contract family | Required boundary proof |
|---|---|
| Native identity and typed shape | `session_meta`, session/thread identity, `turn_context`, typed call input, typed output type and order |
| Semantic positive | Raw input routed as `CurrentNativeV2` reaches an exact true `semantic_goal_drift` result |
| Semantic conservative controls | Alignment, narrowing/progression, and sanctioned replan or equivalent conservative outcome |
| Verification execution | Zero-test execution does not become clean verification evidence |
| Path authority | Directive syntax cannot launder path authority |
| Lexical path identity | Prefix, hyphen, stem, or sibling collisions cannot hide a real pivot |
| Session isolation | Unrelated-session evidence cannot alter the selected result |
| Delegation | Raw typed linkage survives production bundle-v0.2 export and analyzer-owned topology/visibility/ownership semantics; registered metadata-only child acceptance and missing-child rejection remain distinct |
| Closure malformed handling | Malformed unrelated source is excluded; malformed selected source is rejected |
| Analyzer malformed handling | Intentionally incomplete bundle is rejected with the exact `InputError` |
| Public-live boundary | Invalid public event is rejected without forbidden runtime-state mutation |
| Compatibility | Two or three declared `Legacy`/`CurrentNativeV2` pairs match on canonical semantic projections |
| Legacy floor | Existing R6, R7, and R8 walls run unchanged and remain green |

Every matrix entry must contain:

- `case_id`;
- raw rollout format;
- exact event and typed-value variants exercised;
- logical source files and selected direct closure;
- semantic family;
- `parity_mode`: `equivalent`, `current_native_only`, or `not_applicable`;
- historical case references, when applicable;
- expected bundle-v0.2 canonical projection;
- exact expected analyzer score, state, progress, signal, `NoClaim`, or `InputError`;
- exact expected public-live acceptance or rejection and permitted state effects;
- terminal pipeline boundary;
- diagnostic owner for each asserted result or first failure;
- privacy and provenance declarations; and
- local/private stratum labels, when the case is derived from a private structural witness.

## Committed Corpus Design

### Size and diagnostic ownership

- Target: 18–20 cases.
- Acceptable envelope: 15–25.
- Soft cap: 22. Crossing it requires an explicit written justification that the extra case proves a
  distinct required contract rather than duplicating a focused P1–P6 test.
- A case may cover multiple cells only when its terminal boundary and each assertion's diagnostic
  owner remain unambiguous.
- Malformed conditions should remain independently diagnosable rather than being combined into one
  overloaded fixture.

An initial budgeting hypothesis—not a quota—is:

- 7 semantic/progress/path cases;
- 5 closure/delegation cases;
- 3 analyzer/public-boundary cases;
- 2 explicit parity pairs; and
- 1 typed-output/determinism stress case.

### Structural fidelity

At least one positive case must preserve actual envelopes and typed variants from a stream selected by
the repository as `CurrentNativeV2`, from raw JSONL through the first production parser.
Sanitization may replace content and identifiers, but must not flatten the witness into compacted
rows or an analyzer-only bundle.

Every committed case must use synthetic:

- session, turn, call, event, and agent identifiers;
- repository and workspace names;
- user and assistant messages;
- filesystem paths;
- command arguments not required to prove the contract; and
- timestamps or other environmental values, unless a fixed value is required by the case.

### Compatibility pairs

Parity is intentionally narrow:

- one true semantic-goal-drift positive pair;
- one aligned or narrowing negative pair; and
- optionally one zero-test pair if it proves a distinct cross-format contract.

Parity compares canonical semantic projections. Raw JSONL, manifest bytes, absolute paths,
`generated_at`, temporary directories, and other deliberately unstable values are excluded.

## Private Real-Session Lane

The private lane validates recall and input distribution against real current-native sessions without
adding them to Git.

### Inventory

Before selection, produce:

- a fixed local as-of cutoff;
- a stable, sorted inventory of candidates routed by the repository as `CurrentNativeV2`;
- an inventory digest;
- an exact sampler/quota configuration and seed;
- the required named strata and the minimum candidate population that makes each quota mandatory;
- the selected-set digest; and
- a machine-readable underfill report.

A seed is only a tie-breaker. It does not make selection reproducible when the candidate inventory
has changed.

### Selection

Use deterministic greedy set cover over overlapping labels rather than selecting 3–5 independent
sessions for every cell.

- Ordinary sufficiently populated strata target 3 distinct sessions.
- High-risk or sparse strata may target up to 5.
- One session may credit every stratum it genuinely satisfies.
- `unknown` is reported and never substituted into a named stratum.
- The selected set must contain at least one session.
- Every configured bucket whose eligible candidate population meets or exceeds its quota must fill
  that quota; underfill in such a bucket is a sampler or harness failure.
- Inventory-scarcity underfill is permitted only when eligible population is below quota, all
  eligible candidates are selected or a documented privacy/safety exclusion explains otherwise,
  and the receipt records the exact limitation.
- Every configured stratum family with at least one recognized candidate must have a non-`unknown`
  selected representative unless the same inventory-scarcity rule is satisfied.
- Underfilled buckets remain visibly underfilled; no unrelated case may be forced into them.
- Heuristic batch strata must not be presented as proof of scorer-internal containment,
  stable-target, or sanctioned-replan behavior. Those contracts belong to the committed wall.

Raw sessions, raw messages, local paths, repository identities, and private derived outputs remain
outside Git.

## Project Structure

### Expected P7-owned area

```text
crates/agent-drift-sentinel/tests/
  current_native_recall.rs
  fixtures/current_native_recall/
    README.md
    matrix.json
    <case-id>/
      *.jsonl
      expected.json

docs/specs/sfr/
  SFR-RB-100-current-native-recall-corpus-spec.md
  # Plan and task documents are created only after spec approval.

scripts/dev/drift-batch-scan/
  sample_sessions.py
  run_batch.py
  tabulate.py
  test_tabulate.py
  README.md
```

The exact fixture filename layout may be refined during planning, but every case must remain
self-contained, matrix-addressable, recursively privacy-scannable, and readable without a private
store.

### Existing areas that may be extended only when necessary

- `crates/agent-session-compactor/tests/current_native_adapter.rs`
- `crates/agent-session-compactor/tests/bounded_closure.rs`
- `crates/agent-session-compactor/tests/export_bundle.rs`
- `crates/agent-drift-analyzer/tests/input_contract.rs`
- `crates/agent-drift-sentinel/tests/live_event_shape.rs`
- `crates/agent-drift-sentinel/tests/real_session_live.rs`

These are conditional seams, not a default file list. The P7-owned harness should express the
cross-layer contract without scattering duplicate assertions whenever possible.

### Frozen areas

- `crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/**`
- `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
- `crates/agent-drift-analyzer/tests/fixtures/delegated_acceptance/**`
- historical R6/R7/R8 specifications, maps, findings, and review receipts
- production parser, closure, export, analyzer, Sentinel, scheduling, and presentation modules

## Tech Stack

- Rust workspace pinned by `rust-toolchain.toml`; repository MSRV is Rust 1.89+.
- Existing crates:
  - `agent-session-compactor`
  - `agent-drift-analyzer`
  - `agent-drift-sentinel`
- Rust integration tests and existing workspace test helpers.
- JSON/JSONL fixtures with `serde`/`serde_json`.
- Python 3 standard-library scripts for the private batch harness.
- GitNexus for impact analysis before any indexed symbol edit and change detection before commit.
- No new runtime or test dependency without user approval.

## Commands

All commands run from the repository root.

### Baseline and static preflight

```bash
git status --short --branch
git rev-parse HEAD
git diff --check
```

### Focused current-native and closure gates

```bash
cargo test -p agent-session-compactor --test current_native_adapter -- --nocapture
cargo test -p agent-session-compactor --test bounded_closure -- --nocapture
cargo test -p agent-session-compactor --test export_bundle -- --nocapture
```

### Focused analyzer regression gates

```bash
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test delegated_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
```

### Focused Sentinel boundary gates

```bash
cargo test -p agent-drift-sentinel --test real_session_live -- --nocapture
cargo test -p agent-drift-sentinel --test live_event_shape -- --nocapture
cargo test -p agent-drift-sentinel --test replay_input -- --nocapture
```

The plan must add the exact P7-owned test target after the harness name is confirmed.

### Private batch-tool tests

```bash
python3 scripts/dev/drift-batch-scan/test_tabulate.py
```

Any new sampler tests must be runnable without access to `~/.codex/sessions` by using synthetic
inventory inputs.

### Full release wall

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
git diff --check
```

Before commit:

```text
Run GitNexus change detection and verify that only approved P7 symbols, tests, fixtures,
scripts, and documents are affected.
```

## Code Style

Follow existing Rust 2021 and repository test conventions: four-space indentation, snake_case test
names describing behavior, `Result` for fallible shared helpers, `anyhow::Context` for diagnostic
boundaries, deterministic ordered collections, and no panics in production code.

Illustrative test style:

```rust
#[test]
fn current_native_recall_cases_match_the_declared_matrix() {
    let matrix = load_matrix();

    for case in matrix.cases {
        let actual = run_current_native_case(&case);
        assert_eq!(
            canonical_projection(&actual),
            case.expected,
            "P7 case {} failed at {}",
            case.case_id,
            case.owning_stage,
        );
    }
}
```

Test diagnostics must name the `case_id`, terminal boundary, and diagnostic owner:

- native parser;
- closure selection;
- export;
- analyzer input;
- scoring; or
- public-live conversion/runtime.

Python uses the existing standard-library style, deterministic sorted iteration, explicit command-line
arguments, and `unittest`.

## Testing Strategy

### 1. Matrix completeness

A test must prove:

- every fixture directory has exactly one matrix entry;
- every matrix entry resolves to one fixture directory;
- all required contract families are represented;
- every case declares raw format, parity mode, terminal boundary, diagnostic owners, exact
  expectations, and provenance; and
- the committed case count is within the approved envelope.

### 2. End-to-end positive recall

At least one raw fixture routed as repository `CurrentNativeV2` must traverse the shipping
composition path and produce the exact declared semantic-goal-drift result. Adapter-local or
analyzer-only success is insufficient.

### 3. Conservative outcomes

Alignment, narrowing/progression, zero-test, directive/path, lexical collision, unrelated-session,
metadata-only child, and missing-child controls must assert exact results. Absence of a panic or
absence of a signal alone is not sufficient.

### 4. Malformed and incomplete inputs

- Malformed unrelated closure evidence is not selected and cannot poison the result.
- Malformed selected closure evidence fails at closure ownership.
- An intentionally incomplete bundle fails with the selected exact analyzer `InputError`.
- Invalid public-live shape is rejected without forbidden runtime-state mutation.

### 5. Determinism

All nineteen committed cases run in at least two canonicalized wall executions with:

- different temporary roots;
- reversed source creation or stable discovery order for every applicable case;
- warm and cold closure-cache state, where applicable; and
- fixed synthetic timestamps and identifiers.

Canonical semantic projections must match exactly. The test must also prove matrix completeness and
stable case ordering. P7-19 remains the stronger typed-segment order/identity stress witness; it is
not the sole varied-run determinism proof.

### 6. Privacy and provenance

A recursive machine test scans every committed P7 file and proves:

- IDs conform to a documented synthetic grammar;
- no known private absolute path, username, repository name, or raw session identifier appears;
- no credential-like content or secret material appears;
- every fixture’s declared source format and event variants match its actual data; and
- sanitized structural provenance is sufficient to explain which current-native contracts the case
  retains.

A prose-only sanitization claim does not satisfy this gate.

### 7. Historical regression walls

The existing semantic 18-case wall, progress 16-case wall, delegated 10-case matrix, and relevant
R6/R7/R8 focused/full suites remain unchanged and green.

### 8. Private sampling receipt

The local run records inventory cutoff/digest, exact sampler/quota configuration and seed, required
named strata, per-bucket eligible populations, selected-set digest, quota coverage, and underfill.
The selected set must be non-empty. Sufficiently populated buckets must meet quota; scarcity
limitations must satisfy the selection rule above. Every concrete contract-relevant contradiction
surfaced by the private run must be classified through the packet's three-way failure rule before
the receipt can be accepted. The committed validation receipt is included in the recursive privacy
scan, must contain no raw private content, and must not be needed to execute committed CI tests.

## Failure Classification And Stop Rules

Every committed-wall failure and every concrete contract-relevant contradiction surfaced by the
private run is classified before any code is changed:

1. **Fixture or harness error** — malformed synthetic JSON, wrong expectation, stale matrix,
   nondeterministic test helper, or privacy violation while production behavior still conforms.
   This may be corrected within P7.
2. **Production contract defect** — a structurally valid required witness violates a P1–P6
   contract, unrelated malformed evidence poisons selected closure, a conservative control fires,
   a registered metadata-only child is rejected, a missing child is accepted, public-live accepts an
   invalid event, or canonical production output is nondeterministic. Stop P7 and open a separately
   authorized remediation.
3. **New behavior request** — the corpus exposes a desirable semantic expansion not required by the
   approved P1–P7 contract. Record and defer it.

Private ordinary outcomes do not require a broad semantic audit. Triage is mandatory only when the
batch presents concrete evidence of a stated P7 or inherited P1–P6 contract violation. A receipt
cannot summarize such a contradiction without assigning one of the three classifications above.

Immediate remediation triggers include:

- needing a scorer threshold or eligibility change to make a required case pass;
- needing fixture-specific production logic;
- needing to expand direct closure into transitive discovery;
- needing to reinterpret malformed selected evidence as clean;
- needing to rename or alter bundle schema v0.2; or
- discovering that the intended full raw `CurrentNativeV2` path cannot be exercised without a
  production API change.

## Boundaries

### Always do

- Preserve the `multi_agent_version: "v2"` / repository-`CurrentNativeV2` versus bundle-v0.2
  terminology distinction.
- Start from the exact clean committed baseline.
- Freeze and validate the coverage matrix before generating fixtures.
- Run GitNexus impact analysis before editing any indexed Rust symbol.
- Keep committed data sanitized, deterministic, bounded, and independently runnable.
- Record exact outcomes, terminal boundaries, and diagnostic owners.
- Keep historical acceptance walls unchanged and green.
- Run focused gates before full suites.
- Stop and classify a production contradiction rather than widening P7.

### Ask first

- Any production Rust change.
- Any new dependency.
- Any change to CI configuration.
- Any committed case count outside 15–25 or above the 22-case soft cap.
- Any change to scorer thresholds, eligibility, suppression, or analyzer schema.
- Any change to the direct bounded-closure contract.
- Any modification to frozen R6/R7/R8 fixtures or receipts.
- Any proposal to make private-session evidence mandatory in CI.

### Never do

- Commit or upload raw private sessions, messages, identifiers, repository names, or local paths.
- Treat a compact-row/analyzer fixture as proof of raw repository-`CurrentNativeV2` ingest.
- Call the bundle schema v2; it remains v0.2.
- Substitute legacy-shaped synthetic input for the required current-native positive witness.
- Duplicate every historical corpus group merely to create a v2 version.
- Tune production behavior to a fixture.
- Force unrelated sessions into underfilled private strata.
- Claim heuristic batch labels prove scorer-internal semantics.
- Expand P7 into P8, long-term corpus maintenance, fuzzing, or performance work.

## Success Criteria

P7 is eligible for independent review and closeout only when all of the following are true:

1. The approved matrix covers every required P7 contract family and pipeline boundary.
2. At least one raw Codex positive routed as repository `CurrentNativeV2` traverses the complete
   production path and produces the exact semantic-goal-drift result.
3. Current-native session/turn identity, typed input, typed output type/order, and raw typed
   delegation linkage survive production bundle-v0.2 export and analyzer interpretation with exact
   parent/child topology, visibility, linkage evidence, and applicable ownership semantics.
4. Alignment, narrowing/progression, zero-test, directive/path, lexical-collision, and
   unrelated-session controls produce exact conservative outcomes.
5. Registered metadata-only child acceptance and missing-child rejection are separately pinned.
6. Malformed unrelated and malformed selected closure behavior are exact and stage-owned.
7. The intentionally incomplete bundle produces the selected exact analyzer `InputError`.
8. Invalid public-live input is rejected without forbidden runtime-state mutation.
9. Declared `Legacy`/`CurrentNativeV2` pairs match on canonical semantic projections.
10. Current-native-only typed variants are preserved.
11. Matrix completeness, whole-wall two-run determinism across all nineteen cases, provenance, and
    recursive privacy gates pass.
12. Existing R6/R7/R8 walls remain unchanged and green.
13. The private batch uses a frozen quota configuration, selects a non-empty set, fills every
    sufficiently populated required bucket, records only permitted scarcity limitations, and
    classifies every surfaced contract-relevant contradiction before its privacy-clean receipt is
    accepted.
14. All specified focused, three-crate, workspace, formatting, clippy, diff, and GitNexus gates pass.
15. The final bounded P7 change receives fresh independent review before commit/closeout claims.

## Deferred Work

After P7 and the separate P8 receipt:

- additional raw event variants discovered later;
- larger or rotating private samples;
- quota increases;
- broader language/tooling strata;
- fuzzing and property-based malformed input;
- performance and large-corpus benchmarks;
- widening analyzer exports to replace heuristic strata;
- scorer or eligibility changes;
- removal of the legacy compatibility floor; and
- continuous private-corpus automation.

## Open Questions

The specification-stage questions were confirmed when the user directed preparation of the plan and
task ledger. The companion plan must still resolve implementation ordering, per-case ownership, and
the exact private-sampler artifacts without weakening this contract.

Companion documents:

- `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`
- `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`
- active projections in `tasks/plan.md` and `tasks/todo.md`

No implementation task is authorized until the spec/plan/task family has received the requested
bounded review and the user has approved the reviewed family.
