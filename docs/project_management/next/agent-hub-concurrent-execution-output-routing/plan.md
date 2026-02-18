# Plan — agent-hub-concurrent-execution-output-routing

Anchors (authoritative):
- ADR: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- Decision register: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/decision_register.md`
- Contract: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/contract.md`
- Spec manifest: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/spec_manifest.md`

Execution standards (non-negotiable):
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Guardrails (non-negotiable)

- Triads only: every slice ships as code+test+integration (`tasks.json` is authoritative).
- Automation mode: tasks run via triad worktrees; planning docs are edited only on the orchestration branch.
- Worktree rule (must be repeated in every kickoff prompt): `Do not edit planning docs inside the worktree.`
- Cross-platform validation cadence is bounded by `ci_checkpoint_plan.md` (no per-slice cross-platform dispatch).
- Specs and contract win: integration reconciles code/tests to the spec and contract; no implied behavior.

## Platforms (authoritative)

Declared in `docs/project_management/next/agent-hub-concurrent-execution-output-routing/tasks.json` meta:
- Behavior platforms (smoke required): `linux`, `macos`, `windows`
- CI parity platforms (compile parity required): `linux`, `macos`, `windows`
- WSL required: `false`

## Slices (authoritative)

Each slice is exactly one behavior delta and has one slice spec.

- OR0 — Event envelope + canonical trace persistence foundation
  - Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR0-spec.md`
  - Output: schema-enforced envelope + `event_type="agent_event"` trace records for structured agent events.

- OR1 — REPL output routing during PTY passthrough (buffer/drop + deterministic warning records)
  - Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR1-spec.md`
  - Output: strict non-injection during PTY passthrough, bounded buffering of structured event lines, deterministic suppression warning, and config knob plumbing.

## CI checkpoints (authoritative)

Source of truth:
- `docs/project_management/next/agent-hub-concurrent-execution-output-routing/ci_checkpoint_plan.md`

This feature has one bounded checkpoint:
- CP1 (`CP1-ci-checkpoint`) after `OR1-integ-core`, before platform-fix tasks and `OR1-integ` final.

## Execution order (operator runbook)

Execution is allowed only after planning quality gate `ACCEPT`:
- `docs/project_management/next/agent-hub-concurrent-execution-output-routing/quality_gate_report.md`

Then run, in order:
1) `F0-exec-preflight` (fills `execution_preflight_report.md`)
2) OR0 triad: `OR0-code` + `OR0-test` (concurrent) → `OR0-integ` → `OR0-closeout_report.md`
3) OR1 triad: `OR1-code` + `OR1-test` (concurrent) → `OR1-integ-core`
4) CP1 checkpoint: `CP1-ci-checkpoint`
5) Platform-fix tasks when a platform gate fails: `OR1-integ-linux|macos|windows`
6) Final aggregator: `OR1-integ` → `OR1-closeout_report.md`
7) Feature cleanup: `FZ-feature-cleanup`
