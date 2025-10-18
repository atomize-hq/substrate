# PRE_PHASE_4.5 — PR-Sized Implementation Steps

Purpose: Break the PRE_PHASE_4.5 hardening plan into reviewable, mergeable PRs. Each PR includes scope, files, tests, acceptance, risk/rollback, estimate, and sequencing.

Reference docs
- PRE plan: docs/project_management/PRE_PHASE_4_5_HARDENING_PLAN.md
- Vision: docs/VISION.md
- Phase 4 completion: PHASE_4_COMPLETION_REPORT.md

Conventions
- Branch: `pre45/<short-topic>`
- PR title: `pre45: <concise change>`
- CI: ensure workspace builds; privileged tests may be skipped in public CI (document how to run locally).
- Feature gates: keep non-Linux behavior stable and well messaged.

---

## PR‑01: Shell telemetry fix and fs_diff preference
- Goal: Fix compile/runtime bug in `collect_world_telemetry` and prefer returned fs_diff.
- Scope:
  - Fix undefined `world_id` variable.
  - When `ExecResult.fs_diff` is present (from future PR‑03), use it and skip telemetry fallback.
- Files:
  - crates/shell/src/lib.rs (collect_world_telemetry, command completion path)
- Implementation:
  - Rename `_world_id` binding to `world_id`, pass to `WorldHandle { id: world_id.clone() }`.
  - Guard telemetry call behind `if fs_diff.is_none()`.
- Tests:
  - Build on Linux/macOS/Windows; simple run path for `--trace` and `--replay`.
- Acceptance:
  - Shell compiles; `--trace` pretty prints; no panics on Linux.
- Risk/Rollback: Minimal; revert single file.
- Estimate: 0.5 day
- Depends on: none

## PR‑02: world‑api ExecRequest/ExecResult extensions
- Goal: Add `span_id` and `fs_diff` to the API surface.
- Scope:
  - `ExecRequest { span_id: Option<String> }`
  - `ExecResult { fs_diff: Option<FsDiff> }`
  - world-agent: plumb through; shell/shim types updated (no behavior yet).
- Files:
  - crates/world-api/src/lib.rs
  - crates/world-agent/src/service.rs (pass through fields)
  - crates/shim, crates/shell (compile fixes only)
- Tests:
  - Unit build; JSON serde round-trip for new fields.
- Acceptance:
  - Workspace builds; new fields appear in debug logs where printed.
- Risk: Low; additive.
- Estimate: 0.5 day
- Depends on: PR‑01 (optional)

## PR‑03: FsDiff lifecycle and overlay improvements
- Goal: Return accurate diffs at execution time and allow later lookup by span.
- Scope:
  - `SessionWorld::execute` returns `ExecResult` with `fs_diff` when isolated.
  - Maintain `HashMap<span_id, FsDiff>` for `fs_diff(world, span_id)`.
  - `overlayfs::is_modification` uses presence in lower layer to classify mods vs writes; optional small‑file hashing (feature `diff-hash`).
- Files:
  - crates/world/src/session.rs
  - crates/world/src/overlayfs.rs
- Tests:
  - Unit: `is_modification` classification; diff contains writes/mods; truncation summary preserved.
  - Integration (optional privileged): run isolated command and assert non‑empty diff.
- Acceptance:
  - ExecResult carries fs_diff; subsequent `fs_diff(span_id)` returns same.
- Risk: Medium (lifecycle); ensure robust cleanup of overlay temp dirs.
- Rollback: Guard usage behind feature flag if needed.
- Estimate: 1 day
- Depends on: PR‑02

## PR‑04: Isolation refactor — user/mount namespaces, pivot_root, /proc, /dev
- Goal: Correct isolation ordering and safety.
- Scope:
  - Implement ordered unshare (userns → uid/gid maps → mountns), mark mounts private, bind mounts (system ro, project rw), pivot_root inside new ns, mount /proc, minimal /dev, set PR_SET_NO_NEW_PRIVS, baseline seccomp logging.
- Files:
  - crates/world/src/isolation.rs
- Tests:
  - Unit: path creation; helper sequencing returns Ok.
  - Integration (privileged): verify /proc present, ro binds enforced.
- Acceptance:
  - No host side-effects; runs with clear warnings if capabilities missing.
- Risk: High (kernel features); degrade gracefully when unavailable.
- Estimate: 1.5–2 days
- Depends on: none (parallelizable with PR‑03)

## PR‑05: Cgroups v2 attach and limits
- Goal: Enforce memory and CPU limits where supported.
- Scope:
  - Ensure cgroup v2 mount; create per‑world cgroup; write pid to `cgroup.procs`; set `memory.max` and `cpu.max`.
- Files:
  - crates/world/src/isolation.rs (cgroup helpers)
- Tests:
  - Privileged: assert files exist and contain expected values; non‑privileged prints warning and continues.
- Acceptance:
  - Limits take effect on supported systems; otherwise graceful warning.
- Risk: Medium; distro variance.
- Estimate: 1 day
- Depends on: PR‑04

## PR‑06: Netfilter — per‑world rules, LOG prefix, scopes
- Goal: Safe, scoped network filtering with telemetry.
- Scope:
  - Install nftables rules inside netns (preferred) or via cgroup/nfmark matches; create v4/v6 sets for allowed IPs; LOG prefix `substrate-drop-<world_id>`; idempotent removal.
  - `monitor_network_scopes()` parses logs + conntrack (optional) to produce scopes.
