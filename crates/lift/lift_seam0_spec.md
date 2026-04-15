<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-lift-autoplan-restore-20260415-150620.md -->

# `lift` seam 0 spec

This document defines the exact deliverable for **SEAM-0: crate scaffold and workspace wiring**.

SEAM-0 exists to create the package, binary, file layout, feature flags, lint posture, and command skeleton **without implementing any scoring, parsing, repo walking, or Lift business logic**.

---

## 1. Goal

Create a compileable workspace member at `crates/lift` with:

- package name `substrate-lift`
- library crate name `substrate_lift`
- binary name `lift`
- a thin CLI skeleton
- reserved module layout for future seams
- zero domain logic
- stable build/test/doc commands

The result must be a safe base for parallel implementation work in later seams.

---

## 2. Decisions locked by SEAM-0

### 2.1 Names

```text
Workspace path: crates/lift
Cargo package: substrate-lift
Library crate: substrate_lift
Binary / CLI: lift
```

### 2.2 Workspace integration

- Add `"crates/lift"` to the root `[workspace].members` array.
- Do **not** add `substrate-lift` as a root dependency.
- Do **not** modify the root binary list.
- Do **not** modify release/dist wiring yet.

### 2.3 Rust floor and edition

SEAM-0 aligns to the current repository floor unless a separate repo-wide upgrade lands first.

```toml
edition = "2021"
rust-version = "1.89"
```

### 2.4 Publishing posture

SEAM-0 must prevent accidental publishing.

```toml
publish = false
```

### 2.5 Feature-flag posture

SEAM-0 reserves feature names now, but only `cli` is allowed to do real work in this seam.

---

## 3. Exact file and directory shape

### 3.1 Required tree

```text
crates/lift/
  Cargo.toml
  README.md
  src/
    lib.rs
    error.rs
    bin/
      lift.rs
    cli/
      mod.rs
      args.rs
      run.rs
    api/
      mod.rs
    compat/
      mod.rs
    core/
      mod.rs
    policy/
      mod.rs
    repo/
      mod.rs
    languages/
      mod.rs
    graph/
      mod.rs
    detect/
      mod.rs
    resolve/
      mod.rs
    runner/
      mod.rs
  tests/
    cli_help.rs
    compile_matrix.rs
  fixtures/
    README.md
  schemas/
    README.md
  profiles/
    README.md
```

### 3.2 Directory ownership in SEAM-0

- `src/cli/**` may contain real code.
- `src/error.rs` may contain a real error enum.
- every other module is **placeholder-only** in SEAM-0.
- `fixtures/`, `schemas/`, and `profiles/` exist only as reserved directories with README placeholders.

---

## 4. Exact Cargo manifest shape

Use this exact starting point unless the repo root grows shared workspace package metadata first.

```toml
[package]
name = "substrate-lift"
version = "0.1.0"
edition = "2021"
rust-version = "1.89"
description = "Deterministic lift scoring engine and CLI scaffold"
license = "MIT"
repository = "https://github.com/atomize-hq/substrate"
readme = "README.md"
publish = false
keywords = ["lift", "cli", "analysis"]
categories = ["command-line-utilities", "development-tools"]

[lib]
name = "substrate_lift"
path = "src/lib.rs"

[[bin]]
name = "lift"
path = "src/bin/lift.rs"
required-features = ["cli"]
doc = false

[features]
default = ["cli"]
cli = ["dep:clap"]
compat-v1 = []
config-lang = []
rust-lang = []
python-lang = []
javascript-lang = []
typescript-lang = []
substrate-profile = []

[dependencies]
clap = { version = "4.5", features = ["derive"], optional = true }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"

[lints.rust]
unsafe_code = "forbid"
unreachable_pub = "warn"
unused_crate_dependencies = "warn"

[lints.clippy]
dbg_macro = "warn"
print_stdout = "allow"
print_stderr = "allow"
todo = "warn"
unwrap_used = "warn"
```

### 4.1 Manifest rules

- No `tree-sitter`, `cargo_metadata`, `git2`, `serde`, `schemars`, or `anyhow` in SEAM-0.
- No build script.
- No proc-macro targets.
- No examples, benches, or integration test helpers beyond the two test files above.
- `required-features = ["cli"]` is mandatory so `--no-default-features` can still compile the library.

