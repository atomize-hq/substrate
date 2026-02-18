# ADR-0029 — Workflow Router Service (Trace-Driven Triggers, Cross-Workspace Requests)

## Status
- Status: Draft
- Date (UTC): 2026-02-05
- Owner(s): Spenser McConnell (Substrate); Shell maintainers

## Scope
- Feature directory: `docs/project_management/next/host_event_bus_router_daemon/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Decision Register: `docs/project_management/next/host_event_bus_router_daemon/decision_register.md`
- Trace/event foundations:
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- FS path semantics & allow/deny matching:
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Config/policy layering model:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`
- Future alignment (not required to land this ADR):
  - `docs/project_management/next/world-sync/` (internal git support; not yet landed)

## Executive Summary (Operator)

ADR_BODY_SHA256: 86af8acb4a4b7843b575b3738e111e54212356be41b3e0709b4ae82d3d506e48
### Changes (operator-facing)
- Substrate gains an always-on host router that can trigger policy-gated actions from trace events (including cross-workspace)
  - Existing: Substrate records trace events (`~/.substrate/trace.jsonl`), but there is no always-on host service that can “listen” for specific events and route them into follow-on work.
  - New: A host daemon tails the canonical trace stream and produces policy-gated requests/actions when routing rules match, including cross-workspace routing using an explicit workspace registry under `SUBSTRATE_HOME`.
  - Why: Enable reliable “when A completes, trigger B” workflows and selective file-change triggers without introducing an external broker or bypassing workspace policy boundaries.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md#L1`
    - `docs/project_management/next/host_event_bus_router_daemon/decision_register.md`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md#L1`
    - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md#L1`

## Problem / Context
- Substrate is moving toward multi-agent, multi-workspace orchestration. We need a reliable host-side mechanism to:
  - listen for specific events (command completion, selected fs diffs; later workflow/agent events are additive),
  - route them into follow-on work (often in a different workspace),
  - ensure all follow-on execution is policy-gated under the target workspace’s effective policy/config.
- We want this without:
  - introducing an external broker (Kafka/RabbitMQ) as a hard dependency,
  - inventing a second “event schema” that diverges from trace,
  - letting remote inputs bypass world/policy constraints.

## Goals
- Use `trace.jsonl` as the canonical local event log and primary subscription source.
- Implement an always-on **host daemon** (“router”) that:
  - tails `trace.jsonl` using durable cursors,
  - matches events against global + workspace-scoped routing rules,
  - produces durable, policy-gated requests/actions.
- Ensure the router daemon is a host service (not world-agent) so it remains available when worlds are disabled or when a VM/WSL backend is down.
- Support cross-workspace routing by introducing an explicit workspace registry under `SUBSTRATE_HOME` updated by `substrate workspace init|enable|disable`.
- Support selective file-operation triggers based on Substrate-collected fs diffs, scoped to specific file paths/directories (workspace-relative matching).

## Non-Goals
- Replacing the trace system: trace remains canonical.
- Building a general-purpose external message broker (Kafka/RabbitMQ) into Substrate.
- Triggering on out-of-band filesystem changes not mediated by Substrate execution (v1 triggers rely on Substrate-produced fs diffs; future work may add git-backed feeds or watchers).
- Exposing an Internet-accessible webhook gateway in v1 (remote ingress is future; v1 focuses on host daemon + local queues).

## User Contract (Authoritative)

### Terminology
- **Event**: an immutable record appended to `trace.jsonl`.
- **Derived trigger event**: a workflow-router-produced trace event indicating a rule match / routing decision.
  - “Derived” means: appended as an additional record after observing canonical trace events (it does not rewrite or mutate prior trace lines).
- **Request**: a durable “intent to act” object produced by the workflow router, evaluated under target workspace policy before execution.

### Files and locations (Authoritative)
All workflow-router state is stored under `SUBSTRATE_HOME` (default `~/.substrate`):

- Canonical event log (existing):
  - `SUBSTRATE_HOME/trace.jsonl`
- Workflow router state (new):
  - `SUBSTRATE_HOME/workflow/state.json` (durable cursor + dedupe metadata)
  - `SUBSTRATE_HOME/workflow/inbox.jsonl` (durable inbound requests; local-only in v1)
  - `SUBSTRATE_HOME/workflow/work_queue.jsonl` (durable queued actions derived from requests)
- Workspace registry (new):
  - `SUBSTRATE_HOME/workspaces/registry.json` (authoritative list of known workspaces and IDs)

#### Registry schema (v1; strict)
- `registry.json` is a strict JSON object:
  - `schema_version` (integer; v1 = `1`)
  - `workspaces` (array of objects)
