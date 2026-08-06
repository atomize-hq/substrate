#!/usr/bin/env python3
"""Independently verify a local-source evidence receipt."""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path

from protocol_json import load_json


def fail(message: str) -> None:
    raise SystemExit(message)


def run_git(worktree: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(worktree), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        fail(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("receipt", type=Path)
    args = parser.parse_args()

    validator = Path(__file__).with_name("validate_evidence_receipt.py")
    structural = subprocess.run(
        [sys.executable, str(validator), str(args.receipt)],
        check=False,
        capture_output=True,
        text=True,
    )
    if structural.returncode != 0:
        fail(structural.stderr.strip() or structural.stdout.strip())
    try:
        data = load_json(args.receipt)
    except ValueError as exc:
        fail(f"invalid evidence receipt JSON: {exc}")
    if data.get("status") != "EVIDENCE_CLEAN":
        fail("local evidence verification requires EVIDENCE_CLEAN")
    source = data["source"]
    if source.get("publication_mode", "remote") != "local":
        fail("local evidence verification requires a local source")

    worktree = Path(source["worktree"]).resolve()
    if not worktree.is_dir():
        fail("source.worktree does not exist")
    if Path(run_git(worktree, "rev-parse", "--show-toplevel")).resolve() != worktree:
        fail("source.worktree must equal the Git worktree root")
    if run_git(worktree, "symbolic-ref", "-q", "HEAD") != source["target_ref"]:
        fail("live branch ref does not match source.target_ref")
    if run_git(worktree, "rev-parse", source["target_ref"]) != source["commit"]:
        fail("live local branch ref does not equal source.commit")
    if source["local_ref"] != source["commit"]:
        fail("source.local_ref does not equal source.commit")
    if run_git(worktree, "rev-parse", f"{source['commit']}^{{tree}}") != source["tree"]:
        fail("source tree does not match repository truth")
    if run_git(worktree, "status", "--porcelain=v1", "--untracked-files=all"):
        fail("local evidence source worktree is not clean")
    if Path(data["environment"]["project_path"]).resolve() != worktree:
        fail("environment.project_path must equal the local source worktree")

    artifact = Path(data["evidence"]["artifact_path"])
    if not artifact.is_file():
        fail("evidence artifact does not exist")
    digest = "sha256:" + hashlib.sha256(artifact.read_bytes()).hexdigest()
    if digest != data["evidence"]["artifact_sha256"]:
        fail("evidence artifact digest does not match")

    print(
        f"VERIFIED local evidence {data['evidence_id']} "
        f"commit={source['commit']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
