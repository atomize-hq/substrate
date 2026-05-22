# world_process_exec_tracing_parity — plan

## Scope
- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity`
- Orchestration branch: `feat/world-process-exec-tracing-parity`
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Spec ownership map: `docs/project_management/packs/active/world_process_exec_tracing_parity/spec_manifest.md`
- Impact map: `docs/project_management/packs/active/world_process_exec_tracing_parity/impact_map.md`

## Goal
- Land ADR-0028: subprocess-level exec/exit telemetry for in-world executions, with trace correctness + joinability shored up first so higher-volume process events remain analyzable and safe.

## Guardrails (non-negotiable)
- Specs are the single source of truth; integration reconciles code/tests to the specs.
- Canonical trace (`~/.substrate/trace.jsonl`) must be safe-by-default:
  - argv/env redaction is required for new process event families.
  - preexec/builtin tracing must not write raw command bodies into canonical trace.
- “Denied” must be unambiguous from completion records (router daemon alignment; see ADR-0029).
- Keep slices small: each triad is one behavior delta with bounded acceptance criteria.

## Deliverables (authoritative)
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Decision Register: `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md`
- Contract: `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md`
- Schema: `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
- Protocol: `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`
- Security posture: `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md`
- Manual testing playbook: `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
- Smoke scripts: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/`

## Platforms
- Behavior platforms required:
  - Linux (ptrace-based capture in world backend).
  - macOS (Lima): ptrace capture supported because world-service runs in a Linux guest (`docs/WORLD.md`).
  - Windows: must remain build/trace coherent while process capture is out of scope (degrade with explicit diagnostics).
- CI parity platforms required:
  - linux/macos/windows (compile/test parity).

## Triads (slices)
- WPEP0: Trace correctness + joinability foundation (parent_span correctness, completion ergonomics, cmd_id↔span_id bridge, safe preexec posture).
- WPEP1: World-agent API + types + shell persistence for process events (process_events + diagnostics + trace event family).
- WPEP2: Linux in-world process capture implementation + provisioning/caps/truncation (argv may be omitted explicitly).
- WPEP3: Redaction hardening + argv/env capture for process events (Linux-backed backends: native Linux + macOS Lima guest).
