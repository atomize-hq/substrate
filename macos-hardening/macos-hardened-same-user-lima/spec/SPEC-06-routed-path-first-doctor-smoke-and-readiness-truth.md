# Spec: Slice 06 Routed-Path-First Doctor, Smoke, and Readiness Truth

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-policy-input-parity.md`](./design/DESIGN-macos-policy-input-parity.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)
- [`PLAN-04.md`](./PLAN-04.md)
- [`TASKS-04.md`](./TASKS-04.md)
- [`SPEC-05-backend-policy-input-parity.md`](./SPEC-05-backend-policy-input-parity.md)
- [`PLAN-05.md`](./PLAN-05.md)
- [`TASKS-05.md`](./TASKS-05.md)

Required official source set for this slice:
- [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: make macOS readiness validation, smoke proof, and readiness-oriented
docs/scripts prove the routed Substrate path first, while preserving direct
guest entry only as explicit breakglass or post-failure diagnosis.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `05`, and Slice `06` is now
   the next honest seam in the local feature sequence.
2. Slice `04` and Slice `05` are the contract floor for this slice:
   - Slice `04` already converged the selected transport story for shell-side
     runtime consumers,
   - Slice `05` already removed backend-local policy synthesis so readiness
     evidence can now prove backend-mediated macOS behavior honestly.
3. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `06` remains the Phase `1.3`
   seam for routed-path-first doctor/smoke/readiness truth, while Slice `07`
   remains listener-surface hardening and Slice `12` remains broader
   breakglass/docs cutover.
4. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   happy-path readiness story still depends on current Lima transport, SSH, and
   shell semantics.
5. Live repo truth still shows the remaining readiness-story drift this slice
   should own:
   - `scripts/mac/lima-doctor.sh` currently validates host/guest health almost
     entirely through `limactl shell`, guest socket checks, guest `curl`, guest
     `systemctl`, and other guest-direct probes rather than leading with
     `substrate host doctor` or `substrate world doctor`,
   - `scripts/mac/smoke.sh` already proves gateway lifecycle and world-doctor
     flows, but still mixes that routed evidence with direct guest `curl`,
     binary-presence, and `systemctl` checks inside its readiness proof path,
   - `crates/shell/src/execution/platform/macos.rs` already tries selected
     host-visible transport first, but still falls back to guest-direct doctor
     and fallback probe collection through `limactl shell`,
   - `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`
     already mention CLI-first doctor/gateway surfaces, but still present too
     much direct guest `systemctl`, `curl`, journal, and shell usage as normal
     readiness or troubleshooting flow.
6. Gateway contract docs already freeze
   `substrate world gateway sync|status|restart` and
   `substrate world gateway status --json` as authoritative operator surfaces,
   so this slice should reuse that contract rather than inventing a new
   readiness API.
7. This slice may touch scripts and readiness-oriented docs by default, and it
   may touch `crates/shell/src/execution/platform/macos.rs` only if a small
   doctor/reporting adjustment is needed to make routed evidence clearer.
8. This slice should not widen into full warm/provision redesign, listener
   removal, ingress/mount narrowing, guest-unit sandboxing, or full feature
   docs cutover.

If any of these are wrong, correct them before implementation.

## Packet 1 live confirmation and frozen readiness-order decision

Packet `1` was re-grounded on 2026-06-12 against both the required official
Lima source set and the current repo truth before any Slice `06` script/doc
cutover work begins.

Official Lima source verification completed on 2026-06-12:

1. Lima’s `limactl shell` reference confirms that it executes in the instance
   and, by default, uses the first `ssh` executable found on the host `PATH`
   to connect. It also points directly at `LIMA_SHELLENV_ALLOW` and
   `LIMA_SHELLENV_BLOCK` for `--preserve-env` behavior. That means successful
   `limactl shell` access is guest reachability evidence, not proof that the
   routed Substrate readiness path is healthy.
2. Lima’s SSH usage docs confirm that direct SSH is a first-class
   interoperability path separate from `limactl shell`. That reinforces the
   rule that raw SSH success is a guest-access path, not the canonical
   Substrate readiness proof.
3. Lima’s port-forwarding docs confirm that localhost forwarding is adapter
   behavior with version-sensitive defaults (`SSH` in older releases, `GRPC`
   in later ones, including the `v1.0.0` → `v1.0.1` → `v1.1.0` default
   changes). They also confirm that SSH forwarding can run over AF_VSOCK on
   newer VZ/systemd combinations. That makes raw forwarded-port success
   adapter-sensitive and a poor replacement for the owned CLI doctor/gateway
   contract.
4. Lima’s environment-variable docs confirm the instance-selection and advanced
   shellenv knobs relevant to this slice: `LIMA_INSTANCE`,
   `LIMA_SHELLENV_ALLOW`, and `LIMA_SHELLENV_BLOCK`. Those remain advanced
   operator controls, not the normal macOS readiness story.
5. Lima’s breaking-changes docs confirm that the SSH port is no longer
   hard-coded to `60022` for the default instance. That further argues against
   freezing direct host/guest assumptions into the supported readiness path.

Live 2026-06-12 repo-truth confirmation for Packet `1`:

1. `scripts/mac/lima-doctor.sh` does not call `substrate host doctor` or
   `substrate world doctor` today. Its critical pass/fail checks still run
   through `limactl shell`, guest socket tests, guest `curl`, guest
   `systemctl`, and guest package/layout checks.
2. `scripts/mac/smoke.sh` already proves `substrate world gateway
   sync|status|restart` plus `substrate world doctor --json`, but its readiness
   proof still omits `substrate host doctor` and still mixes routed proof with
   direct guest `curl`, guest binary checks, and guest `systemctl` checks.
3. `docs/reference/world/platforms/macos-lima-setup.md` already says the CLI
   doctors are canonical once the VM is provisioned, but its setup, manual
   smoke, and troubleshooting flow still prominently teaches guest-direct
   `limactl shell`, guest `systemctl`, guest `curl`, and guest journal usage.
4. `docs/WORLD.md` already documents the Lima-backed routed architecture, but
   its macOS section still presents helper-script health checks, transport
   adapter detail, guest logs, and `SUBSTRATE_WORLD_SOCKET` override material
   close enough to the happy path that the readiness order remains blurry.
5. `docs/USAGE.md`, `docs/contracts/gateway/operator-contract.md`, and
   `docs/contracts/gateway/status-schema.md` already freeze the owned operator
   surfaces this slice should reuse: `substrate host doctor`,
   `substrate world doctor`, `substrate world gateway sync|status|restart`, and
   `substrate world gateway status --json`.
6. `crates/shell/src/execution/platform/macos.rs` already preserves a
   routed-first assessment shape:
   - `selected_host_visible_transports()` prefers the selected host-visible
     transport set, unless `SUBSTRATE_WORLD_SOCKET` overrides it outright,
   - `assess_world_service_reachability(...)` only sets guest-direct
     capabilities success after routed host-visible probes fail and guest
     `substrate-world-service` is active,
   - world-doctor report collection likewise tries host-visible transports
     first and only then falls back to guest-direct `limactl shell` + in-guest
     `curl`.

Frozen Packet `1` readiness-order decision:

1. for an already provisioned macOS backend, the happy-path readiness order is:
   - `substrate host doctor [--json]`
   - `substrate world doctor [--json]`
   - `substrate world gateway sync|status|restart` as the managed lifecycle
     and wiring proof
   - routed smoke assertions
   - guest-direct diagnosis only after routed failure or as clearly labeled
     breakglass
2. helper scripts and readiness-oriented docs must encode that order rather
   than redefine it,
3. direct guest `limactl shell`, raw in-guest `systemctl`, raw in-guest
   `curl`, guest journal inspection, and similar commands remain exceptional
   diagnostics or breakglass, not the normal proof contract,
4. `SUBSTRATE_WORLD_SOCKET` remains advanced/test/breakglass on macOS and may
   not be restated as the default Lima readiness path.

## Objective

Make the normal macOS readiness story prove routed Substrate behavior first.

This slice is complete only when a reviewer can answer, without guessing:

1. which commands constitute the happy-path macOS readiness proof for an
   already provisioned backend,
2. whether `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` validate
   the routed Substrate path before any guest-direct success can hide a routed
   failure,
3. whether readiness-oriented docs lead with `substrate host doctor`,
   `substrate world doctor`, and gateway lifecycle/status surfaces,
4. which direct guest commands remain available only as breakglass or
   post-failure diagnosis,
5. whether `SUBSTRATE_WORLD_SOCKET` remains clearly classified as an
   advanced/test/breakglass override on macOS,
6. what still remains deferred to later hardening and wider docs-cutover
   slices.

## Frozen in this slice

This slice freezes only:

1. one routed-path-first readiness evidence order for macOS:
   - `substrate host doctor [--json]`
   - `substrate world doctor [--json]`
   - `substrate world gateway sync|status|restart`
   - routed smoke assertions
   - guest-direct diagnosis only after routed failure or as clearly labeled
     breakglass
2. the rule that helper scripts must not report success when routed Substrate
   readiness is unhealthy merely because direct guest probing succeeds,
3. the rule that readiness-oriented docs for an already provisioned backend
   must lead with owned CLI doctor/gateway surfaces rather than `limactl shell`
   or in-guest `curl`,
4. the rule that direct guest shelling, raw in-guest `systemctl`, raw in-guest
   `curl`, and similar commands remain exceptional readiness diagnostics rather
   than the normal contract,
5. the minimum doctor/reporting adjustment needed, if any, so scripts/docs can
   explain routed-versus-fallback evidence truthfully.

## Deferred by design

This slice intentionally does **not** freeze:

1. full feature-wide breakglass reclassification and broad docs cutover owned
   by Slice `12`,
2. lifecycle ownership redesign or warm/provision replacement work from later
   Phase `3` slices,
3. removal of every internal `limactl shell` use from helper scripts,
4. listener removal, ingress narrowing, mount hardening, guest-unit
   unification, or ownership-boundary changes from later slices,
5. backend transport redesign or policy-carrier redesign already handled by
   Slices `04` and `05`.

## Why this slice exists

Phase `1` is only honest if the repo’s readiness evidence proves the routed
contract it now claims to support.

Live repo truth shows the remaining gap clearly:

1. `scripts/mac/lima-doctor.sh` still looks like a guest-health script first
   and a Substrate-readiness script second.
2. `scripts/mac/smoke.sh` already exercises gateway lifecycle and routed doctor
   flows, but still uses direct guest probes as part of the same proof path.
3. `crates/shell/src/execution/platform/macos.rs` already preserves
   selected-transport-first behavior, but its guest-direct fallback paths still
   influence the practical readiness story unless scripts/docs are disciplined
   around them.
4. `docs/reference/world/platforms/macos-lima-setup.md` now says the CLI
   doctor commands are canonical, yet still teaches direct guest `systemctl`,
   direct guest `curl`, and guest journal usage prominently enough to remain
   the operator’s likely first move.
5. `docs/WORLD.md` still mixes routed guidance with override and guest-log
   recovery paths in a way that needs clearer readiness-path ordering.

If Slice `06` does not land now, Phase `1.3` remains only partially true: the
runtime may be more honest, but the proof and operator story are not.

## Source-driven grounding

This slice is source-driven because the claimed macOS happy path depends on
official Lima transport and VM-access semantics:

1. Lima documents `limactl shell` as shell execution in the instance and says
   it uses SSH by default to connect to the instance. That means direct
   `limactl shell` success is not the same thing as proving the routed
   Substrate transport is healthy. Source:
   [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
2. Lima documents direct SSH connectivity as a first-class interoperability
   path. That is useful operationally, but it reinforces that raw SSH access is
   a VM-access path, not itself proof that Substrate’s routed doctor/gateway
   contract is healthy. Source:
   [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
3. Lima documents automatic localhost port forwarding and notes that the
   default forwarder changed across versions (SSH in older releases, GRPC in
   later ones, with version-specific reversions). That makes raw forwarded-port
   success adapter-sensitive and a poor replacement for the owned CLI contract.
   Source:
   [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
4. Lima documents environment and instance-selection variables such as
   `LIMA_INSTANCE`, `LIMA_SHELLENV_ALLOW`, and `LIMA_SHELLENV_BLOCK`. Those are
   advanced operator knobs, so this slice should keep advanced override flows
   clearly separate from the normal readiness story. Source:
   [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
5. Lima’s breaking-changes page notes that the SSH port is no longer
   hard-coded to `60022` for the default instance. That reinforces the broader
   rule that brittle direct-host/guest assumptions should not define the normal
   supported readiness path. Source:
   [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

## Commands

This is a repo-first but source-driven readiness slice. Start by proving the
phase-1 authority stack, the remaining readiness-story drift, and the official
Lima semantics that justify a CLI-first cutover.

```bash
# Review the phase and neighboring slice authority
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-lima-transport-contract.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md

