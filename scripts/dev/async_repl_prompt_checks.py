#!/usr/bin/env python3
"""Automated prompt integrity drills for the async REPL.

This helper reproduces the Stage 4 manual scenarios:
1. Typing while background agent events stream.
2. Editing a long in-flight command under bursty agent output.
3. Using control characters (backspace) during agent bursts.

The script captures the full transcript so we can attach it to
Stage 4 documentation. Run from the repo root:

    python scripts/dev/async_repl_prompt_checks.py \
        --log docs/project_management/now/stage4_prompt_checks_transcript.txt

"""
from __future__ import annotations

import argparse
import io
import sys
import time
from pathlib import Path

try:
    import pexpect  # type: ignore
except ImportError as exc:  # pragma: no cover - developer guidance
    raise SystemExit(
        "pexpect is required. Install with `pip install pexpect`."
    ) from exc


PROMPT = "substrate> "


def expect_prompt(child: "pexpect.spawn", *, timeout: int = 30) -> None:
    """Wait for the REPL prompt, answering cursor-position queries if needed."""

    while True:
        idx = child.expect([PROMPT, "\x1b\[6n", pexpect.TIMEOUT], timeout=timeout)
        if idx == 0:
            return
        if idx == 1:
            child.send("\x1b[1;1R")
            continue
        raise RuntimeError("Timed out waiting for substrate prompt")


def spawn_async_shell() -> "pexpect.spawn":
    cmd = (
        "source ~/.substrate/dev-shim-env.sh && "
        "target/debug/substrate --async-repl --no-world"
    )
    child = pexpect.spawn("bash", ["-lc", cmd], encoding="utf-8", timeout=60)
    expect_prompt(child)
    return child


def _send_command(child: "pexpect.spawn", command: str) -> None:
    child.send(command)
    child.send("\r")


def scenario_typing_mid_stream(child: "pexpect.spawn") -> None:
    child.send(":demo-agent\r")
    expect_prompt(child)

    command = "echo typing while agents stream"
    for ch in command:
        child.send(ch)
        time.sleep(0.04)

    for marker in ["\\[demo\\] Demo agent event #1", "\\[demo\\] Demo agent event #2", "\\[demo\\] Demo agent event #3"]:
        child.expect(marker)

    child.send("\r")
    child.expect("typing while agents stream\r\n")
    expect_prompt(child)


def scenario_long_line_burst(child: "pexpect.spawn") -> None:
    # Schedule three demo agents back-to-back for overlapping output.
    for _ in range(3):
        child.send(":demo-agent\r")
        expect_prompt(child)

    payload = "X" * 96
    command = f"echo {payload}"
    for ch in command:
        child.send(ch)
        time.sleep(0.02)

    # Expect nine progress events (three agents * three events each).
    for _ in range(9):
        child.expect("Demo agent event #", timeout=5)

    child.send("\r")
    child.expect(f"{payload}\r\n")
    expect_prompt(child)


def scenario_control_chars(child: "pexpect.spawn") -> None:
    child.send(":demo-agent\r")
    expect_prompt(child)

    command = "echo control-case"
    for ch in command:
        child.send(ch)
        time.sleep(0.03)

    child.send("\x7f" * 5)  # remove '-case'
    child.send("check")

    for marker in ["Demo agent event #1", "Demo agent event #2", "Demo agent event #3"]:
        child.expect(marker)

    child.send("\r")
    child.expect("controlcheck\r\n")
    expect_prompt(child)


def run_drills(log_path: Path) -> None:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w", encoding="utf-8") as log_file:
        log_file.write("# Stage 4 Prompt Integrity Drills\n")
        log_file.flush()

        child = spawn_async_shell()
        child.logfile = log_file
        try:
            log_file.write("\n## Scenario 1: typing while events stream\n")
            log_file.flush()
            scenario_typing_mid_stream(child)

            log_file.write("\n## Scenario 2: long command under bursty output\n")
            log_file.flush()
            scenario_long_line_burst(child)

            log_file.write("\n## Scenario 3: backspace edits during events\n")
            log_file.flush()
            scenario_control_chars(child)

            _send_command(child, "exit")
            child.expect(pexpect.EOF)
        finally:
            child.close()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--log",
        type=Path,
        default=Path("docs/project_management/now/stage4_prompt_checks_transcript.txt"),
        help="Path to write the captured transcript.",
    )
    args = parser.parse_args(argv)

    run_drills(args.log)
    print(f"Prompt drill transcript written to {args.log}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
