# Execution Rubric: `macos-hardened-same-user-lima`

Status: draft execution control  
Last updated: 2026-06-11

## Purpose

Make the next-slice planning workflow for `macos-hardened-same-user-lima`
durable, visible, and easy to reuse from a short prompt.

This document exists so a future planning turn can be as short as:

> `SPEC-01 just landed. Refer to EXECUTION-RUBRIC.md and plan the next slice.`

and still reliably do all of the following:

1. identify the next honest slice from live repo truth,
2. identify which design docs must be reviewed first,
3. decide whether `spec-driven-development` alone is sufficient or whether
   `source-driven-development` is also mandatory,
4. identify the official sources that must be consulted,
5. produce the next bounded `SPEC-*`, `PLAN-*`, and `TASKS-*` set without
   widening scope silently.

## Relationship to the rest of this feature

Use this document together with:

1. [Feature overview](./README.md)
2. [Roadmap](./ROADMAP.md)
3. [Spec scaffold](./spec/README.md)
4. [Design docs](./spec/design/README.md)
5. [Research note](../research/2026-04-28-macos-lima-parity-lockdown.md)

This rubric does **not** replace the phase READMEs, milestone SOWs, or future
slice docs. It defines the execution gates between them.

## Core execution model

This feature should be run with two layers:

### Stable layer

These docs should change slowly and remain visible:

1. feature `README.md`
2. phase READMEs
3. milestone SOWs
4. `ROADMAP.md`
5. `EXECUTION-RUBRIC.md`
6. `spec/design/DESIGN-*`

### Fluid layer

These should be created one honest seam at a time:

1. next `SPEC-*`
2. next `PLAN-*`
3. next `TASKS-*`

Do **not** pre-write the full `SPEC-*` stack for the entire program. Use this
rubric to keep the overall sequence visible while still planning one slice at a
time.

## Required planning protocol for the next slice

When a future session is asked to plan the next slice, it should follow this
exact protocol.

### Step 1: Confirm the latest landed slice from live repo truth

Do not trust chat memory or the prompt’s implied number by itself.

Inspect:

1. `macos-hardening/macos-hardened-same-user-lima/spec/`
2. the newest landed `SPEC-*`, `PLAN-*`, and `TASKS-*`
3. any related closeout or review notes if they exist

If the latest landed slice is different from the prompt’s assumption, plan from
live repo truth.

### Step 2: Identify the next honest seam

Use these authorities in order:

1. the last landed `TASKS-*` closeout and any explicitly deferred work
2. the current phase/milestone dependency order
3. `ROADMAP.md`
4. the relevant `DESIGN-*` docs
5. live repo truth if any earlier planning artifact has gone stale

The next slice should be:

1. the smallest honest seam,
2. dependency-ordered,
3. small enough to support one bounded `SPEC-*`.

### Step 3: Apply the skill gate

Every future slice in this feature uses:

- `spec-driven-development`

Some slices also require:

- `source-driven-development`

Use the matrix below to decide whether source-driven validation is mandatory
before writing the next slice docs.

### Step 4: Apply the source gate

If `source-driven-development` is required for the seam:

1. detect which external semantics matter,
2. fetch the authoritative official docs,
3. cite the exact official sources in the resulting spec/plan/task docs when
   those semantics drive non-obvious decisions,
4. call out any version-sensitive claims explicitly.

### Step 5: Write the next `SPEC-*`, `PLAN-*`, and `TASKS-*`

The slice output should:

1. cite the relevant phase/milestone docs,
2. cite the relevant `DESIGN-*` docs,
3. cite official external sources when the source gate required them,
4. keep the seam narrow,
5. preserve explicit non-goals and boundaries.

## Short-prompt operating contract

If a user gives a short planning prompt such as:

- `SPEC-01 just landed. Refer to EXECUTION-RUBRIC.md and plan the next slice.`
- `TASKS-03 landed. Use the rubric and write the next spec/plan/tasks.`

the planning session should automatically do all of the following:

1. find the actual latest landed slice,
2. determine the next slice from this rubric and live repo truth,
3. determine whether `source-driven-development` is mandatory,
4. fetch the required official sources if needed,
5. use `spec-driven-development` to draft the next `SPEC-*`, `PLAN-*`, and
   `TASKS-*`.

That short prompt should be sufficient **unless** the repo truth shows a major
dependency inversion or the prior slice surfaced a blocking open question.

