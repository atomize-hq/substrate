# ADR-0028 — In-World Process Execution Tracing Parity (Process Tree Exec/Exit Telemetry)

## Status
- Status: Draft
- Date (UTC): 2026-01-29
- Owner(s): Shell + World-Agent + World runtime

## Scope
- Feature directory: `docs/project_management/next/world_process_exec_tracing_parity/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/world_process_exec_tracing_parity/plan.md`
- Tasks: `docs/project_management/next/world_process_exec_tracing_parity/tasks.json`
- Spec manifest: `docs/project_management/next/world_process_exec_tracing_parity/spec_manifest.md`
- Specs: `docs/project_management/next/world_process_exec_tracing_parity/specs/*`
- Contract (if present): `docs/project_management/next/world_process_exec_tracing_parity/contract.md`
- Decision Register: `docs/project_management/next/world_process_exec_tracing_parity/decision_register.md`
- Impact Map: `docs/project_management/next/world_process_exec_tracing_parity/impact_map.md`
- Manual Playbook: `docs/project_management/next/world_process_exec_tracing_parity/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- World executions gain subprocess-level visibility (exec/exit telemetry) comparable to host shim tracing
  - Existing: host execution is richly observable via shims, but world execution is observable primarily at “one command per world execute/stream” granularity (no structured visibility into spawned subprocess trees).
  - New: world-agent returns a redacted process tree (spawn/exec/exit) for each world execution, and the shell persists these as structured trace events alongside existing spans, policy decisions, and fs diffs.
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
- `crates/world-agent/src/service.rs`:
  - `/v1/execute`: spans are generated after execution, and `ExecRequest.span_id` is not currently populated.
  - `/v1/stream`: span id is generated up-front and `ExecRequest.span_id` is populated.
- `crates/world/src/session.rs` + `crates/world/src/exec.rs`:
  - execution occurs via `std::process::Command` (direct or via wrapper), with no per-process telemetry.
- `crates/trace/src/span.rs`:
  - bug: `ActiveSpan.finish()` reads `SHIM_PARENT_SPAN` from the environment at finish time, after it was mutated at span start; this can yield self-parent spans, breaking tree reconstruction.
- Redaction:
  - shim argv redaction is robust (`crates/shim/src/logger.rs`), including “flag consumes next arg” semantics.
  - `substrate_common::redact_sensitive()` is not sufficient for safe argv/env capture at process granularity (it does not redact values following flags).

## Goals
- Achieve in-world per-process execution tracing parity with the host shim model:
  - capture a process tree for each world execution (exec/exit at minimum; fork/clone for parent relationships),
  - capture argv/env/cwd with safe redaction and data minimization,
  - capture pid/ppid and exit status, plus timing (start timestamp + duration).
- Return process events from world-agent for both:
  - `/v1/execute` (batched in the response),
  - `/v1/stream` (batched on the Exit frame initially).
- Persist process events into `~/.substrate/trace.jsonl` via the existing shell trace append pathway so they are co-located with:
  - command spans,
  - policy decisions,
  - filesystem diffs.
- Fix parent span linkage so process events can reliably attach to spans without broken trees.
- Provide safe caps/truncation to prevent “dependency install explodes response” scenarios.

## Non-Goals
- Streaming each process event live over `/v1/stream` (v1 batches on Exit; streaming per-event is a follow-on optimization).
- Installing shims inside the world filesystem or mutating world PATH to get subprocess tracing.
- Implementing a text-parsing `strace -f` ingestion pipeline.
- Emitting unredacted argv/env, or persisting secrets.

## User Contract (Authoritative)

### World-Agent API (Authoritative)
This ADR extends the world-agent responses to optionally include process events.

- `/v1/execute` response:
  - MUST include `span_id` generated before execution.
  - MUST set `ExecRequest.span_id = Some(span_id)` before calling into the world backend.
  - MUST include `process_events: Option<Vec<WorldProcessEvent>>` (when tracing is supported and succeeds).
  - When tracing is unavailable or fails:
    - `process_events` MUST be omitted (`null`),
    - and the response MUST include a deterministic indicator that tracing was unavailable (e.g., `process_events_unavailable=true` or an equivalent structured diagnostic field).

- `/v1/stream` Exit frame:
  - MUST include `process_events: Option<Vec<WorldProcessEvent>>` using the same semantics as `/v1/execute`.

### Trace output (Host)
- The shell MUST append each `WorldProcessEvent` into `~/.substrate/trace.jsonl` using the existing trace append mechanism, and MUST do so before writing the root command `command_complete` span/event so correlations are stable for downstream analysis.

### Event schema (Authoritative)
World process events are structured trace events aligned with `crates/common/src/log_schema.rs`.

- Event types:
  - `world_process_start`
  - `world_process_exit`

- Minimum required fields (always present):
  - `ts` (timestamp)
  - `event_type` (`world_process_start` or `world_process_exit`)
  - `component` (`world-agent` or `world`; choose one and standardize across the event family)
  - `session_id` (from `SHIM_SESSION_ID` env propagated into the world execution)
  - `world_id`
  - `pid`, `ppid`
  - `cwd`
  - `argv` (redacted array)
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
  - omit entirely (default), or include as `\"<omitted>\"` only if explicitly enabled by a future trace/profile mode.

