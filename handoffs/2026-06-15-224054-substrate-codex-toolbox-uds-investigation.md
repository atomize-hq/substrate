# Handoff: Substrate Codex Toolbox UDS Investigation

## Session Metadata
- Created: 2026-06-15 22:40:54
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: ~1 hour

### Recent Commits (for context)
  - 69248459 fix: satisfy clippy in linux socket doctor
  - c419b479 fix: bridge stale Linux socket groups with ACLs
  - 8deb75a4 fix: stabilize path tests with substrate home override
  - 2400b40e fix: restore linux world spec builders
  - 928ef583 Feat/macos hardening (#78)

## Handoff Chain

- **Continues from**: [2026-06-15-194703-gateway-smoke-permission-denied-investigation.md](./2026-06-15-194703-gateway-smoke-permission-denied-investigation.md)
  - Previous title: Gateway Smoke Permission Denied Investigation
- **Supersedes**: None

> Review the previous handoff for full context before filling this one.

## Current State Summary

Investigated why a host-scoped `cli:codex` agent inside Substrate could discover the runtime-owned toolbox endpoint but could not use it to spawn a world worker. The current conclusion is that the toolbox request never reaches Substrate. The immediate `PermissionError(1, 'Operation not permitted')` occurs inside the Codex runtime sandbox when it attempts any Unix-domain socket `connect()`, including to a self-created socket under `/tmp`. Substrate is still part of the root cause because it currently launches Codex in `WorkspaceWrite` sandbox mode with approval policy `Never`, while advertising a raw UDS toolbox contract that this runtime posture cannot actually consume. No code fix was started in this session; the remaining work is to choose the intended product behavior and add end-to-end coverage for actual toolbox connectivity instead of only env/socket disclosure.

## Codebase Understanding

### Architecture Overview

The host orchestrator toolbox surface is split into two layers. First, Substrate binds a per-session Unix socket and injects `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` plus `SUBSTRATE_AGENT_TOOLBOX_VERSION` into host orchestrator runtimes. It also prepends prompt text describing the seven host tools and the versioned request contract. Second, the selected runtime is expected to act as the adapter that issues newline-delimited JSON tool requests over that socket. The transport and contract disclosure are landed, but the runtime-family adapter seam is not actually proven end-to-end for live Codex. Existing smoke coverage validates that the env vars are present and that the advertised socket path is already bound, but it does not validate that the launched runtime can successfully `connect()` to the socket and issue a host-tool request.

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| `crates/shell/src/execution/prompt_fulfillment.rs` | Builds toolbox env/prompt disclosure and configures Codex attach posture | Shows Substrate telling the runtime to call the UDS directly and hardcoding `ApprovalPolicy::Never` plus `SandboxMode::WorkspaceWrite` on attach |
| `crates/shell/src/execution/agent_runtime/control.rs` | Decides when authoritative host toolbox surface is enabled and injects runtime-owned env | Confirms the host orchestrator path intentionally exports the toolbox UDS to the runtime |
| `crates/shell/tests/agent_public_control_surface_v1.rs` | Current host toolbox public-surface tests | Shows tests only asserting contract disclosure and socket boundness, not real runtime connectability |
| `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md` | Canonical truth note for the host orchestration/tool surface landing | States the missing seam is the runtime-owned adapter above the landed transport |
| `crates/gateway/src/adapter_runtime.rs` | Registers the Codex backend with default config | Entry point showing Substrate is using default unified-agent-api Codex posture unless explicitly overridden |
| `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-0.3.5/src/backends/codex/policy.rs` | Default Codex backend policy/config | Confirms default `sandbox_mode` is `WorkspaceWrite` and `external_sandbox` is false |
| `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-0.3.5/src/backends/codex/exec.rs` | Maps backend config into Codex builder calls | Confirms dangerous bypass is only enabled when `external_sandbox` is explicitly opted in |
| `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.5/src/builder/cli_overrides.rs` | Emits Codex CLI argv | Confirms normal path uses `--sandbox` and `--ask-for-approval`, while dangerous bypass is a separate explicit flag |

### Key Patterns Discovered

- Host-tool dispatch is intentionally runtime-owned at the last mile: Substrate exports the contract and endpoint, but the selected agent runtime must perform the actual tool call.
- The current tests use fake Codex/Claude wrappers that only prove “the socket exists and was disclosed,” which can overstate real interoperability.
- The product language in the truth doc distinguishes “transport landed” from “runtime adapter proven”; that distinction matters here because the transport is present but unusable from the live Codex sandbox.
- World-member path allowlists are stricter than host execution and would create a second-layer access issue for `~/.substrate/run/agent-toolbox/...`, but that is not the first failure on the host Codex path because AF_UNIX `connect()` is already blocked generically.

## Work Completed

### Tasks Finished

- [x] Reproduced the issue in a fresh host `cli:codex` session started through `~/.substrate/bin/substrate agent start`.
- [x] Confirmed the runtime received `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` and `SUBSTRATE_AGENT_TOOLBOX_VERSION`.
- [x] Confirmed a direct Python `socket.connect()` to the exported toolbox UDS failed with `PermissionError(1, 'Operation not permitted')`.
- [x] Confirmed the same `PermissionError(1, 'Operation not permitted')` occurs when Codex creates and connects to its own temporary Unix socket under `/tmp`.
- [x] Confirmed the live Codex runtime reports `Seccomp: 2` and `NoNewPrivs: 1` in `/proc/self/status`.
- [x] Traced the Substrate host toolbox disclosure and Codex attach code paths.
- [x] Traced the unified-agent-api Codex backend defaults and CLI override path.
- [x] Confirmed Substrate is not opting into the Codex dangerous-bypass / external-sandbox path on normal host orchestrator runs.
- [x] Stopped the live investigation session after collecting evidence.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| `handoffs/2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md` | Completed scaffold with investigation findings and resume guidance | Preserve the diagnosis and evidence trail for the next session |

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Treat the immediate failure as a Codex runtime sandbox denial, not a Substrate toolbox authorization rejection | toolbox ACL/path bug vs toolbox server rejection vs generic runtime sandbox syscall denial | A self-created `/tmp` Unix socket failed with the same `EPERM`, proving the denial is broader than the Substrate endpoint |
| Treat Substrate as causally involved through launch posture, not as the syscall-denying component | blame only Codex vs blame only Substrate vs split root cause into runtime denial plus launcher posture | Substrate is advertising a raw UDS transport while launching Codex in a sandbox mode that cannot use AF_UNIX `connect()` |
| Conclude dangerous bypass is not active on the normal host orchestrator path | assume hidden override exists vs inspect adapter/runtime/backend code | The live code path uses default Codex backend config, and the wrapper only emits the dangerous bypass flag when `external_sandbox` is explicitly enabled |
| Stop at diagnosis rather than start implementing a fix | patch tests or launch posture immediately vs report findings first | The user asked for investigation and root-cause reporting, not a remediation patch |

## Pending Work

### Immediate Next Steps

1. Decide the intended product behavior for host orchestrator tool calls: either launch Codex in a posture that can actually use the raw UDS transport, or replace the raw UDS expectation with a runtime-owned adapter path that does not require AF_UNIX `connect()` from inside Codex.
2. Add an end-to-end smoke/regression test that proves a live runtime can successfully call at least one host tool over the advertised transport, rather than only asserting env disclosure and `-S` socket presence.
3. If considering the dangerous-bypass / external-sandbox path, evaluate whether that security posture is acceptable for host orchestrator sessions and document the tradeoff explicitly.

### Blockers/Open Questions

- [ ] Is the intended contract that live Codex must call the exported UDS directly, or was the design expecting a higher-level adapter layer that has not landed yet?
- [ ] If direct UDS is intended, should host orchestrator Codex runs opt into the external-sandbox / dangerous-bypass path, or is there another supported posture that allows AF_UNIX `connect()` without broadening the sandbox too far?
- [ ] Should unsupported runtime/toolbox posture be detected up front and surfaced as “toolbox disclosed but not callable in this runtime posture” instead of relying on downstream smoke failures?

### Deferred Items

- Implementing any launch-posture, adapter, or test changes was deferred because this session was limited to root-cause investigation.
- Investigating the secondary world-member allowlist/path issue for `~/.substrate/run/agent-toolbox/...` was deferred because the host Codex runtime already fails earlier on generic AF_UNIX `connect()`.

## Context for Resuming Agent

### Important Context

The most important fact is that the advertised host toolbox UDS is currently unusable from a live Substrate-launched Codex session, and the evidence shows the failure occurs before Substrate can process any request. The decisive repro was not merely “toolbox socket connect failed”; it was “any Unix-domain socket connect failed inside that runtime.” In a fresh `cli:codex` host session, connecting to `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` via Python returned `PermissionError(1, 'Operation not permitted')`. Creating a brand-new temporary UDS listener under `/tmp` from the same runtime and connecting to it also returned `PermissionError(1, 'Operation not permitted')`. The runtime also reported `Seccomp: 2` and `NoNewPrivs: 1`, which is consistent with a seccomp-constrained sandbox. That means the immediate denial is inside the Codex execution sandbox, not a Substrate-side toolbox auth rule or bad socket ACL.

The second critical point is that Substrate currently launches Codex in that sandboxed posture while simultaneously telling it to use the UDS directly. On the Substrate side, `prompt_fulfillment.rs` builds the toolbox env and prompt contract, and the Codex attach path explicitly uses `ApprovalPolicy::Never` plus `SandboxMode::WorkspaceWrite`. On the gateway/backend side, Codex is registered with default wrapper config. In unified-agent-api, the dangerous bypass path is only selected when `external_sandbox` is explicitly enabled; the default config keeps it off. So the most accurate framing is: Codex is denying the syscall, and Substrate is causing that live posture by launching Codex without the dangerous-bypass / external-sandbox equivalent while advertising a direct raw UDS toolbox contract.

### Assumptions Made

- The live Codex backend used by `~/.substrate/bin/substrate agent start --backend cli:codex` is the relevant user-facing path for this smoke verification.
- A self-created `/tmp` UDS connect failure inside the same runtime is sufficient to rule out “Substrate toolbox endpoint only” as the immediate cause.
- The current cargo-registry sources for `unified-agent-api-0.3.5` and `unified-agent-api-codex-0.3.5` match the binaries being exercised by this workspace.
- The user wanted diagnosis only in this session, not a code change.

### Potential Gotchas

- Do not collapse “Codex denied the syscall” into “Substrate is uninvolved.” The runtime denial is real, but Substrate currently chooses that runtime posture.
- Do not over-trust `agent_public_control_surface_v1` as proof of live interoperability. Its fake runtime only checks whether the socket path exists and is already bound.
- There is a secondary path/isolation concern for world-scoped members because `~/.substrate/run/agent-toolbox/...` is outside project-root allowlists, but that is not the first failing seam on the host path.
- The repo root contains `.codex` as an empty file, not a directory. The `session-handoff` skill default handoff location fails here; use `--dir handoffs`.

## Environment State

### Tools/Services Used

- `~/.substrate/bin/substrate agent start --backend cli:codex ...`: created the live host Codex session used for repro.
- `~/.substrate/bin/substrate agent turn --backend cli:codex --session ... --prompt ...`: used to ask the live runtime to print env vars, run Python socket probes, and report `/proc/self/status`.
- `~/.substrate/bin/substrate agent stop --session ...`: stopped the investigation session after evidence collection.
- `sed`, `find`, `ls`, and repo source inspection: used to trace Substrate and unified-agent-api code paths.
- `python .../create_handoff.py substrate-codex-toolbox-uds-investigation --dir handoffs`: created this handoff scaffold because the default `.codex/handoffs` path is invalid in this repo.

### Active Processes

- No investigation-specific long-lived process remains. The live `cli:codex` repro session was stopped after collecting evidence.
- The normal Substrate host/world background services on the machine were not modified by this session.

### Environment Variables

- `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
- `SUBSTRATE_AGENT_TOOLBOX_VERSION`
- `SUBSTRATE_HOME`
- `CODEX_HANDOFF_DIR`

## Related Resources

- [crates/shell/src/execution/prompt_fulfillment.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/prompt_fulfillment.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/home/spenser/__Active_code/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/home/spenser/__Active_code/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [crates/gateway/src/adapter_runtime.rs](/home/spenser/__Active_code/substrate/crates/gateway/src/adapter_runtime.rs)
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-0.3.5/src/backends/codex/policy.rs`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-0.3.5/src/backends/codex/exec.rs`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-codex-0.3.5/src/builder/cli_overrides.rs`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unified-agent-api-0.3.5/src/backends/codex/tests/external_sandbox.rs`
- [handoffs/2026-06-15-194703-gateway-smoke-permission-denied-investigation.md](/home/spenser/__Active_code/substrate/handoffs/2026-06-15-194703-gateway-smoke-permission-denied-investigation.md)

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
