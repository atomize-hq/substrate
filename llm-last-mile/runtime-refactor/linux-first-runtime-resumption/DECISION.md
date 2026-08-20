# Decision — Linux-First Runtime-Refactor Resumption

## Status and effective date

- **Status:** accepted for active scheduling; documentation-only decision.
- **Effective date:** 2026-08-20.
- **Reentry gate:** `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` (closed by the selection below).
- **Active global implementation packet:** [`A1.3-LINUX-FIRST-PACKET.md`](A1.3-LINUX-FIRST-PACKET.md).
- **Supersedes:** the 2026-08-19 macOS developer-parity decision only where it made
  `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` the global product-work predecessor. It does not weaken,
  close, or authorize the macOS parity lane.

## Context

The 2026-08-19 reset correctly removed protected macOS lifecycle, Keychain, publisher, finalizer,
E03, and Windows work from the ordinary developer-install path. It also made macOS parity the sole
active gate, which unnecessarily blocked return to the remaining runtime-refactor control-plane
work. The product has already established Linux as the first adoption platform; macOS parity is a
separate platform-restoration concern, and Windows remains deferred.

## Decision

Resume runtime-refactor scheduling on the current product branch with Linux-first adoption. The
macOS developer-parity effort is a parallel, non-blocking platform lane. Its future work belongs in
the dedicated `feat/macos-dev-installer-parity` branch after a separate branch/worktree action; no
branch is created by this decision. Windows remains deferred until a later, separately authorized
scheduling decision after the runtime refactor is complete.

The sole global next step is the bounded
`AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` control-plane packet. It is a rebind-and-selection
packet, not product implementation authority. It must:

1. rebind the live product branch, ancestry, worktree/index, source baseline, current control-pack
   state, and the already-landed historical evidence it relies on;
2. select exactly one next runtime-refactor packet from live source and record its named owner,
   path and symbol fences, dependencies, Linux-first acceptance criteria, and exclusions; and
3. publish no runtime, installer, Lima, macOS, Windows, archive, or native-operation change.

Only the selected later packet may receive separate implementation authority. This decision did
not assume that A1.3, A1.4, or any other historical label was the next implementation packet until
the rebind was complete.

## Reentry selection (2026-08-20; controlling)

The reentry packet is closed as documentation/control-plane selection only. Its live branch,
ancestry, control-pack, historical-evidence, Linux-first, macOS-exclusion, and Windows-deferral
rebind selected **A1.3 — public/successor adoption and startup-result completion**. The complete
narrow implementation authority, path/symbol fence, Linux-first acceptance criteria, verification,
shared-surface rule, and exclusions are in
[`A1.3-LINUX-FIRST-PACKET.md`](A1.3-LINUX-FIRST-PACKET.md).

A1.3 is the active next implementation packet. This selection itself performed no runtime,
installer, Lima, macOS, Windows, archive, or native operation and does not claim A1.3 completion.

## macOS lane

`AUTHORITY_REQUIRED:MACOS_DEV_PARITY` remains the lane-local gate for the ordinary user-prefix
macOS developer corridor. It retains every existing admission, Attempt 4 quarantine, protected
architecture exclusion, Linux-preservation, native proof, and restoration requirement in
[`../macos-dev-parity/DECISION.md`](../macos-dev-parity/DECISION.md). Its implementation and native
closure are not predecessors of runtime-refactor work, and closing either lane does not authorize
the other's successor.

If either lane needs a shared script or shared runtime surface, that change requires an explicit
integration packet with exact ownership and proof that Linux behavior is preserved. Neither lane
may silently adopt work from the other merely because the Git histories have a common base.

## Windows disposition

Windows is untouched, incomplete where applicable, and deferred. It is not a prerequisite for the
the selected A1.3 packet, Linux-first adoption, or macOS parity. No Windows implementation,
native proof, cleanup, or completion claim is authorized here.

## Alternatives considered

### Keep macOS parity as the global predecessor

Rejected. It continues to couple routine runtime-refactor progress to a native platform-restoration
loop whose security-hardening work has deliberately been archived.

### Resume the archived R3 macOS/Windows sequence

Rejected. The protected lifecycle architecture remains archived reference material and requires a
separate production threat-model decision before any revival.

### Declare an implementation packet immediately

Rejected at the time of this scheduling decision. The current source and control pack first had to
be rebound so that the next packet was bounded by live ownership, dependencies, and Linux-first
proof rather than a historical label. That rebind later selected A1.3.

## Consequences and non-goals

- The current branch may return to the runtime-refactor control plane without waiting for native
  macOS proof.
- The macOS lane remains deliberately narrow and may proceed independently only under its own
  fresh authorization.
- This scheduling decision itself did not create a branch, modify product source, re-open Linux
  R3, or perform a Linux proof run. Its separately recorded reentry selection later activated the
  bounded A1.3 packet only.
- This decision does not dispatch Windows, protected macOS lifecycle revival, Keychain activity,
  Attempt 4 inspection or remediation, installer execution, Lima actions, or native operations.
