#!/usr/bin/env python3
"""Dump raw target ground-truth behind the fires and disjoint pairs.

Diagnostic helper for the semantic-goal-drift validation pipeline
(see README.md). Prints the raw `structured_objective.target` values
(display / paths / symbols / named_artifacts / workspace_refs) and the
normalized term sets behind (A) every flagged checkpoint and (B) every
target-resolved disjoint adjacent pair, so the junk-target filter can be
designed and hand-verified against real extraction output.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
from collections import defaultdict

GENERIC = {"implement", "debug", "review", "research", "plan", "validate", "docs", "other_task",
           "repo_slice", "crate_or_package", "file_or_directory", "spec_or_design_doc",
           "test_or_verifier", "skill_or_instruction_surface", "external_artifact",
           "conceptual_topic", "unknown_target", "green"}


def norm(raw: str) -> str:
    return "".join(c.lower() if c.isalnum() else "_" for c in raw).strip("_")


def raw_target_values(so: dict):
    t = so.get("target") or {}
    vals = []
    for k in ("display", "paths", "symbols", "named_artifacts", "workspace_refs"):
        v = t.get(k)
        if isinstance(v, list):
            vals += [(k, x) for x in v if x]
        elif v:
            vals.append((k, v))
    return vals


def target_terms(so: dict) -> set[str]:
    terms = set()
    for _, v in raw_target_values(so):
        n = norm(v)
        if n and n not in GENERIC:
            terms.add(n)
    return terms


def conf_ge_medium(c) -> bool:
    return c in ("medium", "high")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--checkpoints-dir", default=os.path.join("batch", "checkpoints"),
                    help="dir of per-session checkpoints (default: ./batch/checkpoints)")
    args = ap.parse_args()

    by_session = defaultdict(list)
    for fp in glob.glob(os.path.join(args.checkpoints_dir, "*.jsonl")):
        for line in open(fp):
            if not line.strip():
                continue
            cp = json.loads(line)
            by_session[cp["_session_file_id"]].append(cp)
    for sid in by_session:
        by_session[sid].sort(key=lambda c: c.get("ordinal", 0))

    print("#" * 78)
    print("# PART A — flagged checkpoints (raw current-goal target)")
    print("#" * 78)
    for sid, cps in by_session.items():
        for cp in cps:
            for s in cp.get("drift_scores", []):
                if s.get("class") == "semantic_goal_drift" and s.get("flagged"):
                    so = cp.get("structured_objective") or {}
                    print(f"\n-- session {sid[:12]} repo={cp['_repo']} month={cp['_month']} "
                          f"ord={cp.get('ordinal')}")
                    print("   raw target values:")
                    for k, v in raw_target_values(so):
                        print(f"     [{k}] {v!r}")
                    print(f"   term set: {sorted(target_terms(so))}")
                    for e in s.get("evidence", []):
                        print(f"   evid: {e.get('reason', '')[:160]}")

    print()
    print("#" * 78)
    print("# PART B — target-resolved disjoint adjacent pairs")
    print("#" * 78)

    def tgt_elig(cp: dict) -> bool:
        so = cp.get("structured_objective") or {}
        unk = {u.get("field_name") for u in so.get("unknowns", [])}
        return (so.get("objective_class") == "task_statement"
                and conf_ge_medium(so.get("confidence"))
                and so.get("target") is not None and "target" not in unk)

    n = 0
    for sid, cps in by_session.items():
        for a, b in zip(cps, cps[1:]):
            if tgt_elig(a) and tgt_elig(b):
                ta = target_terms(a.get("structured_objective") or {})
                tb = target_terms(b.get("structured_objective") or {})
                if ta and tb and ta != tb and ta.isdisjoint(tb):
                    n += 1
                    soa = a.get("structured_objective") or {}
                    sob = b.get("structured_objective") or {}
                    print(f"\n[{n}] session {sid[:12]} repo={a['_repo']} month={a['_month']} "
                          f"ord {a.get('ordinal')}->{b.get('ordinal')}")
                    print(f"   A raw: {[v for _, v in raw_target_values(soa)]}")
                    print(f"     A terms: {sorted(ta)}")
                    print(f"   B raw: {[v for _, v in raw_target_values(sob)]}")
                    print(f"     B terms: {sorted(tb)}")
    print(f"\nTOTAL disjoint pairs: {n}")


if __name__ == "__main__":
    main()