---

## 5. Exact library surface

### 5.1 `src/lib.rs`

```rust
#![forbid(unsafe_code)]

pub mod error;

#[cfg(feature = "cli")]
pub use cli::run as run_cli;

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod api;
pub(crate) mod compat;
pub(crate) mod core;
pub(crate) mod policy;
pub(crate) mod repo;
pub(crate) mod languages;
pub(crate) mod graph;
pub(crate) mod detect;
pub(crate) mod resolve;
pub(crate) mod runner;
```

### 5.2 Public API rule

The **only** public items allowed in SEAM-0 are:

- `error` module
- `run_cli` function when the `cli` feature is enabled

Everything else must remain `pub(crate)` or private.

### 5.3 Placeholder module contract

Every placeholder `mod.rs` must contain only:

- a module-level doc comment
- optionally one `pub(crate)` marker type named `ReservedForFutureSeam`
- no runtime logic
- no dependencies beyond `std`

Example:

```rust
//! Reserved for SEAM-3 and later. No runtime logic in SEAM-0.

#[allow(dead_code)]
pub(crate) struct ReservedForFutureSeam;
```

---

## 6. Exact error shape

### 6.1 `src/error.rs`

SEAM-0 uses a tiny hand-written error enum.

```rust
#[derive(Debug)]
pub enum LiftError {
    NotImplemented(&'static str),
    Cli(String),
}
```

### 6.2 Error rules

- No third-party error libraries in SEAM-0.
- `Display` and `std::error::Error` must be implemented manually.
- `NotImplemented` is allowed **only** for CLI leaf execution paths, not for `--help`.

---

## 7. Exact CLI shape

### 7.1 Top-level command tree

SEAM-0 locks the **names** and **nesting** of the command tree.

```text
lift
  score
    vector
    diff
  estimate
    path
    symbol
  analyze
    path
    symbol
  explain
  validate
    vector
    config
  print-schema
  print-model
```

### 7.2 Help behavior contract

These must all exit `0` and print help:

```text
lift --help
lift help
lift score --help
lift score vector --help
lift score diff --help
lift estimate --help
lift estimate path --help
lift estimate symbol --help
lift analyze --help
lift analyze path --help
lift analyze symbol --help
lift explain --help
lift validate --help
lift validate vector --help
lift validate config --help
lift print-schema --help
lift print-model --help
```

### 7.3 Execution behavior contract

For non-help invocation, SEAM-0 only guarantees this behavior:

- parse the command successfully if the syntax is valid
- return a `NotImplemented` error
- avoid panics
- avoid any filesystem, repo, policy, or parser work

No other runtime semantics are contractual in SEAM-0.

### 7.4 Exact parser shape

Use nested clap derive enums/structs:

```rust
Cli
  command: Option<Command>

Command
  Score(ScoreCommand)
  Estimate(EstimateCommand)
  Analyze(AnalyzeCommand)
  Explain(ExplainArgs)
  Validate(ValidateCommand)
  PrintSchema(PrintSchemaArgs)
  PrintModel(PrintModelArgs)
```

With nested enums:

```rust
ScoreCommand    -> Vector | Diff
EstimateCommand -> Path | Symbol
AnalyzeCommand  -> Path | Symbol
ValidateCommand -> Vector | Config
```

### 7.5 Flag contract

SEAM-0 locks only these common flags now:

- `--help`
- `--version`

All other flags are out of scope for SEAM-0.

---

## 8. Exact file contents that are intentionally placeholders

### `schemas/README.md`
Must say:

- request/response/vector/model schemas are **not** implemented in SEAM-0
- this directory is reserved for SEAM-1 and SEAM-2

### `profiles/README.md`
Must say:

- policy/profile packs are **not** implemented in SEAM-0
- this directory is reserved for SEAM-3 and later

### `fixtures/README.md`
Must say:

- fixture repos are **not** implemented in SEAM-0
- SEAM-0 uses only minimal CLI and compile tests

---

## 9. In-scope

SEAM-0 includes only the following:

