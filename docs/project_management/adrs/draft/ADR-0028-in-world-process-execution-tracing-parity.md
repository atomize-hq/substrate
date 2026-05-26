# ADR-0028 — In-World Process Execution Tracing Parity (Process Tree Exec/Exit Telemetry)

## Status
- Status: Accepted
- Date (UTC): 2026-01-29
- Owner(s): Shell + World-Agent + World runtime

## Stable Curated ADR

- Current stable ADR: `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- This project-management file remains the planning-rich historical source retained for
  compatibility while `docs/project_management/**` is being retired.

## Scope
- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
- Intended branch name(s): `feat/world-process-exec-tracing-parity`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/packs/active/world_process_exec_tracing_parity/plan.md`
- Tasks: `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json`
- Spec manifest: `docs/project_management/packs/active/world_process_exec_tracing_parity/spec_manifest.md`
- Specs:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP0-spec.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP2-spec.md`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP3-spec.md`
- Contract (if present): `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md`
- Decision Register: `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md`
- Impact Map: `docs/project_management/packs/active/world_process_exec_tracing_parity/impact_map.md`
- Manual Playbook: `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 799e22a5981fbf644d83626d36c3bbd7b788b80a22ec85629e59c499c25815af
### Changes (operator-facing)
- World executions gain subprocess-level visibility (exec/exit telemetry) comparable to host shim tracing
  - Existing: host execution is richly observable via shims, but world execution is observable primarily at “one command per world execute/stream” granularity (no structured visibility into spawned subprocess trees).
  - New: world-service returns a redacted process tree (spawn/exec/exit) for each world execution, and the shell persists these as structured trace events alongside existing spans, policy decisions, and fs diffs.
  - Why: eliminate audit/debug blind spots for dependency installers and wrapper-based tools, and make world-first execution diagnosable without stdout/stderr inference.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md#L1`

## Problem / Context
- On the host, execution is richly observable via the shim: each spawned process can emit structured telemetry (argv/cwd/pid/exit/duration/etc) and/or span records.
- In contrast, world execution is observable primarily at “one command per world execute” granularity (`/v1/execute`, `/v1/stream`), with no structured visibility into subprocesses spawned inside the world runtime.
- This creates blind spots for:
  - dependency installers (npm/pip/apt) that spawn many children,
  - wrapper-based entrypoints (tools invoking `bash -lc ...`),
  - policy debugging and audit scenarios where stdout/stderr inference is insufficient.

### Repo reality checks (what exists today)
- `crates/world-service/src/service.rs`:
  - `/v1/execute`: spans are generated after execution, and `ExecRequest.span_id` is not currently populated.
  - `/v1/stream`: span id is generated up-front and `ExecRequest.span_id` is populated.
- `crates/world/src/session.rs` + `crates/world/src/exec.rs`:
  - execution occurs via `std::process::Command` (direct or via wrapper), with no per-process telemetry.
- `crates/trace/src/span.rs`:
  - command completion now reuses the parent captured at span start rather than re-reading `SHIM_PARENT_SPAN` at finish time.
- `crates/shell/src/execution/routing/dispatch/exec.rs` + `crates/shim/src/exec/policy.rs`:
  - shell and shim span lifecycles now enforce `SHIM_PARENT_SPAN` stack discipline (push current span while active; restore/unset on finish/drop).
- `crates/shell/src/execution/invocation/runtime.rs`:
  - script-mode `command_complete` events include the `world_fs_strategy_*` contract fields.
- `crates/replay/src/replay/executor.rs`:
  - `replay_strategy` carries both `cmd_id` and an explicit `span_id` field, so replay joins do not depend on consumers knowing that `cmd_id` stores the span id.
- Redaction:
  - shim argv redaction is robust (`crates/shim/src/logger.rs`), including “flag consumes next arg” semantics.
  - `substrate_common::redact_sensitive()` is not sufficient for safe argv/env capture at process granularity (it does not redact values following flags).
- Provisioning (Linux-backed backends):
  - The `substrate-world-service` systemd unit currently does not grant `CAP_SYS_PTRACE` by default (`scripts/linux/world-provision.sh`, `scripts/mac/lima-warm.sh`), so ptrace-based capture may be blocked unless the unit is updated.

## Goals
- Achieve in-world per-process execution tracing parity with the host shim model:
  - capture a process tree for each world execution (exec/exit at minimum; fork/clone for parent relationships),
  - capture argv/env/cwd with safe redaction and data minimization,
  - capture pid/ppid and exit status, plus timing (start timestamp + duration).
- Return process events from world-service for both:
  - `/v1/execute` (batched in the response),
  - `/v1/stream` (batched on the Exit frame initially).
- Persist process events into `~/.substrate/trace.jsonl` via the existing shell trace append pathway so they are co-located with:
  - command spans,
  - policy decisions,
  - filesystem diffs.
- Fix parent span linkage so process events can reliably attach to spans without broken trees.
- Make trace trees and joins credible before adding high-volume process events:
  - ensure `parent_span` is correct on completion spans (no self-parent loops),
  - enforce `SHIM_PARENT_SPAN` env stack discipline (push on span start; restore on finish),
  - add an explicit bridge between shell `cmd_id` events and shim `span_id` spans.
- Provide safe caps/truncation to prevent “dependency install explodes response” scenarios.

## Non-Goals
- Streaming each process event live over `/v1/stream` (v1 batches on Exit; streaming per-event is a follow-on optimization).
- Installing shims inside the world filesystem or mutating world PATH to get subprocess tracing.
- Implementing a text-parsing `strace -f` ingestion pipeline.
- Emitting unredacted argv/env, or persisting secrets.

## User Contract (Authoritative)

### CLI
- No new CLI commands or flags are introduced by this ADR.
- Operator-visible behavior change:
  - When world execution succeeds and process tracing is supported, `~/.substrate/trace.jsonl` gains additional event records with `event_type` `world_process_start` and `world_process_exit`.
  - When process tracing is unavailable or fails, execution MUST still succeed (see “Failure behavior”), but the world-service response MUST carry an explicit structured diagnostic so operators can distinguish “no subprocesses spawned” from “subprocess tracing unavailable”.

### World-Agent API (Authoritative)
This ADR extends the world-service responses to optionally include process events.

- `/v1/execute` response:
  - MUST include `span_id` generated before execution.
  - MUST set `ExecRequest.span_id = Some(span_id)` before calling into the world backend.
  - MUST include `process_events` (array; MAY be empty).
  - MUST include `process_events_status`: `"ok" | "truncated" | "unavailable" | "error"` (string).
  - MUST include `process_events_reason` when `process_events_status != "ok"`.
  - `process_events` MUST be omitted only if `process_events_status="error"` and the system cannot safely emit any records.
  - Deterministic reason codes (stable strings; non-exhaustive):
    - `"not_supported_platform"`
    - `"backend_disabled"`
    - `"ptrace_not_permitted"`
    - `"capture_overflow"`
    - `"internal_error"`

- `/v1/stream` Exit frame:
  - MUST include `process_events` using the same semantics as `/v1/execute`.
  - MUST include `process_events_status` using the same semantics as `/v1/execute`.
  - MUST include `process_events_reason` when `process_events_status != "ok"`.

### Trace output (Host)
- The shell MUST append each `WorldProcessEvent` into `~/.substrate/trace.jsonl` using the existing trace append mechanism, and MUST do so before writing the root command `command_complete` span/event so correlations are stable for downstream analysis.

### Span correctness + joinability (Host + Shim)
- Span parent linkage:
  - `parent_span` on `command_complete` MUST equal the parent captured at span start (it MUST NOT be re-read from env at finish time).
  - The active span lifecycle MUST push `SHIM_PARENT_SPAN=<current_span_id>` for the duration of command execution and MUST restore the previous value (or unset) when the command completes.
- Cross-component joinability:
  - When `SHIM_PARENT_CMD_ID` is available, spans MUST record it as `parent_cmd_id` on both `command_start` and `command_complete`.
  - Shell `command_start`/`command_complete` events MUST include `span_id` when a span exists for that command, so analysts can join shell summaries ⇄ spans ⇄ world process events deterministically.
- Completion ergonomics:
  - `command_complete` spans MUST include `duration_ms`.
  - When a command is denied by policy, the completion span MUST be unambiguous (`outcome: "denied"` and the `policy_decision` must be present on completion).
- Preexec safety:
  - When `SUBSTRATE_ENABLE_PREEXEC=1`, `builtin_command` records in the canonical trace MUST omit command bodies (metadata + correlation only).
  - If raw command bodies are needed, they MUST be written only to an explicit debug-only file path and MUST be clearly labeled as potentially sensitive.

### Event schema (Authoritative)
World process events are structured trace events aligned with `crates/common/src/lib.rs` (`log_schema` module).

- Event types:
  - `world_process_start`
  - `world_process_exit`

- Minimum required fields (always present):
  - `ts` (timestamp)
  - `event_type` (`world_process_start` or `world_process_exit`)
  - `component` (`world-service`)
  - `session_id` (from `SHIM_SESSION_ID` env propagated into the world execution)
  - `world_id`
  - `pid`, `ppid`
  - `cwd`
  - One of:
    - `argv` (redacted array)
    - `argv_omitted: true`
  - On exit:
    - `exit_code` (or signal termination; see optional fields)
    - `duration_ms`

- Correlation fields (required for acceptance):
  - `parent_span` (attach to host command span id from env `SHIM_PARENT_SPAN`)
  - `parent_cmd_id` (optional but recommended; from env `SHIM_PARENT_CMD_ID`)

- Optional fields (profile-gated / best-effort; may be omitted):
  - `env` (redacted map; see “Data minimization”)
  - `exe` (best-effort `/proc/<pid>/exe` symlink target when readable)
  - `signal` (when terminated by signal)

### Data minimization for env (Authoritative)
Default env capture policy for process events:
- Include only allowlisted keys with redacted values:
  - `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `LC_*`, `TERM`
  - `SHIM_*`, `SUBSTRATE_*`
  - proxy vars (`HTTP*_PROXY`, `NO_PROXY`) with aggressive credential redaction
- For non-allowlisted keys:
  - omit entirely (default), or include as `"<omitted>"` only if explicitly enabled by a future trace/profile mode.

### Caps / truncation (Authoritative)
To bound volume:
- Cap maximum events per execution (default: 10,000).
- Cap argv length per event and env value lengths (default: 4KB/value).
- When truncation occurs, the world-service response MUST include summary fields:
  - `process_events_status: "truncated"`
  - `process_events_dropped: <n>`

## Phase 8 additive — Correlation vocabulary + required/optional matrix (cross-feature)

This section is additive-only and exists to prevent correlation drift across LLM/agents/workflows/router features.

### Vocabulary (canonical field names)

Unless explicitly noted otherwise:
- All fields are top-level JSON keys (no nesting required for joinability).
- All correlation fields are metadata-only (no secrets); redact/cap any field that can accidentally carry secret material (e.g., URLs with embedded credentials).

Canonical correlation fields:
- `session_id`
  - Meaning: the trace session identifier for the current Substrate shell session.
  - Rule: MUST be present on all trace records appended to canonical trace (`trace.jsonl`), including derived/router/toolbox records.
  - Relationship to orchestration: `session_id` scopes records to a particular shell session, but it is not sufficient for multi-agent joins; multi-agent attribution MUST use `orchestration_session_id`/`run_id` when applicable (no heuristic joins).

- `orchestration_session_id`
  - Meaning: multi-agent orchestration session identifier (spans concurrent agents and their events).
  - Rule: MUST be present on any agent/LLM/workflow/toolbox/router record that participates in multi-agent orchestration joins.
  - Relationship to `session_id` (non-negotiable):
    - `orchestration_session_id` is additive and does not supersede `session_id`; canonical trace records still require `session_id`.
    - A single `session_id` MAY contain multiple orchestration sessions (and vice versa); consumers MUST NOT assume a 1:1 mapping and MUST NOT attempt heuristic mapping between them.

- `run_id`
  - Meaning: unit-of-work identifier inside an orchestration session (e.g., an agent task run, an LLM request run, or a workflow run).
  - Rule: MUST be present on structured agent events and other “run-scoped” event families; MAY be omitted on purely interactive human-only command spans.

- `thread_id`
  - Meaning: optional conversation/thread identifier where applicable (LLM + agent interactions).
  - Rule: OPTIONAL; present only when the emitting component has a real thread concept.

- `agent_id`
  - Meaning: actor identifier.
    - `human` for direct operator actions.
    - for agent-driven actions/events, this is the agent inventory id (e.g., `codex`, `claude_code`).
  - Rule: OPTIONAL on command spans; REQUIRED on structured agent events and toolbox tool-call events when an agent is the actor.
  - Non-negotiable clarification: `agent_id` is for attribution/audit as the actor/principal. It MUST NOT be treated as a backend-selection identifier; when a specific backend is involved, `backend_id` MUST be present so allowlist/routing joins are explicit and non-heuristic.

- `role`
  - Meaning: agent role taxonomy label (e.g., `orchestrator`, `member`).
  - Rule: OPTIONAL; when present it MUST reflect the effective role assignment used for gating/attribution.

- `backend_id`
  - Meaning: backend allowlist/routing identifier in `<kind>:<name>` form (e.g., `cli:codex`, `api:openai`).
  - Rule: when a specific backend is involved (LLM engine/backend, agent backend), `backend_id` MUST be present to avoid heuristic mapping from `agent_id` or other fields.

- `world_id`
  - Meaning: identity of the active world boundary (filesystem/network isolation boundary).
  - Rule: REQUIRED on in-world process telemetry (`world_process_*`). When a record is emitted “in-world” or describes an in-world execution/session, it MUST include `world_id` so operators can verify boundary sharing and restarts.

- `span_id` / `parent_span`
  - Meaning: trace span identifiers and linkage.
  - Rule: command spans MUST include `span_id` and MAY include `parent_span`. Any non-span record that attaches to a command span MUST include `cmd_id` and/or `parent_span` to make joins explicit.

- `cmd_id` / `parent_cmd_id`
  - Meaning: command identifier and linkage for non-span event records that reference a command span.
  - Rule: any record that is “about” a particular command execution MUST include `cmd_id`. Any record that is “about” a subprocess tree for a command MUST include `parent_cmd_id` when available.

- `workspace_id`
  - Meaning: stable workspace identity (UUID string) for cross-workspace routing and attribution.
  - Rule: REQUIRED on workflow-router derived events and any record that participates in cross-workspace routing/joins; SHOULD be treated as the **source workspace** identity when both source and target exist.

- `target_workspace_id`
  - Meaning: stable workspace identity (UUID string) for the routing target when a router request/action targets a different workspace.
  - Rule: OPTIONAL; present on workflow-router derived events and request/queue artifacts only when a distinct target workspace exists.

- `request_id`
  - Meaning: workflow-router request identifier (UUID string) for request/action lifecycle joins.
  - Rule: REQUIRED on workflow-router derived event families and router request/queue artifacts.

- `idempotency_key`
  - Meaning: deterministic dedupe/join key for at-least-once router processing (hex string).
  - Rule: REQUIRED on workflow-router derived event families and router request/queue artifacts so duplicates are detectable without heuristics.

- `source_span_id` / `source_cmd_id`
  - Meaning: explicit cause reference to the source trace record that triggered a router rule match.
  - Rule: REQUIRED on workflow-router derived event families (one must be present; `source_span_id` preferred when available).

- `rule_id`
  - Meaning: workflow-router rule identifier.
  - Rule: REQUIRED on workflow-router derived event families.

- `tool_call_id`
  - Meaning: identifier for a toolbox/MCP tool invocation instance.
  - Rule: REQUIRED on toolbox tool-call trace records (`toolbox_tool_call_*`, Phase 8); OPTIONAL elsewhere. Reserved in this vocabulary to avoid later reshapes.

- `workflow_run_id` / `workflow_node_id`
  - Meaning: identifiers for workflow root and workflow node instances.
  - Rule: RESERVED for Phase 7/Phase 8 additions (workflow composition remains Draft), but the keys are reserved here to prevent later naming drift.

### Required/optional matrix (v1; additive-only extensions allowed)

This table records the v1 required set for joinability. New event families introduced by later ADRs MUST extend this matrix additively.

- `command_start` / `command_complete` (span records):
  - REQUIRED: `session_id`, `span_id`
  - CONDITIONAL REQUIRED:
    - `world_id` when the command executed inside a world boundary.
    - `backend_id` when a specific backend is involved (LLM engine/backend, agent backend).
  - OPTIONAL: `cmd_id`, `parent_span`, `agent_id`, `role`, `orchestration_session_id`, `run_id`, `thread_id`

- `world_process_start` / `world_process_exit` (world subprocess telemetry):
  - REQUIRED: `session_id`, `world_id`, `parent_span`
  - OPTIONAL (recommended when available): `parent_cmd_id`

- Structured agent events (ADR-0017 envelope persisted to trace):
  - REQUIRED: `session_id`, `orchestration_session_id`, `run_id`, `agent_id`
  - CONDITIONAL REQUIRED:
    - `backend_id` when the emitting backend’s kind is known (v1 default: known).
    - `world_id` when the emitting backend executes inside a world boundary.
  - OPTIONAL: `thread_id`, `role`, `cmd_id`, `span_id`

- Toolbox tool-call events (ADR-0026; control-plane audit records):
  - REQUIRED:
    - `session_id`
    - `orchestration_session_id`
    - `run_id`
    - `agent_id`
    - `backend_id`
    - `role`
    - `tool_call_id`
  - CONDITIONAL REQUIRED:
    - `world_id` when the tool execution occurs inside a world boundary (future; v1 toolbox is host-scoped).
  - OPTIONAL: `thread_id`, `source_span_id`, `source_cmd_id`

- Workflow-router derived events (ADR-0029; derived from `trace.jsonl`):
  - REQUIRED:
    - `session_id`
    - `request_id` (router request identifier; UUID string; required for all router request/action lifecycle events)
    - `idempotency_key` (hex string; required so at-least-once processing is joinable/dedupable)
    - `workspace_id` (the workspace identity that the router is operating in / routing from; required to avoid heuristic path-based joins)
    - a cause reference (one must be present):
      - `source_span_id` when the source trace record has a span id (preferred), and/or
      - `source_cmd_id`
  - CONDITIONAL REQUIRED:
    - `orchestration_session_id` when tied to orchestration context.
  - OPTIONAL: `rule_id`, `target_workspace_id`, `workflow_run_id`, `backend_id`

Router-derived event families (v1; additive-only list):
- Derived router events appended to canonical trace MUST use explicit `event_type` values (see router decision register DR-0016) such as:
  - `workflow_router_rule_match`
  - `workflow_router_request_enqueued`
  - `workflow_router_request_denied`
  - `workflow_router_request_pending_approval`
  - `workflow_router_action_enqueued`
  - `workflow_router_action_executed`
  - `workflow_router_cursor_gap_detected`

Toolbox tool-call event families (v1; additive-only list; ADR-0026):
- `toolbox_tool_call_start`
- `toolbox_tool_call_complete`

### Joinability rule (non-negotiable)

Any record intended to trigger routing or cross-component attribution MUST carry enough correlation keys to avoid heuristic joins. Specifically:
- A consumer MUST NOT be required to parse stdout/stderr or PTY bytes to join cause→effect.
- A consumer MUST NOT be required to join across multiple records merely to determine deny vs executed; deny/allow/outcome classification must be detectable from the relevant completion/derived record(s).

## Architecture Shape
- Components:
  - `crates/common`:
    - extend `crates/common/src/lib.rs` (`log_schema` module) with constants needed for correlation + process fields (`PARENT_SPAN`, `PARENT_CMD_ID`, `PID`, `PPID`, `ARGV`, `ENV`, `CWD`, etc.)
    - add shared redaction helpers (new module) suitable for argv/env redaction at scale
  - `crates/trace`:
    - fix span parent linkage bug in `crates/trace/src/span.rs` by capturing parent span at span start and restoring env stack discipline on finish
  - `crates/world` (Linux only; behind `cfg(target_os=\"linux\")`):
    - implement ptrace-based process tree capture in world exec paths (`crates/world/src/exec.rs`)
    - note: this Linux runtime powers both native Linux deployments and macOS Lima deployments (world-service runs in a Linux guest; `docs/WORLD.md`)
    - store captured events in session state keyed by `span_id` and provide take semantics to avoid unbounded growth
  - `crates/world-service`:
    - generate and plumb `span_id` consistently for `/v1/execute` and `/v1/stream`
    - retrieve captured events from the backend and return them in responses/frames
  - `crates/transport-api-types`:
    - extend response/frame types to transport `process_events` (greenfield; lockstep updates; breaking allowed)
  - `crates/shell`:
    - parse `process_events` from responses/Exit frames
    - append to trace.jsonl via existing trace append pathway

- End-to-end flow:
  - Inputs:
    - `ExecRequest` including `span_id`
    - effective policy snapshot (for `net_allowed` + other invariants)
    - environment variables propagated into the world (for correlation IDs)
  - Derived state:
    - ptrace-captured process tree events (when supported)
  - Actions:
    - execute command inside world
    - capture exec/exit events for the process tree
    - return events to host via world-service API
    - append events to `trace.jsonl`
  - Outputs:
    - process event records co-located with spans for deterministic reconstruction

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `world-process-exec-tracing-parity` (to be scheduled)
- Prerequisites / hard dependencies:
  - Shared redaction helpers in `crates/common` MUST ship before argv/env emission at process granularity.
  - Span parent linkage bug fix MUST ship before (or alongside) process event emission to keep trace trees valid.
- Cross-feature alignment dependencies:
  - The event/correlation fields MUST remain compatible with the output/event attribution contract used by agent orchestration (see `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`).

## Security / Safety Posture
- Fail-closed vs degrade behavior:
  - Execution MUST succeed even if ptrace-based tracing is unavailable (degrade gracefully).
  - When tracing is unavailable, this MUST be explicit in returned metadata and/or logs; silent omission is not allowed.
- Protected paths/invariants:
  - argv/env MUST be redacted; do not persist secrets.
  - caps/truncation MUST prevent unbounded trace growth for dependency-heavy commands.
  - platform guards MUST ensure ptrace logic is Linux-only unless/until equivalent mechanisms exist on other backends.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - shared argv redaction covers “flag consumes next arg” patterns and `--flag=value` patterns
  - env redaction strips credentials from URL-like values and redacts obvious secret keys
  - span parent linkage: no self-parent spans; stack discipline restores prior parent span
- Integration tests (world-service):
  - child spawn parity:
    - run a deterministic child-spawning command and assert at least two processes are observed with correct parent relationships and exit metadata
  - wrapper-based execution:
    - run a wrapper that invokes an inner shell and assert inner subprocess attribution exists
- Shell-level persistence test:
  - execute a world command that spawns children and assert `trace.jsonl` contains `world_process_start`/`world_process_exit` with correct `parent_span` and no secrets.

### Manual validation
- Manual playbook: `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Backwards compatibility: not a goal for this ADR. Host + world-service + API types are updated in lockstep; breaking changes are allowed.
- Trace volume increase is expected; bounded by caps and truncation summaries.

## Decision Summary
- Decision Register entries:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md`:
    - DR-0001 (In-world tracing mechanism: ptrace vs in-world shims)
    - DR-0002 (Env data minimization policy: allowlist-only vs full redacted map)
    - DR-0003 (Failure behavior: degrade (omit events) vs fail execution)
    - DR-0004 (Span parent linkage fix: capture at start + env stack discipline)
    - DR-0005 (Correlation bridge: shell `cmd_id` ↔ shim `span_id`)
    - DR-0006 (Deny outcome clarity on completion spans)
    - DR-0007 (Policy decision visibility on completion spans)
    - DR-0008 (Shim completion `duration_ms`)
    - DR-0009 (Preexec/builtin command privacy marker)
