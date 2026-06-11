# PLAN-54: Inventory-Selected Second Runtime-Family Host-Orchestrator Tool Surface Parity

Source spec: [SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)  
Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior slice: [PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)  
Plan type: second runtime-family parity above the first validated Codex-backed host-tool floor  
Status: landed and validated on `2026-06-11`

## Closeout outcome

Slice `54` is now the landed baseline for selected-host multi-family host-tool support:

1. selected `claude_code` host starts/turns reuse the same authoritative toolbox env plus prompt-contract disclosure path as the first validated family floor,
2. selected-host reporting surfaces can publish `claude_code` as `smoke_validated` / `selected_runtime_supported` without overclaiming inventory-entry or world-scope support,
3. remaining deferred work is later docs/smoke follow-through after future live-surface widening, plus the broader caller-surface, cross-platform, Family-2, and governance tracks already called out below.

## Objective

Land truthful `claude_code` host-tool parity on top of the already-landed Slice `52` contract freeze and Slice `53` first-family landing.

This slice is complete only when:

1. dynamic orchestrator selection and exact backend policy truth remain unchanged,
2. selected `claude_code` host starts and turns receive the same authoritative toolbox env + prompt-contract exposure model as the first validated family floor,
3. live `claude_code` tool calls preserve Slice `52` semantics end-to-end,
4. worker messaging/lifecycle/steering semantics remain shared,
5. reporting truth upgrades only as far as validation evidence proves,
6. the slice lands without public toolbox verbs, MCP-first redesign, transport redesign, or hidden runtime fallback.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md) has been reviewed and accepted before implementation starts.

## Tracker Update Rule

The canonical running ledger for drift, deferrals, and sequence changes remains:

- [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)

The broader v1 gap tracker that should stay aligned with that remaining-scope truth is:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)

During Packet `1`-`4` execution or review:

1. add new drift or sequencing discoveries to `## Newly Surfaced During Execution` in the remaining-scope note,
2. move intentional non-fixes into `## Deferred / Circle-Back Items`,
3. move reconciled items into `## Resolved Since Last Update`,
4. keep the gap matrix aligned to the same slice-order truth and fix stale references only if that work remains bounded to this slice.

## Repo-Truth Framing

What is already landed:

1. dynamic orchestrator selection from effective config and inventory,
2. exact backend allowlisting and runtime-family realization from inventory truth,
3. the frozen seven-tool Slice `52` contract,
4. the first live Codex-backed host-tool floor from Slice `53`,
5. the landed internal toolbox transport and world-dispatch runtime,
6. truthful non-Codex support-posture reporting that still says `claude_code` parity is not yet proven.

What is still missing:

1. selected `claude_code` host starts/turns taking the same authoritative live host-tool path,
2. end-to-end `claude_code` proof that tool calls preserve Slice `52` semantics,
3. support-posture uplift backed by actual validation instead of by planning prose.

## Locked Decisions

### What this slice changes

1. It lands the second runtime-family parity seam for selected `claude_code` host orchestrators.
2. It reuses the same env-injection plus startup/system-prompt contract-disclosure path already established for the first family floor.
3. It upgrades support/reporting posture only when parity is actually proven.
4. It keeps worker messaging, lifecycle, and steering semantics shared across runtime families.

### What this slice does not change

1. no hard-coded `codex` orchestrator id,
2. no hidden fallback from `claude_code` to `codex`,
3. no public human toolbox execution CLI,
4. no MCP-first transport redesign,
5. no Family-2 host-global ingress reopening,
6. no semantic rewrite of Slice `52` receipts or follow-up handles.

## Major Components And Dependencies

1. **support-posture and parity gating truth**
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs` (only if needed)
   - `crates/shell/src/execution/agents_cmd.rs`
   - owns whether the repo still says `claude_code` is not-yet-validated or has achieved parity

2. **selected host start/turn parity seam**
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/prompt_fulfillment.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - likely the narrowest seam for making selected `claude_code` starts and turns take the same authoritative host-tool path

3. **frozen tool invocation semantics**
   - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
   - remains the authority for tool names, runtime-owned field injection, and receipt/result normalization

4. **internal toolbox runtime**
   - `crates/shell/src/repl/async_repl.rs`
   - already landed; Slice `54` must consume it rather than redesign it

5. **parity regression and operator-truth coverage**
   - `crates/shell/tests/agent_public_control_surface_v1.rs`
   - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - `crates/shell/tests/repl_world_first_routing_v1.rs`
   - plus bounded status/doctor/toolbox truth checks

6. **docs/tracker closeout**
   - `llm-last-mile/REMAINING-overall-scope-2026-06-10.md`
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - Slice `54` spec/plan/tasks docs

## Plan Summary

The narrowest honest Slice `54` is:

1. freeze parity expectations and support-posture uplift rules first,
2. make selected `claude_code` starts/turns take the authoritative host-tool path second,
3. prove live tool-call receipt/follow-up parity third,
4. finish with targeted validation, reporting truth, and tracker/docs closeout last.

## Implementation Order

### Packet 1: Freeze Parity Gate And Support-Posture Uplift Rules

Goal:

1. define exactly what “`claude_code` parity” means in repo truth,
2. identify the current guardrails that keep selected `claude_code` hosts on the old non-tool-staged path,
3. make support/reporting uplift depend on real parity evidence rather than documentation-only optimism.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/agents_cmd.rs`
3. `crates/shell/src/execution/agent_runtime/validator.rs` only if a bounded validation hook is needed

