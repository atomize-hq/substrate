#!/usr/bin/env python3
"""Stress-test Substrate REPL streaming throughput.

Usage:
    python scripts/dev/async_repl_stress.py [--agents N] [--events M] [--delay-ms D]

The helper launches an interactive REPL (via ~/.substrate/dev-shim-env.sh),
verifies Ctrl+C interrupts a chatty command, and then runs `:demo-burst` to
measure aggregate event throughput.
"""
from __future__ import annotations

import argparse
import time
import sys

try:
    import pexpect  # type: ignore
except ImportError as exc:  # pragma: no cover - guidance for developers
    raise SystemExit(
        "pexpect is required for this helper. Install with `pip install pexpect`."
    ) from exc


PROMPT = "substrate> "


def expect_prompt(child: "pexpect.spawn") -> None:
    while True:
        idx = child.expect([PROMPT, "\\x1b\\[6n", pexpect.EOF], timeout=60)
        if idx == 0:
            return
        if idx == 1:
            child.send("\x1b[1;1R")
            continue
        raise RuntimeError("substrate exited before prompt was ready")


def spawn_substrate() -> "pexpect.spawn":
    cmd = "source ~/.substrate/dev-shim-env.sh && target/debug/substrate --no-world"
    child = pexpect.spawn("bash", ["-lc", cmd], encoding="utf-8", timeout=60)
    expect_prompt(child)
    return child


def run_ctrl_c_check(child: "pexpect.spawn") -> None:
    noisy = (
        "python -c \"import sys,time\n"
        "i = 0\n"
        "try:\n"
        "    while True:\n"
        "        sys.stdout.write(f'noise {i}\\n')\n"
        "        sys.stdout.flush()\n"
        "        i += 1\n"
        "        time.sleep(0.001)\n"
        "except KeyboardInterrupt:\n"
        "    pass\""
    )
    child.sendline(noisy)
    time.sleep(0.8)
    child.send("\x03")  # Ctrl+C
    expect_prompt(child)
    print("Ctrl+C interruption: ok")


def run_demo_burst(child: "pexpect.spawn", agents: int, events: int, delay_ms: int) -> float:
    cmd = f":demo-burst {agents} {events} {delay_ms}"
    child.sendline(cmd)
    start = time.perf_counter()
    expect_prompt(child)
    end = time.perf_counter()
    return end - start


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--agents", type=int, default=4)
    parser.add_argument("--events", type=int, default=400)
    parser.add_argument("--delay-ms", type=int, default=0)
    args = parser.parse_args()

    child = spawn_substrate()
    try:
        run_ctrl_c_check(child)
        duration = run_demo_burst(child, args.agents, args.events, args.delay_ms)
        total_events = args.agents * args.events
        rate = total_events / duration if duration else float("inf")
        print(
            f"Burst completed: agents={args.agents} events={args.events} delay_ms={args.delay_ms} "
            f"duration={duration:.3f}s rate={rate:.1f} ev/s"
        )
        child.sendline("exit")
        child.expect(pexpect.EOF)
    finally:
        child.close()


if __name__ == "__main__":
    main()
