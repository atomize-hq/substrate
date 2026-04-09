# substrate-gateway-boundary-and-runtime-ownership — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/minimal_spec_draft.md`
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for CI cadence.
- If slice ids, platform scope, or checkpoint boundaries change, update this plan first, then update `tasks.json` and kickoff prompts.
- `tasks.json` already satisfies the required pre-planning baseline for schema v4 automation and cross-platform scope.
- `tasks.json` does not yet define slice tasks or checkpoint tasks, so `meta.checkpoint_boundaries` remains unset until full planning wires the task graph.

## Machine-readable plan (linted)

Pre-planning note:
- The machine-readable slice list uses the draft slice skeleton from `minimal_spec_draft.md`.
- Mechanical checkpoint validation starts after full planning adds slice integration tasks and the `CP1-ci-checkpoint` ops task to `tasks.json`.

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["SGBRO0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-of-feature checkpoint after SGBRO0. Total slices is 1, which is below defaults.min_triads_per_checkpoint. CP1 validates the gateway boundary ownership authority set, the Substrate-owned status and wiring contract, the fail-closed policy posture, and Linux/macOS/Windows parity before promotion into full planning and execution."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`SGBRO0`) — gateway ownership completion seam

Why this boundary is code-grounded:
- `SGBRO0` is the entire draft slice skeleton in `minimal_spec_draft.md`, so one checkpoint covers the full feature under the default minimum size rule.
- `spec_manifest.md` assigns the user-visible contract, JSON status schema, policy-evaluation rules, platform guarantees, and manual validation flow to one authority set. Splitting that set across multiple checkpoints would separate schema ownership from policy and parity ownership.
- `impact_map.md` identifies one joined risk seam across shell builtins, world-agent lifecycle/status transport, agent API types/client wiring, and operator-facing docs. The checkpoint belongs after that seam is coherent.

What surfaces are stabilized at CP1:
- `contract.md` for `substrate world gateway sync|status|restart`, the stable wiring env names, and the exit-code contract
- `gateway-status-schema-spec.md` for the `status --json` envelope and `client_wiring.*`
- `policy-spec.md` for fail-closed placement, host secret sourcing, and the ban on gateway-local control-plane trust
- `platform-parity-spec.md` for Linux/macOS/Windows guarantees
- `manual_testing_playbook.md` for one-owner-per-surface validation
- `slices/SGBRO0/SGBRO0-spec.md`, `plan.md`, and `tasks.json` alignment for the single-slice execution path

What risk CP1 reduces:
- command-surface drift between shell builtins, world-agent lifecycle/status endpoints, and operator docs
- status-schema drift where `client_wiring.*` or absent-state semantics diverge across human-readable and JSON outputs
- platform drift where Linux, macOS, and Windows expose different placement or lifecycle semantics for the same command family
- policy-boundary drift where host fallback, gateway-local admin state, or gateway-local persistence re-enters the operator contract

Why these gates run at CP1:
- Run `make ci-compile-parity ...` because the touch set crosses `crates/shell`, `crates/world-agent`, and shared API crates, and the pack keeps CI parity across Linux, macOS, and Windows in scope.
- Run `make feature-smoke ... PLATFORM=behavior` because the feature constrains operator-visible gateway lifecycle and status behavior on every behavior platform; compile parity alone does not validate those semantics.
- Run CI Testing in `full` mode because this checkpoint is also the final checkpoint for the feature and it covers contract, policy, platform-parity, and doc-surface alignment together.

## Follow-ups

- Add slice tasks to `tasks.json` for `SGBRO0`, including `SGBRO0-integ` and `SGBRO0-integ-core`, plus the `CP1-ci-checkpoint` ops task wired per the CI checkpoint standard.
- Set `tasks.json` `meta.checkpoint_boundaries = ["SGBRO0"]` when the slice task graph lands.
- Wire the first post-checkpoint slice, if full planning adds one, to depend on `CP1-ci-checkpoint`.
- Replace the draft slice list in this plan if full planning splits `SGBRO0` into multiple accepted slice ids.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership"` after the task graph and checkpoint metadata land.
