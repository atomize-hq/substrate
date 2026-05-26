# substrate-gateway-backend-adapter-contract - platform parity spec

This document defines the feature-local proof surface for cross-platform parity and runtime-boundary verification.
It is a consumer of `docs/contracts/gateway/runtime-parity.md`, not a second owner of that contract.
For this slice, the canonical runtime-parity contract remains read-only.
S1 does not require any edits to `docs/contracts/gateway/runtime-parity.md`.

## Contract boundary

Owned here:

- Linux, macOS, and Windows guarantees for adapter-backed execution when world execution is required.
- The bounded proof that the local parity surface stays aligned with the canonical runtime-parity contract.
- The allowed hidden divergence list for backend transport and bootstrap mechanics.
- The explicit no-second-control-plane rule for this feature-local parity proof.

Not owned here:

- The operator command family or exit taxonomy.
- The `status --json` envelope or `client_wiring.*` field family.
- Fail-closed policy evaluation or host-to-world secret delivery rules.
- Adapter protocol, adapter schema, or session-handle semantics.
- Direct edits to ADR-0040 or any other runtime-ownership ADR.

## Required platforms

- Behavior platforms: `linux`, `macos`, `windows`
- Validation platforms: `linux`, `macos`, `windows`

Windows parity includes WSL-backed runtime mechanics as hidden transport detail only.
WSL is not a separate operator-facing contract surface in this feature.

## Cross-platform guarantee matrix

| Platform | Guarantee | Allowed hidden divergence | Verification anchor |
| --- | --- | --- | --- |
| Linux | Adapter-backed execution stays inside the world boundary when world execution is required, and the stable backend-id / allowlist semantics remain unchanged. | Direct Unix socket transport to `/run/substrate.sock`, socket activation, and provisioning mechanics may differ under the hood. | `docs/contracts/gateway/runtime-parity.md` and this pack-local parity spec. |
| macOS | Adapter-backed execution stays inside the world boundary when world execution is required, and the stable backend-id / allowlist semantics remain unchanged. | Lima-backed forwarding, guest transport, and warm-up mechanics may differ under the hood. | `docs/contracts/gateway/runtime-parity.md` and this pack-local parity spec. |
| Windows | Adapter-backed execution stays inside the world boundary when world execution is required, and the stable backend-id / allowlist semantics remain unchanged. | WSL-backed transport, named-pipe or TCP bridge mechanics, and warm-up details may differ under the hood. | `docs/contracts/gateway/runtime-parity.md` and this pack-local parity spec. |

## Runtime-boundary proof

The proof obligation for this slice is to show that the pack-local parity surface mirrors the canonical runtime boundary rather than redefining it.

Rules:

- `docs/contracts/gateway/runtime-parity.md` remains the canonical runtime-boundary reference for this slice.
- This pack-local spec may narrow or restate the canonical boundary, but it must not widen it.
- Hidden transport and bootstrap differences stay evidence-only unless the canonical contract is explicitly changed upstream.
- No second Substrate control plane may be introduced through parity wording, validation wording, or platform-specific notes.
- ADR-0040 stays evidence-only basis for runtime ownership and is not a direct edit surface for this slice.

## Validation surfaces

The later validation slice may consume this spec alongside the canonical runtime-parity contract and the downstream checkpoint plan.

This spec does not define code behavior, but it does define the proof shape that later validation must preserve:

- one operator-facing lifecycle/status meaning across Linux, macOS, and Windows,
- one canonical runtime-boundary owner,
- one hidden divergence list per platform,
- no widening of the operator contract, status schema, or policy boundary.

## Acceptance criteria

- The Linux/macOS/Windows guarantee matrix is explicit and bounded.
- Windows parity is framed as hidden WSL-backed runtime mechanics, not a separate operator contract.
- The pack-local spec clearly consumes `docs/contracts/gateway/runtime-parity.md` as read-only canonical basis.
- The spec does not imply direct ADR-0040 edits.
- The spec does not introduce a second control plane or any new contract surface.
