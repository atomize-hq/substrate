# Slice 60 Placeholder: Post-Placement-Aware Compatibility Retirement

Depends on:
- [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)
- [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)

Phase: `SPECIFY`  
Status: placeholder only; do not implement before Slice 58 is complete

## Purpose

This is the bounded follow-on **after Slice 58**.

It exists to retire temporary compatibility posture, legacy split-entry assumptions, and any remaining stale product truth that survive the Slice 58 placement-aware cutover.

## Minimal Scope

If Slice 58 lands with temporary compatibility shims or alias posture, Slice 60 should:

1. remove legacy `*_world` compatibility behavior that is no longer needed,
2. remove temporary docs/examples/fixtures that still describe the split-entry topology as current truth,
3. normalize remaining policy/examples/tests to the final placement-derived identity model,
4. prove no contradictory old-vs-new exact backend ids remain in forward product truth.

## Dependency Rule

Do not start Slice 60 until:

1. Slice 59 runtime truth and Codex guest runtime delivery are green,
2. Slice 58 placement-aware config/selector migration is green,
3. any temporary compatibility posture from Slice 58 is explicit enough to retire safely.

## Non-Goals

1. Do not use Slice 60 to finish work that properly belongs in Slice 59 runtime truth.
2. Do not use Slice 60 to introduce the placement-aware schema; that belongs in Slice 58.
3. Do not widen Slice 60 into a new runtime/artifact delivery redesign.

## Exit Condition

Slice 60 is complete when the repo’s forward truth is fully placement-aware and no longer depends on legacy split-entry compatibility language or temporary alias posture.

If Slice 58 lands in one step with no compatibility posture, this placeholder can be retired as unnecessary instead of expanded.
