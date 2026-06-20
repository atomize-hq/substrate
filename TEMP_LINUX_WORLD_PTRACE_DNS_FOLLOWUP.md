# Temporary Follow-Up: Linux World Ptrace / DNS Investigation

Status: temporary planning handoff for a fresh session
Created: 2026-06-20
Repo: `/home/azureuser/__Active_Code/atomize-hq/substrate`
Branch during investigation: `feat/internal-host-orchestrator-world-dispatch-bootstrap`
Priority order: `P0` ptrace/threaded resolver bug, `P1` host-native DNS contract, `P1/P2` doctor and diagnostics

## Why this exists

Recent investigation started from a `codex-runtime` guest install failure during `substrate world deps current sync`.

The immediate installer symptom was:

- `curl: (28) Resolving timed out after ... milliseconds`

That led to two separate findings:

1. The direct cause of the observed `curl` failure appears to be a non-PTY Linux world exec bug around ptrace-based process capture and threaded workloads.
2. Linux host-native world DNS posture is under-specified and drifted from docs, even if it is not the primary cause of this specific `curl` failure.

This document is meant to let a fresh session resume planning without redoing the investigation.

## Important Scope Clarification

This was not diagnosed as a world netfilter/policy restriction failure.

During investigation, the effective posture showed:

- `world.net.filter=false`
- `net_allowed=[]`
- `substrate world doctor --json` reported `requested=false` and `enabled=false`

So the planning work here should not start from the assumption that outbound requests were intentionally restricted by world network policy.

## Executive Summary

### Most likely direct root cause

`curl` in the world is likely failing because its async threaded resolver path hangs under ptrace-based process capture in non-PTY Linux world exec.

Key evidence:

- In-world `curl -I https://...` fails in the resolving phase.
- In-world `curl --resolve host:443:IP ...` succeeds.
- In-world `getent hosts`, Python `socket.getaddrinfo()`, and `wget` succeed.
- The investigated Ubuntu `curl` build reports `AsynchDNS` and appears to use the threaded resolver path rather than `c-ares`.
- Subagent investigation found a trivial threaded Python workload hanging in-world while the same host-side workload succeeds.
- The hung in-world process showed a nonzero `TracerPid`.
- Ptrace options in [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:275) include `TRACEFORK|TRACEVFORK|TRACEEXEC|TRACEEXIT`, but not `TRACECLONE`.

### Important interpretation caveat

Early in the investigation, `127.0.0.53` / stub-resolver drift looked like a plausible primary cause.

That suspicion should now be treated as secondary context, not the leading explanation, because later evidence showed:

- in-world `/etc/resolv.conf` pointed at the systemd stub
- a real in-world resolver query through that stub succeeded
- main-thread resolution succeeded
- threaded resolver behavior failed

The fresh planning session should therefore prioritize the ptrace/thread hypothesis first, while still treating Linux host-native DNS contract cleanup as real follow-up work.

### Important secondary platform issue

Linux host-native world runtime does not appear to own a coherent DNS contract today. It mostly inherits ambient host resolver state, while docs still imply a stronger, more explicit Linux world networking contract.

That should still be fixed, but it should not distract from the ptrace/thread issue that appears to be causing the concrete `curl` failure.

## Land Item 1: Fix Non-PTY Linux World Ptrace / Threaded Workload Bug

Priority: `P0`

### Goal

Make threaded workloads behave correctly under host-native non-PTY Linux world exec.

Concrete user-facing expectation:

- `curl` DNS resolution should work in-world without relying on downloader-specific workarounds.
- Simple threaded programs should not hang under `substrate --world`.

### Current Evidence

- Process capture is constructed for non-PTY world exec in [crates/world/src/session.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:815).
- Ptrace-backed capture is enabled when available in [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:208) and [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:793).
- Ptrace options appear incomplete for threaded child handling in [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:275).
- Subagent conclusion: missing or incomplete traced-thread handling is the best current explanation for `curl` resolver hangs.
- Subagent conclusion: this specific `curl` failure is more likely a threaded workload / ptrace interaction than a dead DNS stub.

### Planning Questions