Why first:

1. it prevents Packet `2` from silently changing runtime behavior without a clear parity definition,
2. it keeps support-posture truth explicit before implementation starts,
3. later packets should consume this truth instead of inventing ad hoc parity heuristics.

Verification checkpoint:

1. the slice still keys off resolved runtime family rather than hard-coded orchestrator ids,
2. selected `claude_code` host-path enablement is explicit,
3. the repo has one clear rule for when support posture may be upgraded.

### Packet 2: Land Selected `claude_code` Start/Turn Parity At The Host Submission Boundary

Goal:

1. make selected `claude_code` host starts and turns receive the same toolbox env injection as the first family floor,
2. make selected `claude_code` host starts and turns receive the same startup/system-prompt contract disclosure,
3. keep any Claude-specific request shaping bounded to the runtime adapter seam,
4. avoid changing selection truth, exact backend policy truth, or transport architecture.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/prompt_fulfillment.rs`
3. `crates/shell/src/execution/agents_cmd.rs`
4. `crates/gateway/src/adapter_runtime.rs` only if a bounded family-specific adapter adjustment is truly required

Why second:

1. this is the first real runtime behavior change,
2. Packet `1` should already define the parity target and uplift rules,
3. once start/turn parity exists, Packet `3` can validate live tool-call semantics end-to-end.

Verification checkpoint:

1. selected `claude_code` starts no longer stay on the old non-tool-staged path,
2. selected `claude_code` turns receive the authoritative host-tool surface,
3. no hidden fallback to `codex` is introduced.

### Packet 3: Prove Live Tool Invocation And Receipt/Follow-Up Parity

Goal:

1. route live `claude_code` host tool calls through `tool_invocation_contract.rs`,
2. preserve runtime-owned request/session/caller/world field injection,
3. preserve `task_run_id` and retained-worker receipt/follow-up semantics,
4. preserve shared worker messaging/lifecycle/steering meaning across runtime families.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
2. `crates/shell/src/execution/prompt_fulfillment.rs`
3. `crates/shell/src/repl/async_repl.rs` only where existing transport hookup requires it
4. focused shell tests for routed parity behavior

Why third:

1. Packet `2` must first make the live host-tool path exist for `claude_code`,
2. this packet proves that parity is semantic, not just cosmetic prompt injection,
3. it is the main guard against family-specific receipt drift.

Verification checkpoint:

1. a live `claude_code` host tool call reaches internal world dispatch,
2. runtime-owned fields remain shell-injected,
3. returned receipts preserve frozen Slice `52` semantics,
4. retained follow-up behavior remains shared across runtime families.

### Packet 4: Validation Wall, Reporting Truth, And Tracker/Docs Closeout

Goal:

1. close the Slice `54` validation wall,
2. upgrade operator/reporting posture to match the new repo truth,
3. update the remaining-scope note and gap matrix so parity becomes the landed baseline rather than a pending seam,
4. fix any bounded stale reference drift encountered while aligning docs.

Primary touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. focused shell tests under `crates/shell/tests/`
3. `llm-last-mile/REMAINING-overall-scope-2026-06-10.md`
4. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
5. Slice `54` spec/plan/tasks docs

Why last:

1. truth surfaces should describe what actually landed,
2. tracker/docs cleanup is easiest once runtime truth is known,
3. Packet `4` is where parity can be converted from implementation fact into durable repo truth.

Verification checkpoint:

1. targeted tests are green,
2. `doctor` / `toolbox status` do not overclaim more than the slice actually proved,
3. remaining-scope and gap-matrix docs reflect the new post-Slice-54 truth.

## Risks And Mitigations

1. **Risk: `claude_code` requires family-specific shaping that leaks into semantic drift**
   - Mitigation: keep any Claude-specific shaping isolated to `prompt_fulfillment.rs` / bounded adapter seams and reuse Slice `52` semantics unchanged above that seam.

2. **Risk: support-posture reporting gets upgraded before parity is truly proven**
   - Mitigation: gate posture uplift on Packet `2` + `3` behavior plus Packet `4` validation evidence.

3. **Risk: selected host start parity and targeted turn parity diverge**
   - Mitigation: treat both start and turn as required parity seams and cover both in tests.

4. **Risk: parity work accidentally reopens worker semantics or steering policy**
   - Mitigation: keep the messaging/lifecycle/policy design docs as authority and reject family-specific control-plane drift.

5. **Risk: docs become partially aligned while tracker truth remains stale**
   - Mitigation: reserve Packet `4` for coordinated remaining-scope + gap-matrix + operator-truth closeout.

## Parallelism Guidance

Mostly sequential:

1. Packet `1` should land before Packet `2`.
2. Packet `2` should establish selected start/turn parity before Packet `3` validates semantic parity.
3. Packet `4` should close only after Packet `3` is stable.

Possible bounded parallelism:

1. test-fixture preparation for selected `claude_code` parity coverage can happen while Packet `2` is implemented,
2. tracker/gap-matrix wording updates can be drafted in parallel with final validation, but should merge only after runtime truth is known.
