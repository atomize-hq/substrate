# WFGADAX1 — Routing fail-closed + exported env var rename (Spec)

## Goal
- Implement Appendix B routing fail-closed semantics and exported state env var rename.

## Acceptance criteria
- `world_fs.fail_closed.routing=true` combined with effective world disable hard errors before execution (exit `2`).
- Runtime routing failures under fail-closed map to exit `3` or `4` per Appendix B.
- `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING=1|0` is exported as output-only state.
- `SUBSTRATE_WORLD_REQUIRE_WORLD` is deleted.

