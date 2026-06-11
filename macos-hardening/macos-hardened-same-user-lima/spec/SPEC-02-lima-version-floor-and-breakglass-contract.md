# Spec: Slice 02 Lima Version Floor and Breakglass Contract

Source phase authority:
- [`../phase-0-security-contract-and-scope/README.md`](../phase-0-security-contract-and-scope/README.md)
- [`../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md`](../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)

Related research note:
- [`../research/2026-04-28-macos-lima-parity-lockdown.md`](../research/2026-04-28-macos-lima-parity-lockdown.md)

Required official source set for this slice:
- [Lima releases lifecycle](https://lima-vm.io/docs/releases/)
- [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
- [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
- [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Phase: `SPECIFY`  
Status: draft slice authority  
Slice focus: freeze the minimum supported Lima/macOS capability contract and
the breakglass boundary for the hardened same-user Lima mode without silently
absorbing the full canonical transport-unification slice.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `01` is landed, and the current Slice `02` authority stack already
   exists under `macos-hardening/macos-hardened-same-user-lima/spec/`.
2. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md),
   [`../ROADMAP.md`](../ROADMAP.md), and [`TASKS-01.md`](./TASKS-01.md), the
   next honest seam is Slice `02`: Lima version floor and breakglass contract.
3. This slice requires both `spec-driven-development` and
   `source-driven-development`, because the decision depends on current official
   Lima lifecycle, VZ, mount, forwarding, and `limactl shell` semantics.
4. The slice must replace vague “recent Lima” wording with one explicit
   lifecycle and capability floor, confirmed against the official Lima source
   set during execution.
5. The slice should freeze only the minimum transport assumptions needed to
   classify supported versus breakglass workflows. Slice `03` still owns the
   full canonical guest endpoint and transport contract.
6. This slice is still docs-and-contract work. It should not widen into runtime
   code changes, transport rewrites, mount changes, or repo-wide doc cutover.

If any of these are wrong, correct them before implementation.

## Objective

Freeze one explicit environment and breakglass contract for
`macos-hardened-same-user-lima` so later slices can stop relying on ambiguous
phrasing such as “recent Lima,” “manual guest access when needed,” or
“transport fallback as needed.”

This slice is complete only when a reviewer can answer, without guessing:

1. what minimum Lima/macOS capability contract the supported mode depends on,
2. whether the supported mode requires the currently supported Lima major line
   rather than an older lifecycle branch,
3. whether `vsock-proxy` is required, optional acceleration, or unsupported for
   the hardened default,
4. which direct guest and host-bypass workflows are `breakglass`,
5. which Substrate-owned commands must remain the normal operator path,
6. which transport and docs changes are still deferred to later slices.

## Frozen in this slice

This slice freezes only:

1. the minimum supported Lima/macOS capability contract for hardened same-user
   mode,
2. the supported-versus-breakglass classification for direct guest and
   host-bypass workflows,
3. the replacement rule that normal lifecycle, diagnostics, and validation
   flows should begin from Substrate-owned commands,
4. the minimal transport baseline statements required to support that
   classification.

## Deferred by design

This slice intentionally does **not** freeze:

1. the full canonical guest endpoint and adapter contract owned by Slice `03`,
2. PTY/non-PTY/readiness/doctor convergence owned by Slice `04`,
3. backend policy input parity mechanics owned by Slice `05`,
4. routed-path-first readiness/doctor/smoke truth owned by Slice `06`,
5. listener removal, mount minimization, or guest unit unification owned by
   later slices,
6. repo-wide breakglass reclassification and docs cutover owned by Slice `12`.

## Why this slice exists

Slice `01` froze the supported same-user posture and the taxonomy labels, but
it deliberately left one critical ambiguity open: what exact Lima/runtime
assumptions and operator escape hatches the supported posture may rely on.

Live repo truth shows that ambiguity is no longer harmless:

1. `scripts/mac/lima/substrate.yaml` already assumes `vmType: "vz"` and broad
   host mounts.
2. `crates/world-mac-lima/src/forwarding.rs` prefers `vsock-proxy`, falls back
   to SSH-backed UDS forwarding, and intentionally skips SSH TCP fallback.
3. `crates/world-mac-lima/src/lib.rs` still contains a stale `127.0.0.1:7788`
   TCP check, while shell-side macOS health and gateway logic still probe host
   TCP `17788` as a compatibility path.
4. `docs/reference/world/platforms/macos-lima-setup.md`,
   `scripts/mac/lima-doctor.sh`, and `scripts/mac/lima-warm.sh` still normalize
   repeated `limactl shell` and direct guest administration flows.

Without an explicit Slice `02`, later transport, docs, and cutover work would
have to guess whether those behaviors are part of the supported contract or
already breakglass debt.

## Official-source grounding that must drive this slice

The official Lima docs currently establish the following facts that this slice
must account for:

1. the Lima releases page shows `v1.x` support ended on February 6, 2026 while
   `v2.x` is the current supported major line,
2. the VZ docs require Lima `>= 0.14` and macOS `>= 13.0`,
3. the VM types docs state `vmType` is selected at instance creation time and
   cannot be changed later, and that starting with Lima `v1.0` new macOS
   instances use VZ by default only on macOS `>= 13.5` unless the config is
   incompatible with VZ,
4. the VZ docs also call out an Intel-macOS `< 13.5` known issue for Linux
   kernel `v6.2` guests on VZ, fixed in macOS `13.5`,
5. the `limactl shell` docs describe it as an SSH-based connection path into
   the instance,
6. the port-forwarding docs show default forwarding behavior changed across
   Lima versions,
7. the mount docs show `virtiofs` on macOS depends on `vmType: vz` and macOS
   `13+`,
8. the breaking-changes docs show several transport and mount assumptions
   shifted materially in `v1.0` and `v2.0`.

This slice should cite the exact official URLs above when those semantics drive
its final decisions.

## Packet 1 resolved environment contract

Task `1.2` freezes the supported environment contract as follows:

1. **Supported Lima lifecycle floor: Lima `v2.x`.**
   - The official Lima lifecycle page shows `v1.x` support ended on
     February 6, 2026, while `v2.x` is the current supported major line.
   - Slice `02` therefore stops using “recent Lima” wording and ties the
     supported hardened default to the currently supported major line instead of
     an older lifecycle branch.
2. **Supported macOS / VM floor: macOS `>= 13.0` with `vmType: "vz"` chosen at
   instance creation time, with explicit documented qualifications.**
   - The official VZ docs require Lima `>= 0.14` and macOS `>= 13.0`, so
     `13.0+` is the supported minimum capability range for running VZ.
   - The official VM-types docs state `vmType` can only be specified when the
     instance is created and cannot be changed later, and that starting with
     Lima `v1.0` new macOS instances use VZ by default only on macOS `>= 13.5`
     unless the config is incompatible with VZ.
   - The official VZ docs also call out an Intel-macOS `< 13.5` known issue for
     Linux kernel `v6.2` guests on VZ that is fixed in macOS `13.5`.
   - Slice `02` therefore keeps `macOS >= 13.0` as the supported floor for the
     hardened same-user Lima contract while carrying forward two separate
     qualifications rather than converting either into a universal floor: the
     pre-`13.5` Intel/Linux-kernel-`v6.2` caveat and the distinct `>= 13.5`
     default-VZ-for-new-instances behavior. This repo already pins
     `vmType: "vz"` in `scripts/mac/lima/substrate.yaml`.
3. **Supported repo capability assumptions already in play: VZ-backed guest
   operation plus VZ-compatible host mount semantics.**
   - Repo truth already depends on `vmType: "vz"` and host mounts in
     `scripts/mac/lima/substrate.yaml`.
   - The official mount docs state `virtiofs` on macOS is supported only with
     macOS `13+` and `vmType: vz`, and the official breaking-changes page says
     Lima `v1.0` changed the default mount type for VZ from `reverse-sshfs` to
     `virtiofs`.
   - Slice `02` therefore freezes a VZ-era mount-capability baseline without
     widening into Slice `08` mount-minimization work.
4. **`vsock-proxy` status for Packet 1: optional acceleration, not part of the
   environment floor.**
   - This is an inference from live repo truth plus the official Lima docs.
   - Live repo truth shows `crates/world-mac-lima/src/forwarding.rs` tries
     `vsock-proxy` first, then falls back to SSH-backed UDS forwarding, and
     intentionally skips SSH TCP fallback.
   - The official `limactl shell` docs describe Lima instance access as
     SSH-based by default, and the official port-forwarding docs describe SSH
     and GRPC as supported forwarding modes while noting AF_VSOCK only for Lima
     `>= 2.0` VZ guests whose guest `systemd` is `v256+` (for example,
     Ubuntu `24.10+`).
   - Packet `1` therefore does **not** require `vsock-proxy` to satisfy the
     supported environment floor, even though later slices may still narrow the
     transport contract further.
5. **Still deferred after Packet 1.**
   - Packet `1` does not freeze the breakglass workflow matrix.
   - Packet `1` does not freeze the final `17788` compatibility-path wording or
     stale `7788` cleanup strategy.
   - Packet `1` does not absorb Slice `03` transport unification or Slice `12`
     docs-cutover work.

## Packet 2 resolved breakglass matrix and supported replacement rule

Tasks `2.1` and `2.2` freeze the direct-guest and compatibility-path contract
as follows.

| Workflow or path | Classification | Why this is the right Slice `02` classification | Supported replacement or framing |
| --- | --- | --- | --- |
| Direct `limactl shell substrate ...` for routine lifecycle, repair, or validation | `breakglass` | The official `limactl shell` docs describe it as an SSH-based host entry path into the Lima guest, so it is not a Substrate-owned control-plane surface. Live repo truth still uses it heavily in `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, and `docs/reference/world/platforms/macos-lima-setup.md`, but repeated use does not promote it into the hardened default. | Normal operation should start from `substrate host doctor`, `substrate world doctor`, `substrate world gateway sync|status|restart`, and routed Lima-backed CLI flows first. |
| Direct guest `systemctl` administration for `substrate-world-service` or related units | `breakglass` | This is direct guest administration rather than the supported same-user control plane described by Slice `01`. It remains necessary for emergency repair and deep debugging, but it must not be the first-line operator story. | Use the Substrate-owned doctor and gateway lifecycle/status commands first; later slices can replace remaining warm/provision gaps without reclassifying guest `systemctl` as routine. |
| Direct guest socket curls such as `curl --unix-socket /run/substrate.sock ...` used as the primary health check | `breakglass` | These probes validate the canonical guest endpoint, but they bypass the supported operator entry points and expose raw guest implementation details directly. | Use `substrate world doctor --json`, `substrate host doctor --json`, and the routed validation surfaces first; reserve direct guest curls for deep debugging and evidence collection. |
| Host-side `SUBSTRATE_WORLD_SOCKET=<path>` override use on macOS | `breakglass` | This override bypasses the authoritative Lima-backed transport-selection path. Live repo truth already treats it as an advanced/test escape hatch rather than the normal default. | Use the default Lima-backed socket discovery and gateway/doctor commands without overrides unless emergency recovery or advanced testing requires a manual socket target. |
| Host TCP `127.0.0.1:17788` compatibility probing when it stays behind Substrate-owned doctor/gateway logic | `degraded-but-supported` | Live repo truth in `crates/shell/src/execution/platform/macos.rs` and `crates/shell/src/builtins/world_gateway.rs` still probes `17788` after preferring the host UDS path. That makes it a retained compatibility behavior inside supported commands, but not the supported default contract itself. | Operators should not target `17788` directly. It remains a compatibility probe only when reached through `substrate host doctor`, `substrate world doctor`, or `substrate world gateway ...` while Slice `03` owns transport unification. |

Packet `2` also freezes four framing rules that later slices must inherit:

1. **Supported replacement rule:** normal lifecycle, diagnostics, and
   validation flows should start from Substrate-owned commands:
   `substrate host doctor`, `substrate world doctor`,
   `substrate world gateway sync|status|restart`, and routed Lima-backed CLI
   execution paths that preserve the canonical `/run/substrate.sock` guest
   endpoint behind the adapter layer.
2. **Compatibility-path rule:** host TCP `17788` is not a supported operator
   target. It is only a retained compatibility probe when hidden behind
   Substrate-owned commands that already prefer the host UDS path first.
3. **Stale-constant rule:** the stale `127.0.0.1:7788` check in
   `crates/world-mac-lima/src/lib.rs` is explicit transport drift, not a
   second supported endpoint. Slice `02` records it as Slice `03` cleanup debt
   rather than baking it into the supported contract.
4. **Scope rule:** Packet `2` does not define the final host-adapter order,
   remove compatibility probes, or rewrite the operator docs. It only freezes
   how those remaining paths must be described until Slice `03` and Slice `12`
   land.

## Tech stack

- Rust workspace: `substrate` `0.2.8`
- Rust edition: `2021`
- MSRV: Rust `1.89`
- Primary slice output type: Markdown planning and contract docs under
  `macos-hardening/macos-hardened-same-user-lima/`

This slice remains documentation- and contract-first. It should not require
changes to Rust runtime code, shell scripts, or guest unit definitions.

## Commands

This is a source-driven docs-and-contract slice. Required commands should prove
repo truth and identify the exact drift the contract must classify.

```bash
# Review the phase-0 milestone authority
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md

# Review the execution rubric, roadmap, and design inputs
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/ROADMAP.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-lima-transport-contract.md

# Inspect the current Lima/runtime assumptions in repo truth
sed -n '1,220p' scripts/mac/lima/substrate.yaml
sed -n '1,260p' crates/world-mac-lima/src/forwarding.rs
sed -n '210,260p' crates/world-mac-lima/src/lib.rs
sed -n '360,460p' crates/shell/src/execution/platform/macos.rs
sed -n '220,280p' crates/shell/src/builtins/world_gateway.rs

# Inventory breakglass-sensitive wording and stale transport markers
rg -n "limactl shell|SUBSTRATE_WORLD_SOCKET|17788|7788|vsock|vmType|mounts:|substrate.sock" \
  macos-hardening/macos-hardened-same-user-lima \
  docs/WORLD.md \
  docs/reference/world/platforms/macos-lima-setup.md \
  scripts/mac/lima-warm.sh \
  scripts/mac/lima-doctor.sh \
  scripts/mac/smoke.sh \
  crates/world-mac-lima/src \
  crates/shell/src/execution/platform/macos.rs \
  crates/shell/src/builtins/world_gateway.rs
```

Optional evidence commands if the planning pass discovers a contradiction that
needs stronger live proof:

```bash
substrate host doctor --json
substrate world doctor --json
substrate world gateway status --json
scripts/mac/lima-doctor.sh
```

## Project structure

This slice should stay grounded to these directories and file families:

```text
macos-hardening/macos-hardened-same-user-lima/
├── README.md                                   → feature-level supported mode contract
├── ROADMAP.md                                  → slice map authority
├── EXECUTION-RUBRIC.md                         → skill and source gate authority
├── phase-0-security-contract-and-scope/
│   ├── README.md                               → phase-0 contract authority
│   └── milestone-0-2-lima-version-and-breakglass-contract-sow.md
└── spec/
    ├── SPEC-02-lima-version-floor-and-breakglass-contract.md
    ├── PLAN-02.md
    ├── TASKS-02.md
    └── design/
        ├── DESIGN-supported-mode-and-breakglass-taxonomy.md
        └── DESIGN-macos-lima-transport-contract.md

scripts/mac/
├── lima/substrate.yaml                         → current capability and mount assumptions
├── lima-warm.sh                                → provisioning and direct-guest repair posture
├── lima-doctor.sh                              → current diagnostic/breakglass posture
└── smoke.sh                                    → current validation and lifecycle posture

crates/
├── world-mac-lima/src/forwarding.rs            → adapter selection and fallback policy
├── world-mac-lima/src/lib.rs                   → stale TCP compatibility proof point
└── shell/src/execution/platform/macos.rs       → host doctor/readiness compatibility probes
```

Expected implementation touch surface for Slice `02` should remain feature-local
and documentation-first.

## Code style

This slice is Markdown-authority work. The style contract should be:

1. assumptions at the top,
2. explicit source-driven citations when official Lima semantics matter,
3. matrix-friendly support and breakglass wording,
4. precise absolute version and lifecycle language instead of “recent” or
   “modern,”
5. explicit non-goals that keep Slice `03` and Slice `12` intact.

Example of acceptable slice output style:

```md
## Supported environment contract

- Supported floor: Lima v2.x on macOS `>= 13.0` with `vmType: "vz"` chosen at
  instance creation for the same-user hardened default, while separately
  carrying Lima's documented pre-`13.5` Intel/Linux-kernel-`v6.2` caveat and
  the `>= 13.5` default-VZ-for-new-instances behavior as qualifications.
- Optional acceleration: `vsock-proxy` may improve the host-to-guest path, but
  it is not part of the environment floor.
- Breakglass-only: direct `limactl shell` or host-side
  `SUBSTRATE_WORLD_SOCKET` override use for routine operation.
- Deferred: canonical adapter unification and stale TCP cleanup remain Slice 03 work.
```

## Testing strategy

This slice does not primarily rely on cargo tests. Its validation is source
discipline, repo-truth alignment, and seam control.

Validation levels:

1. **Official-source review**
   - confirm every version- or behavior-sensitive claim is grounded in the
     official Lima docs cited above
2. **Repo-evidence review**
   - confirm the slice contract accounts for the current repo assumptions in
     `scripts/mac/lima/substrate.yaml`, `crates/world-mac-lima`, and the macOS
     docs/scripts
3. **Breakglass matrix review**
   - confirm direct guest commands, host-side socket override use, and retained
     compatibility probes are classified explicitly
4. **Scope validation**
   - confirm Slice `02` does not absorb full transport-unification, docs
     cutover, or runtime implementation work
5. **Diff review**
   - confirm the touched file set stays within feature-local planning docs
     unless an approved contradiction requires wider doc updates

## Boundaries

- Always:
  - treat Slice `02` as a source-driven contract slice
  - use the Slice `01` taxonomy exactly as landed
  - make the supported operator path start from Substrate-owned commands
  - separate supported capability assumptions from compatibility probes and
    breakglass escapes
  - preserve Slice `03` ownership of the canonical transport contract
- Ask first:
  - widening this slice into runtime code, provisioning, or repo-wide docs
    cutover
  - redefining the support taxonomy labels from Slice `01`
  - deciding transport implementation details that belong to Slice `03`
  - deciding mount minimization or guest unit unification that belong to later
    slices
- Never:
  - use vague “recent Lima” wording after this slice lands
  - treat direct `limactl shell` administration as part of the normal operator
    story
  - treat host TCP compatibility probes as the hardened supported default
  - silently absorb multiple milestone-scale seams into Slice `02`

## Success criteria

Slice `02` is successful when:

1. the docs define one explicit supported environment contract in capability and
   lifecycle terms,
2. the docs classify `limactl shell`, direct guest `systemctl`, direct guest
   socket curls, and host-side `SUBSTRATE_WORLD_SOCKET` override use explicitly
   instead of implicitly,
3. the docs state whether `vsock-proxy` is required, optional acceleration, or
   unsupported for the hardened default,
4. the docs explain the status of host TCP `17788` and stale `7788`
   references without accidentally freezing the whole transport design,
5. the docs leave canonical transport unification, operator docs cutover, and
   runtime hardening to their later slices,
6. a future short prompt can continue with Slice `03` without reopening Slice
   `02`.

## Remaining open questions for later packets

1. Is host TCP `17788` best classified as compatibility-only, breakglass-only,
   or fully deprecated-but-retained pending Slice `03` cleanup?
2. Which exact direct guest and host-bypass workflows Packet `2` should name
   first in the explicit breakglass matrix without widening into repo-wide docs
   cutover work.