1. workspace membership wiring
2. package manifest
3. binary target wiring
4. feature flags
5. reserved directory layout
6. minimal error type
7. CLI help tree and parser skeleton
8. compile/test/doc/lint baseline
9. README placeholders for future dirs

---

## 10. Out of scope

SEAM-0 explicitly excludes:

1. Lift vector structs
2. request/response API structs
3. model AST
4. scoring logic
5. v1 compatibility logic
6. repo snapshotting or git integration
7. policy loading
8. language adapters
9. graph/scope resolution
10. detectors and evidence
11. vector resolution
12. JSON output contracts
13. `lift.toml` or any runtime config file
14. publishing to crates.io
15. release automation
16. `substrate lift ...` wrapper integration

If any of the above appears in the SEAM-0 PR, the seam has leaked.

---

## 11. Acceptance criteria

All criteria are required.

### 11.1 Workspace acceptance

- root `Cargo.toml` includes `"crates/lift"` in `[workspace].members`
- `cargo metadata --format-version 1 --no-deps` includes package `substrate-lift`

### 11.2 Build acceptance

These commands must pass:

```bash
cargo check -p substrate-lift
cargo check -p substrate-lift --no-default-features
cargo check -p substrate-lift --all-features
cargo test -p substrate-lift
cargo doc -p substrate-lift --no-deps
cargo fmt --all --check
```

### 11.3 CLI acceptance

These commands must pass:

```bash
cargo run -p substrate-lift --bin lift -- --help
cargo run -p substrate-lift --bin lift -- score --help
cargo run -p substrate-lift --bin lift -- estimate --help
cargo run -p substrate-lift --bin lift -- analyze --help
cargo run -p substrate-lift --bin lift -- validate --help
cargo run -p substrate-lift --bin lift -- print-schema --help
cargo run -p substrate-lift --bin lift -- print-model --help
```

### 11.4 Test acceptance

`tests/cli_help.rs` must assert:

- top-level help exits 0
- representative nested help invocations exit 0
- help text includes `score`, `estimate`, `analyze`, `validate`, `print-schema`, `print-model`

`tests/compile_matrix.rs` must assert at least:

- `run_cli` is available when `cli` is enabled
- the crate compiles with `--no-default-features`
- the binary is gated behind `required-features = ["cli"]`

### 11.5 Negative acceptance

- `cargo check -p substrate-lift --no-default-features` must **not** try to compile clap-backed CLI code
- no non-help CLI path may panic
- no SEAM-0 code may touch the filesystem except the clap-generated help flow

---

## 12. Invariants

These are permanent unless a later explicit architecture decision changes them.

1. The binary name is `lift`.
2. The package name is `substrate-lift`.
3. SEAM-0 exposes no domain API.
4. The scoring engine is not started in SEAM-0.
5. All non-CLI seam directories exist after SEAM-0.
6. `lib.rs` remains the only root of library wiring.
7. `src/bin/lift.rs` must be a thin trampoline into library code.
8. The CLI module owns clap; no other module may depend on clap in SEAM-0.
9. The crate must compile without default features.
10. No heavy analysis dependencies are allowed in SEAM-0.
11. Publishing is disabled.
12. Help output is stable enough for snapshot-style testing.
13. The crate is safe Rust only.
14. Placeholder modules may not smuggle in business logic.

---

## 13. Falsification questions

If any answer is “yes”, SEAM-0 is not done or the boundary has been violated.

1. Does any module outside `src/cli/**` import or depend on `clap`?
2. Does any SEAM-0 file contain scoring logic, model logic, or repo logic?
3. Does `src/bin/lift.rs` do anything other than call into the library?
4. Does `cargo check -p substrate-lift --no-default-features` fail?
5. Does any help command exit non-zero?
6. Does any SEAM-0 command path read repo files, call git, or touch the network?
7. Are any future seam modules exported publicly from `lib.rs`?
8. Is `publish = false` missing?
9. Did SEAM-0 add `serde`, `tree-sitter`, `cargo_metadata`, or other future-seam deps?
10. Did SEAM-0 add actual JSON schemas rather than reserving the schema directory?
11. Did SEAM-0 add command flags whose semantics depend on future seams?
12. Can the crate panic on an otherwise valid CLI parse path?
13. Does the workspace stop compiling after adding `crates/lift`?
14. Is the binary named anything other than `lift`?
15. Is the package named anything other than `substrate-lift`?

