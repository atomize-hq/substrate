
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
