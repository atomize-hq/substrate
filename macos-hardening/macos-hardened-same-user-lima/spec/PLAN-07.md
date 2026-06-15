# PLAN-07: Remove Default Extra Listener Surface

Source spec:
- [`SPEC-07-remove-default-extra-listener-surface.md`](./SPEC-07-remove-default-extra-listener-surface.md)

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md`](../phase-2-same-user-hardening/milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](./SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)
- [`PLAN-06.md`](./PLAN-06.md)
- [`TASKS-06.md`](./TASKS-06.md)

Plan type: source-driven phase-2 listener-surface hardening slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `07`: remove the default guest TCP listener from
the macOS warm/repair path so same-user Lima’s hardened default listener story
matches the already-frozen socket-first transport and readiness contract.

This plan should produce a bounded landing that:

1. freezes `/run/substrate.sock` as the only hardened default guest listener,
2. removes default `SUBSTRATE_AGENT_TCP_PORT=61337` injection from the guest
   service path used by macOS warm/repair flows,
3. keeps any retained host compatibility routing explicitly separate from the
   guest listener contract,
4. updates listener-oriented evidence and docs so they no longer imply that
   guest TCP is part of the hardened default,
5. leaves mount narrowing, guest-unit source-of-truth unification, and broad
   breakglass/docs cutover to later slices.

## Packet 1 source gate and frozen listener decision

Packet `1` should freeze the listener contract before any implementation edits
begin.

Official source verification completed on 2026-06-12:

1. Lima `limactl shell` uses SSH by default and documents env-propagation
   controls such as `LIMA_SHELLENV_ALLOW` / `LIMA_SHELLENV_BLOCK`.
2. Lima SSH is a first-class interoperability path for software that expects
   SSH connectivity.
3. Lima port forwarding is adapter behavior with version-sensitive defaults
   (`SSH`/`GRPC`) and newer VZ/AF_VSOCK behavior.
4. Lima environment-variable docs confirm version-sensitive adapter toggles such
   as `LIMA_SSH_OVER_VSOCK` and `LIMA_SSH_PORT_FORWARDER`.
5. Lima breaking changes confirm that stale hard-coded SSH-port assumptions are
   unsafe.

Live 2026-06-12 repo-truth confirmation for Slice `07`:

1. `scripts/mac/lima-warm.sh` still writes
   `Environment=SUBSTRATE_AGENT_TCP_PORT=61337` into the guest service unit.
2. `scripts/mac/lima/substrate.yaml` already centers the intended guest
   listener contract on `ListenStream=/run/substrate.sock`.
3. `crates/world-service/src/lib.rs` already supports a no-default-TCP mode
   when `SUBSTRATE_AGENT_TCP_PORT` is absent and no TCP listeners are inherited.
4. `crates/world-mac-lima/src/transport.rs` still retains
   `127.0.0.1:17788` as a host compatibility endpoint, which this slice must
   keep clearly distinct from the guest listener contract.
5. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` still need wording
   tightening so compatibility TCP or breakglass guest access are not read as
   the hardened default.

Frozen Packet `1` decision:

1. the hardened default guest listener is `/run/substrate.sock` only,
2. guest TCP listener enablement via `SUBSTRATE_AGENT_TCP_PORT` must be removed
   from the macOS warm/repair default,
3. retained host compatibility routing may survive temporarily, but must be
   labeled compatibility or breakglass rather than supported listener truth,
