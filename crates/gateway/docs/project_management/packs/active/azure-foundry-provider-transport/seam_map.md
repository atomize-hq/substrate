# Seam Map - Azure Foundry Provider Transport

This seam map extracts the remaining Azure runtime-provider work from the already-landed gateway foundation. It starts from the closeout-backed truth of `azure-kimi-claude-gateway` instead of reopening normalization, surface, policy, or boundary design.

Constraint posture:

- `C-02` through `C-06` from the upstream pack remain basis and are not re-owned here
- the current gap is the Azure runtime transport/config/live-verification path sitting below the landed public Anthropic surface and above the already-landed normalized event contract
- live Azure smoke testing is part of the planned delivery shape, not optional post-landing cleanup

## Horizon summary

- **Active seam**: `SEAM-2`
- **Next seam**: `null`
- **Future seams**: none extracted by default in this follow-on pack

## Seam roster

| Seam | Horizon / state | Type | Why this is a seam | Likely value | Touch surface | Verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` `azure-foundry-runtime-transport` | `future` / `landed` | `integration` | it owned the concrete Azure runtime contract instead of letting auth, URL shape, and deployment semantics leak through the generic OpenAI path | Azure is now a real first-class provider/runtime target for the landed gateway contracts | provider config schema, registry wiring, auth construction, config examples, runtime transport tests | landed `C-07`, runtime boundary, config/examples, and deterministic transport verification recorded in seam closeout |
| `SEAM-2` `azure-live-smoke-operator-readiness` | `active` / `exec-ready` | `conformance` | it turns the landed transport contract into an operator-usable live verification and troubleshooting path instead of leaving real-Azure proof implicit | a real operator can configure, smoke-test, and debug the gateway against Azure-hosted Kimi through the landed `/v1/messages` path | live smoke harness or procedure, redacted diagnostics, verification docs, troubleshooting surfaces, operator examples | redacted live Azure evidence showing think and default routes succeed and failure modes are understandable without reopening public or normalization contracts |

## Ordering rationale

1. `SEAM-1` landed first because Azure traffic previously flowed through a generic OpenAI-compatible transport path that hard-coded bearer auth and `/chat/completions`.
2. `SEAM-2` is now active because live smoke testing and operator troubleshooting only become reliable after the runtime transport contract is concrete, published, and backed by deterministic verification.

## Non-seams and pruned candidates

- A new normalization seam was rejected because `SEAM-2` of the upstream pack already landed `C-02`; this follow-on pack should only reopen that area if a fresh stale trigger emerges during Azure runtime validation.
- A new Anthropic surface seam was rejected because `/v1/messages` behavior is already frozen by `C-03`; Azure runtime work must consume that surface rather than redesign it.
- A new planner/executor seam was rejected because `C-04` already fixes the internal policy posture; this pack only needs to preserve the intended `Kimi-K2-Thinking` and `Kimi-K2.5` routing targets while making Azure transport real.
- A generic provider cleanup seam was rejected because the user asked for Azure Foundry support, not a broad refactor of every OpenAI-compatible provider.
