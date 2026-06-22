# Plan: World Codex Direct-Member Bootstrap Compatibility Bridge Completion

Source spec: [SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)  
Related architectural inputs:
- [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md)
- [DESIGN-codex-world-home-auth-and-config-mapping.md](./DESIGN-codex-world-home-auth-and-config-mapping.md)
- [DESIGN-workspace-scoped-adapter-overlay-model.md](./DESIGN-workspace-scoped-adapter-overlay-model.md)  
Plan type: bounded transitional bootstrap repair  
Status: draft for review  
Implementation posture: complete the direct world-member compatibility bridge without widening into generic projection or future capability surfaces

## Objective

Repair the current direct `cli:codex-world` member bootstrap gap so the existing transitional path becomes truthful enough to run on the diagnosed June 21, 2026 machine profile, while still staying visibly transitional and narrow.

This plan must land:

1. a pinned compatibility-bridge contract in tests,
2. bounded non-secret bootstrap config materialization into isolated `CODEX_HOME`,
3. fail-closed behavior when the bridge cannot derive truthful startup config,
4. docs and implementation guardrails that prevent future MCP/app-runtime/skills work from piggybacking on this bridge,
5. rebuilt-runtime smoke proof that the original world-dispatch flow now succeeds.

## Plan Summary

The repo already has the first half of the compatibility bridge:

1. shell injects a runtime-internal seed-home hint only when exact-backend policy allows host reads,
2. `world-service` materializes an isolated `CODEX_HOME`,
3. auth seeding proved useful but incomplete.

The remaining bug is not “no auth,” and it should not be fixed by broadening the bridge into “copy host `.codex` and hope.”

So the correct sequence is:

1. pin the bridge boundary in tests,
2. add the smallest non-secret bootstrap config rendering needed for truthful model/provider startup,
3. fail closed if that narrow bootstrap subset cannot be derived,
4. update docs/guardrails so the slice is clearly transitional,
5. rebuild and prove the live smoke,
6. then stop before any generic projection, workspace-overlay, or future capability widening.

## Locked Decisions

### What changes

1. The direct world-member bridge stops being “auth only”; it becomes “auth plus bounded non-secret bootstrap config.”
2. The isolated `CODEX_HOME` receives a rendered compatibility `config.toml` containing only the narrow startup subset needed for truthful model/provider selection.
3. The bridge fails closed when it cannot derive that narrow startup subset instead of silently allowing Codex to fall back to an unsupported default model.
4. Docs and code comments explicitly describe the bridge as transitional.

### What does not change

1. The long-term target architecture is still gateway-front-door auth delivery through the in-world gateway.
2. Slice `59` runtime-realizability truth remains unchanged.
3. Exact backend allowlist semantics remain unchanged.
4. No general Codex config projection framework lands in this slice.
5. No MCP/app-runtime/skills/plugin/workspace-overlay projection or reconciliation lands in this slice.
6. No broader lane/retained-home persistence model lands in this slice.

## Implementation Order

### Phase 1: Freeze The Compatibility Bridge Boundary In Tests

Goal:

1. make the remaining bug and the desired bridge boundary explicit in automation,
2. prove the slice is not merely “copy one more file somehow,”
3. keep exact-backend gating pinned while implementation proceeds.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
3. focused world-service and shell tests

Required changes:

1. add/expand world-service coverage around isolated `CODEX_HOME` preparation so the bridge contract includes bounded config rendering/materialization in addition to auth seeding,
2. pin the rule that only the exact allowlisted backend may receive the internal seed-home hint,
3. if feasible, add an assertion that the bridge does not project broader config/state artifacts.

Verification checkpoint:

1. test coverage distinguishes auth-only materialization from bounded auth-plus-config materialization,
2. exact-backend shell gating remains green,
3. the failing direct path is pinned tightly enough that later widening cannot hide inside the same seam.

### Phase 2: Land Bounded Non-Secret Bootstrap Config Materialization

Goal:

1. preserve truthful direct Codex startup inside isolated `CODEX_HOME`,
2. keep the bridge narrow enough that it cannot become the substrate for future capability work,
3. keep the implementation internal to the direct member seam.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. minimal helper code adjacent to the direct member bootstrap seam
3. `crates/world-service/Cargo.toml` only if a minimal parsing/rendering dependency is truly needed

