# Spec: Slice 07 Remove Default Extra Listener Surface

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md`](../phase-2-same-user-hardening/milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](./SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)
- [`PLAN-06.md`](./PLAN-06.md)
- [`TASKS-06.md`](./TASKS-06.md)
- [`../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)
- [`../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)

Required official source set for this slice:
- [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Lima SSH](https://lima-vm.io/docs/usage/ssh/)
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)
- conditional only if implementation changes socket-activation or unit-sandbox semantics rather than just the extra-listener default:
  - [systemd.socket](https://www.freedesktop.org/software/systemd/man/systemd.socket.html)
  - [systemd.exec](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: remove the default guest TCP listener posture from same-user Lima so
macOS hardening uses `/run/substrate.sock` as the only hardened default guest
listener while preserving any retained host-side compatibility routing as
clearly non-default compatibility or breakglass behavior.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `06`, and Slice `07` is now
   the next honest dependency-ordered seam in the local feature sequence.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `07` remains the Phase `2.1`
   listener-surface hardening seam, while Slice `08` / Slice `09` remain the
   ingress and mount seam, Slice `10` remains guest-unit source-of-truth and
   sandbox unification, and Slice `12` remains the broader breakglass/docs
   cutover seam.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   hardened listener story still depends on current Lima shell, SSH,
   port-forwarding, and environment-variable semantics.
4. Live repo truth still shows the concrete remaining listener drift this slice
   should own:
   - `scripts/mac/lima-warm.sh` still writes
     `Environment=SUBSTRATE_AGENT_TCP_PORT=61337` into the guest
     `substrate-world-service.service` unit,
   - `scripts/mac/lima/substrate.yaml` already defines socket activation with
     `ListenStream=/run/substrate.sock` and does not itself require a guest TCP
     listener to describe the intended hardening contract,
   - `crates/world-service/src/lib.rs` still treats `SUBSTRATE_AGENT_TCP_PORT`
     as an opt-in switch that binds a loopback TCP listener only when that env
     var is supplied or when TCP listeners are inherited,
   - `crates/world-mac-lima/src/transport.rs` still exposes
     `127.0.0.1:17788` as a retained host-side compatibility endpoint even
     though the canonical guest service endpoint is `/run/substrate.sock`,
   - readiness and setup docs still contain wording that can blur guest TCP
     listener removal with host compatibility routing or breakglass guest
     access.
5. Slice `06` already froze the routed readiness proof order, so Slice `07`
   should not reopen the CLI-first readiness story; it should narrow the guest
   listener surface underneath that story.
6. This slice may touch the macOS warm/repair script, readiness evidence
   helpers, and listener-oriented docs by default, but it should not silently
   widen into mount minimization, guest-unit generation unification, or a broad
   transport redesign.
7. If implementation proves that a Rust symbol must change outside shell script
   and doc surfaces, GitNexus impact analysis is mandatory before the edit and
   `gitnexus_detect_changes()` is mandatory before committing.

If any of these are wrong, correct them before implementation.

## Packet 1 live confirmation and frozen listener decision

Packet `1` was re-grounded on 2026-06-12 against both the required official
Lima source set and current repo truth before any Slice `07` implementation
begins.

Official source verification completed on 2026-06-12:

1. Lima’s `limactl shell` reference says it executes in the instance and, by
   default, uses the first `ssh` executable found on the host `PATH` to
   connect. It also documents `LIMA_SHELLENV_ALLOW` and
   `LIMA_SHELLENV_BLOCK` as advanced controls for `--preserve-env`. That means
   successful `limactl shell` access is guest-reachability evidence, not proof
   that a guest TCP listener is part of the supported Substrate contract.
2. Lima’s SSH usage docs say plain SSH is a supported interoperability path for
   software that expects SSH connectivity. That reinforces that SSH reachability
   is an access path into the VM, not the definition of the hardened
   `world-service` listener contract.
3. Lima’s port-forwarding docs say localhost forwarding is adapter behavior and
   that the default forwarder has changed across versions (`SSH`, then `GRPC`,
   then `SSH`, then `GRPC`). That makes forwarded-port success version-sensitive
   and unsuitable as the stable definition of the hardened guest listener.
4. Lima’s environment-variable docs say `LIMA_INSTANCE` selects the instance,
   `LIMA_SHELLENV_ALLOW` / `LIMA_SHELLENV_BLOCK` control shell env propagation,
   `LIMA_SSH_OVER_VSOCK` is VZ/systemd-version-sensitive, and
   `LIMA_SSH_PORT_FORWARDER` has versioned defaults. Those are adapter and
   operator knobs, not the supported guest listener contract.
5. Lima’s breaking-changes docs say the SSH port is no longer hard-coded to
   `60022` for the default instance. That is another reason not to freeze the
   supported listener story around direct SSH or raw forwarded-port assumptions.

Live 2026-06-12 repo-truth confirmation for Packet `1`:

1. `scripts/mac/lima-warm.sh` still writes a guest service unit containing:
   - `Environment=SUBSTRATE_AGENT_TCP_PORT=61337`
   - `Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock`
   and therefore still widens the guest listener posture during warm/repair.
2. `scripts/mac/lima/substrate.yaml` already installs a guest socket unit with
   `ListenStream=/run/substrate.sock`, `SocketMode=0660`, and
   `SocketGroup=substrate`, which means the profile already reflects the
   socket-first direction more honestly than the warm/repair rewrite path.
3. `crates/world-service/src/lib.rs` only binds a loopback TCP listener when
   `SUBSTRATE_AGENT_TCP_PORT` is present or when TCP listeners are inherited.
   Without that env var and without inherited TCP listeners, it logs TCP as
   `disabled`.
4. `crates/world-mac-lima/src/transport.rs` still treats `/run/substrate.sock`
   as the canonical guest endpoint while retaining `127.0.0.1:17788` as a
   compatibility host endpoint.
5. `docs/reference/world/platforms/macos-lima-setup.md` still includes wording
   such as “TCP forwarding resets” that can be read as a normal runtime story
   even though the supported routed path is already CLI-first and the guest
   hardening direction is socket-first.
6. `docs/WORLD.md` still documents the macOS path in a way that can blur guest
   listener removal with host-side compatibility routing and breakglass guest
   diagnostics if the slice does not tighten that wording.

Frozen Packet `1` listener decision:

1. the hardened default guest listener on macOS is `/run/substrate.sock` only,
2. Slice `07` removes default injection of `SUBSTRATE_AGENT_TCP_PORT` from the
   macOS guest service path,
3. any retained host-side compatibility route such as `127.0.0.1:17788`
   remains compatibility or breakglass evidence only and must not be described
   as the guest listener contract,
4. direct `limactl shell` or raw SSH access remains guest access / breakglass,
   not the normal listener contract,
5. `SUBSTRATE_WORLD_SOCKET` remains advanced/test/breakglass on macOS and must
   not be restated as the default supported path,
6. unit source-of-truth unification is deferred; this slice owns the listener
   default removal, not the whole unit-generation architecture.

## Objective

Make same-user Lima’s default guest listener posture match the already-frozen
transport and readiness contract.

This slice is complete only when a reviewer can answer, without guessing:

1. whether the default macOS guest service path still enables a loopback TCP
   listener,
2. whether `/run/substrate.sock` is now the only hardened default guest
   listener,
3. whether retained host compatibility routing is clearly separated from the
   guest listener contract,
4. whether doctor/smoke evidence remains green without depending on guest TCP,
5. whether docs stop implying that guest TCP is part of the hardened default,
6. what work still remains deferred to Slice `08`, Slice `10`, and Slice `12`.

## Frozen in this slice

This slice freezes only:

1. the rule that same-user Lima must not inject
   `SUBSTRATE_AGENT_TCP_PORT=61337` by default in the guest service path,
2. the rule that `/run/substrate.sock` is the only hardened default guest
   listener on macOS,
3. the rule that retained host compatibility TCP routing is not equivalent to a
   supported guest TCP listener contract,
4. the minimum evidence and wording updates needed so scripts and docs describe
   the tightened listener posture honestly,
5. the minimum implementation needed to keep routed doctor/gateway/smoke flows
   green after the guest TCP default is removed.

## Deferred by design

This slice intentionally does **not** freeze:

1. mount minimization, ingress narrowing, or copy/sync redesign from Slice `08`
   and Slice `09`,
2. guest-unit source-of-truth unification and sandbox consolidation from Slice
   `10`,
3. a reopened transport adapter redesign or transport-selection rewrite,
4. broad lifecycle command redesign from Phase `3`,
5. full feature-wide breakglass reclassification and docs cutover from Slice
   `12`,
6. removal of all host compatibility TCP behavior if the repo still needs it as
   a temporary compatibility path outside the guest listener default.

## Why this slice exists

Phase `2.1` is only honest if the repo removes the default extra listener it
already says should not be part of the hardened same-user Lima posture.

Live repo truth shows the remaining gap clearly:

1. the profile-level Lima config already centers socket activation on
   `/run/substrate.sock`,
2. the macOS warm/repair path still reinstalls a service unit that enables a
   guest loopback TCP listener via `SUBSTRATE_AGENT_TCP_PORT=61337`,
3. shared `world-service` runtime code already supports a no-TCP mode when the
   env var is absent, so the remaining issue is default macOS provisioning
   posture rather than a cross-platform runtime limitation,
4. the host-side transport layer still retains compatibility TCP reachability,
   which must not be confused with the guest listener contract,
5. docs and helper evidence still need wording cleanup so the hardened default
   is not described as if guest TCP were normal.

If Slice `07` does not land now, Phase `2` continues with a hidden contradiction:
repo-level docs and design say the hardened path is socket-first, but the actual
warm/repair path still reinstalls a guest TCP listener by default.

## Source-driven grounding

This slice is source-driven because the claims it makes about listener posture
and fallback classification depend on official Lima semantics:

1. `limactl shell` and SSH are documented guest-access mechanisms, not proof of
   the `world-service` listener contract.
2. port forwarding defaults are documented as version-sensitive adapter
   behavior, so host forwarded-port success must not define the supported guest
   listener story.
3. Lima’s environment variables expose operator and adapter controls whose
   behavior changes across versions or VM modes; they are not timeless support
   contracts.
4. Lima’s breaking-changes docs explicitly warn against freezing stale SSH-port
   assumptions into current support claims.

Because of that, Slice `07` should continue to treat guest TCP as a removable
macOS-local default and treat host compatibility routing as explicitly
non-authoritative.

## Commands

Start by confirming the authority stack, the remaining listener drift, and the
specific files that still encode it.

```bash
# Review the phase-2 authority stack and adjacent slices
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md

# Confirm the listener drift still exists
rg -n "SUBSTRATE_AGENT_TCP_PORT|61337|SUBSTRATE_WORLD_SOCKET|ListenStream=/run/substrate.sock|17788" \
  scripts/mac/lima-warm.sh \
  scripts/mac/lima/substrate.yaml \
  scripts/mac/lima/substrate-dev.yaml \
  scripts/mac/lima/substrate-arch.yaml \
  crates/world-service/src/lib.rs \
  crates/world-mac-lima/src/transport.rs \
  docs/WORLD.md \
  docs/reference/world/platforms/macos-lima-setup.md

# Validate shell-script syntax before and after edits
bash -n scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh
```

If implementation needs a Rust-symbol change beyond scripts/docs, run the
required GitNexus impact analysis before editing the touched symbol and run
`gitnexus_detect_changes()` before committing.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md`
4. `scripts/mac/lima-warm.sh`
5. `scripts/mac/lima-doctor.sh` only if evidence or messaging must be adjusted
   to prove the tightened listener contract honestly
6. `scripts/mac/smoke.sh` only if evidence or breakglass labeling must be
   adjusted to prove the tightened listener contract honestly
7. `docs/WORLD.md`
8. `docs/reference/world/platforms/macos-lima-setup.md`
9. targeted tests or nearby script coverage only as needed

By default this slice should **not** widen into:

1. `scripts/mac/lima/substrate.yaml` / `substrate-dev.yaml` /
   `substrate-arch.yaml` template unification work,
2. `crates/world-mac-lima/` transport redesign,
3. ingress or mount-contract changes,
4. guest-unit sandbox/source-of-truth redesign,
5. broad feature-wide docs cutover.

## Acceptance criteria

This slice is ready for implementation closeout only when:

1. the macOS warm/repair path no longer injects
   `SUBSTRATE_AGENT_TCP_PORT=61337` by default,
2. a fresh or repaired guest can still satisfy routed doctor/gateway/smoke
   proof without relying on a guest TCP listener,
3. `/run/substrate.sock` is the only hardened default guest listener described
   by scripts and docs,
4. any retained host compatibility routing is clearly labeled compatibility or
   breakglass rather than normal default behavior,
5. the final diff stays listener-scoped and does not silently absorb the mount
   or unit-unification seams.
