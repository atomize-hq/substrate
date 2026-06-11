# Design Docs: `macos-hardened-same-user-lima`

Status: draft index  
Last updated: 2026-06-11

## Purpose

Index the cross-slice design contracts for the macOS hardened same-user Lima
landing work.

These docs are intentionally narrower than the phase/milestone program docs and
broader than any single future `SPEC-*`. They exist so later slices can build
on shared decisions instead of re-arguing the same contract in each spec.

## Design set

1. [DESIGN-supported-mode-and-breakglass-taxonomy.md](./DESIGN-supported-mode-and-breakglass-taxonomy.md)
   - freezes the support taxonomy and same-user support posture
2. [DESIGN-macos-lima-transport-contract.md](./DESIGN-macos-lima-transport-contract.md)
   - freezes the canonical guest endpoint and host-to-guest transport story
3. [DESIGN-macos-policy-input-parity.md](./DESIGN-macos-policy-input-parity.md)
   - freezes how backend-mediated Lima execution must consume policy/world
     inputs already resolved by the shell
4. [DESIGN-macos-ingress-and-mount-contract.md](./DESIGN-macos-ingress-and-mount-contract.md)
   - freezes the ingress classes and narrowed mount direction
5. [DESIGN-macos-guest-unit-source-of-truth.md](./DESIGN-macos-guest-unit-source-of-truth.md)
   - freezes one authoritative guest unit definition strategy
6. [DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md](./DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
   - freezes the operator contract for CLI-first lifecycle, diagnostics, and
     evidence surfaces

## Use with future specs

Future `SPEC-*` docs should:

1. cite the exact `DESIGN-*` files they depend on,
2. reuse frozen terminology from those docs,
3. widen a design only through an explicit design update rather than ad hoc
   slice-local wording.
