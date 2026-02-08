# world_process_exec_tracing_parity — impact map

Standard:
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`

## Inputs
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Decision Register: `docs/project_management/next/world_process_exec_tracing_parity/decision_register.md`
- Spec manifest: `docs/project_management/next/world_process_exec_tracing_parity/spec_manifest.md`

## Touch set (create/edit)

### Docs (this Planning Pack)
- Create/edit:
  - `docs/project_management/next/world_process_exec_tracing_parity/*`
- ADR references updated:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

### Trace schema + span lifecycle (host + shim)
- Edit:
  - `crates/trace/src/span.rs` (parent linkage correctness, completion ergonomics fields)
  - `crates/shell/src/execution/routing/dispatch/exec.rs` (cmd_id↔span_id bridge; parent env stack discipline)
  - `crates/shim/src/exec/policy.rs` (deny outcome marker; parent env discipline as needed)
  - `crates/replay/src/replay/executor.rs` (explicit `span_id` in replay_strategy for join clarity)

### Preexec/builtin command tracing
- Edit:
  - `crates/shell/src/scripts/bash_preexec.rs` (canonical trace omits body; optional debug-only raw log)
  - `docs/internals/env/inventory.md` (document `SUBSTRATE_PREEXEC_RAW_LOG`)

### World process exec/exit event family (new)
- Edit/create (planned by WPEP1/WPEP2):
  - `crates/world-agent` request/response/stream frame models (carry `process_events` + diagnostics)
  - `crates/agent-api-types` transport additions for `process_events`
  - `crates/shell` parsing + trace append for `world_process_*`
  - `crates/world` Linux backend capture + session storage (ptrace)
  - `crates/common` log schema constants + shared redaction helpers
  - Provisioning (Linux-backed backends):
    - `scripts/linux/world-provision.sh` (add `CAP_SYS_PTRACE` to world-agent systemd unit)
    - `scripts/mac/lima-warm.sh` (add `CAP_SYS_PTRACE` to guest systemd unit)
    - `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima/substrate-dev.yaml` (keep unit templates aligned)

## Cascading implications / contradiction risks
- `event_type` overlap (`command_complete` used by shell command summaries and span records) remains; consumers must filter by `component`. This feature improves joinability via explicit bridge fields so filtering is less error-prone.
- Router daemon (ADR-0029) requires deny/executed to be unambiguous from completion records; this pack introduces `outcome: "denied"` on deny completion spans.
- Preexec `builtin_command` is high-volume and potentially sensitive; canonical trace omits bodies to prevent secret leakage. Raw debug log remains opt-in and must not become triggerable by default.

## Cross-queue scan (high-level)
- ADR-0029 depends on ADR-0028 landing first; this pack prioritizes span correctness + joinability before adding new high-volume event families.
- ADR-0017 output attribution remains compatible; process event attribution uses `parent_span`/`parent_cmd_id` links.

## Sequencing alignment
- Sprint id: `world_process_exec_tracing_parity` (see `docs/project_management/next/sequencing.json`)
- Dependencies: ADR-0028 before ADR-0029.
