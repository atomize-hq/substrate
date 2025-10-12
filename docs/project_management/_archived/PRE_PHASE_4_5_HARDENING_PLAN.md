# PRE_PHASE_4.5 Hardening & Cleanup Plan

Purpose: stabilize core isolation, telemetry, and trace plumbing before Phase 4.5 so advanced features (concurrent agent output and graph intelligence) sit on a reliable, portable base.

Status: Draft for approval
Owner: Substrate Core
Timebox: 5–7 working days

## Objectives
- Correctness: world isolation, fs-diff lifecycle, and network filtering behave predictably on Linux; graceful fallbacks elsewhere.
- Observability: consistent spans and diffs available at command completion for ingestion and replay.
- Consolidation: one graph integration surface (substrate-graph); trace remains JSONL source of truth.
- Developer ergonomics: clear CLI behavior, helpful errors, realistic tests and CI jobs.

## Success Criteria (Exit)
- World isolation sequence uses proper namespaces/cgroups; basic limits enforced where supported; no host side-effects.
- FsDiff delivered reliably at command completion and/or queryable for the same execution; writes vs mods distinguished where feasible.
- Netfilter is scoped to the world (netns/cgroup match); drop/allow logs observable for scope tracking.
- Trace JSONL is the only write path; kuzu integration resides in substrate-graph; ingestion runs against the same spans used by shell/shim/replay.
- CLI --trace/--replay flows stable; helpful messages on unsupported platforms.
- Tests: green unit set + targeted integration tests for overlayfs, fs_diff, and netfilter (root/container‑gated).

## Scope (Workstreams)

### 1) Isolation & Namespaces (Linux)
- Add unshare sequence and ordering
  - CLONE_NEWUSER → write uid/gid maps → CLONE_NEWNS (+ UTS/IPC/PID/NET as needed).
  - Mark mounts private after CLONE_NEWNS, then bind mounts + remount ro, pivot_root inside the new namespace.
- Minimal filesystem
  - Mount clean /proc, create handful of /dev nodes (null, zero, random, urandom, tty). Keep read‑only bind mounts for system dirs.
- Cgroups v2
  - Ensure cgroup2 mounted; create per‑world subtree; write pid(s) to cgroup.procs; set memory.max and cpu.max from spec.
- Fallbacks
  - Detect missing capabilities (no root/no userns) and degrade to “observe‑only” without isolation; emit clear warnings.

Deliverables
- world/src/isolation.rs: refactor to implement the sequence above and attach to cgroup subtree; robust error messages.
- Tests: basic unit checks for path creation; containerized integration test (skipped without CAP_SYS_ADMIN).

### 2) Network Filtering (Linux)
- Scope rules per world
  - Prefer: create per‑world network namespace and install nftables rules inside it.
  - Alternative: use cgroup mark or nfmark to match only world processes if netns is unavailable.
- Logging & telemetry
  - Add LOG rules with prefix substrate-drop-<world_id> to correlate drops.
  - Limit via rate‑limit to avoid log storms. Keep conntrack parsing optional.
- Domain → IP resolution
  - Retain cached resolution per allowed_domains; support IPv6.

Deliverables
- world/src/netfilter.rs: install in netns or with cgroup/nfmark matches; add LOG prefixes; robust remove_rules.
- Tests: unit tests for rule strings; runtime tests gated on root.

### 3) FsDiff Lifecycle
- Preserve diff at execution time
  - Return FsDiff from exec for isolated runs; include in ExecResult so shell/shim can write it into the completion span immediately.
  - Optional: keep a per‑span overlay handle map to support post‑hoc fs_diff queries.
- Improve modification detection
  - Mark writes vs mods by checking presence in the lower layer.
  - Add size deltas; optionally hash small files (<N KB) under a privacy flag.

Deliverables
- world/src/session.rs: exec returns ExecResult { exit, stdout, stderr, scopes_used }, plus fs_diff (inline or companion call).
- world/src/overlayfs.rs: add is_modification() check; optional small‑file hash.

### 4) Shell & CLI Polish
- Bug fix: collect_world_telemetry uses undefined world_id (rename binding and wire correctly).
- Clear messages for non‑Linux when world features are invoked; suggest Docker/Lima fallback.
- Ensure SUBSTRATE_WORLD=enabled initializes trace once and sets SUBSTRATE_WORLD_ID consistently.