Required changes:

1. read the host-side Codex config only from the already policy-gated seed-home source,
2. derive the smallest non-secret bootstrap subset required for truthful model/provider startup,
3. render that subset into isolated `CODEX_HOME/config.toml`,
4. keep auth artifact materialization behavior intact,
5. do not project unrelated config domains or broader home state,
6. strip internal hints before final child spawn as the current bridge already intends.

Verification checkpoint:

1. the isolated direct-member home now contains bounded compatibility config instead of relying on Codex defaults,
2. broader config/state domains are not materialized,
3. the direct path remains internal and exact-backend-gated.

### Phase 2.5: Add Fail-Closed Diagnostics For Missing Bootstrap Truth

Goal:

1. avoid a silent fallback to unsupported defaults,
2. make future debugging explain the real missing input,
3. keep the bridge honest when the host seed-home lacks the needed startup subset.

Primary touch surface:

1. `crates/world-service/src/member_runtime.rs`
2. nearby tests that assert explanation-ready failure behavior

Required changes:

1. emit a direct explanation when bounded startup config cannot be derived,
2. keep that explanation scoped to the direct compatibility bridge rather than implying the gateway target architecture changed,
3. ensure the failure remains fail-closed and does not permit ambient default-model fallback.

Verification checkpoint:

1. missing bounded startup config produces an explanation-ready failure,
2. the implementation no longer degrades into ambiguous exit-1 behavior solely from unsupported model fallback,
3. no new public contract is invented around the internal hint env var.

### Phase 3: Add Guardrails, Docs, And Live Proof

Goal:

1. make the bridge visibly transitional in repo truth,
2. prove the real rebuilt runtime fixes the user-facing smoke,
3. stop the slice before future capability widening begins.

Primary touch surface:

1. `docs/USAGE.md`
2. narrowly scoped code comments near direct-member bootstrap logic if they improve future guardrails
3. rebuilt installed runtime and live smoke commands

Required changes:

1. update operator/developer docs to reflect that direct `cli:codex-world` member bootstrap currently depends on a bounded compatibility bridge,
2. state that the bridge is transitional and should retire when gateway-front-door realization lands,
3. rebuild/redeploy the installed runtime,
4. rerun the live smoke floor and the exact June 21, 2026 public bootstrap smoke.

Verification checkpoint:

1. docs no longer imply “auth seeding alone” is the whole direct bootstrap story,
2. the rebuilt runtime proves the smoke is fixed,
3. nothing in the docs or code encourages reuse of this bridge for MCP/app-runtime/skills/workspace-overlay work.

## Risks And Mitigations

### Risk 1: The slice quietly turns into whole-file config copying

Mitigation:

1. render a bounded startup subset instead of copying a whole `config.toml` blindly,
2. keep broad config domains explicitly out of scope,
3. test for narrow artifact materialization, not just for “some config file exists.”

### Risk 2: The slice creates a second steady-state authority plane

Mitigation:

1. preserve the bridge as compatibility-only in spec, docs, and comments,
2. keep auth authority conceptually separate from config projection,
3. keep retirement intent explicit and tied to later gateway-front-door realization.

### Risk 3: The slice passes locally but remains ambiguous in future debugging

Mitigation:

1. add fail-closed diagnostics for missing bounded startup truth,
2. keep the live smoke proof in the closeout wall,
3. preserve the June 21 handoffs as source authorities in the slice docs.

## Sequencing And Parallelism

1. Phase 1 must land before Phase 2 so the bridge boundary stays pinned.
2. Phase 2 must land before Phase 2.5 so diagnostics describe the real new contract.
3. Phase 2.5 must land before Phase 3 so the live proof is explanation-ready if it still fails.
4. This slice should remain largely sequential; there is little honest parallel work beyond minor docs drafting while code changes are in review.

## Review Checkpoints

After each phase:

1. confirm the scoped checkpoint is green,
2. confirm the slice has not widened into broader Codex config projection,
3. confirm the target architecture is still the gateway-front-door model rather than the compatibility bridge,
4. confirm future capability surfaces have not been smuggled into the bridge.
