# PLAN-03: Canonical Guest Endpoint and Transport Contract

Source spec:
- [`SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](./SPEC-03-canonical-guest-endpoint-and-transport-contract.md)

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-02-lima-version-floor-and-breakglass-contract.md`](./SPEC-02-lima-version-floor-and-breakglass-contract.md)
- [`PLAN-02.md`](./PLAN-02.md)
- [`TASKS-02.md`](./TASKS-02.md)

Plan type: source-driven phase-1 runtime-contract slice for the macOS hardened
same-user Lima program  
Phase: `PLAN`  
Status: draft plan

## Plan summary

The next honest seam is Slice `03`: define and implement one canonical guest
endpoint and transport contract so the backend and shell stop disagreeing on
the same macOS Lima transport story.

This plan should produce a bounded landing that:

1. centralizes the macOS transport authority in code,
2. removes stale `7788` drift from Slice `03`-owned surfaces,
3. aligns backend and minimal shell-facing transport mappings on one contract,
4. leaves PTY/non-PTY/doctor/readiness consumer convergence for Slice `04`.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md`
4. `crates/world-mac-lima/src/transport.rs`
5. `crates/world-mac-lima/src/forwarding.rs`
6. `crates/world-mac-lima/src/lib.rs`
7. `crates/shell/src/execution/platform_world/mod.rs`
8. `crates/shell/src/builtins/world_gateway.rs` only if a shell-visible
   contradiction remains after transport centralization
9. targeted tests in the same files or existing shell test coverage as needed

Top-level docs such as `docs/WORLD.md`,
`docs/reference/world/platforms/macos-lima-setup.md`,
`scripts/mac/lima-doctor.sh`, and
`crates/shell/src/execution/platform/macos.rs` are follow-on scope unless
execution proves an immediate contradiction and the expansion is explicitly
approved.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice
2. `source-driven-development` is also required

Plan consequence:

1. use live repo truth plus official Lima docs as co-equal authorities,
2. preserve the Slice `02` support-versus-breakglass framing while changing
   only the implementation contract underneath it,
3. treat any disagreement between current code and current official Lima
   transport semantics as an explicit design/input issue rather than silently
   guessing.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when
they drive decisions:

1. [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
2. [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
3. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
4. [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
5. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

## Major components and dependencies

1. **canonical transport authority**
   - define one shared code-owned authority for:
     - canonical guest socket path,
     - managed host UDS path,
     - retained compatibility TCP port if one remains,
     - transport-kind descriptions and endpoint derivation
2. **backend transport drift removal**
   - make `world-mac-lima` consume the shared authority instead of ad hoc
     literals
   - remove stale `7788` probing from `lib.rs`
3. **minimal shell-facing alignment**
   - ensure shell transport mapping no longer contradicts the backend on the
     same contract
   - stop after the minimum alignment needed for Slice `03`; leave broader
     doctor/readiness/routed-consumer convergence to Slice `04`
4. **validation and handoff clarity**
   - prove the changed surfaces are consistent
   - leave Slice `04` with a stable transport authority to reuse

Dependency order:

1. source-backed contract freeze first
2. centralized transport authority second
3. backend adoption and stale-constant removal third
4. shell-facing alignment fourth
5. tests and Slice `04` handoff last

## Locked decisions

### What this slice changes

1. It creates one code-owned transport authority for the macOS Lima contract.
2. It removes stale `7788` drift from the backend-owned transport surfaces.
3. It aligns the minimum shell-facing transport mapping needed so the shell and
   backend no longer contradict each other.
4. It records whether any retained `17788` compatibility path is still
   necessary and, if so, gives it one explicit meaning.

### What this slice does not change

1. no doctor/readiness convergence work
2. no broad PTY/non-PTY routed-consumer convergence beyond the minimum shell
   mapping needed to stop direct contradiction
3. no repo-wide docs cutover
4. no backend policy parity work
5. no mount, ingress, listener, or guest-unit hardening work

## Implementation order

### Packet 1: Freeze and centralize the transport authority

Goal:

1. confirm the live authority stack and official Lima source set,
2. decide the single source of truth for canonical guest socket path, host UDS
   path, and retained compatibility TCP port,
3. centralize those facts in `world-mac-lima`.

Primary touch surface:

1. `SPEC-03-canonical-guest-endpoint-and-transport-contract.md`
2. `crates/world-mac-lima/src/transport.rs`
3. `crates/world-mac-lima/src/forwarding.rs`

Why first:

1. every downstream code path depends on these constants and endpoint shapes,
2. Slice `04` is blocked until the authority exists.

Verification checkpoint:

1. official Lima port-forwarding, SSH, `limactl shell`, VZ, and breaking-change
   semantics are cited for non-obvious transport decisions,
2. `transport.rs` no longer advertises a stale `7788` contract,
3. the slice still has not absorbed doctor/readiness or top-level docs work.

### Packet 2: Remove backend drift and perform minimal shell-facing alignment

Goal:

1. make `world-mac-lima` backend call sites consume the shared authority,
2. eliminate stale backend probing and contradictory endpoint strings,
3. update minimal shell-facing transport mapping so the shell stops disagreeing
   with the backend on the same contract.

Primary touch surface:

1. `crates/world-mac-lima/src/lib.rs`
2. `crates/world-mac-lima/src/forwarding.rs`
3. `crates/shell/src/execution/platform_world/mod.rs`
4. `crates/shell/src/builtins/world_gateway.rs` only if required

Why second:

1. backend drift must be removed before consumer convergence can be honest,
2. shell-facing contradictions should be collapsed now only where they directly
   restate the same contract.

Verification checkpoint:

1. the backend no longer probes `7788`,
2. shell transport mapping and backend transport authority agree on any
   retained compatibility TCP port,
3. no code path touched by Slice `03` describes SSH TCP fallback as the
   automatic default,
4. `crates/shell/src/execution/platform/macos.rs` and broader routed consumer
   convergence remain untouched unless explicitly approved.

### Packet 3: Targeted validation and clean handoff to Slice `04`

Goal:

1. run targeted tests and drift checks,
2. run GitNexus change-scope verification before commits,
3. leave the next seam clearly as PTY/non-PTY/doctor/readiness convergence.

Primary touch surface:

1. Slice `03` docs
2. targeted tests only as required by the implemented transport changes

Why third:

1. the handoff is only honest once the code changes and tests are stable,
2. this packet prevents Slice `03` from quietly broadening into Slice `04`.

Verification checkpoint:

1. targeted tests pass in `world-mac-lima` and affected shell transport tests,
2. stale `7788` drift is gone from Slice `03`-owned surfaces,
3. GitNexus reports only the expected symbol/process impact,
4. Slice `04` remains the next honest transport-consumer seam.

## Risks and mitigations

### Risk 1: Slice `03` absorbs Slice `04`

Mitigation:

1. stop after transport-authority centralization plus minimal shell alignment,
2. treat `crates/shell/src/execution/platform/macos.rs`,
   `world_ops.rs`, and `world_persistent_session.rs` as deferred surfaces by
   default.

### Risk 2: The slice bakes in stale or host-specific SSH assumptions

Mitigation:

1. require current official Lima SSH and `limactl shell` docs for any claim
   about host SSH connectivity,
2. avoid encoding hidden hard-coded SSH-port assumptions because current Lima
   docs and breaking changes show those are version-sensitive.

### Risk 3: The slice keeps TCP compatibility paths without clear meaning

Mitigation:

1. allow retained host TCP only with one explicit constant and one explicit
   compatibility reason,
2. ensure the default forwarding story still centers on the canonical guest UDS
   endpoint.

### Risk 4: The slice silently widens into docs cutover

Mitigation:

1. keep top-level docs and shell doctor UX out of the default landing boundary,
2. treat any wider doc edits as separately approved contradiction cleanup.

## Parallelism guidance

This slice is only lightly parallelizable.

Parallelizable work:

1. reviewing official Lima source pages,
2. inventorying current transport constants and endpoint strings,
3. identifying the exact symbols that need GitNexus impact analysis

Sequential work:

1. centralizing the transport authority,
2. removing backend drift,
3. applying minimal shell-facing alignment,
4. running targeted validation and GitNexus detect-changes

## Exit criteria

This plan is ready to hand off to `TASKS-03` only when:

1. the slice remains bounded to transport-authority centralization and direct
   contradiction removal,
2. official Lima sources are explicitly named for the transport decisions that
   matter,
3. the code touch set is small enough to avoid silently absorbing Slice `04`,
4. GitNexus impact/detect-changes gates are part of the execution contract,
5. the next seam after implementation is still Slice `04`.
