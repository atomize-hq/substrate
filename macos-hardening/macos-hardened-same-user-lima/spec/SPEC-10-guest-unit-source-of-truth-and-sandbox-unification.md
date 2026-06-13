# Spec: Slice 10 Guest Unit Source of Truth and Sandbox Unification

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-09-ingress-cutover-and-explicit-staging-path.md`](./SPEC-09-ingress-cutover-and-explicit-staging-path.md)
- [`PLAN-09.md`](./PLAN-09.md)
- [`TASKS-09.md`](./TASKS-09.md)
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)

Required official source set for this slice:
- [systemd.exec](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)
- [systemd.socket](https://www.freedesktop.org/software/systemd/man/systemd.socket.html)
- [Lima `limactl copy`](https://lima-vm.io/docs/reference/limactl_copy/)
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: unify the macOS guest `substrate-world-service` service/socket
contract behind one authoritative source and one rendered sandbox policy,
consuming the Slice `09` staged-workspace ingress result without widening into
Slice `11` operator-surface productization or Slice `12` broad docs/breakglass
cutover.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `09`, and Slice `10` is now
   the next honest dependency-ordered seam in the local feature sequence.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `10` owns guest-unit source of
   truth plus sandbox unification, Slice `11` still owns the broader
   Substrate-owned lifecycle/diagnostics and sync productization seam, and
   Slice `12` still owns the broad breakglass/docs cutover.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   decision boundary depends on current systemd socket/service sandbox
   semantics, not repo wording alone.
4. Live repo truth on 2026-06-13 still shows the exact drift this slice must
   consume:
   - `scripts/mac/lima/substrate.yaml` still provisions handwritten guest
     `substrate-world-service.service` and `.socket` units during VM creation,
   - `scripts/mac/lima-warm.sh` still rewrites handwritten guest
     `substrate-world-service.service` and `.socket` units during warm/repair,
   - the socket units are effectively identical across those two paths, but the
     service units still diverge on hardening-critical fields including
     `Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`,
     `Environment=SUBSTRATE_HOME=<guest-home>/.substrate`, optional
     `WORLD_NETFILTER_ENABLE=1`, the presence of `${SUBSTRATE_GUEST_HOME}` in
     `ReadWritePaths=`, and inclusion of `CAP_CHOWN` in
     `CapabilityBoundingSet=` / `AmbientCapabilities=`,
   - `docs/WORLD.md` now describes the macOS guest service contract in a way
     that matches the warm-script-rendered unit more closely than the
     profile-bootstrap unit, proving the repo already has doc-vs-bootstrap
     drift,
   - `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` still prove routed
     readiness and gateway behavior, but they do not yet prove that freshly
     created and repaired guests render the same unit contract.
5. Slice `09` already cut ingress over to the guest-local staged workspace root
   at `/var/lib/substrate/staged-workspace/current`, so Slice `10` should
   consume that frozen writable-path result rather than reopen mount design.
6. Slice `07` already removed the default guest TCP-listener contradiction, so
   Slice `10` must preserve the socket-first listener contract rather than
   revisit it.
7. The smallest honest implementation is still script/config/docs first. This
   slice should not widen into Rust code or Phase `3` CLI contract changes
   unless live proof forces a minimal assist.
8. If a single authoritative source requires parameterization, that
   parameterization must still flow through one checked-in canonical contract
   rather than through two separately handwritten unit bodies.

If any of these are wrong, correct them before execution.

## Packet 1 official-source confirmation and frozen unification direction

Packet `1` was re-grounded on 2026-06-13 against both the required official
source set and current repo truth before any Slice `10` docs were written.

Official source verification completed on 2026-06-13 using the required
`systemd.exec`, `systemd.socket`, Lima `limactl copy`, and Lima Filesystem
mount docs:

1. `systemd.exec` documents `ProtectHome=` plus `ReadWritePaths=` as part of
   the filesystem-protection/sandbox contract, and it explicitly treats
   `RuntimeDirectory=` / `StateDirectory=` as service-managed writable-path
   mechanisms rather than incidental implementation details.
2. `systemd.exec` also documents that writable carve-outs under stricter
   filesystem protection must be explicitly allow-listed, which makes the final
   guest writable-path set a hardening contract, not a convenience setting.
3. `systemd.socket` documents `ListenStream=` for the socket endpoint and
   `SocketMode=` for file-system sockets, so socket ownership/mode and service
   wiring belong to the authoritative unit contract.
4. Lima’s `limactl copy` docs confirm the current host↔guest staging primitive
   is an explicit supported copy path. That means Slice `10` should preserve
   the Slice `09` guest-local staged-workspace result instead of reopening
   mounted-checkout assumptions.
5. Lima’s Filesystem mounts docs keep mount posture as explicit guest exposure
   configuration, which reinforces that Slice `10` must encode the already-cut
   staged ingress and writable-root posture rather than recreate broader host
   visibility inside unit policy.

Live 2026-06-13 repo-truth confirmation for Packet `1`:

1. `scripts/mac/lima/substrate.yaml` still provisions a guest service unit with
   these notable hardening-critical fields:
   - `ProtectHome=read-only`
   - `ReadWritePaths=/var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp`
   - no `Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`
   - no `Environment=SUBSTRATE_HOME=...`
   - no conditional `WORLD_NETFILTER_ENABLE=1`
   - no `CAP_CHOWN` in capability sets
2. `scripts/mac/lima-warm.sh` still rewrites a guest service unit with the same
   base structure but adds or changes these hardening-critical fields:
   - `Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`
   - `Environment=SUBSTRATE_HOME=${SUBSTRATE_GUEST_HOME}`
   - conditional `Environment=WORLD_NETFILTER_ENABLE=1`
   - `ReadWritePaths=${SUBSTRATE_GUEST_HOME} /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp`
   - `CAP_CHOWN` present in both capability sets
3. The socket unit body is functionally identical across the YAML bootstrap and
   warm/repair path today, which means the real remaining drift is service-unit
   authority and service-sandbox content.
4. `docs/WORLD.md` now states that the provisioning scripts wire
   `SUBSTRATE_HOME` into the unit and keep that path writable, which is true of
   the warm-script-rendered unit but false of the profile-bootstrap unit.
5. `docs/reference/world/platforms/macos-lima-setup.md` now teaches the staged
   workspace under `/var/lib/substrate/staged-workspace/current`, which means
   the final unit sandbox needs to preserve that guest-local writable-root
   contract after unification.
6. `scripts/mac/lima-doctor.sh` still verifies routed readiness and optional
   breakglass guest state, but it does not yet verify that the rendered unit in
   the guest matches one authoritative checked-in contract.

Frozen Packet `1` unification direction:

1. Slice `10` owns one authoritative guest unit contract for both
   `substrate-world-service.service` and `substrate-world-service.socket`.
2. That contract must preserve the already-landed Slice `07` socket-first
   listener posture and the Slice `09` guest-local staged-workspace/writable
   root posture.
3. Hardening-critical fields may be parameterized, but they may no longer be
   maintained as two separately handwritten unit bodies.
4. The authoritative mechanism is a checked-in canonical unit source plus one
   render/install path under `scripts/mac/lima/`, with
   `scripts/mac/lima/substrate.yaml` reduced to VM bootstrap prerequisites only
   while the warm/create flow installs the actual service/socket units from that
   canonical source after guest creation.
5. The frozen Phase `2` target contract that Packet `2` must render identically
   across fresh-create and warm/repair is:
   - socket: `ListenStream=/run/substrate.sock`, `SocketMode=0660`,
     `SocketUser=root`, `SocketGroup=substrate`, `DirectoryMode=0750`, and
     `RemoveOnStop=yes`
   - service environment: `Environment=RUST_LOG=info`,
     `Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`,
     `Environment=SUBSTRATE_HOME=<guest-home>/.substrate`, plus conditional
     `Environment=WORLD_NETFILTER_ENABLE=1` only when the host-side netfilter
     input requests it
   - service runtime/sandbox: `RuntimeDirectory=substrate`,
     `StateDirectory=substrate`, `WorkingDirectory=/var/lib/substrate`,
     `ProtectSystem=strict`, `ProtectHome=read-only`,
     `ReadWritePaths=<guest-home>/.substrate /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp`,
     and explicit preservation of the managed gateway-runtime surface under
     `/run/substrate/substrate-gateway-runtime/`
   - capabilities: `CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE`
6. Packet `2` remains script/config/docs first; no Rust/backend assist is
   currently required to keep the unit-authority cutover honest, though the
   slice may still adjust doctor/smoke/docs enough to prove and describe parity
   without widening into the broader Phase `3`
   operator-surface/productization story.

## Objective

Make the macOS guest service contract reviewable from one authoritative source
instead of by diffing bootstrap and repair scripts.

This slice is complete only when a reviewer can answer, without guessing:

1. what the authoritative guest service/socket contract is,
2. whether a freshly created and a repaired VM install the same hardening
   settings,
3. which guest writable paths are intentionally preserved after Slice `09`,
4. whether `ProtectHome=`, `ReadWritePaths=`, capabilities, environment, and
   socket ownership/mode are unified rather than drifting,
5. which validation surfaces prove rendered-unit parity,
6. what work still remains deferred to Slice `11` and Slice `12`.

## Frozen in this slice

This slice freezes only:

1. one authoritative source for the macOS guest service/socket contract,
2. one rendered sandbox policy for create/bootstrap and warm/repair,
3. the final Phase `2` writable-path and environment contract consumed from
   Slice `07` and Slice `09`,
4. the validation steps needed to prove rendered-unit parity and preserved
   routed behavior,
5. the minimum doc-truth updates required to describe the unified contract
   honestly.

## Deferred by design

This slice intentionally does **not** freeze:

1. broad Substrate-owned lifecycle/sync UX productization from Slice `11`,
2. the broad breakglass reclassification and docs cutover from Slice `12`,
3. a reopened ingress redesign or mount-policy rethink after Slice `09`,
4. Linux, WSL, or non-macOS unit-installation redesign beyond shared doc
   consistency where required,
5. broad Rust/backend redesign unless a minimal assist is proven mandatory.

## Why this slice exists

Phase `2.3` is only honest if the repo stops asking reviewers to infer the guest
sandbox contract from two drifting handwritten unit definitions.

Live repo truth now makes that the next seam:

1. Slice `07` already removed the default guest TCP-listener contradiction.
2. Slice `09` already converted ingress to a guest-local staged workspace and
   narrowed the writable-root story accordingly.
3. The remaining Phase `2` hardening gap is that create/bootstrap and
   warm/repair still define the guest service contract in two places, and those
   places still disagree on environment, writable paths, and capabilities.
4. Until that drift is removed, docs and validation cannot honestly point to
   one macOS guest service contract.

## Validation command wall

Use these commands as the default evidence wall for this slice when they become
relevant during implementation:

```bash
bash -n scripts/mac/lima-warm.sh
bash -n scripts/mac/lima-doctor.sh
bash -n scripts/mac/smoke.sh
scripts/mac/lima-warm.sh --check-only
scripts/mac/lima-doctor.sh
scripts/mac/smoke.sh
substrate host doctor --json
substrate world doctor --json
substrate world gateway status --json
```

If execution widens into any Rust/backend surface, add the smallest targeted
crate tests that prove the widened scope explicitly.

## Success criteria

1. There is exactly one authoritative macOS guest unit source for
   `substrate-world-service.service` and `.socket`.
2. Fresh-create and warm/repair flows install the same rendered unit contract.
3. The unified service contract preserves the Slice `07` listener result and
   the Slice `09` staged-workspace/writable-root result.
4. Doctor and/or smoke evidence proves rendered-unit parity rather than only VM
   liveness.
5. Docs stop implying that bootstrap and repair may legitimately carry
   different unit/sandbox settings.
6. Slice `11` and Slice `12` remain clearly deferred.
