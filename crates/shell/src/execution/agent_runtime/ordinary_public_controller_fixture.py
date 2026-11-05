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


def invoke_world_tool(tool_name, arguments):
    endpoint = os.environ["SUBSTRATE_AGENT_TOOLBOX_ENDPOINT"]
    assert endpoint.startswith("unix://")
    request = dict(version=1, tool_name=tool_name,
                   tool_call_id="task20-public-world-%s" % turn, arguments=arguments)
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


stage("world_tool_begin")
if turn == 1:
    tool_name = "spawn_world_worker"
    outcome = invoke_world_tool(tool_name, dict(
        target_backend_id="cli:codex-world", payload=dict(prompt="task20 native World source turn 1")))
    handle_file = root / "public-world-source.json"
elif turn == 2:
    source = json.loads((root / "public-world-source.json").read_text())
    source_id = source.get("participant_id") or source.get("target_participant_id")
    assert source_id
    tool_name = "fork_world_worker"
    outcome = invoke_world_tool(tool_name, dict(
        participant_id=source_id, payload=dict(prompt="task20 native World child turn 1")))
    handle_file = root / "public-world-child.json"
else:
    assert turn == 3
    child = json.loads((root / "public-world-child.json").read_text())
    child_id = child.get("participant_id") or child.get("target_participant_id")
    assert child_id
    tool_name = "continue_world_worker"
    outcome = invoke_world_tool(tool_name, dict(
        participant_id=child_id, payload=dict(prompt="task20 native World child turn 2")))
    handle_file = root / "public-world-child-resume.json"
with handle_file.open("x") as output:
    json.dump(outcome, output)
if turn == 3:
    (root / "public-world-child-receipt-ready").touch(mode=0o600, exist_ok=False)
    # A foreground receipt is not the World terminal. Keep the genuine controller
    # and its observer alive until the actor joins the exact durable B2 Exit.
    terminal_deadline = time.monotonic() + 120
    while not (root / "public-world-child-terminal-joined").exists():
        assert time.monotonic() < terminal_deadline, "World terminal not joined"
        time.sleep(0.01)
stage("world_tool_returned")
