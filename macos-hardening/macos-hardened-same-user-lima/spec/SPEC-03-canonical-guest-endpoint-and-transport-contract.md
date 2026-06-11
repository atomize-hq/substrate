# Spec: Slice 03 Canonical Guest Endpoint and Transport Contract

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-02-lima-version-floor-and-breakglass-contract.md`](./SPEC-02-lima-version-floor-and-breakglass-contract.md)
- [`PLAN-02.md`](./PLAN-02.md)
- [`TASKS-02.md`](./TASKS-02.md)

Required official source set for this slice:
- [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Phase: `SPECIFY`  
Status: draft slice authority  
Slice focus: freeze one code-owned canonical transport contract for
`macos-hardened-same-user-lima` so the repo stops carrying contradictory port,
socket, and adapter stories, while still keeping PTY/non-PTY/doctor/readiness
consumer convergence in Slice `04`.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `02` is landed, so the support taxonomy and breakglass framing are
   already frozen and should be reused rather than reopened here.
2. Per [`../ROADMAP.md`](../ROADMAP.md), the next honest seam after Slice `02`
   is Slice `03`: canonical guest endpoint and transport contract.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   transport claims depend on current official Lima forwarding and SSH
   semantics.
4. Milestone `1.1` is broader than one slice, so Slice `03` must stop after
   transport-contract centralization and explicit stale-constant cleanup rather
   than silently absorb the full PTY/non-PTY/doctor/readiness convergence owned
   by Slice `04`.
5. Live repo truth still contains real transport drift that this slice must
   resolve:
   - `crates/world-mac-lima/src/transport.rs` still advertises
     `127.0.0.1:7788`,
   - `crates/world-mac-lima/src/forwarding.rs` actually uses host TCP `17788`
     for VSock and SSH TCP variants,
   - `crates/world-mac-lima/src/lib.rs` still probes stale `7788`,
   - shell-side macOS transport mapping and gateway fallback already prefer
     host UDS and otherwise use `17788`.
6. This slice may touch targeted runtime code and tests, but should not widen
   into top-level operator docs, `scripts/mac/lima-doctor.sh`,
   `crates/shell/src/execution/platform/macos.rs`, or repo-wide cutover wording
   unless a direct contradiction forces it and the expansion is called out
   explicitly.

If any of these are wrong, correct them before implementation.

## Objective

Freeze and implement one canonical macOS Lima transport contract that later
Phase `1` slices can reuse without guessing.

This slice is complete only when a reviewer can answer, without guessing:

1. what the canonical guest service endpoint is,
2. which host-visible adapter surfaces are part of the supported
   Substrate-owned path,
3. where the single source of truth for guest socket paths, host UDS paths,
   and any retained compatibility TCP port lives in code,
4. whether SSH TCP fallback is part of automatic forwarding, a temporary
   compatibility path, or unsupported for default forwarding,
5. which remaining consumer-parity work is still deferred to Slice `04`.

## Frozen in this slice

This slice freezes only:

1. the canonical guest service endpoint for macOS Lima as
   `/run/substrate.sock`,
2. one code-owned transport authority for:
   - the canonical guest socket path,
   - the managed host UDS path,
   - the retained compatibility TCP port if one still exists,
   - transport-kind metadata and descriptions,
3. the elimination of stale `7788` drift from the transport authority and the
   backend surfaces that directly consume it,
4. the contract that SSH TCP fallback is not part of automatic default
   forwarding if the guest service remains UDS-only,
5. the minimum shell-facing transport mapping updates required so the shell and
   `world-mac-lima` no longer disagree on the same contract.

## Deferred by design

This slice intentionally does **not** freeze:

1. PTY/non-PTY/readiness/doctor transport convergence owned by Slice `04`,
2. backend policy input parity owned by Slice `05`,
3. routed-path-first doctor/smoke/readiness truth owned by Slice `06`,
4. listener removal, ingress narrowing, or mount minimization owned by later
   Phase `2` slices,
5. repo-wide operator-doc rewrites and breakglass cutover owned by Slices `11`
   and `12`,
6. daemon or ownership-separation work outside the same-user Lima model.

## Why this slice exists

Slice `02` deliberately froze only the minimum transport assumptions needed for
support-versus-breakglass classification. It left the actual transport contract
for Slice `03`.

Live repo truth now shows why that separation is necessary:

1. `crates/world-mac-lima/src/transport.rs` still models a stale TCP endpoint
   (`127.0.0.1:7788`) that no longer matches the active forwarding code.
2. `crates/world-mac-lima/src/forwarding.rs` uses `/run/substrate.sock` as the
   guest target and uses host TCP `17788` for VSock and SSH TCP variants, while
   intentionally skipping automatic SSH TCP fallback because the guest agent is
   UDS-only.
3. `crates/world-mac-lima/src/lib.rs` still probes `7788` in
   `test_agent_connection`, which means backend readiness proof can disagree
   with the forwarding layer.
4. `crates/shell/src/execution/platform_world/mod.rs` and
   `crates/shell/src/builtins/world_gateway.rs` already treat `17788` as the
   macOS TCP compatibility path, so the shell and backend are not even drifting
   in the same direction.
5. `docs/WORLD.md` still documents a broader adapter/fallback story, but that
   docs-level reconciliation should remain deferred until the runtime contract
   stops contradicting itself.

Without an explicit Slice `03`, Slice `04` would be forced to converge PTY,
doctor, and readiness consumers on top of transport primitives that still
disagree on their own constants and endpoint descriptions.

## Official-source grounding that must drive this slice

The official Lima docs currently establish transport semantics this slice must
respect:

1. the port-forwarding docs say Lima supports SSH and GRPC port forwarders and
   that the default forwarding mode changed across releases,
2. those same docs say SSH-over-AF_VSOCK requires Lima `>= 2.0`, a VZ-based
   VM, and guest `systemd` `v256+`,
3. the `limactl shell` docs describe host-to-guest shell access as SSH-based
   and note that Lima uses the first `ssh` executable in the host `PATH` by
   default,
4. the SSH docs show Lima exposes generated SSH config and an instance-specific
   SSH local port, so the transport contract should not depend on a hidden
   hard-coded SSH port,
5. the breaking-changes docs record that Lima `v2.0.0` removed the old
   hard-coded default-instance SSH port assumption and that `v1.0.0` changed
   default VZ-related behavior,
6. the VZ docs remain relevant because AF_VSOCK transport assumptions apply
   only to VZ guests.

This slice should cite the exact official URLs above when those semantics drive
its final decisions.

## Commands

This is a source-driven runtime-contract slice. Required commands should prove
repo truth, official-source truth, and the exact implementation boundary.

```bash
# Review the phase-1 milestone authority
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md