# Inspect the current readiness-story drift in repo truth
rg -n "host doctor|world doctor|gateway status|gateway sync|gateway restart|limactl shell|systemctl|curl|SUBSTRATE_WORLD_SOCKET" \
  crates/shell/src/execution/platform/macos.rs \
  scripts/mac/lima-doctor.sh \
  scripts/mac/smoke.sh \
  docs/WORLD.md \
  docs/reference/world/platforms/macos-lima-setup.md \
  docs/USAGE.md \
  docs/contracts/gateway/operator-contract.md \
  docs/contracts/gateway/status-schema.md

sed -n '1,220p' scripts/mac/lima-doctor.sh
sed -n '200,320p' scripts/mac/smoke.sh
sed -n '1,220p' crates/shell/src/execution/platform/macos.rs
sed -n '180,260p' docs/reference/world/platforms/macos-lima-setup.md

# If code symbols will change, run GitNexus before editing them
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context host_doctor_main --repo substrate --file crates/shell/src/execution/platform/macos.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context world_doctor_main --repo substrate --file crates/shell/src/execution/platform/macos.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/shell/src/execution/platform/macos.rs:host_doctor_main' --repo substrate --direction upstream --depth 3 --include-tests
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/shell/src/execution/platform/macos.rs:world_doctor_main' --repo substrate --direction upstream --depth 3 --include-tests