---

## 14. Risks and mitigations

### Risk 1: `lift` is a very generic binary name
- **Impact:** path/name collisions in developer environments
- **Mitigation:** keep package name `substrate-lift`; only binary is generic
- **Status:** accepted by design

### Risk 2: premature public API lock-in
- **Impact:** later seams inherit accidental surface area
- **Mitigation:** expose only `error` and `run_cli`; everything else is `pub(crate)`

### Risk 3: feature-flag churn later
- **Impact:** manifest instability across seams
- **Mitigation:** reserve future feature names in SEAM-0 even if they are empty

### Risk 4: heavy dependencies sneak in early
- **Impact:** slow builds and tangled boundaries
- **Mitigation:** explicit dependency denylist in acceptance criteria

### Risk 5: placeholder modules become logic dumps
- **Impact:** clean seam boundaries erode immediately
- **Mitigation:** placeholder module contract limits content to docs + reserved marker type

### Risk 6: mixed repo/toolchain assumptions
- **Impact:** workspace compile failures
- **Mitigation:** align `edition` and `rust-version` to the current repo floor in SEAM-0

### Risk 7: help output becomes unstable
- **Impact:** flaky tests and churn
- **Mitigation:** keep help surface minimal in SEAM-0; delay semantic flags

---

## 15. Test plan

### 15.1 Unit-level

- `error` display text tests
- CLI parse tests for top-level and nested help

### 15.2 Integration-level

- `tests/cli_help.rs`
- `tests/compile_matrix.rs`

### 15.3 Non-goal tests in SEAM-0

Do **not** add:

- fixture repo tests
- JSON schema tests
- repo snapshot tests
- parser tests
- scoring tests

---

## 16. PR checklist

Every SEAM-0 PR must answer all of the following in the description:

- [ ] Added `crates/lift` to workspace members
- [ ] Package name is `substrate-lift`
- [ ] Binary name is `lift`
- [ ] `publish = false` set
- [ ] `required-features = ["cli"]` set on binary
- [ ] `cargo check -p substrate-lift` passes
- [ ] `cargo check -p substrate-lift --no-default-features` passes
- [ ] `cargo test -p substrate-lift` passes
- [ ] `cargo doc -p substrate-lift --no-deps` passes
- [ ] No heavy future-seam deps were added
- [ ] No business logic was added outside the CLI skeleton
- [ ] Placeholder dirs and files exist

---

## 17. Recommended minimal contents

### `src/bin/lift.rs`

```rust
fn main() {
    #[cfg(feature = "cli")]
    {
        if let Err(err) = substrate_lift::run_cli() {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("lift was built without the 'cli' feature");
        std::process::exit(1);
    }
}
```

### `src/cli/run.rs`

```rust
pub fn run() -> Result<(), crate::error::LiftError> {
    let cli = crate::cli::args::Cli::parse();
    crate::cli::args::dispatch(cli)
}
```

### `README.md`

Must state clearly:

- this crate is scaffold-only in SEAM-0
- later seams add scoring and analysis
- the canonical executable name is `lift`

---

## 18. Explicit non-goals for review discipline

Reviewers should reject SEAM-0 if they see any of the following words in code outside comments/tests/docs:

- `tree_sitter`
- `cargo_metadata`
- `git2`
- `serde`
- `schema`
- `score`
- `vector`
- `policy`
- `detector`
- `evidence`
- `snapshot`

These are future seam concerns.

---

## 19. Exit condition

SEAM-0 is complete when:

- the workspace recognizes `crates/lift`
- the `lift` binary exists and has the locked help tree
- the crate builds/tests/docs cleanly with and without default features
- the codebase contains only scaffold logic, not domain logic
- later seams can land without changing names, package layout, or top-level wiring

---

## /autoplan Review

Run date: 2026-04-15
Branch: `feat/lift`
Base branch: `main`
Review mode: `SELECTIVE_EXPANSION`
UI scope: `no`
Outside voice status: `codex-only` for CEO and eng passes. No delegated subagent was used in this session.

### Review input snapshot

- Reviewed documents:
  - `crates/lift/README.md`
  - `crates/lift/lift_seam0_spec.md`
