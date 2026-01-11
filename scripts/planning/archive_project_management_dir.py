#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Iterator


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
    src_dir_repo: Path | None
    dst_dir_repo: Path | None
    replacements: list[tuple[str, str]]
    strict_needles: list[str]


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

    strict_needles = [
        old_prefix,
        f"./{old_prefix}",
    ]

    return ArchivePlan(
        src_dir_repo=src_dir_repo,
        dst_dir_repo=dst_dir_repo,
        replacements=replacements,
        strict_needles=strict_needles,
    )

def compute_rewrite_only_plan(from_prefix: str, to_prefix: str) -> ArchivePlan:
    from_prefix = from_prefix.strip()
    to_prefix = to_prefix.strip()
    if not from_prefix or not to_prefix:
        raise ValueError("--from and --to must be non-empty")
    replacements = [
        (from_prefix, to_prefix),
        (f"./{from_prefix}", f"./{to_prefix}"),
    ]
    strict_needles = [
        from_prefix,
        f"./{from_prefix}",
    ]
    return ArchivePlan(
        src_dir_repo=None,
        dst_dir_repo=None,
        replacements=replacements,
        strict_needles=strict_needles,
    )


def should_skip_dir(dirname: str) -> bool:
    # Keep this conservative: avoid rewriting vendored/build/worktree content.
    return dirname in {
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        "out",
        "wt",
        ".venv",
        "__pycache__",
    }


def iter_repo_text_files(repo_root: Path) -> Iterator[Path]:
    for root, dirnames, filenames in os.walk(repo_root):
        dirnames[:] = [d for d in dirnames if not should_skip_dir(d)]
        for name in filenames:
            path = Path(root) / name
            if is_text_file(path):
                yield path


def rewrite_paths_in_file(path: Path, replacements: list[tuple[str, str]]) -> tuple[bool, str]:
    try:
        raw = path.read_bytes()
    except OSError:
        return False, ""

    # Avoid huge file rewrites (logs, corpora, large fixtures).
    if len(raw) > 5 * 1024 * 1024:
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


def find_remaining_references(
    repo_root: Path, strict_needles: list[str]
) -> list[tuple[Path, str]]:
    remaining: list[tuple[Path, str]] = []
    for path in iter_repo_text_files(repo_root):
        try:
            raw = path.read_bytes()
        except OSError:
            continue
        if b"\x00" in raw:
            continue
        if len(raw) > 5 * 1024 * 1024:
            continue
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            continue
        for needle in strict_needles:
            if needle in text:
                remaining.append((path.relative_to(repo_root), needle))
                break
    return remaining


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
    parser.add_argument("--src", help="Directory to archive (e.g. docs/project_management/next/<feature>)")
    parser.add_argument("--dry-run", action="store_true", help="Print planned actions without modifying anything")
    parser.add_argument(
        "--rewrite-only",
        action="store_true",
        help=(
            "Rewrite references without moving directories. Use --from/--to to retro-fix already-archived packs."
        ),
    )
    parser.add_argument("--from", dest="from_prefix", help="When --rewrite-only: rewrite this prefix")
    parser.add_argument("--to", dest="to_prefix", help="When --rewrite-only: rewrite to this prefix")
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
        if args.rewrite_only and args.from_prefix and args.to_prefix:
            plan = compute_rewrite_only_plan(args.from_prefix, args.to_prefix)
        else:
            if not args.src:
                print(
                    "ERROR: --src is required unless running --rewrite-only with both --from and --to",
                    file=sys.stderr,
                )
                return 2
            plan = compute_archive_plan(repo_root, repo_root / args.src)
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2

    src_abs = (repo_root / plan.src_dir_repo) if plan.src_dir_repo else None
    dst_abs = (repo_root / plan.dst_dir_repo) if plan.dst_dir_repo else None

    if plan.src_dir_repo is not None and plan.dst_dir_repo is not None:
        print(f"SRC={plan.src_dir_repo.as_posix()}")
        print(f"DST={plan.dst_dir_repo.as_posix()}")
    else:
        print("SRC=(none)")
        print("DST=(none)")
    for old, new in plan.replacements:
        print(f"REWRITE={old} -> {new}")

    if args.dry_run:
        print("DRY_RUN=1")
    else:
        print("DRY_RUN=0")

    if args.rewrite_only:
        print("MOVE=0")
    else:
        print("MOVE=1")

    if not args.rewrite_only:
        assert src_abs is not None
        assert dst_abs is not None
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
    for path in iter_repo_text_files(repo_root):
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

    if args.dry_run:
        print("REMAINING_REFERENCES=SKIPPED_DRY_RUN")
        return 0

    remaining = find_remaining_references(repo_root, plan.strict_needles)
    if remaining:
        print(f"REMAINING_REFERENCES={len(remaining)}", file=sys.stderr)
        for path, needle in remaining:
            print(f"REMAINING={path.as_posix()} needle={needle}", file=sys.stderr)
        print(
            "ERROR: stale references remain; rerun with --rewrite-only after updating replacements or fix manually",
            file=sys.stderr,
        )
        return 2
    print("REMAINING_REFERENCES=0")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
