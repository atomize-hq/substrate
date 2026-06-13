# PLAN-10: Guest Unit Source of Truth and Sandbox Unification

Source spec:
- [`SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](./SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-09-ingress-cutover-and-explicit-staging-path.md`](./SPEC-09-ingress-cutover-and-explicit-staging-path.md)
- [`PLAN-09.md`](./PLAN-09.md)
- [`TASKS-09.md`](./TASKS-09.md)

Plan type: source-driven phase-2 unit-unification slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `10`: remove bootstrap-vs-repair drift by
landing one authoritative guest service/socket contract and one rendered
sandbox policy for same-user Lima after the Slice `09` staged-ingress cutover.

This plan should produce a bounded landing that:

1. removes separately handwritten guest unit authority across
   `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh`,
2. unifies hardening-critical service fields such as environment,
   `ProtectHome=`, `ReadWritePaths=`, capabilities, runtime directories, and
   socket ownership/mode,
3. preserves Slice `07` socket-first listener behavior and Slice `09`
   guest-local staged-workspace/writable-root behavior,
4. adds validation that proves rendered-unit parity for freshly created and
   repaired VMs,
5. makes only the minimal doc-truth updates required by the unification,
6. leaves Slice `11` and Slice `12` with explicit boundaries.

## Packet 1 source gate, drift inventory, and canonical-source decision

Packet `1` should freeze the exact unification strategy before edits start.

Official source verification completed on 2026-06-13 using the required
`systemd.exec`, `systemd.socket`, Lima `limactl copy`, and Lima Filesystem
mount docs:

1. `ProtectHome=`, `ReadWritePaths=`, `RuntimeDirectory=`, and
   `StateDirectory=` are sandbox-contract fields, not incidental script text.
2. `ListenStream=` and `SocketMode=` are part of the authoritative socket
   contract.
3. Slice `09`’s staged-workspace path remains an explicit guest-local contract,
   not something this slice should reopen.

Live 2026-06-13 repo-truth confirmation for Slice `10`:

1. `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh` still each
   write guest service/socket units.
2. The socket unit bodies already match closely, but the service unit bodies
   still drift on `SUBSTRATE_WORLD_SOCKET`, `SUBSTRATE_HOME`, optional
   `WORLD_NETFILTER_ENABLE`, guest-home writability, and `CAP_CHOWN`.
3. `docs/WORLD.md` now aligns more closely with the warm-script-rendered unit
   than with the profile-bootstrap unit.
4. `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` do not yet prove
   rendered-unit parity.

Frozen Packet `1` implementation decision:

1. Slice `10` will use a checked-in canonical unit source plus a single
   render/install path under `scripts/mac/lima/`.
2. The canonical source may be parameterized, but create/bootstrap and
   warm/repair may no longer maintain separately handwritten service bodies.
3. `scripts/mac/lima/substrate.yaml` should reduce itself to VM bootstrap
   prerequisites only; the actual service/socket contract should be installed
   by the canonical render/install path after guest creation so the YAML is no
   longer a second handwritten authority.
4. The frozen Phase `2` target contract for Packet `2` is:
   - socket parity on `ListenStream=/run/substrate.sock`, `SocketMode=0660`,
     `SocketUser=root`, `SocketGroup=substrate`, `DirectoryMode=0750`, and
     `RemoveOnStop=yes`
   - service environment parity on `RUST_LOG=info`,
     `SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`,
     `SUBSTRATE_HOME=<guest-home>/.substrate`, and conditional
     `WORLD_NETFILTER_ENABLE=1` when host-side netfilter support is requested
   - runtime/sandbox parity on `Group=substrate`, `UMask=0027`,
     `RuntimeDirectory=substrate`, `RuntimeDirectoryMode=0750`,
     `StateDirectory=substrate`, `StateDirectoryMode=0750`,
     `WorkingDirectory=/var/lib/substrate`, `ProtectSystem=strict`,
     `ProtectHome=read-only`,
     `ReadWritePaths=<guest-home>/.substrate /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp`,
     and explicit preservation of the managed gateway-runtime surface under
     `/run/substrate/substrate-gateway-runtime/`
   - capability parity on `CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE`
5. Packet `2` should remain script/config/docs first and should not escalate
   into Rust/backend surfaces unless parity cannot stay honest otherwise.

## Packet 2 canonical unit source and create/repair convergence

Packet `2` performs the actual authority cutover.

Primary responsibilities:

1. introduce one authoritative checked-in source for
   `substrate-world-service.service` and `.socket`,
2. make the warm/repair path install rendered units from that source,
3. make the fresh-create/bootstrap path consume that same source or stop being
   a second handwritten authority,
4. unify the final service sandbox contract around the already-frozen Phase `2`
   listener and ingress results,
5. keep any supported parameterization explicit and minimal.

Likely touched files:

1. `scripts/mac/lima-warm.sh`
2. `scripts/mac/lima/substrate.yaml`
3. new canonical unit source under `scripts/mac/lima/` (for example,
   `scripts/mac/lima/units/` plus a minimal render/install helper if needed)

Packet `2` should review, but not edit by default:

1. `crates/world-service/src/lib.rs`
2. `crates/world-service/src/gateway_runtime.rs`
3. `docs/USAGE.md`

Escalate into those surfaces only if the unified unit contract cannot remain
honest without a minimal assist.

## Packet 3 validation parity and minimal doc-truth cutover

Packet `3` adapts the proof surfaces to the new unit contract.

Primary responsibilities:

1. add validation that freshly created and repaired VMs render the same unit
   contract,
2. keep `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`,
   `substrate host doctor --json`, `substrate world doctor --json`, and
   `substrate world gateway status --json` central,
3. update only the doc statements made false by removing dual handwritten unit
   authority,
4. keep broader operator-story/productization edits deferred.

Likely touched files:

1. `scripts/mac/lima-doctor.sh`
2. `scripts/mac/smoke.sh`
3. `docs/WORLD.md`
4. `docs/reference/world/platforms/macos-lima-setup.md`

Packet `3` should not silently widen into:

1. broad Phase `3` lifecycle/sync UX productization,
2. a full breakglass/docs cutover,
3. reopened ingress redesign,
4. unrelated Linux or WSL service-installation work.

## Packet 4 final validation and downstream handoff

Packet `4` closes the slice out honestly.

Primary responsibilities:

1. confirm there is exactly one authoritative guest unit contract,
2. confirm create/bootstrap and warm/repair install the same rendered service
   and socket definitions,
3. confirm the final sandbox preserves the Slice `07` and Slice `09` results,
4. confirm doctor/smoke/docs now describe unit parity honestly,
5. record any remaining operator-surface/productization work for Slice `11`,
6. record any remaining broad breakglass/doc-cutover work for Slice `12`.

## Default landing boundary

Unless execution proves a small backend assist is mandatory, this slice should
land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md`
4. `scripts/mac/lima/substrate.yaml`
5. `scripts/mac/lima-warm.sh`
6. the new canonical unit-source files under `scripts/mac/lima/`
7. `scripts/mac/lima-doctor.sh`
8. `scripts/mac/smoke.sh`
9. `docs/WORLD.md`
10. `docs/reference/world/platforms/macos-lima-setup.md`

This slice should review, but not edit by default:

1. `crates/world-service/src/lib.rs`
2. `crates/world-service/src/gateway_runtime.rs`
3. `docs/USAGE.md`
4. `scripts/mac/orchestration-smoke.sh`

By default this slice should **not** widen into:

1. broad Substrate-owned lifecycle/sync command productization,
2. the full docs/breakglass cutover,
3. a reopened ingress or mount-policy redesign,
4. cross-platform unit-installation redesign outside the macOS hardening seam,
5. unrelated transport or policy rework already frozen earlier in the feature.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus official systemd docs as co-equal authority,
2. treat Slice `09`’s staged-ingress result as inherited contract rather than a
   fresh design question,
3. stop if the work starts turning into the broader operator-surface seam that
   belongs to Slice `11`.

## Official source set this plan must use

The plan assumes the resulting slice cites these official docs when they drive
decisions:

1. [systemd.exec](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)
2. [systemd.socket](https://www.freedesktop.org/software/systemd/man/systemd.socket.html)
3. [Lima `limactl copy`](https://lima-vm.io/docs/reference/limactl_copy/)
4. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

## Major components and dependencies

1. **drift inventory and target contract**
   - identify every hardening-critical field that currently differs
   - freeze the final authoritative field set
2. **canonical unit source**
   - one checked-in service/socket authority
   - minimal, explicit parameterization only where supported
3. **create/repair convergence**
   - fresh-create/bootstrap and warm/repair consume the same rendered contract
4. **validation parity**
   - doctor/smoke evidence proves rendered-unit parity, not just liveness
5. **minimal doc truth alignment**
   - docs stop implying dual unit authority is acceptable
6. **downstream handoff clarity**
   - Slice `11` and Slice `12` receive explicit deferred boundaries

Dependency order:

1. source gate and target-contract freeze first,
2. canonical unit source and create/repair convergence second,
3. validation and doc-truth cutover third,
4. final verification and downstream handoff last.

## Locked decisions

### What this slice changes

1. It removes dual handwritten guest unit authority on macOS.
2. It lands one authoritative rendered service/socket contract.
3. It finalizes the Phase `2` guest service sandbox around the already-landed
   listener and ingress results.
4. It adds validation for rendered-unit parity.
5. It performs minimal docs truth corrections required by the unification.

### What this slice does not change

1. It does not design the final Substrate-owned lifecycle/sync UX.
2. It does not reopen Slice `09` ingress design.
3. It does not perform the broad breakglass/doc-cutover.
4. It does not silently widen into unrelated Rust/backend work.
