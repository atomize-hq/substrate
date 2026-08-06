#!/usr/bin/env python3
"""Install this skill into the global Codex skill root as a stable symlink."""

from __future__ import annotations

import argparse
import os
from pathlib import Path


SKILL_NAME = "orchestrate-top-level-tasks"
REQUIRED_PATHS = (
    "SKILL.md",
    "references/protocol.md",
    "assets/meta-orchestrator-prompt-template.md",
    "assets/increment-orchestrator-prompt-template.md",
    "assets/evidence-task-prompt-template.md",
    "scripts/validate_receipt.py",
    "scripts/validate_state.py",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--codex-home",
        type=Path,
        help="Codex home containing skills/ (defaults to CODEX_HOME or ~/.codex)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verify the global installation without changing it",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    raise SystemExit(message)


def verify(source: Path, target: Path) -> None:
    if not os.path.lexists(target):
        fail(f"global skill is not installed: {target}")
    if not target.is_symlink():
        fail(f"refusing non-symlink global skill target: {target}")
    if target.resolve() != source:
        fail(f"global skill points to {target.resolve()}, expected {source}")
    missing = [relative for relative in REQUIRED_PATHS if not (target / relative).is_file()]
    if missing:
        fail("global skill is incomplete: " + ", ".join(missing))


def main() -> None:
    args = parse_args()
    source = Path(__file__).resolve().parents[1]
    codex_home = args.codex_home
    if codex_home is None:
        codex_home = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
    target = codex_home.expanduser().resolve() / "skills" / SKILL_NAME

    if args.check:
        verify(source, target)
        print(f"VERIFIED global skill {target} -> {source}")
        return

    target.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(target):
        verify(source, target)
        print(f"ALREADY installed global skill {target} -> {source}")
        return

    target.symlink_to(source, target_is_directory=True)
    verify(source, target)
    print(f"INSTALLED global skill {target} -> {source}")


if __name__ == "__main__":
    main()
