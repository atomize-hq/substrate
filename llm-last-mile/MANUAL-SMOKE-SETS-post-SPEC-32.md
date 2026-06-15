# Manual Smoke Sets Post-SPEC-32

Status: draft manual smoke package for all landed post-`SPEC-32` orchestration/runtime slices through Slice `56`.

## Purpose

This document converts the landed post-`SPEC-32` orchestration/runtime work into six human-run manual smoke sets.

It is intentionally operator-facing:

1. each set is organized as a small number of higher-value verification flows rather than one flow per slice,
2. each flow is grounded in current repo truth from source docs, tests, and code,
3. each set states exact platform expectations, including Linux source-of-truth behavior, supported macOS/Lima parity paths, and intentional Windows/WSL fail-closed boundaries.

## Source Anchors

Primary source anchors for this manual package:

1. `AGENTS.md`
2. `llm-last-mile/SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md`
3. relevant `SPEC-*`, `PLAN-*`, and `TASKS-*` through Slice `56`
4. `llm-last-mile/REMAINING-overall-scope-2026-06-10.md`
5. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
6. `docs/USAGE.md`
7. `docs/WORLD.md`
8. `docs/REPLAY.md`
9. relevant shell/world/agent runtime tests and platform smoke scripts

## Recommended Execution Order

Run the six sets in this order unless you are explicitly revalidating a single seam in isolation:

1. Set 1: Core World Dispatch and Retained Worker Lifecycle
2. Set 2: Worker-to-Host Obligation and Response Loop
3. Set 3: Deferred Work, Auto-Attach, and Host Inbox Materialization
4. Set 4: Public/Operator Surface Contract and Session Continuity
5. Set 5: Toolbox Transport and Runtime-Family Host-Tool Parity
6. Set 6: Platform Parity and Fail-Closed Boundaries

Rationale:

1. Sets `1` through `3` validate the runtime and deferred-work foundations.
2. Sets `4` and `5` validate the surfaced operator contract above those foundations.
3. Set `6` closes by proving which platform claims are actually supported and which remain intentionally fail-closed.

## Shared Operator Assumptions

Assume the following unless a set says otherwise:

1. run from the repo root: `/home/azureuser/__Active_Code/atomize-hq/substrate`
2. prefer a fresh disposable `SUBSTRATE_HOME` per smoke run
3. use the local debug binary when exact command behavior matters:
   - `target/debug/substrate`
4. on Linux, run from a shell that actually has `substrate` group access when world-backed flows require `/run/substrate.sock`
5. capture command output for each step because several pass/fail checks depend on exact JSON fields or explicit fail-closed errors

---

## Smoke Set 1: Core World Dispatch and Retained Worker Lifecycle

### 1. What this set proves

This set proves the post-`SPEC-32` host-orchestrator to world-dispatch seam on Linux: a live host runtime can use the internal toolbox to launch world work, obtain authoritative task or worker identity, and then steer retained work through inspect, fork, cancel, and stop using exact handles. It also proves the contract stays fail-closed when the target backend, action policy, or identity shape is wrong.

### 2. Landed seams/slices covered

Primary slices:

1. `SPEC-32` world-dispatch bootstrap
2. `SPEC-33` retained worker continue and event bootstrap
3. `SPEC-34` steering-policy hardening
4. `SPEC-35` retained inspect snapshot
5. `SPEC-36` retained stop closeout
6. `SPEC-37` cancel world work
7. `SPEC-38` retained worker fork
8. `SPEC-47` exact active-task identity and inspect/cancel widening

### 3. Platform scope

1. Linux: required full pass surface
2. macOS/Lima: read-side and public surfaces may work, but this internal live world-dispatch bootstrap is not the supported validation floor for this set
3. Windows/WSL: expected fail-closed or unsupported for this set

### 4. Preconditions

1. `cargo build -p substrate`
2. Linux world service is provisioned and healthy:
   - `target/debug/substrate world doctor --json`
3. `python3` and `jq` are available
4. You know one exact host backend id and one exact world backend id from:
   - `target/debug/substrate agent list --json`
   - `target/debug/substrate agent doctor --json`

### 5. Setup

Prepare an isolated home and runtime:

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
cargo build -p substrate
export S="$PWD/target/debug/substrate"
export TMP="$(mktemp -d /tmp/substrate-set1-XXXXXX)"
export HOME="$TMP/home"
export SUBSTRATE_HOME="$TMP/substrate-home"
mkdir -p "$HOME" "$SUBSTRATE_HOME"
```

Identify the backends you will use:

```bash
"$S" agent list --json | jq '.'
"$S" agent doctor --json | jq '.'
```

Set them explicitly before continuing:

```bash
export HOST_BACKEND='<exact host backend id>'
export WORLD_BACKEND='<exact world backend id>'
```

Start a live host-owned orchestrator session in Terminal A:

```bash
"$S"
```

Inside the REPL, submit one host-targeted start:

```text
::$HOST_BACKEND start retained host runtime
```

In Terminal B, export the live toolbox endpoint:

```bash
"$S" agent toolbox env --json | tee "$TMP/toolbox-env.json"
export SUBSTRATE_AGENT_TOOLBOX_ENDPOINT="$(jq -r '.SUBSTRATE_AGENT_TOOLBOX_ENDPOINT' "$TMP/toolbox-env.json")"
export SUBSTRATE_AGENT_TOOLBOX_VERSION="$(jq -r '.SUBSTRATE_AGENT_TOOLBOX_VERSION' "$TMP/toolbox-env.json")"
```

Create a small helper that sends one versioned request over the UDS and prints each returned frame:

```bash
cat > "$TMP/toolbox_send.py" <<'PY'
import os, socket, sys
endpoint = os.environ["SUBSTRATE_AGENT_TOOLBOX_ENDPOINT"]
if not endpoint.startswith("unix://"):
    raise SystemExit(f"expected unix:// endpoint, got {endpoint}")
path = endpoint[len("unix://"):]
payload = sys.stdin.read().strip()
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(path)
sock.sendall(payload.encode() + b"\n")
sock.shutdown(socket.SHUT_WR)
while True:
    data = sock.recv(65536)
    if not data:
        break
    sys.stdout.buffer.write(data)
