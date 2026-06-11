# Spec: Slice 04 PTY, Non-PTY, Doctor, and Readiness Transport Convergence

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](./SPEC-03-canonical-guest-endpoint-and-transport-contract.md)
- [`PLAN-03.md`](./PLAN-03.md)
- [`TASKS-03.md`](./TASKS-03.md)
- [`design/DESIGN-macos-policy-input-parity.md`](./design/DESIGN-macos-policy-input-parity.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Required official source set for this slice:
- [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
- [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: make the macOS shell-side PTY, persistent-session, non-PTY-adjacent,
doctor, and readiness consumers all reuse the Slice `03` transport contract
instead of restating their own socket, TCP, or guest-probe ladders.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `03`, and its spec/plan/tasks
   explicitly leave PTY/non-PTY/doctor/readiness consumer convergence to Slice
   `04`.
2. Slice `03` either already landed or is treated as the frozen contract for:
   - guest socket `/run/substrate.sock`,
   - managed host UDS path,
   - retained compatibility TCP meaning, if one still exists,
   - non-default classification of SSH TCP fallback.
3. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `04` is still the next honest
   seam after Slice `03`, while Slice `05` remains backend policy input parity
   and Slice `06` remains routed-path-first doctor/smoke/readiness truth.
4. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because it
   still depends on current official Lima forwarding, SSH, `limactl shell`,
   and VZ semantics.
5. Live repo truth still contains shell-side transport-consumer duplication that
   this slice must resolve:
   - `crates/shell/src/execution/platform/macos.rs` still probes the managed
     host socket path directly, then host TCP `17788`, then falls back to
     in-guest `limactl shell` + `curl`,
   - `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
     still hand-builds per-transport WebSocket connection paths after platform
     readiness,
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs` still has its
     own PTY WebSocket connection ladder,
   - these consumers do not yet read as one obvious shared shell-side transport
     behavior, even when they already consult `PlatformWorldContext`.
6. This slice may touch shell runtime code and tests, but should not widen into
   backend policy semantics, top-level docs, helper scripts, or repo-wide
   operator wording unless a direct contradiction forces it and the expansion is
   called out explicitly.

If any of these are wrong, correct them before implementation.

## Objective

Make the shell-side macOS consumers that prove or use routed world reachability
behave like one transport system.

This slice is complete only when a reviewer can answer, without guessing:

1. how PTY and persistent-session bootstrap obtain the selected macOS transport,
2. whether doctor/readiness probes follow the same selected transport contract
   before any breakglass guest introspection,
3. where the shell-side helper or authority lives for converting the selected
   transport into actual agent, WebSocket, or readiness connections,
4. whether `SUBSTRATE_WORLD_SOCKET` override behavior remains exceptional but
   consistent across shell-side consumers,
5. which remaining work is still deferred to Slice `05` and Slice `06`.

## Frozen in this slice

This slice freezes only:

1. one shell-side transport-consumer contract for macOS that reuses the Slice
   `03` transport authority,
2. the rule that PTY, persistent-session, and doctor/readiness runtime code
   should not restate separate endpoint ladders when the selected transport is
   already known,
3. the rule that routed readiness checks must try the selected shell-visible
   transport before any guest-direct `limactl shell` fallback,
4. the minimum helper extraction or API shaping needed so the affected shell
   consumers share the same transport-selection behavior,
5. the classification that `SUBSTRATE_WORLD_SOCKET` override behavior remains an
   advanced/test/breakglass bypass on macOS even when the consuming code is
   converged.

## Deferred by design

This slice intentionally does **not** freeze:

1. backend policy input parity owned by Slice `05`,
2. script/docs cutover and routed-path-first readiness storytelling owned by
   Slice `06`,
3. gateway lifecycle contract consolidation owned by later Phase `3` slices,
4. ingress, mount, listener, or guest-unit hardening work from later phases,
5. repo-wide breakglass reclassification or docs cutover work.

## Why this slice exists

Slice `03` centralizes the transport contract, but it intentionally stops short
of making every shell-side consumer use that contract the same way.

Live repo truth shows the remaining gap clearly:

1. `crates/shell/src/execution/platform_world/mod.rs` already detects and stores
   a selected `WorldTransport`, but downstream consumers still partially expand
   that transport into their own ladders.
2. `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
   uses the selected transport, but still hand-builds transport-specific
   WebSocket setup logic after readiness.
3. `crates/shell/src/execution/routing/dispatch/world_ops.rs` does the same for
   PTY execution, which risks PTY and persistent-session transport behavior
   drifting independently.
4. `crates/shell/src/execution/platform/macos.rs` still performs doctor and
   readiness proof by probing `~/.substrate/sock/agent.sock`, then host TCP
   `17788`, then direct guest `curl` through `limactl shell`, which means the
   runtime evidence path can diverge from the selected transport contract.
5. Slice `03` deliberately left top-level docs and helper scripts alone, so if
   Slice `04` does not converge these runtime consumers first, Slice `06` would
   be forced to rewrite readiness docs around behavior that is still internally
   inconsistent.

## Official-source grounding that must drive this slice

The current official Lima docs still matter for this consumer-convergence work:

1. the port-forwarding docs say Lima supports SSH and GRPC port forwarders and
   document the current default history, so the shell should not treat an older
   forwarding assumption as timeless,
2. those docs also say SSH over AF_VSOCK requires Lima `>= 2.0`, a VZ guest,
   and guest `systemd` `v256+`, so VSock-related reachability claims remain
   version-sensitive,
3. the SSH docs show Lima exposes generated SSH config and an instance-specific
   `SSHLocalPort`, so the shell should not invent a hidden hard-coded SSH port
   contract,
4. the `limactl shell` docs say the first host `ssh` in `PATH` is used by
   default and that shell access is SSH-based, so guest-direct fallback remains
   an externally mediated path rather than a magic local primitive,
5. the environment-variables docs describe current `LIMA_SSH_PORT_FORWARDER`
   and `LIMA_SSH_OVER_VSOCK` semantics, including the deprecation of the latter
   in favor of `.ssh.overVsock`,
6. the breaking-changes docs record that Lima `v2.0.0` removed the old fixed
   default-instance SSH port assumption,
7. the VZ docs remain relevant because the repo’s same-user Lima posture still
   assumes VZ-backed macOS guests.

This slice should cite the exact official URLs above when those semantics drive
its final decisions.

## Commands

This is a source-driven shell-runtime convergence slice. Required commands
should prove live repo truth, official-source truth, and the exact execution
boundary.

```bash
# Review the phase-1 authority stack
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md

# Review the execution rubric, roadmap, prior slice, and design inputs
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/ROADMAP.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/PLAN-03.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/TASKS-03.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-lima-transport-contract.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-policy-input-parity.md

# Inspect the deferred consumer surfaces from Slice 03
sed -n '1,320p' crates/shell/src/execution/platform_world/mod.rs
sed -n '300,470p' crates/shell/src/execution/platform/macos.rs
sed -n '740,840p' crates/shell/src/execution/routing/dispatch/world_persistent_session.rs
sed -n '920,980p' crates/shell/src/execution/routing/dispatch/world_ops.rs
sed -n '1290,1565p' crates/shell/src/execution/routing/dispatch/world_ops.rs

# Inventory remaining shell-side transport duplication
rg -n "17788|7788|agent.sock|SUBSTRATE_WORLD_SOCKET|doctor|readiness|WorldTransport|/v1/stream" \
  crates/shell/src/execution/platform_world/mod.rs \
  crates/shell/src/execution/platform/macos.rs \
  crates/shell/src/execution/routing/dispatch/world_persistent_session.rs \
  crates/shell/src/execution/routing/dispatch/world_ops.rs \
  crates/shell/src/builtins/world_gateway.rs

# Targeted validation for the implementation slice
cargo test -p shell macos_no_override_current_thread_start_uses_async_readiness_without_panic -- --nocapture
cargo test -p shell macos_socket_override_bypasses_platform_async_readiness -- --nocapture
cargo test -p shell doctor_ok_json -- --nocapture
cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture
cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture
cargo test -p shell macos_gateway_client_endpoint -- --nocapture
cargo test -p shell unix_and_tcp_transports_format_endpoints -- --nocapture
cargo fmt --all -- --check
```

If GitNexus reports a stale index before symbol-impact work begins, refresh it
first:

```bash
npx gitnexus analyze
```

## Project structure

This slice should stay grounded to these directories and file families:

```text
macos-hardening/macos-hardened-same-user-lima/
├── ROADMAP.md
├── EXECUTION-RUBRIC.md
├── phase-1-runtime-parity-foundation/
│   ├── README.md
│   ├── milestone-1-1-transport-contract-unification-sow.md
│   ├── milestone-1-2-policy-application-parity-sow.md
│   └── milestone-1-3-doctor-smoke-readiness-parity-sow.md
└── spec/
    ├── SPEC-03-canonical-guest-endpoint-and-transport-contract.md
    ├── SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md
    ├── PLAN-04.md
    ├── TASKS-04.md
    └── design/
        ├── DESIGN-macos-lima-transport-contract.md
        ├── DESIGN-supported-mode-and-breakglass-taxonomy.md
        └── DESIGN-macos-policy-input-parity.md

crates/shell/
├── src/execution/platform_world/mod.rs                 → selected transport and shell-facing context
├── src/execution/platform/macos.rs                     → macOS doctor/readiness runtime ladder
├── src/execution/routing/dispatch/world_persistent_session.rs
│                                                       → persistent-session bootstrap transport use
├── src/execution/routing/dispatch/world_ops.rs         → PTY and non-PTY-adjacent transport consumers
└── src/builtins/world_gateway.rs                       → endpoint-regression reference and tests only by default
```

## Code style

This slice should centralize consumer-side transport behavior instead of letting
multiple shell paths perform transport-specific branching independently.

Required style:

1. prefer one shell-side helper layer for connecting through `WorldTransport`
   rather than repeating per-transport WebSocket or agent-client setup,
2. no new magic literals for `17788`, host UDS paths, or guest socket paths in
   downstream shell consumers,
3. explicit naming that distinguishes:
   - selected routed transport,
   - compatibility-only TCP behavior,
   - breakglass override behavior,
4. comments should explain why guest-direct probing is still retained where it
   remains, and label it as fallback or breakglass rather than normal routed
   proof,
5. keep policy payload construction, world-network routing, and backend policy
   semantics unchanged unless a direct transport-consumer contradiction forces a
   tiny supporting change that is called out explicitly.

Example shape:

```rust
fn connect_world_ws_via_selected_transport(
    transport: &WorldTransport,
) -> anyhow::Result<tungs::WebSocketStream<WsIo>> {
    // one transport-specific connection ladder used by PTY and persistent session
}

fn doctor_client_via_selected_transport(
    transport: &WorldTransport,
) -> anyhow::Result<transport_api_client::AgentClient> {
    // selected transport first, compatibility-only fallback only if still justified
}
```

The exact helper names may differ, but the slice should leave one obvious
shell-side place where later doctor/smoke/doc cutover work can reuse the same
consumer behavior.

## Testing strategy

This slice is a targeted shell-runtime convergence slice. Validation should
emphasize transport-consumer reuse, bounded fallback behavior, and clean
handoff to later slices.

Validation levels:

1. **Official-source review**
   - confirm the forwarding, SSH, shell, VZ, and env-var claims are grounded in
     current official Lima docs
2. **Persistent-session transport regression**
   - confirm persistent-session startup still respects async readiness and
     override behavior
3. **PTY / shell transport regression**
   - confirm shell transport formatting and gateway endpoint regression tests
     remain aligned with the selected contract
4. **Doctor/readiness regression**
   - confirm doctor JSON tests still pass while the runtime path is converged
     toward the selected transport
5. **Drift search**
   - confirm targeted shell consumers do not keep restating stale socket/TCP
     literals without going through the shared contract
6. **Scope validation**
   - confirm backend policy semantics, helper scripts, and top-level docs were
     not silently absorbed into this slice

## Boundaries

- Always:
  - reuse the Slice `01` / Slice `02` support taxonomy and Slice `03`
    transport contract exactly as landed
  - ground transport-consumer decisions in current official Lima docs
  - keep `SUBSTRATE_WORLD_SOCKET` override behavior classified as
    advanced/test/breakglass on macOS
  - converge shell-side transport consumers before attempting docs-first
    readiness cutover
  - run GitNexus impact analysis before editing touched symbols and
    `gitnexus_detect_changes()` before committing
- Ask first:
  - widening into `crates/world-mac-lima/` transport authority work beyond tiny
    helper exports needed for reuse
  - changing policy payload construction or backend policy semantics that
    belong to Slice `05`
  - widening into `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`,
    `docs/WORLD.md`, or `docs/reference/world/platforms/macos-lima-setup.md`
- Never:
  - reopen the canonical transport contract already owned by Slice `03`
  - silently absorb Slice `05` backend policy parity
  - silently absorb Slice `06` script/docs cutover
  - promote guest-direct `limactl shell` probing into the supported default
    readiness path

## Success criteria

Slice `04` is successful when:

1. PTY and persistent-session shell paths consume one obvious transport helper
   or shared behavior layer instead of duplicating transport branching,
2. macOS doctor/readiness runtime code tries the selected transport contract
   first and only uses guest-direct probing as explicit fallback material,
3. targeted shell consumers no longer scatter their own host socket / TCP
   endpoint ladders independently of the selected transport,
4. `SUBSTRATE_WORLD_SOCKET` override behavior remains consistent across the
   touched shell consumers,
5. Slice `05` remains the next honest backend-policy seam,
6. Slice `06` remains the later docs/scripts and routed-path-first readiness
   cutover seam.

## Remaining open questions for later slices

After Slice `04`, the follow-on questions should be narrower and more honest:

1. What exact contract seam should carry broker-resolved policy/world inputs
   into backend-mediated Lima execution so backend parity becomes real
   (Slice `05`)?
2. Which helper scripts and top-level docs can be rewritten to lead with routed
   doctor/gateway/smoke validation once the runtime consumers are converged
   (Slice `06`)?
3. Which compatibility-only transport probes can be removed entirely only after
   the docs/scripts and operator lifecycle surfaces have finished cutting over
   (Slices `06`, `11`, and `12`)?
