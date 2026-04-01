# world-disabled-reason-attribution — spec manifest (pre-planning)

This file enumerates every contract, telemetry, path-redaction, and platform surface for ADR-0038 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Prerequisite planning pack:
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`

## Slice IDs (canonical)

ADR-0038 uses placeholder slice ids (`C0`, `C1`, `C2`). This pack uses feature-derived slice ids per the triad setup standard.

Canonical slice ids:
- Slice prefix: `WDRA`
- `WDRA0` — shared replay disable-attribution classifier seam
- `WDRA1` — replay stderr copy + replay_strategy telemetry wiring
- `WDRA2` — regression coverage, docs alignment, and cross-platform validation seam

## Required spec documents (authoritative)

- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/platform-parity-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/quality_gate_report.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/kickoff_prompts/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/kickoff_prompts/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/WDRA1-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/kickoff_prompts/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/kickoff_prompts/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | Notes |
| --- | --- | --- |
| Replay origin summary and host warning copy | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | Exact reason fragments and formatting rules live here. |
| Replay selection invariants and out-of-scope boundaries | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | `--world > --no-world > SUBSTRATE_REPLAY_USE_WORLD` stays unchanged. |
| Trace/log field additions for `replay_strategy` | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | Field names, emission rules, and redaction rules live here. |
| Tokenized path display and non-secret env display contract | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | Human-visible fragments reuse ADR-0037 semantics. |
| Cross-platform guarantees and permitted divergences | `docs/project_management/packs/draft/world-disabled-reason-attribution/platform-parity-spec.md` | Linux/macOS/Windows parity lives here. |
| Manual verification steps and smoke assertions | `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md` | The playbook mirrors smoke scripts and test filters. |
| Slice order and triad workflow guardrails | `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md` | Execution order and checkpoint cadence live here. |
| Task graph, dependency wiring, prompt paths | `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json` | Automation metadata lives here. |
| Slice-local behavior and AC ids | slice specs under `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/` | Each slice spec owns only one behavior delta. |

## Explicit out-of-scope surfaces

This feature does not own these categories:
- New replay subcommands or new replay JSON envelopes.
- New config keys or new environment variables.
- Replay routing behavior changes, backend-selection changes, timeout changes, or exit-code changes.
- Doctor/health attribution semantics beyond reuse of the ADR-0037 shared classifier.

## Full-planning follow-ups

- Lock the human-visible reason fragments in `contract.md` and the structured telemetry fields in `telemetry-spec.md` before slice specs are treated as final.
- Keep replay-local opt-out tokens (`--no-world flag`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`, `--flip-world`) stable and distinct from effective-config disable attribution.
- Keep absolute host paths and raw env values out of replay stderr and out of `replay_strategy` telemetry.