1. Is missing `PTRACE_O_TRACECLONE` the main bug, or only one symptom of broader traced-thread handling gaps?
2. Does the capture loop properly continue newly created traced threads?
3. Is the failure specific to non-PTY exec, or does PTY streaming have the same issue?
4. Should the product:
   - fix ptrace thread support directly, or
   - automatically fall back to a non-ptrace capture mode when threaded execution is detected or when ptrace thread support is incomplete?

### Likely Code Seams To Inspect

- [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs)
- [crates/world/src/session.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs)
- Any process telemetry / process event code paths used by non-PTY Linux exec

### Suggested Reproduction / Validation Cases

- In-world `curl -I https://github.com`
- In-world `curl --resolve github.com:443:<ip> https://github.com`
- In-world trivial Python thread start/join
- In-world trivial pthread-based native test if helpful
- Compare non-PTY and PTY execution behavior

### Acceptance Criteria

- Plain `curl -I https://github.com` succeeds in non-PTY world exec.
- A minimal threaded workload completes in non-PTY world exec.
- Process telemetry remains correct, or there is a clearly documented degradation/fallback.
- Regression coverage exists for threaded child behavior.

## Land Item 2: Define A Coherent Linux Host-Native DNS Contract

Priority: `P1`

### Goal

Stop relying on ambient host resolver presentation inside host-native Linux worlds. Substrate should own and expose a coherent resolver contract for the world runtime.

### Current Evidence

- There is dormant stub-resolver code in [crates/world/src/dns.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/dns.rs:107), but subagent investigation found it is not on the live host-native runtime path.
- Linux world setup is effectively a lightweight/no-op isolation path in [crates/world/src/session.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:660) and [crates/world/src/session.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:798).
- Live non-PTY world exec inherits host `/etc` state in [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:1015) and [crates/world/src/exec.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/exec.rs:1416).
- Netfilter DNS allow rules derive from `/etc/resolv.conf` in [crates/world/src/netfilter.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/world/src/netfilter.rs:93), but this is only rule construction, not runtime resolver provisioning.
- `world enable --provision-deps` DNS remediation in [crates/shell/src/builtins/world_enable/runner/provision_deps.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_enable/runner/provision_deps.rs:488) is guest-oriented and should not be treated as the host-native Linux strategy.
- Subagent correction: the Linux host-native DNS problem should not currently be framed as “the stub is always dead.” The better framing is “the runtime does not own a coherent resolver contract and should not rely on ambient stub presentation.”

### Recommended Product Posture

For Linux host-native worlds:

- derive concrete reachable upstream resolver IPs from authoritative host sources
- prefer non-loopback/non-stub resolver sources when `/etc/resolv.conf` only exposes loopback stubs
- materialize a world-local resolver view for execution
- use the same effective resolver set for DNS-related netfilter allow rules
- fail clearly when a usable upstream resolver set cannot be derived

### Important Non-Goals

- Do not mutate host-native Linux DNS configuration.
- Do not revive the dormant per-world dnsmasq stub path as the main design.
- Do not treat Lima guest DNS behavior as the Linux host-native model.

### Planning Questions

1. What is the exact source precedence for deriving effective upstream resolvers?
   - `/etc/resolv.conf`
   - `/run/systemd/resolve/resolv.conf`
   - `resolvectl`
   - other distro-specific authoritative sources if needed
2. Where should the world-local `resolv.conf` be projected from?
3. How should full isolation vs workspace isolation consume that resolver view?
4. How should doctor output represent:
   - configured resolver view
   - derived upstream resolver set
   - source of truth used
   - failure mode when only loopback/stub entries are visible

### Docs / Contract Drift To Reconcile

- [docs/WORLD.md](/home/azureuser/__Active_Code/atomize-hq/substrate/docs/WORLD.md) still implies a stronger Linux netns-first model than the current host-native runtime seems to implement.
- The fresh plan should include doc updates once runtime truth is settled.

### Acceptance Criteria

- Linux host-native world runtime has a documented, implemented resolver contract.
- World execution does not depend on ambient stub-only resolver exposure.
- Netfilter DNS allow rules and world runtime resolver view derive from the same effective upstream set.
- Docs match implementation.

