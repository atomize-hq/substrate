#!/usr/bin/env python3
"""Coverage-diagnostic tabulation of the batch scan.

Step 3 of the semantic-goal-drift real-world validation pipeline
(see README.md). Reproduces the F1-F4 coverage metrics: current-bar
eligibility, actual firings, eligibility-failure breakdown, the
hypothetical target-resolved bar, adjacent-pair (rolling) analysis,
per-month / per-repo diversity, and best-effort validation strata.

R6-3.6 note: this script now prints an explicit eligibility/firing
funnel and clearly labels which suppression buckets are NOT derivable
from the current checkpoint export without duplicating Rust scorer logic
or widening the export surface.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import re
from collections import Counter, defaultdict

from run_batch import checkpoint_set_summary

# objective_class / target-class enum tokens that are not grounded artifacts
GENERIC = {"implement", "debug", "review", "research", "plan", "validate", "docs", "other_task",
           "repo_slice", "crate_or_package", "file_or_directory", "spec_or_design_doc",
           "test_or_verifier", "skill_or_instruction_surface", "external_artifact",
           "conceptual_topic", "unknown_target", "green"}
EXT = {"md", "rs", "json", "html", "py", "ts", "tsx", "js", "jsx", "toml", "yaml", "yml",
       "txt", "css", "sh", "lock", "cfg", "ini", "tsv", "csv", "sql", "proto", "rb", "go",
       "java", "kt", "c", "h", "cpp", "hpp", "xml", "svg", "png", "jpg", "mdx", "env"}
MODEL = re.compile(r"^(gpt|claude|gemini|llama|mistral|opus|sonnet|haiku|qwen|deepseek|grok|o1|o3)[_-]?\d")
RUST_HINTS = {"cargo", "cargo.toml", "cargo.lock", "clippy", "rustfmt", "rustc"}
NODE_HINTS = {"node", "npm", "npx", "pnpm", "yarn", "vitest", "jest", "tsx", "tsc", "vite"}
PYTHON_HINTS = {"python", "pytest", "uv", "pip", "poetry"}
DOC_HINTS = {"docs", "readme", "spec", "specs", "plan", "plans", "task", "tasks", "findings", "map"}
DOC_EXT = {"md", "mdx", "txt", "rst", "adoc"}
RUST_EXT = {"rs", "toml", "lock"}
JS_TS_EXT = {"js", "jsx", "ts", "tsx", "mjs", "cjs"}
PYTHON_EXT = {"py"}
FORBIDDEN_RECEIPT_KEYS = {
    "path",
    "relative_path",
    "cwd",
    "repo",
    "session_id",
    "source_file_id",
    "message",
    "text",
    "selected_ids",
    "repositories",
}


def metric_bucket() -> dict[str, int]:
    return {"cp": 0, "cur_elig": 0, "tgt_elig": 0, "sgd": 0, "disjoint": 0}


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


def toks(term: str) -> list[str]:
    return [t for t in term.split("_") if t]


def has_ext(term: str) -> bool:
    return any(t in EXT for t in toks(term))


def is_junk(term: str) -> bool:
    ts = toks(term)
    if not ts:
        return True
    if "n" in ts:
        return True
    if max((sum(c.isalpha() for c in t) for t in ts), default=0) < 3:
        return True
    if MODEL.match(term) and not has_ext(term):
        return True
    if ts[-1] == "etc":
        return True
    return False


def stable_target_proxy_terms(so: dict) -> set[str]:
    return {term for term in target_terms(so) if not is_junk(term)}


def conf_ge_medium(c) -> bool:
    return c in ("medium", "high")


def structured_target_values(so: dict | None) -> list[str]:
    target = (so or {}).get("target") or {}
    values: list[str] = []
    for key in ("display", "paths", "symbols", "named_artifacts", "workspace_refs"):
        value = target.get(key)
        if isinstance(value, list):
            values.extend(v for v in value if v)
        elif value:
            values.append(value)
    return values


def task_frame_values(cp: dict) -> list[str]:
    frame = cp.get("task_frame") or {}
    values: list[str] = []
    for key in ("working_set_paths", "tools", "command_families", "verification_commands"):
        value = frame.get(key)
        if isinstance(value, list):
            values.extend(v for v in value if v)
        elif value:
            values.append(value)
    return values


def checkpoint_values(cp: dict) -> list[str]:
    so = cp.get("structured_objective") or {}
    values = task_frame_values(cp) + structured_target_values(so)
    primary_intent = so.get("primary_intent")
    if primary_intent:
        values.append(str(primary_intent))
    return values


def checkpoint_terms(cp: dict) -> set[str]:
    terms: set[str] = set()
    for value in checkpoint_values(cp):
        n = norm(str(value))
        if n:
            terms.add(n)
            terms.update(toks(n))
    return terms


def checkpoint_extensions(cp: dict) -> set[str]:
    exts: set[str] = set()
    for term in checkpoint_terms(cp):
        term_toks = toks(term)
        if term_toks and term_toks[-1] in EXT:
            exts.add(term_toks[-1])
    return exts


def infer_language_repo_type(cp: dict) -> str:
    terms = checkpoint_terms(cp)
    exts = checkpoint_extensions(cp)
    families = set()
    if exts & RUST_EXT or terms & RUST_HINTS:
        families.add("rust")
    if exts & JS_TS_EXT or terms & NODE_HINTS:
        families.add("js_ts")
    if exts & PYTHON_EXT or terms & PYTHON_HINTS:
        families.add("python")
    if len(families) > 1:
        return "mixed"
    if len(families) == 1:
        return next(iter(families))
    if exts and exts <= DOC_EXT:
        return "docs_only"
    if terms & DOC_HINTS:
        return "docs_only"
    return "unknown"


def infer_workflow_type(cp: dict) -> str:
    so = cp.get("structured_objective") or {}
    archetype = ((cp.get("session_archetype") or {}).get("label") or "").lower()
    progress = ((cp.get("session_progress") or {}).get("dimension") or "").lower()
    intent = (so.get("primary_intent") or "").lower()
    candidates = set()

    archetype_map = {
        "planning": "docs_planning",
        "verification_closeout": "verification",
        "autonomous_implementation": "implementation",
        "troubleshooting": "review_fix",
    }
    progress_map = {
        "planning_convergence": "docs_planning",
        "verification_closeout_narrowing": "verification",
        "implementation_verification_wall": "implementation",
        "troubleshooting_frontier": "review_fix",
        "parent_visible_orchestration": "mixed",
    }
    intent_map = {
        "implement": "implementation",
        "validate": "verification",
        "debug": "review_fix",
        "review": "review_fix",
        "plan": "docs_planning",
        "docs": "docs_planning",
        "research": "docs_planning",
    }

    for source, mapping in ((archetype, archetype_map), (progress, progress_map), (intent, intent_map)):
        label = mapping.get(source)
        if label:
            candidates.add(label)

    if not candidates:
        verification_commands = (cp.get("task_frame") or {}).get("verification_commands") or []
        if verification_commands:
            return "verification"
        return "unknown"
    if "mixed" in candidates or len(candidates) > 1:
        return "mixed"
    return next(iter(candidates))


def infer_tooling_type(cp: dict) -> str:
    terms = checkpoint_terms(cp)
    exts = checkpoint_extensions(cp)
    scores = {
        "cargo_rust": len(terms & RUST_HINTS) + len(exts & {"rs"}),
        "node_npm": len(terms & NODE_HINTS) + len(exts & JS_TS_EXT),
        "python_pytest": len(terms & PYTHON_HINTS) + len(exts & PYTHON_EXT),
    }
    best_label = max(scores, key=scores.get)
    best_score = scores[best_label]
    if best_score > 0:
        if list(scores.values()).count(best_score) > 1:
            return "unknown"
        return best_label
    if exts and exts <= DOC_EXT:
        return "generic_filesystem_doc"
    if terms & DOC_HINTS:
        return "generic_filesystem_doc"
    if checkpoint_values(cp):
        return "generic_filesystem_doc"
    return "unknown"


def print_stratum_table(title: str, rows: dict, order: list[str], source_note: str, total_cp: int) -> None:
    print(title)
    print(f"  heuristic source: {source_note}")
    print("  category                 | cp  | cur-elig | tgt-elig | sgd-fired | disjoint-pairs")
    for cat in order + [c for c in sorted(rows) if c not in order]:
        if cat not in rows:
            continue
        data = rows[cat]
        print(f"  {cat:24s} | {data['cp']:3d} | {data['cur_elig']:7d}  | {data['tgt_elig']:7d}  | "
              f"{data['sgd']:8d}  | {data['disjoint']}")
    unknown_cp = rows.get("unknown", {}).get("cp", 0)
    if total_cp and unknown_cp == total_cp:
        print("  UNRESOLVED: this stratum remains 100% unknown under the current export/tooling and")
        print("  does not satisfy the packet's validation-strata gate.")
    print()


def validate_sanitized_receipt(value, key_path=()) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in FORBIDDEN_RECEIPT_KEYS:
                joined = ".".join((*key_path, key))
                raise ValueError(f"forbidden private key in receipt: {joined}")
            validate_sanitized_receipt(child, (*key_path, key))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            validate_sanitized_receipt(child, (*key_path, str(index)))
    elif isinstance(value, str):
        windows_path = (
            len(value) >= 3
            and value[0].isalpha()
            and value[1:3] in (":\\", ":/")
        )
        if value.startswith("/") or "~/.codex" in value or windows_path:
            joined = ".".join(key_path)
            raise ValueError(f"absolute private path in receipt: {joined}")


def validate_successful_batch_receipt(batch_receipt: dict) -> None:
    if batch_receipt.get("schema_version") != "p7-private-batch-v1":
        raise ValueError("batch receipt has the wrong schema_version")
    batch = batch_receipt.get("batch")
    if not isinstance(batch, dict):
        raise ValueError("batch receipt is missing batch counts")
    attempted = batch.get("attempted_count")
    succeeded = batch.get("succeeded_count")
    failed = batch.get("failed_count")
    if not all(isinstance(value, int) for value in (attempted, succeeded, failed)):
        raise ValueError("batch receipt has invalid batch counts")
    if attempted < 1:
        raise ValueError("batch receipt must prove a non-empty execution")
    if failed != 0 or succeeded != attempted:
        raise ValueError("batch receipt contains failed sessions")
    checkpoint_set = batch_receipt.get("checkpoint_set")
    if not isinstance(checkpoint_set, dict):
        raise ValueError("batch receipt is missing checkpoint set authority")
    if checkpoint_set.get("schema_version") != 1:
        raise ValueError("checkpoint set has the wrong schema_version")
    if checkpoint_set.get("file_count") != succeeded:
        raise ValueError("checkpoint set file count does not match successful sessions")


def validate_batch_inputs(
    batch_receipt: dict, checkpoints_dir: str | os.PathLike[str]
) -> dict:
    validate_sanitized_receipt(batch_receipt)
    validate_successful_batch_receipt(batch_receipt)
    actual = checkpoint_set_summary(checkpoints_dir)
    if actual != batch_receipt["checkpoint_set"]:
        raise ValueError("checkpoint set does not match batch receipt")
    return actual


def build_sanitized_receipt(
    batch_receipt: dict,
    *,
    analysis_coverage: dict,
    heuristic_strata: dict,
) -> dict:
    validate_successful_batch_receipt(batch_receipt)
    if (
        analysis_coverage.get("sessions_analyzed")
        != batch_receipt["batch"]["succeeded_count"]
    ):
        raise ValueError("analysis session count does not match batch receipt")
    if (
        analysis_coverage.get("total_checkpoints")
        != batch_receipt["checkpoint_set"]["checkpoint_count"]
    ):
        raise ValueError("analysis checkpoint count does not match checkpoint set")
    receipt = {
        "schema_version": "p7-private-recall-validation-v1",
        "selection": batch_receipt["selection"],
        "batch": batch_receipt["batch"],
        "checkpoint_set": batch_receipt["checkpoint_set"],
        "analysis_coverage": analysis_coverage,
        "heuristic_strata": heuristic_strata,
        "limitations": {
            "scorer_internal_claims": [
                "structural_containment_not_derived",
                "stable_target_hygiene_not_derived",
                "sanctioned_replan_not_derived",
            ],
            "heuristic_strata_are_distribution_observations_only": True,
        },
        "privacy": {
            "raw_private_fields_included": False,
        },
    }
    validate_sanitized_receipt(receipt)
    return receipt


def sanitized_metric_rows(rows: dict) -> dict:
    return {
        category: {
            "checkpoints": metrics["cp"],
            "current_bar_eligible": metrics["cur_elig"],
            "target_resolved_eligible": metrics["tgt_elig"],
            "semantic_goal_drift_fires": metrics["sgd"],
            "disjoint_pairs_upper_bound": metrics["disjoint"],
        }
        for category, metrics in sorted(rows.items())
    }


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--checkpoints-dir", default=os.path.join("batch", "checkpoints"),
                    help="dir of per-session checkpoints from run_batch.py "
                         "(default: ./batch/checkpoints)")
    ap.add_argument("--batch-receipt", default=None,
                    help="sanitized receipt from run_batch.py "
                         "(default: sibling batch_receipt.json)")
    ap.add_argument("--receipt-out", default=None,
                    help="privacy-safe aggregate receipt "
                         "(default: sibling p7_private_receipt.json)")
    args = ap.parse_args()
    batch_dir = os.path.dirname(os.path.abspath(args.checkpoints_dir))
    batch_receipt_path = args.batch_receipt or os.path.join(
        batch_dir, "batch_receipt.json"
    )
    receipt_out = args.receipt_out or os.path.join(
        batch_dir, "p7_private_receipt.json"
    )
    with open(batch_receipt_path) as handle:
        batch_receipt = json.load(handle)
    validate_batch_inputs(batch_receipt, args.checkpoints_dir)

    by_session = defaultdict(list)
    seen_session_ids = set()
    for fp in sorted(glob.glob(os.path.join(args.checkpoints_dir, "*.jsonl"))):
        file_session_ids = set()
        for line in open(fp):
            if not line.strip():
                continue
            cp = json.loads(line)
            session_id = cp["_session_file_id"]
            file_session_ids.add(session_id)
            by_session[session_id].append(cp)
        if len(file_session_ids) != 1:
            raise ValueError("checkpoint file must contain exactly one session")
        session_id = next(iter(file_session_ids))
        if session_id in seen_session_ids:
            raise ValueError("checkpoint session appears in more than one file")
        seen_session_ids.add(session_id)
    for sid in by_session:
        by_session[sid].sort(key=lambda c: c.get("ordinal", 0))
    if len(by_session) != batch_receipt["batch"]["succeeded_count"]:
        raise ValueError("checkpoint session count does not match batch receipt")

    tot_cp = 0
    tot_sessions = len(by_session)
    ts = ts_conf = cur_eligible = 0
    structured_present = structured_target_present = stable_target_proxy = 0
    fail_reason = Counter()
    target_resolved = target_resolved_no_primary = blocked_only_by_sc_del = 0
    sgd_flagged = rolling_lines = kickoff_lines = 0
    by_month = defaultdict(lambda: {"cp": 0, "ts_conf": 0, "cur_elig": 0, "tgt_elig": 0})
    by_repo = defaultdict(lambda: {"cp": 0, "cur_elig": 0, "tgt_elig": 0})
    # R6-3.5 delegation stratification: report precision-relevant metrics separately per delegation
    # category so single-agent and delegated/opaque traces are not treated as equal-weight evidence.
    by_delegation = defaultdict(metric_bucket)
    by_language = defaultdict(metric_bucket)
    by_workflow = defaultdict(metric_bucket)
    by_tooling = defaultdict(metric_bucket)

    for sid, cps in by_session.items():
        for cp in cps:
            tot_cp += 1
            m = cp["_month"]
            r = cp["_repo"]
            d = cp.get("_delegation", "unknown")
            lang = infer_language_repo_type(cp)
            workflow = infer_workflow_type(cp)
            tooling = infer_tooling_type(cp)
            by_month[m]["cp"] += 1
            by_repo[r]["cp"] += 1
            by_delegation[d]["cp"] += 1
            by_language[lang]["cp"] += 1
            by_workflow[workflow]["cp"] += 1
            by_tooling[tooling]["cp"] += 1
            so = cp.get("structured_objective") or {}
            oc = so.get("objective_class")
            conf = so.get("confidence")
            unk = {u.get("field_name") for u in so.get("unknowns", [])}
            tgt_present = so.get("target") is not None
            if so:
                structured_present += 1
            if tgt_present:
                structured_target_present += 1
            if stable_target_proxy_terms(so):
                stable_target_proxy += 1
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
                    by_language[lang]["cur_elig"] += 1
                    by_workflow[workflow]["cur_elig"] += 1
                    by_tooling[tooling]["cur_elig"] += 1
                else:
                    for f in unk:
                        fail_reason[f] += 1
                if tgt_present and "target" not in unk:
                    target_resolved += 1
                    by_month[m]["tgt_elig"] += 1
                    by_repo[r]["tgt_elig"] += 1
                    by_delegation[d]["tgt_elig"] += 1
                    by_language[lang]["tgt_elig"] += 1
                    by_workflow[workflow]["tgt_elig"] += 1
                    by_tooling[tooling]["tgt_elig"] += 1
                    if "primary_goal" not in unk:
                        target_resolved_no_primary += 1
                    if unk <= {"success_conditions", "deliverables"} and unk:
                        blocked_only_by_sc_del += 1
            for s in cp.get("drift_scores", []):
                if s.get("class") == "semantic_goal_drift":
                    if s.get("flagged"):
                        sgd_flagged += 1
                        by_delegation[d]["sgd"] += 1
                        by_language[lang]["sgd"] += 1
                        by_workflow[workflow]["sgd"] += 1
                        by_tooling[tooling]["sgd"] += 1
                    for e in s.get("evidence", []):
                        rr = e.get("reason", "")
                        if rr.startswith("rolling semantic goal drift"):
                            rolling_lines += 1
                        elif rr.startswith("semantic goal drift kickoff anchor"):
                            kickoff_lines += 1

    adjacent_pairs_total = 0
    pairs_both_tgt = same_target = changed_target = changed_disjoint = 0

    def tgt_elig(cp: dict) -> bool:
        so = cp.get("structured_objective") or {}
        unk = {u.get("field_name") for u in so.get("unknowns", [])}
        return (so.get("objective_class") == "task_statement"
                and conf_ge_medium(so.get("confidence"))
                and so.get("target") is not None and "target" not in unk)

    for sid, cps in by_session.items():
        for a, b in zip(cps, cps[1:]):
            adjacent_pairs_total += 1
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
                        a_delegation = a.get("_delegation", "unknown")
                        by_delegation[a_delegation]["disjoint"] += 1
                        by_language[infer_language_repo_type(a)]["disjoint"] += 1
                        by_workflow[infer_workflow_type(a)]["disjoint"] += 1
                        by_tooling[infer_tooling_type(a)]["disjoint"] += 1

    def pct(a, b):
        return f"{100 * a / b:.1f}%" if b else "n/a"

    print("=" * 70)
    print("BATCH SCAN — COVERAGE DIAGNOSTIC")
    print("=" * 70)
    print(f"sessions analyzed:        {tot_sessions}")
    print(f"total checkpoints:        {tot_cp}")
    print(f"checkpoints with structured_objective: {structured_present}")
    print(f"checkpoints with structured target:    {structured_target_present}")
    print(f"stable-target proxy (analysis only):   {stable_target_proxy}")
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
    print("ELIGIBILITY / FIRING FUNNEL (available directly from current export + analysis-only target proxy):")
    print(f"  total sessions:                      {tot_sessions}")
    print(f"  total checkpoints:                   {tot_cp}")
    print(f"  structured_objective present:        {structured_present}")
    print(f"  structured target present:           {structured_target_present}")
    print(f"  stable-target proxy present:         {stable_target_proxy}")
    print(f"  current-bar eligible:                {cur_eligible}")
    print(f"  target-resolved eligible:            {target_resolved}")
    print(f"  adjacent checkpoint pairs total:     {adjacent_pairs_total}")
    print(f"  adjacent pairs both target-eligible: {pairs_both_tgt}")
    print(f"  same-target exact-match suppressions:{same_target}")
    print(f"  changed-target candidate pairs:      {changed_target}")
    print(f"  remaining disjoint pairs:            {changed_disjoint}")
    print(f"  emitted semantic_goal_drift fires:   {sgd_flagged}")
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
    print(f"BY REPO (privacy-safe diversity): {len(by_repo)} distinct repos")
    top = sorted(by_repo.items(), key=lambda kv: -kv[1]["cp"])[:12]
    print("  anonymized rank | cp  | cur-elig | tgt-elig")
    for rank, (_, d) in enumerate(top, 1):
        print(f"  repo-rank-{rank:02d}    | {d['cp']:3d} | {d['cur_elig']:7d}  | {d['tgt_elig']}")
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
    print()
    print_stratum_table(
        "BY LANGUAGE / REPO TYPE (best-effort checkpoint heuristic):",
        by_language,
        ["rust", "js_ts", "python", "docs_only", "mixed", "unknown"],
        "task_frame.working_set_paths + structured_objective.target values, with command/tool and file-extension hints",
        tot_cp,
    )
    print_stratum_table(
        "BY WORKFLOW TYPE (best-effort checkpoint heuristic):",
        by_workflow,
        ["implementation", "docs_planning", "verification", "review_fix", "mixed", "unknown"],
        "session_archetype.label + session_progress.dimension, with structured_objective.primary_intent / verification-command fallback",
        tot_cp,
    )
    print_stratum_table(
        "BY TOOLING TYPE (best-effort checkpoint heuristic):",
        by_tooling,
        ["cargo_rust", "node_npm", "python_pytest", "generic_filesystem_doc", "unknown"],
        "task_frame.command_families + tools + verification_commands, with file-extension fallback when commands are absent",
        tot_cp,
    )
    print("NOT DERIVABLE FROM CURRENT EXPORT WITHOUT DUPLICATING RUST SCORER LOGIC OR WIDENING EXPORT:")
    print("  - structural-containment suppressions")
    print("  - scorer-true stable-target-hygiene suppressions (the 'stable-target proxy' above is analysis-only)")
    print("  - sanctioned_replan suppressions")
    receipt = build_sanitized_receipt(
        batch_receipt,
        analysis_coverage={
            "sessions_analyzed": tot_sessions,
            "total_checkpoints": tot_cp,
            "structured_objective_present": structured_present,
            "structured_target_present": structured_target_present,
            "stable_target_proxy_present": stable_target_proxy,
            "current_bar_eligible": cur_eligible,
            "target_resolved_eligible": target_resolved,
            "adjacent_pairs_total": adjacent_pairs_total,
            "target_resolved_adjacent_pairs": pairs_both_tgt,
            "same_target_suppressions": same_target,
            "changed_target_pairs": changed_target,
            "disjoint_pairs_upper_bound": changed_disjoint,
            "semantic_goal_drift_fires": sgd_flagged,
        },
        heuristic_strata={
            "language_repo": sanitized_metric_rows(by_language),
            "workflow": sanitized_metric_rows(by_workflow),
            "tooling": sanitized_metric_rows(by_tooling),
            "delegation": sanitized_metric_rows(by_delegation),
        },
    )
    with open(receipt_out, "w") as handle:
        json.dump(receipt, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print(f"\nprivacy-safe aggregate receipt: {receipt_out}")


if __name__ == "__main__":
    main()
