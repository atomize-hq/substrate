#!/usr/bin/env python3
"""Run each sampled session through compactor -> analyzer in isolation.

Step 2 of the semantic-goal-drift real-world validation pipeline
(see README.md). Each session is compacted and analyzed inside its own
temp codex-home so cross-session exact-dedupe in the compactor cannot
contaminate results. Per-session `checkpoints.jsonl` outputs are written
under `<batch-dir>/checkpoints/<session_id>.jsonl`, each row annotated
with `_month`, `_repo`, `_session_file_id` for downstream tabulation.

Requires the release/debug binaries to be built first:
    cargo build -p agent-session-compactor -p agent-drift-analyzer
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from collections import Counter


def find_repo_root(start: str) -> str:
    d = os.path.abspath(start)
    while d != "/":
        if os.path.exists(os.path.join(d, "Cargo.toml")) and os.path.isdir(os.path.join(d, "crates")):
            return d
        d = os.path.dirname(d)
    raise SystemExit("could not locate repo root (Cargo.toml + crates/) above " + start)


# Session-level delegation stratification (R6-3.5). Coarse marker scan over the raw rollout using
# the analyzer's own delegation vocabulary (crates/agent-drift-analyzer/src/inference/mod.rs). This
# is a *reporting* tag so the batch precision numbers can be reported separately for delegated vs
# single-agent traces; it is intentionally coarser than the analyzer's per-checkpoint
# DelegationContext (topology + child_work_visibility). Categories mirror the R6-3.5 charter:
# single_agent / delegated_parent_opaque / delegated_child_visible / unknown.
_DELEGATION_MARKERS = ("multi_agent_v1", "spawn_agent", "wait_agent", "close_agent")
_CHILD_VISIBILITY_SIGNALS = (
    "child rollout", "separate rollout", "subagent", "spawned agent", "child session id",
)


def classify_session_delegation(rollout_path: str) -> str:
    try:
        with open(rollout_path, "r", errors="replace") as f:
            blob = f.read().lower()
    except OSError:
        return "unknown"
    has_marker = any(m in blob for m in _DELEGATION_MARKERS)
    has_child_visibility = any(s in blob for s in _CHILD_VISIBILITY_SIGNALS)
    if not has_marker and not has_child_visibility:
        return "single_agent"
    if has_child_visibility:
        return "delegated_child_visible"
    return "delegated_parent_opaque"


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", default=None,
                    help="repo root (default: auto-detect from this script's location)")
    ap.add_argument("--profile", default="debug", choices=["debug", "release"],
                    help="which target/<profile> binaries to use (default: debug)")
    ap.add_argument("--selected", default="selected_sessions.jsonl",
                    help="manifest from sample_sessions.py (default: ./selected_sessions.jsonl)")
    ap.add_argument("--batch-dir", default="batch",
                    help="output dir for per-session checkpoints (default: ./batch)")
    ap.add_argument("--compact-timeout", type=int, default=180)
    ap.add_argument("--analyze-timeout", type=int, default=300)
    args = ap.parse_args()

    repo = args.repo or find_repo_root(__file__)
    compact = os.path.join(repo, "target", args.profile, "agent-session-compactor")
    analyze = os.path.join(repo, "target", args.profile, "agent-drift-analyzer")
    for b in (compact, analyze):
        if not os.path.exists(b):
            raise SystemExit(f"missing binary {b}\n  build first: "
                             f"cargo build -p agent-session-compactor -p agent-drift-analyzer")

    ckdir = os.path.join(args.batch_dir, "checkpoints")
    work = os.path.join(args.batch_dir, "work")
    os.makedirs(ckdir, exist_ok=True)
    os.makedirs(work, exist_ok=True)

    sessions = [json.loads(l) for l in open(args.selected) if l.strip()]
    status = []
    ok = 0
    for i, s in enumerate(sessions, 1):
        sid = s["session_id"]
        path = s["path"]
        tmp = tempfile.mkdtemp(dir=work)
        ch = os.path.join(tmp, "codex-home", "sessions")
        os.makedirs(ch, exist_ok=True)
        bundle = os.path.join(tmp, "bundle")
        analysis = os.path.join(tmp, "analysis")
        os.makedirs(bundle, exist_ok=True)
        os.makedirs(analysis, exist_ok=True)
        rec = {"session_id": sid, "month": s["month"], "repo": s["repo"],
               "size": s["size"], "ok": False, "n_checkpoints": 0, "error": None}
        delegation = classify_session_delegation(path)
        rec["delegation"] = delegation
        try:
            shutil.copy(path, ch)
            r1 = subprocess.run([compact, "--codex-home", os.path.join(tmp, "codex-home"),
                                 "--output-dir", bundle], capture_output=True, text=True,
                                timeout=args.compact_timeout)
            if r1.returncode != 0:
                rec["error"] = "compact:" + (r1.stderr.strip().splitlines()[-1]
                                             if r1.stderr.strip() else "rc%d" % r1.returncode)
                raise RuntimeError(rec["error"])
            r2 = subprocess.run([analyze, "--input-dir", bundle, "--output-dir", analysis],
                                capture_output=True, text=True, timeout=args.analyze_timeout)
            if r2.returncode != 0:
                rec["error"] = "analyze:" + (r2.stderr.strip().splitlines()[-1]
                                             if r2.stderr.strip() else "rc%d" % r2.returncode)
                raise RuntimeError(rec["error"])
            cpsrc = os.path.join(analysis, "checkpoints.jsonl")
            n = 0
            with open(os.path.join(ckdir, sid + ".jsonl"), "w") as out:
                for line in open(cpsrc):
                    if not line.strip():
                        continue
                    cp = json.loads(line)
                    cp["_month"] = s["month"]
                    cp["_repo"] = s["repo"]
                    cp["_session_file_id"] = sid
                    cp["_delegation"] = delegation
                    out.write(json.dumps(cp) + "\n")
                    n += 1
            rec["ok"] = True
            rec["n_checkpoints"] = n
            ok += 1
        except Exception as e:
            if not rec["error"]:
                rec["error"] = str(e)[:200]
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        status.append(rec)
        if i % 15 == 0 or i == len(sessions):
            print(f"  {i}/{len(sessions)} done ok={ok}", flush=True)

    with open(os.path.join(args.batch_dir, "status.jsonl"), "w") as f:
        for r in status:
            f.write(json.dumps(r) + "\n")
    print(f"\nRAN {len(sessions)} sessions | ok={ok} | failed={len(sessions) - ok}")
    errs = Counter((r["error"] or "").split(":")[0] for r in status if not r["ok"])
    if errs:
        print("failure kinds:", dict(errs))


if __name__ == "__main__":
    main()
