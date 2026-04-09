# substrate-gateway-boundary-and-runtime-ownership — CI checkpoint plan (pre-planning)

This file defines **when** cross-platform CI gates run for this feature.

The canonical checkpoint-plan path for this pack is `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/ci_checkpoint_plan.md`; this staged copy is the Phase B promotion candidate only.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/minimal_spec_draft.md` (draft slice skeleton + boundary invariants)
- Slice specs: see `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`.
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, or contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group once slice tasks exist in `tasks.json`.
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan’s slice list is based on the draft slice skeleton in `pre-planning/minimal_spec_draft.md` and is not mechanically validated yet.
- Scope note: this pack is currently planned as documentation-only boundary clarification. The checkpoint gate selection below assumes no production-code or smoke-script delta lands inside this pack. If full planning introduces runtime work, revise this plan before task wiring.

## Machine-readable plan (linted)

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
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Single end-of-feature checkpoint for the full draft slice set (1 slice < min_triads_per_checkpoint=4). Boundary after SGBRO0 completes the docs-only contract seam: command grammar, Substrate-versus-gateway ownership, status-schema freeze, and platform-parity wording are coherent enough for cross-platform audit/CI review. Feature smoke is deferred because the current pack scope introduces no runtime behavior delta and `impact_map.md` marks implementation surfaces as evidence-only."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`SGBRO0`) — boundary-contract completion seam

Why this boundary is code-grounded:
- Total draft slices for this pack is 1 (`SGBRO0`), so the CI checkpoint standard allows a single checkpoint because total slices is below `min_triads_per_checkpoint=4`.
- `SGBRO0` is the only coherent seam available in pre-planning: the operator-visible contract is not reviewable until the command grammar, ownership matrix, stable `status --json` scope, and platform-parity posture are defined together.
- `pre-planning/minimal_spec_draft.md` explicitly keeps this slice docs-only and treats runtime/backend mechanics as delegated surfaces, so splitting an earlier checkpoint would create incomplete contract validation rather than useful safety.

What surfaces this checkpoint stabilizes (from `pre-planning/spec_manifest.md`):
- `contract.md`: canonical `substrate world gateway {sync,status,restart}` grammar, stable non-secret wiring names, and exit-code posture.
- `runtime-boundary-spec.md`: Substrate-owned versus `substrate-gateway`-owned responsibilities, ADR-0023 supersession, and delegation boundaries to ADR-0027, ADR-0028, ADR-0017, and ADR-0041.
- `gateway-status-schema-spec.md`: stable `substrate world gateway status --json` surface and the explicit non-goal of freezing `sync --json`.
- `platform-parity-spec.md`: Linux/macOS/Windows parity contract and allowed divergence posture.
- `manual_testing_playbook.md` plus `slices/SGBRO0/SGBRO0-spec.md`: deterministic review coverage for the docs-only pack.

What risk is reduced by running cross-platform CI here (from `pre-planning/impact_map.md`):
- Prevents contradictory operator guidance from surviving across ADR-0040, archived gateway materials, and the future feature-local docs.
- Forces a single review seam before downstream runtime packs treat this boundary as settled, reducing the risk of mixed command grammar, duplicated config ownership, or accidental expansion into adapter/runtime mechanics.
- Preserves cross-platform planning discipline even for a docs-only pack: compile parity and quick CI testing remain the checkpoint defaults if non-doc changes appear unexpectedly, while `ci_audit` should record a docs-only skip when the diff stays documentation-only.
- Feature smoke is intentionally not required at this checkpoint because `impact_map.md` constrains the current pack to authoring work only and marks runtime/code surfaces as evidence-only rather than in-scope behavior changes.

## Follow-ups

- Full-planning task wiring:
  - Add slice triad tasks for `SGBRO0` and add `CP1-ci-checkpoint` (type `ops`) once the pack leaves pre-planning.
  - Set `tasks.json` `meta.checkpoint_boundaries = ["SGBRO0"]` when slice tasks exist and keep it aligned with this plan.
  - Replace the current draft slice list only if full planning splits or merges `SGBRO0`; update this plan first if that happens.
- Mechanical validity:
  - Replace the draft `checkpoints[].slices` list with the final accepted slice ids if full planning changes them.
  - Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership"` only after slice tasks and the checkpoint task exist in `tasks.json`.
- Scope guard:
  - If full planning introduces runtime implementation or smoke-script work despite the current docs-only baseline, revisit `feature_smoke` and `ci_testing` mode before wiring checkpoint tasks.
