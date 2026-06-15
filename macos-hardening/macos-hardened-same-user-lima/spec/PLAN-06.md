# PLAN-06: Routed-Path-First Doctor, Smoke, and Readiness Truth

Source spec:
- [`SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](./SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-policy-input-parity.md`](./design/DESIGN-macos-policy-input-parity.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)
- [`PLAN-04.md`](./PLAN-04.md)
- [`TASKS-04.md`](./TASKS-04.md)
- [`SPEC-05-backend-policy-input-parity.md`](./SPEC-05-backend-policy-input-parity.md)
- [`PLAN-05.md`](./PLAN-05.md)
- [`TASKS-05.md`](./TASKS-05.md)

Plan type: source-driven phase-1 readiness-evidence and docs/script cutover
slice for `macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `06`: make the routed Substrate path the
authoritative readiness proof for macOS, both in scripts and in
readiness-oriented docs.

This plan should produce a bounded landing that:

1. freezes one readiness evidence order for already provisioned macOS backends,
2. aligns `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` to fail on
   routed-path problems before guest-direct success can mask them,
3. updates readiness-oriented docs to lead with CLI doctor and gateway
   lifecycle/status surfaces,
4. preserves direct guest entry only as breakglass or post-failure diagnosis,
5. leaves full lifecycle ownership redesign and broad docs cutover to later
   slices.

## Packet 1 source gate and frozen readiness-order decision

Packet `1` should freeze the readiness proof order before any script or doc
edits begin.

Official Lima source verification completed on 2026-06-12:

1. [`limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
   confirms that Lima shell access uses SSH by default and documents the
   relevant `LIMA_SHELLENV_ALLOW` / `LIMA_SHELLENV_BLOCK` behavior for
   `--preserve-env`.