## Skill and source gate matrix

| Slice family / functionality | `spec-driven-development` | `source-driven-development` | Why | Required authority set |
| --- | --- | --- | --- | --- |
| Supported mode / support taxonomy | Required | Light / targeted | primarily repo and support contract work | repo docs, phase docs, research note |
| Lima version floor / breakglass contract | Required | Required | Lima semantics and version drift matter | repo docs, Lima official docs |
| Transport contract | Required | Required | forwarding and guest-endpoint claims are version-sensitive | repo docs, Lima official docs |
| Routed consumer parity for PTY/non-PTY/doctor/readiness | Required | Required | transport and reachability semantics matter | repo docs, Lima official docs |
| Backend policy input parity | Required | Light / targeted | mostly repo-truth propagation, but some contract validation may be needed | repo docs, targeted external docs only if needed |
| Ingress and mount contract | Required | Required | mount/default behavior must be official-source grounded | repo docs, Lima official docs |
| Listener removal and guest surface narrowing | Required | Required | transport/listener semantics are platform-sensitive | repo docs, Lima official docs, systemd docs if unit changes are involved |
| Guest unit source of truth and sandbox unification | Required | Required | socket activation and service sandbox semantics matter | repo docs, systemd official docs |
| Operator lifecycle and diagnostics contract | Required | Targeted | repo-first, but supported-path claims may depend on external semantics | repo docs, targeted Lima/systemd docs |
| Breakglass reclassification and docs cutover | Required | Targeted | wording is repo-first, but exceptional-path claims should stay source-backed | repo docs, targeted Lima docs |
| True ownership separation / daemon-owned macOS control plane | Required | Heavy / mandatory | Apple and Lima semantics are core to the design | repo docs, Apple official docs, Lima official docs |

## Source-driven trigger rules

`source-driven-development` is mandatory when **any** of the following are
true:

1. the slice depends on Lima version behavior,
2. the slice depends on default mounts, port forwarding, direct SSH, or
   `limactl shell` behavior,
3. the slice depends on systemd socket activation or sandbox semantics such as
   `ProtectHome=` or `ReadWritePaths=`,
4. the slice makes a security or support claim grounded in Apple daemon,
   ServiceManagement, or Virtualization behavior,
5. the slice proposes to rely on or remove a platform fallback whose behavior
   is documented externally rather than only in this repo.

`source-driven-development` is usually light or optional when:

1. the slice is mostly repo taxonomy or support classification,
2. the slice is reconciling already-landed repo behavior into a clearer
   planning authority,
3. external platform semantics are background context rather than decision
   drivers.

## Source authority map

When `source-driven-development` is required, use these official sources first.

### Lima