PY
```

### 6. Step-by-step manual procedure

1. Prove the internal surface is live but still fail-closed for a bad target backend.

```bash
printf '%s\n' \
'{"version":1,"tool_name":"run_world_task","tool_call_id":"set1-bad-backend","arguments":{"target_backend_id":"cli:not-a-real-backend","payload":{"prompt":"reply with one word"}}}' \
| python3 "$TMP/toolbox_send.py"
```

2. Spawn one retained worker and save the authoritative `participant_id`.

```bash
printf '%s\n' \
'{"version":1,"tool_name":"spawn_world_worker","tool_call_id":"set1-spawn","arguments":{"target_backend_id":"'"$WORLD_BACKEND"'","payload":{"prompt":"State that you are alive, then wait for follow-up instructions."}}}' \
| python3 "$TMP/toolbox_send.py" | tee "$TMP/spawn.jsonl"

export PARTICIPANT_ID="$(jq -r 'select(.frame_kind=="result" and .ok==true) | .outcome.participant_id' "$TMP/spawn.jsonl")"
```

3. Inspect the retained worker by exact `participant_id`.

```bash
printf '%s\n' \
'{"version":1,"tool_name":"inspect_world_worker","tool_call_id":"set1-inspect","arguments":{"participant_id":"'"$PARTICIPANT_ID"'","payload":{}}}' \
| python3 "$TMP/toolbox_send.py" | tee "$TMP/inspect.jsonl"
```

4. Fork the retained worker from the exact source participant and capture the child identity.

```bash
printf '%s\n' \
'{"version":1,"tool_name":"fork_world_worker","tool_call_id":"set1-fork","arguments":{"participant_id":"'"$PARTICIPANT_ID"'","payload":{"child_prompt":"State that you are the forked worker.","fork_reason":"manual_smoke"}}}' \
| python3 "$TMP/toolbox_send.py" | tee "$TMP/fork.jsonl"

export CHILD_PARTICIPANT_ID="$(jq -r 'select(.frame_kind=="result" and .ok==true) | .outcome.participant_id' "$TMP/fork.jsonl")"
```

5. Stop the child, then stop the original source worker.

```bash
printf '%s\n' \
'{"version":1,"tool_name":"stop_world_worker","tool_call_id":"set1-stop-child","arguments":{"participant_id":"'"$CHILD_PARTICIPANT_ID"'","payload":{}}}' \
| python3 "$TMP/toolbox_send.py" | tee "$TMP/stop-child.jsonl"

printf '%s\n' \
'{"version":1,"tool_name":"stop_world_worker","tool_call_id":"set1-stop-source","arguments":{"participant_id":"'"$PARTICIPANT_ID"'","payload":{}}}' \
| python3 "$TMP/toolbox_send.py" | tee "$TMP/stop-source.jsonl"
```

6. Close the exact task-identity widening path with the landed targeted tests for ephemeral inspect and cancel.

```bash
cargo test -p shell --test repl_world_first_routing_v1 \
  c3_internal_toolbox_run_world_task_streams_registered_task_run_id_before_terminal_result_and_ephemeral_inspect_uses_it \
  -- --nocapture

cargo test -p shell --test repl_world_first_routing_v1 \
  c3_internal_toolbox_run_world_task_ephemeral_cancel_uses_registered_task_run_id_and_execute_cancel_surface \
  -- --nocapture
```

7. Prove mixed or wrong exact-target shapes still fail closed.

```bash
cargo test -p shell agent_runtime::dispatch_contract::world_dispatch_contract_rejects_inspect_world_worker_retained_mode_with_task_run_id -- --nocapture
cargo test -p shell agent_runtime::dispatch_contract::world_dispatch_contract_rejects_inspect_world_worker_ephemeral_mode_with_mixed_identity -- --nocapture
cargo test -p shell agent_runtime::dispatch_contract::world_dispatch_contract_rejects_cancel_world_work_retained_mode_with_task_run_id -- --nocapture
cargo test -p shell agent_runtime::dispatch_contract::world_dispatch_contract_rejects_cancel_world_work_ephemeral_mode_with_mixed_identity -- --nocapture
```

### 7. What to verify after each step

1. Step 1: returned frame is `frame_kind=result`, `ok=false`; the error explains the backend or posture was invalid
2. Step 2: returned result is `ok=true`; the outcome includes exact `participant_id`, `target_backend_id`, `world_id`, and `world_generation`
3. Step 3: returned result is `ok=true`; the outcome is snapshot-only and does not imply mutation
4. Step 4: returned result is `ok=true`; the outcome includes both the child `participant_id` and the exact `source_participant_id`
5. Step 5: both stop calls return `ok=true`; no follow-up handle remains routable for the stopped participants
6. Step 6: the first test shows a streamed `task_run_id_registered` event before terminal result; the second shows exact `task_run_id` cancel routing
7. Step 7: each targeted negative test passes because the contract rejects mixed `participant_id` plus `task_run_id` targeting

### 8. Pass/fail criteria

Pass if:

1. the live internal toolbox path works on Linux for retained spawn, inspect, fork, and stop,
2. exact identities are surfaced by the outcomes rather than inferred,
3. ephemeral task identity is proven by the targeted `task_run_id` tests,
4. bad backend selection and mixed exact-target shapes fail closed.

Fail if:

1. `toolbox env` does not publish a live endpoint while the host runtime is active,
2. retained outcomes omit exact identity fields,
3. fork loses lineage, or stop leaves the worker routable,
4. mixed retained and ephemeral selectors are accepted.

### 9. Cleanup

1. exit the REPL in Terminal A
2. remove the disposable home:

```bash
rm -rf "$TMP"
```

### 10. Notes / intentional fail-closed cases

1. There is still no public human CLI for `run_world_task`, `spawn_world_worker`, or the other six internal verbs; Set 1 therefore uses the internal toolbox UDS plus targeted tests.
2. Linux is the source-of-truth pass surface for live internal world dispatch.
3. macOS/Lima and Windows/WSL should not be treated as supported live-pass platforms for this internal bootstrap seam.

---

## Smoke Set 2: Worker-to-Host Obligation and Response Loop

### 1. What this set proves

This set proves the retained worker to durable obligation to host response/control loop. A human operator verifies that the landed response classes are exact, deny-by-default until enabled, persist or do not persist exactly as intended, and close or remain pending according to delivery success.

### 2. Landed seams/slices covered

Primary slices:

1. `SPEC-39` retained worker approval and fork obligation bootstrap
2. `SPEC-40` host approval response bootstrap
3. `SPEC-41` follow-up and blocked obligation hardening
4. `SPEC-42` host clarification response bootstrap
5. `SPEC-43` retained host control directive bootstrap
6. `SPEC-44` retained worker control ack bootstrap
7. `SPEC-45` retained host fork command bootstrap
8. `SPEC-46` retained host progress ack bootstrap

### 3. Platform scope

1. Linux: required pass platform for this validation wall
2. macOS/Lima: not a live-pass surface for this set; use only as reference
3. Windows/WSL: not a supported pass surface for this set

### 4. Preconditions

1. the workspace builds the `shell` and `substrate-broker` test targets
2. `RUST_TEST_THREADS=1` is exported to keep the targeted tests readable
3. `SUBSTRATE_HOME`, `SUBSTRATE_WORLD_SOCKET`, and `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` are unset before starting

### 5. Setup

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
cargo test -p shell --no-run
cargo test -p substrate-broker --no-run
export RUST_TEST_THREADS=1
unset SUBSTRATE_HOME SUBSTRATE_WORLD_SOCKET SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE
```