Deliverables
- crates/shell/src/lib.rs: fix collect_world_telemetry; refine init and errors; keep PTY behavior unchanged.

### 5) Trace Consolidation
- Single source of truth
  - Keep JSONL writing in substrate-trace only; remove kuzu integration from trace.
  - Add append_to_trace helper usage consistency across components.

Deliverables
- crates/trace/src/lib.rs: remove #[cfg(feature="graph")] kuzu_integration module; tighten span writing.
- CLIs unaffected; substrate-graph becomes the ingestion/query surface.

### 6) Replay Alignment
- Align replay world execution with world-api types
  - Use world-api::WorldSpec and ExecRequest; respect SUBSTRATE_REPLAY_USE_WORLD and platform gating.
- Carry fs_diff and scopes_used back to ReplayResult.

Deliverables
- crates/replay/src/replay.rs: update execute_in_world, execute_direct; ensure span correlation env vars set.

### 7) Broker & Policy Wiring
- Persist approvals across sessions (already present); add pattern simplification and expiry.
- Feed allowed domains to world-agent / world backend creation.
- Document policy schema with examples; add “observe‑only” vs “enforce” description.

Deliverables
- crates/broker: ensure get net_allowed for world creation callers; doc updates.

### 8) Graph Service “Flexibility Foundations”
- One integration point
  - substrate-graph provides GraphService: connect, ensure_schema, ingest(span), ingest_batch, query_*(), raw_query().
  - Compile‑time kuzu backend default; mock backend for tests.
- Ingestion
  - Live tail or batch from trace.jsonl; privacy filters (path ignore patterns, cmd arg redaction already in common::redact_sensitive).
- Query facade
  - High‑level helpers: what_changed(span), security_alerts(session?), agent_patterns(agent_id), with paging.
- CLI
  - substrate graph ingest/status/what‑changed… wired to service.

Deliverables
- crates/substrate-graph: implement GraphService trait + Kuzu backend wrapper skeleton; CLI glue lives in shell or a small bin.

### 9) Tests & CI
- Unit tests remain fast; integration tests for overlayfs/netfilter gated by root/container.
- GitHub Actions (or local CI): Linux job with CAP_SYS_ADMIN; macOS job builds but skips world tests; Windows job builds core crates.

### 10) Documentation & Examples
- User docs: world usage, policy examples, tracing locations.
- Dev docs: how to run privileged tests locally (docker run --privileged or podman), how to switch SUBSTRATE_WORLD modes.

## Timeline (suggested 5–7 days)
- Day 1–2: Isolation refactor + cgroups attach + basic tests.
- Day 2–3: Netfilter scoping + drop logging + conntrack optional.
- Day 3–4: FsDiff lifecycle + shell bug fix + replay alignment.
- Day 4–5: Trace consolidation → substrate-graph surface; CLI polish; docs.
- Buffer: 1–2 days for integration testing and CI stabilization.

## Risks & Mitigations
- Privileged APIs vary across distros → detect capabilities and degrade gracefully; document.
- nftables/conntrack not present → feature‑gate; bypass with clear warning; keep “observe‑only”.
- Overlayfs semantics → robust cleanup on drop; guard unmount errors; ensure no host pollution.
- CI permissions → run privileged tests in containers with CAP_SYS_ADMIN; skip otherwise.

## Engineering Tasks (by file)
- world/src/isolation.rs: add unshare sequence; mount private after CLONE_NEWNS; pivot_root inside namespace; mount cgroup v2; attach pids; set cpu.max/memory.max; improve errors.
- world/src/netfilter.rs: install in netns or match by cgroup/nfmark; add LOG rules with prefix substrate-drop-<world_id>; strict remove_rules; IPv6; optional conntrack path.
- world/src/overlayfs.rs: implement is_modification() (check lower existence); optional small‑file hashing; ensure cleanup.
- world/src/session.rs: return FsDiff from execute; maintain per‑span overlay map if post‑hoc queries needed; pass allowed_domains into filter.
- crates/shell/src/lib.rs: fix collect_world_telemetry binding; improve non‑Linux messages; ensure trace init respects SUBSTRATE_WORLD.
- crates/trace/src/lib.rs: remove kuzu_integration; keep JSONL only; ensure policy_violation/budget_exceeded helper spans consistent.
- crates/replay/src/replay.rs: align WorldSpec/ExecRequest usage; ensure fs_diff/scopes_used are captured.
- crates/broker: expose net_allowed for world creation; tighten approval persistence (pattern simplification keeps existing behavior).
- crates/substrate-graph: implement service trait and CLI glue; move all kuzu logic here.

