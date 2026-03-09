import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestValidateSliceSpecDocOnly(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; validator tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_slice_spec_doc_only"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_slice_spec_doc_only.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_slice_spec_doc_only.py not found at expected canonical path.")

    def _run(self, paths: list[Path]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.validator), "--paths", *[str(p) for p in paths]]
        return subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))

    def _spec_text(self, slice_id: str, *, ac_lines: list[str], extra: str = "") -> str:
        return (
            f"# {slice_id} slice spec fixture\n\n"
            "## Behavior delta (single)\n"
            "- Existing: fixture\n"
            "- New: fixture\n"
            "- Why: fixture\n\n"
            "## Scope\n"
            "- fixture\n\n"
            "## Behavior (authoritative)\n"
            "### Fixture\n"
            "Text.\n\n"
            "## Acceptance criteria\n"
            + "\n".join(ac_lines)
            + "\n\n"
            "## Out of scope\n"
            "- fixture\n"
            + (("\n" + extra) if extra else "")
        )

    def test_fail_acceptance_criteria_over_8(self) -> None:
        slice_id = "WDAP1"
        ac_lines = [f"- AC-{slice_id}-{i:02d}: fixture" for i in range(1, 10)]
        spec_path = self.tmp_root / "slices" / slice_id / f"{slice_id}-spec.md"
        _write_text(spec_path, self._spec_text(slice_id, ac_lines=ac_lines))

        res = self._run([spec_path])
        self.assertEqual(res.returncode, 1)
        self.assertIn("acceptance criteria count is 9; must be 1..8", res.stderr)

    def test_fail_missing_required_header(self) -> None:
        slice_id = "WDAP1"
        spec_path = self.tmp_root / "slices" / slice_id / f"{slice_id}-spec.md"
        _write_text(
            spec_path,
            (
                f"# {slice_id} fixture\n\n"
                "## Behavior delta (single)\n"
                "- Existing: fixture\n"
                "- New: fixture\n"
                "- Why: fixture\n\n"
                "## Scope\n"
                "- fixture\n\n"
                "## Behavior (authoritative)\n"
                "### Fixture\n"
                "Text.\n\n"
                "## Acceptance criteria\n"
                f"- AC-{slice_id}-01: fixture\n\n"
                # Intentionally omit "## Out of scope"
            ),
        )
        res = self._run([spec_path])
        self.assertEqual(res.returncode, 1)
        self.assertIn("missing header: '## Out of scope'", res.stderr)

    def test_fail_forbidden_placeholder(self) -> None:
        slice_id = "WDAP1"
        spec_path = self.tmp_root / "slices" / slice_id / f"{slice_id}-spec.md"
        _write_text(
            spec_path,
            self._spec_text(
                slice_id,
                ac_lines=[f"- AC-{slice_id}-01: fixture"],
                extra="TBD: placeholder must be rejected\n",
            ),
        )
        res = self._run([spec_path])
        self.assertEqual(res.returncode, 1)
        self.assertIn("forbidden placeholder", res.stderr)

    def test_pass_minimal_valid_spec(self) -> None:
        slice_id = "WDAP1"
        spec_path = self.tmp_root / "slices" / slice_id / f"{slice_id}-spec.md"
        _write_text(
            spec_path,
            self._spec_text(
                slice_id,
                ac_lines=[
                    f"- AC-{slice_id}-01: fixture one",
                    f"- AC-{slice_id}-02: fixture two",
                ],
            ),
        )
        res = self._run([spec_path])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "")

