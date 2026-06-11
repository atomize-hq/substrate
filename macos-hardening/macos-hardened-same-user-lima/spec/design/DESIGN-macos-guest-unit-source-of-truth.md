# Design: macOS Guest Unit Source of Truth

Status: draft design input  
Last updated: 2026-06-11

## Why this doc exists

The phase docs already identify a hardening-critical drift point: the guest
`substrate-world-service` service and socket definitions are duplicated across
the Lima profile and the warm/repair script.

This design exists to freeze one architectural direction before a `SPEC-*`
tries to implement unit unification:

1. one authoritative source for hardening-critical unit contents,
2. one rendering/install path consumed by create and repair flows,
3. one contract for service/socket sandbox settings and runtime paths.

## Relationship to the phase docs

This design composes with:

1. [`../../phase-2-same-user-hardening/README.md`](../../phase-2-same-user-hardening/README.md)
2. [`../../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)
3. [`DESIGN-macos-lima-transport-contract.md`](./DESIGN-macos-lima-transport-contract.md)
4. [`DESIGN-macos-ingress-and-mount-contract.md`](./DESIGN-macos-ingress-and-mount-contract.md)

## Problem statement

How should the repo define guest unit authority so that:

1. creation and repair cannot silently recreate different service contracts,
2. hardening-critical listener, sandbox, runtime-path, and environment settings
   are defined once,
3. doctor, smoke, and docs can reason about one unit contract?

## Frozen direction

This design freezes the following:

1. **Single hardening authority**
   - the guest service and socket definition must have one authoritative source
     for hardening-critical content
2. **Shared rendered content**
   - VM create/bootstrap and warm/repair flows should consume the same rendered
     unit content or generation source
3. **Hardening-critical fields are not optional**
   - listener env vars
   - socket ownership/mode
   - runtime directory settings
   - service user/group
   - capabilities and capability bounding
   - writable/runtime paths
   - managed gateway runtime paths under
     `/run/substrate/substrate-gateway-runtime/`
4. **Reduced listener contract must be encoded here**
   - the authoritative unit source must encode the no-default-TCP hardening
     direction rather than preserve legacy injection by accident

## Non-goals

This design does not:

1. choose the exact code-generation mechanism yet,
2. require systemd replacement,
3. define the final CLI lifecycle UX,
4. force Linux and macOS unit installation to become identical beyond shared
   documentation where appropriate.

## Architectural options

Future specs may evaluate different implementation shapes, but must preserve
the frozen direction above. Possible shapes include:

1. checked-in canonical unit template rendered into both paths
2. generated units from one code or script source
3. one path treated as canonical and the other regenerated from it

The implementation mechanism is open; the single source of hardening truth is
not.

## File and surface implications

This design should guide future work across:

1. `scripts/mac/lima/substrate.yaml`
2. `scripts/mac/lima-warm.sh`
3. `scripts/mac/lima-doctor.sh`
4. `crates/world-service/src/lib.rs`
5. docs that currently quote or imply guest unit contents

## Questions future specs should answer

1. What exact unit fields currently drift between the YAML and warm script?
2. What generation path best fits the repo’s existing provisioning style?
3. How should doctor or smoke evidence prove the rendered units match the
   hardened contract?
