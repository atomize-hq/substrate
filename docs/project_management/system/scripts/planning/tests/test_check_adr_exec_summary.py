import os
import sys
import unittest
from pathlib import Path


PLANNING_DIR = Path(__file__).resolve().parents[1]
if str(PLANNING_DIR) not in sys.path:
    sys.path.insert(0, str(PLANNING_DIR))

import check_adr_exec_summary as mod  # noqa: E402


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _make_adr_text(*, standards_lines: str) -> str:
    """
    Create an ADR with a valid Executive Summary section and a correct ADR_BODY_SHA256.
    """
    placeholder = "0" * 64
    text = (
        "# ADR Test\n\n"
        "## Standards\n"
        f"{standards_lines}\n"
        "\n"
        "## Executive Summary (Operator)\n"
        "\n"
        "- Existing: foo\n"
        "- New: bar\n"
        "- Why: baz\n"
        "\n"
        f"ADR_BODY_SHA256: {placeholder}\n"
        "\n"
        "## Details\n"
        "\n"
        "Some body content.\n"
    )

    section = mod._find_exec_section(text)
    assert section is not None
    body_hash = mod._adr_body_hash(text, section)
    return text.replace(f"ADR_BODY_SHA256: {placeholder}", f"ADR_BODY_SHA256: {body_hash}")


class TestCheckAdrExecSummary(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        # Keep all scratch files inside the repo so git repo-root discovery works.
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; repo root discovery required for these tests.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "check_adr_exec_summary"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

    def test_fails_on_legacy_standards_refs(self) -> None:
        adr_path = self.tmp_root / "ADR-9991-legacy-refs.md"
        legacy = (
            "- `docs/project_management/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`\n"
            "- `docs/project_management/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`\n"
        )
        _write_text(adr_path, _make_adr_text(standards_lines=legacy))
        rc = mod.check_adr(path=adr_path, fix=False)
        self.assertEqual(rc, 1)

    def test_fix_rewrites_legacy_refs_and_updates_hash(self) -> None:
        adr_path = self.tmp_root / "ADR-9992-fix-legacy-refs.md"
        legacy = (
            "- `docs/project_management/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`\n"
            "- `docs/project_management/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`\n"
            "- `docs/project_management/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`\n"
            "- Exit code taxonomy: `docs/project_management/standards/shared/EXIT_CODE_TAXONOMY.md`\n"
        )
        _write_text(adr_path, _make_adr_text(standards_lines=legacy))

        rc = mod.check_adr(path=adr_path, fix=True)
        self.assertEqual(rc, 0)

        updated = adr_path.read_text(encoding="utf-8")
        self.assertNotIn("docs/project_management/standards/", updated)
        self.assertIn(
            "docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md", updated
        )
        self.assertIn("docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md", updated)
        self.assertIn(
            "docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md", updated
        )
        self.assertIn("docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md", updated)

        section = mod._find_exec_section(updated)
        self.assertIsNotNone(section)
        body_hash = mod._adr_body_hash(updated, section)  # type: ignore[arg-type]
        self.assertIn(f"ADR_BODY_SHA256: {body_hash}", updated)

    def test_fix_is_atomic_when_unresolvable(self) -> None:
        adr_path = self.tmp_root / "ADR-9993-unresolvable.md"
        legacy = "- `docs/project_management/standards/DOES_NOT_EXIST.md`\n"
        original = _make_adr_text(standards_lines=legacy)
        _write_text(adr_path, original)

        rc = mod.check_adr(path=adr_path, fix=True)
        self.assertEqual(rc, 1)

        after = adr_path.read_text(encoding="utf-8")
        self.assertEqual(after, original)

    def test_passes_with_canonical_paths(self) -> None:
        adr_path = self.tmp_root / "ADR-9994-canonical.md"
        canonical = (
            "- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`\n"
            "- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`\n"
            "- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`\n"
        )
        _write_text(adr_path, _make_adr_text(standards_lines=canonical))
        rc = mod.check_adr(path=adr_path, fix=False)
        self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main()
