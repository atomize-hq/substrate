#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Optional

ADR_PATH_RE = re.compile(r"docs/project_management/adrs/[^\s`]+ADR-\d{4}[^\s`]*\.md")
LEGACY_TRIAD_FOLLOWUP_RE = re.compile(
    r"\b(tasks\.json|kickoff prompts?|task graph|checkpoint task|meta\.checkpoint_boundaries|full[- ]planning)\b",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class SourceRef:
    path: str
    line: int

    def fmt(self) -> str:
        return f"`{self.path}#L{self.line}`"


@dataclass
class Item:
    kind: str
    title: str
    sources: list[SourceRef] = field(default_factory=list)


def repo_root() -> Path:
    return Path(
        subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
    )


def read_lines(path: Path) -> Optional[list[str]]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return None


def read_json(path: Path) -> Optional[dict]:
    try:
        with path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError:
        return None
    except (OSError, json.JSONDecodeError, TypeError, ValueError):
        return None
    if isinstance(data, dict):
        return data
    return None


def resolve_pack_doc(feature_dir_abs: Path, feature_dir_rel: str, filename: str) -> tuple[Path, str]:
    """
    Resolve a canonical pack artifact path.

    Preferred location (new layout):
      <FEATURE_DIR>/pre-planning/<filename>

    Legacy fallback (older packs; compatibility only):
      <FEATURE_DIR>/<filename>

    Returns (absolute_path, repo_relative_path_str).
    """
    feature_dir_rel = feature_dir_rel.rstrip("/") + "/"

    preferred_abs = (feature_dir_abs / "pre-planning" / filename).resolve()
    preferred_rel = f"{feature_dir_rel}pre-planning/{filename}"

    legacy_abs = (feature_dir_abs / filename).resolve()
    legacy_rel = f"{feature_dir_rel}{filename}"

    if preferred_abs.exists():
        return preferred_abs, preferred_rel
    if legacy_abs.exists():
        return legacy_abs, legacy_rel
    return preferred_abs, preferred_rel


def find_heading(lines: list[str], predicate) -> Optional[int]:
    for i, line in enumerate(lines):
        if line.startswith("## "):
            if predicate(line[3:].strip()):
                return i
    return None


def find_heading_any(lines: list[str], headings: Iterable[str]) -> Optional[int]:
    expected = set(headings)
    return find_heading(lines, lambda h: h in expected)


def section_slice(lines: list[str], heading_idx: int) -> tuple[int, int]:
    start = heading_idx + 1
    end = len(lines)
    for i in range(start, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    return start, end


def normalize_key(text: str) -> str:
    t = text.lower()
    t = t.replace("`", "")
    t = re.sub(r"[^a-z0-9\s]+", " ", t)
    t = re.sub(r"\s+", " ", t).strip()
    return t


def best_kind(kinds: Iterable[str]) -> str:
    # Highest wins.
    order = ["misalignment", "gate", "checkpoint", "ci", "risk", "followup"]
    for k in order:
        if k in kinds:
            return k
    return "followup"


def extract_numbered_items(
    path_rel: str, lines: list[str], heading_predicate, number_re: re.Pattern[str]
) -> list[Item]:
    idx = find_heading(lines, heading_predicate)
    if idx is None:
        return []
    start, end = section_slice(lines, idx)
    out: list[Item] = []
    for j in range(start, end):
        m = number_re.match(lines[j])
        if not m:
            continue
        title = m.group(2).strip()
        kind = "dr" if title.startswith("DR-") else "followup"
        out.append(Item(kind=kind, title=title, sources=[SourceRef(path_rel, j + 1)]))
    return out


def extract_numbered_items_typed(
    path_rel: str,
    lines: list[str],
    heading_predicate,
    number_re: re.Pattern[str],
    *,
    kind_default: str,
    dr_prefix_ok: bool = True,
) -> list[Item]:
    idx = find_heading(lines, heading_predicate)
    if idx is None:
        return []
    start, end = section_slice(lines, idx)
    out: list[Item] = []
    for j in range(start, end):
        m = number_re.match(lines[j])
        if not m:
            continue
        title = m.group(2).strip()
        kind = "dr" if (dr_prefix_ok and title.startswith("DR-")) else kind_default
        out.append(Item(kind=kind, title=title, sources=[SourceRef(path_rel, j + 1)]))
    return out


def extract_triage_bullets(
    path_rel: str, lines: list[str], heading_text: str, kind: str
) -> list[Item]:
    idx = find_heading(lines, lambda h: h == heading_text)
    if idx is None:
        return []
    start, end = section_slice(lines, idx)
    out: list[Item] = []
    bullet_re = re.compile(r"^(\s*)-\s+(.+?)\s*$")
    for j in range(start, end):
        m = bullet_re.match(lines[j])
        if not m:
            continue
        indent = len(m.group(1))
        if indent != 0:
            # For triage sections we care about top-level bullets only.
            continue
        title = m.group(2).strip()
        out.append(Item(kind=kind, title=title, sources=[SourceRef(path_rel, j + 1)]))
    return out


def extract_gate_items(path_rel: str, lines: list[str]) -> list[Item]:
    idx = find_heading(lines, lambda h: h.startswith("Sequencing + gates"))
    if idx is None:
        return []
    start, end = section_slice(lines, idx)
    out: list[Item] = []
    gate_re = re.compile(r"^(\s*)-\s+(Gate\s+[^—]+—\s+.+?)\s*$")
    for j in range(start, end):
        m = gate_re.match(lines[j])
        if not m:
            continue
        indent = len(m.group(1))
        if indent != 0:
            continue
        out.append(Item(kind="gate", title=m.group(2).strip(), sources=[SourceRef(path_rel, j + 1)]))
    return out


def extract_impact_map_followups(path_rel: str, lines: list[str]) -> list[Item]:
    idx = find_heading(lines, lambda h: h.startswith("Follow-ups"))
    if idx is None:
        return []
    start, end = section_slice(lines, idx)

    bullet_re = re.compile(r"^(\s*)-\s+(.+?)\s*$")
    dr_re = re.compile(r"^DR-\d{4}\s+—\s+")
    path_desc_re = re.compile(r"^`([^`]+)`\s+—\s+(.+)$")
    path_ctx_re = re.compile(r"^`([^`]+)`:\s*$")
    quoted_heading_re = re.compile(r'^[“"\'`].+')

    out: list[Item] = []
    # Stack of (indent, doc_path) contexts for operator-doc targets lists.
    doc_stack: list[tuple[int, str]] = []

    for j in range(start, end):
        m = bullet_re.match(lines[j])
        if not m:
            continue

        indent = len(m.group(1))
        text = m.group(2).strip()

        while doc_stack and indent <= doc_stack[-1][0]:
            doc_stack.pop()

        if dr_re.match(text):
            out.append(Item(kind="dr", title=text, sources=[SourceRef(path_rel, j + 1)]))
            continue

        m_path_desc = path_desc_re.match(text)
        if m_path_desc:
            out.append(
                Item(
                    kind="followup",
                    title=f"{m_path_desc.group(1)} — {m_path_desc.group(2)}",
                    sources=[SourceRef(path_rel, j + 1)],
                )
            )
            continue

        m_path_ctx = path_ctx_re.match(text)
        if m_path_ctx:
            doc_stack.append((indent, m_path_ctx.group(1)))
            continue

        if doc_stack and quoted_heading_re.match(text):
            doc_path = doc_stack[-1][1]
            out.append(
                Item(
                    kind="followup",
                    title=f"{doc_path} — {text}",
                    sources=[SourceRef(path_rel, j + 1)],
                )
            )
            continue

    return out


def extract_contract_authority_selections(
    impact_map_path: Path, impact_map_rel: str
) -> list[tuple[str, str, SourceRef]]:
    lines = read_lines(impact_map_path)
    if lines is None:
        return []
    pack_re = re.compile(r"^(\s*)-\s+Planning Pack:\s+`([^`]+)`")
    selected_re = re.compile(r"^\s*-\s+Selected:\s+Option\s+([AB])\b")

    out: list[tuple[str, str, SourceRef]] = []
    current_pack: Optional[str] = None
    current_indent: Optional[int] = None

    for i, line in enumerate(lines):
        m_pack = pack_re.match(line)
        if m_pack:
            current_pack = m_pack.group(2).rstrip("/") + "/"
            current_indent = len(m_pack.group(1))
            continue

        if current_pack is None:
            continue

        # If we hit a new planning-pack entry at same or lower indent, close the block.
        if line.lstrip().startswith("- Planning Pack:"):
            indent = len(line) - len(line.lstrip())
            if current_indent is not None and indent <= current_indent:
                current_pack = None
                current_indent = None
                continue

        m_sel = selected_re.match(line)
        if m_sel:
            out.append((current_pack, m_sel.group(1), SourceRef(impact_map_rel, i + 1)))
            # Only one selection per block is expected; keep scanning but don't double-add for the same pack.
            current_pack = None
            current_indent = None

    return out


def find_first_line_containing(lines: list[str], needle: str) -> Optional[int]:
    for i, line in enumerate(lines):
        if needle in line:
            return i + 1
    return None


def extract_adr_paths_from_inputs(paths: list[tuple[Path, str]]) -> list[tuple[str, SourceRef]]:
    seen: set[str] = set()
    out: list[tuple[str, SourceRef]] = []
    for abs_path, rel_path in paths:
        lines = read_lines(abs_path)
        if lines is None:
            continue
        idx = find_heading(lines, lambda h: h == "Inputs")
        if idx is None:
            continue
        start, end = section_slice(lines, idx)
        for i in range(start, end):
            for match in ADR_PATH_RE.finditer(lines[i]):
                adr_path = match.group(0)
                if adr_path in seen:
                    continue
                seen.add(adr_path)
                out.append((adr_path, SourceRef(rel_path, i + 1)))
    return out


def extract_seed_adr_paths_from_metadata(
    feature_dir_abs: Path, feature_dir_rel: str
) -> list[tuple[str, SourceRef]]:
    metadata_rel = f"{feature_dir_rel}fse_pre_planning.json"
    metadata_abs = feature_dir_abs / "fse_pre_planning.json"
    metadata = read_json(metadata_abs)
    if metadata is None:
        return []

    adr_paths = metadata.get("adr_paths")
    if not isinstance(adr_paths, list):
        return []

    lines = read_lines(metadata_abs) or []
    out: list[tuple[str, SourceRef]] = []
    seen: set[str] = set()
    for adr_path in adr_paths:
        if not isinstance(adr_path, str):
            continue
        adr_path = adr_path.strip()
        if not adr_path or adr_path in seen:
            continue
        seen.add(adr_path)
        line_no = find_first_line_containing(lines, adr_path) or 1
        out.append((adr_path, SourceRef(metadata_rel, line_no)))
    return out


def authoritative_adr_paths(
    feature_dir_abs: Path,
    feature_dir_rel: str,
    *,
    spec_manifest_abs: Path,
    spec_manifest_rel: str,
    impact_map_abs: Path,
    impact_map_rel: str,
) -> list[tuple[str, SourceRef]]:
    seeded = extract_seed_adr_paths_from_metadata(feature_dir_abs, feature_dir_rel)
    if seeded:
        return seeded

    return extract_adr_paths_from_inputs(
        [
            (spec_manifest_abs, spec_manifest_rel),
            (impact_map_abs, impact_map_rel),
        ]
    )


def should_skip_legacy_followup(item: Item) -> bool:
    if item.kind in {"misalignment", "gate", "risk", "dr"}:
        return False
    return LEGACY_TRIAD_FOLLOWUP_RE.search(item.title) is not None


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Generate the wrapper-compiled FSE pre-planning alignment report. "
            "Pre-full/post-full convergence utilities remain compatibility-only and are not "
            "part of the active FSE pre-planning lane."
        )
    )
    parser.add_argument(
        "--feature-dir",
        required=True,
        help="FSE feature pack dir (repo-relative)",
    )
    args = parser.parse_args()

    root = repo_root()
    feature_dir_rel = args.feature_dir.rstrip("/") + "/"
    feature_dir_abs = (root / feature_dir_rel).resolve()

    items: list[Item] = []

    # Wrapper-detected misalignments.
    misalignments: list[Item] = []

    spec_manifest_abs, spec_manifest_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "spec_manifest.md")
    impact_map_abs, impact_map_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "impact_map.md")
    minimal_spec_abs, minimal_spec_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "minimal_spec_draft.md")
    ci_plan_abs, ci_plan_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "ci_checkpoint_plan.md")
    triage_abs, triage_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "workstream_triage.md")

    adr_paths = authoritative_adr_paths(
        feature_dir_abs,
        feature_dir_rel,
        spec_manifest_abs=spec_manifest_abs,
        spec_manifest_rel=spec_manifest_rel,
        impact_map_abs=impact_map_abs,
        impact_map_rel=impact_map_rel,
    )

    declared_dirs: list[tuple[str, SourceRef, SourceRef]] = []
    adr_feature_dir_re = re.compile(r"Feature directory:\s*`([^`]+)`")
    for adr_rel, discovered_from in adr_paths:
        adr_abs = (root / adr_rel).resolve()
        adr_lines = read_lines(adr_abs)
        if adr_lines is None:
            continue
        for i, line in enumerate(adr_lines):
            m = adr_feature_dir_re.search(line)
            if not m:
                continue
            declared = m.group(1).rstrip("/") + "/"
            declared_dirs.append((declared, SourceRef(adr_rel, i + 1), discovered_from))
            break

    if declared_dirs:
        # If multiple ADRs disagree, treat as drift too.
        for declared, src, discovered_from in declared_dirs:
            if declared != feature_dir_rel:
                title = f"ADR feature-dir drift: ADR declares `{declared}` but pack dir is `{feature_dir_rel}` (hard gate: reconcile to avoid dual-authority docs)."
                drift_item = Item(kind="misalignment", title=title, sources=[src, discovered_from])

                # Handoff-only critical heuristic for this drift: check if spec_manifest.md follow-ups mention it.
                spec_manifest_lines = read_lines(spec_manifest_abs) or []
                minimal_spec_lines = read_lines(minimal_spec_abs) or []

                drift_token = declared.rstrip("/")
                # Heuristic: track whether drift is called out in canonical follow-ups sections.
                spec_fu_idx = find_heading(spec_manifest_lines, lambda h: h == "Follow-ups")
                min_fu_idx = find_heading_any(
                    minimal_spec_lines,
                    (
                        "Follow-ups for full planning",
                        "Follow-ups for downstream FSE planning and decomposition",
                    ),
                )
                in_spec_followups = False
                in_min_followups = False
                if spec_fu_idx is not None:
                    s, e = section_slice(spec_manifest_lines, spec_fu_idx)
                    in_spec_followups = drift_token in "\n".join(spec_manifest_lines[s:e])
                if min_fu_idx is not None:
                    s, e = section_slice(minimal_spec_lines, min_fu_idx)
                    in_min_followups = drift_token in "\n".join(minimal_spec_lines[s:e])

                # Also locate in spec-manifest handoff if present, to ensure it doesn't get lost.
                spec_handoff_rel = f"{feature_dir_rel}logs/spec-manifest/handoff.md"
                spec_handoff_lines = read_lines(feature_dir_abs / "logs/spec-manifest/handoff.md") or []
                if spec_handoff_lines:
                    ln = find_first_line_containing(spec_handoff_lines, drift_token)
                    if ln:
                        drift_item.sources.append(SourceRef(spec_handoff_rel, ln))

                if minimal_spec_lines:
                    ln = find_first_line_containing(minimal_spec_lines, drift_token)
                    if ln:
                        drift_item.sources.append(SourceRef(minimal_spec_rel, ln))

                if not in_spec_followups:
                    drift_item.title += " (note: missing from spec_manifest.md follow-ups; ensure it stays tracked in later FSE planning)"

                if spec_handoff_lines and (not in_spec_followups and not in_min_followups):
                    drift_item.title += " (handoff-only critical)"

                misalignments.append(drift_item)

    # Cross-pack authority conflict (impact_map selection mismatch)
    impact_map_abs, impact_map_rel = resolve_pack_doc(feature_dir_abs, feature_dir_rel, "impact_map.md")
    selections = extract_contract_authority_selections(impact_map_abs, impact_map_rel)
    for other_pack, opt, src in selections:
        other_pack_rel = other_pack.rstrip("/") + "/"
        other_pack_abs = (root / other_pack_rel).resolve()
        other_impact_abs, other_impact_rel = resolve_pack_doc(other_pack_abs, other_pack_rel, "impact_map.md")
        other_selections = extract_contract_authority_selections(other_impact_abs, other_impact_rel)
        # Find reciprocal selection where referenced pack mentions this pack.
        reciprocal = next(
            (o for o in other_selections if o[0].rstrip("/") + "/" == feature_dir_rel), None
        )
        if reciprocal is None:
            continue
        _, other_opt, other_src = reciprocal
        if other_opt != opt:
            title = (
                f"Cross-pack contract authority conflict: `{feature_dir_rel}` selects Option {opt} "
                f"but `{other_pack}` selects Option {other_opt} (hard decision: converge on one authoritative contract doc)."
            )
            misalignments.append(Item(kind="misalignment", title=title, sources=[src, other_src]))

    # Consolidated follow-ups across canonical artifacts.
    spec_manifest_lines = read_lines(spec_manifest_abs) or []
    impact_map_lines = read_lines(impact_map_abs) or []
    minimal_spec_lines = read_lines(minimal_spec_abs) or []
    ci_plan_lines = read_lines(ci_plan_abs) or []
    triage_lines = read_lines(triage_abs) or []

    items.extend(
        extract_numbered_items_typed(
            spec_manifest_rel,
            spec_manifest_lines,
            lambda h: h == "Follow-ups",
            re.compile(r"^\s*(\d+)\.\s+(.+?)\s*$"),
            kind_default="followup",
            dr_prefix_ok=True,
        )
    )
    items.extend(extract_impact_map_followups(impact_map_rel, impact_map_lines))
    items.extend(
        extract_numbered_items_typed(
            ci_plan_rel,
            ci_plan_lines,
            lambda h: h == "Follow-ups",
            re.compile(r"^\s*(\d+)\)\s+(.+?)\s*$"),
            kind_default="checkpoint",
            dr_prefix_ok=False,
        )
    )
    items.extend(
        extract_numbered_items_typed(
            minimal_spec_rel,
            minimal_spec_lines,
            lambda h: h in {
                "Follow-ups for full planning",
                "Follow-ups for downstream FSE planning and decomposition",
            },
            re.compile(r"^\s*(\d+)[\.\)]\s+(.+?)\s*$"),
            kind_default="followup",
            dr_prefix_ok=True,
        )
    )
    items.extend(
        extract_triage_bullets(
            triage_rel,
            triage_lines,
            "Risk and unknowns",
            "risk",
        )
    )
    items.extend(
        extract_triage_bullets(
            triage_rel,
            triage_lines,
            "Follow-ups",
            "followup",
        )
    )
    items.extend(extract_gate_items(triage_rel, triage_lines))

    # Include wrapper-detected misalignments as gates in the consolidated list.
    consolidated: list[Item] = []
    for mi in misalignments:
        consolidated.append(mi)

    # Deduplicate non-DR items by normalized title.
    merged: dict[str, Item] = {}
    dr_items: dict[str, Item] = {}

    for it in items:
        if should_skip_legacy_followup(it):
            continue
        if it.kind == "dr":
            dr_id = (it.title.split("—", 1)[0]).strip()
            if dr_id not in dr_items:
                dr_items[dr_id] = Item(kind="dr", title=it.title, sources=list(it.sources))
            else:
                dr_items[dr_id].sources.extend(it.sources)
            continue

        key = normalize_key(it.title)
        if key not in merged:
            merged[key] = Item(kind=it.kind, title=it.title, sources=list(it.sources))
        else:
            merged[key].sources.extend(it.sources)
            merged[key].kind = best_kind([merged[key].kind, it.kind])

    for it in merged.values():
        consolidated.append(it)

    # Also fold in gate items and misalignments that may already exist in merged by title.
    # (No-op; keys handle duplicates.)

    # Render output
    print("## Misalignment / follow-ups (wrapper-detected)")
    if not misalignments:
        print("- None detected")
    else:
        for mi in misalignments:
            srcs = ", ".join(s.fmt() for s in mi.sources)
            print(f"- {mi.title} (sources: {srcs})")
    print("")

    print("## Consolidated FSE pre-planning follow-ups (wrapper-compiled)")

    gates = [it for it in consolidated if it.kind in ("misalignment", "gate")]
    checkpoints = [it for it in consolidated if it.kind == "checkpoint"]
    risks = [it for it in consolidated if it.kind == "risk"]
    followups = [it for it in consolidated if it.kind == "followup"]

    def render_list(title: str, its: list[Item]) -> None:
        print(f"### {title}")
        if not its:
            print("- None")
            print("")
            return
        # Stable-ish ordering: by first source path then line.
        its_sorted = sorted(
            its,
            key=lambda x: (
                x.sources[0].path if x.sources else "",
                x.sources[0].line if x.sources else 0,
                x.title,
            ),
        )
        for it in its_sorted:
            srcs = ", ".join(s.fmt() for s in it.sources)
            print(f"- {it.title} (sources: {srcs})")
        print("")

    render_list("Gates / hard decisions", gates)

    print("### Decision Register required")
    if not dr_items:
        print("- None detected")
        print("")
    else:
        for dr_id in sorted(dr_items.keys()):
            it = dr_items[dr_id]
            srcs = ", ".join(s.fmt() for s in it.sources)
            print(f"- {it.title} (sources: {srcs})")
        print("")

    render_list("Checkpoint intent follow-ups", checkpoints)
    render_list("Risks + unknowns", risks)
    render_list("Other follow-ups", followups)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
