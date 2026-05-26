# Substrate Gateway Operator Contract

This document is the durable operator contract for the gateway boundary.
It is the canonical descriptive reference for the named operator contract.
It points at the owned contract refs for machine-readable status and policy evaluation.

## Contract

The operator command family is:

- `substrate world gateway sync`
- `substrate world gateway status`
- `substrate world gateway restart`
- `substrate world gateway status --json`

Rules:

- `substrate world gateway status --json` is the authoritative machine-readable wiring surface.
- Human-readable `substrate world gateway status` may abbreviate details, but it must not redefine the JSON wiring meaning.
- `substrate world gateway status` may legitimately return unavailable before the first successful `sync`/`restart`, even when config/policy permit gateway lifecycle.
- `substrate world gateway status` is observational only: it reports the current managed runtime posture and does not launch the gateway or wait for readiness.
- When `substrate world gateway sync` or `restart` reports a managed runtime log path to inspect, that path is part of the Substrate-owned operator experience and must be inspectable by authorized operators without ad hoc root-only recovery steps.
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

## Boundaries

- This document does not define the `status --json` field list; that contract is owned by `docs/contracts/gateway/status-schema.md`.
- This document does not define policy decision tables or trust-boundary logic; that contract is owned by `docs/contracts/gateway/policy-evaluation.md`.
- This document does not define runtime transport, endpoint shapes, or parity details.
- The later-slice proof surfaces for this contract are:
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/builtins/mod.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/USAGE.md`
- Those surfaces are publication and verification targets for later work, not implementation targets for this contract.