- Each workspace entry is a strict object:
  - `workspace_id` (UUID string; matches `<workspace_root>/.substrate/workspace_id`)
  - `workspace_root` (canonical absolute path)
  - `enabled` (bool)
  - `label` (optional string)

### Workspace identity (Authoritative)
- Each workspace MUST have a stable `workspace_id`.
- `workspace_id` is a random UUID generated once and persisted in workspace-local metadata:
  - `<workspace_root>/.substrate/workspace_id` (single-line UTF-8; UUID string)
- The registry MUST store:
  - `workspace_id`
  - `workspace_root` (canonical absolute path)
  - `enabled` boolean (mirrors `workspace.disabled` behavior)
  - optional `label`
- Registry drift is fail-closed (see Decision Register DR-0018).

### Rule scoping and precedence (Authoritative)
- Rules exist at:
  - global scope (loaded from `SUBSTRATE_HOME/workflow/rules.yaml`), and
  - workspace scope (loaded from `<workspace_root>/.substrate/workflow/rules.yaml`).
- Rule evaluation precedence for a given event:
  1. workspace-scoped rules (for the owning workspace, when applicable)
  2. global rules (fallback)
- For a given `rule_id`, the workspace-scoped rule overrides a global rule with the same `rule_id`.

### Trigger taxonomy (Authoritative)
Only an explicit allowlist of event families is triggerable. v1 supports:
- Execution completion events:
  - `command_complete` spans in `trace.jsonl`
- Filesystem diff-derived events:
  - `fs_change` derived events emitted by the workflow router from `command_complete.fs_diff` indicating create/modify/delete/rename of workspace-relative paths
  - path matching MUST reuse the same workspace-relative semantics and matcher behavior as ADR-0018
- Derived workflow-router event types and required correlation keys are defined by Decision Register DR-0016.
  - Phase 8 additive alignment: any derived workflow-router trace record appended to canonical `trace.jsonl` MUST follow the correlation vocabulary and joinability rules defined in ADR-0028 (Phase 8 additive section), including carrying `session_id` and an explicit cause reference (`source_span_id` and/or `source_cmd_id`).

### File operation triggers (Authoritative)
- File triggers MUST be derived from Substrate-produced fs diffs (not OS filesystem watching in v1).
- The v1 diff source of truth for triggers is `command_complete.fs_diff` (trace span field).
- Triggers MUST support include/exclude path matching using workspace-relative paths:
  - exact file match
  - subtree match
  - pattern/glob match (as defined by ADR-0018)

### Policy gating (Authoritative)
- A trigger match MUST NOT execute work directly.
- Instead, it creates a durable **request** that is evaluated under the **target workspace’s** effective config + effective policy.
- If policy denies the requested action, the request MUST be recorded as denied with an explainable reason and MUST NOT execute.
- If policy requires approval, the request MUST be recorded as pending approval and MUST NOT execute until approved (see Decision Register DR-0019).

### Cross-workspace routing (Authoritative)
- A rule may route a request from an event in workspace A to a target workspace B.
- The target MUST be resolved via `workspace_id` (path may be used only as a fallback / debug surface).
- The workflow router MUST re-resolve effective config/policy for workspace B at execution time (not reuse workspace A’s).

### Daemon behavior (Authoritative)
- The workflow router service MUST be host-level and MUST run independently of world-agent availability.
- It runs as a `substrate` subcommand:
  - `substrate workflow serve [--foreground]`
- It MUST degrade gracefully:
  - if it cannot read `trace.jsonl`, it does not lose cursor state and retries,
  - if it cannot resolve a target workspace, it records a failed request and continues.

### Request queue semantics (Authoritative)
- The workflow router uses durable JSONL queues under `SUBSTRATE_HOME/workflow/`:
  - `inbox.jsonl` stores **requests** (durable “intent to act”).
  - `work_queue.jsonl` stores **actions** derived from requests after routing/policy evaluation.
- Processing semantics:
  - Handling is at-least-once; duplicate processing MUST be bounded via dedupe keys.
  - Each request/action MUST have a stable idempotency key derived from:
    - the source event identity (prefer `span_id` when available; e.g., `span_id` + `event_type` + rule_id), and
    - the target workspace identity (`workspace_id`).
  - The workflow router MUST persist per-subscriber cursors and dedupe state in `workflow/state.json` so restarts do not replay unboundedly.
  - Queue items are append-only; acknowledgement is cursor-based (see Decision Register DR-0014).
  - Retry/backoff and rate limiting are bounded and explicit (see Decision Register DR-0015).

