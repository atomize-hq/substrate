#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _usage_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _run(cmd: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)


def _git_repo_root() -> Path:
    res = _run(["git", "rev-parse", "--show-toplevel"])
    if res.returncode != 0:
        _usage_error(f"not in a git repo/worktree (git rev-parse failed): {res.stderr.strip() or res.stdout.strip()}")
    out = res.stdout.strip()
    if not out:
        _usage_error("git rev-parse returned empty repo root")
    return Path(out).resolve()


def _git_common_dir(repo_root: Path) -> Path:
    res = _run(["git", "rev-parse", "--git-common-dir"], cwd=repo_root)
    if res.returncode != 0:
        _usage_error(f"failed to locate git common dir (git rev-parse --git-common-dir): {res.stderr.strip()}")
    raw = res.stdout.strip()
    if not raw:
        _usage_error("git rev-parse --git-common-dir returned empty path")
    p = Path(raw)
    if p.is_absolute():
        return p.resolve()
    return (repo_root / p).resolve()


def _git_dirty(repo_root: Path) -> bool:
    res = _run(["git", "status", "--porcelain=v1"], cwd=repo_root)
    return bool(res.stdout.strip())


def _normalize_repo_relpath(path: str) -> str:
    p = (path or "").strip().replace("\\", "/")
    while p.startswith("./"):
        p = p[2:]
    return p.rstrip("/")


def _pm_resolve_feature_dir(repo_root: Path, feature_dir: str) -> str:
    res = _run(
        [
            sys.executable,
            "scripts/planning/pm_paths.py",
            "resolve-feature-dir",
            "--feature-dir",
            feature_dir,
        ],
        cwd=repo_root,
    )
    if res.returncode != 0:
        _usage_error(
            f"failed to resolve feature dir via pm_paths.py: {feature_dir!r} ({res.stderr.strip() or res.stdout.strip()})"
        )
    return _normalize_repo_relpath(res.stdout.strip())


@dataclass(frozen=True)
class _TaskmetaEdit:
    path: Path
    changed: bool
    skipped: bool


def _read_json_file(path: Path) -> dict:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as e:
        _usage_error(f"failed to read JSON file: {path} ({e})")
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as e:
        _usage_error(f"invalid JSON: {path} ({e})")
    if not isinstance(data, dict):
        _usage_error(f"expected JSON object in {path}, got {type(data).__name__}")
    return data


def _write_json_atomic(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)


def _discover_worktrees_from_registry(registry_abs: Path) -> list[Path]:
    if not registry_abs.exists():
        return []
    data = _read_json_file(registry_abs)
    entries = data.get("entries", [])
    if not isinstance(entries, list):
        return []
    worktrees: list[Path] = []
    for e in entries:
        if not isinstance(e, dict):
            continue
        wt = e.get("worktree")
        if isinstance(wt, str) and wt.strip():
            worktrees.append(Path(wt).expanduser())
    return worktrees


def _discover_worktrees_from_git(repo_root: Path) -> list[Path]:
    res = _run(["git", "worktree", "list", "--porcelain"], cwd=repo_root)
    if res.returncode != 0:
        return []
    worktrees: list[Path] = []
    for line in res.stdout.splitlines():
        if line.startswith("worktree "):
            worktrees.append(Path(line[len("worktree ") :].strip()).expanduser())
    return worktrees


def _recursive_rewrite_strings(value, *, old_prefix: str, new_prefix: str):
    if isinstance(value, str):
        v = _normalize_repo_relpath(value)
        if v == old_prefix:
            return new_prefix, True
        if v.startswith(old_prefix + "/"):
            return new_prefix + v[len(old_prefix) :], True
        return value, False
    if isinstance(value, list):
        any_changed = False
        out = []
        for item in value:
            new_item, changed = _recursive_rewrite_strings(item, old_prefix=old_prefix, new_prefix=new_prefix)
            any_changed = any_changed or changed
            out.append(new_item)
        return out, any_changed
    if isinstance(value, dict):
        any_changed = False
        out = {}
        for k, v in value.items():
            new_v, changed = _recursive_rewrite_strings(v, old_prefix=old_prefix, new_prefix=new_prefix)
            any_changed = any_changed or changed
            out[k] = new_v
        return out, any_changed
    return value, False


def _edit_taskmeta(taskmeta_path: Path, *, from_dir: str, to_dir: str, dry_run: bool) -> _TaskmetaEdit:
    data = _read_json_file(taskmeta_path)
    raw_feature_dir = data.get("feature_dir")
    if not isinstance(raw_feature_dir, str) or not raw_feature_dir.strip():
        return _TaskmetaEdit(path=taskmeta_path, changed=False, skipped=True)

    current = _normalize_repo_relpath(raw_feature_dir)
    if current == to_dir:
        return _TaskmetaEdit(path=taskmeta_path, changed=False, skipped=False)
    if current != from_dir:
        return _TaskmetaEdit(path=taskmeta_path, changed=False, skipped=True)

    data["feature_dir"] = to_dir
    if not dry_run:
        _write_json_atomic(taskmeta_path, data)
    return _TaskmetaEdit(path=taskmeta_path, changed=True, skipped=False)


