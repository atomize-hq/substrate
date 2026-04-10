# impact_map — llm_gateway_in_world

Historical evidence only. This placeholder impact map preserves ADR-0023-era planning and does not define the current operator boundary.
The live operator contract is `docs/contracts/substrate-gateway-operator-contract.md`.

Primary components (anticipated):
- `crates/world-agent` (supervision/transport to in-world gateway)
- `crates/shell` (CLI: historical `substrate world status|sync gateway`, client wiring output, explain surfaces)
- `crates/broker` (policy gates: allowlist + fail-closed routing + require-approval)
- `crates/trace` / `crates/common` (structured events/spans)
- new gateway/manager crates (TBD by implementation plan)

Cross-ADR alignment:
- ADR-0027 (config/policy surfaces and precedence)
- ADR-0017 (structured event routing / attribution)
- ADR-0028 Phase 8 circle-back (trace classifications + correlation fields)
