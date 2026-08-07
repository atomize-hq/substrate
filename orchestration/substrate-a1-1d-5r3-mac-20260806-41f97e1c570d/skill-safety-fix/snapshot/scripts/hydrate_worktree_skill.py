#!/usr/bin/env python3
"""Copy the complete repository-local skill suite into a task worktree."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path


ORCHESTRATION_SKILL = "orchestrate-top-level-tasks"
REQUIRED_PATHS = (
    f"{ORCHESTRATION_SKILL}/SKILL.md",
    f"{ORCHESTRATION_SKILL}/references/protocol.md",
    f"{ORCHESTRATION_SKILL}/assets/meta-orchestrator-prompt-template.md",
    f"{ORCHESTRATION_SKILL}/assets/increment-orchestrator-prompt-template.md",
    f"{ORCHESTRATION_SKILL}/assets/evidence-task-prompt-template.md",
    f"{ORCHESTRATION_SKILL}/scripts/hydrate_worktree_skill.py",
    f"{ORCHESTRATION_SKILL}/scripts/validate_receipt.py",
    f"{ORCHESTRATION_SKILL}/scripts/validate_state.py",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("worktree", type=Path, help="fresh task worktree to hydrate")
    parser.add_argument(
        "--source",
        type=Path,
        help=(
            "authoritative repository-local .agents/skills root; the legacy "
            "orchestrate-top-level-tasks skill root is also accepted"
        ),
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify an existing hydrated skill suite without changing it",
    )
    return parser.parse_args()


def fail(message: str) -> None:
    raise SystemExit(message)


def normalize_source(source: Path) -> Path:
    source = source.expanduser().resolve()
    if source.name == ORCHESTRATION_SKILL and (source / "SKILL.md").is_file():
        return source.parent
    return source


def included_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        relative = path.relative_to(root)
        if "__pycache__" in relative.parts or path.name == ".DS_Store" or path.suffix == ".pyc":
            continue
        if path.is_symlink():
            fail(f"skill-suite hydration refuses symlink: {path}")
        if path.is_file():
            files.append(relative)
    return sorted(files)


def skill_names(root: Path) -> list[str]:
    return sorted(
        path.name
        for path in root.iterdir()
        if path.is_dir() and (path / "SKILL.md").is_file()
    )


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


def validate_suite(root: Path, label: str) -> None:
    if not root.is_dir():
        fail(f"{label} is not a directory: {root}")
    missing = [relative for relative in REQUIRED_PATHS if not (root / relative).is_file()]
    if missing:
        fail(f"{label} is incomplete: " + ", ".join(missing))
    names = skill_names(root)
    if not names:
        fail(f"{label} has no skills: {root}")
    non_skills = sorted(
        path.name
        for path in root.iterdir()
        if path.is_dir()
        and path.name != "__pycache__"
        and not (path / "SKILL.md").is_file()
    )
    if non_skills:
        fail(f"{label} contains directories without SKILL.md: " + ", ".join(non_skills))


def report(status: str, source: Path, target: Path) -> None:
    records, digest = inventory(target)
    names = skill_names(target)
    print(
        json.dumps(
            {
                "status": status,
                "source": str(source),
                "target": str(target),
                "skill_count": len(names),
                "skills": names,
                "file_count": len(records),
                "sha256": digest,
            },
            sort_keys=True,
        )
    )


def verify_equal(source: Path, target: Path) -> None:
    validate_suite(target, "hydrated skill suite")
    source_records, source_digest = inventory(source)
    target_records, target_digest = inventory(target)
    if source_records != target_records or source_digest != target_digest:
        fail(
            "hydrated skill suite differs from repository-local source: "
            f"source={source_digest} target={target_digest}"
        )


def is_verified_legacy_single_skill(source: Path, target: Path) -> bool:
    children = sorted(path.name for path in target.iterdir())
    if children != [ORCHESTRATION_SKILL]:
        return False
    legacy_target = target / ORCHESTRATION_SKILL
    legacy_source = source / ORCHESTRATION_SKILL
    try:
        source_records, source_digest = inventory(legacy_source)
        target_records, target_digest = inventory(legacy_target)
    except (OSError, SystemExit):
        return False
    return source_records == target_records and source_digest == target_digest


def populate(source: Path, target: Path) -> None:
    for relative in included_files(source):
        source_path = source / relative
        target_path = target / relative
        target_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source_path, target_path)


def replace_suite(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=".skills.hydrate-", dir=target.parent))
    backup: Path | None = None
    try:
        populate(source, temporary)
        validate_suite(temporary, "temporary hydrated skill suite")
        verify_equal(source, temporary)
        if target.exists():
            backup = Path(tempfile.mkdtemp(prefix=".skills.legacy-", dir=target.parent))
            backup.rmdir()
            os.replace(target, backup)
        try:
            os.replace(temporary, target)
        except BaseException:
            if backup is not None and backup.exists() and not target.exists():
                os.replace(backup, target)
            raise
        if backup is not None and backup.exists():
            shutil.rmtree(backup)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)
        if backup is not None and backup.exists():
            shutil.rmtree(backup)


def main() -> None:
    args = parse_args()
    default_source = Path(__file__).resolve().parents[2]
    source = normalize_source(args.source or default_source)
    worktree = args.worktree.expanduser().resolve()
    validate_suite(source, "repository-local skill suite")
    if not worktree.is_dir() or not (worktree / ".git").exists():
        fail(f"task worktree is not an existing Git checkout: {worktree}")

    target = worktree / ".agents" / "skills"
    if source == target.resolve(strict=False):
        fail("source and hydrated target are the same path")

    if target.is_symlink():
        fail(f"hydrated skill suite must be a repository-local copy, not a symlink: {target}")

    if target.exists():
        try:
            verify_equal(source, target)
        except SystemExit:
            if args.check:
                raise
            if not is_verified_legacy_single_skill(source, target):
                raise
            replace_suite(source, target)
            verify_equal(source, target)
            report("upgraded", source, target)
            return
        report("verified", source, target)
        return

    if args.check:
        fail(f"hydrated skill suite is missing: {target}")

    replace_suite(source, target)
    verify_equal(source, target)
    report("hydrated", source, target)


if __name__ == "__main__":
    main()