# Review the execution rubric, roadmap, and design inputs
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/ROADMAP.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-lima-transport-contract.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md

# Inspect current transport drift in repo truth
sed -n '1,220p' crates/world-mac-lima/src/transport.rs
sed -n '1,320p' crates/world-mac-lima/src/forwarding.rs
sed -n '180,260p' crates/world-mac-lima/src/lib.rs
sed -n '140,180p' crates/shell/src/execution/platform_world/mod.rs
sed -n '198,255p' crates/shell/src/builtins/world_gateway.rs

# Inspect deferred consumer surfaces so Slice 03 does not absorb Slice 04
sed -n '340,460p' crates/shell/src/execution/platform/macos.rs
sed -n '560,740p' crates/shell/src/execution/routing/dispatch/world_ops.rs
sed -n '680,760p' crates/shell/src/execution/routing/dispatch/world_persistent_session.rs

# Inventory transport constants and endpoint strings
rg -n "7788|17788|/run/substrate.sock|agent.sock|vsock|SshTcp|UnixSocket|SUBSTRATE_WORLD_SOCKET" \
  crates/world-mac-lima/src \
  crates/shell/src/execution/platform_world/mod.rs \
  crates/shell/src/builtins/world_gateway.rs \
  crates/shell/src/execution/platform/macos.rs \
  crates/shell/src/execution/routing/dispatch/world_ops.rs \
  crates/shell/src/execution/routing/dispatch/world_persistent_session.rs

# Targeted validation for the implementation slice
cargo test -p world-mac-lima -- --nocapture
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
├── ROADMAP.md                                  → slice map authority
├── EXECUTION-RUBRIC.md                         → skill and source gate authority
├── phase-1-runtime-parity-foundation/
│   ├── README.md                               → phase-1 sequencing authority
│   └── milestone-1-1-transport-contract-unification-sow.md
└── spec/
    ├── SPEC-03-canonical-guest-endpoint-and-transport-contract.md
    ├── PLAN-03.md
    ├── TASKS-03.md
    └── design/
        ├── DESIGN-macos-lima-transport-contract.md
        └── DESIGN-supported-mode-and-breakglass-taxonomy.md

