# Execution Preflight Gate Report — agent-hub-concurrent-execution-output-routing

Date (UTC): 2026-04-05T11:08:08Z

Standard:

- `docs/project_management/system/standards/execution/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:

- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing`

## Recommendation

RECOMMENDATION: **REVISE**

Triads must not begin yet. The pack is mechanically wired for schema-v4 boundary-only execution, and the governing ADR is accepted, but the smoke layer still fails the start-gate standard.

1. The smoke scripts do not yet mirror the manual feature-validation workflow.
   - The manual playbook validates real `substrate` flows (`:demo-agent`, PTY overlap, trace assertions, warning assertions), but all three smoke scripts only dispatch test binaries via `cargo test` and report `OK` on suite success.

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/quality_gate_report.md`)
- [x] ADR accepted and still matches intent
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Required planning artifacts exist: `spec_manifest.md`, `impact_map.md`, `ci_checkpoint_plan.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms; WSL is not required for this pack)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
  - `OR0` is a single foundation delta: envelope enforcement plus canonical `agent_event` trace persistence.
  - `OR1` is a single routing delta: PTY passthrough non-injection, buffering/drop behavior, deterministic warnings, and config knob plumbing.
- Any required splits before starting execution:
  - None. The slices are narrow enough for triad execution as planned.

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/tasks.json` meta:

- Declared behavior platforms (smoke required): `linux, macos, windows`
- Declared CI parity platforms (parity required): `linux, macos, windows`
- WSL required: `false`
- Schema and automation invariants verified:
  - `schema_version = 4`
  - `cross_platform = true`
  - `execution_gates = true`
  - `automation.enabled = true`
  - `meta.checkpoint_boundaries = ["OR1"]`
- Checkpoint structure verified:
  - `CP1-ci-checkpoint` exists and depends on `OR1-integ-core`.
  - Boundary-only platform-fix tasks exist only for `OR1`: `OR1-integ-core`, `OR1-integ-linux`, `OR1-integ-macos`, `OR1-integ-windows`, `OR1-integ`.
  - `OR0` correctly uses only `OR0-integ` and does not define boundary-only platform-fix tasks.
- `ci_checkpoint_plan.md` machine-readable JSON parses successfully and declares a single `CP1` checkpoint covering `OR0` + `OR1` after `OR1-integ-core`.
- Kickoff prompt coverage verified:
  - Every task id in `tasks.json` has a kickoff prompt file.
  - Every kickoff prompt contains the exact rule line: `Do not edit planning docs inside the worktree.`

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human validates the feature.

Manual playbook:

- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/manual_testing_playbook.md`

Smoke scripts:

- Linux smoke: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`

Parity notes (map smoke ↔ manual; include concrete assertions):

- Manual step(s):
  - Linux/macOS manual flow creates a temp workspace and `SUBSTRATE_HOME`, runs `substrate --no-world`, overlaps `:demo-agent` with `:pty`, then asserts trace contents and the post-passthrough warning behavior.
  - Windows manual flow runs `substrate --no-world --command ":demo-agent"` and asserts `event_type="agent_event"` exists in canonical trace.
- Smoke command(s):
  - Linux/macOS: `cargo test -p substrate-common --test agent_hub_event_envelope_schema`, `cargo test -p shell --test agent_hub_trace_persistence`, and for `OR1`, `cargo test -p shell --test repl_output_routing` plus `cargo test -p shell --test repl_config_max_pty_buffered_lines`.
  - Windows: the same `cargo test` suites.
- Expected output/assertion(s):
  - Manual playbook expects real CLI behavior, trace-file assertions, and operator-visible suppression behavior.
  - Smoke scripts currently assert only that the selected test suites exit `0` and then print `OK: ... smoke`.

## Gaps (must fix before execution begins):

- Gap 1: Smoke does not mirror the manual workflow closely enough.
  - The scripts do not execute `substrate` commands, do not create temporary homes/workspaces, do not assert `trace.jsonl` contents, and do not check the operator-visible warning/output behavior described in the manual playbook.
  - As written, the smoke layer is a thin test-suite wrapper, not a minimal execution-grade reproduction of the feature behavior.

## 3) CI Dispatch Path Is Runnable

Checkpoint task dispatch commands (copied from `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/kickoff_prompts/CP1-ci-checkpoint.md`):

- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/agent-hub-concurrent-execution-output-routing" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- CI testing (quick):
  - `make ci-testing CI_WORKFLOW_REF="feat/agent-hub-concurrent-execution-output-routing" CI_REMOTE=origin CI_CLEANUP=1 CI_MODE=quick CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- Feature Smoke dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" PLATFORM=behavior SMOKE_SLICE_ID="OR1" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/agent-hub-concurrent-execution-output-routing" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- Advisory CI audit (recommended before dispatch):
  - `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/agent-hub-concurrent-execution-output-routing" --ledger-path "docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/logs/OR1/ci-audit/ledger.jsonl"`
  - `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/agent-hub-concurrent-execution-output-routing" --feature-dir "docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" --ledger-path "docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/logs/OR1/ci-audit/ledger.jsonl"`
  - Evidence recorder (recommended after dispatch):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/logs/OR1/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "feat/agent-hub-concurrent-execution-output-routing" --run-id "<id>" --tested-sha "<sha>" --feature-dir "docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing"`

Policy note:

- Docs/planning-only changes (anything under `docs/`) may skip CI and smoke only when `ci_audit.sh` outputs `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.

Runner readiness:

- Workflow wiring and runner labels are defined in-repo:
  - CI Testing: `[self-hosted, Linux, linux-host]`, `[self-hosted, macOS]`, `[self-hosted, Windows]`
  - Feature Smoke: `[self-hosted, Linux, linux-host]`, `[self-hosted, macOS]`, `[self-hosted, Windows]`
- Live runner registration was not exercised during this preflight.

Run ids/URLs (if executed during preflight):

- CI compile parity: not run
- Linux smoke: not run
- macOS smoke: not run
- Windows smoke: not run

## 4) Required Fixes Before Starting OR0

- Replace the current smoke scripts with execution-grade smoke flows that minimally reproduce the manual assertions:
  - Linux/macOS smoke should drive `substrate` with a temp workspace/home, overlap `:demo-agent` with PTY passthrough, and assert canonical trace plus suppression warning behavior.
  - Windows smoke should drive the `:demo-agent` path and assert canonical trace persistence only, consistent with the platform parity spec.
- Re-run `F0-exec-preflight` after those fixes land; do not start `OR0-code` / `OR0-test` before the rerun returns `ACCEPT`.