2. [Usage: SSH](https://lima-vm.io/docs/usage/ssh/) confirms that direct SSH is
   a first-class interoperability path distinct from the Substrate-owned
   readiness contract.
3. [Port Forwarding](https://lima-vm.io/docs/config/port/) confirms that
   forwarded localhost reachability is adapter behavior with version-sensitive
   default forwarders (`SSH`/`GRPC`), not a timeless contract surface.
4. [Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
   confirms that `LIMA_INSTANCE`, `LIMA_SHELLENV_ALLOW`, and
   `LIMA_SHELLENV_BLOCK` are advanced environment controls, not the ordinary
   readiness path.
5. [Breaking changes](https://lima-vm.io/docs/releases/breaking/) confirms
   that the SSH port is no longer hard-coded to `60022` for the default
   instance, so the slice must not freeze brittle direct-port assumptions into
   the supported readiness story.

Live 2026-06-12 repo-truth confirmation for Slice `06`:

1. `scripts/mac/lima-doctor.sh` currently checks VM health, socket existence,
   guest `curl`, guest `systemctl`, and guest package/layout facts directly via
   `limactl shell`, with no routed `substrate host doctor` /
   `substrate world doctor` lead-in at all.
2. `scripts/mac/smoke.sh` already includes:
   - `substrate world gateway sync`
   - `substrate world gateway status --json`
   - `substrate world gateway restart`
   - `substrate world doctor --json`
   but still omits `substrate host doctor` from the readiness proof and still
   mixes the routed checks with direct guest `curl`, direct guest binary
   checks, and guest `systemctl` readiness checks in the same proof flow.
3. `crates/shell/src/execution/platform/macos.rs` already prefers selected
   host-visible transport and only then falls back to guest-direct capabilities
   or world-doctor probing via `limactl shell`.
4. `docs/reference/world/platforms/macos-lima-setup.md` already says the CLI
   doctor commands are canonical, but still presents direct guest probing
   prominently in troubleshooting and setup guidance.
5. `docs/WORLD.md` still mixes routed readiness guidance with guest-log,
   override, and manual guest-entry material closely enough that the happy-path
   ordering remains fuzzy.
6. `docs/USAGE.md`, `docs/contracts/gateway/operator-contract.md`, and
   `docs/contracts/gateway/status-schema.md` already freeze the owned operator
   surfaces this slice should reuse instead of redefining them:
   - `substrate host doctor`
   - `substrate world doctor`
   - `substrate world gateway sync|status|restart`
   - `substrate world gateway status --json`

Frozen Packet `1` evidence order decision:

1. `substrate host doctor` and `substrate world doctor` come first,
2. managed gateway lifecycle/status proof comes second,
3. routed smoke proof comes third,
4. script wrappers should encode that order rather than redefine it,
5. guest-direct `limactl shell`, in-guest `curl`, guest `systemctl`, and guest
   journal access remain fallback or breakglass evidence only,
6. `SUBSTRATE_WORLD_SOCKET` remains advanced/test/breakglass on macOS and must
   not be restated as the default Lima readiness path.

Why this is the bounded choice:

1. Slice `04` and Slice `05` already established the transport and backend
   parity floor that Slice `06` must now prove,
2. the gateway contract docs already freeze owned readiness/status surfaces, so
   the slice should reuse them instead of inventing new health endpoints,
3. Phase `1.3` is fundamentally a proof-story and evidence-order seam, not a
   broad architecture seam.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md`
4. `scripts/mac/lima-doctor.sh`
5. `scripts/mac/smoke.sh`
6. `docs/WORLD.md`
7. `docs/reference/world/platforms/macos-lima-setup.md`
8. `docs/USAGE.md` only if readiness wording there needs alignment
9. `docs/contracts/gateway/operator-contract.md` only if a small wording
   alignment is needed
10. `docs/contracts/gateway/status-schema.md` only if a small wording
    alignment is needed
11. `crates/shell/src/execution/platform/macos.rs` only if a small doctor
    output or fallback-visibility tweak is needed to support the cutover
12. targeted tests or nearby shell coverage only as needed

By default this slice should **not** widen into:

1. `scripts/mac/lima-warm.sh`,
2. `crates/world-mac-lima/`, `crates/world-api/`, or policy-carrier surfaces,
3. listener removal, ingress/mount hardening, guest-unit sandboxing, or
   ownership-boundary work,
4. full feature-wide breakglass/docs cutover beyond readiness-oriented paths,
5. gateway lifecycle redesign beyond tiny wording alignment.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus official Lima docs as co-equal authority,
2. keep the slice centered on evidence order and operator truth, not on
   backend redesign,
3. treat any requirement to change unit semantics or broader lifecycle
   semantics as explicit scope pressure for a later slice.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when
they drive decisions:

1. [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
2. [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
3. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
4. [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
5. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Conditional official sources:

1. if implementation changes how docs explain guest service state semantics,
   fetch targeted systemd docs for only those exact claims,
2. if implementation changes gateway/unit lifecycle meaning rather than just
   evidence ordering, stop and treat that as scope pressure.

## Major components and dependencies

1. **readiness evidence order freeze**
   - define one authoritative order for happy-path readiness proof
   - keep routed CLI and gateway surfaces ahead of any guest-direct probes
2. **script cutover**
   - align `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` to encode
     the frozen readiness order
   - keep breakglass probes explicit rather than deleting every guest probe
3. **doc cutover**
   - update `docs/WORLD.md` and
     `docs/reference/world/platforms/macos-lima-setup.md` to match the frozen
     order
   - only touch other docs if a direct contradiction requires it
4. **optional doctor-output clarity**
   - adjust `crates/shell/src/execution/platform/macos.rs` only if scripts/docs
     need clearer routed-versus-fallback evidence from the owned doctor
     surfaces
5. **validation and handoff clarity**
   - prove the slice stayed narrow
   - leave Slice `07` and Slice `12` with a truthful readiness-story floor

Dependency order:

1. source-backed readiness order first,
2. repo-truth script inventory second,
3. script cutover third,
4. doc cutover fourth,
5. optional doctor-output clarity fifth,
6. final validation and handoff last.

## Locked decisions

### What this slice changes

1. It defines one happy-path readiness order for already provisioned macOS
   backends.
2. It makes helper scripts prove routed readiness before guest-direct success.
3. It makes readiness-oriented docs lead with owned CLI/gateway surfaces.
4. It preserves direct guest commands only as breakglass or post-failure
   diagnosis.

### What this slice does not change

1. no transport-contract redesign,
2. no backend policy-carrier redesign,
3. no warm/provision replacement,
4. no listener, ingress, mount, or guest-unit hardening,
5. no broad Phase `3` lifecycle redesign.

## Implementation order

### Packet 1: Freeze the readiness order and source gate

Goal:

1. confirm the authority stack, official Lima source set, and live repo drift,
2. freeze the exact happy-path readiness order,
3. identify which script/doc sections are contradicting that order today.

Primary touch surface:

1. `SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`
2. `PLAN-06.md`
3. `TASKS-06.md`

Why first:

1. the slice should not start “rewriting docs and scripts” before defining what
   counts as canonical readiness evidence,
2. later packet boundaries depend on distinguishing normal proof from
   breakglass diagnostics.

Verification checkpoint:

1. official Lima shell/SSH/port-forwarding semantics are cited for non-obvious
   decisions,
2. the happy-path readiness order is explicit,
3. the exact contradiction surfaces are named explicitly:
   - `scripts/mac/lima-doctor.sh`
   - `scripts/mac/smoke.sh`
   - `docs/WORLD.md`
   - `docs/reference/world/platforms/macos-lima-setup.md`
   - `crates/shell/src/execution/platform/macos.rs` only if code clarity is
     needed

### Packet 2: Align the helper scripts to routed-path-first proof

Goal:

1. make `scripts/mac/lima-doctor.sh` prove routed readiness first,
2. make `scripts/mac/smoke.sh` prove routed doctor/gateway/PTY truth before
   guest-direct diagnosis,
3. preserve guest-direct steps only as explicit fallback or breakglass.

Why second:

1. helper scripts are the most direct executable expression of the readiness
   story,
2. docs should reflect the script truth that actually exists after Packet `2`.

Verification checkpoint:

1. `bash -n` passes for both scripts,
2. routed readiness failure can no longer be masked by guest-direct success in
   the intended proof flow,
3. guest-direct commands are clearly labeled fallback/breakglass in comments or
   structure.

### Packet 3: Cut over readiness-oriented docs and optional doctor clarity

Goal:

1. update the user-facing readiness docs to match the routed-path-first script
   truth,
2. adjust owned doctor output only if the scripts/docs need clearer routed
   evidence language.

Why third:

1. docs should be rewritten against the final script truth, not ahead of it,
2. code should stay untouched unless the doc/script cutover proves that the
   owned doctor surfaces are still too explanation-hostile.

Verification checkpoint:

1. readiness-oriented doc sections lead with CLI doctor/gateway commands,
2. direct guest instructions are clearly classified as breakglass or
   post-failure diagnosis,
3. if `platform/macos.rs` changes, GitNexus impact analysis was run before
   editing the touched symbol(s).

### Packet 4: Final validation and next-slice handoff clarity

Goal:

1. validate that Slice `06` stayed narrow,
2. run final targeted verification for scripts, docs, and optional doctor
   output,
3. leave a clean handoff to Slice `07` and the later broader docs-cutover work.

Why last:

1. this slice only earns Phase `1.3` credibility if the final evidence shows
   the happy path is routed-first everywhere the user will actually read or run
   it,
2. the handoff needs to make it explicit that listener removal and broader
   breakglass/docs cutover remain later work.

Verification checkpoint:

1. targeted shell tests stay green,
2. script syntax checks pass,
3. doc diffs are limited to readiness-oriented surfaces,
4. the final closeout explicitly states what remains deferred to Slice `07`
   and Slice `12`.

## Major risks and mitigations

### Risk 1: The slice quietly widens into full operator-lifecycle redesign

Mitigation:

1. keep `lima-warm.sh` and broader lifecycle surfaces out of default scope,
2. treat any need for new owned lifecycle commands as later-slice pressure, not
   as a reason to silently widen Slice `06`.

### Risk 2: Scripts still need guest probes for post-failure diagnosis

Mitigation:

1. preserve those probes,
2. move them behind routed failure or clearly label them breakglass,
3. make success/failure semantics depend on routed proof first.

### Risk 3: Docs overclaim Linux-equivalent trust or conceal override semantics

Mitigation:

1. keep same-user limitations explicit,
2. keep `SUBSTRATE_WORLD_SOCKET` classified as advanced/test/breakglass,
3. avoid teaching raw guest access as a degraded-but-supported middle tier.
