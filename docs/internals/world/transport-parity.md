# Transport Parity Architecture Sketch

Status: archived pre-hardening sketch
Last updated: 2026-06-22

## Why this file changed

This document previously described a dual-listener / `61337`-centric transport
model that no longer matches current repo truth for the hardened macOS
same-user Lima path.

Do **not** use this file as live transport authority.

## Current live authority

Use these instead:

1. `crates/world-mac-lima/src/transport.rs`
2. `crates/world-mac-lima/src/forwarding.rs`
3. `crates/shell/src/execution/platform_world/mod.rs`
4. `crates/shell/src/execution/platform/macos.rs`
5. `docs/WORLD.md`
6. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md`
7. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`

## Current truth summary

1. The canonical guest endpoint is `/run/substrate.sock`.
2. macOS transport constants are centralized in
   `crates/world-mac-lima/src/transport.rs`.
3. Host-visible `127.0.0.1:17788` is compatibility-facing naming and VSock
   host reachability when VSock forwarding is active; it is not proof of a
   guest raw TCP listener.
4. Automatic SSH TCP fallback is intentionally skipped when the guest agent is
   UDS-only.
5. macOS doctor/readiness code still owns additional probing and breakglass
   fallback logic beyond the selected-transport authority in
   `platform_world/mod.rs`.

If a future session needs a fresh internal transport deep-dive, create a new
doc from current repo truth rather than reviving the older `61337` model.
