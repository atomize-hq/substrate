#!/usr/bin/python3
"""Synthetic native protocol in the private normal-service composition proof.

All authority, admission, SDK lifecycle and terminal publication remain real
product paths. This fixture makes only malformed local gateway requests.
"""
import hashlib
import json
import os
import pathlib
import re
import sys
import urllib.error
import urllib.request
import uuid
import time

args = sys.argv[1:]
if args and args[0] in ("--version", "--help", "features"):
    print({"--version": "codex 1.2.3", "--help": "Usage: codex --add-dir",
           "features": '{"features":["add_dir"]}'}[args[0]])
    raise SystemExit(0)

root = pathlib.Path("/home/fixture/workspace/world-observations")
home = pathlib.Path(os.environ["CODEX_HOME"])
assert str(home).startswith("/proc/self/fd/")
assert not os.environ.get("SUBSTRATE_ORDINARY_HOME_HANDOFF")
assert not (home / "auth.json").exists()
for key in ("OPENAI_API_KEY", "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID"):
    assert not os.environ.get(key), key
config = (home / "config.toml").read_text()
assert 'model = "gpt-6-astra"' in config
assert 'model_provider = "substrate-managed-gateway"' in config
origin = re.search(r'base_url\s*=\s*"(http://127[.]0[.]0[.]1:[0-9]+)', config).group(1)
assert "requires_openai_auth = false" in config


def normal_gateway_refusal():
    headers = {"Content-Type": "application/json"}
    request = urllib.request.Request(origin + "/v1/responses", data=b"{", headers=headers)
    try:
        urllib.request.urlopen(request, timeout=5)
    except urllib.error.HTTPError as error:
        return error.code
    raise AssertionError("malformed native gateway request was accepted")


assert normal_gateway_refusal() == 400
cgroup_target = pathlib.Path(os.environ["SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH"])
cgroup_members_raw = cgroup_target.read_text()
cgroup_self = pathlib.Path("/proc/self/cgroup").read_text()
cgroup_info = cgroup_target.parent.stat()
cgroup_procs_info = cgroup_target.stat()
cgroup_observation = dict(pid=os.getpid(), target=str(cgroup_target.resolve()),
                          members_raw=cgroup_members_raw, self_cgroup=cgroup_self,
                          device=cgroup_info.st_dev, inode=cgroup_info.st_ino,
                          procs_device=cgroup_procs_info.st_dev,
                          procs_inode=cgroup_procs_info.st_ino)
with (root / "world-cgroup.jsonl").open("a") as output:
    output.write(json.dumps(cgroup_observation) + "\n")
lines = cgroup_members_raw.splitlines()
assert lines and all(line.isascii() and line.isdecimal() for line in lines), cgroup_observation
cgroup_members = sorted({int(line) for line in lines})
assert os.getpid() in cgroup_members, cgroup_observation
history = home / "public-world-source-history"


def record_native_turn(kind, thread_id, turns, state_file, **extra):
    info = state_file.stat()
    entry = dict(kind=kind, pid=os.getpid(), parent_pid=os.getppid(), uid=os.getuid(),
                 thread_id=thread_id, turns=turns, home=str(home.resolve()),
                 home_device=home.stat().st_dev, home_inode=home.stat().st_ino,
                 state_device=info.st_dev, state_inode=info.st_ino,
                 state_sha256=hashlib.sha256(state_file.read_bytes()).hexdigest(),
                 config_sha256=hashlib.sha256(config.encode()).hexdigest(),
                 cgroup_target=str(cgroup_target.resolve()),
                 cgroup_members=cgroup_members, cgroup_self=cgroup_self,
                 cgroup_device=cgroup_info.st_dev, cgroup_inode=cgroup_info.st_ino,
                 cgroup_procs_device=cgroup_procs_info.st_dev,
                 cgroup_procs_inode=cgroup_procs_info.st_ino,
                 executable_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
                 malformed_body_refusal=True, **extra)
    log = root / "world-native.jsonl"
    with log.open("a") as output:
        output.write(json.dumps(entry) + "\n")
    log.chmod(0o644)


# The owned proof vehicle stages this synthetic protocol only in its private namespace.
prompt = sys.stdin.read()
if "resume" not in args:
    assert "task20 current effects source turn 1" in prompt
    assert not history.exists()
    state = dict(thread_id="thread-task20-current-effects", turns=1)
    with history.open("x") as output:
        json.dump(state, output)
    kind = "start"
else:
    thread_id = args[args.index("resume") + 1]
    state = json.loads(history.read_text())
    assert thread_id == state["thread_id"] == "thread-task20-current-effects"
    assert state["turns"] in (1, 2)
    original_inode = history.stat().st_ino
    state["turns"] += 1
    assert ("task20 current effects turn %s" % state["turns"]) in prompt
    history.write_text(json.dumps(state))
    assert history.stat().st_ino == original_inode
    kind = "resume_completed" if state["turns"] == 2 else "resume_active"
assert "--last" not in args and "--fork-session" not in args
record_native_turn(kind, state["thread_id"], state["turns"], history)
for event in (
    dict(type="thread.started" if kind == "start" else "thread.resumed", thread_id=state["thread_id"]),
    dict(type="turn.started", thread_id=state["thread_id"], turn_id=kind),
):
    print(json.dumps(event), flush=True)
if kind == "resume_active":
    # This process has no release or natural-completion path. The actual SDK
    # owner must kill it; SIGKILL cannot provide a signal-handler marker.
    while True:
        time.sleep(1)
output_path = pathlib.Path(args[args.index("--output-last-message") + 1])
assert output_path.is_absolute() and output_path.parent == pathlib.Path(os.environ.get("TMPDIR", "/tmp"))
with output_path.open("x") as output:
    output.write("task20 current effects " + kind)
print(json.dumps(dict(type="turn.completed", thread_id=state["thread_id"], turn_id=kind)), flush=True)
