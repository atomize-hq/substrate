# impact_map — llm_gateway_in_world

Placeholder impact map for ADR-0023. Will be expanded when Phase 4 planning is hardened.

Primary components (anticipated):
- `crates/world-agent` (supervision/transport to in-world gateway)
- `crates/shell` (CLI: `substrate llm *`, env output, explain surfaces)
- `crates/broker` (policy gates: allowlist + fail-closed routing + require-approval)
- `crates/trace` / `crates/common` (structured events/spans)
- new gateway/manager crates (TBD by implementation plan)

Cross-ADR alignment:
- ADR-0027 (config/policy surfaces and precedence)
- ADR-0017 (structured event routing / attribution)
- ADR-0028 Phase 8 circle-back (trace classifications + correlation fields)

