# Phase 2: Same-User Hardening

## Status

Draft

## Purpose / outcome

Tighten the current macOS Lima deployment so the same-user model exposes the smallest practical guest and host attack surface without pretending to provide the Linux ownership boundary. Phase 2 ends when the guest listener surface is reduced, host-to-guest mounts are explicitly minimized, and the guest service definition is unified behind one hardened source of truth.

This phase does not invent new operator surfaces. It hardens the already-supported Lima-backed path that today is exercised through `substrate host doctor`, `substrate world doctor`, `substrate world gateway sync|status|restart`, `scripts/mac/smoke.sh`, and `scripts/mac/orchestration-smoke.sh`.

## Why this phase exists

As of HEAD, macOS can match much of Linux's world behavior but not Linux's `root:substrate` ownership boundary because Lima still runs as the same host user that requested it. The remaining risk is not a single bug; it is a posture problem spread across a few concrete seams:

- `scripts/mac/lima-warm.sh` still writes `Environment=SUBSTRATE_AGENT_TCP_PORT=61337`, leaving a default extra listener path enabled in the guest.
- `scripts/mac/lima/substrate.yaml` still mounts broad host state, including a read-only `$HOME` mount that is wider than the hardened contract should require.
- Guest service definitions are still duplicated between the provisioning YAML and the warm script, which makes sandbox drift likely.
- Current docs still normalize direct in-guest setup and troubleshooting habits that are acceptable for breakglass but not for the hardened default.
- `SUBSTRATE_WORLD_SOCKET` still exists as an advanced/test/breakglass bypass and should stay outside the normal same-user Lima contract.

Phase 2 exists to reduce that spread before Phase 3 finishes the operator-surface cutover.

## In-scope

- Remove default extra listener paths on macOS/Lima where the hardened contract only requires `/run/substrate.sock`.
- Define and implement a reduced mount posture for the Lima guest, including explicit ingress requirements for source checkouts, auth material, and runtime artifacts.
- Unify macOS guest systemd unit generation so service sandboxing and socket-activation settings are defined once and consumed consistently.
- Tighten smoke, doctor, gateway, and orchestration evidence so same-user hardening regressions are visible.

## Out-of-scope

- Solving the fundamental same-user ownership limitation of Lima.
- Replacing Lima with a different macOS backend.
- Designing the final Substrate-owned lifecycle CLI surface; that belongs to Phase 3.
- Linux, WSL, or non-macOS world backend changes except where shared contracts must stay aligned.

## Architectural approach

Phase 2 keeps the same-user Lima architecture but narrows it in three successive steps:

1. Remove listener sprawl so `world-service` is consumed through the Unix socket and inherited socket-activation path by default.
2. Replace "mount broad host state and rely on convention" with an explicit ingress contract that names what the guest is allowed to see and why.
3. Collapse duplicated guest service definitions into one generated contract so hardening changes apply identically whether the VM is created from the profile or repaired by the warm script.

The core rule for the phase is that macOS should behave like Linux wherever behavior parity is possible, and should explicitly document the remaining non-parity point as same-user Lima ownership, not as a diffuse set of extra listeners, broad mounts, or doc-endorsed bypasses.

## Dependencies / sequencing

- This phase assumes the existing Lima backend remains the active macOS world backend.
- This phase depends on Phase 1 milestone 1.1 through milestone 1.3 being complete first, so transport, backend-policy, shared-world, and readiness contracts are stable before listener, mount, and unit hardening start encoding them.
- Milestone 2.1 should land before 2.3 because unit unification should encode the reduced listener surface, not preserve the old one.
- Milestone 2.2 should land before 2.3 if the sandbox `ReadWritePaths`, environment, or guest-visible paths depend on the new ingress contract.
- Phase 3 depends on this phase because CLI-owned lifecycle and diagnostics should not entrench today's wider listener, mount, or admin surface.

## Concrete repo surfaces and file pointers

- `scripts/mac/lima-warm.sh`
  - writes guest service and socket units
  - currently injects `SUBSTRATE_AGENT_TCP_PORT=61337`
  - currently repairs the VM through direct `limactl shell` administration
- `scripts/mac/lima/substrate.yaml`
  - defines current guest mounts
  - still includes broad host visibility
  - embeds a second copy of the `substrate-world-service` unit
- `crates/world-service/src/lib.rs`
  - binds the Unix socket by default
  - conditionally enables loopback TCP via `SUBSTRATE_AGENT_TCP_PORT`
  - already supports inherited socket-activation listeners
- `scripts/mac/lima-doctor.sh`
  - currently validates guest health through direct `limactl shell` probes more than the hardened default should
- `scripts/mac/smoke.sh`
  - exercises doctor/gateway/runtime parity and can remain the primary regression harness for the hardening work
- `scripts/mac/orchestration-smoke.sh`
  - already validates shared-owner/orchestration flows and should keep doing so through hardening
- `docs/WORLD.md`
  - documents transport selection and the current `SUBSTRATE_WORLD_SOCKET` bypass behavior
- `docs/cross-platform/mac_world_setup.md`
  - still teaches direct `limactl shell` setup, service, and troubleshooting flows too prominently

## Deliverables

- One phase packet that sequences the same-user hardening work into milestones 2.1 through 2.3.
- A hardened listener contract for macOS that removes default TCP exposure.
- A mount and ingress contract for the Lima guest that is narrower than "broad `$HOME` plus project convenience mounts."
- One source of truth for guest `substrate-world-service` unit and socket definitions.
- Updated evidence expectations for doctor, gateway, smoke, orchestration, and doc review.

## Acceptance criteria

- Every default macOS/Lima startup path uses the Unix socket contract without requiring `SUBSTRATE_AGENT_TCP_PORT`.
- The Lima profile and repair flow no longer mount broad host `$HOME` state by default unless a narrowly justified exception is explicitly documented and tested.
- There is one authoritative guest service/unit definition path for macOS hardening-critical settings.
- `SUBSTRATE_WORLD_SOCKET` remains available only as an advanced/test/breakglass override, not as the documented standard path.
- A reviewer can identify the remaining non-parity point with Linux as same-user Lima ownership, not as a diffuse set of extra listeners, broad mounts, or doc-endorsed manual bypasses.

## Validation / evidence plan

- Diff and review the rendered or generated guest unit contents to prove the TCP listener default is gone and the sandbox settings are unified.
- Capture `scripts/mac/lima-warm.sh --check-only`, `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, and `scripts/mac/orchestration-smoke.sh` evidence before and after the changes.
- Probe `world-service` listener mode through logs or explicit checks so the evidence distinguishes socket-activation-only operation from TCP-enabled operation.
- Review `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` for direct-admin guidance that still contradicts the hardened default.

## Risks / open questions

- Some host-to-guest flows may currently rely on the broad `$HOME` mount in ways that are not yet explicit, especially auth or toolchain bootstrap paths.
- Removing the default TCP listener may expose hidden forwarding or gateway assumptions on macOS.
- Unit unification can fail if the repo still needs a bootstrap profile unit and a repair unit; if that happens, the generated source of truth must still be singular.
- The exact ingress contract for auth files, SSH material, and package-manager caches remains to be frozen in milestone 2.2.

## Milestones

1. [milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md](./milestone-2-1-remove-extra-listeners-and-tighten-agent-surface-sow.md)
2. [milestone-2-2-mount-minimization-and-ingress-contract-sow.md](./milestone-2-2-mount-minimization-and-ingress-contract-sow.md)
3. [milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md](./milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)
