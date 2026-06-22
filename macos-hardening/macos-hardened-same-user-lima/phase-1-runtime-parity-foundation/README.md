# Phase 1: Runtime Parity Foundation

Status: landed with documented residual architecture splits
Last updated: 2026-06-22

## Purpose

Capture what Phase 1 already landed in HEAD and what truth still needs to be
described honestly.

This phase is no longer about inventing doctor/gateway support for macOS. That
support already exists and is part of the current operator contract.

## What Phase 1 landed

Phase 1 landed the runtime-parity seams that made the supported macOS path
credible:

1. transport constants and endpoint naming were converged around the canonical
   guest socket at `/run/substrate.sock`,
2. backend-mediated policy-input handling was tightened so the Lima-backed path
   no longer relies on permissive policy synthesis,
3. doctor/smoke/readiness proof moved to a routed-path-first posture.

## Residual truths that remain

Phase 1 is not “fully collapsed into one shell-side transport authority” in the
strongest original planning sense.

Current repo truth is:

1. `crates/world-mac-lima/src/transport.rs` is the authority for canonical
   guest socket constants, managed host UDS path, compatibility TCP naming, and
   transport descriptions,
2. `crates/shell/src/execution/platform_world/mod.rs` is the shell-side
   selected-transport authority for routed world execution,
3. `crates/shell/src/execution/platform/macos.rs` still owns macOS-specific
   doctor/readiness probing and breakglass fallback behavior.

That split is real and must be described honestly wherever Slice `04` is
referenced.

## 17788 semantics in current truth

The retained host loopback `127.0.0.1:17788` is:

1. a compatibility-facing host-side transport name,
2. the VSock host-visible endpoint when VSock forwarding is the selected path,
3. **not** proof of a guest raw TCP listener,
4. **not** proof that SSH TCP fallback exists by default.

Forwarding intentionally skips automatic SSH TCP fallback when the guest agent
remains UDS-only.

## Current downstream authority

For live runtime truth, read:

1. [`../../docs/WORLD.md`](../../docs/WORLD.md)
2. [`../../docs/reference/world/platforms/macos-lima-setup.md`](../../docs/reference/world/platforms/macos-lima-setup.md)
3. [`../spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md`](../spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md)
4. [`../spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](../spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)
5. [`../spec/SPEC-05-backend-policy-input-parity.md`](../spec/SPEC-05-backend-policy-input-parity.md)
6. [`../spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](../spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)
