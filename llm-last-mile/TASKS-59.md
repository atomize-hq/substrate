# TASKS: World-Scoped CLI Runtime Realizability And Codex Guest Runtime Delivery

Source spec: [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Source plan: [PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Related follow-on: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Phase: `TASKS`  
Execution model: five sequential `/incremental-implementation` sessions  
Status: draft for review

## Execution Packets

This slice should be implemented as five sequential packets:

1. fail-closed validator/remediation wall,
2. Codex world-deps package or runtime bundle,
2.5. remediation of guest-tuple fail-closed and host-runtime leakage gaps,
3. prod/dev installer-time provisioning surfaces,
4. final end-to-end proof and handoff to Slice 58.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Fail-Closed Runtime Truth And Remediation

Session goal:

1. make world-scoped Codex launchability guest-truth instead of host `which` truth,
2. fail closed before retained worker bootstrap,
3. emit actionable remediation text.

### Tasks

- [ ] Task 1.1: Add world-scoped runtime-realizability gating before bootstrap
  - Acceptance: current world-scoped Codex path fails before retained worker bootstrap when guest runtime truth is missing; host-only binary resolution no longer counts as sufficient world launchability proof.
  - Verify:
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [ ] Task 1.2: Emit stable remediation diagnostics for missing guest runtime truth
  - Acceptance: user-facing/runtime-facing errors mention the world runtime remediation path instead of surfacing only a late `127`; diagnostics remain specific to world-scoped runtime posture and do not blur host and world.
  - Verify:
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)
    - relevant runtime/control-surface tests

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. world-scoped Codex no longer claims launchability from host `which` success alone,
2. validator/materialization fails before retained worker bootstrap when guest runtime is absent,
3. remediation guidance is explicit,
4. host-scoped Codex behavior remains unchanged.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Codex Guest Runtime Package Or Runtime Bundle

Session goal:

1. deliver a guest-visible Codex runtime through Substrate-owned world-deps packaging,
2. install under the Substrate-managed prefix,
3. resolve the standalone-binary verification seam explicitly.

### Tasks

- [ ] Task 2.1: Author the Codex world-deps package/bundle and install script
  - Acceptance: a Substrate-owned world-deps package or bundle named `codex-runtime` can install Codex under `/var/lib/substrate/world-deps/<package>` and expose `/var/lib/substrate/world-deps/bin/codex`; artifact sourcing targets official release artifacts.
  - Verify:
    - `cargo test -p shell world_deps -- --nocapture`
    - manual dry-run/package inspection as appropriate for the package authoring path
  - Files:
    - world-deps package inventory under the relevant inventory tree
    - associated install script(s)
    - [`docs/reference/world/deps/authoring_packages.md`](../docs/reference/world/deps/authoring_packages.md) only if operator contract examples need updating

- [ ] Task 2.2: Align the published UAA dependency wiring to `0.3.7`
  - Acceptance: every Slice 59-touched manifest that already pins UAA exactly now resolves `unified-agent-api = "=0.3.7"` and aligned exact sibling UAA crates at `=0.3.7`, and the lockfile reflects that published dependency state.
  - Verify:
    - `rg -n 'unified-agent-api.*0\\.3\\.7|unified-agent-api-codex.*0\\.3\\.7|unified-agent-api-claude-code.*0\\.3\\.7' /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/Cargo.toml /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/Cargo.lock`
  - Files:
    - [`crates/shell/Cargo.toml`](../crates/shell/Cargo.toml)
    - [`crates/gateway/Cargo.toml`](../crates/gateway/Cargo.toml)
    - [`crates/world-service/Cargo.toml`](../crates/world-service/Cargo.toml)
    - [`Cargo.lock`](../Cargo.lock)

- [ ] Task 2.3: Integrate UAA-backed validated version selection and record the verified runtime dependency posture
  - Acceptance: Substrate uses the published `0.3.7` `codex`-feature surface, calls `agent_api::resolve_runtime_support("codex", target_triple)`, uses `record.version` as the validated version to acquire, does not read generated/internal UAA files directly, and explicitly proves one of two outcomes: (a) the Linux Codex artifact is self-contained in the guest, or (b) the package widens into a runtime bundle that includes the required guest runtime dependencies.
  - Verify:
    - package-level smoke proof in the target guest environment
    - `cargo test -p shell world_deps -- --nocapture`
    - targeted tests for the UAA-backed version-resolution call path if added in Substrate
  - Files:
    - Substrate dependency/runtime-selection code that invokes UAA
    - package inventory / script files
    - nearby docs or implementation notes that state the verified outcome

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. the guest-visible `codex` entrypoint resolves from `/var/lib/substrate/world-deps/bin`,
2. package installs are idempotent,
3. the verified self-contained-vs-bundle outcome is explicit,
4. the path does not rely on host NVM/npm state,
5. version selection comes from the published `unified-agent-api = "=0.3.7"` Rust API rather than downstream duplicated logic.

Do not start Packet 2.5 until Packet 2 verification is green.

## Packet 2.5: Guest-Tuple Fail-Closed And Host/World Separation Remediation

Session goal:

1. resolve the remaining Packet 2 review disagreement around guest-tuple truth gaps,
2. prove host Codex truth cannot leak into world Codex truth,
3. keep mixed-platform host/runtime posture explicit before installer work begins.

### Tasks

- [ ] Task 2.5.1: Keep guest-target truth explicit and fail closed on unsupported or unmapped guest tuples
  - Acceptance: Substrate derives the intended world-runtime guest target from the actual guest posture it is provisioning for, and if UAA does not publish validated support for that tuple or Substrate lacks a pinned official release mapping for a UAA-validated tuple, Substrate fails closed with explicit guest-target diagnostics instead of silently remapping, broadening, or treating host runtime truth as sufficient.
  - Verify:
    - `cargo test -p shell world_deps -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - targeted guest-target resolution tests added for the Packet 2.5 seam
  - Files:
    - narrow world-deps/runtime-selection code under [`crates/shell/src/builtins/world_deps/`](../crates/shell/src/builtins/world_deps/)
    - adjacent runtime-selection seams only if required to keep guest-target truth explicit
    - nearby tests covering unsupported or validated-but-unmapped guest tuples

- [ ] Task 2.5.2: Prove host Codex cannot satisfy world runtime truth and make the separation explicit
  - Acceptance: tests and diagnostics prove that host `codex` availability on `PATH` can satisfy only host-scoped/orchestrator Codex truth and never the world-scoped Codex runtime contract; mixed-platform postures that may need both a host Codex binary and a Linux guest Codex binary are treated as two distinct contracts.
  - Verify:
    - `cargo test -p shell world_deps -- --nocapture`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - targeted regression tests for host-PATH leakage and mixed host/world runtime separation
  - Files:
    - the narrowest runtime/world-deps diagnostics seams needed for Packet 2.5
    - nearby tests or implementation notes that make the host-vs-world separation undeniable

### Packet 2.5 Checkpoint

Packet 2.5 is complete only when:

1. unsupported or validated-but-unmapped guest tuples fail closed before Substrate claims the world runtime is installed or launchable,
2. host Codex presence on `PATH` does not make world Codex runtime truth pass,
3. host-scoped and world-scoped Codex runtime truth remain explicitly separate, including on mixed-platform hosts,
4. the remaining Packet 2 review disagreement is resolved without widening into installer work or Slice 58 migration.

Do not start Packet 3 until Packet 2.5 verification is green.

## Packet 3: Prod/Dev Installer-Time Provisioning Support

Session goal:

1. expose install-time provisioning of the Codex world runtime on both installer surfaces,
2. keep provision-vs-sync behavior explicit,
3. align help/docs with actual behavior.

### Tasks

- [ ] Task 3.1: Add the prod installer/runtime-provisioning surface
  - Acceptance: prod install surfaces expose `--provision-agent-runtime <runtime_family>`; this slice supports `codex` only and fails closed for unsupported values; the flag provisions/install as needed and then runs `substrate world deps current sync`; help text and docs say so explicitly.
  - Verify:
    - installer help/usage inspection
    - targeted script tests or dry-run validation if available
  - Files:
    - [`scripts/substrate/install-substrate.sh`](../scripts/substrate/install-substrate.sh)
    - [`scripts/substrate/install.sh`](../scripts/substrate/install.sh)
    - [`docs/INSTALLATION.md`](../docs/INSTALLATION.md)

- [ ] Task 3.2: Add the dev installer/runtime-provisioning surface
  - Acceptance: dev install exposes the same `--provision-agent-runtime <runtime_family>` shape, supports `codex` only in this slice, fails closed for unsupported values, provisions/install as needed and then runs `substrate world deps current sync`, and documents behavior consistently with prod install.
  - Verify:
    - installer help/usage inspection
    - targeted script tests or dry-run validation if available
  - Files:
    - [`scripts/substrate/dev-install-substrate.sh`](../scripts/substrate/dev-install-substrate.sh)
    - [`scripts/substrate/world-enable.sh`](../scripts/substrate/world-enable.sh)
    - [`docs/INSTALLATION.md`](../docs/INSTALLATION.md)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. both prod and dev installers expose an explicit runtime-provisioning flag,
2. that flag is generic to runtime family even though Slice 59 only implements `codex`,
3. help/docs state that the flag provisions/install as needed and then runs sync,
4. installer UX does not imply host-runtime fallback,
5. operator truth is consistent across surfaces.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: End-To-End Proof And Slice 58 Handoff Gate

Session goal:

1. prove the runtime path is semantically truthful end-to-end,
2. stop before any Slice 58 placement-aware migration begins.

### Tasks

- [ ] Task 4.1: Run the final validation wall and smoke proof
  - Acceptance: validator/runtime tests, world-deps tests, installer help/docs checks, and guest-runtime smoke proof are green; proof shows the world runtime is resolved from guest-visible world-deps paths instead of host-local NVM paths.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell world_deps -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - packet-specific guest smoke proof command(s) captured in the implementation session
  - Files:
    - no planned source edits; this is the validation gate after the implementation packets above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. fake world launchability is gone,
2. guest runtime delivery is real and documented,
3. installer support is live on both surfaces,
4. Slice 58 can now change config/selector shape without reopening runtime semantics.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 2.5.
3. Packet 2.5 blocks Packet 3.
4. Packet 3 blocks Packet 4.
5. Packet 4 must be green before Slice 58 implementation starts.

## Inter-Packet Review Rules

After each packet:

1. confirm its verification commands are green,
2. confirm the checkpoint is satisfied,
3. confirm no packet silently widened into placement-aware selector migration,
4. confirm the Substrate-owned world-deps delivery path remains the chosen contract.

Implementation-gate rules for packet sessions:

1. run GitNexus impact analysis before editing any production symbol in validator, dispatch, world-deps, or installer code,
2. warn if any impact result is HIGH or CRITICAL before proceeding,
3. run `gitnexus_detect_changes()` before committing.

Reopen spec/plan/tasks only if:

1. the repo cannot express guest runtime truth without first landing Slice 58 schema changes,
2. the official release artifact cannot be made to fit the world-deps script-package contract,
3. installer surfaces need fundamentally different concepts between dev and prod,
4. or live evidence proves world refresh/restart is required after sync.

## Packet Session Final Message Requirements

Every packet implementation session should end by stating:

1. which verification commands passed or failed,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether spec/plan/tasks need reopening,
5. the GitNexus impact-analysis results for each production symbol edited in that packet,
6. and whether Slice 58 remains cleanly deferred until Slice 59 is complete.