## Quick Wins & Graph High‑Value Ideas (to inform substrate‑graph)
- Policy Assist: suggest cmd_allowed/net_allowed from clean histories with confidence and diff preview.
- Impact Map: given a file, show agents/commands that wrote/read it across time/projects; cluster by repo.
- Provenance Trails: end‑to‑end subgraphs from network → exec → file writes to reveal risky chains (e.g., curl → bash).
- Determinism Score: rank commands/repos by volatility (exit variance, stdout diffs, fs_diff churn) to target replay.
- Supply‑Chain Lens: aggregate package manager commands (npm/pip/cargo) with versions/domains; flag risky upgrades.
- Agent Coaching: per‑agent slow/failing commands; recommend pre‑approvals/domains to reduce friction.
- Policy Simulator: “what‑if” evaluate prospective policy changes against historical graph.

## Graph Service Design Principles (flexibility)
- Single public trait GraphService usable from any crate; runtime created via factory; compile‑time backends via features.
- Ingestion API accepts Span (from substrate-trace) and FsDiff; privacy filter opt‑in; paging for all queries.
- Strict separation: trace writes JSONL; graph reads/derives views.
- CLI and programmatic access share the same service facade.

## Concurrent Output: Path A vs Path B (with multi‑agent “Docking” vision)

Context: Future supports registering multiple agents (Claude, Codex, Gemini, Cline, etc.), running concurrently across same/different projects, with a panel showing active/dormant state and a live feed.

Path A (Recommended for 4.5 / PRE_PHASE_4.5)
- Keep Reedline for user input; add a non‑polling renderer thread that blocks on agent/event channels; use reedline::suspend_guard to draw messages, then restore prompt.
- Pros: minimal churn; zero idle CPU; easy to integrate; compatible with today’s shell; works cross‑platform.
- Cons: still bridges sync editor with async events; complex prompt redraws under heavy concurrency.

Path B (Future Phase, alongside “Agent Docking”)
- Full async TUI/terminal loop (tokio + crossterm/tui), or GUI front‑end, with a central event bus; the shell becomes a client to the “Agent Hub.”
- Pros: native async; scalable multiplexed views (per agent, per project); clean separation of input, rendering, and streaming.
- Cons: larger refactor; replaces Reedline; more surface to maintain.

Recommendation
- Adopt Path A now to unblock PRE_PHASE_4.5 and 4.5. In parallel, introduce an Agent Hub core (within host‑proxy or a new crate) exposing:
  - Registry: register/unregister agents (id, name, capabilities, project context).
  - Event bus: agent_registered, status, task_started, task_progress, pty_data, task_finished, alerts.
  - Subscriptions: per‑project and global feeds over WebSocket/Unix socket.
- Later (Phase 5): migrate the shell UI to consume the Hub’s feed with a richer async TUI (Path B) without breaking the backend.

Path A Implementation Notes (for agents)
- Shell: add a background thread with `rx.blocking_recv()` to avoid polling; call a small prompt‑safe printer guarded by Reedline’s `suspend_guard`.
- Create `AGENT_EVENT_TX: OnceCell<UnboundedSender<AgentEvent>>` accessible from host‑proxy/agent-hub client; push events received over the hub socket.
- CLI: add `substrate agents list` to display registry status from Agent Hub.
- Acceptance: idle CPU ~0.0% when agents idle; prompt not corrupted under bursty PTY output; works on Linux/macOS/Windows.

## Deliverables Summary
- Isolation/netfilter/fs_diff correctness and tests.
- Trace consolidation; replay alignment.
- substrate-graph service surface (facade + mock backend), initial CLI commands (status/ingest/what‑changed).
- Docs for users and contributors.

