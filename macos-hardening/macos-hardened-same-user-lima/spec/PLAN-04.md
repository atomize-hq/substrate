# PLAN-04: PTY, Non-PTY, Doctor, and Readiness Transport Convergence

Source spec:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](./SPEC-03-canonical-guest-endpoint-and-transport-contract.md)
- [`PLAN-03.md`](./PLAN-03.md)
- [`TASKS-03.md`](./TASKS-03.md)

Plan type: source-driven phase-1 shell-runtime consumer-convergence slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `04`: converge the shell-side macOS PTY,
persistent-session, doctor, and readiness consumers onto the Slice `03`
transport contract.

This plan should produce a bounded landing that:

1. defines one shared shell-side consumer behavior for `WorldTransport`,
2. removes transport-ladder duplication from PTY and persistent-session
   consumers first,
3. makes macOS doctor/readiness runtime code prove the selected transport
   before guest-direct fallback,
4. leaves backend policy semantics to Slice `05`,
5. leaves script/docs cutover and happy-path operator storytelling to Slice
   `06`.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-04.md`
4. `crates/shell/src/execution/platform_world/mod.rs`
5. `crates/shell/src/execution/platform/macos.rs`
6. `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
7. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
8. targeted tests in the same files or existing shell test coverage as needed

By default this slice should **not** widen into:

1. `crates/world-mac-lima/` contract changes beyond tiny helper exports needed
   to consume the Slice `03` authority,
2. `crates/shell/src/builtins/world_gateway.rs` logic changes except where a
   direct endpoint-regression contradiction requires a tiny alignment fix,
3. `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`,
   `docs/WORLD.md`, `docs/reference/world/platforms/macos-lima-setup.md`, or
   `docs/USAGE.md`,
4. backend policy propagation or world-backend API widening.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus current official Lima docs as co-equal authority,
2. preserve the Slice `03` transport contract while changing only the shell
   consumers that use or prove that contract,
3. treat any need to alter backend policy or docs cutover as an explicit
   out-of-scope pressure that should defer to Slice `05` or Slice `06`.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when they
drive decisions:

