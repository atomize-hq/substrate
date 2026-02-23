import json
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_json(path: Path, obj: object) -> None:
    _write_text(path, json.dumps(obj, indent=2, sort_keys=True) + "\n")


def _impact_map_strict(create_tokens: list[str]) -> str:
    create_lines = "\n".join(f"- `{t}`" for t in create_tokens) if create_tokens else "- None"
    return (
        "# Impact Map Fixture\n\n"
        "## Touch set (explicit)\n\n"
        "### Create\n"
        f"{create_lines}\n\n"
        "### Edit\n"
        "- None\n\n"
        "### Deprecate\n"
        "- None\n\n"
        "### Delete\n"
        "- None\n"
    )


class TestPmLiftReport(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; report script requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift_report"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.report = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift_report.py"
        if not cls.report.is_file():
            raise unittest.SkipTest("pm_lift_report.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.report), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def test_intake_happy_path(self) -> None:
        fixture = self.tmp_root / "intake_valid.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "touch": {},\n'
            '  "contract": {},\n'
            '  "qa": {},\n'
            '  "docs": {},\n'
            '  "ops": {},\n'
            '  "risk": {},\n'
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)

        res = self._run(["--intake", str(fixture)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("== Work Lift (advisory) ==", res.stdout)
        self.assertIn("Context: intake", res.stdout)
        self.assertIn("Lift Score:", res.stdout)
        self.assertIn("Confidence:", res.stdout)

    def test_pack_strict_happy_path_expected_values(self) -> None:
        feature_dir = self.tmp_root / "pack_strict"
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 2}})
        _write_text(feature_dir / "impact_map.md", _impact_map_strict(["__pm_lift_test__/a.txt", "__pm_lift_test__/b.txt"]))

        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("Context: pack", res.stdout)
        self.assertIn("Lift Score: 6", res.stdout)
        self.assertIn("Estimated slices: 1", res.stdout)

    def test_pack_legacy_skip(self) -> None:
        feature_dir = self.tmp_root / "pack_legacy"
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 1}})
        _write_text(feature_dir / "impact_map.md", _impact_map_strict(["__pm_lift_test__/a.txt"]))

        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue(res.stdout.startswith("SKIP:"), msg=res.stdout)
        self.assertIn("meta.slice_spec_version < 2", res.stdout)

    def test_error_propagation_intake_missing_markers(self) -> None:
        fixture = self.tmp_root / "intake_missing_markers.md"
        _write_text(fixture, "# Missing markers\n")

        res = self._run(["--intake", str(fixture)])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("pm_lift failed", res.stderr)


if __name__ == "__main__":
    unittest.main()