## Acceptance Tests (sample)
- Isolation: run simple commands inside world; verify /proc is mounted, read‑only binds enforced; limits reflected in cgroup files.
- Netfilter: when allowed_domains empty, connection to github.com is dropped and logged with correct prefix; when allowed, connection succeeds.
- FsDiff: npm install isolated run produces non‑empty writes/mods; summaries present when truncated.
- CLI: substrate --trace <SPAN> prints span; --replay <SPAN> executes and shows fs_diff and scopes.
- Graph: substrate graph ingest ~/.substrate/trace.jsonl then what‑changed <SPAN> returns file list.

## Backward Compatibility
- All new features are opt‑in via SUBSTRATE_WORLD and CLI flags; non‑Linux continues without world isolation; trace JSONL format remains stable.

---

Document version: 2025‑09‑04 (draft)

---

# Appendix A: Detailed Implementation Guide (Agent‑Executable)

This appendix turns each workstream into step‑by‑step instructions with code sketches, acceptance criteria, and local test commands.

## A1) Isolation & Namespaces (Linux)

Steps
1. Add helpers in `crates/world/src/isolation.rs`:
   - `unshare_userns`, `write_uid_gid_maps`, `unshare_mountns`, `make_mounts_private`, `setup_bind_mounts`, `pivot_root`, `mount_proc`, `create_minimal_devices`, `ensure_cgroup2_mounted`, `create_cgroup`, `attach_self_to_cgroup`, `set_cgroup_limits`.
2. In `LinuxIsolation::apply`, call the helpers in this exact order (fail‑fast with context on errors):
   - userns → uid/gid maps → mountns → mounts private → bind mounts (project rw; system ro) → pivot_root → mount /proc → minimal /dev → no_new_privs → seccomp → cgroup2 mount/attach/limits.
3. Add platform gating: on non‑Linux return a clear error; in Linux without capabilities, log warning and skip isolation.

Code sketch
```rust
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use nix::unistd::{getuid, getgid, pivot_root};

fn unshare_userns() -> anyhow::Result<()> {
    unshare(CloneFlags::CLONE_NEWUSER)?;
    std::fs::write("/proc/self/setgroups", "deny").ok();
    let uid = getuid(); let gid = getgid();
    std::fs::write("/proc/self/uid_map", format!("0 {} 1", uid))?;
    std::fs::write("/proc/self/gid_map", format!("0 {} 1", gid))?;
    Ok(())
}
```

Acceptance
- `/proc` mounted in world, system dirs ro, project rw; cgroup files set; no changes leak to host.

Local test
- `sudo -E cargo test -p world -- --nocapture`

## A2) Netfilter (Linux)

Steps
1. In `world/src/netfilter.rs`, ensure `resolve_domains()` fills IPv4 and IPv6 sets (use `dns_lookup`).
2. Implement `install_rules_linux()` to:
   - add table/chain with `hook output policy drop`
   - allow loopback, established/related, DNS
   - create `allowed4/allowed6` sets; add resolved IPs
   - allow daddr in sets; add `limit … log prefix "substrate-drop-<WORLD_ID> "`; then `counter drop`
3. Implement `remove_rules()` idempotently.
4. Parser: `parse_dropped_packets()` must search for exact prefix and extract dst IP; `parse_conntrack()` optional.

Acceptance
- Blocked connections are logged with the prefix; allowed ones succeed; `monitor_network_scopes()` returns scopes.

Local test
- `sudo -E RUST_LOG=info cargo test -p world -- --nocapture test_nftables_rules`

## A3) FsDiff Lifecycle

API changes
- world‑api/src/lib.rs:
  - `ExecRequest { cmd, cwd, env, pty, span_id: Option<String> }`
  - `ExecResult { exit, stdout, stderr, scopes_used, fs_diff: Option<FsDiff> }`

Steps
1. Update world‑agent service to accept/forward `span_id` and return `fs_diff`.
2. In `world/src/session.rs`, when isolating:
   - call `overlayfs::execute_with_overlay(...)`
   - store `FsDiff` in `self.fs_by_span: HashMap<String, FsDiff>` and return it in `ExecResult`.
3. Implement `compute_fs_diff(span_id)` to look up from map.
4. In `overlayfs.rs`, implement `is_modification(rel)` = lower layer path exists.