- [Environment Variables](https://lima-vm.io/docs/config/environment-variables/)
- [Usage: SSH](https://lima-vm.io/docs/usage/ssh/)
- [Reference: limactl shell](https://lima-vm.io/docs/reference/limactl_shell/)
- [Configuration guide](https://lima-vm.io/docs/config/)
- [VZ driver](https://lima-vm.io/docs/config/vmtype/vz/)
- [Port Forwarding](https://lima-vm.io/docs/config/port/)
- [Reference: limactl create](https://lima-vm.io/docs/reference/limactl_create/)
- [Breaking changes](https://lima-vm.io/docs/releases/breaking/)
- [FAQ](https://lima-vm.io/docs/faq/)
- [Internal data structure](https://lima-vm.io/docs/dev/internals/)

### systemd

- [systemd.socket](https://www.freedesktop.org/software/systemd/man/systemd.socket.html)
- [systemd.exec](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)

### Apple / macOS

- [launch_activate_socket](https://developer.apple.com/documentation/xpc/launch_activate_socket)
- [SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice)
- [VZVirtioFileSystemDevice](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

### Repo-local authority

For Substrate behavior, live repo truth remains primary:

1. `crates/world-mac-lima/`
2. `crates/world-service/`
3. `crates/shell/`
4. `scripts/mac/`
5. `docs/WORLD.md`
6. `docs/reference/world/platforms/macos-lima-setup.md`
7. relevant contract docs under `docs/contracts/`

## Slice readiness checklist

Before opening the next `SPEC-*`, confirm:

- [ ] the latest landed slice was confirmed from live repo truth
- [ ] the next seam is the smallest honest dependency-ordered slice
- [ ] the relevant `DESIGN-*` docs were reviewed
- [ ] the relevant phase and milestone docs were reviewed
- [ ] the source-driven gate was applied
- [ ] official docs were fetched if the seam requires them
- [ ] the slice is narrow enough to avoid absorbing multiple milestone-scale seams

## Slice closeout checklist

Before declaring a slice ready to hand off to implementation:

- [ ] `SPEC-*`, `PLAN-*`, and `TASKS-*` all exist
- [ ] assumptions are explicit at the top of the spec
- [ ] commands and verification surfaces are concrete
- [ ] boundaries and non-goals are explicit
- [ ] required external sources are cited when the source gate required them
- [ ] the next likely seam or explicit deferral is visible for the following planning turn

## Design-doc consumption map

Use this as the default crosswalk when selecting design inputs for the next
slice.

| Design doc | Primary slice families |
| --- | --- |
| `DESIGN-supported-mode-and-breakglass-taxonomy.md` | 01, 02, 11, 12 |
| `DESIGN-macos-lima-transport-contract.md` | 03, 04, 07 |
| `DESIGN-macos-policy-input-parity.md` | 05 |
| `DESIGN-macos-ingress-and-mount-contract.md` | 08, 09, 10, 11 |
| `DESIGN-macos-guest-unit-source-of-truth.md` | 10 |
| `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md` | 06, 09, 11, 12 |

## Current slice queue

This queue is the default planning order unless live repo truth forces a
different next seam.

1. Slice `01`: supported mode and support taxonomy
   - source gate: light / targeted
   - primary design docs:
     - `DESIGN-supported-mode-and-breakglass-taxonomy.md`
2. Slice `02`: Lima version floor and breakglass contract
   - source gate: required
   - primary design docs:
     - `DESIGN-supported-mode-and-breakglass-taxonomy.md`
3. Slice `03`: canonical guest endpoint and transport contract
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-lima-transport-contract.md`
4. Slice `04`: PTY, non-PTY, doctor, and readiness transport convergence
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-lima-transport-contract.md`
     - `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`
5. Slice `05`: backend policy input parity
   - source gate: light / targeted
   - primary design docs:
     - `DESIGN-macos-policy-input-parity.md`
6. Slice `06`: routed-path-first doctor/smoke/readiness truth
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`
     - `DESIGN-macos-lima-transport-contract.md`
7. Slice `07`: remove default extra listener surface
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-lima-transport-contract.md`
8. Slice `08`: ingress inventory and narrowed mount contract
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-ingress-and-mount-contract.md`
9. Slice `09`: ingress implementation and/or Substrate-managed sync path
   - source gate: required
   - primary design docs:
     - `DESIGN-macos-ingress-and-mount-contract.md`
     - `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`
10. Slice `10`: guest unit/service source of truth and sandbox unification
    - source gate: required
    - primary design docs:
      - `DESIGN-macos-guest-unit-source-of-truth.md`
      - `DESIGN-macos-ingress-and-mount-contract.md`
11. Slice `11`: Substrate-owned lifecycle and diagnostics contract
    - source gate: targeted
    - primary design docs:
      - `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`
      - `DESIGN-supported-mode-and-breakglass-taxonomy.md`
12. Slice `12`: breakglass reclassification and docs cutover
    - source gate: targeted
    - primary design docs:
      - `DESIGN-supported-mode-and-breakglass-taxonomy.md`
      - `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`

## Boundaries

### Always

1. Plan one honest slice at a time.
2. Keep the same-user limitation explicit.
3. Use live repo truth to validate earlier planning assumptions.
4. Use official external docs whenever the source gate requires them.

### Ask first

1. Renumbering slices after they start landing.
2. Collapsing multiple planned slices into one if the live seam is smaller than
   expected.
3. Introducing a true ownership-separated macOS architecture ahead of the
   earlier same-user hardening work unless repo truth clearly forces it.

### Never

1. Treat the phase READMEs or milestone SOWs as implementation-ready specs by
   themselves.
2. Skip the source gate on a slice whose behavior depends on Lima, systemd, or
   Apple platform semantics.
3. Let a short planning prompt justify skipping the design-doc or live-truth
   review steps.

## Related docs

- [Feature overview](./README.md)
- [Roadmap](./ROADMAP.md)
- [Spec scaffold](./spec/README.md)
- [Design docs](./spec/design/README.md)
- [Research note](../research/2026-04-28-macos-lima-parity-lockdown.md)
