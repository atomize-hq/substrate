# TASKS: World-Scoped CLI Runtime Realizability And Codex Guest Runtime Delivery

Source spec: [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Source plan: [PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./PLAN-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Related follow-on: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: draft for review

## Execution Packets

This slice should be implemented as four sequential packets:

1. fail-closed validator/remediation wall,
2. Codex world-deps package or runtime bundle,
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
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [ ] Task 1.2: Emit stable remediation diagnostics for missing guest runtime truth
  - Acceptance: user-facing/runtime-facing errors mention the world runtime remediation path instead of surfacing only a late `127`; diagnostics remain specific to world-scoped runtime posture and do not blur host and world.
  - Verify:
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
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
  - Acceptance: a Substrate-owned world-deps package or bundle can install Codex under `/var/lib/substrate/world-deps/<package>` and expose `/var/lib/substrate/world-deps/bin/codex`; artifact sourcing targets official release artifacts.
  - Verify:
    - `cargo test -p shell world_deps -- --nocapture`
    - manual dry-run/package inspection as appropriate for the package authoring path
  - Expected files touched:
    - world-deps package inventory under the relevant inventory tree
    - associated install script(s)
    - [`docs/reference/world/deps/authoring_packages.md`](../docs/reference/world/deps/authoring_packages.md) only if operator contract examples need updating

- [ ] Task 2.2: Record the verified runtime dependency posture
  - Acceptance: the implementation explicitly proves one of two outcomes: (a) the Linux Codex artifact is self-contained in the guest, or (b) the package widens into a runtime bundle that includes the required guest runtime dependencies.
  - Verify:
    - package-level smoke proof in the target guest environment
    - `cargo test -p shell world_deps -- --nocapture`
  - Expected files touched:
    - package inventory / script files
    - nearby docs or implementation notes that state the verified outcome

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. the guest-visible `codex` entrypoint resolves from `/var/lib/substrate/world-deps/bin`,
2. package installs are idempotent,
3. the verified self-contained-vs-bundle outcome is explicit,
4. the path does not rely on host NVM/npm state.

Do not start Packet 3 until Packet 2 verification is green.

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
  - Expected files touched:
    - [`scripts/substrate/install-substrate.sh`](../scripts/substrate/install-substrate.sh)
    - [`scripts/substrate/install.sh`](../scripts/substrate/install.sh)
    - [`docs/INSTALLATION.md`](../docs/INSTALLATION.md)

- [ ] Task 3.2: Add the dev installer/runtime-provisioning surface
  - Acceptance: dev install exposes the same `--provision-agent-runtime <runtime_family>` shape, supports `codex` only in this slice, fails closed for unsupported values, provisions/install as needed and then runs `substrate world deps current sync`, and documents behavior consistently with prod install.
  - Verify:
    - installer help/usage inspection
    - targeted script tests or dry-run validation if available
  - Expected files touched:
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
  - Expected files touched:
    - no planned source edits; this is the validation gate after the implementation packets above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. fake world launchability is gone,
2. guest runtime delivery is real and documented,
3. installer support is live on both surfaces,
4. Slice 58 can now change config/selector shape without reopening runtime semantics.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.
4. Packet 4 must be green before Slice 58 implementation starts.

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
