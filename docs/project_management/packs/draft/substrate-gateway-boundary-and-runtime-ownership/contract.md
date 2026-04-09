# substrate-gateway-boundary-and-runtime-ownership — contract surface

This file mirrors the committed operator contract in `docs/contracts/substrate-gateway-operator-contract.md`
and keeps the feature-local publication surface aligned to the live source of truth for the operator
boundary and the owned contract refs for `C-02` and `C-03`.

## Contract

The operator command family is:

- `substrate world gateway sync`
- `substrate world gateway status`
- `substrate world gateway restart`
- `substrate world gateway status --json`

Rules:

- `substrate world gateway status --json` is the authoritative machine-readable wiring surface.
- Human-readable `substrate world gateway status` may abbreviate details, but it must not redefine the JSON wiring meaning.
- The stable non-secret wiring env outputs remain:
  - `SUBSTRATE_LLM_OPENAI_BASE_URL`
  - `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`
- Those env values point to Substrate-managed gateway endpoints, not upstream provider endpoints.
- Those env values are the only stable non-secret wiring exports in scope here, and they are intended for in-world clients rather than direct host reachability.

Exit codes:

- `0`: success
- `2`: invalid configuration, invalid policy, or invalid integration state
- `3`: transient runtime failure
- `4`: required gateway or world component unavailable
- `5`: policy or safety failure

Exit `4` is the absent-state result for the gateway entrypoints. Do not collapse it into invalid integration, transient failure, or policy/safety failure.

Ownership split:

- Substrate owns policy evaluation, world placement, lifecycle control, host-to-world secret delivery, operator UX, and canonical tracing.
- `substrate-gateway` owns the in-world front door, provider/planner/executor internals, and normalized event generation.
- Gateway-local config files, admin mutation surfaces, and token persistence are not required Substrate contract surfaces.

## Publication and verification surfaces

Publication surfaces:

- this feature-local contract file
- the durable operator contract reference under `docs/contracts/`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- later-slice publication and verification surfaces:
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/builtins/mod.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/USAGE.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`

Verification surfaces:

- the operator contract must stay aligned with ADR-0040 and the committed operator contract reference
- downstream implementation work must preserve the command family, JSON authority rule, stable env semantics, exit taxonomy, and ownership split without widening this contract into schema or runtime details
- `C-02` is published through `docs/contracts/substrate-gateway-status-schema.md`
- `C-03` is published through `docs/contracts/substrate-gateway-policy-evaluation.md`
- the later-slice proof surfaces listed above are publication and verification targets for subsequent slices, not implementation targets for this slice

## Boundaries

- This contract does not define the `status --json` field list.
- This contract does not define policy decision tables or trust-boundary logic.
- This contract does not define runtime transport, endpoint shapes, or parity details.
