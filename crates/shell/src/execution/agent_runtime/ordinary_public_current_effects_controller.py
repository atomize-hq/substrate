# Inserted only in the owned public composition fixture's synthetic native CLI.
# root, turn, session and stage come from HOST_NATIVE_FIXTURE. The real helper
# publishes this endpoint and supplies all session/caller/World authority.
import socket

for early_event in (
    dict(type="thread.started" if turn == 1 else "thread.resumed", thread_id=session),
    dict(type="turn.started", thread_id=session, turn_id="host-turn-%s" % turn),
):
    print(json.dumps(early_event), flush=True)

if turn == 1:
    # Synchronize with the actual owner publication triggered by those SDK
    # events. Reading this marker supplies no dispatch permission.
    registration_deadline = time.monotonic() + 30
    while True:
        hsa = json.loads(pathlib.Path(
            "/home/fixture/.substrate/authority-v1/state-root-v1.json"
        ).read_text())
        transactions = list(hsa.get("start_transaction_map", {}).values())
        if len(transactions) == 1 and transactions[0]["state"]["kind"] == "ContinuationRegistered":
            break
        assert time.monotonic() < registration_deadline, "actual Start registration not published"
        time.sleep(0.01)


def invoke_world_tool(tool_name, arguments, call_id):
    endpoint = os.environ["SUBSTRATE_AGENT_TOOLBOX_ENDPOINT"]
    assert endpoint.startswith("unix://")
    request = dict(version=1, tool_name=tool_name,
                   tool_call_id=call_id, arguments=arguments)
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        # Retained operations perform fresh Root admission checks before startup.
        # Their request budget stays below the enclosing 300s command/540s run caps.
        client.settimeout(240 if tool_name in ("fork_world_worker", "continue_world_worker") else 120)
        client.connect(endpoint[len("unix://"):])
        client.sendall((json.dumps(request, separators=(",", ":")) + "\n").encode())
        with client.makefile("r", encoding="utf-8") as stream:
            for line in stream:
                frame = json.loads(line)
                with (root / "public-world-tool-frames.jsonl").open("a") as output:
                    output.write(json.dumps(dict(turn=turn, tool_name=tool_name, frame=frame)) + "\n")
                if frame.get("frame_kind") == "event":
                    continue
                assert frame.get("ok") is True, "actual World toolbox refusal: %r" % frame
                outcome = frame.get("outcome")
                assert isinstance(outcome, dict), "actual toolbox outcome missing"
                assert "pending_admission" not in outcome and "startup_failure" not in outcome, (
                    "actual World launch did not produce a successful receipt: %r" % outcome)
                return outcome
    raise AssertionError("actual toolbox closed without terminal response")


# Every receipt below is returned by the actual configured owner/tool transport.
assert turn == 1
current_effects_deadline = time.monotonic() + 470

def write_effects_receipt(name, value):
    path = root / name
    with path.open("x") as output:
        json.dump(value, output)
    path.chmod(0o600)
    (root / (name + ".ready")).touch(mode=0o600, exist_ok=False)

def await_effects_actor(name):
    while not (root / name).exists():
        assert time.monotonic() < current_effects_deadline, name
        time.sleep(0.01)

stage("world_tool_begin")
source = invoke_world_tool("spawn_world_worker", dict(
    target_backend_id="cli:codex-world",
    payload=dict(prompt="task20 current effects source turn 1")), "task20-current-effects-spawn")
write_effects_receipt("public-world-source.json", source)
participant = source.get("participant_id") or source.get("target_participant_id")
assert participant
completed = invoke_world_tool("continue_world_worker", dict(
    participant_id=participant,
    payload=dict(prompt="task20 current effects turn 2")), "task20-current-effects-completed")
write_effects_receipt("current-effects-completed-receipt.json", completed)
await_effects_actor("current-effects-restored")
active = invoke_world_tool("continue_world_worker", dict(
    participant_id=participant,
    payload=dict(prompt="task20 current effects turn 3")), "task20-current-effects-active")
write_effects_receipt("current-effects-active-receipt.json", active)
await_effects_actor("current-effects-cancel-ready")
# These exact identities came from this controller's actual accepted receipt.
# The existing owning helper keeps its observer lease and performs normal Root
# delivery; the test actor grants no cancellation authority.
exact_target = dict(target_kind="accepted_retained_turn",
                    acceptance_record_id=active["acceptance_record_id"],
                    active_run_id=active["active_run_id"],
                    message_id=active["message_id"],
                    target_participant_id=active["target_participant_id"])
cancelled = invoke_world_tool("cancel_world_work", dict(
    exact_target=exact_target, payload={}), "task20-current-effects-cancel")
write_effects_receipt("current-effects-cancel-receipt.json", cancelled)
await_effects_actor("current-effects-finished")
stage("world_tool_returned")