### Caps / truncation (Authoritative)
To bound volume:
- Cap maximum events per execution (default: 10,000).
- Cap argv length per event and env value lengths (default: 4KB/value).
- When truncation occurs, the world-agent response MUST include summary fields:
  - `process_events_truncated: true`
  - `process_events_dropped: <n>`

## Architecture Shape
- Components:
  - `crates/common`:
    - extend `crates/common/src/log_schema.rs` with constants needed for correlation + process fields (`PARENT_SPAN`, `PARENT_CMD_ID`, `PID`, `PPID`, `ARGV`, `ENV`, `CWD`, etc.)
    - add shared redaction helpers (new module) suitable for argv/env redaction at scale
  - `crates/trace`:
    - fix span parent linkage bug in `crates/trace/src/span.rs` by capturing parent span at span start and restoring env stack discipline on finish
  - `crates/world` (Linux only; behind `cfg(target_os=\"linux\")`):
    - implement ptrace-based process tree capture in world exec paths (`crates/world/src/exec.rs`)
    - store captured events in session state keyed by `span_id` and provide take semantics to avoid unbounded growth
  - `crates/world-agent`:
    - generate and plumb `span_id` consistently for `/v1/execute` and `/v1/stream`
    - retrieve captured events from the backend and return them in responses/frames
  - `crates/agent-api-types`:
    - extend response/frame types to transport `process_events` (optional, backward compatible)
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
    - return events to host via world-agent API
    - append events to `trace.jsonl`
  - Outputs:
    - process event records co-located with spans for deterministic reconstruction

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `world-process-exec-tracing-parity` (to be scheduled)
- Prerequisites / hard dependencies:
  - Shared redaction helpers in `crates/common` MUST ship before argv/env emission at process granularity.
  - Span parent linkage bug fix MUST ship before (or alongside) process event emission to keep trace trees valid.
- Cross-feature alignment dependencies:
  - The event/correlation fields MUST remain compatible with the output/event attribution contract used by agent orchestration (see `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`).

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
- Integration tests (world-agent):
  - child spawn parity:
    - run a deterministic child-spawning command and assert at least two processes are observed with correct parent relationships and exit metadata
  - wrapper-based execution:
    - run a wrapper that invokes an inner shell and assert inner subprocess attribution exists
- Shell-level persistence test:
  - execute a world command that spawns children and assert `trace.jsonl` contains `world_process_start`/`world_process_exit` with correct `parent_span` and no secrets.

### Manual validation
- Manual playbook: `docs/project_management/next/world_process_exec_tracing_parity/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work:
  - Agent API response additions MUST be backward compatible (`Option`, `serde(default)`, and `skip_serializing_if`).
  - Trace volume increase is expected; bounded by caps and truncation summaries.

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/world_process_exec_tracing_parity/decision_register.md`:
    - DR-0001 (In-world tracing mechanism: ptrace vs in-world shims vs strace parsing)
    - DR-0002 (Env data minimization policy: allowlist-only vs full redacted map)
    - DR-0003 (Failure behavior: degrade (omit events) vs fail execution)
    - DR-0004 (Span parent linkage fix: capture at start + env stack discipline)