### 6. Step-by-step manual procedure

1. Prove deny-by-default gating before target resolution.

```bash
cargo test -p substrate-broker pcm1_policy_yaml_accepts_llm_agents_and_workflow_router_families -- --nocapture

for t in \
  dispatch_contract_continue_world_worker_approval_response_denies_by_default_before_target_resolution \
  dispatch_contract_continue_world_worker_clarification_response_denies_by_default_before_target_resolution \
  dispatch_contract_continue_world_worker_control_directive_denies_by_default_before_target_resolution \
  dispatch_contract_continue_world_worker_progress_ack_denies_by_default_before_target_resolution \
  dispatch_contract_continue_world_worker_fork_command_denies_by_default_before_target_resolution
do
  cargo test -p shell "$t" -- --nocapture
done
```

2. Prove worker-originated durable obligations are persisted exactly once and preserve reviewable fields.

```bash
for t in \
  continue_world_worker_stream_preserves_packet_one_worker_request_when_normal_event_follows \
  dispatch_contract_continue_world_worker_persists_packet_three_worker_obligations_exactly_once_on_live_path \
  dispatch_contract_continue_world_worker_persists_fork_request_without_allocating_child_worker \
  persist_packet_two_worker_request_obligations_preserves_exact_reviewable_fields
do
  cargo test -p shell "$t" -- --nocapture
done
```

3. Prove approval and clarification responses close only after successful delivery and remain pending when delivery fails.

```bash
for t in \
  dispatch_contract_continue_world_worker_approval_response_closes_after_successful_delivery \
  dispatch_contract_continue_world_worker_approval_response_leaves_obligation_pending_when_delivery_fails \
  dispatch_contract_continue_world_worker_clarification_response_closes_follow_up_exactly_once_after_successful_delivery \
  dispatch_contract_continue_world_worker_clarification_response_leaves_follow_up_pending_when_delivery_fails
do
  cargo test -p shell "$t" -- --nocapture
done
```

4. Prove control-directive and progress-ack behavior is exact and non-obligating when intended.

```bash
for t in \
  continue_world_worker_dispatch_contract_accepts_control_ack_only_for_typed_control_directive_turns \
  dispatch_contract_continue_world_worker_control_directive_surfaces_control_ack_without_persisting_obligation \
  dispatch_contract_continue_world_worker_progress_ack_surfaces_progress_update_without_persisting_obligation \
  continue_world_worker_progress_ack_summary_stays_seen_progress_only
do
  cargo test -p shell "$t" -- --nocapture
done
```

5. Prove fork-command delivery and the invalid-context fail-closed boundary.

```bash
for t in \
  dispatch_contract_continue_world_worker_fork_command_submits_rendered_prompt_to_exact_retained_source \
  dispatch_contract_continue_world_worker_fork_command_rejects_terminal_exact_source_before_delivery
do
  cargo test -p shell "$t" -- --nocapture
done

cargo test -p shell --test repl_world_first_routing_v1 \
  c3_internal_toolbox_control_ack_fail_closed_for_invalid_contexts_and_out_of_scope_worker_events \
  -- --nocapture
```

### 7. What to verify after each step

1. Step 1: the deny-by-default tests pass specifically because resolution is rejected before stale or wrong targets are considered
2. Step 2: the live-path obligation tests show one durable record per exact worker-originated request and preserve the expected reviewable fields
3. Step 3: successful delivery closes the obligation or follow-up exactly once; delivery failure leaves it pending
4. Step 4: control-ack and progress-ack surface exact feedback without being misclassified as durable review obligations
5. Step 5: fork-command routing targets the exact retained source; invalid or terminal contexts fail closed

### 8. Pass/fail criteria

Pass if:

1. each response/control class preserves its intended persistence semantics,
2. deny-by-default policy remains in force until exact enablement,
3. failed delivery does not silently close obligations,
4. invalid control-ack contexts are rejected.

Fail if:

1. any response class bypasses policy or target validation,
2. obligations double-persist or close on failed delivery,
3. control-ack or progress-ack creates durable review state when it should not,
4. fork-command silently allocates or redirects without exact retained source authority.

### 9. Cleanup

No special cleanup is required beyond unsetting any temporary env vars you introduced for the run.

### 10. Notes / intentional fail-closed cases

1. This set is intentionally test-driven because the public operator surface does not yet expose a human CLI for these internal obligation-response payloads.
2. The contract is supposed to be strict here: wrong event class, wrong context, or policy-disabled routes should fail closed rather than degrade.

