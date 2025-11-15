# Substrate Isolated Shell – Execution Plan

This plan translates the architecture + dependency graph into concrete phases, concurrent workstreams, and agent responsibilities. Each task specifies the artifacts required from code agents and from test agents, plus integration checkpoints.

## Roles & Workflow

For every major task:
1. **Implementation Agent (Code)** – writes production code only.
2. **Verification Agent (Tests)** – writes/updates unit + integration tests referencing the same acceptance criteria.
3. **Integration Agent** – merges the two worktrees, resolves conflicts, runs the documented test commands, and ensures expectations are met.

Each task description below includes:
- Expected behaviors/contracts for both code/test agents.
- Required files (cross-reference the file audit).
- Test commands the integration agent must run.

## Phase Breakdown

### Phase A – Manifest & Manager Init Infrastructure

**Workstream A1: Manifest Parser (crates/common)**
- *Code Agent*: implement `manager_manifest.rs`, add serde schemas, loading API.
- *Test Agent*: create unit tests covering parsing, overlay merge, validation failures.
- *Integration Agent*: run `cargo test -p substrate-common manager_manifest`.
- **Dependencies**: none (can start immediately).

**Workstream A2: Manager Init Module (crates/shell)**
- Begins once parser merged.
- *Code Agent*: add `manager_init.rs`, config structs, snippet writer, CLI env var plumbing; no CLI commands yet.
- *Test Agent*: add unit tests mocking manifest results; verify skip flags.
- *Integration Agent*: run `cargo test -p substrate-shell manager_init`.

**Workstream A3: Shell Env Injection**
- Requires A2.
- *Code Agent*: update `lib.rs`, `async_repl.rs`, `pty_exec.rs` to inject PATH/snippets per session; add `--no-world` handling; generate `manager_env.sh`.
- *Test Agent*: integration tests verifying host PATH untouched and shell injection only occurs in Substrate. Use existing shell tests or add new ones.
- *Integration Agent*: run `cargo test -p substrate-shell --features shell-tests` (exact command defined once tests exist).

### Phase B – Shim Enhancements & Doctor CLI

**Workstream B1: Shim Hinting (crates/shim)**
- Requires Phase A completion (manifest + env var contracts).
- *Code Agent*: update `exec.rs`, add hint detection, `no_world` bypass.
- *Test Agent*: extend `tests/integration.rs` for hint logging + pass-through.
- *Integration Agent*: run `cargo test -p substrate-shim`.

**Workstream B2: Shim Doctor CLI (crates/shell)**
- Requires B1 (needs hint data).
- *Code Agent*: add CLI parser branches, report struct, repair logic.
- *Test Agent*: CLI integration tests (snapshots or textual asserts).
- *Integration Agent*: run targeted `cargo test -p substrate-shell --test shim_doctor`.

**Workstream B3: Docs & Config Updates**
- Can run parallel with B1/B2 but must merge after functionality ready.
- *Code Agent*: update docs per audit; add new env var descriptions.
- *Test Agent*: not applicable (docs) – instead run spellcheck/format if applicable.

### Phase C – World CLI & Installer Changes

**Workstream C1: World Enable Command**
- Depends on Phase A (env injection) but independent of B.
- *Code Agent*: implement `substrate world enable` command, call provisioning scripts, update config file.
- *Test Agent*: integration tests mocking provisioning call; verify metadata toggled.
- *Integration Agent*: run CLI tests + targeted script dry-run (CI friendly).

**Workstream C2: World Deps CLI**
- Requires C1 (world CLI plumbing) and manifest (guest recipes).
- *Code Agent*: implement `world deps status/install/sync`.
- *Test Agent*: add integration tests using mocked world agent (feature flag).
- *Integration Agent*: run `cargo test -p substrate-shell --test world_deps`.

**Workstream C3: Installer Upgrades**
- Depends on C1 (command exists).
- *Code Agent*: modify install/uninstall scripts (`--no-world`, metadata, snippet creation).
- *Test Agent*: add shell-script tests or CI jobs (if feasible) verifying host PATH untouched.
- *Integration Agent*: run script dry-runs inside container (document command).

### Phase D – Expansion & Health Commands

**Workstream D1: Additional Managers**
- After Phase B stable.
- *Code Agent*: extend manifest entries, add detection logic where necessary.
- *Test Agent*: new unit tests for each added manager.

**Workstream D2: Health/Doctor Enhancements**
- Build on B2.
- *Code Agent*: extend doctor CLI for world deps summary, aggregated health output.
- *Test Agent*: CLI snapshot tests.

## Concurrent Execution Guidance

- A1 and A2 can proceed concurrently once the manifest API is stubbed; define a trait or feature flag to unblock early work.
- B1 (shim) should start only after env injection merges to avoid churn.
- C1/C2 can begin while B-phase work runs; they touch different areas (world CLI vs shim).
- Installer changes (C3) should be last to avoid disrupting dev loops until features are ready.

## Documentation & Handoff

- Each task’s code agent must update the relevant section in `docs/project_management/next/substrate_isolated_shell_plan.md` to mark completion.
- Test agents must annotate their MR/PR with explicit commands run so integration agent can replicate.
- Integration agent owns the final verification: merges code+test branches, runs agreed commands, checks docs, and files bugs for regressions.

## Sign-off Criteria

- Pass-through behavior verified: host PATH unchanged after install; `substrate -c 'echo hi'` uses shims internally only.
- Manager auto-init + hints validated via automated tests.
- Shim doctor + `world enable` + `world deps` commands exercised in CI.
- Installer supports `--no-world` and upgrade path.
- Docs and config reference all new behaviors/env vars.

Follow this plan to keep the implementation parallelizable, verifiable, and transparent.***
