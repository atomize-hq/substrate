# Plan: Agent Drift Sentinel Real Session Live v0.5

## Scope

This plan implements
[agent-drift-sentinel-real-session-live-v0.5-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md:1).

The goal is to turn the bounded sentinel-local live seam into honest real-session live support for
one active Codex session.

This slice should:

- discover one active session under a real `CODEX_HOME`
- monitor the session’s append-only rollout JSONL
- rerun the compactor/analyzer pipeline for that session as new data arrives
- feed newly emitted checkpoints into the existing live runtime
- enable a bounded `--mode live` CLI path and prove it on an actual active session

This slice should not:

- integrate with `shell`, `world`, `shim`, or broader Substrate runtime paths
- reimplement compactor/analyzer logic inside the sentinel
- broaden to multi-session orchestration

## Why This Slice Comes Next

The current repo is at a clean gate:

- fixture-backed live mode is already proven and intentionally held at `L8`
- the compactor already discovers real session files under `~/.codex/sessions`
- the compactor can already scope work by `session_id`
- the analyzer already produces the checkpoint stream sentinel needs
- the user explicitly wants proof over an actual active session, not just fixture streams

That makes real-session coordination the next honest gap.

## Implementation Strategy

Build the real-session path from upstream artifact truth to sentinel runtime:

1. define a live-session request and coordinator seam
2. discover and poll one target rollout file under a real `CODEX_HOME`
3. invoke compactor and analyzer libraries for the target session into a bounded working directory
4. detect newly emitted checkpoints and forward them to the existing live runtime
5. enable the bounded live CLI path
6. prove it against an actually active session while the implementation agent is generating live
   session traffic

## Major Components

### 1. Live Session Discovery And Polling

Deliver first:

- session-scoped discovery using existing compactor discovery rules
- append-only polling over the target rollout artifact
- deterministic change detection so the pipeline only reruns when the session file grows

Why first:

- without a stable real-session source, the rest of the live path is synthetic again

### 2. Library-Owned Pipeline Coordination

Deliver second:

- call compactor and analyzer library APIs directly for one target session
- store intermediate outputs in a bounded state directory
- keep reruns deterministic and session-scoped

Why second:

- it preserves the current module boundaries and avoids subprocess orchestration drift

### 3. Checkpoint Delta Delivery

Deliver third:

- compare the current analyzer output to the last delivered live cursor/checkpoint id
- emit only new checkpoints into the existing live runtime

Why third:

- real-session polling is only useful if the sentinel behaves incrementally instead of replaying
  the full history on every loop

### 4. CLI Enablement

Deliver fourth:

- enable the bounded `--mode live` path
- require `--session-id`
- allow `--codex-home` and a state/output directory

Why fourth:

- the binary should stay thin until the library path is proven

### 5. Real Active-Session Proof

Deliver fifth:

- run the live sentinel against an actually active session while the agent is working
- capture the exact `SESSION_ID`, command, and observed output behavior

Why fifth:

- this is the user’s requested proof wall, and archived/static playback is not enough

## Risks And Mitigations

### Risk 1: Sentinel starts parsing raw rollout semantics directly

Mitigation:

- keep raw rollout parsing in the compactor crate
- keep checkpoint generation in the analyzer crate
- pass only analyzer checkpoints into the live runtime

### Risk 2: Polling causes duplicate warning floods

Mitigation:

- track last delivered cursor or checkpoint id
- emit only newly observed checkpoints into the existing runtime

### Risk 3: Real-session proof quietly uses archived artifacts

Mitigation:

- require a documented active `SESSION_ID`
- require the live command to run while the implementation agent is producing real session events

### Risk 4: Runtime wiring scope expands into Substrate execution layers

Mitigation:

- limit the source of truth to Codex session artifacts under `CODEX_HOME`
- keep all touched runtime code inside the sentinel crate and its direct library dependencies

## Verification Wall

Minimum automated wall:

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Required live proof:

1. identify an actually active `SESSION_ID`
2. run the bounded `--mode live` sentinel command against that session
3. keep the source session active while the sentinel runs
4. record the exact command and observed checkpoint/warning progression