- Reviewed code reality:
  - `crates/lift/Cargo.toml`
  - `crates/lift/src/lib.rs`
  - `crates/lift/src/bin/lift.rs`
  - root `Cargo.toml`
- Verified commands:
  - `cargo check -p substrate-lift`
  - `cargo check -p substrate-lift --no-default-features`
  - `cargo check -p substrate-lift --all-features`
  - `cargo test -p substrate-lift`
  - `cargo doc -p substrate-lift --no-deps`
  - `cargo run -p substrate-lift --bin lift -- --help`
  - `cargo run -p substrate-lift --bin lift -- score --help`
- Current reality:
  - compile/doc checks pass
  - test suite passes with zero tests
  - help commands do not render clap help, they print the stub message from `src/bin/lift.rs`
  - required SEAM-0 directories and placeholder files do not exist yet

### Premise challenge

Solid premises:

- One workspace member with a thin binary and hard internal boundaries is the right starting containment for SEAM-0.
- Keeping SEAM-0 scaffold-only is the right boundary. No scoring, parsing, repo walking, or policy logic belongs here.
- Locking names, package identity, and no-default-features compilation is useful because later seams should not reopen those basics.

Assumed premises:

- The README treats a generic reusable engine as already justified. That is still a hypothesis, not a proven product need.
- The README treats v1 semantic parity as a core goal before showing that legacy semantics are worth preserving.
- The spec locks a wide CLI taxonomy before the branch has proven which workflows matter.

Recommendation:

- Keep SEAM-0 as the strict scaffold seam.
- Reframe the big-picture README as `Substrate-first, generic later if earned`.
- Treat the full CLI taxonomy as provisional beyond what SEAM-0 must exercise for help and parser shape.

### Existing code leverage

What already exists in this branch:

- Workspace wiring already includes `crates/lift`.
- Package identity already uses `substrate-lift`, `substrate_lift`, and `lift`.
- The branch already proves the crate can compile with and without default features.

What does not exist yet:

- `publish = false`
- `required-features = ["cli"]` on the binary target
- the reserved module tree under `src/`
- placeholder `fixtures/`, `schemas/`, and `profiles/` directories
- `src/error.rs`
- a real CLI parser/help tree
- any SEAM-0 tests

### Dream state mapping

Current:

- Workspace member exists.
- Crate is a stub.
- Help surface is not implemented.
- Acceptance checks are mostly passing because the crate does almost nothing.

This plan if implemented literally:

- SEAM-0 becomes a strict scaffold seam with a help-only CLI tree, reserved layout, and compile/test/doc baseline.
- Later seams inherit stable names, feature flags, and top-level wiring.

12-month ideal:

- `lift` changes real Substrate decisions on real diffs.
- Reuse outside Substrate is earned by a second consumer, not assumed in advance.
- Deterministic scoring, evidence, and policy semantics are explicit enough that fixtures catch regressions instead of hiding them.

Dream state delta:

- The current spec is good at boundary control.
- The README overcommits on generic-engine scope and long-term CLI/API shape too early.
- The missing move is a product thesis and an internal-first checkpoint before broadening scope.

### Implementation alternatives

| Approach | Pros | Cons | Recommendation |
|---|---|---|---|
| Keep current SEAM-0 spec exactly as written | Strong scaffold boundary, concrete acceptance | Overcommits CLI taxonomy early, collides with README defaults and long-term surface | Reject as-is |
| Keep strict SEAM-0 scaffold, but narrow or explicitly provisionalize long-term CLI/API commitments | Preserves scaffold value, reduces accidental API lock-in, keeps implementation parallel-friendly | Requires clarifying contradictions now | Recommended |
| Promote README broad engine roadmap as the immediate contract | Big vision is explicit | Turns SEAM-0 into architecture theater before proving usefulness | Reject |

### Mode-specific scope decision

Accepted scope for review:

- SEAM-0 scaffold correctness
- alignment between the SEAM-0 spec, README, and current branch state
- exact acceptance/test gaps

Deferred from this review:

- implementing the scaffold
- deciding the full long-term engine surface beyond the user challenges below

### CEO review

#### CODEX SAYS (CEO, codex-only)