---

## Smoke Set 3: Deferred Work, Auto-Attach, and Host Inbox Materialization

### 1. What this set proves

This set proves the landed Family-2 local deferred-work semantics: retained-worker obligations become host-targeted inbox envelopes, router-owned auto-attach stays deny-by-default and exact-host-scoped, and the only accepted local materialization order is `host_inbox -> local obligation -> router consumption`.

### 2. Landed seams/slices covered

Primary slices:

1. `SPEC-48` router-owned session auto-attach execution boundary
2. `SPEC-49` host-targeted obligation envelope and wrong-host fail-closed boundary
3. `SPEC-50` ingress-ready obligation identity and causation envelope
4. `SPEC-51` host-global inbox layering and local-obligation materialization boundary

### 3. Platform scope

1. Linux: required pass platform for this set
2. macOS/Lima: not a claimed live-pass platform for the Family-2 local semantics
3. Windows/WSL: expected fail-closed or unsupported for this set

### 4. Preconditions

1. the `shell` test target builds
2. you can run targeted integration tests locally
3. any existing `SUBSTRATE_HOME` for the run is disposable

### 5. Setup

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
cargo test -p shell --no-run
export RUST_TEST_THREADS=1
```

### 6. Step-by-step manual procedure

1. Prove the canonical projection from retained-worker continue events into deferred-work canonical state.

```bash
cargo test -p shell \
  dispatch_contract_persist_continue_world_worker_obligation_projects_supported_events_into_canonical_state \
  -- --nocapture
```

2. Prove exact `host_inbox -> local obligation` materialization.

```bash
cargo test -p shell \
  host_inbox_state_store_materializes_valid_records_into_one_local_obligation \
  -- --nocapture
```

3. Prove the materialization entrypoint keeps the router consuming obligations only, not raw host inbox records.

```bash
cargo test -p shell \
  host_inbox_materialization_entrypoint_keeps_router_consuming_obligations_only \
  -- --nocapture
```

4. Prove router-owned auto-attach stays deny-by-default when the router policy is disabled.

```bash
cargo test -p shell \
  router_owned_auto_attach_session_trigger_fails_closed_for_detached_continue_world_worker_obligations_when_router_policy_is_disabled \
  -- --nocapture
```

5. Prove wrong-host records fail closed and do not create local obligations.

```bash
cargo test -p shell \
  host_inbox_state_store_fail_closes_wrong_host_records_without_creating_obligations \
  -- --nocapture

cargo test -p shell \
  execute_session_auto_attach_fails_closed_before_launch_when_target_host_is_foreign \
  -- --nocapture
```

6. Prove inbox-read failure does not widen authority and only preserves best-effort discovery.

```bash
cargo test -p shell \
  router_owned_auto_attach_discovery_continues_when_host_inbox_pre_pass_cannot_read_directory \
  -- --nocapture
```

### 7. What to verify after each step

1. Step 1: the persisted canonical state preserves supported event classes and identity/causation metadata
2. Step 2: exactly one valid host inbox record becomes exactly one local obligation
3. Step 3: the router consumes the local obligation layer, not the raw inbox layer
4. Step 4: auto-attach is blocked when the router policy is disabled
5. Step 5: wrong-host records do not silently attach, route, or materialize locally
6. Step 6: directory-read problems do not grant broader authority; they only preserve bounded fail-closed behavior

### 8. Pass/fail criteria

Pass if:

1. the host inbox is treated as a host-targeted staging layer rather than a directly consumable router queue,
2. local obligation materialization is exact and single-hop,
3. auto-attach remains router-owned and policy-gated,
4. wrong-host and foreign-host paths fail closed.

Fail if:

1. router code consumes raw host inbox records directly,
2. wrong-host records become local obligations,
3. auto-attach succeeds when disabled,
4. materialization loses identity or causation metadata.

### 9. Cleanup

No special cleanup is required after the targeted test run.

### 10. Notes / intentional fail-closed cases

1. The local deferred-work truth in this slice is intentionally Linux-first and internal. There is still no public human inbox-management CLI.
2. The canonical order matters: `host_inbox -> local obligation -> router`. Reversing or collapsing that order is a regression.

---

## Smoke Set 4: Public/Operator Surface Contract and Session Continuity

### 1. What this set proves

This set proves the current operator-facing contract: exact targeting by `orchestration_session_id`, host-rooted world-backed start, exact `start|turn|reattach|fork|stop`, REPL exact targeting, durable parked or attention postures, helper-owned streaming, and the strict split between readable status surfaces and fail-closed control surfaces.

### 2. Landed seams/slices covered

Primary slices:

1. `SPEC-30` public world-scoped start and capability flags
2. `SPEC-31` lazy host attach for host-rooted world start
3. `SPEC-53` first runtime-family host-tool surface landing
4. `SPEC-54` second runtime-family parity
5. `SPEC-55` broader caller surface contract freeze
6. `SPEC-56` read-side and strict-control surface hardening

### 3. Platform scope

1. Linux: required full pass surface, including host-rooted world-backed start
2. macOS/Lima: public host flows should work; `agent start --scope world` is expected fail-closed
3. Windows/WSL: treat world-backed public agent flows as fail-closed or unsupported unless explicitly validated later

### 4. Preconditions

1. `target/debug/substrate` exists
2. at least one host backend is configured and realizable
3. on Linux, a world backend is configured and realizable if you want to run the world-scoped path
4. `jq` is available

### 5. Setup

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
export S="$PWD/target/debug/substrate"
export TMP="$(mktemp -d /tmp/substrate-set4-XXXXXX)"
export HOME="$TMP/home"
export SUBSTRATE_HOME="$TMP/substrate-home"
mkdir -p "$HOME" "$SUBSTRATE_HOME"
```

Record available backends and current posture:

```bash
"$S" agent doctor --json | tee "$TMP/doctor.json"
"$S" agent list --json | tee "$TMP/list.json"
"$S" agent toolbox status --json | tee "$TMP/toolbox-status-pre.json"
```

