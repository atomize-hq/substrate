#!/usr/bin/env python3
"""Independently verify a CLOSED_CLEAN receipt against local Git truth."""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path

from protocol_json import load_json


def fail(message: str) -> None:
    raise SystemExit(message)


def run_git(worktree: Path, *args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", "-C", str(worktree), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if check and result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        fail(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip()


def commit_parents(worktree: Path, commit: str) -> list[str]:
    fields = run_git(worktree, "rev-list", "--parents", "-n", "1", commit).split()
    if not fields or fields[0] != commit:
        fail(f"unable to resolve commit topology for {commit}")
    return fields[1:]


def changed_files(worktree: Path, old: str, new: str) -> list[str]:
    output = run_git(worktree, "diff", "--name-only", f"{old}..{new}")
    return sorted(line for line in output.splitlines() if line)


def remote_oid(worktree: Path, remote: str, target_ref: str) -> str | None:
    result = subprocess.run(
        ["git", "-C", str(worktree), "ls-remote", "--refs", remote, target_ref],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        fail(f"unable to query remote {remote} {target_ref}: {detail}")
    lines = [line for line in result.stdout.splitlines() if line.strip()]
    if not lines:
        return None
    if len(lines) != 1:
        fail("remote query returned more than one exact ref")
    oid, ref = lines[0].split("\t", 1)
    if ref != target_ref:
        fail("remote query returned an unexpected ref")
    return oid


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("receipt", type=Path)
    parser.add_argument("--state", required=True, type=Path)
    parser.add_argument("--expected-next", required=True)
    args = parser.parse_args()

    validator = Path(__file__).with_name("validate_receipt.py")
    structural = subprocess.run(
        [
            sys.executable,
            str(validator),
            str(args.receipt),
            "--expected-next",
            args.expected_next,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if structural.returncode != 0:
        fail(structural.stderr.strip() or structural.stdout.strip())

    try:
        data = load_json(args.receipt)
    except ValueError as exc:
        fail(f"invalid receipt JSON: {exc}")
    if data.get("status") != "CLOSED_CLEAN":
        fail("local verification requires CLOSED_CLEAN")
    state_validator = Path(__file__).with_name("validate_state.py")
    state_check = subprocess.run(
        [sys.executable, str(state_validator), str(args.state)],
        check=False,
        capture_output=True,
        text=True,
    )
    if state_check.returncode != 0:
        fail(state_check.stderr.strip() or state_check.stdout.strip())
    try:
        state = load_json(args.state)
    except ValueError as exc:
        fail(f"invalid state JSON: {exc}")
    if state.get("publication_mode", "remote") != "local":
        fail("CLOSED_CLEAN requires local publication state")
    if data["expected_base"] != state["expected_base"]:
        fail("receipt expected_base does not match durable state")

    closed = data["closed"]
    worktree = Path(closed["worktree"]).resolve()
    if not worktree.is_dir():
        fail("closed.worktree does not exist")
    if Path(run_git(worktree, "rev-parse", "--show-toplevel")).resolve() != worktree:
        fail("closed.worktree must equal the Git worktree root")

    branch_ref = run_git(worktree, "symbolic-ref", "-q", "HEAD")
    if branch_ref != closed["branch_ref"]:
        fail("live branch ref does not match closed.branch_ref")
    if run_git(worktree, "rev-parse", closed["branch_ref"]) != closed["control_commit"]:
        fail("live local branch ref does not equal closed.control_commit")
    if closed["local_ref"] != closed["control_commit"]:
        fail("closed.local_ref does not equal closed.control_commit")
    if run_git(worktree, "status", "--porcelain=v1", "--untracked-files=all"):
        fail("closed worktree is not clean")

    base = data["expected_base"]
    if run_git(worktree, "rev-parse", f"{base['commit']}^{{tree}}") != base["tree"]:
        fail("expected base tree does not match repository truth")
    if commit_parents(worktree, closed["work_commit"]) != [base["commit"]]:
        fail("work commit must have exactly the expected base as its sole parent")
    if commit_parents(worktree, closed["control_commit"]) != [closed["work_commit"]]:
        fail("control commit must have exactly the work commit as its sole parent")
    if run_git(worktree, "rev-parse", f"{closed['work_commit']}^{{tree}}") != closed["work_tree"]:
        fail("work tree does not match repository truth")
    if (
        run_git(worktree, "rev-parse", f"{closed['control_commit']}^{{tree}}")
        != closed["control_tree"]
    ):
        fail("control tree does not match repository truth")

    actual_work = changed_files(worktree, base["commit"], closed["work_commit"])
    actual_control = changed_files(
        worktree,
        closed["work_commit"],
        closed["control_commit"],
    )
    actual_union = changed_files(worktree, base["commit"], closed["control_commit"])
    if actual_work != closed["work_changed_files"]:
        fail("work changed-file inventory does not match repository truth")
    if actual_control != closed["control_changed_files"]:
        fail("control changed-file inventory does not match repository truth")
    if actual_union != data["changed_files"]:
        fail("combined changed-file inventory does not match repository truth")

    review_path = Path(data["review"]["record_path"])
    if not review_path.is_file():
        fail("review record does not exist")
    digest = "sha256:" + hashlib.sha256(review_path.read_bytes()).hexdigest()
    if digest != data["review"]["record_sha256"]:
        fail("review record digest does not match")

    observation = data["publication"]["remote_observation"]
    state_observation = state["remote_observation"]
    if (
        observation["name"] != state_observation["name"]
        or observation["target_ref"] != state_observation["target_ref"]
        or observation["before"] != state_observation["before"]
    ):
        fail("receipt remote-before observation does not match durable state")
    remotes = set(run_git(worktree, "remote").splitlines())
    remote_name = observation["name"]
    if remote_name is None:
        if remotes:
            fail("null remote observation is invalid when a remote is configured")
    else:
        if remote_name not in remotes:
            fail("observed remote is not configured in the repository")
        if remote_oid(worktree, remote_name, observation["target_ref"]) != observation["after"]:
            fail("live remote does not match the unchanged remote observation")

    print(
        "VERIFIED local closeout "
        f"{data['increment']} work={closed['work_commit']} "
        f"control={closed['control_commit']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
