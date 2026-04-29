<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-orchestration-session-identity-autoplan-restore-20260429-130952.md -->
# PLAN-02: Session Participant Record Cutover

Source file: [02-session-participant-record.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/02-session-participant-record.md)  
Branch: `feat/orchestration-session-identity`  
Plan type: backend-only, no UI scope  
Review posture: `/autoplan` completeness pass with `/plan-eng-review` structure and rigor  
Status: execution-ready

## What This Plan Does

`PLAN-01` fixed the parent session authority boundary. The shell now persists a real orchestration session record under [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:23) and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:34).

The next bottleneck is the child runtime model. The live record is still an orchestrator-shaped manifest in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:36), and the main operator surface still collapses live state by `agent_id` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:617). That means the repo can talk about member sessions, but the authoritative runtime store still cannot represent them honestly.

This slice replaces that narrow child-handle model with a participant record that can represent:

1. the host-scoped orchestrator participant,
2. future host-scoped or world-scoped member participants under the same `orchestration_session_id`,
3. replacement lineage after invalidation or restart, and
4. additive participant correlation on status and `AgentEvent` surfaces.

The goal is concrete:

1. introduce a canonical participant record and participant store,
2. read legacy `handles/*.json` compatibly during rollout,
3. stop collapsing live state to one row per `agent_id`,
4. keep toolbox authorization host-orchestrator-only and fail-closed, and
5. add participant identity to runtime events without breaking existing tuple consumers.

This is the participant-model slice. It is not the shared-world owner slice, not the restart-generation slice, and not the grouped session-store rewrite.

## Scope Challenge

### Why this is the right second slice

The repo already has the parent session object from `PLAN-01`. Without this slice, every later world/member feature still has to pretend that one orchestrator-shaped handle is "the runtime." That is the wrong source of truth for multi-participant orchestration.

What matters is not inventing more machinery. What matters is making the already-existing runtime store honest. Right now the docs can describe members and lineage, but the authoritative live registry still cannot. That mismatch will bite status, toolbox, restart invalidation, and any future `session_fork` or `session_resume` work.

### What already exists

| Sub-problem | Existing code | Reuse or replace |
|---|---|---|
| Parent orchestration identity | [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:23) | Reuse exactly |
| Atomic JSON persistence | [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:65) | Reuse exactly |
| Live-ownership rules | [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:179) | Reuse, but move behind participant semantics |
| Orchestrator bootstrap writer | [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1374) | Extend |
| Role vocabulary | [mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs:5) | Reuse exactly |
| Status tuple parsing and nested-row correlation | [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:457) | Extend, do not replace |
| Parent-gated live orchestrator lookup | [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:225) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1072) | Rework into participant query helpers |
| Canonical event envelope | [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:71) | Reuse with additive fields only |
| Contract fixtures | [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:263) | Reuse and widen |

### Minimum change that achieves the goal

Do this, and only this:

- replace the orchestrator-specific manifest type with a participant record type,
- add a `participants/` directory and participant query APIs,
- keep legacy `handles/` as a compatibility read path during rollout,
- convert orchestrator bootstrap to write the participant shape,
- make status/toolbox/doctor consume participant records first,
- add additive participant fields to `AgentEvent`,
- update docs and tests.

Do not:

- invent a long-lived agent-hub service,
- redesign nested gateway status rows,
- pull slice `03`, `04`, `05`, or `06` into this plan,
- add a cache layer to the runtime store,
- invent a generic member lifecycle framework before the shell owns a concrete member launch seam.

### Complexity check

This slice touches more than 8 files. That is normally a smell. Here it is acceptable because the authority seam already crosses:

- `crates/common/src/agent_events.rs`
- `crates/shell/src/execution/agent_runtime/`
- `crates/shell/src/repl/async_repl.rs`
- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
- `crates/common/tests/agent_hub_event_envelope_schema.rs`
- `docs/USAGE.md`
- `docs/TRACE.md`
- pack/ADR docs

The guardrail is simple: no new daemon, no new cache, no speculative registry abstraction, no generic "participant manager" service object. Keep it boring.

### Search-before-building posture

`[Layer 1]` wins here. The repo already has the right building blocks:

- atomic JSON writes,
- parent session persistence,
- liveness validation,
- role constants,
- fail-closed operator surfaces,
- tuple-compatible event schema.

The correct move is to reshape those into a participant model, not to introduce a second persistence engine or a runtime cache. This is one of those classic software moments where the fancy option is worse. A 200-line cache to avoid reading a few small JSON files would be software comedy.

### Hard non-goals

- shared-world ownership authority from [03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)
- world-binding promotion from slice `04`
- restart invalidation behavior from slice `05`
- session-centric grouped store layout from slice `06`
- public resume/fork/stop CLI productization
- nested gateway record redesign

## Architecture Contract

### No-ambiguity rules

These are hard rules:

1. There is exactly one canonical runtime child record type after this slice: a participant record. "Manifest" becomes a compatibility detail, not the public model.
2. `participant_id` is the canonical child identity. No status or toolbox surface may infer identity from `agent_id` alone.
3. Parent orchestration sessions remain the sole authority for the control-plane session. Participants are children, not replacements for the parent record.
4. `role=orchestrator` plus `execution.scope=host` is the only combination that can authorize toolbox publication.
5. World-scoped participants require both `world_id` and `world_generation`. Host-scoped participants omit both.
6. Doctor stays pre-runtime. It may validate participant files when present, but it must not require a live participant registry for a healthy result.
7. Event-schema changes are additive only. Existing consumers that ignore participant fields must keep working.
8. Legacy `handles/*.json` files are read compatibly during rollout, but new writes go to `participants/*.json` only.
9. Every participant lookup that could return more than one valid answer must fail closed. No "latest by agent" heuristics.
10. The absence of a production member-launch seam is not permission to keep the store orchestrator-shaped. It is permission to ship the participant-capable substrate first and wire later slices into it.

### Target data model

Replace [AgentRuntimeSessionManifest](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:98) with a participant-centric shape:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantExecution {
    pub scope: AgentExecutionScope,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantHandle {
    pub participant_id: String,
    pub orchestration_session_id: String,
    pub agent_id: String,
    pub backend_id: String,
    pub role: String, // orchestrator | member
    pub protocol: String,
    pub execution: AgentRuntimeParticipantExecution,
    pub state: AgentRuntimeSessionState,
    pub opened_at: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: Option<String>,
    pub orchestrator_participant_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantRecord {
    #[serde(flatten)]
    pub handle: AgentRuntimeParticipantHandle,
    pub internal: AgentRuntimeSessionInternal,
}
```

Implementation rules:

- `participant_id` may reuse the current `session_handle_id` format (`ash_<uuid>`) for the first cut. No need to spend an innovation token on ID syntax.
- `AgentRuntimeSessionInternal` stays in place. It already holds the right ephemeral ownership fields.
- `orchestrator_participant_id` is `None` on the orchestrator participant and required on member participants.
- `parent_participant_id` is used only for forked or explicitly derived children.
- `resumed_from_participant_id` is used only for replacement lineage after invalidation or restart.

### Ownership mode generalization

Current `ownership_mode` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:78) is effectively hard-coded to attached orchestrator control. That has to widen:

```rust
enum AgentRuntimeOwnershipMode {
    AttachedControl,
    MemberRuntime,
    Replaced,
}
```

Semantics:

- `attached_control`: current shell-owned orchestrator lifecycle
- `member_runtime`: live member participant owned by shell/runtime logic
- `replaced`: prior participant retained only for lineage and audit

Do not use free-form strings forever. Keep the first patch additive if needed, then move to a proper enum before this slice closes.

### Participant lifecycle contract

`PLAN-01` already owns the parent session lifecycle. This slice must make participant lifecycle line up with it instead of freelancing its own truth.

| Moment | Parent state | Participant state | Notes |
|---|---|---|---|
| parent exists, no child yet | `Allocating` | no participant yet | child creation never becomes the thing that legitimizes the parent |
| participant file first persisted | `Allocating` | `Allocating` | bootstrap may write the child record before durable control is proven |
| durable control proven | `Active` | `Ready` | this is the first point where the participant may become authoritative-live |
| command loop actively serving | `Active` | `Running` | status and event emission may surface the participant |
| graceful shutdown requested | `Stopping` | `Stopping` | parent and participant transition in the same shutdown block |
| graceful shutdown completed | `Stopped` | `Stopped` | historical row remains queryable, but never live |
| ownership lost after prior success | `Invalidated` | `Invalidated` | toolbox and live status must fail closed immediately |
| replacement lineage recorded later | `Active` on the replacement parent-child pair | `Replaced` on the old participant | replaced rows remain auditable but never authoritative-live |

Hard rules:

- a participant is authoritative-live only when both its own ownership checks pass and its parent session is `Active`
- `Replaced`, `Stopped`, `Failed`, and `Invalidated` participants persist for audit, but they do not suppress a distinct live participant unless the `participant_id` matches exactly
- this slice does not add a participant-specific restart state machine; it adds the identity and lineage fields that later restart work will consume

### On-disk layout and compatibility rules

Target directory:

```text
~/.substrate/run/agent-hub/participants/<participant_id>.json
```

Compatibility rules:

1. Read both:
   - `~/.substrate/run/agent-hub/handles/*.json`
   - `~/.substrate/run/agent-hub/participants/*.json`
2. Write only:
   - `~/.substrate/run/agent-hub/participants/*.json`
3. Upgrade legacy handle files in memory using:
   - `session_handle_id -> participant_id`
   - `parent_session_handle_id -> parent_participant_id`
   - `resumed_from_session_handle_id -> resumed_from_participant_id`
   - `orchestrator_participant_id = None` for legacy orchestrator rows
4. When both a legacy handle and a participant record describe the same child identity, prefer the participant file.
5. Do not attempt an eager file-system migration step in this slice. Read-compat is enough.

### Required store APIs

Extend [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:20) with participant-native helpers:

```rust
pub(crate) fn participants_dir(&self) -> PathBuf;
pub(crate) fn persist_participant(&self, participant: &AgentRuntimeParticipantRecord) -> Result<()>;
pub(crate) fn load_participant(&self, participant_id: &str) -> Result<Option<AgentRuntimeParticipantRecord>>;
pub(crate) fn list_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>>;
pub(crate) fn list_live_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>>;
pub(crate) fn list_live_participants_for_session(
    &self,
    orchestration_session_id: &str,
) -> Result<Vec<AgentRuntimeParticipantRecord>>;
pub(crate) fn resolve_live_orchestrator_participant(
    &self,
    orchestrator_agent_id: &str,
) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeParticipantRecord)>>;
pub(crate) fn validate_participant_record(
    &self,
    participant: &AgentRuntimeParticipantRecord,
) -> Result<()>;
```

Rules:

- `list_live_participants()` keeps the existing ownership checks from [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:224).
- `resolve_live_orchestrator_participant()` filters to `role=orchestrator` and `execution.scope=host`, then parent-gates through the orchestration session record.
- any ambiguous live answer is an error, not a winner selection.
- operator-path helpers may scan the store on demand, but event emission paths must not.

### Status contract

This is the core bug.

Today:

- trace-derived pure-agent rows are keyed by `(orchestration_session_id, agent_id)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:434),
- live runtime rows are collapsed by `agent_id` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:617),
- fallback suppression is also keyed by `agent_id` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:648).

That loses same-agent siblings by construction.

Required change:

1. Add additive `participant_id` to every pure-agent status row.
2. Make participant-aware rows key by `(orchestration_session_id, participant_id)`.
3. Keep legacy trace fallback for pre-participant events only, and isolate it from live participant rows.
4. Sort `sessions` by:
   - `orchestration_session_id`
   - `participant_id`
   - `agent_id`
5. Keep nested rows tied to `parent_run_id`, but widen their parent shape to include `parent.participant_id`.

Recommended status row shape:

```json
{
  "participant_id": "ash_001",
  "orchestration_session_id": "sess_001",
  "agent_id": "codex",
  "backend_id": "cli:codex",
  "client": "codex",
  "router": "agent_hub",
  "protocol": "uaa.agent.session",
  "execution": { "scope": "world" },
  "role": "member",
  "last_event_at": "2026-04-29T12:55:04Z",
  "world_id": "world-123",
  "world_generation": 7
}
```

This is additive. Existing consumers can ignore `participant_id`, but the operator surface stops being lossy.

### Toolbox contract

Toolbox stays boring and strict.

Required changes:

- replace the manifest-only resolver in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:929) with a participant query,
- require:
  - `role=orchestrator`
  - `execution.scope=host`
  - authoritative live ownership
  - active parent orchestration session
- keep endpoint derivation on `orchestration_session_id`.

Member participants never authorize toolbox publication. Not once. Not "if they're the newest." Not if the trace looks convincing. No cute shortcuts.

### Doctor contract

Doctor remains pre-runtime.

Required change:

- after the current config, policy, orchestrator, and world checks in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1302), add a passive participant-store validation check:
  - `pass` when no participant files exist
  - `pass` when participant files parse and obey role/scope/world invariants
  - `fail` when participant files exist but are malformed or impossible

Examples of impossible participant state:

- `role=orchestrator` with `execution.scope=world`
- `role=member` with missing `orchestrator_participant_id`
- `execution.scope=world` with only one of `world_id` or `world_generation`
- `resumed_from_participant_id` pointing at itself

### Event publication contract

Additive fields on [AgentEvent](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:74):

```rust
pub participant_id: Option<String>,
pub parent_participant_id: Option<String>,
pub resumed_from_participant_id: Option<String>,
```

Producer rules:

- [translate_wrapper_event()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2187) and [build_runtime_message_event()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2261) populate participant fields from the live participant record.
- world restart alerts in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2473) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2518) stay additive. If no participant context exists, they omit participant fields instead of inventing them.
- nested gateway-backed records remain separate. They do not become participant rows.

### Member-writer posture

There is no concrete production member-launch seam on this branch yet. That is fine. The plan should not invent one.

What this slice must do:

- make the participant model capable of representing members,
- add constructors and store APIs for member participants,
- cover member rows in fixtures and contract tests,
- keep the first production writer hookup small and shell-side once slice `03` or `04` provides the concrete member lifecycle.

What this slice must not do:

- create a fake generic member runtime service just so the plan can say it wrote "something."

That would be under-tested complexity pretending to be progress.

## Architecture Diagrams

### Runtime store and consumer flow

```text
CURRENT
=======
Parent session: sessions/<orchestration_session_id>.json
        │
        └── Child runtime: handles/<session_handle_id>.json
                │
                ├── status live path collapses by agent_id
                ├── toolbox resolves one live orchestrator manifest
                └── events carry no participant lineage

TARGET
======
Parent session: sessions/<orchestration_session_id>.json
        │
        ├── participant: participants/<participant_id>.json   role=orchestrator scope=host
        ├── participant: participants/<participant_id>.json   role=member       scope=host
        └── participant: participants/<participant_id>.json   role=member       scope=world
                │
                ├── status resolves by participant_id
                ├── toolbox filters to live host orchestrator participant only
                ├── doctor validates participant files when present
                └── AgentEvent publishes participant_id + lineage fields additively
```

### Status and toolbox lookup

```text
substrate agent status --json
    │
    ├── read live participants
    │       ├── parent-gate orchestrator row
    │       ├── include member rows without collapsing siblings
    │       └── suppress legacy trace fallback only when a matching participant_id exists
    │
    └── read trace agent events
            ├── participant-aware rows key by (session, participant_id)
            ├── legacy rows stay fallback-only
            └── nested rows correlate by parent_run_id + parent.participant_id

substrate agent toolbox status|env
    │
    └── resolve live orchestrator participant
            ├── role=orchestrator
            ├── execution.scope=host
            ├── parent session active
            └── endpoint => <orchestration_session_id>.sock
```

## Implementation Plan

### Ordered execution checklist

1. Lock the participant contract in the pack/ADR docs and status JSON examples.
2. Replace the child manifest model with a participant record plus compatibility parser.
3. Add `participants/` storage and participant query helpers to the runtime store.
4. Convert orchestrator bootstrap to write participant records.
5. Update status, toolbox, and doctor to consume participant records first.
6. Add additive participant lineage to `AgentEvent` and runtime producers.
7. Extend tests and docs. Remove handle-only assumptions once coverage is green.

### Execution sequencing and merge gates

This slice only stays boring if the sequencing is strict:

1. Land the type rename plus compatibility parser first.
   Exit gate: legacy `handles/*.json` fixtures still parse into the participant model.
2. Land participant-native store helpers second.
   Exit gate: read-path tests prove `participants/` wins over `handles/` for the same child identity, and ambiguous live answers fail closed.
3. Cut over the orchestrator writer third.
   Exit gate: bootstrap writes `participants/*.json` only, and parent-child linkage still resolves through the existing parent record.
4. Cut over operator surfaces fourth.
   Exit gate: `status`, `toolbox`, and `doctor` all consume participant-native helpers and no remaining live projection deduplicates by `agent_id`.
5. Land event schema and runtime publication after the participant surface is stable.
   Exit gate: additive participant lineage fields roundtrip without breaking legacy consumers.
6. Run the contract test and doc sweep last.
   Exit gate: status JSON examples, trace docs, and integration fixtures all use participant terminology and sibling-safe semantics.

Do not merge a later step while it still depends on an unstabilized helper signature from an earlier step. That is how a clean slice turns into a half-renamed swamp.

### Workstream 1: Participant model and compatibility parser

Files:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:36)
- [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs:1)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md)

Do:

- rename or replace `AgentRuntimeSessionHandle` / `AgentRuntimeSessionManifest` with participant naming,
- keep a compatibility deserializer for legacy handle JSON,
- add constructors:
  - `new_orchestrator_participant(...)`
  - `new_member_participant(...)`
  - `new_replacement_participant(...)`
- codify role/scope/world invariants close to the model.

### Workstream 2: Participant store and parent-gated queries

Files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:20)

Do:

- add `participants_dir()` and participant persistence helpers,
- read `participants/*.json` first, `handles/*.json` second,
- keep parent orchestration-session APIs unchanged,
- replace `find_live_orchestrator()` with participant-native query helpers,
- expose a validation helper reused by doctor and tests.

### Workstream 3: Orchestrator writer cutover

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1374)

Do:

- allocate `participant_id` instead of `session_handle_id`,
- persist participant records under `participants/`,
- update parent `active_session_handle_id` naming only if the parent record is widened in this slice; otherwise keep the serialized parent field stable and document that it now points to the active participant id,
- thread the participant snapshot through ready/running/stopping/invalidated transitions.

Recommendation:

Keep the parent record field name stable for this slice if renaming it would force a bigger cross-packet doc churn. The important thing is the child identity model, not winning a naming purity contest today.

### Workstream 4: Status, toolbox, and doctor cutover

Files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:406)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md)

Do:

- add `participant_id` to status rows,
- stop collapsing live rows by `agent_id`,
- widen nested parent shape to include `parent.participant_id`,
- make toolbox resolve from participant query helpers,
- add passive doctor validation for present participant files.

### Workstream 5: Event envelope and runtime emission

Files:

- [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:71)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2187)
- [crates/common/tests/agent_hub_event_envelope_schema.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/tests/agent_hub_event_envelope_schema.rs)

Do:

- add additive participant fields to `AgentEvent`,
- populate them on pure-agent runtime events,
- keep omission rules strict when lineage does not apply,
- document that legacy traces may omit participant fields.

### Workstream 6: Contract tests and docs

Files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Do:

- add participant fixtures,
- add sibling-same-agent status coverage,
- add malformed participant-file doctor coverage,
- update status JSON examples and trace docs,
- remove wording that says live discovery is manifest-backed.

## Architecture Review Findings

### Finding 1

`[P1] (confidence: 10/10) [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:434) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:617) collapse runtime truth by `(orchestration_session_id, agent_id)` and then by `agent_id` alone.`

This is the main correctness bug. Two live member participants for the same agent under one orchestration session cannot both survive projection. The plan fixes it by making `participant_id` the canonical child key on both live and trace-backed rows.

Recommendation: accept this as the primary slice objective and keep the fix explicit. No heuristics. No "latest row wins" fallback for participant-aware data.

### Finding 2

`[P1] (confidence: 9/10) [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:106) still constructs every child record as `role=orchestrator` with `ownership_mode=attached_control`.`

That makes the current model semantically incapable of representing member state. The plan resolves this by renaming the model, adding member constructors, and widening `ownership_mode`.

Recommendation: do not bolt member fields onto an orchestrator-named type. Rename the thing and move on.

### Finding 3

`[P1] (confidence: 9/10) [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:202) and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:225) still expose orchestrator-specific lookup helpers.`

The store can validate one active orchestrator, but it cannot answer the real questions the later packet needs:

- list live participants for one orchestration session,
- load one participant by id,
- validate participant role/scope combinations,
- keep same-agent siblings distinct.

Recommendation: make the store participant-native now. Do not stack more logic on top of orchestrator-only helpers.

### Finding 4

`[P2] (confidence: 8/10) [AgentEvent](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:74) carries no participant lineage, while runtime producers in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2187) already stamp role/backend/world identity.`

Without `participant_id`, later restart/fork lineage becomes hard to correlate. The plan resolves this with additive fields only.

Recommendation: add the fields now while the participant model is being introduced. Waiting until slice `05` would force a second schema churn.

## Code Quality Review

This slice has a real DRY risk because it is migrating a live type, a store boundary, and operator surfaces at the same time. The plan needs explicit guardrails so the implementation does not leave two truths behind.

### Finding 1

`[P2] (confidence: 9/10) The role/scope/world invariants can easily get duplicated across constructors, store validation, doctor checks, and status projection if the implementation is careless.`

That is how these migrations rot. One path starts rejecting `role=member` without `orchestrator_participant_id`, another path forgets, and six weeks later the store contains impossible state that only one command notices.

Recommendation: put participant invariants in one model-level validation path and have store, doctor, and tests call it. Do not hand-roll the same rules three times.

### Finding 2

`[P2] (confidence: 8/10) Legacy-handle compatibility can sprawl if the upgrade logic leaks beyond the read boundary in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:20).`

If `agents_cmd.rs`, `async_repl.rs`, and tests all know how to translate `session_handle_id` into `participant_id`, the repo will carry two mental models for far too long. Not great.

Recommendation: keep the legacy upgrade shim at the persistence boundary. Everyone above the store should speak participant terminology only.

### Finding 3

`[P2] (confidence: 8/10) Terminology drift will create fake complexity if public APIs keep saying "manifest" or "handle" after the canonical type becomes a participant record.`

This matters because later slices will read the new helper names as the contract. If the code says `find_live_orchestrator_manifest()` but returns a participant, the type system compiles and the humans still lose.

Recommendation: reserve `handles/` and `manifest` wording for the compatibility path and on-disk legacy explanation only. New helpers, docs, and test names should say `participant`.

## Error & Rescue Registry

These are the failure cases the implementation must rescue explicitly instead of letting them become quiet operator drift.

| Failure | Detection point | Required rescue behavior | Fail-open forbidden? |
|---|---|---|---|
| malformed `participants/*.json` | participant load path, doctor | reject the file, surface explicit validation failure, keep toolbox unauthorized | yes |
| legacy handle file cannot upgrade cleanly | compatibility parser | omit that record from live selection, return a clear error on validation surfaces, preserve other healthy rows | yes |
| same child identity exists in both `handles/` and `participants/` | store enumeration | prefer `participants/`, ignore legacy duplicate, never merge fields across the two records | yes |
| parent session points at a child id that is missing or non-live | parent-gated lookup | fail closed for toolbox and live status projection, require bootstrap or cleanup to repair | yes |
| multiple live host orchestrator participants survive lookup | orchestrator resolver | return ambiguity, not newest-wins, and block toolbox publication | yes |
| runtime event path lacks a live participant snapshot | runtime event builder | omit additive participant lineage fields instead of inventing or backfilling them from trace history | yes |

Operator rescue rule:

- `substrate agent doctor --json` is the place that explains bad participant-store state to the operator
- `substrate agent status --json` may show historical trace rows, but it must never use trace history to re-authorize a broken live participant
- `substrate agent toolbox status|env` stays stricter than status, because a wrong endpoint is worse than no endpoint

## Test Review

100% coverage is the goal for the new participant contract. This slice changes identity, lookup, and fail-closed behavior. That is exactly the kind of change that quietly passes the demo path and then burns you later unless the tests are over-complete.

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/session.rs
    │
    ├── [GAP] Participant constructor: orchestrator record emits participant_id, no world fields on host
    ├── [GAP] Participant constructor: member record requires orchestrator_participant_id
    ├── [GAP] Compat upgrader: legacy handle JSON maps session_handle_id -> participant_id
    ├── [GAP] Validation: world-scoped member rejects missing world_id/world_generation
    └── [GAP] Validation: host-scoped orchestrator rejects world fields
 
[+] crates/shell/src/execution/agent_runtime/state_store.rs
    │
    ├── [GAP] participants/ write + load roundtrip
    ├── [GAP] participants/ preferred over legacy handles/ for same child identity
    ├── [GAP] list_live_participants filters dead owner PID and invalid ownership
    ├── [GAP] resolve_live_orchestrator_participant fails closed on ambiguity
    └── [GAP] list_live_participants_for_session preserves same-agent siblings

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [GAP] host orchestrator bootstrap writes participant record instead of handle manifest
    ├── [GAP] ready/running/stopping/stopped transitions persist participant snapshots
    └── [GAP] runtime events publish participant_id on orchestrator lifecycle rows

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── [GAP] status JSON includes participant_id on live rows
    ├── [GAP] same-agent sibling participants both appear in one session
    ├── [GAP] legacy trace fallback does not suppress distinct live siblings
    ├── [GAP] toolbox status/env accepts only live host orchestrator participant
    └── [GAP] doctor passive participant validation fail-closes malformed files

[+] crates/common/src/agent_events.rs
    │
    ├── [GAP] participant_id roundtrips when present
    ├── [GAP] parent_participant_id omits by absence when unset
    └── [GAP] resumed_from_participant_id omits by absence when unset

─────────────────────────────────
COVERAGE: 0/20 paths tested (0%)
  Code paths: 0/20
QUALITY:  ★★★: 0  ★★: 0  ★: 0
GAPS: 20 paths need tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] substrate agent status --json
    │
    ├── [GAP] [→E2E] Live host orchestrator + live world member sibling both render
    ├── [GAP] [→E2E] Two same-agent member participants under one orchestration session both render
    ├── [GAP]         Legacy trace-only session still renders without participant_id
    └── [GAP]         Malformed participant/world identity fails closed with explicit reason

[+] substrate agent toolbox status --json
    │
    ├── [GAP]         Live host orchestrator participant yields endpoint
    ├── [GAP]         Member-only live state does not authorize endpoint
    └── [GAP]         Ambiguous live orchestrator participants fail closed

[+] substrate agent toolbox env --json
    │
    ├── [GAP]         Allowed orchestrator participant exports endpoint/version
    └── [GAP]         Trace history alone does not resurrect participant authorization

[+] substrate agent doctor --json
    │
    ├── [GAP]         No participant files remains healthy
    ├── [GAP]         Malformed participant file fails closed
    └── [GAP]         Impossible role/scope/world combination fails closed

[+] Runtime event correlation
    │
    ├── [GAP] [→EVAL] Participant lineage fields survive schema roundtrip
    └── [GAP]         Runtime lifecycle rows omit lineage fields cleanly when not applicable

─────────────────────────────────
COVERAGE: 0/13 flows tested (0%)
  User flows: 0/13
GAPS: 13 flows need tests (2 integration-style, 1 schema/eval-style)
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/session.rs`

Add unit tests for:

- orchestrator participant constructor
- member participant constructor
- legacy handle upgrade
- validation of role/scope/world invariants
- replacement lineage constructor

#### `crates/shell/src/execution/agent_runtime/state_store.rs`

Add unit tests for:

- participants-dir read/write
- dual-read behavior with `participants/` preferred over `handles/`
- live participant enumeration keeping same-agent siblings
- ambiguous orchestrator participant failure
- doctor validation helper on malformed participant JSON

#### `crates/shell/src/repl/async_repl.rs`

Extend the existing bootstrap tests to assert:

- participant file exists under `participants/`
- parent session still points at the active child identity
- runtime lifecycle events publish `participant_id`
- stopped and invalidated orchestrator rows persist participant snapshots, not handle manifests

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add integration coverage for:

- two live participants with same `agent_id` under one `orchestration_session_id`
- `participant_id` presence on status JSON rows
- toolbox authorization from live host orchestrator participant only
- doctor passive validation of malformed participant files
- legacy trace fallback when no participant record exists

#### `crates/common/tests/agent_hub_event_envelope_schema.rs`

Add schema tests for:

- `participant_id`
- `parent_participant_id`
- `resumed_from_participant_id`
- omission rules when unset

### Test commands

Run at minimum:

```bash
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell participant -- --nocapture
cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture
```

Then run:

```bash
cargo test -p substrate-shell -- --nocapture
cargo test --workspace -- --nocapture
```

### QA artifact

Primary QA artifact for follow-up verification:

[spensermcconnell-feat-orchestration-session-identity-eng-review-test-plan-20260429-125504.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-orchestration-session-identity-eng-review-test-plan-20260429-125504.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
|---|---|---|---|---|---|
| legacy handle upgrade | old `handles/*.json` cannot map to participant fields and disappears from status | planned | yes | yes | no |
| participant write path | orchestrator bootstrap writes participant file but parent still points to stale child id | planned | yes | yes | no |
| same-agent sibling projection | second member silently disappears because status still keys by `agent_id` | planned | yes | yes | yes until fixed |
| toolbox authorization | member-only participant set yields a toolbox endpoint | planned | yes | yes | yes until fixed |
| passive doctor validation | malformed participant file is ignored and doctor reports healthy | planned | yes | yes | yes until fixed |
| event publication | runtime events omit participant lineage after migration | planned | yes | yes | no |
| restart lineage | replacement participant reuses prior id and loses causal chain | planned | yes | yes | no |
| live query ambiguity | two live host orchestrator participants for one agent newest-win silently | planned | yes | yes | yes until fixed |

Critical gap rule:

If any operator surface can still collapse distinct live participants or authorize toolbox from a non-orchestrator participant, the slice is not done.

## Performance Review

No major performance concern if the implementation stays direct.

Hard rules:

- do not scan `participants/` on runtime hot event paths
- operator commands may scan participant files because they are interactive control-plane surfaces, not high-frequency loops
- when a participant already knows its parent `orchestration_session_id`, use direct parent lookup
- do not add a cache just to avoid reading a handful of JSON files

Footguns to avoid:

1. Reading both `participants/` and `handles/` on every event emission path. Wrong place.
2. Rebuilding "latest participant by agent" maps after this slice. That reintroduces the exact bug we are fixing.
3. Renaming too many fields in one pass. The cheapest correct move is additive fields plus compatibility parsing, not a flag day.

## Worktree Parallelization Strategy

There is a real parallel split once the shared type surface lands.

### Dependency table

| Step | Modules touched | Depends on |
|---|---|---|
| Participant model + compat parser | `crates/shell/src/execution/agent_runtime/`, pack docs | - |
| Participant store queries | `crates/shell/src/execution/agent_runtime/` | participant model |
| Orchestrator writer cutover | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | participant model + store |
| Status/toolbox/doctor cutover | `crates/shell/src/execution/`, docs | participant model + store |
| Event schema + runtime publication | `crates/common/src/`, `crates/shell/src/repl/` | participant model |
| Tests + docs cleanup | `crates/shell/tests/`, `crates/common/tests/`, `docs/` | all above |

### Parallel lanes

Lane A: participant model + compat parser -> participant store queries -> orchestrator writer cutover  
Why: these steps share `agent_runtime/` and define the child identity surface.

Lane B: status/toolbox/doctor cutover  
Why: can branch after Lane A stabilizes participant types and query helpers.

Lane C: event schema + runtime publication  
Why: depends on participant types but is otherwise independent except for one shared touchpoint in `async_repl.rs`.

Lane D: tests + docs cleanup  
Why: goes last so it can validate the final API, not a moving target.

### Lane ownership rules

To keep worktrees from stepping on each other:

- Lane A owns `crates/shell/src/execution/agent_runtime/` and the participant type names. Nobody else edits those files until Lane A merges.
- Lane B owns `crates/shell/src/execution/agents_cmd.rs`, `docs/USAGE.md`, and `docs/TRACE.md` after Lane A merges.
- Lane C owns `crates/common/src/agent_events.rs`, `crates/common/tests/agent_hub_event_envelope_schema.rs`, and the event-emission touchpoints in `async_repl.rs` after Lane A merges.
- Lane D owns integration tests and doc cleanup after B and C merge.

If a lane needs to cross that boundary, stop and re-split the work. Parallelism is only worth it when ownership is obvious.

### Execution order

1. Launch Lane A first.
2. After Lane A lands the participant surface and helper signatures, launch Lane B and Lane C in parallel worktrees.
3. Merge Lane C before Lane B only if `async_repl.rs` stayed isolated to event publication; otherwise rebase whichever lane touched the shared block second.
4. Merge B and C only after both pass their lane-local tests.
5. Run Lane D last.

Lane-local merge gates:

- Lane A: participant constructor, compat upgrade, and store tests green
- Lane B: status/toolbox/doctor contract tests green
- Lane C: event envelope schema tests green
- Lane D: full targeted shell/common suite green and docs updated

### Conflict flags

- Lane A and Lane C both touch `crates/shell/src/repl/async_repl.rs`
- Lane A and Lane B both touch `crates/shell/src/execution/agent_runtime/state_store.rs`
- Lane B and Lane D both touch `docs/USAGE.md` and `docs/TRACE.md`
- Lane C and Lane D can both touch `crates/common/tests/agent_hub_event_envelope_schema.rs` if test cleanup starts too early

Safe rule: do not start B or C before A defines the participant type names and store helper signatures.

## Deferred Work

There is no `TODOS.md` in this repo root, so deferred items stay here explicitly.

1. Production member-writer hookup for shared-world members  
Why: requires the concrete lifecycle from slices `03` and `04`.

2. Restart invalidation semantics for member replacement  
Why: belongs to slice `05`, even though this slice adds the lineage fields.

3. Grouped session-centric runtime store layout  
Why: belongs to slice `06`; this slice keeps compatibility reads and participant-native queries only.

## NOT in Scope

- shared-world ownership lock-in
- world restart generation authority
- public `session_start` / `session_resume` / `session_fork` CLI UX
- nested gateway schema redesign
- toolbox feature expansion or mutating toolbox tools
- background registry service or cache

## Definition of Done

The slice is done when all of these are true:

1. There is one canonical child runtime record type that can represent orchestrator and member participants.
2. New runtime writes go to `~/.substrate/run/agent-hub/participants/*.json`.
3. Legacy `handles/*.json` files are still readable during rollout.
4. `substrate agent status --json` adds `participant_id` and no longer collapses live rows by `agent_id`.
5. Toolbox authorization resolves from a live host-scoped orchestrator participant only.
6. Doctor remains pre-runtime, but malformed participant files fail closed when present.
7. Pure-agent `AgentEvent` rows can publish participant lineage additively.
8. Docs and contract tests reflect participant terminology and sibling-safe status semantics.

## Completion Summary

- Step 0: Scope Challenge - scope accepted as-is, but explicitly bounded away from slices `03` through `06`
- Architecture Review: 4 material findings
- Code Quality Review: 3 implementation guardrails around invariant ownership, compat-boundary DRY, and terminology cutover
- Test Review: coverage diagrams produced, 33 concrete gaps/assertions identified
- Performance Review: 0 major issues, 3 direct-lookup/no-cache rules
- Error & Rescue Registry: written
- NOT in scope: written
- What already exists: written
- TODOS.md updates: deferred scope captured in-plan because no `TODOS.md` exists
- Failure modes: 3 critical gaps flagged until the participant-keyed cutover lands
- Outside voice: skipped, no separate codex-review run was performed
- Parallelization: 4 lanes, 2 bounded parallel after the foundation lane
- Lake Score: complete version chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | Scope | Make `participant_id` the canonical child identity | Mechanical | Completeness | Same-agent siblings are impossible otherwise | Keeping `agent_id` as the lookup key |
| 2 | Scope | Keep parent orchestration session as the top-level authority | Mechanical | Explicit over clever | Parent and child solve different identity layers | Replacing the parent record with participants |
| 3 | Storage | Add `participants/` and keep `handles/` as read-compat only | Mechanical | Pragmatic | Safe rollout without a flag day | In-place destructive migration |
| 4 | Model | Reuse current `ash_<uuid>` child id format for `participant_id` | Mechanical | Boring by default | Correctness matters more than new ID branding | New bespoke participant-id scheme |
| 5 | Ownership | Widen `ownership_mode` to include member and replaced semantics | Mechanical | Completeness | Member/runtime lineage needs explicit ownership state | Free-form attached-control strings forever |
| 6 | Status | Add `participant_id` to status JSON and stop collapsing by agent | Mechanical | Completeness | Operator truth must not be lossy | Newest-wins per agent heuristics |
| 7 | Toolbox | Keep toolbox authorization host-orchestrator-only | Mechanical | Systems over heroes | Member rows must never become control-plane authority | Authorization from any live participant |
| 8 | Doctor | Add passive participant validation only | Mechanical | Minimal diff | Doctor should stay pre-runtime while still catching bad files | Making healthy doctor depend on live participants |
| 9 | Events | Add participant lineage fields additively to `AgentEvent` now | Mechanical | Boil the lake | Avoid a second schema churn in slice `05` | Delaying lineage fields |
| 10 | Scope | Do not invent a generic production member-launch framework in this slice | Mechanical | Pragmatic | The branch has no concrete member seam yet | Shipping speculative lifecycle infrastructure |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/autoplan` | Scope & strategy | 1 | CLEAR | Correct second slice, explicit non-goals, no accidental absorption of world ownership or grouped-store work |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-voice run performed |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | Lossy status key fixed at the root, participant store migration bounded, fail-closed operator/test obligations made explicit |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 blocking design decisions remain inside this slice. The only intentional deferrals are later packet slices.

**VERDICT:** CEO + ENG CLEARED. `PLAN-02` is ready to implement after `PLAN-01`, and before the shared-world, world-binding, restart-generation, and grouped-store follow-ons.