- The strongest strategic complaint is that the README jumps from `scaffold a crate` to `generic reusable engine` without proving an internal user.
- The second complaint is that the CLI taxonomy is frozen too early for a seam whose job is just to build safe wiring.
- The third complaint is that compatibility and multi-language promises are treated as inevitable instead of earned.

#### CEO consensus table

| Dimension | Primary review | Codex | Consensus |
|---|---|---|---|
| Premises valid? | Mixed | Mixed | Confirmed concern |
| Right problem to solve? | Reframe internal-first | Reframe internal-first | Confirmed concern |
| Scope calibration correct? | Too broad in long-term commitments | Too broad in long-term commitments | Confirmed concern |
| Alternatives sufficiently explored? | No | No | Confirmed concern |
| Competitive/market risks covered? | Weakly | Weakly | Confirmed concern |
| 6-month trajectory sound? | Only if scope narrows | Only if scope narrows | Confirmed concern |

CEO summary:

- The branch should keep the SEAM-0 scaffold goal.
- The README should stop speaking as if external reuse, v1 parity, and multi-language adoption are already justified.
- The real product bet should be written down before later seams make the wrong thing rigid.

### Error & Rescue Registry

| Failure | User-visible effect | Detection | Rescue |
|---|---|---|---|
| `lift --help` prints stub text instead of help | Users cannot discover the locked command tree | CLI acceptance commands | Implement clap tree and help tests |
| Binary bypasses library trampoline | Feature gating and error handling drift out of sync | `src/bin/lift.rs` review, trampoline test | Route all execution through `run_cli` |
| SEAM-0 spec and README disagree on defaults/API surface | Parallel work starts from conflicting contracts | review diffs, doc review | Pick one authority and align the other |
| No placeholder tree exists | Later seams invent layout ad hoc | file tree inspection | Add reserved modules and directory READMEs now |
| Zero tests allow regressions to slip | branch appears healthy while contract is broken | `cargo test` plus file inspection | Add `cli_help.rs` and `compile_matrix.rs` immediately |

### Failure Modes Registry

| Failure mode | Severity | Why it matters |
|---|---|---|
| Full CLI tree is locked before workflows are validated | High | Creates accidental public surface and slows future correction |
| README defaults conflict with SEAM-0 defaults | High | Implementers will choose different truths and diverge |
| Hard internal seams have no enforcement mechanism | Medium | One-crate plan decays into a god crate unless guarded |
| Compat-v1 is treated as a default commitment | Medium | Preserves legacy semantics before proving they deserve it |
| Determinism claims are underspecified | Medium | Later seams will each invent their own version of deterministic behavior |

### Design review

Skipped. No UI scope was detected in the plan or code under review.

### Engineering review

#### Scope challenge

The minimum set of changes that actually achieves SEAM-0 is smaller than the README’s future architecture. The branch only needs:

- manifest fixes
- reserved tree and placeholder files
- tiny error type
- thin trampoline
- clap help tree
- acceptance tests

Anything beyond that is future-seam work and should stay out.

#### CODEX SAYS (eng, codex-only)

- The spec and README contradict each other on feature defaults, API/public surface, and even versioning.
- The binary is already regressed against the spec because it does not call into library code and does not expose clap help.
- The one-crate design needs enforcement if “hard seams” is supposed to mean anything.

#### ENG consensus table

| Dimension | Primary review | Codex | Consensus |
|---|---|---|---|
| Architecture sound? | Only after contradictions are resolved | Not yet | Confirmed concern |
| Test coverage sufficient? | No | No | Confirmed concern |
| Performance risks addressed? | N/A for SEAM-0 runtime, doc/perf risk later | N/A for SEAM-0 runtime, doc/perf risk later | Confirmed |
| Security threats covered? | Mostly N/A for SEAM-0, but boundary drift risk exists | Same | Confirmed |
| Error paths handled? | No | No | Confirmed concern |
| Deployment/distribution risk manageable? | Yes for SEAM-0 because publishing is out of scope | Yes | Confirmed |

#### Architecture ASCII diagram

Current branch:

```text
target/debug/lift
    |
    +--> src/bin/lift.rs
            |
            +--> eprintln!("lift scaffold created; CLI implementation is not wired yet")

No library trampoline
No cli module
No error module
No parser tree
No tests
```

