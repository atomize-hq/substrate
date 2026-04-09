#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


TOOL_ITEM_TYPES = {
    "command_execution",
    "function_call",
    "mcp_tool_call",
    "tool_call",
}
TOOL_FAILURE_STATUSES = {"cancelled", "error", "errored", "failed"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Summarize Codex --json events into a deterministic run_state.json."
    )
    parser.add_argument("--phase", required=True)
    parser.add_argument("--agent", required=True)
    parser.add_argument("--events-path", required=True)
    parser.add_argument("--last-message-run-path", required=True)
    parser.add_argument("--exit-code", required=True, type=int)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def item_has_tool_error(item: object) -> bool:
    if not isinstance(item, dict):
        return False
    if item.get("type") not in TOOL_ITEM_TYPES:
        return False
    status = item.get("status")
    if isinstance(status, str) and status.lower() in TOOL_FAILURE_STATUSES:
        return True
    exit_code = item.get("exit_code")
    if isinstance(exit_code, int) and exit_code != 0:
        return True
    error = item.get("error")
    return bool(error)


def build_run_state(args: argparse.Namespace) -> dict[str, object]:
    events_path = Path(args.events_path)

    thread_id = None
    turn_completed = False
    assistant_message_present = False
    tool_error_count = 0

    if events_path.is_file():
        with events_path.open(encoding="utf-8", errors="replace") as handle:
            for raw_line in handle:
                line = raw_line.strip()
                if not line:
                    continue
                try:
                    event = json.loads(line)
                except json.JSONDecodeError:
                    continue

                event_type = event.get("type")
                if event_type == "thread.started" and thread_id is None:
                    value = event.get("thread_id")
                    if isinstance(value, str) and value:
                        thread_id = value
                elif event_type == "turn.completed":
                    turn_completed = True

                item = event.get("item")
                if isinstance(item, dict):
                    if event_type == "item.completed" and item.get("type") == "agent_message":
                        assistant_message_present = True
                    if item_has_tool_error(item):
                        tool_error_count += 1

    return {
        "phase": args.phase,
        "agent": args.agent,
        "exit_code": args.exit_code,
        "turn_completed": turn_completed,
        "assistant_message_present": assistant_message_present,
        "tool_error_count": tool_error_count,
        "thread_id": thread_id,
        "events_path": args.events_path,
        "last_message_run_path": args.last_message_run_path,
    }


def main() -> int:
    args = parse_args()
    run_state = build_run_state(args)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(run_state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
