# Design: macOS Lima Transport Contract

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The macOS hardening work needs one shared transport contract before the repo can
land bounded runtime parity specs. The current phase docs already identify the
transport drift, but a dedicated design is still needed to freeze:

1. the canonical guest endpoint,
2. the role of host-side transport adapters,
3. the meaning of stale port references and fallback paths,
4. the classification of transport bypasses.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-1-runtime-parity-foundation/README.md`](../../phase-1-runtime-parity-foundation/README.md)
2. [`../../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md`](../../phase-1-runtime-parity-foundation/milestone-1-1-transport-contract-unification-sow.md)
3. [`../../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md`](../../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md)

## Problem statement

How should the repo define the supported host-to-guest transport story so that:

1. PTY, non-PTY, doctor, and readiness logic all describe the same endpoint
   contract,
2. transport adapters do not masquerade as separate behavioral modes,
3. stale port constants and fallback probes do not define the supported path,
4. breakglass bypasses remain clearly classified as exceptional?

## Frozen direction

This design freezes the following:

1. **Canonical guest service endpoint**
   - `/run/substrate.sock` inside the Lima guest is the canonical service
     endpoint
2. **Host-side transport adapter model**
   - VSock, SSH-backed UDS forwarding, or other host-side reachability
     mechanisms are adapters to the same guest endpoint
   - they are not independent product modes
3. **Routed behavior parity goal**
   - PTY, non-PTY, readiness probing, and doctor reporting should consume the
     same transport-selection rules
4. **Fallback classification**
   - stale `7788` references must be removed
   - host TCP `17788` probes may survive temporarily only as compatibility or
     breakglass checks, not as the supported contract
   - guest raw TCP listeners such as `SUBSTRATE_AGENT_TCP_PORT=61337` are not
     part of the hardened default
5. **Bypass classification**
   - `SUBSTRATE_WORLD_SOCKET` remains advanced/test/breakglass on macOS, not
     the standard Lima-backed path

## Non-goals

This design does not:

1. choose the final forwarding implementation detail for every environment,
2. require a different virtualization backend,
3. redefine the supported-mode taxonomy from scratch,
4. specify the full doctor UX.

## Transport model

### Canonical service

All supported macOS Lima world traffic should be conceptually targeting:

1. guest `world-service`
2. guest Unix socket `/run/substrate.sock`

### Adapter layer

Transport implementation details may include:

1. host-to-guest forwarding setup
2. SSH config discovery
3. readiness probing
4. adapter recovery logic

Those details may vary internally, but they should all resolve to the same
guest-local contract.

### Behavioral consequence

If two call sites appear to rely on different service endpoints, that is drift,
not a supported dual-mode design.

## File and surface implications

This contract should guide future work across:

1. `crates/world-mac-lima/src/lib.rs`
2. `crates/world-mac-lima/src/forwarding.rs`
3. `crates/world-mac-lima/src/transport.rs`
4. `scripts/mac/lima-warm.sh`
5. `scripts/mac/lima-doctor.sh`
6. `scripts/mac/smoke.sh`
7. `docs/WORLD.md`
8. `docs/reference/world/platforms/macos-lima-setup.md`

## Questions future specs should answer

1. Which code paths still encode the `17788` versus `7788` disagreement?
2. Which PTY and non-PTY flows currently select transport differently?
3. Which readiness checks are still proving the wrong endpoint story?
4. What minimal compatibility probes, if any, should remain after the supported
   contract is unified?
