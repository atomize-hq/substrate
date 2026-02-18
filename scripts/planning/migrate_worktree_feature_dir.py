#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys


def _repo_root() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        print(f"ERROR: failed to locate repo root via git: {e}", file=sys.stderr)
        raise SystemExit(2)


def _pm_system_root(repo_root: str) -> str:
    raw = os.environ.get("PM_SYSTEM_ROOT") or "docs/project_management/system"
    if os.path.isabs(raw):
        return raw
    return os.path.join(repo_root, raw)


def main() -> None:
    repo_root = _repo_root()
    pm_system_root = _pm_system_root(repo_root)
    target = os.path.join(pm_system_root, "scripts/planning/migrate_worktree_feature_dir.py")
    os.execv(sys.executable, [sys.executable, target, *sys.argv[1:]])


if __name__ == "__main__":
    main()