Set the exact backends you plan to use:

```bash
export HOST_BACKEND='<exact host backend id>'
export WORLD_BACKEND='<exact world backend id, if present>'
```

### 6. Step-by-step manual procedure

1. Read the exact public control surface.

```bash
"$S" agent turn --help
"$S" agent reattach --help
```

2. Start one host-scoped durable session and capture the exact `orchestration_session_id`.

```bash
"$S" agent start --backend "$HOST_BACKEND" --prompt "say startup prompt success" --json \
  | tee "$TMP/start-host.json"

export SESSION_A="$(jq -r '.orchestration_session_id' "$TMP/start-host.json")"
```

3. Prove non-canonical selectors fail closed.

```bash
"$S" agent turn --session bogus-participant-id --backend "$HOST_BACKEND" --prompt "should fail" --json
```

4. Submit an exact follow-up turn to the same durable session.

```bash
"$S" agent turn --session "$SESSION_A" --backend "$HOST_BACKEND" --prompt "say follow-up prompt success" --json \
  | tee "$TMP/turn-host.json"
```

5. Fork the session, capture the successor id, then stop the original.

```bash
"$S" agent fork --session "$SESSION_A" --json | tee "$TMP/fork-host.json"
export SESSION_B="$(jq -r '.orchestration_session_id' "$TMP/fork-host.json")"

"$S" agent stop --session "$SESSION_A" --json | tee "$TMP/stop-host-a.json"
```

6. Compare readable status surfaces with strict control surfaces.

```bash
"$S" agent status --json | tee "$TMP/status.json"
"$S" agent toolbox status --json | tee "$TMP/toolbox-status-live.json"
"$S" agent toolbox env --json | tee "$TMP/toolbox-env-live.json"
```

7. Reattach to the exact successor session, then stop it.

```bash
"$S" agent reattach --session "$SESSION_B" --json | tee "$TMP/reattach-b.json"
"$S" agent stop --session "$SESSION_B" --json | tee "$TMP/stop-host-b.json"
```

8. Prove REPL exact targeting behavior.

```bash
"$S"
```

Inside the REPL:

```text
plain text in the REPL
::bogus-backend this should fail
::$HOST_BACKEND REPL exact target smoke
```

9. From the same REPL, create one exact world-targeted member slot, then continue from the public surface using the exact public pair.

```text
::$WORLD_BACKEND REPL world exact target smoke
```

In a second terminal, discover the resulting exact `orchestration_session_id` from `agent status --json`, then submit:

```bash
"$S" agent turn --session "<exact session id from status>" --backend "$WORLD_BACKEND" --prompt "second exact world turn" --json
```

10. Linux-only: prove host-rooted world-backed start works and public follow-up stays exact.

```bash
"$S" agent start --backend "$WORLD_BACKEND" --scope world --prompt "linux world start smoke" --json \
  | tee "$TMP/start-world.json"
```

11. Non-Linux or selected unsupported posture: prove `--scope world` fails closed instead of silently degrading.

```bash
"$S" agent start --backend "$WORLD_BACKEND" --scope world --prompt "should fail closed here" --json
```

### 7. What to verify after each step

1. Step 1: help text exposes only the narrow public contract, not internal world-dispatch verbs
2. Step 2: start returns one exact `orchestration_session_id`
3. Step 3: the non-canonical selector fails closed; it does not reinterpret `participant_id` as a session selector
4. Step 4: follow-up succeeds only when both exact `--session` and exact `--backend` are supplied
5. Step 5: fork returns a new durable session id; stop cleanly closes the original
6. Step 6: `agent status` remains readable; `toolbox status` stays exact-session anchored; `toolbox env` publishes env only when a live host session exists
7. Step 7: reattach is exact-session continuity only, not a new prompt-taking path
8. Step 8: plain REPL text is shell execution, not hidden default-agent routing; malformed backend targeting fails closed; valid exact targeting works
9. Step 9: public follow-up can target the exact existing world-backed session; it does not guess from history
10. Step 10: on Linux, `--scope world` still returns a host-rooted durable session with authoritative world binding
11. Step 11: on unsupported platforms or postures, `--scope world` returns an explicit fail-closed error such as `unsupported_platform_or_posture`

### 8. Pass/fail criteria

Pass if:

1. all public prompt-taking and control operations require exact session targeting,
2. read-side surfaces remain readable without becoming authorization shortcuts,
3. `toolbox env` stays strict,
4. Linux world-backed start is host-rooted and exact, while unsupported platforms fail closed.

Fail if:

1. the public surface accepts `participant_id` or compatibility handles as session selectors,
2. REPL plain text is reinterpreted as default agent routing,
3. `toolbox env` succeeds with no current live host session,
4. unsupported `--scope world` paths silently degrade into host-only execution.

### 9. Cleanup

1. exit any open REPL session
2. stop any remaining started sessions:

```bash
"$S" agent status --json
```

3. remove the disposable home:

```bash
rm -rf "$TMP"
```

### 10. Notes / intentional fail-closed cases

1. Public operator continuity is intentionally anchored to exact `orchestration_session_id`. Compatibility identifiers are not accepted as control selectors.
2. `agent status` is allowed to be a readable surface. `toolbox env` is not.
3. On macOS/Lima, the claimed parity path is forwarded host parity, not public world-member start parity.

---

## Smoke Set 5: Toolbox Transport and Runtime-Family Host-Tool Parity

### 1. What this set proves

This set proves the runtime-owned toolbox transport and the current runtime-family parity claim: `toolbox status|env` tell the truth, the frozen seven-tool adapter contract is the one live contract, Codex is a validated host-tool floor, selected-host `claude_code` follows the same semantic path, and world-backed sessions project binding proof onto the same session-scoped toolbox surface.

### 2. Landed seams/slices covered

Primary slices:

1. `SPEC-52` runtime-owned host-orchestrator tool adapter contract freeze
2. `SPEC-53` first runtime-family host-tool surface landing
3. `SPEC-54` second runtime-family host-tool parity

### 3. Platform scope

