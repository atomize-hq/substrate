#!/usr/bin/env python3

import argparse
from pathlib import Path


SENTINEL = "Do not edit planning docs inside the worktree."


def ensure_in_file(path: Path) -> bool:
    text = path.read_text(encoding="utf-8")
    if SENTINEL in text:
        return False

    lines = text.splitlines(keepends=True)
    out = []
    inserted = False

    for i, line in enumerate(lines):
        out.append(line)
        if inserted:
            continue
        if line.strip() == "## Start Checklist":
            out.append(f"\n{SENTINEL}\n\n")
            inserted = True

    if not inserted:
        out.append(f"\n\n{SENTINEL}\n")

    updated = "".join(out)
    if updated == text:
        return False

    path.write_text(updated, encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Ensure kickoff prompts contain the canonical no-doc-edits sentinel.")
    parser.add_argument("--root", default="docs/project_management/next", help="Root directory to scan (default: docs/project_management/next)")
    args = parser.parse_args()

    root = Path(args.root)
    if not root.exists():
        raise SystemExit(f"Root does not exist: {root}")

    changed = 0
    for kickoff_dir in root.rglob("kickoff_prompts"):
        if not kickoff_dir.is_dir():
            continue
        for path in kickoff_dir.glob("*.md"):
            if ensure_in_file(path):
                changed += 1

    print(f"Updated kickoff prompts: {changed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