1. [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
2. [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
3. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
4. [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
5. [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
6. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

## Major components and dependencies

1. **shared shell-side transport helper behavior**
   - freeze `crates/shell/src/execution/platform_world/mod.rs` as the default
     shell-side authority layer for turning the selected `WorldTransport` into
     shared WebSocket, agent-client, and readiness-facing connection behavior
   - keep request payload building and doctor-policy interpretation outside that
     helper boundary unless a direct contradiction forces a tiny adjacent export
2. **persistent-session and PTY convergence**
   - make persistent-session and PTY code reuse that shared behavior instead of
     maintaining parallel ladders
3. **doctor/readiness runtime convergence**
   - make macOS doctor/readiness runtime code prove the selected transport
     contract first and classify guest-direct probing as fallback only
4. **validation and handoff clarity**
   - prove the changed consumers are aligned
   - leave Slice `05` and Slice `06` with a stable shell-side transport proof
     story to build on

Dependency order:

1. source-backed consumer contract freeze first,
2. Packet `1` symbol-impact inventory and helper-boundary freeze second,
3. shared helper extraction or reuse third,
4. PTY and persistent-session convergence fourth,
5. doctor/readiness runtime convergence fifth,
6. tests and clear handoff last.

## Locked decisions

### What this slice changes

1. It creates or clarifies one shell-side helper/behavior layer for selected
   transport use.
2. It removes direct transport-ladder duplication from the affected PTY and
   persistent-session consumers.
3. It makes doctor/readiness runtime code attempt the selected transport first.
4. It keeps override and compatibility behavior explicit rather than implicit.

### What this slice does not change

1. no backend policy parity work,
2. no shell request payload or broker policy redesign,
3. no helper script or top-level docs cutover,
4. no ingress, mount, listener, or guest-unit hardening work,
5. no reopening of Slice `03` transport constants unless a tiny export surface
   is needed for reuse.

## Implementation order

### Packet 1: Freeze the shell-side consumer contract and helper boundary

Goal:

1. confirm the authority stack and official Lima source set,
2. decide the single shell-side helper boundary for transport consumers,
3. inventory which PTY, persistent-session, and doctor/readiness paths still
   duplicate transport behavior.

Primary touch surface:

1. `SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`
2. `PLAN-04.md`
3. possibly a tiny adjacent shell helper surface only if needed

Why first:

1. the slice should not converge consumers one by one without first freezing
   what “same transport behavior” means,
2. Slice `05` and Slice `06` both depend on this consumer contract being stable.

Verification checkpoint:

1. official Lima forwarding, SSH, `limactl shell`, VZ, environment-variable,
   and breaking-change semantics are cited for non-obvious decisions,
2. the shared helper or consumer boundary is obvious,
3. the exact Packet `2` / Packet `3` consumer symbols are named explicitly:
   - `platform_world::detect`
   - `world_persistent_session::build_ws_and_start_session_frame`
   - `world_ops::build_agent_client_and_request_impl`
   - `world_ops::build_agent_client_and_member_dispatch_request_impl`
   - `world_ops::build_agent_client_and_pending_diff_request_impl`
   - `world_ops::execute_world_pty_over_ws_macos`
   - `platform/macos.rs::collect_world_doctor_assessment`
4. GitNexus commands are pinned to
   `GITNEXUS_HOME=/tmp/gitnexus-ff74-only` so this checkout does not resolve
   against sibling `substrate` indexes,
5. the slice still has not absorbed policy parity or docs/script cutover.

### Packet 2: Converge PTY and persistent-session consumers

Goal:

1. make PTY and persistent-session bootstrap reuse the same selected transport
   behavior,
2. remove duplicated per-transport WebSocket setup where possible,
3. keep request payload and policy content unchanged.

Primary touch surface:

1. `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
3. `crates/shell/src/execution/platform_world/mod.rs` if helper wiring is
   needed

Why second:

1. these are the most direct runtime consumers of the selected transport,
2. converging them first lowers the risk that doctor/readiness logic freezes the
   wrong consumer story.

Verification checkpoint:

1. persistent-session override behavior still passes,
2. PTY transport connection still works against the same selected transport
   semantics,
3. no policy-routing or member-dispatch semantics were silently widened.

### Packet 3: Converge macOS doctor/readiness runtime behavior

Goal:

1. make macOS doctor/readiness runtime code prove the selected routed transport
   first,
2. retain guest-direct probing only as explicit fallback or breakglass
   material,
3. avoid widening into scripts/docs cutover.

Primary touch surface:

1. `crates/shell/src/execution/platform/macos.rs`
2. any shared helper surface from Packet `1`

Why third:

1. doctor/readiness is the runtime evidence layer most likely to drift into
   Slice `06`,
2. it should be updated only after the direct transport consumers are aligned.

Verification checkpoint:

1. doctor JSON tests still pass,
2. selected-transport-first behavior is visible in code structure,
3. guest-direct probing remains clearly fallback-only where it still exists,
4. helper scripts and top-level docs remain untouched.

### Packet 4: Targeted validation and clean handoff to Slice `05` / `06`

Goal:

1. run targeted tests and drift checks,
2. run GitNexus change-scope verification before commits,
3. leave the next seams clearly as backend policy parity and later docs/script
   cutover.

Primary touch surface:

1. Slice `04` docs,
2. targeted tests only as required by the consumer convergence work.

Why fourth:

1. the slice is only honest once the consumer behavior and tests are stable,
2. this packet prevents Slice `04` from quietly absorbing Slice `05` or Slice
   `06`.

Verification checkpoint:

1. targeted shell tests pass,
2. touched consumers no longer scatter their own transport ladders,
3. GitNexus reports only the expected symbol/process impact,
4. Slice `05` remains the next honest seam,
5. Slice `06` remains the script/docs cutover seam.

## Risks and mitigations

### Risk 1: Slice `04` absorbs Slice `05`

Mitigation:

1. keep request payload construction and policy semantics unchanged,
2. treat any backend or policy-carrier widening as explicit out-of-scope
   pressure.

### Risk 2: Slice `04` absorbs Slice `06`

Mitigation:

1. keep helper scripts and top-level docs out of the default landing boundary,
2. stop at runtime consumer convergence rather than operator-story rewrites.

### Risk 3: PTY and persistent-session converge differently anyway

Mitigation:

1. prefer one reusable transport connection helper,
2. review PTY and persistent-session diffs together as one packet.

### Risk 4: doctor/readiness still bakes in stale forwarding assumptions

Mitigation:

1. require current official Lima docs for forwarding, SSH, shell, and VZ
   semantics,
2. avoid any hidden fixed SSH-port or pre-v2 forwarding assumptions.

## Parallelism guidance

This slice is only lightly parallelizable.

Parallelizable work:

1. reviewing the official Lima source pages,
2. inventorying consumer-side transport duplication,
3. identifying the exact symbols that need GitNexus impact analysis,
4. recording `gitnexus status`, `gitnexus context`, and `gitnexus impact`
   results for the full Packet `1` symbol set
   (`platform_world::detect`,
   `world_persistent_session::build_ws_and_start_session_frame`,
   `world_ops::build_agent_client_and_request_impl`,
   `world_ops::build_agent_client_and_member_dispatch_request_impl`,
   `world_ops::build_agent_client_and_pending_diff_request_impl`,
   `world_ops::execute_world_pty_over_ws_macos`, and
   `platform/macos.rs::collect_world_doctor_assessment`) with
   `GITNEXUS_HOME=/tmp/gitnexus-ff74-only`.

Sequential work:

1. freezing the helper boundary,
2. converging PTY and persistent-session consumers,
3. converging doctor/readiness runtime behavior,
4. running targeted validation and GitNexus detect-changes.

## Exit criteria

This plan is ready to hand off to `TASKS-04` only when:

1. the slice remains bounded to shell-side transport-consumer convergence,
2. official Lima sources are explicitly named for the semantics that matter,
3. the code touch set is small enough to avoid silently absorbing Slice `05`
   or Slice `06`,
4. GitNexus impact/detect-changes gates are part of the execution contract,
5. the next seam after implementation is still Slice `05`.
