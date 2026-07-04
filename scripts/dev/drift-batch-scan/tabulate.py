#!/usr/bin/env python3
"""Coverage-diagnostic tabulation of the batch scan.

Step 3 of the semantic-goal-drift real-world validation pipeline
(see README.md). Reproduces the F1-F4 coverage metrics: current-bar
eligibility, actual firings, eligibility-failure breakdown, the
hypothetical target-resolved bar, adjacent-pair (rolling) analysis,
and per-month / per-repo diversity.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
from collections import Counter, defaultdict

# objective_class / target-class enum tokens that are not grounded artifacts
GENERIC = {"implement", "debug", "review", "research", "plan", "validate", "docs", "other_task",
           "repo_slice", "crate_or_package", "file_or_directory", "spec_or_design_doc",
           "test_or_verifier", "skill_or_instruction_surface", "external_artifact",
           "conceptual_topic", "unknown_target", "green"}


def norm(raw: str) -> str:
    return "".join(c.lower() if c.isalnum() else "_" for c in raw).strip("_")


def target_terms(so: dict) -> set[str]:
    t = so.get("target")
    terms: set[str] = set()
    if not t:
        return terms
    for v in ([t.get("display")] + t.get("paths", []) + t.get("symbols", [])
              + t.get("named_artifacts", []) + t.get("workspace_refs", [])):
        if not v:
            continue
        n = norm(v)
        if n and n not in GENERIC:
            terms.add(n)
    return terms


def conf_ge_medium(c) -> bool:
    return c in ("medium", "high")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--checkpoints-dir", default=os.path.join("batch", "checkpoints"),
                    help="dir of per-session checkpoints from run_batch.py "
                         "(default: ./batch/checkpoints)")
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

    tot_cp = 0
    tot_sessions = len(by_session)
    ts = ts_conf = cur_eligible = 0
    fail_reason = Counter()
    target_resolved = target_resolved_no_primary = blocked_only_by_sc_del = 0
    sgd_flagged = rolling_lines = kickoff_lines = 0
    by_month = defaultdict(lambda: {"cp": 0, "ts_conf": 0, "cur_elig": 0, "tgt_elig": 0})
    by_repo = defaultdict(lambda: {"cp": 0, "cur_elig": 0, "tgt_elig": 0})
    # R6-3.5 delegation stratification: report precision-relevant metrics separately per delegation
    # category so single-agent and delegated/opaque traces are not treated as equal-weight evidence.
    by_delegation = defaultdict(
        lambda: {"cp": 0, "cur_elig": 0, "tgt_elig": 0, "sgd": 0, "disjoint": 0})

    for sid, cps in by_session.items():
        for cp in cps:
            tot_cp += 1
            m = cp["_month"]
            r = cp["_repo"]
            d = cp.get("_delegation", "unknown")
            by_month[m]["cp"] += 1
            by_repo[r]["cp"] += 1
            by_delegation[d]["cp"] += 1
            so = cp.get("structured_objective") or {}
            oc = so.get("objective_class")
            conf = so.get("confidence")
            unk = {u.get("field_name") for u in so.get("unknowns", [])}
            tgt_present = so.get("target") is not None
            is_ts = oc == "task_statement"
            is_tsconf = is_ts and conf_ge_medium(conf)
            if is_ts:
                ts += 1
            if is_tsconf:
                ts_conf += 1
                by_month[m]["ts_conf"] += 1
                if not unk:
                    cur_eligible += 1
                    by_month[m]["cur_elig"] += 1
                    by_repo[r]["cur_elig"] += 1
                    by_delegation[d]["cur_elig"] += 1
                else:
                    for f in unk:
                        fail_reason[f] += 1
                if tgt_present and "target" not in unk:
                    target_resolved += 1
                    by_month[m]["tgt_elig"] += 1
                    by_repo[r]["tgt_elig"] += 1
                    by_delegation[d]["tgt_elig"] += 1
                    if "primary_goal" not in unk:
                        target_resolved_no_primary += 1
                    if unk <= {"success_conditions", "deliverables"} and unk:
                        blocked_only_by_sc_del += 1
            for s in cp.get("drift_scores", []):
                if s.get("class") == "semantic_goal_drift":
                    if s.get("flagged"):
                        sgd_flagged += 1
                        by_delegation[d]["sgd"] += 1
                    for e in s.get("evidence", []):
                        rr = e.get("reason", "")
                        if rr.startswith("rolling semantic goal drift"):
                            rolling_lines += 1
                        elif rr.startswith("semantic goal drift kickoff anchor"):
                            kickoff_lines += 1

    pairs_both_tgt = same_target = changed_target = changed_disjoint = 0

    def tgt_elig(cp: dict) -> bool:
        so = cp.get("structured_objective") or {}
        unk = {u.get("field_name") for u in so.get("unknowns", [])}
        return (so.get("objective_class") == "task_statement"
                and conf_ge_medium(so.get("confidence"))
                and so.get("target") is not None and "target" not in unk)

    for sid, cps in by_session.items():
        for a, b in zip(cps, cps[1:]):
            if tgt_elig(a) and tgt_elig(b):
                pairs_both_tgt += 1
                ta = target_terms(a.get("structured_objective") or {})
                tb = target_terms(b.get("structured_objective") or {})
                if ta == tb:
                    same_target += 1
                else:
                    changed_target += 1
                    if ta and tb and ta.isdisjoint(tb):
                        changed_disjoint += 1
                        by_delegation[a.get("_delegation", "unknown")]["disjoint"] += 1

    def pct(a, b):
        return f"{100 * a / b:.1f}%" if b else "n/a"

    print("=" * 70)
    print("BATCH SCAN — COVERAGE DIAGNOSTIC")
    print("=" * 70)
    print(f"sessions analyzed:        {tot_sessions}")
    print(f"total checkpoints:        {tot_cp}")
    print(f"TaskStatement:            {ts}  ({pct(ts, tot_cp)})")
    print(f"TaskStatement + conf>=Med:{ts_conf}  ({pct(ts_conf, tot_cp)})")
    print(f"CURRENT-BAR eligible (TS+conf+no-unknowns): {cur_eligible}  "
          f"({pct(cur_eligible, tot_cp)} of all cp, {pct(cur_eligible, ts_conf)} of TS+conf)")
    print()
    print("ACTUAL FIRING:")
    print(f"  semantic_goal_drift FLAGGED checkpoints: {sgd_flagged}")
    print(f"  rolling evidence lines:                  {rolling_lines}")
    print(f"  kickoff-anchor evidence lines:           {kickoff_lines}")
    print()
    print("ELIGIBILITY-FAILURE BREAKDOWN (among TS+conf>=Med with >=1 unknown):")
    for f, c in fail_reason.most_common():
        print(f"  blocked-by {f:18s}: {c}")
    print()
    print("HYPOTHETICAL TARGET-RESOLVED BAR (TS + conf>=Med + grounded target, ignore sc/deliverables):")
    print(f"  target-resolved eligible:            {target_resolved}  ({pct(target_resolved, tot_cp)} of all cp)")
    print(f"  ...also no primary_goal unknown:     {target_resolved_no_primary}")
    print(f"  checkpoints unlocked ONLY by dropping success_conditions/deliverables gate: {blocked_only_by_sc_del}")
    print(f"  net gain vs current bar:             {target_resolved - cur_eligible} more eligible checkpoints")
    print()
    print("ADJACENT-PAIR ANALYSIS under target-resolved bar (rolling's real opportunity):")
    print(f"  adjacent pairs both target-eligible: {pairs_both_tgt}")
    print(f"    same target:                       {same_target}")
    print(f"    changed target:                    {changed_target}")
    print(f"    changed AND term-disjoint (would fire rolling if not sanctioned): {changed_disjoint}")
    print("  (note: sanctioned_replan is not exported per-checkpoint, so a disjoint pair that is an")
    print("   authorized replan cannot be distinguished here; treat 'changed_disjoint' as an upper bound.)")
    print()
    print("BY MONTH (temporal diversity):")
    print("  month   | cp  | TS+conf | cur-elig | tgt-elig")
    for m in sorted(by_month):
        d = by_month[m]
        print(f"  {m} | {d['cp']:3d} | {d['ts_conf']:6d}  | {d['cur_elig']:7d}  | {d['tgt_elig']}")
    print()
    print(f"BY REPO (diversity): {len(by_repo)} distinct repos")
    top = sorted(by_repo.items(), key=lambda kv: -kv[1]["cp"])[:12]
    print("  repo                 | cp  | cur-elig | tgt-elig")
    for r, d in top:
        print(f"  {r[:20]:20s} | {d['cp']:3d} | {d['cur_elig']:7d}  | {d['tgt_elig']}")
    print()
    print("BY DELEGATION CATEGORY (R6-3.5 stratification — do NOT treat delegated/opaque traces as")
    print("equal-weight proof of scorer precision; session-level marker heuristic, coarser than the")
    print("analyzer's per-checkpoint DelegationContext):")
    print("  category                 | cp  | cur-elig | tgt-elig | sgd-fired | disjoint-pairs")
    order = ["single_agent", "delegated_parent_opaque", "delegated_child_visible", "unknown"]
    for cat in order + [c for c in sorted(by_delegation) if c not in order]:
        if cat not in by_delegation:
            continue
        d = by_delegation[cat]
        print(f"  {cat:24s} | {d['cp']:3d} | {d['cur_elig']:7d}  | {d['tgt_elig']:7d}  | "
              f"{d['sgd']:8d}  | {d['disjoint']}")


if __name__ == "__main__":
    main()