#### Queue record schemas (v1; strict)
- `inbox.jsonl` records (one JSON object per line):
  - `schema_version` (integer; v1 = `1`)
  - `request_id` (UUID string)
  - `idempotency_key` (hex string)
  - `created_ts` (RFC3339 UTC string)
  - `source` (object):
    - `trace_path` (string; usually `SUBSTRATE_HOME/trace.jsonl`)
    - `source_event_type` (string; `command_complete` or `fs_change`)
    - `source_span_id` (string; optional; when the source trace record has a `span_id`, this MUST be set)
    - `source_cmd_id` (string)
    - `source_ts` (RFC3339 UTC string)
  - `rule_id` (string)
  - `target_workspace_id` (UUID string)
  - `action` (object; v1 only):
    - `kind` = `workflow.run`
    - `workflow_id` (string)
    - `args` (array of strings; optional)
- `work_queue.jsonl` records:
  - Same envelope as `inbox.jsonl`, plus:
    - `work_item_id` (UUID string)
    - `enqueued_ts` (RFC3339 UTC string)
  - Work items MUST NOT include any sensitive payload beyond what is required to execute the workflow (no stdout/pty bytes, no env dumps).

### Config
- Rule declarations live in config (not policy), but are always gated by policy before executing any resulting actions (see Decision Register DR-0006).
- Rules are loaded from two locations:
  - Global: `SUBSTRATE_HOME/workflow/rules.yaml`
  - Workspace: `<workspace_root>/.substrate/workflow/rules.yaml`
- For a given `rule_id`, the workspace-scoped rule overrides a global rule with the same `rule_id`.
- If multiple rules resolve to the same `rule_id` at the same scope, it is a hard error (fail-closed for that rules file).

#### Rule file schema (v1; strict)
- `rules.yaml` is a strict YAML mapping with:
  - `version: 1`
  - `rules: []`
- Each rule is a strict mapping:
  - `id` (string; unique within the file)
  - `enabled` (bool; default `true`)
  - `when` (mapping)
  - `then` (mapping)
- v1 triggers:
  - `when.event_type` MUST be one of: `command_complete`, `fs_change`
  - If `when.event_type == "fs_change"`:
    - `when.ops` is an optional list of: `create`, `modify`, `delete`, `rename` (default: all)
    - `when.paths.include` and `when.paths.exclude` are lists of ADR-0018 matchers (workspace-relative)
  - If `when.event_type == "command_complete"`:
    - `when.exit_codes` is an optional list of integers (default: all)
- v1 actions:
  - `then.target_workspace_id` is a UUID string (required)
  - `then.action.kind` MUST be `workflow.run`
  - `then.action.workflow_id` is a string (required)
  - `then.action.args` is an optional list of strings (default: empty)

### CLI
- `substrate workflow serve [--foreground]`
  - Behavior: runs the workflow router service.
  - Exit codes:
    - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
    - `0`: clean shutdown
    - `4`: service cannot start due to platform prerequisites
- `substrate workflow status [--json]`
  - Behavior: reports whether the workflow router service is running and healthy (see Decision Register DR-0011), plus the current trace cursor and queue sizes.
  - Exit codes: `0` success; `4` service not available.
- `substrate workflow workspaces list [--json]`
  - Behavior: prints registry entries (workspace_id, root, enabled).
  - Exit codes: `0` success; `3` registry read/parse failure.
- `substrate workflow doctor [--json]`
  - Behavior: validates that the workflow router can start deterministically and fail-closed where required:
    - trace follower can load cursor state and validate rotation handling inputs
    - workspace registry is readable and workspace roots are resolvable
    - rule files parse strictly (schema) and only use the v1 trigger allowlist families
  - Exit codes:
    - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
    - `0`: all checks pass
    - `2`: config/schema error
    - `3`: required dependency unavailable (trace path missing/unreadable, registry missing)
    - `4`: unsupported / missing prerequisites for required posture
    - `5`: policy/safety violation (explicit deny)
- `substrate workflow run <workflow_id> [-- <args...>]`
  - Behavior: runs a workflow instance. This ADR reserves the command shape; execution semantics are defined by Phase 7 (ADR-0021 / ADR-0022).

### Platform guarantees
- Linux: the workflow router service can run under systemd (user or system instance). Service managers MUST be able to invoke `substrate workflow serve --foreground`.
- macOS: the workflow router service can run under launchd (agent or daemon). launchd MUST be able to invoke `substrate workflow serve --foreground`.
- Windows: the workflow router service can run as a Windows service or scheduled task. The service wrapper MUST invoke `substrate workflow serve --foreground`.

## Architecture Shape
- Workflow router service:
  - tails `SUBSTRATE_HOME/trace.jsonl` with durable cursor state
  - applies routing rules (workspace + global)
  - writes requests to inbox / work_queue
  - emits derived trace events for:
    - rule match
    - request enqueued
    - request denied / pending approval / executed
