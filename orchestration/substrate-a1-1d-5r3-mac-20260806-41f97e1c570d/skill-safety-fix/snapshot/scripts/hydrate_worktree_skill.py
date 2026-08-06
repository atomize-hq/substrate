#!/usr/bin/env python3
"""Copy this repository-local skill into a fresh task worktree and verify it."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path


SKILL_NAME = "orchestrate-top-level-tasks"
REQUIRED_PATHS = (
    "SKILL.md",
    "references/protocol.md",
    "assets/meta-orchestrator-prompt-template.md",
    "assets/increment-orchestrator-prompt-template.md",
    "assets/evidence-task-prompt-template.md",
    "scripts/hydrate_worktree_skill.py",
    "scripts/validate_receipt.py",
    "scripts/validate_state.py",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("worktree", type=Path, help="fresh task worktree to hydrate")
    parser.add_argument(
        "--source",
        type=Path,
        help="authoritative repository-local skill root (defaults to this script's parent skill)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify an existing hydrated copy without changing it",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    raise SystemExit(message)


def included_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        relative = path.relative_to(root)
        if "__pycache__" in relative.parts or path.name == ".DS_Store" or path.suffix == ".pyc":
            continue
        if path.is_symlink():
            fail(f"skill hydration refuses symlink: {path}")
        if path.is_file():
            files.append(relative)
    return sorted(files)


def inventory(root: Path) -> tuple[list[dict[str, object]], str]:
    records: list[dict[str, object]] = []
    aggregate = hashlib.sha256()
    for relative in included_files(root):
        path = root / relative
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        mode = path.stat().st_mode & 0o777
        relative_text = relative.as_posix()
        records.append(
            {"path": relative_text, "sha256": digest, "mode": f"{mode:04o}"}
        )
        aggregate.update(relative_text.encode())
        aggregate.update(b"\0")
        aggregate.update(digest.encode())
        aggregate.update(b"\0")
        aggregate.update(f"{mode:04o}".encode())
        aggregate.update(b"\n")
    return records, aggregate.hexdigest()


def validate_root(root: Path, label: str) -> None:
    if not root.is_dir():
        fail(f"{label} is not a directory: {root}")
    missing = [relative for relative in REQUIRED_PATHS if not (root / relative).is_file()]
    if missing:
        fail(f"{label} is incomplete: " + ", ".join(missing))


def report(status: str, source: Path, target: Path) -> None:
    records, digest = inventory(target)
    print(
        json.dumps(
            {
                "status": status,
                "source": str(source),
                "target": str(target),
                "file_count": len(records),
                "sha256": digest,
            },
            sort_keys=True,
        )
    )


def verify_equal(source: Path, target: Path) -> None:
    validate_root(target, "hydrated skill")
    source_records, source_digest = inventory(source)
    target_records, target_digest = inventory(target)
    if source_records != target_records or source_digest != target_digest:
        fail(
            "hydrated skill differs from repository-local source: "
            f"source={source_digest} target={target_digest}"
        )


def copy_skill(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=f".{SKILL_NAME}.hydrate-", dir=target.parent))
    try:
        for relative in included_files(source):
            source_path = source / relative
            target_path = temporary / relative
            target_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source_path, target_path)
        validate_root(temporary, "temporary hydrated skill")
        verify_equal(source, temporary)
        os.replace(temporary, target)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def main() -> None:
    args = parse_args()
    source = (args.source or Path(__file__).resolve().parents[1]).expanduser().resolve()
    worktree = args.worktree.expanduser().resolve()
    validate_root(source, "repository-local skill")
    if not worktree.is_dir() or not (worktree / ".git").exists():
        fail(f"task worktree is not an existing Git checkout: {worktree}")

    target = worktree / ".agents" / "skills" / SKILL_NAME
    if source == target.resolve(strict=False):
        fail("source and hydrated target are the same path")

    if target.exists() or target.is_symlink():
        if target.is_symlink():
            fail(f"hydrated skill must be a repository-local copy, not a symlink: {target}")
        verify_equal(source, target)
        report("verified", source, target)
        return

    if args.check:
        fail(f"hydrated skill is missing: {target}")

    copy_skill(source, target)
    verify_equal(source, target)
    report("hydrated", source, target)


if __name__ == "__main__":
    main()
