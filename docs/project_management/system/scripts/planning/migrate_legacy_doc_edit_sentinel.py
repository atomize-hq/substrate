#!/usr/bin/env python3

import argparse
import os
import re
from pathlib import Path


REPLACEMENTS = [
    # Canonical kickoff prompt sentinel variants.
    (
        re.compile(r"Do\s+\*\*not\*\*\s+edit\s+docs/tasks/session_log\.md\s+from\s+the\s+worktree\.", re.IGNORECASE),
        "Do not edit planning docs inside the worktree.",
    ),
    (
        re.compile(r"Do\s+not\s+edit\s+docs/tasks/session_log\.md\s+from\s+the\s+worktree\.", re.IGNORECASE),
        "Do not edit planning docs inside the worktree.",
    ),
    (
        re.compile(r"Do\s+not\s+edit\s+docs/tasks/session_log\.md\s+inside\s+the\s+worktree\.", re.IGNORECASE),
        "Do not edit planning docs inside the worktree.",
    ),
    # Checklist shorthand variants.
    (
        re.compile(r"\(no\s+docs/tasks/session_log\.md\s+edits\)", re.IGNORECASE),
        "(no planning docs edits)",
    ),
    (
        re.compile(r"no\s+docs/tasks/session_log\.md\s+edits", re.IGNORECASE),
        "no planning docs edits",
    ),
    # Inline checklist fragments (tasks.json frequently uses lowercase / no trailing period).
    (
        re.compile(r"do\s+not\s+edit\s+docs/tasks/session_log\.md\s+inside\s+the\s+worktree", re.IGNORECASE),
        "do not edit planning docs inside the worktree",
    ),
    # Generic phrasing used in some kickoff prompts.
    (
        re.compile(r"Do\s+not\s+edit\s+docs/tasks/logs\s+inside\s+the\s+worktree\.", re.IGNORECASE),
        "Do not edit planning docs inside the worktree.",
    ),
    (
        re.compile(r"no\s+docs/tasks/log\s+edits\s+in\s+worktree\.?", re.IGNORECASE),
        "Do not edit planning docs inside the worktree.",
    ),
]


def rewrite_file(path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    updated = original
    for pattern, replacement in REPLACEMENTS:
        updated = pattern.sub(replacement, updated)

    if updated == original:
        return False

    path.write_text(updated, encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Migrate legacy `docs/tasks/session_log.md` sentinel phrases to the canonical planning-doc sentinel.")
    parser.add_argument("--root", default="docs/project_management/next", help="Root directory to rewrite (default: docs/project_management/next)")
    args = parser.parse_args()

    root = Path(args.root)
    if not root.exists():
        raise SystemExit(f"Root does not exist: {root}")

    changed = 0
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix not in {".md", ".json"}:
            continue
        if rewrite_file(path):
            changed += 1

    print(f"Updated files: {changed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