# Targeted validation surfaces for the implementation slice
bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh
cargo test -p shell doctor_ok_json -- --nocapture
cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture
cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture
cargo test -p shell macos_gateway_client_endpoint -- --nocapture
```

## Primary repo surfaces

Default in-scope surfaces:

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
11. `crates/shell/src/execution/platform/macos.rs` only if the owned doctor
    outputs need a small clarity improvement to support the script/docs cutover

Default out-of-scope surfaces:

1. `scripts/mac/lima-warm.sh`
2. `crates/world-mac-lima/`
3. `crates/world-api/`
4. ingress, mount, listener, or guest-unit hardening surfaces
5. broad feature docs outside readiness-oriented excerpts

## Success criteria

This slice is successful only when:

1. the happy-path readiness docs for an already provisioned macOS backend lead
   with routed Substrate commands,
2. `scripts/mac/lima-doctor.sh` fails when routed readiness is unhealthy even
   if guest-direct probes still succeed,
3. `scripts/mac/smoke.sh` proves routed PTY/non-PTY/doctor readiness and
   gateway lifecycle before any guest-direct diagnostic steps,
4. direct guest commands are clearly labeled breakglass or post-failure
   diagnostics,
5. same-user Lima is still described honestly as not providing the Linux
   ownership boundary,
6. the slice does not silently widen into later-phase lifecycle or hardening
   work.
