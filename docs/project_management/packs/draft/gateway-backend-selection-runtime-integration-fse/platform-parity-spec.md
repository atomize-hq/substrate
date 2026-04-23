# gateway-backend-selection-runtime-integration - platform parity spec

This slice-local spec consumes the canonical runtime parity contract and the automated parity matrix from slice 1 as upstream inputs.
It does not restate the supported-backend matrix, and it does not publish rollout or compatibility promises.

## Contract boundary

Owned here:

- Linux, macOS, and Windows evidence expectations for the gateway lifecycle surface.
- Manual/smoke validation shape for explicit unsupported-backend visibility.
- Hidden transport differences that stay underneath one operator-facing lifecycle/status contract.

Not owned here:

- Backend selection or policy rules.
- The canonical runtime parity contract.
- Rollout guidance, compatibility messaging, or support promises.
- Operator command names, status schema widening, or auth payload semantics.

## Upstream inputs

The evidence surfaces in this slice consume, rather than redefine:

- `docs/contracts/substrate-gateway-runtime-parity.md`
- the automated parity matrix established in slice 1

That upstream proof already covers the `cli:codex` regression floor and the landed `api:openai` parity path.
This slice only adds platform-readable evidence and explicit negative-case visibility.

## Platform evidence matrix

| Platform | Evidence obligation | Allowed hidden divergence | Verification anchor |
| --- | --- | --- | --- |
| Linux | Run the manual and smoke validation against the unsupported-backend negative case and confirm the operator-facing failure remains explicit. | Direct Unix socket transport and socket activation may differ under the hood. | Canonical runtime parity contract plus the Linux smoke script and manual playbook. |
| macOS | Run the manual and smoke validation against the unsupported-backend negative case and confirm the operator-facing failure remains explicit. | Lima-backed forwarding and guest transport may differ under the hood. | Canonical runtime parity contract plus the macOS smoke script and manual playbook. |
| Windows | Run the manual and smoke validation against the unsupported-backend negative case and confirm the operator-facing failure remains explicit. | WSL-backed transport and bridge mechanics may differ under the hood. | Canonical runtime parity contract plus the Windows smoke script and manual playbook. |

## Unsupported-backend visibility

The only explicit negative case this slice owns is a selected backend that is not backed by inventory.

Rules:

- The failure must remain an invalid-integration error.
- The failure must happen before gateway dispatch, so there is no silent fallback to `cli:codex`.
- The same visible failure class must hold for `status`, `sync`, and `restart`.
- Platform transport differences are verification details only and must not alter the failure class.

## Acceptance criteria

- Linux, macOS, and Windows all have the same negative-case evidence shape.
- The manual and smoke surfaces point back to the canonical runtime parity contract and the slice-1 automated matrix.
- Unsupported backends remain explicit failures with no fallback language in the evidence surfaces.
- The spec stays evidence-only and does not introduce rollout or compatibility claims.