Required SEAM-0 shape:

```text
target/debug/lift
    |
    +--> src/bin/lift.rs
            |
            +--> substrate_lift::run_cli()
                    |
                    +--> src/cli/run.rs
                            |
                            +--> src/cli/args.rs clap parser
                                    |
                                    +--> help path => clap renders help, exit 0
                                    |
                                    +--> valid non-help leaf => LiftError::NotImplemented
```

#### Code quality review

- The crate is under-engineered relative to the spec, not over-engineered.
- The quality problem is not messy code, it is missing code that the spec already claims is required.
- The biggest structural smell in the docs is contradiction, not implementation complexity.

#### Test review

Code path coverage for SEAM-0:

```text
CODE PATH COVERAGE
===========================
[+] crates/lift/src/bin/lift.rs
    |
    ├── [GAP] thin trampoline into library
    ├── [GAP] cli-enabled path renders clap help
    └── [GAP] cli-disabled path exits with explicit error

[+] crates/lift/src/lib.rs
    |
    ├── [GAP] public surface limited to error + run_cli
    └── [GAP] no-default-features compile path avoids clap-backed code

[+] Planned cli tree
    |
    ├── [GAP] lift --help
    ├── [GAP] lift help
    ├── [GAP] lift score --help
    ├── [GAP] lift estimate --help
    ├── [GAP] lift analyze --help
    ├── [GAP] lift validate --help
    ├── [GAP] lift print-schema --help
    └── [GAP] lift print-model --help

[+] Non-help parse paths
    |
    ├── [GAP] valid leaf returns LiftError::NotImplemented without panic
    ├── [GAP] invalid syntax returns clap parse error with stable exit behavior
    └── [GAP] invalid subcommand returns clap parse error with stable exit behavior

---------------------------------
COVERAGE: 0/13 paths tested (0%)
GAPS: 13 paths need tests
---------------------------------
```

Regression rule applied:

- Current `lift --help` and `lift score --help` behavior is already regressed against the spec. These need regression tests in the first implementation pass.

Test plan artifact:

- `/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-lift-eng-review-test-plan-20260415-150620.md`

#### Performance review

SEAM-0 itself has no meaningful runtime performance surface beyond clap startup. The performance risk is organizational:

- broad default features create unnecessary compile/test matrix cost
- one-crate boundaries without enforcement increase future compile churn and coupling

### Cross-phase themes

- Internal-first framing appeared in both the CEO and eng passes.
- Contradictory contracts between `README.md` and this spec appeared in both passes.
- The branch currently passing checks while violating the seam contract appeared in both passes.

### NOT in scope

- adding scoring logic
- adding request/response or schema structs
- adding repo walking, git, policy, graph, detect, or resolution logic
- adding publishing/release automation
- solving the full generic-engine product thesis in SEAM-0

### Deferred to `TODOS.md`

`TODOS.md` does not exist in this repo root today. Deferred items that should be captured if you approve the review direction:

- write a short product thesis for `lift`
- add a determinism ADR before post-SEAM-0 engine work
- choose an enforcement mechanism for hard internal seams

### User challenges

1. Reframe the README from `generic reusable engine` to `Substrate-first decision primitive until reuse is earned`.
2. Relax the idea that SEAM-0 must permanently lock the full long-term CLI taxonomy before workflows are validated.

### Taste decisions

1. Keep one crate for now, but add seam-enforcement tests or lints.
2. Keep compat feature names reserved, but do not make `compat-v1`, `rust-lang`, and `config-lang` default in SEAM-0.

### Approval resolution

Approval status: approved with overrides on 2026-04-15.

Resolved by user:

- Choice 1: keep this as a single crate inside the existing Substrate monorepo. The review should not imply a near-term multi-crate split unless a later seam proves it necessary.
- Choice 2: keep the broad default feature posture from `README.md`.
- Challenge 1: keep the generic reusable engine framing in `README.md`. Do not reframe the project as Substrate-first.
- Challenge 2: treat the CLI taxonomy as the current intended plan, but review it for coherence at each seam and change it if real usage reveals a better shape.
- Additional direction: keep CLI commands as thin wrappers over orthogonal library operations so future API endpoints can reuse the same core without command-specific logic leaks.