- Workspace attribution:
  - For each consumed trace record, the router determines the owning workspace by resolving `cwd` to a workspace root (marker: `<workspace_root>/.substrate/workspace.yaml`) and reading its `workspace_id` from `<workspace_root>/.substrate/workspace_id`.
  - If no enabled workspace is resolvable for an event `cwd`, the router MUST treat it as non-routable (emit a derived deny/failure event with a stable reason code).
- Service boundary:
  - the workflow router is a host service (not in-world) and must remain available in host-only mode.
  - it MAY reuse transport patterns and code organization from world-agent, but must not depend on world-agent being available.
- Event recursion guard:
  - workflow-router-emitted events MUST be identifiable (e.g., `component=workflow_router`) and MUST be excluded from re-trigger evaluation by default to avoid infinite loops.
- FS trigger derivation:
  - the workflow router derives file-change trigger inputs from fs diff events already persisted to trace, and applies ADR-0018 matching semantics for include/exclude.

## Sequencing / Dependencies
- Prerequisites:
  - ADR-0028 must land first (trace schema/correlation + redaction requirements; span parent correctness).
  - ADR-0017 must be stable (structured output vs PTY bytes; routing attribution).
  - ADR-0018 path semantics must be treated as authoritative for fs-trigger matching.
- Follow-on alignment:
  - World-sync/internal git support can provide higher-quality change classification later, but is not required for v1.

## Security / Safety Posture
- Fail-closed rules:
  - No workflow-router-triggered execution may occur without policy evaluation under the target workspace.
  - File triggers must not become an exfil channel: only path metadata required for routing is used by default; content is not used.
  - Workflow router policy gating uses explicit `workflow.router.*` keys (see Decision Register DR-0017).
  - v1 policy keys (exact; patch-only under global/workspace `policy.yaml` per ADR-0013):
    - `workflow.router.enabled` (bool; default `false`)
    - `workflow.router.allow_cross_workspace` (bool; default `false`)
    - `workflow.router.allowed_rule_ids` (list; default `[]`)
    - `workflow.router.allowed_workflow_ids` (list; default `[]`)
    - `workflow.router.allowed_target_workspace_ids` (list; default `[]`)
- Protected paths/invariants:
  - All workflow-router state lives under `SUBSTRATE_HOME` with user-only permissions.
  - Requests are durable and auditable; every request produces an observable trace record of allow/deny/approval outcomes.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - cursor persistence and idempotency/dedupe behavior (at-least-once handling without runaway repeats)
  - workspace registry read/write behavior and `workspace_id` stability
  - path matching semantics are identical to ADR-0018 matcher behavior
- Integration tests:
  - cross-workspace routing: workspace A event produces request targeting workspace B and evaluates under B policy
  - file triggers: only configured include paths trigger (deny all other changes)

### Manual validation
- Manual playbook: `docs/project_management/next/host_event_bus_router_daemon/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/host_event_bus_router_daemon/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/host_event_bus_router_daemon/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/host_event_bus_router_daemon/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/host_event_bus_router_daemon/decision_register.md`:
    - DR-0001 (Service packaging: standalone binary vs `substrate` subcommand)
    - DR-0002 (Event ingestion: tail `trace.jsonl` vs direct publish API)
    - DR-0003 (Derived workflow-router events location: append to `trace.jsonl` vs separate log)
    - DR-0004 (Durable queue format: JSONL + state vs sqlite)
    - DR-0005 (Workspace identity: path-hash id vs explicit id stored in workspace metadata)
    - DR-0006 (Rule declarations: config vs policy)
    - DR-0007 (Trigger taxonomy: strict allowlist vs general event matching)
    - DR-0008 (FS triggers source: fs diffs vs external watchers/git feeds)
    - DR-0009 (Remote ingress: none in v1 vs authenticated inbound requests)
    - DR-0010 (Idempotency key strategy: deterministic derived key vs random per request)
    - DR-0011 (Service liveness + single-instance detection: file heartbeat vs control socket)
    - DR-0012 (Trace cursor model: inode+offset+rewind vs event-key scan)
    - DR-0013 (`state.json` schema: single versioned doc vs multiple files)
    - DR-0014 (Ack model: cursor-based append-only vs in-queue status mutation)
    - DR-0015 (Retry/backoff + rate limiting: bounded retries vs no retries)
    - DR-0016 (Derived event taxonomy: explicit event types vs generic wrapper)
    - DR-0017 (Policy gating key paths: dedicated workflow keys vs reuse generic cmd gating)
    - DR-0018 (Registry drift rules: fail-closed vs auto-repair)
    - DR-0019 (Approval handling: hold pending vs treat-as-deny)
    - DR-0020 (v1 action kinds: workflow-only vs general-purpose actions)