1. Linux: required full pass surface
2. macOS/Lima: host-session parity surfaces should be readable; world-member host-tool validation is not the claimed pass floor here
3. Windows/WSL: world-backed host-tool parity is not claimed; fail-closed behavior is acceptable and expected

### 4. Preconditions

1. `target/debug/substrate` exists
2. `jq` is available
3. you are willing to use disposable fake runtime wrappers to capture env injection and prompt/runtime behavior

### 5. Setup

Create a disposable harness:

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
export S="$PWD/target/debug/substrate"
export TMP="$(mktemp -d /tmp/substrate-set5-XXXXXX)"
export HOME="$TMP/home"
export SUBSTRATE_HOME="$TMP/substrate-home"
mkdir -p "$HOME" "$SUBSTRATE_HOME/agents" "$TMP/workspace/.substrate"
cd "$TMP/workspace"
```

Create fake Codex and Claude wrappers that record the injected toolbox env and stay alive long enough for follow-up turns:

```bash
cat > "$TMP/fake-codex.sh" <<SH
#!/bin/sh
STATE_FILE="$TMP/fake-codex.count"
SCRIPT_DIR="$TMP"
count=0
if [ -f "$STATE_FILE" ]; then count=$(cat "$STATE_FILE"); fi
count=$((count + 1))
printf '%s' "$count" > "$STATE_FILE"
printf '%s\n' "$@" > "$SCRIPT_DIR/fake-codex-$count.args"
TOOLBOX_ENDPOINT="${SUBSTRATE_AGENT_TOOLBOX_ENDPOINT-}"
TOOLBOX_VERSION="${SUBSTRATE_AGENT_TOOLBOX_VERSION-}"
TOOLBOX_BOUND=
case "$TOOLBOX_ENDPOINT" in
  unix://*)
    TOOLBOX_SOCKET="${TOOLBOX_ENDPOINT#unix://}"
    if [ -S "$TOOLBOX_SOCKET" ]; then TOOLBOX_BOUND=1; else TOOLBOX_BOUND=0; fi
    ;;
esac
{
  printf 'SUBSTRATE_AGENT_TOOLBOX_ENDPOINT=%s\n' "$TOOLBOX_ENDPOINT"
  printf 'SUBSTRATE_AGENT_TOOLBOX_VERSION=%s\n' "$TOOLBOX_VERSION"
  printf 'SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_BOUND=%s\n' "$TOOLBOX_BOUND"
} > "$SCRIPT_DIR/fake-codex-$count.env"
cat > "$SCRIPT_DIR/fake-codex-$count.stdin"
if [ "$count" -eq 1 ]; then
  trap 'exit 0' INT TERM
  printf '{"type":"thread.started","thread_id":"thread-test"}\r\n'
  printf '{"type":"turn.started","thread_id":"thread-test","turn_id":"turn-1"}\r\n'
  printf '{"type":"item.completed","thread_id":"thread-test","turn_id":"turn-1","item_id":"msg-1","status":"completed","item_type":"agent_message","content":{"text":"startup prompt success"}}\r\n'
  printf '{"type":"turn.completed","thread_id":"thread-test","turn_id":"turn-1"}\r\n'
  while :; do sleep 1; done
fi
printf '{"type":"thread.resumed","thread_id":"thread-test"}\r\n'
printf '{"type":"turn.started","thread_id":"thread-test","turn_id":"turn-%s"}\r\n' "$count"
printf '{"type":"item.completed","thread_id":"thread-test","turn_id":"turn-%s","item_id":"msg-%s","status":"completed","item_type":"agent_message","content":{"text":"follow-up prompt success"}}\r\n' "$count" "$count"
printf '{"type":"turn.completed","thread_id":"thread-test","turn_id":"turn-%s"}\r\n' "$count"
SH
chmod +x "$TMP/fake-codex.sh"

cat > "$TMP/fake-claude.sh" <<SH
#!/bin/sh
STATE_FILE="$TMP/fake-claude.count"
SCRIPT_DIR="$TMP"
count=0
if [ -f "$STATE_FILE" ]; then count=$(cat "$STATE_FILE"); fi
count=$((count + 1))
printf '%s' "$count" > "$STATE_FILE"
ARGS_PATH="$SCRIPT_DIR/fake-claude-$count.args"
PROMPT_PATH="$SCRIPT_DIR/fake-claude-$count.prompt"
STDIN_PATH="$SCRIPT_DIR/fake-claude-$count.stdin"
: > "$ARGS_PATH"
last_arg=
for arg in "$@"; do
  printf '%s\n' "$arg" >> "$ARGS_PATH"
  last_arg="$arg"
done
printf '%s' "$last_arg" > "$PROMPT_PATH"
TOOLBOX_ENDPOINT="${SUBSTRATE_AGENT_TOOLBOX_ENDPOINT-}"
TOOLBOX_VERSION="${SUBSTRATE_AGENT_TOOLBOX_VERSION-}"
TOOLBOX_BOUND=
case "$TOOLBOX_ENDPOINT" in
  unix://*)
    TOOLBOX_SOCKET="${TOOLBOX_ENDPOINT#unix://}"
    if [ -S "$TOOLBOX_SOCKET" ]; then TOOLBOX_BOUND=1; else TOOLBOX_BOUND=0; fi
    ;;
esac
{
  printf 'SUBSTRATE_AGENT_TOOLBOX_ENDPOINT=%s\n' "$TOOLBOX_ENDPOINT"
  printf 'SUBSTRATE_AGENT_TOOLBOX_VERSION=%s\n' "$TOOLBOX_VERSION"
  printf 'SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_BOUND=%s\n' "$TOOLBOX_BOUND"
} > "$SCRIPT_DIR/fake-claude-$count.env"
cat > "$STDIN_PATH"
printf '{"type":"system","subtype":"init","session_id":"thread-test"}\n'
if [ "$count" -eq 1 ]; then
  printf '{"type":"assistant","session_id":"thread-test","message":{"content":[{"type":"text","text":"startup prompt success"}]}}\n'
