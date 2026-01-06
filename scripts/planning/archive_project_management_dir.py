#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


def run(cmd: list[str], *, cwd: Path | None = None, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, check=check, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def git_root() -> Path:
    res = run(["git", "rev-parse", "--show-toplevel"])
    return Path(res.stdout.strip())


def git_status_dirty(repo_root: Path) -> bool:
    res = run(["git", "status", "--porcelain=v1"], cwd=repo_root)
    return bool(res.stdout.strip())


def is_text_file(path: Path) -> bool:
    # Heuristic: treat known project_management file types as text.
    text_suffixes = {
        ".md",
        ".txt",
        ".json",
        ".yaml",
        ".yml",
        ".toml",
        ".tmpl",
        ".sh",
        ".ps1",
        ".py",
    }
    if path.suffix in text_suffixes:
        return True
    if path.name.endswith(".md.tmpl"):
        return True
    return False


@dataclass(frozen=True)
class ArchivePlan:
    src_dir_repo: Path
    dst_dir_repo: Path
    replacements: list[tuple[str, str]]


def compute_archive_plan(repo_root: Path, src_dir: Path) -> ArchivePlan:
    pm_root = repo_root / "docs" / "project_management"
    try:
        src_abs = src_dir.resolve()
    except FileNotFoundError:
        raise ValueError(f"Source directory does not exist: {src_dir}") from None

    if not src_abs.is_dir():
        raise ValueError(f"Source is not a directory: {src_abs}")

    try:
        rel_to_pm = src_abs.relative_to(pm_root)
    except ValueError:
        raise ValueError(f"Source must be under {pm_root}: {src_abs}") from None

    parts = rel_to_pm.parts
    if len(parts) < 2:
        raise ValueError(
            f"Refusing to archive a top-level bucket; expected something like docs/project_management/next/<name>: {src_abs}"
        )

    bucket = parts[0]
    if bucket == "_archived":
        raise ValueError(f"Source is already archived: {src_abs}")

    tail = Path(*parts[1:])
    dst_abs = pm_root / "_archived" / tail

    src_dir_repo = src_abs.relative_to(repo_root)
    dst_dir_repo = dst_abs.relative_to(repo_root)

    if dst_abs.exists():
        raise ValueError(f"Destination already exists: {dst_abs}")

    old_prefix = f"docs/project_management/{bucket}/{tail.as_posix()}"
    new_prefix = f"docs/project_management/_archived/{tail.as_posix()}"
    replacements = [
        (old_prefix, new_prefix),
        (f"./{old_prefix}", f"./{new_prefix}"),
    ]

    return ArchivePlan(src_dir_repo=src_dir_repo, dst_dir_repo=dst_dir_repo, replacements=replacements)


def iter_project_management_files(repo_root: Path) -> Iterable[Path]:
    pm_root = repo_root / "docs" / "project_management"
    for root, _, filenames in os.walk(pm_root):
        for name in filenames:
            path = Path(root) / name
            if is_text_file(path):
                yield path


def rewrite_paths_in_file(path: Path, replacements: list[tuple[str, str]]) -> tuple[bool, str]:
    try:
        raw = path.read_bytes()
    except OSError:
        return False, ""

    # Skip binary-ish files.
    if b"\x00" in raw:
        return False, ""

    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return False, ""

    original = text
    for old, new in replacements:
        text = text.replace(old, new)

    if text == original:
        return False, ""

    return True, text


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Archive a docs/project_management directory into docs/project_management/_archived/ and rewrite in-repo references.\n\n"
            "Mapping rule:\n"
            "  docs/project_management/<bucket>/<tail> -> docs/project_management/_archived/<tail>\n\n"
            "Example:\n"
            "  docs/project_management/next/env_var_taxonomy_and_override_split -> docs/project_management/_archived/env_var_taxonomy_and_override_split\n"
        )
    )
    parser.add_argument("--src", required=True, help="Directory to archive (e.g. docs/project_management/next/<feature>)")
    parser.add_argument("--dry-run", action="store_true", help="Print planned actions without modifying anything")
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow running with a dirty git working tree (not recommended)",
    )
    args = parser.parse_args()

    repo_root = git_root()
    if not args.allow_dirty and git_status_dirty(repo_root):
        print("ERROR: git working tree is dirty; commit/stash changes or pass --allow-dirty", file=sys.stderr)
        return 2

    try:
        plan = compute_archive_plan(repo_root, repo_root / args.src)
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2

    src_abs = repo_root / plan.src_dir_repo
    dst_abs = repo_root / plan.dst_dir_repo

    print(f"SRC={plan.src_dir_repo.as_posix()}")
    print(f"DST={plan.dst_dir_repo.as_posix()}")
    for old, new in plan.replacements:
        print(f"REWRITE={old} -> {new}")

    if args.dry_run:
        print("DRY_RUN=1")
    else:
        print("DRY_RUN=0")

    if args.dry_run:
        print(f"[dry-run] git mv {plan.src_dir_repo.as_posix()} {plan.dst_dir_repo.as_posix()}")
    else:
        dst_abs.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "mv", plan.src_dir_repo.as_posix(), plan.dst_dir_repo.as_posix()],
            cwd=repo_root,
            check=True,
        )

    modified: list[Path] = []
    for path in iter_project_management_files(repo_root):
        changed, new_text = rewrite_paths_in_file(path, plan.replacements)
        if not changed:
            continue
        modified.append(path.relative_to(repo_root))
        if not args.dry_run:
            path.write_text(new_text, encoding="utf-8", newline="\n")

    if modified:
        print(f"UPDATED_FILES={len(modified)}")
        for p in sorted(modified):
            print(f"UPDATED={p.as_posix()}")
    else:
        print("UPDATED_FILES=0")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