Acceptance
- `ExecResult.fs_diff` populated for isolated commands; later `fs_diff(world, span_id)` returns identical data.

## A4) Shell & CLI

Steps
1. Fix variable bug in `crates/shell/src/lib.rs::collect_world_telemetry` (use `world_id` binding for handle).
2. When a world exec path returns `ExecResult.fs_diff`, prefer it over telemetry fetch.
3. On non‑Linux, log a single user‑friendly info about isolation not being available.

Acceptance
- Shell builds; `--trace/--replay` show diffs on Linux; helpful messages elsewhere.

## A5) Trace Consolidation

Steps
1. Remove `#[cfg(feature = "graph")]` kuzu module from `crates/trace/src/lib.rs`.
2. Remove `kuzu` dep and `graph` feature from `crates/trace/Cargo.toml`.
3. Ensure only JSONL writing remains.

Acceptance
- Workspace compiles; kuzu appears only in substrate‑graph.

## A6) Replay Alignment

Steps
1. Build `ExecRequest` with `span_id: Some(state.span_id.clone())` in `execute_in_world`.
2. Copy returned `fs_diff` and `scopes_used` into `ExecutionResult` → `ReplayResult`.

Acceptance
- `substrate --replay <SPAN>` prints fs_diff and scopes when world is enabled.

## A7) Broker & Policy Wiring

Steps
1. Add function in broker to expose `net_allowed` (or entire `Policy`).
2. Pass domains into `WorldSpec.allowed_domains` (both shell and world‑agent paths).

Acceptance
- Changing policy affects allowlist for new worlds.

## A8) Graph Service Foundations

Trait
```rust
#[derive(Clone, Debug)]
pub struct GraphConfig { pub backend: String, pub db_path: std::path::PathBuf }
#[async_trait::async_trait]
pub trait GraphService: Send + Sync {
  async fn connect(cfg: GraphConfig) -> Result<Self, GraphError> where Self: Sized;
  async fn ensure_schema(&self) -> Result<(), GraphError>;
  async fn ingest_span(&self, span: &substrate_trace::Span) -> Result<(), GraphError>;
  async fn what_changed(&self, span_id: &str, limit: usize) -> Result<Vec<FileChange>, GraphError>;
}
```

CLI
- `substrate graph ingest <file>`
- `substrate graph what-changed <span_id> [--limit N]`

Acceptance
- `what-changed` matches FsDiff content for the span.

## A9) Tests & CI

Add a reference GH Actions job (privileged Linux) and instructions to run privileged tests in a container or with `sudo`. See main plan for YAML.

---

# Appendix B: API Change Log (PRE_PHASE_4.5)

- world‑api
  - ExecRequest: add `span_id: Option<String>`
  - ExecResult: add `fs_diff: Option<FsDiff>`
- world
  - SessionWorld::execute returns ExecResult with fs_diff; `fs_diff(world, span_id)` looks up stored diff.
- shell
  - Fix `collect_world_telemetry` world_id; consume ExecResult.fs_diff when present.
- trace
  - Remove kuzu integration/feature; JSONL only.
- substrate‑graph
  - Provide GraphService facade (+ mock backend) and basic CLI endpoints.

---

# Appendix C: Implementation Checklist (Copy‑Paste for Tracking)

- [ ] Isolation: userns/mountns sequence; pivot_root; /proc; /dev; cgroup2 mount/attach; cpu/mem limits.
- [ ] Netfilter: per‑world table/chain; sets; allow rules; LOG prefix; drop; removal; optional conntrack.
- [ ] FsDiff: ExecRequest.span_id; ExecResult.fs_diff; per‑span map; overlay mods vs writes.
- [ ] Shell: fix world_id; prefer ExecResult.fs_diff; friendly non‑Linux messages.
- [ ] Trace: remove kuzu integration and feature; JSONL only.
- [ ] Replay: pass span_id; flow fs_diff/scopes_used back.
- [ ] Broker: expose net_allowed; pass through to WorldSpec.
- [ ] Graph: GraphService skeleton + ingest + what‑changed CLI.
- [ ] Docs/CI: update docs; add privileged Linux job example; container instructions.