Implications for later seams:

- The README remains the source of truth for the broad product direction.
- This SEAM-0 spec still owns the scaffold boundary and acceptance contract.
- Future seams should preserve an architecture where command handlers are delivery adapters, not the home of business logic.

### Completion summary

| Phase | Status | Key result |
|---|---|---|
| CEO | approved_with_overrides | Generic-engine framing retained by explicit user decision |
| Design | skipped | No UI scope |
| Eng | approved_with_overrides | Scaffold gaps remain real, but the single-crate and broad-default choices are accepted |

Overall verdict:

- SEAM-0 as a scaffold seam is correct.
- The current branch is not yet SEAM-0-complete.
- The docs should be aligned before implementation continues so later seams do not inherit contradictions.
- Alignment now means: preserve the generic-engine README direction while tightening this spec around scaffold truth and thin-wrapper delivery boundaries.

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Review `lift_seam0_spec.md` as the authoritative plan file and `README.md` as big-picture context | mechanical | P3 pragmatic | User pointed to the spec for SEAM-0 specifics and the README for big-picture context | Reviewing the README alone |
| 2 | CEO | Skip design review | mechanical | P3 pragmatic | UI scope detection found no user-facing UI work in the plan | Running a design pass on non-UI scope |
| 3 | CEO | Keep SEAM-0 scaffold-only | mechanical | P1 completeness | The seam already states that scoring, parsing, repo walking, and policy logic are out of scope | Expanding SEAM-0 into partial engine logic |
| 4 | CEO | Treat `generic reusable engine` as assumed, not proven | user_challenge | P2 boil lakes | Both review passes found the internal-first framing stronger than promising external reuse now | Accepting the README framing unchanged |
| 5 | CEO | Treat the full CLI taxonomy as too rigid for a scaffold seam | user_challenge | P5 explicit | Locking many future nouns now creates accidental public surface before workflow validation | Freezing the entire long-term CLI shape without caveat |
| 6 | Eng | Hold one crate for SEAM-0 but require seam-enforcement follow-up | taste | P3 pragmatic | One package is fine for scaffold speed, but “hard seams” need a guardrail | Immediate multi-crate split |
| 7 | Eng | Require `required-features = ["cli"]` and a binary trampoline test | mechanical | P1 completeness | The spec already requires both and the current branch fails them | Trusting manual review only |
| 8 | Eng | Treat current help output as a regression | mechanical | P6 bias toward action | The spec explicitly requires help text and current runtime does not provide it | Deferring help tests until later seams |
| 9 | Eng | Keep compat feature names reserved but not default-enabled in SEAM-0 | taste | P5 explicit | Empty default features create fake commitments and compile matrix noise | Keeping current default feature set from the README |
| 10 | Eng | Add a determinism ADR before post-SEAM-0 engine work | mechanical | P1 completeness | Determinism is a core claim in the README but its rules are underspecified | Letting later seams invent determinism ad hoc |
| 11 | Gate | Keep the project framed for broad reuse, not Substrate-first | user_override | User sovereignty | User explicitly rejected the Substrate-first reframing and wants the README direction preserved | Reframing the project around internal-only adoption |
| 12 | Gate | Keep the broad default feature posture | user_override | User sovereignty | User explicitly approved the broader default feature posture from `README.md` | Narrowing defaults to `cli` only |
| 13 | Gate | Keep one crate and do not imply a near-term multi-crate split | user_override | User sovereignty | User clarified the monorepo shape and wants this review focused on the core crate, not a crate explosion | Pushing an early multi-crate restructure |
| 14 | Gate | Treat CLI taxonomy as an initial plan subject to seam-by-seam coherence review | user_override | User sovereignty | User wants flexibility based on real usage while keeping the current planned command taxonomy | Freezing the CLI forever or discarding the current taxonomy now |
| 15 | Gate | Add orthogonality rule: CLI commands are thin wrappers over reusable library operations that can later back API endpoints | mechanical | P5 explicit | This keeps delivery surfaces replaceable and prevents command-specific logic from becoming the core API | Letting CLI handlers own business logic |
