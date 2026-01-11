# ADR-0010 — World Backend Contract + Capability Divergence Surfacing

## Status
- Status: Proposed
- Lifecycle: ***proposed*** → draft → review → refined → accepted
- Date (UTC): 2026-01-11
- Owner(s): Shell / World / Broker maintainers

## Scope
- This ADR defines a **cross-backend contract** for “world” backends (Linux host-native, Linux guest-rootfs, macOS/Lima, Windows/WSL, future Docker/Podman).
- It focuses on making backend capability divergence explicit while keeping the operator-facing contract stable.

## Related Docs
- Isolation support matrix (Linux vs macOS/Lima): `docs/ISOLATION_SUPPORT_MATRIX.md`
- World architecture + doctor: `docs/WORLD.md`
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- ADR standard/template: `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <intentionally omitted while Status=Proposed>

### Changes (operator-facing)
- Stable contracts across divergent backends
  - Existing: backend differences can be implicit, leading to confusion when a mode/feature behaves differently across platforms or degrades at runtime.
  - New: doctor and trace explicitly report what is supported, what was enforced, and whether any fallback occurred; CLI/exit-code semantics stay stable.
  - Why: users tolerate capability differences when they are visible and the top-level contract is stable.

## Problem / Context
- Substrate’s “world” concept spans multiple backend implementations and privilege postures.
- Capability divergence is inevitable (kernel features, platform constraints, guest vs host execution), but “silent divergence” undermines trust.
- We need a short, enforceable contract so future backends (e.g., Docker/Podman) can plug in without weakening security/UX guarantees.

## Goals
- Define a backend contract that keeps:
  - CLI surfaces stable,
  - exit codes stable and scriptable,
  - fail-closed vs degrade rules consistent,
  - protected-path and filesystem safety invariants consistent,
  - observability (doctor + trace) strong enough that differences are explicit.

## Non-Goals
- Designing or implementing a new backend (Docker/Podman, guest-rootfs, etc.).
- Changing policy schema/keys (unless explicitly required by a later draft).
- Defining a universal package-manager abstraction (apt/dnf/pacman); this ADR is about backend contracts, not tool provisioning.

## User Contract (Authoritative; Proposed)

### Doctor (pre-flight truth)
`substrate world doctor --json` SHOULD report:
- `world.backend.kind`: one of `linux_host_native | linux_guest_rootfs | macos_lima | windows_wsl | docker | podman | ...`
- `world.os`: best-effort identity of the in-world OS (`/etc/os-release`, arch, kernel) when applicable.
- Capability booleans (names TBD in draft) for at least:
  - `supports_isolation.workspace`
  - `supports_isolation.full`
  - `supports_fs_diff.non_pty`
  - `supports_fs_diff.pty`
  - `supports_system_packages_provisioning`
  - `supports_network_isolation`
- For each false capability, a structured reason and remediation hint.

### Trace (post-fact truth)
Each executed command span SHOULD include:
- `world.backend.kind`
- `world.in_world`: `true|false`
- `world.fs.isolation.requested` and `world.fs.isolation.effective`
- `world.fallback.reason` when any fallback occurred (including host fallback)
- `fs_diff.present` and a reason when absent (`unsupported | empty | error`)

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `4` is the canonical “unsupported/missing prerequisites” outcome for capability gaps.

## Backend Contract Checklist (Proposed)

1) **Identity + capability reporting (doctor)**
- Report backend kind, OS identity (when meaningful), and key capability booleans with remediation.

2) **Stable CLI + exit-code semantics**
- Same command surfaces across backends.
- Same exit code meanings (`0/2/3/4/5`) even when implementation differs.

3) **Consistent fail-closed vs degrade rules**
- If policy requests a guarantee (e.g., require world, full isolation, read-only), backend must enforce or fail closed.
- If fallback is allowed, fallback is explicit (one warning line) and trace-recorded.

4) **Filesystem safety invariants**
- Protected paths are never mutated (`.git`, `.substrate`, `.substrate-git`, sockets/dev nodes).
- `workspace` isolation at minimum prevents absolute-path project escapes (or fails closed when required).
- `full` isolation makes host paths not nameable (or fails closed when required).

5) **Observability invariants (trace)**
- Always record whether execution was in-world or host, isolation achieved, and fallback reasons.

6) **Provisioning invariants**
- Runtime sync/install MUST NOT mutate OS packages.
- OS-level package mutation MUST route through an explicit provisioning entry point (and be unsupported when it would mutate the host OS on platforms where the world backend runs on the host).

7) **Validation hooks**
- Each backend has a smoke script that asserts:
  - doctor capability fields,
  - a success path,
  - at least one unsupported path returns exit `4` with an actionable message substring,
  - trace annotations record strategy/fallback as expected.

## Sequencing / Dependencies
- This ADR is intended to become a standards anchor that other ADRs reference once accepted.
- No implementation work is permitted under Status=Proposed.

## Security / Safety Posture
- Primary posture: avoid “false guarantees” by making all backend capability gaps explicit (doctor) and all runtime fallbacks explicit (trace).

## Validation Plan (Authoritative; Proposed)
- Not applicable until Status transitions to Draft (this ADR is a contract proposal only).

## Rollout / Backwards Compatibility
- Not applicable until Status transitions to Draft (this ADR may introduce new doctor/trace fields, likely additive).

## Decision Summary
- None yet (proposal). A Draft should introduce a decision register if multiple non-trivial A/B choices are needed (field naming, minimum capability set, schema versioning strategy).