- Files:
  - crates/world/src/netfilter.rs
- Tests:
  - Unit: rule strings; domain resolution; get_scopes_used formatting.
  - Privileged: install/remove rules; verify LOG lines appear under blocked domain.
- Acceptance:
  - Block/allow behavior matches `allowed_domains`; scopes emitted.
- Risk: Medium; requires root and nftables.
- Estimate: 1–1.5 days
- Depends on: PR‑04 (netns)

## PR‑07: Broker → world allowed_domains wiring
- Goal: Pass policy `net_allowed` to worlds.
- Scope:
  - Add broker accessor for allowed domains; thread into `WorldSpec.allowed_domains` from shell and world-agent.
  - Update policy docs.
- Files:
  - crates/broker/src/lib.rs (new getter)
  - crates/shell/src/lib.rs (shell path)
  - crates/world-agent/src/service.rs (agent path)
- Tests:
  - Unit: broker getter returns expected domains.
- Acceptance:
  - New worlds use updated allowlist.
- Risk: Low
- Estimate: 0.5 day
- Depends on: PR‑06 (to observe effect), parallelizable

## PR‑08: Replay alignment with world‑api
- Goal: Use world isolation path and propagate fs_diff/scopes into replay results.
- Scope:
  - Build `ExecRequest` with `span_id`; set env for correlation; copy `fs_diff` and scopes into `ExecutionResult` and `ReplayResult`.
- Files:
  - crates/replay/src/replay.rs
- Tests:
  - Unit: direct execution still passes; when `SUBSTRATE_REPLAY_USE_WORLD=1` on Linux, replay returns fs_diff/scopes.
- Acceptance:
  - `substrate --replay <SPAN>` shows fs_diff/scopes when available.
- Risk: Low/Medium; gating by env avoids breaking non‑Linux.
- Estimate: 0.5 day
- Depends on: PR‑02, PR‑03

## PR‑09: Trace consolidation — remove kuzu from trace
- Goal: Make JSONL the sole write path; centralize graph integration.
- Scope:
  - Delete trace’s `#[cfg(feature="graph")]` kuzu integration; remove `kuzu` dep/feature from trace Cargo.toml.
- Files:
  - crates/trace/src/lib.rs
  - crates/trace/Cargo.toml
- Tests:
  - Workspace build; trace unit tests pass.
- Acceptance:
  - No Kuzu references in trace; only substrate‑graph will link Kuzu.
- Risk: Low
- Estimate: 0.5 day
- Depends on: none

## PR‑10: substrate‑graph facade (mock backend) + CLI skeleton
- Goal: Provide a single integration surface and CLI without Kuzu yet.
- Scope:
  - Define `GraphService` trait (connect, ensure_schema, ingest_span, what_changed, raw_query minimal).
  - Add mock backend (in‑memory HashMaps) to support CLI.
  - Add `substrate graph` subcommands: `ingest`, `status`, `what-changed`.
- Files:
  - crates/substrate-graph/src/lib.rs (+ new modules)
  - crates/shell/src/lib.rs (CLI dispatch)
- Tests:
  - Unit: mock backend; CLI handlers compile and print expected output on mock data.
- Acceptance:
  - `substrate graph status/ingest/what-changed` work against mock backend.
- Risk: Low/Medium (CLI touches shell parsing)
- Estimate: 1 day
- Depends on: PR‑09

## PR‑11: Docs update (user + dev)
- Goal: Bring docs in sync with behavior and provide runbooks.
- Scope:
  - Update PRE plan references, policy docs, isolation usage, graph CLI primer, replay updates.
- Files:
  - docs/ (update affected files; add HOWTOs referenced in PRE plan)
- Acceptance:
  - New docs exist; link from README where appropriate.
- Risk: Low
- Estimate: 0.5 day
- Depends on: PRs 01–10 as relevant

## PR‑12: CI scaffolding for privileged Linux + container instructions
- Goal: Provide a reference privileged job and local container guidance.
- Scope:
  - Add a reference GH Actions job (commented or separate workflow) for privileged tests; document local docker/podman commands.
- Files:
  - .github/workflows/<pre45>.yml
  - docs (append to PRE plan Appendix A)
- Acceptance:
  - CI builds workspace; privileged tests are either runnable in a controlled context or skipped by default.
- Risk: Low/Medium (infra)
- Estimate: 0.5 day
- Depends on: earlier PRs for specific tests

---

## Dependencies & Parallelization
- Critical path: PR‑04 → PR‑05 → PR‑06 (isolation → cgroups → netfilter)
- FsDiff path: PR‑02 → PR‑03 → (PR‑08 replay uses it)
- Shell fix (PR‑01), Trace consolidation (PR‑09) can go early.
- Graph facade (PR‑10) can proceed after PR‑09; it is mock‑backed and independent of Kuzu.
- Broker wiring (PR‑07) can happen in parallel and verified after PR‑06.

## Definition of Done (per PR)
- Builds on stable toolchain; unit tests green.
- Non‑Linux builds continue to work; world/isolation gracefully degrades.
- User‑visible behavior matches updated docs (where applicable).
- Rollback is a single revert (avoid broad refactors in one PR).

## Rollback & Forward Strategy
- Each PR is self‑contained and can be reverted if regressions occur.
- Where kernel features are involved, keep behavior behind capability checks and feature flags to allow forward progress without blocking the rest of the plan.

## Estimation Summary
- Total: ~7–9 dev days (parallelizable across 2–3 engineers) excluding review time.