else
  printf '{"type":"assistant","session_id":"thread-test","message":{"content":[{"type":"text","text":"follow-up prompt success"}]}}\n'
fi
printf '{"type":"user","session_id":"thread-test","message":{"content":[{"type":"text","text":"ack"}]}}\n'
printf '{"type":"result","subtype":"success","session_id":"thread-test","is_error":false}\n'
if [ "$count" -eq 1 ]; then
  trap 'exit 0' INT TERM
  while :; do sleep 1; done
fi
SH
chmod +x "$TMP/fake-claude.sh"
```

Create a disposable config, policy, and agent inventory:

```bash
cat > "$SUBSTRATE_HOME/config.yaml" <<'YAML'
agents:
  enabled: true
  hub:
    orchestrator_agent_id: codex
  toolbox:
    enabled: true
    bind:
      transport: uds
YAML

cat > "$SUBSTRATE_HOME/policy.yaml" <<'YAML'
id: set5-policy
name: set5-policy
world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
agents:
  allowed_backends:
    - "cli:codex"
    - "cli:claude_code"
    - "cli:codex_world"
YAML

cat > "$SUBSTRATE_HOME/agents/codex.yaml" <<YAML
version: 1
id: codex
config:
  kind: cli
  enabled: true
  protocol: pure_agent
  execution:
    scope: host
  cli:
    runtime_family: codex
    binary: $TMP/fake-codex.sh
    mode: persistent
  capabilities:
    session_start: true
    session_resume: true
    session_fork: true
    session_stop: true
    status_snapshot: true
    event_stream: true
    llm: true
    mcp_client: false
YAML

cat > "$SUBSTRATE_HOME/agents/claude_code.yaml" <<YAML
version: 1
id: claude_code
config:
  kind: cli
  enabled: true
  protocol: pure_agent
  execution:
    scope: host
  cli:
    runtime_family: claude_code
    binary: $TMP/fake-claude.sh
    mode: persistent
  capabilities:
    session_start: true
    session_resume: true
    session_fork: true
    session_stop: true
    status_snapshot: true
    event_stream: true
    llm: true
    mcp_client: false
YAML

cat > "$SUBSTRATE_HOME/agents/codex_world.yaml" <<YAML
version: 1
id: codex_world
config:
  kind: cli
  enabled: true
  protocol: pure_agent
  execution:
    scope: world
  cli:
    runtime_family: codex
    binary: $TMP/fake-codex.sh
    mode: persistent
  capabilities:
    session_start: true
    session_resume: true
    session_fork: true
    session_stop: true
    status_snapshot: true
    event_stream: true
    llm: true
    mcp_client: false