### Specific caution for planning

Do not let this item accidentally turn into “fix curl.” The goal here is platform contract cleanup:

- what resolver view Linux host-native worlds should see
- how that view is derived
- how runtime execution and netfilter share the same truth
- how failure is surfaced when only unusable resolver inputs are available

## Land Item 3: Add Doctor / Diagnostics For Resolver And Threaded Execution Health

Priority: `P1/P2`

### Goal

Make this class of bug easy to diagnose without ad hoc shell probing.

### Why this matters

The original investigation required multiple custom commands to separate:

- general network egress
- DNS reachability
- resolver-stack differences
- threaded workload behavior
- ptrace/process-capture side effects

That is too much manual work for an operator-facing failure.

### Suggested Doctor Additions

1. Real in-world DNS probe
   - Read in-world `/etc/resolv.conf`
   - If it points to loopback/stub resolvers, run a real query against that target
   - Report whether the configured resolver is actually answering

2. Resolver differential probe
   - Main-thread `getaddrinfo()` test
   - Threaded `getaddrinfo()` test with short timeout
   - If main-thread works but threaded resolution hangs, classify that specifically

3. Threaded workload smoke test
   - Tiny `pthread_create` or Python thread start/join probe
   - Report failure as a process-capture/thread compatibility problem

4. Process-capture backend details
   - Which backend is active
   - Whether ptrace is in use
   - Whether clone/thread events are supported by the active backend

5. Optional curl differential probe
   - Plain `curl`
   - `curl --resolve ...`
   - If `--resolve` works and plain `curl` fails, classify this as a resolver-path-specific failure instead of generic egress failure

### Likely Surfaces

- `substrate world doctor --json`
- Possibly `substrate host doctor --json` if some inputs are host-derived
- `substrate --shim-status --json` only if that surface is already used for world health context

### Acceptance Criteria

- A fresh operator can distinguish:
  - DNS stub unavailable
  - usable main-thread DNS but broken threaded resolver path
  - generic network egress failure
  - ptrace/process-capture incompatibility
- The doctor output gives concrete remediation hints.

## Recommended Landing Order

1. Land Item 1 first.
   Reason: it appears to be the direct cause of the concrete `curl` failure.

2. Land Item 2 next.
   Reason: it fixes the broader Linux host-native DNS product contract and removes ambient resolver drift.

3. Land Item 3 alongside or immediately after 1/2.
   Reason: better diagnostics reduce future investigation cost and make rollout safer.

## Important Current Workaround Context

The current repo also contains a mitigation for the codex-runtime installer path:

- the codex runtime install script was changed to prefer `wget` / `python3` over `curl`
- the extraction path was adjusted to avoid tar metadata restore failures in the world staging dir
- the main files touched for that mitigation were:
  - [crates/shell/src/builtins/world_deps/inventory.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/inventory.rs)
  - [crates/shell/src/builtins/world_deps/surfaces.rs](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_deps/surfaces.rs)
- those mitigation changes were validated with:
  - focused `cargo test -p shell world_deps_codex_runtime -- --nocapture`
  - live `target/debug/substrate world deps current install codex-runtime`
  - live `target/debug/substrate --world -c 'codex --version'`

That mitigation unblocks runtime installation, but it should not be mistaken for the permanent product fix to the underlying runtime issues described above.

## Suggested First Steps For A Fresh Session

1. Re-read this document and validate whether the branch still contains the temporary codex-runtime mitigation in the files listed above.
2. Reproduce the ptrace/thread symptom on the current build before changing code.
3. Confirm whether the current process-capture path still lacks correct traced-thread handling.
4. Only after that, produce the execution-ready plan for all three landing items.

## Suggested Fresh-Session Opening Prompt

If a fresh session is picking this up, a good starting request is:

> Read `TEMP_LINUX_WORLD_PTRACE_DNS_FOLLOWUP.md`, validate the ptrace/thread hypothesis against the current non-PTY Linux world exec path, then produce an execution-ready plan for the three listed landing items with specific code seams, test strategy, and rollout order.
