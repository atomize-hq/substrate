# macOS Hardened Same-User Lima

Status: current-state feature authority
Owner: Substrate world backend / macOS hardening
Last updated: 2026-06-22

## Purpose

Describe the **current** same-user Lima posture in HEAD for
`macos-hardened-same-user-lima`: what is landed, what remains
degraded-but-supported, what is breakglass only, and what is still not fully
productized.

This README is no longer a speculative landing brief. It is the feature-level
summary of the committed stack that landed across Slices `01` through `12`.

## Current HEAD summary

### Landed

The main hardening landing is real in committed code:

1. backend-mediated policy-input parity now fail-closes instead of synthesizing
   permissive defaults,
2. the default extra guest TCP listener was removed from the hardened macOS
   path,
3. the hardened default now stages the workspace into
   `/var/lib/substrate/staged-workspace/current` instead of depending on broad
   mounted host-home visibility,
4. canonical guest unit authority now lives under
   `scripts/mac/lima/units/`,
5. routed-path-first doctor, gateway, smoke, and orchestration proof is the
   normal macOS evidence stack.

### Still true, but not fully productized

1. `scripts/mac/lima-warm.sh` remains the current create/warm/repair plus
   staged-workspace copy wrapper, so the full lifecycle story is not yet
   reduced to a single polished CLI-only provisioning surface.
2. `scripts/mac/lima-doctor.sh` remains a deeper routed-first troubleshooting
   wrapper rather than a fully retired legacy helper.
3. `substrate workspace sync` exists and is world-gated, but it is **not** yet
   the frozen normal macOS same-user Lima sync/copy contract. The truthful
   interim path remains the staged-workspace flow behind `substrate world
   enable` / `scripts/mac/lima-warm.sh`.
4. transport constants are centralized, but shell transport selection and
   macOS doctor/readiness probing still retain intentionally split
   responsibilities rather than one fully collapsed shell-side transport
   authority.

## Support taxonomy

The canonical wording source remains
[`spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md).
Feature and operator docs should use only these labels:

1. `supported`
2. `degraded-but-supported`
3. `breakglass`

## Current operator posture

### supported

The normal same-user Lima operator path is:

1. `substrate host doctor [--json]`
2. `substrate world doctor [--json]`
3. `substrate world gateway sync|status|restart`
4. `substrate world enable` when provisioning is needed
5. `substrate world deps current sync` when guest dependency reconciliation is
   needed

### degraded-but-supported

These are still part of the truthful live contract, but they are not the
preferred polished day-to-day surface:

1. `scripts/mac/lima-warm.sh` for create/warm/repair plus staged-workspace
   copy
2. `scripts/mac/lima-doctor.sh` for routed-first deeper troubleshooting after
   the owned CLI proof
3. retained host compatibility references to `127.0.0.1:17788` only when they
   are reached through Substrate-owned transport selection; they are not a
   direct supported operator endpoint

### breakglass

These remain available for explicit debugging or recovery, not as the default
operator path:

1. raw `limactl shell`
2. plain SSH into the guest
3. direct guest `systemctl`
4. direct guest `journalctl`
5. direct guest socket `curl`
6. host-side `SUBSTRATE_WORLD_SOCKET` override use

## Same-user non-parity boundary

This feature still does **not** claim:

1. Linux-equivalent host-side ownership isolation,
2. a privilege boundary against the owning macOS host user,
3. that direct guest administration is the normal happy path,
4. that every macOS lifecycle/sync surface is fully productized.

The remaining non-parity point with Linux should be described as the
same-user Lima ownership model, not as extra default listeners, broad default
mounts, or ambiguous unit authority. Those earlier drifts are already closed in
HEAD.

## Current authority stack

Read these in order for live truth:

1. [`docs/WORLD.md`](../../docs/WORLD.md)
2. [`docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
3. [`docs/USAGE.md`](../../docs/USAGE.md)
4. [`spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
5. [`spec/README.md`](./spec/README.md)
6. the committed `SPEC-*`, `PLAN-*`, and `TASKS-*` stack under [`spec/`](./spec/)

`ROADMAP.md` is now a historical / retrospective map. It is no longer the live
execution authority for creating the spec scaffold because that scaffold already
exists.

## Phase summary

### Phase 0 — security contract and scope

Landed as the contract freeze for:

- supported / degraded-but-supported / breakglass taxonomy,
- same-user Lima non-parity wording,
- breakglass classification of guest-direct and override flows.

### Phase 1 — runtime parity foundation

Landed the transport, policy, and routed-readiness hardening seams, while
leaving two honest residual truths:

1. doctor/readiness transport logic is still partially split between
   `platform_world/mod.rs` and `platform/macos.rs`,
2. compatibility TCP naming and realizable forwarding are related but not
   identical concepts and must be documented that way.

### Phase 2 — same-user hardening

Landed:

1. default extra-listener removal,
2. staged-workspace ingress cutover,
3. canonical unit-template authority.

### Phase 3 — Substrate-owned operations

Landed the operator cutover framing:

1. owned CLI doctor/gateway/deps/enable surfaces are taught first,
2. helper scripts are classified as degraded-but-supported,
3. guest-direct flows are classified as breakglass.

What remains intentionally explicit is that the current sync/copy story is
still the staged-workspace interim contract, not yet `substrate workspace sync`
as the frozen normal macOS path.

## Related docs

- [Phase 0: Security Contract and Scope](./phase-0-security-contract-and-scope/README.md)
- [Phase 1: Runtime Parity Foundation](./phase-1-runtime-parity-foundation/README.md)
- [Phase 2: Same-User Hardening](./phase-2-same-user-hardening/README.md)
- [Phase 3: Substrate-Owned Operations](./phase-3-substrate-owned-operations/README.md)
- [Execution Rubric](./EXECUTION-RUBRIC.md)
- [Roadmap](./ROADMAP.md)
- [Spec README](./spec/README.md)