YAML
```

### 6. Step-by-step manual procedure

1. Prove pre-runtime truth: `toolbox status` is readable, `toolbox env` is strict, and UDS is the supported pre-runtime transport.

```bash
"$S" world doctor --json | jq '.'
"$S" agent doctor --json | tee "$TMP/doctor-codex.json"
"$S" agent toolbox status --json | tee "$TMP/toolbox-status-pre.json"
"$S" agent toolbox env --json ; echo "exit=$?"
```

2. Linux-only negative control: change the transport to TCP and prove the surface does not overclaim a live pre-runtime TCP endpoint.

```bash
perl -0pi -e 's/transport: uds/transport: tcp/' "$SUBSTRATE_HOME/config.yaml"
"$S" agent toolbox status --json | tee "$TMP/toolbox-status-tcp.json"
perl -0pi -e 's/transport: tcp/transport: uds/' "$SUBSTRATE_HOME/config.yaml"
```

3. With `codex` selected, prove the first validated host-tool floor.

```bash
"$S" agent start --backend cli:codex --prompt "codex startup" --json | tee "$TMP/start-codex.json"
export CODEX_SESSION="$(jq -r '.orchestration_session_id' "$TMP/start-codex.json")"
"$S" agent turn --session "$CODEX_SESSION" --backend cli:codex --prompt "codex follow-up" --json | tee "$TMP/turn-codex.json"
```

4. Inspect the fake Codex env capture.

```bash
cat "$TMP/fake-codex-1.env"
cat "$TMP/fake-codex-2.env"
```

5. Switch the selected host orchestrator to `claude_code` and prove the same semantic path.

```bash
perl -0pi -e 's/orchestrator_agent_id: codex/orchestrator_agent_id: claude_code/' "$SUBSTRATE_HOME/config.yaml"
"$S" agent doctor --json | tee "$TMP/doctor-claude.json"
"$S" agent start --backend cli:claude_code --prompt "claude startup" --json | tee "$TMP/start-claude.json"
export CLAUDE_SESSION="$(jq -r '.orchestration_session_id' "$TMP/start-claude.json")"
"$S" agent turn --session "$CLAUDE_SESSION" --backend cli:claude_code --prompt "claude follow-up" --json | tee "$TMP/turn-claude.json"
cat "$TMP/fake-claude-1.env"
cat "$TMP/fake-claude-2.env"
```

6. Linux-only: keep the selected host as `claude_code`, then start a world-backed Codex member and inspect the session-bound toolbox projection.

```bash
"$S" agent start --backend cli:codex_world --scope world --prompt "world startup" --json | tee "$TMP/start-world.json"
"$S" agent toolbox status --json | tee "$TMP/toolbox-status-world.json"
"$S" agent toolbox env --json | tee "$TMP/toolbox-env-world.json"
```

### 7. What to verify after each step

1. Step 1: `toolbox status` succeeds before runtime start; `toolbox env` fails closed with exit `3`
2. Step 2: the transport truth changes in status output, but the surface does not pretend a live endpoint exists without a live host session
3. Step 3: the Codex path starts and turns successfully on the selected host path
4. Step 4: the fake Codex env capture contains `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`, `SUBSTRATE_AGENT_TOOLBOX_VERSION`, and `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT_BOUND=1`
5. Step 5: the selected-host Claude path also receives the same runtime-owned env hints and succeeds on the same `start` and `turn` semantics
6. Step 6: when the live session is world-backed, `toolbox status` projects the same session-rooted endpoint plus authoritative `world_id` and `world_generation`

### 8. Pass/fail criteria

Pass if:

1. `toolbox status` and `toolbox env` tell the truth both before and after runtime start,
2. Codex and selected-host `claude_code` receive the same runtime-owned toolbox env contract,
3. world-backed sessions project authoritative binding proof onto the same toolbox surface,
4. the surface remains session-scoped and runtime-owned rather than model-invented.

Fail if:

1. `toolbox env` succeeds pre-runtime,
2. either runtime family launches without the injected endpoint/version,
3. the fake wrappers show an unbound or missing UDS endpoint during a live host session,
4. world binding is guessed or omitted when the live session actually has authoritative proof.

### 9. Cleanup

1. stop any sessions created during the set:

```bash
"$S" agent status --json
```

2. remove the disposable harness:

```bash
rm -rf "$TMP"
```

### 10. Notes / intentional fail-closed cases

1. The frozen contract is seven internal host tools. This set does not widen into public `substrate agent toolbox <verb>` execution.
2. The endpoint is runtime-owned and session-bound. The model is not supposed to invent it.
3. On Windows/WSL, world-backed host-tool parity is not claimed. Fail-closed behavior is acceptable and expected.

---

## Smoke Set 6: Platform Parity and Fail-Closed Boundaries

### 1. What this set proves

This set proves the platform truth after the landed slices: Linux is the source-of-truth platform, macOS/Lima is the supported forwarded parity path, and Windows/WSL remains a fail-closed boundary for support not yet claimed.

### 2. Landed seams/slices covered

Primary slices and related platform hardening:

1. Linux-first world and replay path in the core docs and smokes
2. macOS/Lima forwarded parity scripts and orchestration smokes
3. current Windows/WSL warm and smoke scripts, with explicit fail-closed expectations

### 3. Platform scope

1. Linux: required full pass and source-of-truth
2. macOS/Lima: supported forwarded parity path
3. Windows host and WSL guest: fail-closed boundary where support is not claimed for all surfaces touched by Sets 1 through 5

### 4. Preconditions

1. you are running on the platform you intend to validate
2. `cargo build -p substrate --bin substrate` and `cargo build -p world-service` complete
3. on Linux, privileged world prerequisites are satisfied
4. on macOS, Lima and Virtualization.framework prerequisites are satisfied
5. on Windows, PowerShell 7 and WSL are installed before attempting warm scripts

### 5. Setup

```bash
cd /home/azureuser/__Active_Code/atomize-hq/substrate
cargo build -p substrate --bin substrate
cargo build -p world-service
export SUBSTRATE_BIN="$PWD/target/debug/substrate"
```

### 6. Step-by-step manual procedure

1. Linux: prove the canonical world socket path and world execution baseline.

```bash
scripts/linux/world-socket-verify.sh --profile release --skip-cleanup
```

2. Linux: prove execution plus replay on the source-of-truth platform.

```bash
"$SUBSTRATE_BIN" -c 'printf set6-linux > smoke-set6-linux.txt'
tail -n 20 ~/.substrate/trace.jsonl
```

Take the exact span id from the latest trace line, then replay it:

```bash
"$SUBSTRATE_BIN" --replay --replay-verbose <EXACT_SPAN_ID>
```

3. macOS: warm Lima and prove the supported forwarded parity path.

```bash
scripts/mac/lima-warm.sh
scripts/mac/lima-doctor.sh
"$SUBSTRATE_BIN" world doctor --json
"$SUBSTRATE_BIN" agent doctor --json
```

4. macOS: run the shipped smoke scripts.

```bash
scripts/mac/smoke.sh
scripts/mac/orchestration-smoke.sh
```

5. Windows host: warm WSL and capture the explicit boundary.

```powershell
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
$LASTEXITCODE
wsl -l -v
```

6. Windows host: run the current WSL smoke and inspect the local app-data footprint.

```powershell
pwsh -File scripts/windows/wsl-smoke.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
$LASTEXITCODE
Get-ChildItem "$env:LOCALAPPDATA\Substrate" -Force
```

7. Optional WSL guest negative control: prove guest-side provisioning does not overclaim unsupported parity.

```bash
scripts/wsl/provision.sh
echo $?
```

### 7. What to verify after each step

1. Step 1: Linux verification confirms the canonical world socket expectations and does not report a broken source-of-truth path
2. Step 2: the command writes the file, emits a replayable trace span, and replay succeeds through the world-backed path
3. Step 3: Lima warms successfully and both doctor surfaces show a realizable forwarded world path
4. Step 4: the shipped macOS smokes complete without silently dropping the forwarded parity claim
5. Step 5: Windows warm either succeeds on the narrow currently supported path or fails explicitly; it must not overclaim parity with Linux
6. Step 6: the Windows smoke outcome matches the current documented boundary and leaves inspectable local state
7. Step 7: guest-side provisioning does not silently expand support claims beyond what the docs state

### 8. Pass/fail criteria

Pass if:

1. Linux remains the cleanest, full-fidelity execution and replay path,
2. macOS/Lima proves the supported forwarded parity path with the shipped scripts,
3. Windows/WSL behavior is explicit and fail-closed where support is not claimed.

Fail if:

1. Linux cannot run or replay the source-of-truth path,
2. macOS/Lima scripts succeed only by silently skipping critical checks,
3. Windows/WSL appears to succeed while violating the documented fail-closed boundary.

### 9. Cleanup

1. Linux:

```bash
rm -f smoke-set6-linux.txt
```

2. macOS:

```bash
scripts/mac/lima-stop.sh
```

3. Windows/WSL:

No extra cleanup is required unless you explicitly want to remove the warmed distro or cached local state.

### 10. Notes / intentional fail-closed cases

1. This set is supposed to make the support boundary explicit, not aspirational.
2. Linux is the source-of-truth platform for the slices covered by this package.
3. macOS support is the Lima-forwarded parity path, not a native equivalent of every Linux-only world seam.
4. Windows and WSL should be treated as fail-closed when the repo docs do not explicitly claim support for a given world-backed or orchestration path.