crates/
├── world-mac-lima/src/transport.rs             → current transport authority drift
├── world-mac-lima/src/forwarding.rs            → actual forwarding kinds and ports
├── world-mac-lima/src/lib.rs                   → backend readiness/connection drift
├── shell/src/execution/platform_world/mod.rs   → shell transport mapping for macOS
└── shell/src/builtins/world_gateway.rs         → gateway endpoint resolution

Deferred consumer surfaces for Slice 04:
├── shell/src/execution/platform/macos.rs
├── shell/src/execution/routing/dispatch/world_ops.rs
└── shell/src/execution/routing/dispatch/world_persistent_session.rs
```

## Code style

This slice should centralize transport facts instead of restating them in
multiple call sites.

Required style:

1. one transport authority module or type for shared constants and endpoint
   descriptions,
2. no new magic port literals in downstream call sites,
3. adapter terminology that treats VSock, host UDS, and retained TCP as
   reachability mechanisms to the same guest endpoint,
4. explicit comments when a compatibility path remains temporarily,
5. repo-standard `Result<T, anyhow::Error>` and contextual errors for runtime
   changes.

Example shape:

```rust
pub const GUEST_WORLD_SOCKET_PATH: &str = "/run/substrate.sock";
pub const MACOS_HOST_COMPAT_PORT: u16 = 17788;

pub enum TransportEndpoint {
    Unix(std::path::PathBuf),
    Tcp { host: String, port: u16 },
}
```

The exact type names may differ, but the slice should leave one obvious place
where later consumers can reuse the same contract.

## Testing strategy

This slice is a targeted runtime-contract slice. Validation should emphasize
constant unification, endpoint derivation, and bounded downstream adoption.

Validation levels:

1. **Official-source review**
   - confirm the forwarding and SSH claims are grounded in current official
     Lima docs
2. **Transport-authority regression tests**
   - confirm endpoint derivation and forwarding-kind tests still pass in
     `world-mac-lima`
3. **Shell transport-mapping tests**
   - confirm macOS gateway endpoint fallback and world-transport formatting stay
     aligned with the centralized contract
4. **Drift search**
   - confirm stale `7788` references are gone from the surfaces Slice `03`
     owns
5. **Scope validation**
   - confirm doctor/readiness code and top-level docs were not silently
     absorbed into this slice

## Boundaries

- Always:
  - reuse the Slice `01` and Slice `02` support taxonomy and breakglass
    framing exactly as landed
  - keep `/run/substrate.sock` as the canonical guest endpoint
  - ground transport-contract decisions in current official Lima docs
  - centralize transport constants before trying to converge every consumer
  - run GitNexus impact analysis before editing touched symbols and
    `gitnexus_detect_changes()` before committing
- Ask first:
  - widening into `crates/shell/src/execution/platform/macos.rs`,
    `scripts/mac/lima-doctor.sh`, or top-level macOS docs
  - removing the retained `17788` compatibility path entirely if live repo
    truth shows more routed consumers still depend on it
  - changing breakglass classifications already frozen in Slice `02`
- Never:
  - reintroduce `7788` as a supported or compatibility endpoint
  - describe SSH TCP fallback as the automatic default when the guest service is
    still UDS-only
  - silently absorb Slice `04` consumer convergence work
  - silently widen into Phase `2` hardening or Phase `3` docs cutover

## Success criteria

Slice `03` is successful when:

1. there is one code-owned macOS transport authority for the canonical guest
   endpoint and host-side adapter metadata,
2. `world-mac-lima` no longer carries stale `7788` transport drift,
3. shell-side macOS transport mapping no longer contradicts the backend on the
   same port/socket contract,
4. any retained `17788` compatibility path has one explicit meaning and one
   shared constant instead of scattered literals,
5. automatic SSH TCP fallback is not presented as part of the default
   forwarding contract,
6. Slice `04` remains the clear next seam for PTY/non-PTY/doctor/readiness
   convergence.

## Remaining open questions for later slices

After Slice `03`, the follow-on questions should be narrower and more honest:

1. Which PTY, non-PTY, doctor, readiness, and persistent-session consumers in
   the shell should be switched to the shared transport authority first
   (Slice `04`)?
2. Which compatibility probes can be removed only after those consumers are
   converged (Slices `04` and `06`)?
3. When should top-level macOS operator docs stop describing direct guest or
   compatibility transport paths as routine operator behavior (Slices `11` and
   `12`)?
