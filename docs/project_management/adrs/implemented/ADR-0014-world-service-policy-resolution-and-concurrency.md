# ADR-0014: World-Agent Policy Resolution and Concurrency

Status: Approved  
Owner: Substrate core team  
Date: 2026-01-18  

## Executive Summary (Operator)

ADR_BODY_SHA256: f593408c9cc872e61a9c5fb74272f077ada540949a67630eae622cb43c1f1d14

### Changes (operator-facing)
- Host-resolved policy snapshot becomes the sole policy input to world-service enforcement.
  - Existing: world-service resolves policy internally and relies on shared broker state, which can diverge from the invoking user’s effective policy and can contaminate concurrent requests.
  - New: world-service consumes an explicit host-provided policy snapshot per request/session and uses only that snapshot as the policy input to isolation enforcement.
  - Why: eliminates global policy home mismatch and cross-request policy contamination; makes enforcement deterministic and auditable.
  - Links:
    - `crates/shell/src/execution/policy_snapshot.rs`
    - `crates/world-service/src/service.rs`
    - `crates/world-service/src/pty.rs`

## Context

Substrate policy is designed to be resolved as a single “effective merged policy” per directory:

- Defaults (built-in)
- Global policy patch at `$SUBSTRATE_HOME/policy.yaml`
- Workspace policy patch at `<workspace_root>/.substrate/policy.yaml` when a workspace exists

The Substrate CLI and shim use `crates/broker` to resolve and apply policy decisions. The Linux world
backend also needs policy-derived inputs to enforce isolation (filesystem + network) inside a world
via the world-service.

Today, the world-service also links `crates/broker` and performs policy resolution internally.

## Problem

### 1) Policy home mismatch (global patch inconsistency)

The world-service is typically deployed as a systemd service (often running as `root`). Its process
environment may not have `SUBSTRATE_HOME` set to the same location as the invoking user’s Substrate
home. As a result:

- The CLI can show an effective policy that includes a user’s global patch.
- The world-service can resolve a different effective policy (often missing the global patch), causing
  isolation behavior to diverge (e.g., `world_fs.write_allowlist` not honored in `isolation=full`).

This violates the expectation that the “effective merged policy” is consumed consistently across
shell/shim/world-service.

### 2) Request concurrency bug (cross-request policy contamination)

The world-service runs a multithreaded Tokio runtime and services requests concurrently.
In the request path, the agent uses broker global state roughly as:

1. `detect_profile(&cwd)` (writes broker state)
2. `world_fs_policy()` / `allowed_domains()` (reads broker state)

Because the broker state is shared process-wide, concurrent requests can interleave between (1) and
(2), causing request A to read request B’s policy. This is a correctness and security foot-gun:
policy-derived enforcement (filesystem allowlists, network allowlists) can be applied to the wrong
request.

## Goals

- Ensure world-service enforcement uses the same effective policy semantics as the CLI.
- Eliminate cross-request policy contamination under concurrency.
- Clarify what “global policy” means in a world-service deployment:
  - single-user workstation,
  - multi-user host with shared world-service,
  - guest deployments (macOS Lima / Windows WSL) where host global paths may not exist.

## Non-goals

- Redesign of the entire policy language.
- Introduction of a new privileged broker daemon (unless chosen explicitly).

## Current Behavior (Ground Truth)

- The broker resolves effective policy by reading:
  - `$SUBSTRATE_HOME/policy.yaml` via `substrate_common::paths::policy_file()`
  - `<workspace_root>/.substrate/policy.yaml` discovered from `cwd`
- The world-service calls broker APIs per request and relies on a shared broker handle.
- Full isolation writability depends on `world_fs.write_allowlist` being resolved by the world-service
  and translated into mount-time remounts (via `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST`).

## Options Considered

### Option A: Serialize world-service policy resolution with a lock

Add a mutex around:
`detect_profile(cwd)` + reads of `world_fs_policy()/allowed_domains()`.

Pros:
- Minimal code change; no API changes.
- Eliminates the concurrency interleaving bug.

Cons:
- Still relies on process-global policy home (`SUBSTRATE_HOME`) and local filesystem visibility.
- Still ambiguous for multi-user deployments.
- World-agent continues to parse YAML and duplicate broker logic.

### Option B: Per-request broker instance (snapshot by construction)

Construct a new broker instance per request, resolve effective policy into a local `Policy`, and use
it for enforcement inputs.

Pros:
- Fixes concurrency without global locking.
- Keeps world-service self-contained.

Cons:
- Still relies on local filesystem + `SUBSTRATE_HOME` semantics.
- More CPU/io (parsing YAML per request) unless cached carefully.

### Option C: Host-resolved policy snapshot passed to world-service (preferred direction)

Move effective policy resolution to the host-side Substrate CLI (the “authoritative policy
resolver”), and pass a policy snapshot (or the subset needed for enforcement) to world-service as part
of the request.

Pros:
- Single source of truth for policy merging.
- Works in guest deployments (Lima/WSL) even when the host global patch is not visible in the guest.
- Removes world-service dependence on `$SUBSTRATE_HOME` semantics.
- Enables future multi-tenant designs (host can decide which policy applies to which caller).

Cons / Risks:
- Requires request schema changes and versioning.
- Requires careful threat modeling: a privileged world-service must not accept untrusted “policy”
  inputs from arbitrary clients without authentication/authorization.

## Proposed Direction

Adopt Option C in stages:

1. **Short-term correctness:** ensure the world-service deployment has an explicit, intended policy
   home (e.g., set `SUBSTRATE_HOME` in the systemd unit to the Substrate install prefix) to align
   global patch behavior on single-user workstations.
2. **Medium-term correctness:** change world-service to use per-request policy snapshots rather than
   process-global mutable broker state (either by passing policy from host, or by per-request
   resolution that returns a local `Policy` value).
3. **Security/multi-user:** define and enforce how the world-service determines the caller identity
   and which policy applies, and ensure policy inputs cannot be spoofed.

## Follow-up Decisions (Recorded Elsewhere)

Decisions required to execute this ADR are recorded in the feature Decision Register:
- `docs/project_management/_archived/world-service-policy-snapshot/decision_register.md`
