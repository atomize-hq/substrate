#!/usr/bin/env python3
"""Analysis-only junk-target filter over the existing batch data.

Step 4 (cheapest gate) of the semantic-goal-drift real-world validation
pipeline (see README.md). This is the "analysis-only junk-target filter
over the existing batch data before writing extraction code" from the
FINDINGS charter's Step 2. It suppresses obvious extraction garbage at the
*term* level and re-counts how many of

  - the actual fires
  - the current-bar-eligible checkpoints (which rested on junk-only targets)
  - the target-resolved disjoint adjacent pairs

survive. Survivors that are NOT garbage are the graduated-distance /
real-change residue that would justify opening R6-3.X.2.

The junk categories match the charter's named extraction defects:
  R1 escaped-newline residue  -> a standalone single 'n' token (from literal \\n)
  R2 number/coordinate run     -> no token carries >=3 alphabetic chars
  R3 bare model/version token  -> GPT-5.4 / claude-3 / ... with no file extension
  R4 prose-enumeration marker  -> trailing 'etc' ("knobs/envs/cli/etc")

NOTE: this term-level heuristic is deliberately conservative. It is a gate,
not the production extractor; a couple of prose fragments ("closeout/
review-ready", "linux/mac/windows") slip through and are hand-annotated in
the FINDINGS doc as residual extraction gaps (still extraction's job, not
the distance metric's).
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import re
from collections import defaultdict

GENERIC = {"implement", "debug", "review", "research", "plan", "validate", "docs", "other_task",
           "repo_slice", "crate_or_package", "file_or_directory", "spec_or_design_doc",
           "test_or_verifier", "skill_or_instruction_surface", "external_artifact",
           "conceptual_topic", "unknown_target", "green"}

EXT = {"md", "rs", "json", "html", "py", "ts", "tsx", "js", "jsx", "toml", "yaml", "yml",
       "txt", "css", "sh", "lock", "cfg", "ini", "tsv", "csv", "sql", "proto", "rb", "go",
       "java", "kt", "c", "h", "cpp", "hpp", "xml", "svg", "png", "jpg", "mdx", "env"}
MODEL = re.compile(r"^(gpt|claude|gemini|llama|mistral|opus|sonnet|haiku|qwen|deepseek|grok|o1|o3)[_-]?\d")


def norm(raw: str) -> str:
    return "".join(c.lower() if c.isalnum() else "_" for c in raw).strip("_")


def toks(term: str) -> list[str]:
    return [t for t in term.split("_") if t]


def has_ext(term: str) -> bool:
    return any(t in EXT for t in toks(term))


def is_junk(term: str) -> bool:
    """True if a normalized target term is extraction garbage (not a grounded artifact)."""
    ts = toks(term)
    if not ts:
        return True
    if "n" in ts:                                                   # R1 escaped-newline residue
        return True
    if max((sum(c.isalpha() for c in t) for t in ts), default=0) < 3:  # R2 number/coordinate run
        return True
    if MODEL.match(term) and not has_ext(term):                     # R3 bare model/version token
        return True
    if ts[-1] == "etc":                                             # R4 prose-enumeration marker
        return True
    return False


def raw_target_values(so: dict):
    t = so.get("target") or {}
    vals = []
    for k in ("display", "paths", "symbols", "named_artifacts", "workspace_refs"):
        v = t.get(k)
        if isinstance(v, list):
            vals += [x for x in v if x]
        elif v:
            vals.append(v)
    return vals


def all_terms(so: dict) -> set[str]:
    out = set()
    for v in raw_target_values(so):
        n = norm(v)
        if n and n not in GENERIC:
            out.add(n)
    return out


def stable_terms(so: dict) -> set[str]:
    return {t for t in all_terms(so) if not is_junk(t)}


def conf_ge_medium(c) -> bool:
    return c in ("medium", "high")


def tgt_elig(cp: dict) -> bool:
    so = cp.get("structured_objective") or {}
    unk = {u.get("field_name") for u in so.get("unknowns", [])}
    return (so.get("objective_class") == "task_statement" and conf_ge_medium(so.get("confidence"))
            and so.get("target") is not None and "target" not in unk)


def cur_elig(cp: dict) -> bool:
    so = cp.get("structured_objective") or {}
    unk = {u.get("field_name") for u in so.get("unknowns", [])}
    return (so.get("objective_class") == "task_statement"
            and conf_ge_medium(so.get("confidence")) and not unk)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
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

    fires_total = fires_survive = 0
    for sid, cps in by_session.items():
        for cp in cps:
            for s in cp.get("drift_scores", []):
                if s.get("class") == "semantic_goal_drift" and s.get("flagged"):
                    fires_total += 1
                    if stable_terms(cp.get("structured_objective") or {}):
                        fires_survive += 1

    cur_total = cur_junk_only = 0
    for sid, cps in by_session.items():
        for cp in cps:
            if cur_elig(cp):
                cur_total += 1
                so = cp.get("structured_objective") or {}
                if all_terms(so) and not stable_terms(so):
                    cur_junk_only += 1

    base_disjoint = []
    stable_disjoint = []
    for sid, cps in by_session.items():
        for a, b in zip(cps, cps[1:]):
            if tgt_elig(a) and tgt_elig(b):
                soa = a.get("structured_objective") or {}
                sob = b.get("structured_objective") or {}
                aa, ab = all_terms(soa), all_terms(sob)
                if aa and ab and aa != ab and aa.isdisjoint(ab):
                    base_disjoint.append((sid, a, b))
                sa, sb = stable_terms(soa), stable_terms(sob)
                if sa and sb and sa != sb and sa.isdisjoint(sb):
                    stable_disjoint.append((sid, a, b, sa, sb))

    def pct(a, b):
        return f"{100 * a / b:.1f}%" if b else "n/a"

    print("=" * 74)
    print("JUNK-TARGET FILTER — cheapest-gate re-count (analysis only, no code changed)")
    print("=" * 74)
    print(f"\n[1] ACTUAL FIRES: {fires_survive}/{fires_total} survive junk suppression "
          f"({pct(fires_survive, fires_total)} retained)")
    print(f"\n[2] CURRENT-BAR ELIGIBLE resting on junk-only targets: {cur_junk_only}/{cur_total} "
          f"({pct(cur_junk_only, cur_total)}) had a target term set that is 100% garbage")
    print("\n[3] TARGET-RESOLVED DISJOINT ADJACENT PAIRS:")
    print(f"    baseline (all terms):        {len(base_disjoint)}")
    print(f"    survive stable-only filter:  {len(stable_disjoint)}  "
          f"(killed by junk suppression: {len(base_disjoint) - len(stable_disjoint)})")
    print("\n    Surviving pairs (stable terms only) — hand-verify category:")
    for sid, a, b, sa, sb in stable_disjoint:
        print(f"      · {sid[:12]} {a['_repo']}/{a['_month']} ord {a.get('ordinal')}->{b.get('ordinal')}")
        print(f"          A: {sorted(sa)}")
        print(f"          B: {sorted(sb)}")


if __name__ == "__main__":
    main()
