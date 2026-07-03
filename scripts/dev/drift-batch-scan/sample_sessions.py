#!/usr/bin/env python3
"""Sample codex rollout sessions equally across months with repo diversity.

Step 1 of the semantic-goal-drift real-world validation pipeline
(see README.md). Writes a `selected_sessions.jsonl` manifest that
`run_batch.py` consumes.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import re
from collections import Counter

DEFAULT_MONTHS = [
    "2025-08", "2025-09", "2025-10", "2025-11", "2025-12",
    "2026-01", "2026-02", "2026-03", "2026-04", "2026-05", "2026-06", "2026-07",
]


def read_cwd_and_sid(path: str):
    try:
        with open(path, "r") as f:
            first = f.readline()
        obj = json.loads(first)
        if obj.get("type") != "session_meta":
            return None, None
        pl = obj.get("payload", {})
        return pl.get("cwd"), pl.get("id")
    except Exception:
        return None, None


def norm_repo(cwd: str | None) -> str:
    if not cwd:
        return "(unknown)"
    # collapse worktree hash segments: .../worktrees/<hash>/<repo> -> .../<repo>
    c = re.sub(r"/worktrees/[0-9a-fA-F]+/", "/", cwd)
    # collapse conductor/temp worktree style too
    c = re.sub(r"/\.conductor/[^/]+/", "/", c)
    return os.path.basename(c.rstrip("/")) or c


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--sessions-root", default=os.path.expanduser("~/.codex/sessions"),
                    help="codex rollout session store (default: ~/.codex/sessions)")
    ap.add_argument("--out", default="selected_sessions.jsonl",
                    help="manifest output path (default: ./selected_sessions.jsonl)")
    ap.add_argument("--per-month", type=int, default=10)
    ap.add_argument("--max-attempts-per-month", type=int, default=60,
                    help="bound head-1 reads per month")
    ap.add_argument("--min-bytes", type=int, default=1024, help="skip empty/degenerate")
    ap.add_argument("--max-bytes", type=int, default=15 * 1024 * 1024,
                    help="skip absurdly large (pipeline safety)")
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument("--months", nargs="*", default=DEFAULT_MONTHS,
                    help="YYYY-MM buckets to sample (default: 2025-08 .. 2026-07)")
    args = ap.parse_args()

    import random
    random.seed(args.seed)

    selected = []
    summary = []
    for ym in args.months:
        y, m = ym.split("-")
        files = glob.glob(os.path.join(args.sessions_root, y, m, "**", "rollout-*.jsonl"),
                          recursive=True)
        random.shuffle(files)
        picked = []           # (path, cwd, repo, sid, size)
        repos_seen = set()
        pool = []             # candidates read so far (for backfill)
        attempts = 0
        # First pass: prefer new repos
        for path in files:
            if len(picked) >= args.per_month or attempts >= args.max_attempts_per_month:
                break
            try:
                size = os.path.getsize(path)
            except OSError:
                continue
            if size < args.min_bytes or size > args.max_bytes:
                continue
            attempts += 1
            cwd, sid = read_cwd_and_sid(path)
            if not sid:
                continue
            repo = norm_repo(cwd)
            pool.append((path, cwd, repo, sid, size))
            if repo not in repos_seen:
                repos_seen.add(repo)
                picked.append((path, cwd, repo, sid, size))
        # Backfill from pool (repeat repos allowed) if under quota
        if len(picked) < args.per_month:
            picked_paths = {p[0] for p in picked}
            for cand in pool:
                if len(picked) >= args.per_month:
                    break
                if cand[0] in picked_paths:
                    continue
                picked.append(cand)
                picked_paths.add(cand[0])
        for path, cwd, repo, sid, size in picked:
            selected.append({"path": path, "month": ym, "cwd": cwd, "repo": repo,
                             "session_id": sid, "size": size})
        summary.append((ym, len(picked), len({p[2] for p in picked})))

    with open(args.out, "w") as f:
        for s in selected:
            f.write(json.dumps(s) + "\n")

    print("month    | picked | distinct_repos")
    for ym, n, r in summary:
        print(f"{ym} |   {n:2d}   |   {r}")
    print(f"\nTOTAL selected: {len(selected)}")
    rc = Counter(s["repo"] for s in selected)
    print(f"distinct repos overall: {len(rc)}")
    print("top repos:", dict(rc.most_common(12)))
    print(f"manifest -> {args.out}")


if __name__ == "__main__":
    main()
