#!/usr/bin/env python3
"""Collect idle CPU metrics for the async REPL."""
from __future__ import annotations

import argparse
import subprocess
import time
from pathlib import Path

import pexpect  # type: ignore

PROMPT = "substrate> "


def expect_prompt(child: "pexpect.spawn") -> None:
    while True:
        idx = child.expect([PROMPT, r"\x1b\[6n", pexpect.EOF], timeout=60)
        if idx == 0:
            return
        if idx == 1:
            child.send("\x1b[1;1R")
            continue
        raise RuntimeError("substrate exited before prompt was ready")


def capture_top(pid: int, dest: Path, samples: int) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    with dest.open("w") as fh:
        subprocess.run(
            ["/usr/bin/top", "-b", "-d", "1", "-n", str(samples), "-p", str(pid)],
            stdout=fh,
            check=True,
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("docs/project_management/now/stage4_idle_top_linux.txt"),
        help="File to write top output",
    )
    parser.add_argument("--samples", type=int, default=5)
    args = parser.parse_args()

    cmd = "source ~/.substrate/dev-shim-env.sh && target/debug/substrate --async-repl --no-world"
    child = pexpect.spawn(
        "bash",
        ["-lc", cmd],
        encoding="utf-8",
        timeout=60,
    )
    try:
        expect_prompt(child)
        pid = child.pid
        capture_top(pid, args.output, args.samples)
        child.sendline("")
        child.sendline("exit")
    finally:
        child.close(force=True)


if __name__ == "__main__":
    main()