def _edit_registry(registry_abs: Path, *, from_dir: str, to_dir: str, dry_run: bool) -> bool:
    if not registry_abs.exists():
        return False
    data = _read_json_file(registry_abs)

    changed = False
    feature_dir = data.get("feature_dir")
    if isinstance(feature_dir, str):
        normalized = _normalize_repo_relpath(feature_dir)
        if normalized == from_dir:
            data["feature_dir"] = to_dir
            changed = True

    rewritten, nested_changed = _recursive_rewrite_strings(data, old_prefix=from_dir, new_prefix=to_dir)
    if nested_changed:
        data = rewritten
        changed = True

    if changed and not dry_run:
        _write_json_atomic(registry_abs, data)
    return changed


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Migrate in-flight triad worktrees by rewriting .taskmeta.json and triad registry feature_dir from --from to --to.",
        epilog=(
            "Controlled test procedure:\n"
            "  1) Create a temporary worktree and add a .taskmeta.json at its root with feature_dir set to --from.\n"
            "  2) Create a dummy registry file at <git-common-dir>/triad/features/<feature>/worktrees.json with an entry pointing at that worktree.\n"
            "  3) Run with --dry-run and confirm it prints CHANGE lines for the expected files.\n"
            "  4) Run without --dry-run and confirm feature_dir is rewritten in both .taskmeta.json and worktrees.json.\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("--from", dest="from_dir", required=True, help="e.g. docs/project_management/next/<feature>")
    ap.add_argument("--to", dest="to_dir", required=True, help="e.g. docs/project_management/packs/active/<feature>")
    ap.add_argument("--dry-run", action="store_true", help="Print planned changes without writing files")
    ap.add_argument(
        "--allow-dirty-reason",
        default="",
        help="Allow running with uncommitted changes; prints the provided reason to stdout before changes",
    )
    args = ap.parse_args(argv)

    repo_root = _git_repo_root()
    from_dir = _pm_resolve_feature_dir(repo_root, args.from_dir)
    to_dir = _pm_resolve_feature_dir(repo_root, args.to_dir)

    from_feature = Path(from_dir).name
    to_feature = Path(to_dir).name
    if not from_feature or not to_feature or from_feature != to_feature:
        _usage_error(f"--from and --to must have the same final path component (feature): {from_dir!r} -> {to_dir!r}")

    if _git_dirty(repo_root) and not args.allow_dirty_reason.strip():
        _usage_error("git working tree is dirty; commit/stash changes or pass --allow-dirty-reason '<why>'")

    if args.allow_dirty_reason.strip():
        print(f"ALLOW_DIRTY_REASON: {args.allow_dirty_reason.strip()}")

    git_common_dir = _git_common_dir(repo_root)
    registry_abs = git_common_dir / "triad" / "features" / from_feature / "worktrees.json"

    worktrees: list[Path] = []
    seen: set[Path] = set()

    for wt in _discover_worktrees_from_registry(registry_abs):
        wt_abs = wt.expanduser().resolve()
        if wt_abs not in seen:
            seen.add(wt_abs)
            worktrees.append(wt_abs)

    for wt in _discover_worktrees_from_git(repo_root):
        wt_abs = wt.expanduser().resolve()
        if wt_abs not in seen:
            seen.add(wt_abs)
            worktrees.append(wt_abs)

    changed_paths: list[Path] = []
    skipped = 0
    taskmeta_updated = 0

    for wt in worktrees:
        taskmeta_path = wt / ".taskmeta.json"
        if not taskmeta_path.exists():
            continue
        edit = _edit_taskmeta(taskmeta_path, from_dir=from_dir, to_dir=to_dir, dry_run=args.dry_run)
        if edit.skipped:
            skipped += 1
            continue
        if edit.changed:
            taskmeta_updated += 1
            changed_paths.append(edit.path)

    registry_changed = _edit_registry(registry_abs, from_dir=from_dir, to_dir=to_dir, dry_run=args.dry_run)
    if registry_changed:
        changed_paths.append(registry_abs)

    for p in sorted({cp.resolve() for cp in changed_paths}):
        print(f"CHANGE: {p}")

    print(
        "SUMMARY: "
        f"taskmeta_updated={taskmeta_updated} "
        f"registry_updated={1 if registry_changed else 0} "
        f"skipped={skipped}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