4. Slice `07` must not silently absorb guest-unit generation unification,
5. Slice `07` should reuse Slice `06`’s routed-readiness proof story rather
   than reopen it.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md`
4. `scripts/mac/lima-warm.sh`
5. `scripts/mac/lima-doctor.sh` only if evidence or language must change to
   prove socket-only default honestly
6. `scripts/mac/smoke.sh` only if evidence or language must change to prove
   socket-only default honestly
7. `docs/WORLD.md`
8. `docs/reference/world/platforms/macos-lima-setup.md`
9. targeted tests or nearby shell coverage only as needed

By default this slice should **not** widen into:

1. `scripts/mac/lima/substrate.yaml`, `substrate-dev.yaml`, or
   `substrate-arch.yaml` unification work,
2. `crates/world-mac-lima/` transport-strategy redesign,
3. ingress/mount hardening,
4. guest-unit sandbox/source-of-truth redesign,
5. broad Phase `3` lifecycle work,
6. full feature-wide breakglass/docs cutover.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus official Lima docs as co-equal authority,
2. keep the slice centered on guest-listener default removal and contract
   wording, not transport redesign,
3. treat any need to redesign unit-generation architecture as explicit scope
   pressure for Slice `10`.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when they
drive decisions:

1. [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
2. [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
3. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
4. [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
5. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Conditional official sources:

1. if implementation changes socket-activation semantics or unit sandbox claims
   rather than just the extra-listener default, fetch targeted official
   `systemd.socket` / `systemd.exec` docs for only those exact claims,
2. if implementation requires transport-adapter redesign beyond listener-default
   removal, stop and treat that as scope pressure.

## Major components and dependencies

1. **listener-contract freeze**
   - define one hardened default guest listener posture
   - keep compatibility routing distinct from guest listener truth
2. **warm/repair cutover**
   - remove default `SUBSTRATE_AGENT_TCP_PORT=61337` injection from
     `scripts/mac/lima-warm.sh`
   - preserve `/run/substrate.sock`-centered service behavior
3. **evidence alignment**
   - prove routed doctor/gateway/smoke still work without a guest TCP listener
   - keep guest-direct or compatibility evidence clearly secondary
4. **doc alignment**
   - update `docs/WORLD.md` and
     `docs/reference/world/platforms/macos-lima-setup.md` so they no longer
     imply guest TCP is normal default behavior
5. **validation and handoff clarity**
   - prove the slice stayed listener-scoped
   - leave ingress/mount and unit-source-of-truth seams honestly deferred

Dependency order:

1. source-backed listener decision first,
2. warm/repair path cutover second,
3. evidence alignment third,
4. doc alignment fourth,
5. final validation and handoff last.

## Locked decisions

### What this slice changes

1. It removes default guest TCP listener enablement from the macOS warm/repair
   path.
2. It freezes `/run/substrate.sock` as the only hardened default guest
   listener.
3. It makes listener-oriented evidence and docs distinguish guest listener truth
   from retained host compatibility routing.
4. It keeps routed readiness proof green without relying on guest TCP.

### What this slice does not change

1. no transport-strategy redesign,
2. no mount or ingress redesign,
3. no guest-unit generation/source-of-truth redesign,
4. no broad lifecycle CLI redesign,
5. no full breakglass/docs cutover.

## Implementation order

### Packet 1: Freeze the listener contract and source gate

Goal:

1. confirm the authority stack, official source set, and live listener drift,
2. freeze the exact hardened default listener decision,
3. identify which script/doc sections still imply that guest TCP is normal.

Primary touch surface:

1. `SPEC-07-remove-default-extra-listener-surface.md`
2. `PLAN-07.md`
3. `TASKS-07.md`

Why first:

1. the slice should not start editing warm scripts or docs before freezing what
   the hardened default actually is,
2. later packets depend on distinguishing guest listener truth from host
   compatibility routing.

Verification checkpoint:

1. official Lima shell/SSH/port-forwarding/environment semantics are cited for
   non-obvious decisions,
2. the hardened default listener decision is explicit,
3. the exact contradiction surfaces are named explicitly:
   - `scripts/mac/lima-warm.sh`
   - `crates/world-service/src/lib.rs`
   - `crates/world-mac-lima/src/transport.rs`
   - `docs/WORLD.md`
   - `docs/reference/world/platforms/macos-lima-setup.md`

### Packet 2: Remove default guest TCP enablement from warm/repair

Goal:

1. cut `SUBSTRATE_AGENT_TCP_PORT=61337` from the macOS warm/repair path,
2. keep the rendered guest service centered on `/run/substrate.sock`,
3. update any local assertions or comments that still assume guest TCP.

Why second:

1. warm/repair is the concrete remaining implementation contradiction,
2. later evidence and docs should be rewritten against the actual tightened
   behavior.

Verification checkpoint:

1. `bash -n scripts/mac/lima-warm.sh` passes,
2. `rg -n "SUBSTRATE_AGENT_TCP_PORT|61337" scripts/mac/lima-warm.sh` shows the
   removed default or only explicitly intentional breakglass references,
3. rendered service expectations still preserve `/run/substrate.sock`.

### Packet 3: Align evidence and docs to the tightened listener posture

Goal:

1. ensure doctor/smoke evidence does not imply guest TCP is required,
2. update listener-oriented docs so compatibility TCP and guest-direct access
   are clearly secondary.

Why third:

1. evidence should be rewritten against the final warm/repair truth,
2. docs should describe the narrowed listener surface that actually exists.

Verification checkpoint:

1. `bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh` passes if those
   files are touched,
2. doc sections lead with socket-first/routed behavior rather than guest TCP,
3. compatibility TCP and guest-direct commands are clearly labeled
   compatibility or breakglass.

### Packet 4: Final validation and next-slice handoff clarity

Goal:

1. validate that Slice `07` stayed listener-scoped,
2. run final targeted verification for scripts/docs and any minimal fallout,
3. leave a clean handoff to Slice `08` / Slice `09` and Slice `10`.

Why last:

1. Phase `2.1` only becomes honest if the final evidence shows the extra
   listener default is gone,
2. the handoff must make it explicit that ingress and unit-source-of-truth work
   still remain.

Verification checkpoint:

1. targeted syntax checks pass,
2. final diff stays limited to listener-oriented surfaces,
3. final closeout states explicitly that Slice `08` / `09` own ingress and
   mount work and Slice `10` owns guest-unit source-of-truth work.

## Major risks and mitigations

### Risk 1: The slice quietly widens into unit unification

Mitigation:

1. treat YAML/template consolidation as deferred to Slice `10`,
2. keep this slice focused on the warm/repair-installed default listener.

### Risk 2: Compatibility TCP wording regresses into being treated as supported default behavior

Mitigation:

1. always distinguish guest listener truth from host compatibility routing,
2. keep compatibility TCP labeled compatibility or breakglass only.

### Risk 3: Routed proof is accidentally described as depending on guest TCP

Mitigation:

1. reuse Slice `06`’s routed-first proof order,
2. make any remaining guest-direct checks clearly secondary,
3. confirm docs and helper evidence still pass without implying guest TCP is
   required.
